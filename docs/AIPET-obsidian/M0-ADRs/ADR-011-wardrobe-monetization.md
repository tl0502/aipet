# ADR-011: 装扮付费 schema 范式

- **状态**: Accepted
- **决策日期**: 2026-05-01
- **Owner**: M0 决策周(产品 + 工程)
- **Reviewers**: M0 决策周复审通过
- **目标完成**: M0 决策周内(W0)
- **影响范围**: O 装扮系统 / DB schema / P2-R3 商店上线

## 背景

PRD v0.6 §2.17(冻结决策 17)锁定:**装扮免费空投 + 付费 schema 预留;MVP 不开商店,P2-R3 再开**。

架构 v0.3 §2.3 给出了 `AccessoryMeta.tier / unlock` 字段范式:
- `tier: 'free' | 'paid'`
- `unlock: 'always' | 'milestone:xxx' | 'date_range:[s,e]' | 'purchase:sku'`

需要 M0 内完整定义这个 schema,让 MVP 的免费空投与未来的付费完全解耦,**避免 P2-R3 时不得不破坏性升级 DB schema**。

## 关键决策点

1. `unlock` 字段是字符串(灵活但弱类型)还是结构化对象?
2. `purchase` 路径预留哪些字段?(SKU / 价格 / 货币 / 平台?)
3. 内置免费配饰是否在 inventory 表预解锁,还是按需解锁?
4. 用户上传自定义配饰的 tier 默认是什么?
5. 跨设备付费(如果 P2-R3 上云)的迁移路径?

## 备选方案

### 选项 A:**结构化对象 schema**(推荐,DB 用 JSON 列存储)

```typescript
type Accessory = {
  id: string;
  name: string;
  category: 'hat' | 'scarf' | 'glasses' | 'ear' | 'tail_accessory' | 'background_skin' | ...;
  tier: 'free' | 'paid';
  unlock: UnlockSpec;
  asset_path: string;
  anchor: AccessoryAnchor;  // 锚点信息
  metadata: AccessoryMetadata;  // 可选元数据:作者、创作日期、tags
};

type UnlockSpec =
  | { kind: 'always' }
  | { kind: 'milestone', milestone_id: string }
  | { kind: 'date_range', start: string, end: string, year_recurring?: boolean }
  | { kind: 'purchase', sku: string, price_cny?: number, price_usd?: number, platform?: 'web'|'app_store'|'wechat' }
  | { kind: 'gift', code?: string, expires_at?: string }
  | { kind: 'user_upload' };  // 用户上传的自定义配饰
```

**优点**
- 强类型,前端 / 后端共享
- 易扩展(新增 unlock kind 不影响老 kind)
- DB 用 SQLite 的 JSON 列存储 + 应用层 schema 校验

**缺点**
- DB 查询时 JSON 提取略麻烦(SQLite JSON1 扩展支持)

### 选项 B:**字符串 + 关键字解析**

```typescript
unlock: string;  // e.g. "always" / "milestone:first_launch_30d" / "date_range:2026-02-10..2026-02-17:recur" / "purchase:sku_red_scarf:9.9:cny"
```

**优点**
- 简单
- 易调试(肉眼可读)

**缺点**
- 弱类型,容易写错
- 解析需要 regex,扩展时 fragility 高

### 选项 C:**多列展开**

```sql
CREATE TABLE accessories_meta (
  id TEXT PRIMARY KEY,
  ...
  unlock_kind TEXT,
  unlock_milestone_id TEXT,
  unlock_date_start TEXT,
  unlock_date_end TEXT,
  unlock_sku TEXT,
  unlock_price_cny INTEGER,
  ...
);
```

**优点**
- DB 查询直接(can index)

**缺点**
- 字段稀疏(大多数行只用其中 1-2 列)
- 新增 kind 需要 ALTER TABLE

## 我的倾向

**🌟 倾向选项 A(结构化对象 + JSON 列)**:
1. 类型安全 + 扩展性最好。
2. SQLite JSON1 性能足够(MVP 内置配饰不超过 50 个,日均查询 < 10 次)。
3. 与 .soul.md 的 frontmatter 风格一致(用户接受度高)。

### 内置配饰预定义示例(M0 输出)

```json
[
  {
    "id": "basic_scarf_red",
    "name": "基础红围巾",
    "category": "scarf",
    "tier": "free",
    "unlock": { "kind": "always" },
    "asset_path": "accessories/scarf_red.png",
    "anchor": { "x": 0.5, "y": 0.7, "scale": 1.0, "z_index": 1, "layer_target": "neck_slot" }
  },
  {
    "id": "lunar_new_year_2026_red_scarf",
    "name": "春节红围巾(2026)",
    "category": "scarf",
    "tier": "free",
    "unlock": {
      "kind": "date_range",
      "start": "2026-02-08",
      "end": "2026-02-22",
      "year_recurring": true  // 每年重复
    },
    "asset_path": "accessories/lny_red_scarf.png",
    "anchor": { ... }
  },
  {
    "id": "milestone_30d_glasses",
    "name": "30 天纪念眼镜",
    "category": "glasses",
    "tier": "free",
    "unlock": { "kind": "milestone", "milestone_id": "first_launch_30d" },
    "asset_path": "accessories/anniversary_glasses.png",
    "anchor": { ... }
  },
  {
    "id": "fancy_hat_v1",
    "name": "豪华帽子",
    "category": "hat",
    "tier": "paid",
    "unlock": {
      "kind": "purchase",
      "sku": "fancy_hat_v1",
      "price_cny": 9.9,
      "price_usd": 1.99,
      "platform": "web"
    },
    "asset_path": "accessories/fancy_hat.png",
    "anchor": { ... }
  }
]
```

### MVP 期 paid 配饰的处理

- `WardrobeService.list_inventory()` **过滤掉** `tier='paid'` 的所有项。
- 但 schema 已经在 DB 里准备好,P2-R3 上线商店时仅需:
  1. 解除过滤
  2. 实现支付链路
  3. 实现"已购"状态的 unlock 检查

### 用户上传自定义配饰

- `tier='free'`、`unlock.kind='user_upload'`
- 仅本地有效,不能跨设备同步(除非 P1 云同步打开)
- 不能直接进入"商店"流程(用户上传 ≠ 自定义 SKU)

## 决策

**选定:选项 A — 结构化对象 schema + JSON 列存储**。

`Accessory` 与 `UnlockSpec` 的 TypeScript / Rust 类型定义见"我的倾向"段;`UnlockSpec` 支持 5 种 kind:**always / milestone / date_range / purchase / gift / user_upload**(实质 6 种,后两种实施期添加)。

DB 中 `accessories_inventory` 表用 SQLite JSON 列(`metadata JSON`)存 `unlock` / `anchor` / 其他元数据;`tier='paid'` 在 MVP 期由 `WardrobeService.list_inventory()` **强制过滤**,前端永远看不到付费配饰。

P2-R3 商店上线时仅需:① 解除 list_inventory 的 paid 过滤 ② 实现支付链路 ③ 实现"已购"unlock 检查。**schema 不破坏性变更**。

**M0 末交付内置配饰清单**:8 件配饰(帽子 ×3、围巾 ×2、眼镜 ×2、其他点缀 ×3)+ 4 套节气皮肤(春节红围巾、圣诞帽、生日帽、情人节信封)。

## 后果

### 正面
- 类型安全 + 扩展性最好;新增 unlock kind(如 raffle / event_code)零破坏性。
- SQLite JSON1 性能足够(MVP 内置配饰 ≤ 50 个,日均查询 < 10 次)。
- 与 .soul.md frontmatter 风格一致,用户/开发者接受度高。

### 负面
- DB 查询时 JSON1 提取需要 SQL 表达式(`json_extract(metadata, '$.unlock.kind')`),比纯列 SELECT 略重。
- 缓解:对常用查询(`tier`、`unlock.kind`)建 generated column + index,性能匹配纯列方案。

## 实施动作

- [ ] 锁定 `Accessory` 与 `UnlockSpec` 的 TypeScript / Rust 类型定义
- [ ] 在 SQLite `accessories_inventory` 表增加 `metadata JSON` 列
- [ ] 输出内置配饰清单(8 件配饰 + 4 套节气,JSON manifest)
- [ ] CI 校验:`tier='paid'` 的配饰不能在 MVP 启动时被 list_inventory 返回
- [ ] M4 实施时 WardrobeService 严格按 schema 工作
- [ ] P2-R3 设计商店时复用本 schema 不需要破坏性变更

## 引用

- PRD v0.6 §2.17(冻结决策 17)
- PRD v0.6 §7.15(模块 O)
- 架构 v0.3 §2.3(WardrobeService)
- 架构 v0.3 §4(SQLite Schema)

## 复审签字

| Reviewer | 角色 | 同意? | 备注 |
|---|---|---|---|
| M0 决策周 | 产品 | ☑ | 与 PRD §2.M0 决策 31 一致 |
| M0 决策周 | 工程 | ☑ | UnlockSpec 类型定义 + paid 过滤 CI 校验 M4 落地 |
| M0 决策周 | 商业化 | — | MVP 期不开商店,P2-R3 启动前再签 |

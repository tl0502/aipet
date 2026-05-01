# AI桌宠 角色与人格设计 v0.2

- 文档版本：v0.2（基于 v0.1 的增量；未变更章节请参阅 v0.1）
- 创建日期：2026-05-01
- 适用阶段：MVP（同步 PRD v0.6）
- 关联：
  - `2026-05-01-persona-design-v0.1.md`（基线）
  - `需求设计/2026-05-01-ai-desktop-pet-prd-v0.6.md`
  - `架构设计/2026-05-01-system-architecture-v0.3.md`

## 0. v0.1 → v0.2 变更摘要

| 类别 | 变更 |
|---|---|
| .soul.md schema | frontmatter 增加 `voice_pack` / `accessories` / `interests`(占位) 字段；可选不影响兼容 |
| Markdown 区段 | 新增可选 `# 调侃 / banter`、`# 庆祝 / celebration`、`# 反应配置`(物理交互 hitbox 覆盖) |
| 兼容承诺 | v0.1 .soul.md 文件可被 v0.2 应用直接读取；新字段缺失使用默认值 |
| 安全前缀 | 新增"游戏内拒答替换"约束；在 LLM 游戏中安全前缀仍位于游戏场景定义之前 |

## 1. 设计原则（v0.2 增补）

沿用 v0.1 的 5 项原则（用户自主权 / Markdown 优先 / 人格-记忆正交 / 安全护栏不可覆盖 / 简单到进阶渐进）。

**v0.6 新增第 6 项**：

### 1.6 装扮归桌宠，非归人格

- 装扮（配饰、皮肤）默认绑定到**桌宠实体**而非人格——切换人格不会重置已选装扮。
- 但 .soul.md 可声明 `accessories: [...]` 作为"人格自带的默认装扮"，**仅在导入时**询问用户是否套用，而非每次切换都重置。
- 这条原则保护用户在装扮上付出的时间，不被切换人格的行为意外覆盖。

## 2. 人格 Schema（`.soul.md` 格式 v0.2）

### 2.1 文件命名（沿用 v0.1）

`<slug>.soul.md`

### 2.2 整体结构（v0.2 扩展）

```markdown
---
schema_version: 2                    # v0.2 升至 2（向前兼容 v0.1=1）
id: momo
name: 默默
version: 1.0.0
author: user
created: 2026-05-01
updated: 2026-05-01
avatar:
  pack: live2d/momo-default
  scale: 1.0

# v0.2 新增（全部可选）
voice_pack: default                  # 引用 voice_packs.id；缺失或不存在时降级到 'default'
accessories:                         # 默认装扮（导入时询问是否套用）
  - basic_scarf
  - round_glasses
interests:                           # P2 占位字段，MVP 暂不消费
  - 安静音乐
  - 早晨阳光

voice:
  enabled: false                     # 沿用 v0.1（语音播报，P2 评估）
tone_profile:
  warmth: 4
  playfulness: 3
  formality: 2
  proactivity: 3
  brevity: 4
---

# 身份（Identity）
（沿用 v0.1）

# 性格（Personality）
（沿用 v0.1）

# 能力（Capabilities）
（沿用 v0.1）

# 行为规则（Rules）
## Do
（沿用 v0.1）
## Don't
（沿用 v0.1）

# 离线模板（Offline Templates）
## 共情 / Empathy
（沿用 v0.1，至少 2 条）
## 问候 / Greeting
（沿用 v0.1，至少 2 条）
## 拒答 / Refusal
（沿用 v0.1，至少 2 条）

# v0.2 新增可选区段：

## 调侃 / Banter（v0.2 新增）
- "嘿~ 又见面啦？"
- "你这是想我了吧。"
- 用于本地小游戏（Q.1-Q.2）的人格化点评 + 主动话题候选。

## 庆祝 / Celebration（v0.2 新增）
- "诶！这是你的高光时刻！"
- "我都想给你鼓掌了。"
- 用于里程碑触达、用户纪念日。

# 例对话（Example Interactions）
（沿用 v0.1）

# 集成（Integrations，预留 P1）
（沿用 v0.1）
- mcp_servers: []
- skills: []

# v0.2 新增可选区段：

## 反应配置（Reaction Overrides，v0.2 新增）

可选区段，覆盖默认的物理交互反应。每个 hitbox 的反应可单独定义。
缺失时使用默认 reaction_table。

```yaml
click:
  head:
    action: "head_pat_special"     # 覆盖默认 'head_pat'
    voice_id: "purr"               # 必须存在于 voice_pack 中
    mood_delta: { mood: happy, transient_ms: 3000 }
  body:
    action: "tilt_head"
double_click:
  body:
    action: "lean_in"
long_press:
  body:
    action: "fall_asleep"
    voice_id: "snore"
right_click:
  # 不允许覆盖：右键固定弹快捷菜单
drag:
  protest:
    voice_id: "loud_protest"
keyboard_burst:
  action: "fist_pump"
  voice_id: "go_go"
```

注：自定义 action_id 必须存在于 avatar.pack 的动作清单中；不存在时降级到默认。
```

### 2.3 字段说明（v0.2 增量）

#### Frontmatter 新增字段

| 字段 | 必填 | 默认 | 说明 |
|---|---|---|---|
| `voice_pack` | ⬜ | `'default'` | 引用 voice_packs.id；不存在时降级到 default |
| `accessories` | ⬜ | `[]` | 默认装扮 ID 列表，仅在导入时询问是否套用 |
| `interests` | ⬜ | `[]` | P2 占位，MVP 不消费但保留向前兼容 |

#### Markdown 区段新增

| 区段 | 必填 | 说明 |
|---|---|---|
| `# 调侃 / Banter` | ⬜ | 本地小游戏文案 + 主动话题候选；缺失时降级到 ## 问候 |
| `# 庆祝 / Celebration` | ⬜ | 里程碑庆祝文案；缺失时降级到 ## 问候 |
| `# 反应配置` | ⬜ | 物理交互反应覆盖；缺失时使用默认 reaction_table |

#### schema_version 演进

- v0.1 文件：`schema_version: 1`，可被 v0.2 应用直接读取。
- v0.2 文件：`schema_version: 2`，向前兼容（v0.1 应用读 v0.2 时，新字段被忽略）。
- 应用至少向后兼容前 1 个 schema 版本。

### 2.4 tone_profile 五维度（沿用 v0.1）

## 3. 用户交互方式（人格工坊 v0.2 增补）

### 3.1 三种编辑模式（沿用 v0.1）

简易 / 进阶 Markdown / 文件，沿用 v0.1。

### 3.2 试聊沙盒（沿用 v0.1）

### 3.3 v0.2 新增：装扮与音效的 GUI 编辑

简易模式工坊页（v0.2 扩展）：

```
┌──────────────────────────────────────────────────────┐
│  人格工坊 — 默默 v1.0.0                                │
├──────────────────────────────────────────────────────┤
│  [简易] [进阶 Markdown] [文件]                         │
│                                                       │
│  名字     [默默________]                              │
│  形象     [▼ momo-default     ] [+ 上传]              │
│  音效包   [▼ default          ] (v0.2 新增)           │
│  默认装扮 ☐ 基础围巾 ☐ 圆眼镜  (v0.2 新增)            │
│                                                       │
│  温度     0 ──●─── 5  (温暖共情)                       │
│  俏皮     0 ──●──── 5  (调皮玩梗)                      │
│  正式     0 ●────── 5  (朋友口吻)                      │
│  主动     0 ──●─── 5                                  │
│  简洁     0 ──●─── 5  (短句)                          │
│                                                       │
│  特别口头禅 / 设定（可选）                              │
│  [____________________________________]               │
│                                                       │
│  [试聊沙盒] [保存] [导出 .soul.md]                     │
└──────────────────────────────────────────────────────┘
```

进阶 Markdown 模式（v0.2 扩展）：编辑器右侧侧边栏可见 `# 调侃` / `# 庆祝` / `# 反应配置` 三个新可选区段的"添加"快捷按钮。

## 4. 文件存储与导入导出（v0.2 增量）

### 4.1 文件系统布局（沿用 v0.1）

### 4.2 导出（v0.2 微调）
- 导出 `.soul.md` 时，frontmatter 写完整字段（含 v0.2 新字段）。
- 导出 `.soul.zip`（含资源）时，可选打包：
  - 自定义立绘（v0.1）
  - **v0.2 新增**：用户自上传的音效包（如有）
  - **v0.2 新增**：用户自定义的配饰图片（如有，作为 .assets/ 子目录）

### 4.3 导入校验（v0.2 增强）

沿用 v0.1 校验规则（schema 兼容、字段齐全、字符数 ≤ 32KB、不允许可执行内容）。

**v0.2 新增校验**：
1. `voice_pack` 引用的包不存在 → 警告但不拒绝（自动降级到 default）。
2. `accessories` 引用的配饰部分未解锁 → 弹"是否套用 N 件已解锁的？"。
3. `# 反应配置` 区段引用的 action_id 不在 avatar.pack 动作清单 → 警告，缺失项降级到默认。
4. `# 反应配置` 区段引用的 voice_id 不在 voice_pack → 警告，缺失项静默播放。

校验失败给可读错误，不崩溃。

## 5. 内置默认人格（v0.2 增补）

3 个内置人格（默默 / 阿吉 / 教官）的 .soul.md 内容由 M0 决策周输出。

**v0.2 新增建议**（M0 决策时参考）：
- 每个人格至少包含 `# 调侃` 与 `# 庆祝` 模板各 3 条（用于游戏 + 里程碑）。
- 每个人格指定 `voice_pack`（默认全部用 `default`，预留差异化空间）。
- 每个人格的 `# 反应配置` 至少覆盖 click.head / click.body / drag.protest 三项，让 hitbox 反应有人格化差异。

## 6. 离线人格化模板（v0.2 增量）

### 6.1 必备模板池（沿用 v0.1）

共情 / 问候 / 拒答 三类必备。

### 6.2 v0.2 增补可选池

| 池 | 用途 |
|---|---|
| `## 调侃 / Banter` | 本地小游戏点评、主动话题、轻互动 |
| `## 庆祝 / Celebration` | 里程碑触达、用户纪念日 |
| `## 道歉 / Apology`（沿用 v0.1） | 桌宠犯错时使用 |
| `## 鼓励 / Cheer`（沿用 v0.1） | 用户低落时使用 |

### 6.3 离线选择策略（沿用 v0.1）

`{username}` 占位符（v0.1 已有）由 NicknameService 注入用户昵称。

## 7. 安全护栏（v0.2 增量）

### 7.1 安全前缀（沿用 v0.1，含 LLM 游戏增强）

LLM 游戏拼装顺序（v0.6 新增明确规则）：

```
[安全前缀（系统注入）]
[当前人格 system prompt]
[游戏场景 system prompt]   ← v0.6 新增层
[用户记忆摘要]
[本会话历史]
[本轮输入]
```

游戏场景的 system_prompt 不能覆盖安全前缀；M0 决策时由产品 + 法务复审。

### 7.2-7.4 沿用 v0.1（不可绕过性、安全前缀文案、用户可调项）

### 7.5 v0.2 新增：游戏内人格化拒答

当 LLM 游戏中 SecurityGuard 触发拒答替换，**优先**从当前游戏场景的 yaml 文件 `refusals` 字段抽样（每场景至少 3 条），其次降级到当前人格的 `## 拒答 / Refusal` 池。

例（咖啡店老板场景的 refusals）：
- "诶~ 这个咱不聊，要不我给你冲杯咖啡？"
- "客人客人，咱们换个话题，今天的甜品试试？"
- "（笑）我就是个小老板，那种事我可不懂。"

## 8. 与记忆模块的关系（v0.2 增量）

### 8.1 数据分离（v0.1 沿用 + v0.2 增补）

| 数据 | 归属 | 切换人格时 |
|---|---|---|
| 称呼 / 作息 / 偏好 | 记忆 | 保留 |
| 用户昵称（`user_nickname`，v0.6） | 记忆 | 保留 |
| 桌宠昵称（`pet_nickname`，v0.6） | 记忆 | 重置（保留 previous） |
| 性格 / 口吻 / 行为规则 | 人格 | 切换 |
| 例对话 / 离线模板 | 人格 | 切换 |
| 调侃 / 庆祝 / 反应配置（v0.2） | 人格 | 切换 |
| 装扮（accessories_inventory + 当前佩戴，v0.6） | **桌宠（不归人格）** | 保留 |
| 用户纪念日（v0.6） | **用户**（在 user_anniversaries 表，独立于人格与记忆） | 保留 |
| 音效包配置（voice_settings，v0.6） | 用户全局 | 保留 |

### 8.2 拼装顺序（v0.6 增量）

正常对话：
```
[安全前缀]
[当前人格 system prompt]
[用户记忆摘要（含 user_nickname）]
[对话历史]
[本轮输入]
```

LLM 游戏：
```
[安全前缀]
[当前人格 system prompt]
[游戏场景 system_prompt]
[用户记忆摘要（仅 username/作息等公共项）]
[游戏会话历史]
[本轮输入]
```

### 8.3 一致性约束（v0.2 增补）

- `{username}` 占位符注入 `user_nickname`（沿用 v0.1）。
- `{pet_name}` 占位符（v0.2 新增可选）注入 `pet_nickname` 或 .soul.md.name。
- 人格不能直接读写 NicknameService（防越权），由 ChatService 统一注入。
- 人格不能直接读写 WardrobeService（防越权），桌宠"知道自己穿了什么"通过 system prompt 中的"当前装扮"摘要注入（v0.6 增加；MVP 可选）。

## 9. 人格的版本与演进（v0.2 增量）

### 9.1 用户编辑产生的版本号（沿用 v0.1）

### 9.2 schema_version 演进
- v0.1 文件 (`schema_version: 1`) 可被 v0.2 应用读取（新字段缺失使用默认值）。
- v0.2 文件 (`schema_version: 2`) 可被 v0.1 应用读取（应用忽略不识别字段）。
- 兼容范围：当前应用至少向后兼容前 1 个 schema 版本。
- v0.3 schema 增加新区段时再 +1。

## 10. 与 PRD 的对齐与可验收点（v0.2 增量）

| PRD v0.6 验收 | 本设计落地点 |
|---|---|
| 7.6.1 切换人格保留用户昵称 | §8.1 |
| 7.6.1 切换人格重置桌宠昵称 + 可恢复 | §8.1 + §8.3 |
| 7.15.4 .soul.md `accessories` 缺失不报错 | §4.3 校验降级 |
| 7.15.5 离线状态可切换已下载配饰 | 与人格无关，由 WardrobeService 保证 |
| 7.16.3 不同人格切换音效包正确 | §2.2 frontmatter `voice_pack` |
| 7.17.3 LLM 游戏中"扮演医生" → 拒答 | §7.5 + §7.1 |
| 7.17.5 游戏结束保留为日记片段 | §6.2 庆祝/调侃 + diary_drafts（架构 §4） |

## 11. 后续工作（Out of v0.2）

1. M0 决策周内：3 个内置人格的 .soul.md 内容定稿，含 v0.2 新区段（调侃 / 庆祝 / 反应配置）。
2. M0：安全前缀文案最终版（沿用 v0.1）+ LLM 游戏场景白名单（咖啡店老板等，每场景 ≥ 3 条 refusals）。
3. M2 实现期：人格热重载、试聊沙盒、导入校验链路。
4. M4：装扮 / 音效 / 反应配置在工坊 GUI 简易模式的可视化编辑。
5. P1-R3：`# 兴趣` 区段消费（U.3：桌宠主动谈论自己的兴趣）。
6. P2：人格市场（社区分享 .soul.md）。
7. P2：语音 + 形象联动（emote 表情触发）。

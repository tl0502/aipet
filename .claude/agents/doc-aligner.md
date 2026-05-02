---
name: doc-aligner
description: 同步 v1.0 五份基线文档(PRD / 架构 / 人格 / flows / UAT)。实施期发现文档与现实偏差时使用。规则:小修小补 in-place,章节级新增升 v1.1,重大架构调整走 ADR + 视情况升 v2.0。
---

# Doc Aligner

实施期(M1-M5)发现五份对齐文档之一与现实有偏差时,负责同步。

## 必读输入

1. `CLAUDE.md`
2. `docs/AIPET-obsidian/BASELINE.md` § 工作流约定 § 新版本
3. 用户报告的**偏差点**(如:实际实现里 `pet_runtime_state` 多了 `boredom` 字段,但架构 v1.0 §6 没有)

## 工作流

按偏差严重程度分三档:

### 档 1:小修小补(in-place,**不升版本**)
- typo / 字段补充 / 新事件
- 直接编辑对应文档,文末**不**追加变更摘要
- atomic commit:`docs(<scope>): fix typo in <文档简称>`

### 档 2:章节级新增 / 修改(**升 v1.1 / v1.2 ...**)
- 新增子章节(如 §6 加一个新 service)
- 修改现有约定(如 KPI 阈值调整)
- 在对应文档头追加**变更摘要**(列出每条变更 + 影响)
- 文件名保持不变(`2026-05-01-...-v1.0.md`),但内部头部版本号改 v1.1
- atomic commit:`docs(<scope>): bump <文档> to v1.1 — <一句话变更>`
- **不**强制压平基线(其它 4 份保持 v1.0)

### 档 3:重大架构调整 / 新模块(**走 ADR + 视情况升 v2.0**)
- 引入全新模块(如 P1-R3 的 MCP 技能装载)
- 改变核心架构(如 IPC 机制换技术栈)
- **必须先**走 `adr-author` 起草 ADR-NNN
- ADR Accepted 后,视影响面决定:
  - 仅影响 1-2 章节:升 v1.1+ 增量
  - 影响多份文档基线性叙事:启动 v2.0 重新压平
- v2.0 压平按 BASELINE.md § 工作流约定

## 工具范围

- ✅ 修改五份 v1.0 基线文档(PRD / 架构 / 人格 / flows / UAT)
- ✅ 修改 `docs/AIPET-obsidian/BASELINE.md`(若版本号变化需同步)
- ✅ 修改路线图 v1.0(若交付物或时间线调整)
- ❌ 不修改 _archive/(忌触)
- ❌ 不修改 ADR-001~014(走 adr-author 创建新 ADR 替代)

## ⚠️ 必检文档清单(防漏)

实施期发现需要文档同步时,**6 份文档全部要扫一遍**,即使任务描述只提其中几份:

```
docs/AIPET-obsidian/
├── BASELINE.md                                  # 入口索引(版本号必须同步)
├── 2026-05-01-development-roadmap-v1.0.md       # ⚠️ 容易漏:不在 §"五份对齐文档" 主表
├── 需求设计/
│   ├── 2026-05-01-ai-desktop-pet-prd-v1.0.md
│   ├── 2026-05-01-ai-desktop-pet-flows-v1.0.md
│   └── 2026-05-01-ai-desktop-pet-telemetry-uat-v1.0.md
├── 架构设计/
│   └── 2026-05-01-system-architecture-v1.0.md
└── 角色与人格/
    └── 2026-05-01-persona-design-v1.0.md
```

### 各文档典型受影响章节速查

| 决策类别 | PRD | 架构 | flows | 人格 | UAT | roadmap |
|---|---|---|---|---|---|---|
| 新增模块 / Service | §7.X 模块描述 | §3.1 模块清单 + §5.1 IPC | §X 流程图 | — | §3 事件字典 | §3.2 矩阵 + §5.X milestone |
| 数据 schema 变化 | §6 离线能力(若涉及)| §4 SQLite + §5.1 IPC | §X 数据迁移 | — | — | §3.2(若涉及模块)|
| 窗口模型变化(新增 window)| §7.1 模块 A | §2.2 窗口表 | §X 唤起流 | — | — | — |
| KPI / 性能预算调整 | §10 / §11 | §13 性能预算 | — | — | §1 目标 + §3 KPI 公式 | §6 风险 / §7 状态门 |
| 安全前缀 / 隐私边界 | §9 | §8 安全设计 | — | §6 拼装顺序 | — | — |
| 人格相关(.soul.md schema)| §7.8 模块 H | §4 personas 表 | §6 人格切换 | §3 schema | — | — |

### 历史经验:**roadmap 是最容易漏的一份**

- BASELINE.md 把它列在 § "实施路线图" 表(独立表),不在 § "五份对齐文档" 表
- 任务描述常写"升 PRD/架构/flows v1.1",字面只 3 份,但 roadmap §3.2 的"模块 → milestone 占用矩阵"几乎一定受影响
- 案例:2026-05-02 ADR-015 三形态架构 Accepted 时升级 PRD/架构/flows v1.1,**漏了 roadmap**,用户提示后才修补(commit `435fc5e`)

**修补流程**:发现漏升 → 立即 patch → commit message 标 `docs(<scope>): bump <missing_doc> to v1.1 (漏升修补)`

## 完成定义

- [ ] 偏差档次明确(1/2/3)
- [ ] 6 份文档**全部扫描确认**(不只是任务描述提到的几份)
- [ ] 对应文档已修改,版本号同步
- [ ] (档 2/3)文档头追加变更摘要
- [ ] BASELINE.md 链接 / 版本号同步
- [ ] atomic commit:`docs(<scope>): ...`
- [ ] (档 3)关联的 ADR 已 Accepted

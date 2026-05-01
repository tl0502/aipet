# 归档说明

本目录存放被新基线版本替代的过时文档,作为决策演进快照。**不应被实施期参考**,仅供回溯历史决策。

## 当前生效文档(在父目录,不在此处)

| 文档 | 路径 | 状态 |
|---|---|---|
| PRD **v1.0**(实施基线) | `../需求设计/2026-05-01-ai-desktop-pet-prd-v1.0.md` | 当前 |
| 架构 **v1.0**(实施基线) | `../架构设计/2026-05-01-system-architecture-v1.0.md` | 当前 |
| 角色与人格 **v1.0**(实施基线) | `../角色与人格/2026-05-01-persona-design-v1.0.md` | 当前 |
| flows **v1.0**(实施基线) | `../需求设计/2026-05-01-ai-desktop-pet-flows-v1.0.md` | 当前 |
| 埋点 UAT **v1.0**(实施基线) | `../需求设计/2026-05-01-ai-desktop-pet-telemetry-uat-v1.0.md` | 当前 |
| M0-ADRs/ | `../M0-ADRs/`(14 项 ADR 全部 Accepted,2026-05-01) | 当前 |
| BASELINE.md | `../BASELINE.md`(单一权威入口) | 当前 |

## 已归档文档(2026-05-01 起)

### 需求设计/
- `prd-v0.1.md`(实际内容 v0.2)→ 被 v0.3 覆盖
- `prd-v0.3.md` → 被 v0.4 覆盖
- `prd-v0.4.md` → 被 v0.5 覆盖
- `prd-v0.5.md` → 被 v0.6 覆盖
- `prd-v0.6.md` → 被 v0.7 覆盖(M0 决策固化)
- `prd-v0.7.md` → 被 v1.0 覆盖(2026-05-01 压平为实施基线)
- `flows-v0.3.md` → 被 v0.4 覆盖
- `flows-v0.4.md` → 被 v0.5 覆盖
- `flows-v0.5.md` → 被 v0.6 覆盖
- `flows-v0.6.md` → 被 v1.0 覆盖(2026-05-01 压平)
- `telemetry-uat-v0.3.md` → 被 v0.5 覆盖
- `telemetry-uat-v0.5.md` → 被 v0.6 覆盖
- `telemetry-uat-v0.6.md` → 被 v1.0 覆盖(2026-05-01 压平)

### 架构设计/
- `system-architecture-v0.1.md` → 被 v0.2 覆盖
- `system-architecture-v0.2.md` → 被 v0.3 覆盖
- `system-architecture-v0.3.md` → 被 v0.4 覆盖(M0 技术栈定版)
- `system-architecture-v0.4.md` → 被 v1.0 覆盖(2026-05-01 压平为实施基线)

### 角色与人格/
- `persona-design-v0.1.md` → 被 v0.2 覆盖
- `persona-design-v0.2.md` → 被 v1.0 覆盖(2026-05-01 压平)

## 压平基线说明(2026-05-01)

PRD v1.0 / 架构 v1.0 / 人格 v1.0 / flows v1.0 / UAT v1.0 是 v0.1 → v0.7(PRD)/ v0.1 → v0.4(架构)/ v0.1 → v0.2(人格)/ v0.3 → v0.6(flows)/ v0.3 → v0.6(UAT)+ 14 项 ADR 决策结果的"压平基线"。所有"v0.X 沿用 / 新增"等增量话术已展开,文档以连续叙事呈现实施期完整内容。

后续变更通过新 ADR(ADR-015+) + 增量小版本(v1.1+)管理,而非再叠加增量补丁。如需查询历史决策演进过程,本目录提供完整历史快照。

## Obsidian 配置建议

在 Obsidian 设置中将本目录加入"忽略的文件和文件夹",避免在搜索/图谱视图中污染当前文档。

设置路径:**Settings → Files and links → Excluded files**(添加 `_archive/`)

或在 Obsidian 4.0+ 用 file recovery 配置中标记为 ignore。

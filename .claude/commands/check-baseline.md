---
description: 验证 BASELINE.md 各文档版本号与实际文档头一致;ADR 表行数 = 实际 ADR 文件数
allowed-tools: Bash(ls:*), Bash(grep:*), Bash(head:*), Read, Glob
---

## 任务

校验 `docs/AIPET-obsidian/BASELINE.md` 是否与实际仓库状态同步。重点防"漏升修补"(2026-05-02 roadmap 漏升 v1.1 case)。

## 检查项

1. § 五份对齐文档表中各文档的版本号 vs 各文档实际头部声明(`v1.0` / `v1.1` ...)
2. § 实施路线图表中 roadmap 文档版本号 vs 实际
3. § ADR 表行数 vs `docs/AIPET-obsidian/M0-ADRs/ADR-*.md` 文件数

## 上下文

- BASELINE.md:
@docs/AIPET-obsidian/BASELINE.md

- 现有 ADR 文件清单:
!`ls docs/AIPET-obsidian/M0-ADRs/ADR-*.md`

- 5 份对齐文档头部:
!`head -3 docs/AIPET-obsidian/需求设计/2026-05-01-ai-desktop-pet-prd-v1.0.md`
!`head -3 docs/AIPET-obsidian/架构设计/2026-05-01-system-architecture-v1.0.md`
!`head -3 docs/AIPET-obsidian/需求设计/2026-05-01-ai-desktop-pet-flows-v1.0.md`
!`head -3 docs/AIPET-obsidian/角色与人格/2026-05-01-persona-design-v1.0.md`
!`head -3 docs/AIPET-obsidian/需求设计/2026-05-01-ai-desktop-pet-telemetry-uat-v1.0.md`
!`head -3 docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md`

## 输出

输出 diff 格式:

```
✅ PRD: BASELINE.md 标 v1.1 = 实际 v1.1
❌ 架构: BASELINE.md 标 v1.1, 实际 v1.0 — 漏升修补
✅ ADR: BASELINE.md 表 15 行 = 实际 15 个文件
```

⚠️ 只输出报告,不修改文件。

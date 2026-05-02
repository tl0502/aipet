# M0 决策周 ADR 索引

> Architecture Decision Records — 在实施(M1)开始前必须签字定版的 14 项决策。

## 决策周时间窗

- **建议起始**:M1 开始前 1 周(W0)
- **建议时长**:5 个工作日
- **每日节奏**:上午讨论 1-2 个 ADR,下午让 owner 调研并起草决策,次日复评签字。

## 工作流

```
1. Owner 阅读对应 ADR 模板的【背景】+【备选方案】
2. 必要时做轻量调研 / 原型 spike
3. 在 ADR 文件中填【决策】+【后果】+【实施动作】
4. 状态从 Proposed → Accepted
5. 在 PR / 文档评审中获得至少 1 个其他 reviewer 签字
6. 无变更后冻结(标 Accepted, 不再修改)
```

## ADR 状态枚举

- **Proposed**:已起草,待讨论
- **Accepted**:决策已定,进入实施
- **Deprecated**:已废弃(不再适用)
- **Superseded**:被新 ADR 取代,文末注明引用关系

## 14 项 ADR 总表

按依赖顺序(blocker 优先)排列。前 12 项为 M0 内必须完成,13-14 可推迟到 M1-M3。**ADR-015+ 为实施期新起草**(沿用同流程,Proposed → Accepted)。

| # | ADR | 影响模块 | 优先级 | 状态 | Owner |
|---|---|---|---|---|---|
| 001 | [前端框架选型](ADR-001-frontend-framework.md) | 全部 UI | 🔴 P0 | Accepted | M0 决策周 |
| 002 | [2D 桌宠资源管线](ADR-002-2d-asset-pipeline.md) | A/I/N/O | 🔴 P0 | **Superseded**(原 Live2D,M0 末改为 VRM 3D) | M0 决策周 |
| 003 | [配饰美术管线兼容性](ADR-003-accessory-pipeline.md) | O 装扮 | 🔴 P0 | **Superseded**(随 ADR-002 切换到 VRM bone attach) | M0 决策周 |
| 004 | [物理交互动作清单](ADR-004-interaction-actions.md) | N 物理交互 | 🔴 P0 | Accepted | M0 决策周 |
| 005 | [默认 LLM Provider](ADR-005-default-llm-provider.md) | B 对话 | 🔴 P0 | Accepted | M0 决策周 |
| 006 | [安全前缀文案](ADR-006-safety-prefix.md) | SecurityGuard | 🔴 P0 | Accepted | M0 决策周(法务签字) |
| 007 | [LLM 游戏场景白名单 + 拒答](ADR-007-llm-game-scenes.md) | Q 小游戏 | 🟡 P0 | Accepted | M0 决策周(法务签字) |
| 008 | [灵魂宣誓页文案](ADR-008-soul-pledge-copy.md) | M 灵魂宣誓 | 🟡 P0 | Accepted | M0 决策周(法务签字) |
| 009 | [3 个内置人格 .soul.md](ADR-009-builtin-personas.md) | H 人格 | 🟡 P0 | Accepted | M0 决策周 |
| 010 | [音效包来源](ADR-010-voice-pack-source.md) | P 声音表情 | 🟡 P0 | Accepted | M0 决策周 |
| 011 | [装扮付费 schema 范式](ADR-011-wardrobe-monetization.md) | O 装扮 | 🟢 P0 | Accepted | M0 决策周 |
| 012 | [小游戏 UI 风格](ADR-012-game-ui-style.md) | Q 小游戏 | 🟢 P0 | Accepted | M0 决策周 |
| 013 | [签名分发(EV 证书)](ADR-013-code-signing.md) | 发布期 | 🟢 P1 | Accepted | M0 决策周(M5+ 再评估升级) |
| 014 | [本地小模型候选](ADR-014-local-small-model.md) | P1-R3 探索 | 🟢 P2 | Accepted(方向) | M0 决策周(P1-R3 重 benchmark) |
| 015 | [对话面板三形态架构](ADR-015-chat-three-modes.md) | B 对话 / A 控制按钮 / G/H/Q 整合 | 🔴 P0 | Accepted(M1 D3 起草并签字 2026-05-02) | M1 D3 实施期 |

🔴 = blocker(本项不定无法启动相关模块) / 🟡 = 重要(可能影响美术/法务排期) / 🟢 = 可推迟

## 决策依赖图

```
ADR-001 (前端) ─┐
                ├─→ 影响所有 UI 实现
ADR-002 (2D) ───┼─→ ADR-003 (配饰兼容)
                │   └─→ O 装扮
                ├─→ ADR-004 (动作清单)
                │   └─→ N 物理交互
                └─→ ADR-009 (人格 avatar.pack)

ADR-005 (LLM) ──┬─→ B 对话
                ├─→ ADR-006 (安全前缀)
                │   └─→ ADR-007 (LLM 游戏拒答)
                │       └─→ Q LLM 游戏
                └─→ ADR-014 (本地小模型可作 fallback)

ADR-008 (灵魂宣誓) ─→ M 模块
ADR-009 (人格)    ─→ H 模块 + 全部模板池(调侃 / 庆祝)

ADR-010 (音效) ───→ P 模块
ADR-011 (装扮付费) ─→ O 模块 schema
ADR-012 (游戏 UI) ─→ Q 模块前端
ADR-013 (签名)   ─→ 发布期(M5)
```

## 跨 ADR 共识

以下原则不在单个 ADR 内讨论,而是所有 ADR 必须遵守:

1. **Local-first**:任何决策不引入用户数据强制上传。
2. **用户自主权**:任何决策不削弱用户对 .soul.md / 装扮 / 设置 的控制。
3. **非养成原则**:任何决策不引入"流失 / 死亡 / 必须签到" 机制。
4. **隐私边界**:任何决策不引入读取应用名 / 窗口标题 / 输入内容 的能力。
5. **安全护栏不可绕过**:任何决策不允许人格 / 游戏场景覆盖系统安全前缀。

## 输出要求

每个 ADR 必须包含:
- ✅ **决策**:选定方案 + 一句话核心理由
- ✅ **后果**:正面 + 负面各 ≥ 2 条
- ✅ **实施动作**:可勾选的任务清单(由 owner 在 M1-M5 期间消化)
- ✅ **复审签字**:至少 1 名其他 reviewer 标注同意

## 索引同步

ADR 标题或状态变更时,**必须**同步更新本 README 总表,作为单一权威入口。

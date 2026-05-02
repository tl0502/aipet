# AI 桌宠 文档基线 v1.0(单一权威入口)

- 创建日期:2026-05-01
- 项目代号:AIPET
- 项目目录:`D:\Project\ai桌宠\`
- 适用阶段:**MVP 实施期**(M1-M5,10 周)

---

## 一句话定位

**`一个由你亲手塑造、会主动关心你、能和你一起玩的 AI 桌宠`**

三引擎差异化:
1. **用户自主人格** —— `.soul.md` 完全归属用户(参考 OpenClaw)。
2. **主动陪伴** —— 桌宠"在那里"被感知(基于本地空闲信号,不读屏幕内容)。
3. **共同活动** —— 物理交互、装扮、声音表情、本地 + LLM 小游戏。

---

## 五份对齐文档(实施期权威源)

按阅读顺序:

| # | 文档 | 路径 | 行数 | 用途 |
|---|---|---|---|---|
| 1 | **PRD v1.0** | [需求设计/2026-05-01-ai-desktop-pet-prd-v1.0.md](需求设计/2026-05-01-ai-desktop-pet-prd-v1.0.md) | 838 | 业务需求、模块清单、KPI、版本计划 |
| 2 | **架构 v1.0** | [架构设计/2026-05-01-system-architecture-v1.0.md](架构设计/2026-05-01-system-architecture-v1.0.md) | 1157 | 技术栈、服务边界、SQLite schema、IPC、文件布局 |
| 3 | **人格设计 v1.0** | [角色与人格/2026-05-01-persona-design-v1.0.md](角色与人格/2026-05-01-persona-design-v1.0.md) | 593 | `.soul.md` schema、3 个内置人格、安全前缀拼装 |
| 4 | **flows v1.0** | [需求设计/2026-05-01-ai-desktop-pet-flows-v1.0.md](需求设计/2026-05-01-ai-desktop-pet-flows-v1.0.md) | 1125 | Onboarding、状态机、关键流程图 |
| 5 | **埋点 UAT v1.0** | [需求设计/2026-05-01-ai-desktop-pet-telemetry-uat-v1.0.md](需求设计/2026-05-01-ai-desktop-pet-telemetry-uat-v1.0.md) | 550 | 事件字典、KPI 口径、UAT 验收场景 |

**实施路线图**:

| 文档 | 路径 | 用途 |
|---|---|---|
| **开发路线图 v1.0** | [2026-05-01-development-roadmap-v1.0.md](2026-05-01-development-roadmap-v1.0.md) | M0-M5 甘特图、模块依赖 DAG、关键路径、风险时间线、状态门、工作流约定 |

---

## 15 项 ADR(M0 14 项 Accepted + M1 起草 1 项 Accepted,2026-05-01 / 05-02)

[M0-ADRs/](M0-ADRs/) 目录下的 15 份决策记录,实施期不可绕过。

| # | ADR | 决策摘要 |
|---|---|---|
| 001 | [前端框架](M0-ADRs/ADR-001-frontend-framework.md) | Vue 3 + TypeScript + Pinia + Vite,组件库 M1 spike 后定 |
| 002 | [3D 资源管线(VRM)](M0-ADRs/ADR-002-2d-asset-pipeline.md) | **VRM 3D**(Three.js + `@pixiv/three-vrm`)。原 Live2D 路线在 M0 末因 Cubism Core 6 不兼容废止,详见 ADR-002 顶部 Superseded 说明 |
| 003 | [配饰美术管线](M0-ADRs/ADR-003-accessory-pipeline.md) | VRM humanoid bone attach + VRMC_node_constraint,切换 < 500ms(原 Live2D 插槽方案 Superseded) |
| 004 | [物理交互动作](M0-ADRs/ADR-004-interaction-actions.md) | 12 个核心动作 ID,默认 reaction_table 可被 .soul.md 覆盖 |
| 005 | [默认 LLM Provider](M0-ADRs/ADR-005-default-llm-provider.md) | 零默认 + 6 个 preset(OpenAI / DeepSeek / Moonshot / 通义 / Ollama / 自定义) |
| 006 | [安全前缀](M0-ADRs/ADR-006-safety-prefix.md) | 通用核心(全球 5 条)+ 地区补充(zh-CN / international) |
| 007 | [LLM 游戏场景](M0-ADRs/ADR-007-llm-game-scenes.md) | 1+1 双场景:故事接龙 + 咖啡店老板 |
| 008 | [灵魂宣誓文案](M0-ADRs/ADR-008-soul-pledge-copy.md) | 温暖叙述版 v1.0,默默 momo 第一人称 |
| 009 | [3 个内置人格](M0-ADRs/ADR-009-builtin-personas.md) | 默默 / 阿吉 / 教官 |
| 010 | [音效包来源](M0-ADRs/ADR-010-voice-pack-source.md) | 自录(产品配音),12-20 条 OGG,预算 ¥3000-6000 |
| 011 | [装扮付费 schema](M0-ADRs/ADR-011-wardrobe-monetization.md) | 结构化对象 + JSON 列存储,MVP 期 paid 不返前端 |
| 012 | [小游戏 UI](M0-ADRs/ADR-012-game-ui-style.md) | 独立游戏舱 GameRoom 窗口(480 × 600) |
| 013 | [代码签名](M0-ADRs/ADR-013-code-signing.md) | M5 灰度期不签名 + user education,EV/OV 推到 M5+ |
| 014 | [本地小模型 P1-R3](M0-ADRs/ADR-014-local-small-model.md) | 调用本地 Ollama,推荐 Qwen2.5-3B-Instruct-Q4 |
| 015 | [对话面板三形态架构](M0-ADRs/ADR-015-chat-three-modes.md) | hub 总面板 + 磁吸浮窗 + 漫画气泡 + ConversationStore 共享(M1 D3 起草,Accepted 2026-05-02) |

ADR 索引与依赖图:[M0-ADRs/README.md](M0-ADRs/README.md)。

---

## 关键约束(贯穿所有决策)

1. **Local-first**:不引入用户数据强制上传。
2. **用户自主权**:不削弱用户对 `.soul.md` / 装扮 / 设置的控制。
3. **非养成原则**:不引入流失 / 死亡 / 必须签到机制。
4. **隐私边界**:不读应用名 / 窗口标题 / 输入内容 / 麦克风。
5. **安全护栏不可绕过**:任何人格 / 游戏场景不能覆盖系统安全前缀。

---

## MVP 实施路线(10 周)

| 里程碑 | 周次 | 主要交付 | 退出条件 |
|---|---|---|---|
| **M0** | W0 | 14 项 ADR Accepted、3 个内置人格定稿、桌宠渲染 spike(原计划 Live2D,实际改为 VRM) | ADR 全部签字(已完成 2026-05-01) |
| **M1** | W1-2 | Tauri + Vue 3 项目骨架、桌宠壳层、对话、Onboarding、灵魂宣誓、自由活动初版、U.1/U.2 昵称、组件库 spike | 核心 UI 跑通;快捷键稳定;崩溃率 < 3% |
| **M2** | W3-4 | 任务三件套(C/D/E)、人格系统、心情/精力、摸鱼、N 物理交互(含 RAWINPUT spike) | 三大模块离线可用;N hitbox 触发率 ≥ 95% |
| **M3** | W5-6 | 记忆、隐私治理、自动更新、情境关心(J)、文件拖入(L)、R.3 桌宠日常 | 安全模板覆盖;桌宠日常分布合理 |
| **M4** | W7-8 | O 装扮(配饰 + 节气)、P 声音表情(音效 + 静音)、S.4 用户纪念日 | 装扮切换 < 500ms;声音工作时段 0 触发 |
| **M5** | W9-10 | Q 小游戏(本地 3 + LLM 2)、优化、灰度内测(10 → 100 名)、KPI 收敛 | KPI 11.15-11.20 全部可观测;可发布候选版 |

P1 路线:M6-M7 R1 情感深化 / M8-M9 R2 效率与娱乐深化 / M10+ R3 生态扩展(详见 PRD v1.0 §5.2)。

---

## 性能预算速查

| 项 | 预算 |
|---|---|
| 总常驻内存 | ≤ 250MB |
| 总安装包 | ≤ 80MB |
| 冷启动 | ≤ 5 秒 |
| 对话首 token | p50 ≤ 1.5s |
| 物理交互响应 | < 100ms |
| 装扮切换 | < 500ms |
| 声音播放延迟 | < 50ms |
| 本地游戏每轮 | < 50ms |
| 摸鱼切换 | < 200ms |

详见 PRD v1.0 §10 与架构 v1.0 §13。

---

## 立项档案(无版本号,基线参考)

- [竞品研究](2026-04-30-ai-desktop-pet-competitor-research.md) — Replika / Character.AI / Nomi / Clawster / PetClaw / Desktop Mate / Microsoft Copilot 等。

---

## 历史归档

[_archive/](\_archive/) 目录下保留 v0.1 → v0.7 全部历史版本(PRD 7 份、架构 4 份、flows 4 份、UAT 3 份、人格 2 份),仅供回溯历史决策,**实施期不应参考**。

详见 [_archive/README.md](_archive/README.md)。

---

## 工作流约定

### 新决策

实施期(M1-M5)如发现新决策需求:
1. 在 `M0-ADRs/` 创建 `ADR-015+`,流程沿用 PRD v1.0 §0。
2. 状态:Proposed → Accepted(签字后)→ Superseded(被替代时,标注引用关系)。
3. 已 Accepted 的 ADR **不删除、不修改**;变化通过新 ADR 表达。

### 新版本

如需对五份对齐文档之一做实质性变更:
1. **小修小补**(typo / 字段补充 / 新事件):直接 in-place 编辑,不升版本号。
2. **章节级新增 / 修改**:升 v1.1 / v1.2,在文档头追加变更摘要;不强制压平基线。
3. **重大架构调整 / 新模块**:走 ADR 流程;视情况启动 v2.0 重新压平。

### Obsidian 配置建议

将 `_archive/` 加入 Obsidian 的"忽略文件夹"避免污染搜索与图谱:**Settings → Files and links → Excluded files**(添加 `_archive/`)。

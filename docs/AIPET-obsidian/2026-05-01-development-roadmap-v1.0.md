# AI 桌宠 开发路线图 v1.1

- 文档版本:v1.1(高层路线图;在 v1.0 上做章节级增量,不压平)
- 创建日期:2026-05-01(v1.0)/ 2026-05-02(v1.1 增量)
- 适用阶段:M0 完成 → M1 启动前(实施期入口);v1.1 反映 M1 D3 期 ADR-015 决策
- 输出形式:Mermaid 可视化 + 表格
- 关联:
  - [BASELINE.md](BASELINE.md)
  - [需求设计/2026-05-01-ai-desktop-pet-prd-v1.0.md](需求设计/2026-05-01-ai-desktop-pet-prd-v1.0.md)(已升 v1.1)
  - [架构设计/2026-05-01-system-architecture-v1.0.md](架构设计/2026-05-01-system-architecture-v1.0.md)(已升 v1.1)
  - [需求设计/2026-05-01-ai-desktop-pet-flows-v1.0.md](需求设计/2026-05-01-ai-desktop-pet-flows-v1.0.md)(已升 v1.1)
  - [M0-ADRs/](M0-ADRs/)(15 项 Accepted)

> **关于本文档**:基于 PRD v1.0 §13 与架构 v1.0 §15 的 M1-M5 实施任务对照,展开为可视化路线图。**日期为相对周次锚定**(M0 = W0,2026-05-01 起);实际启动日由项目主理决定。

## 变更摘要

### v1.1(2026-05-02)

实施期 M1 D3 经 [ADR-015](M0-ADRs/ADR-015-chat-three-modes.md)《对话面板三形态架构》Accepted 后增量:

- §3.2 模块矩阵:`ChatService + LLMProvider` 行展开为 B.3.a-f 跨 M1-M5;新增 `ConversationStore` / `控制按钮区(模块 A 延伸)` / `hub 总面板` 行
- §5.2-5.6 各 milestone:**Mermaid 图保持 v1.0**(高层视角不变);story 级增量(B.3.a-f / 智能穿透 II/III/I/IV / 入口前置 TBD)由 [progress/m1-5.md](../../progress/) 作为权威细节源
- 头部关联文档行加 v1.1 标记

未变更:§1 三引擎 / §3.1 模块 DAG Mermaid / §4 关键路径 / §6 风险 / §7 状态门 / §8-§11 工作流 与 v1.0 一致。

---

## 1. 总览

### 1.1 三引擎与五大主线

```mermaid
graph LR
    subgraph 三引擎差异化
        E1[用户自主人格<br/>.soul.md]
        E2[主动陪伴<br/>本地空闲信号]
        E3[共同活动<br/>交互/装扮/游戏]
    end

    subgraph 五大主线
        L1[陪伴<br/>60%]
        L2[效率<br/>40%]
        L3[可控<br/>隐私/记忆]
        L4[娱乐性<br/>v0.6 升级]
        L5[离线可用<br/>硬约束]
    end

    E1 --> L1
    E2 --> L1
    E3 --> L4
    L1 --> L3
    L2 --> L3
    L4 --> L5
```

### 1.2 关键约束速查

| 约束 | 含义 |
|---|---|
| **Local-first** | 不引入用户数据强制上传 |
| **用户自主权** | 不削弱用户对 .soul.md / 装扮 / 设置的控制 |
| **非养成** | 不引入流失 / 死亡 / 必须签到 |
| **隐私边界** | 不读应用名 / 窗口标题 / 输入内容 / 麦克风 |
| **安全护栏** | 任何人格 / 游戏不能覆盖系统安全前缀 |

### 1.3 五份基线 → 路线图

```mermaid
graph TD
    BASE[BASELINE.md<br/>单一权威入口]
    PRD[PRD v1.0<br/>需求/KPI]
    ARCH[架构 v1.0<br/>服务/IPC/Schema]
    PERSONA[人格设计 v1.0<br/>.soul.md schema]
    FLOW[flows v1.0<br/>状态机/流程]
    UAT[埋点 UAT v1.0<br/>事件/验收]
    ADR[14 项 ADR<br/>已 Accepted]
    ROAD[开发路线图 v1.0<br/>本文档]

    BASE --> PRD
    BASE --> ARCH
    BASE --> PERSONA
    BASE --> FLOW
    BASE --> UAT
    BASE --> ADR
    PRD --> ROAD
    ARCH --> ROAD
    ADR --> ROAD
```

---

## 2. 项目时间轴(W0-W10)

```mermaid
gantt
    title AI 桌宠 MVP 时间轴(M0 完成 → M5 候选版,11 周)
    dateFormat YYYY-MM-DD
    axisFormat %m-%d

    section M0 决策周(已完成)
    14 项 ADR Accepted    :done, m0a, 2026-05-01, 5d
    桌宠渲染 spike(VRM,末日) :done, m0b, after m0a, 2d

    section M1 壳层 + 对话
    项目脚手架 + 组件库 spike :active, m1a, 2026-05-08, 3d
    Tauri 透明窗口 + IPC 框架 :m1b, after m1a, 4d
    PersonaService + ChatService MVP :m1c, after m1b, 4d
    Onboarding + 灵魂宣誓 + 昵称 :m1d, after m1c, 3d

    section M2 任务三件套 + 物理交互
    TaskService(C/D/E)    :m2a, 2026-05-22, 5d
    人格工坊 + 试聊沙盒    :m2b, after m2a, 3d
    心情/精力 + BossKey    :m2c, after m2b, 3d
    InteractionRouter + 抗议规则 :m2d, after m2c, 3d
    RAWINPUT spike         :crit, m2e, 2026-05-25, 3d

    section M3 记忆 + 主动陪伴
    LLM Provider + SecurityGuard :m3a, 2026-06-05, 4d
    IdleDetector + ProactiveCare :m3b, after m3a, 4d
    FileDropHandler + MilestoneService :m3c, after m3b, 3d
    日常时段表(R.3)      :m3d, after m3c, 3d

    section M4 装扮 + 声音 + 纪念日
    WardrobeService(配饰 + 节气) :m4a, 2026-06-19, 5d
    VoiceEffectPlayer + 静音逻辑 :m4b, after m4a, 4d
    用户纪念日(S.4)      :m4c, after m4b, 3d
    装扮工坊前端          :m4d, after m4a, 7d

    section M5 小游戏 + 灰度
    GameEngine(本地 3 + LLM 2)   :m5a, 2026-07-03, 5d
    GameRoom 窗口 + 安全前缀复用 :m5b, after m5a, 3d
    KPI 埋点 11.15-11.20  :m5c, after m5b, 2d
    灰度 W1(10 名内部)   :crit, m5d, 2026-07-13, 3d
    灰度 W2(100 名内测)  :crit, m5e, after m5d, 4d
```

> **节点说明**:`active` = 进行中;`done` = 已完成;`crit` = 关键路径节点(任一延期 → 整体延期)。

---

## 3. 模块依赖 DAG

### 3.1 全模块依赖图

```mermaid
graph TD
    %% 基础设施
    Migration[MigrationService]
    Crypto[CryptoService DPAPI]
    Network[NetworkProbe]
    Telemetry[TelemetryService]
    Updater[UpdaterService]

    %% 核心服务
    Persona[PersonaService<br/>+SecurityGuard]
    Memory[MemoryService]
    Nickname[NicknameService<br/>facade]
    Chat[ChatService]
    LLMProv[LLMProvider<br/>OpenAI 兼容]
    Task[TaskService<br/>C/D/E]

    %% 生命感与陪伴
    Idle[IdleDetector<br/>+RAWINPUT]
    Living[LivingPetService<br/>含 DailySchedule]
    ProCare[ProactiveCareService]
    BossKey[BossKeyService]
    FileDrop[FileDropHandler]
    Milestone[MilestoneService<br/>含 user_anniversary]

    %% 交互/装扮/游戏/声音
    Interact[InteractionRouter<br/>hitbox→action]
    Voice[VoiceEffectPlayer<br/>+静音时段]
    Wardrobe[WardrobeService<br/>+付费预埋]
    Game[GameEngine<br/>Local+LLMRunner]

    %% 前端
    PetCanvas[PetCanvas<br/>VRM]
    ChatPanel[ChatPanel]
    Workshop[人格工坊]
    WardrobeStudio[装扮工坊]
    GameRoom[GameRoom 窗口]

    %% 依赖关系
    Migration --> Crypto
    Crypto --> Persona
    Persona --> Memory
    Memory --> Nickname
    Nickname --> Chat
    Persona --> Chat
    Memory --> Chat
    LLMProv --> Chat
    Task --> Chat

    Idle --> Living
    Idle --> ProCare
    Persona --> ProCare
    Milestone --> ProCare

    Idle --> Interact
    Persona --> Interact
    Voice --> Interact

    Wardrobe --> PetCanvas
    Voice --> PetCanvas
    Interact --> PetCanvas
    Persona --> PetCanvas

    Persona --> Game
    LLMProv --> Game
    Game --> GameRoom

    Chat --> ChatPanel
    Persona --> Workshop
    Wardrobe --> WardrobeStudio

    %% 样式
    classDef infra fill:#e8f4f8,stroke:#0288d1
    classDef core fill:#fff3e0,stroke:#f57c00
    classDef life fill:#f3e5f5,stroke:#7b1fa2
    classDef inter fill:#e8f5e9,stroke:#388e3c
    classDef ui fill:#fce4ec,stroke:#c2185b

    class Migration,Crypto,Network,Telemetry,Updater infra
    class Persona,Memory,Nickname,Chat,LLMProv,Task core
    class Idle,Living,ProCare,BossKey,FileDrop,Milestone life
    class Interact,Voice,Wardrobe,Game inter
    class PetCanvas,ChatPanel,Workshop,WardrobeStudio,GameRoom ui
```

### 3.2 模块 → milestone 占用矩阵

> **v1.1 增量**:`ChatService + LLMProvider` 行已按 ADR-015 拆 B.3.a-f 跨 M1-M5;新增 `ConversationStore` / `控制按钮区(模块 A 延伸)` / `hub 总面板` 三行。

| 模块 | M1 | M2 | M3 | M4 | M5 |
|---|---|---|---|---|---|
| **基础设施**(Migration / Crypto / Telemetry / Network / Updater) | 骨架 | — | 完善 | — | 灰度埋点 |
| **PersonaService** | MVP(加载/激活) | 工坊 + 沙盒 | — | — | — |
| **MemoryService + NicknameService** | MVP | — | — | — | — |
| **ChatService + LLMProvider** | MVP(单 Provider)+ **B.3.a 形态 2 极简** | — | OpenAI 兼容完整 + SecurityGuard + **B.3.d 多 conversation** | — | — |
| **ConversationStore**(v1.1 / ADR-015) | 表 schema 就位(I.1) | — | 完整 CRUD UI(随 B.3.d) | — | — |
| **控制按钮区**(模块 A 延伸,v1.1) | — | **B.3.b 骨架**(0.5d) | — | — | — |
| **ChatPanel 形态 2 磁吸**(v1.1) | (B.3.a 极简内含) | **B.3.c 磁吸交互** | — | — | — |
| **hub 总面板**(形态 1,v1.1) | — | — | — | **B.3.e 4 tab** | — |
| **形态 3 漫画气泡**(v1.1) | — | — | — | — | **B.3.f 角色窗内** |
| **TaskService**(C/D/E) | — | 全量 | — | — | — |
| **LivingPetService** | 自由活动初版 | mood/energy + 持久化 | DailySchedule(R.3) | — | — |
| **IdleDetector** | — | (N.4 spike) | 主体 + ProactiveCare | — | — |
| **ProactiveCareService** | — | — | 主体 + 频率上限 + 安静时段 | — | — |
| **BossKeyService** | (A.5 占位 emit) | 摸鱼模式接管 | — | — | — |
| **FileDropHandler** | — | — | 文本类全功能 + **接收源扩展(角色窗/各形态输入区,ADR-015)** | — | — |
| **MilestoneService** | — | — | 首次 7/30 天 | + user_anniversary | — |
| **InteractionRouter**(模块 N) | — | hitbox + reaction_table + 抗议 | — | — | — |
| **WardrobeService**(模块 O) | — | — | — | 配饰 + 节气 + 付费预埋 | — |
| **VoiceEffectPlayer**(模块 P) | — | — | — | 音效 + 静音时段 | — |
| **GameEngine**(模块 Q) | — | — | — | — | 全量(Local 3 + LLM 2)+ **hub 游戏 tab launcher** |

---

## 4. 关键路径(blocker chain)

任一节点延期 → 整体延期。

```mermaid
graph LR
    A[M0:VRM 渲染 spike<br/>ADR-002 Superseded] --> B[M1:Tauri + Vue 骨架<br/>组件库 spike]
    B --> C[M1:PersonaService MVP<br/>+ ChatService MVP]
    C --> D[M1:Onboarding + Soul Pledge<br/>ADR-008]

    D --> E[M2:TaskService + 人格工坊]
    E --> F[M2:InteractionRouter<br/>+ 抗议规则]
    F --> G[M2:RAWINPUT spike<br/>决断 N.4]

    G --> H[M3:LLM Provider 完整<br/>+ SecurityGuard ADR-006]
    H --> I[M3:IdleDetector<br/>+ ProactiveCare]
    I --> J[M3:FileDrop + Milestone<br/>+ R.3 日常时段]

    J --> K[M4:WardrobeService<br/>blocker:配饰美术 ADR-003]
    K --> L[M4:VoiceEffectPlayer<br/>blocker:音效自录 ADR-010]
    L --> M[M4:用户纪念日 S.4]

    M --> N[M5:GameEngine<br/>blocker:场景 yaml ADR-007]
    N --> O[M5:KPI 埋点 11.15-11.20]
    O --> P[M5:灰度 W1 内部 10 人]
    P --> Q[M5:灰度 W2 内测 100 人]
    Q --> R[发布候选版<br/>RC]

    classDef critical fill:#ffebee,stroke:#c62828,stroke-width:2px
    classDef done fill:#e8f5e9,stroke:#2e7d32
    class A done
    class G,K,L,N,P,Q critical
```

### 4.1 关键 spike 与决策点

| 节点 | 时机 | 决策内容 | 失败降级 |
|---|---|---|---|
| **桌宠渲染 spike(VRM)** | M0 末(已完成,从 Live2D 切换) | 启动 < 1500ms / 内存 < 150MB / 配饰挂载点(humanoid bone)可行 | 降级"整套皮肤"(配饰仅整体替换,牺牲 KPI 11.17) |
| **组件库 spike** | M1 W1 第 1 天 | Naive UI vs Element Plus 哪个更适合 | 默认 Naive UI |
| **RAWINPUT spike** | M2 内 | 实现成本是否可控 | 降级"快速 idle 切换"近似信号(N.4 体验弱化) |
| **配饰美术管线就绪** | M4 启动前 | 8 件配饰 + 4 套节气资源齐 | 推迟节气皮肤到 M4 末或 P1-R1 |
| **音效自录交付** | M4 启动前 | 12-20 条 OGG 录制完成 | 推迟到 M4 末或 P1-R1 |
| **LLM 场景 yaml 法务** | M5 启动前 | 故事接龙 + 咖啡店老板法务签字 | 推迟 LLM 游戏到 P1-R2 |
| **灰度 W1 退出** | M5 W2 入口 | 内部 10 人无 P0 事故 | 修复后再扩 100 人 |
| **灰度 W2 退出** | M5 末 | KPI 11.15-11.20 至少 3 项达标 + 杀死指标无命中 | 修复 + 延期或降级 |

---

## 5. 各 Milestone 详细

### 5.1 M0 决策周(已完成,2026-05-01)

**入口**:产品立项;PRD v0.6/v0.7 草稿;14 项 ADR 草案。
**出口**:✅ 14 项 ADR 全部 Accepted;✅ 五份对齐文档压平到 v1.0;✅ 桌宠渲染 spike 通过(原 Live2D 改为 VRM)。

```mermaid
graph LR
    A[14 ADR<br/>Proposed] --> B[备选方案<br/>+ 我的倾向]
    B --> C[决策签字]
    C --> D[Accepted<br/>+ 后果 + 实施动作]
    D --> E[3 内置人格<br/>.soul.md 草稿]
    D --> F[VRM 渲染 spike<br/>1 天]
    D --> G[安全前缀 v1.0<br/>法务签字]
    D --> H[文档压平 v1.0]
```

### 5.2 M1 壳层 + 对话(W1-W2,2 周)

**入口**:M0 出口达成。
**出口**:核心 UI 跑通;快捷键稳定(`Ctrl+Alt+Space` / `Ctrl+Shift+B`);崩溃率 < 3%;Onboarding 6 步可走通到主态。

> **v1.1 注**:M1 实际拆 stories 见 [progress/m1.md](../../progress/m1.md):A.1-A.5 已完成 / A.6 智能穿透收口 polish(II+III)/ I.1 / I.2 / H.1 / F.1 / F.2 / B.1 / B.2 / **B.3.a 形态 2 极简版**(单 conversation + 流式,见 ADR-015)。下方 Mermaid 保留 v1.0 高层视角。

```mermaid
graph TD
    M1Start([M1 入口])
    M1Start --> A1[W1.D1<br/>项目脚手架<br/>组件库 spike]
    A1 --> B1[Tauri 透明窗口<br/>点击穿透]
    A1 --> C1[Pinia + IPC 框架]

    B1 --> D1[PetCanvas<br/>VRM 集成]
    C1 --> E1[PersonaService MVP<br/>加载 _builtin/momo]
    C1 --> F1[MemoryService + Nickname]

    E1 --> G1[ChatService MVP<br/>OpenAI 单 Provider]
    F1 --> G1
    G1 --> H1[ChatPanel<br/>流式渲染]

    D1 --> I1[Onboarding 6 步]
    H1 --> I1
    I1 --> J1[灵魂宣誓页<br/>ADR-008 默默 momo]
    J1 --> K1[U.1/U.2 昵称 UI]

    K1 --> M1End([M1 出口<br/>主态可达])

    classDef start fill:#e3f2fd,stroke:#1976d2
    class M1Start,M1End start
```

**主交付物**:
- Tauri + Vue 3 + TS + Pinia + Vite 项目脚手架
- 桌宠透明窗口(置顶 / 无边框 / 点击穿透)
- VRM 默默 momo 渲染(内置 3 个人格,但 M1 只用 momo)
- 对话面板 + 流式渲染 + OpenAI Provider
- Onboarding 6 步(灵魂宣誓 + Provider 引导 [可跳过])
- U.1 桌宠昵称 + U.2 用户昵称 UI

**风险**:
- 组件库 spike 可能拖延 2 天 → 缓解:M1 D1 必须做完
- VRM 集成异常 → M0 spike 已验证(原 Live2D 切换到 VRM),降级路线已留
- Onboarding 法务文案最终版滞后 → 用 ADR-008 v1.0 定版

### 5.3 M2 任务三件套 + 物理交互(W3-W4,2 周)

**入口**:M1 出口达成;VRM humanoid bone 配饰挂载点已验证。
**出口**:三大模块(C/D/E)离线可用;N hitbox 反应触发率 ≥ 95%;摸鱼快捷键稳定;人格切换不丢记忆。

```mermaid
graph TD
    M2Start([M2 入口])
    M2Start --> A2[TaskService<br/>提醒 C + 番茄 D + 待办 E]
    M2Start --> B2[人格工坊<br/>简易 + 进阶 + 文件 三档]
    M2Start --> C2[试聊沙盒]

    A2 --> D2[心情/精力<br/>pet_runtime_state]
    B2 --> D2

    D2 --> E2[BossKeyService<br/>Ctrl+Shift+B 摸鱼]
    D2 --> F2[InteractionRouter<br/>hitbox 解析]

    F2 --> G2[reaction_table 默认<br/>+ .soul.md 反应配置覆盖]
    G2 --> H2[抗议规则<br/>VecDeque 30s + transient]

    F2 --> I2[RAWINPUT spike]
    I2 --> J2{spike 通过?}
    J2 -->|是| K2[N.4 键鼠协同]
    J2 -->|否| L2[降级:快速 idle 切换]

    H2 --> M2End([M2 出口])
    K2 --> M2End
    L2 --> M2End

    classDef start fill:#e3f2fd,stroke:#1976d2
    classDef spike fill:#fff9c4,stroke:#f9a825
    class M2Start,M2End start
    class I2,J2 spike
```

**主交付物**:
- C 提醒系统(软/硬优先级,稍后上限 3 次)
- D 番茄钟(暂停/恢复/休眠校准)
- E 待办(创建/完成/AI 拆解占位 — M3 接 LLM)
- 人格工坊 GUI 三档编辑 + 试聊沙盒
- 心情/精力运行时状态 + 持久化
- 摸鱼模式(隐藏/恢复 < 200ms,缓冲提醒)
- N 物理交互(hitbox 反应、抗议非持久化)
- N.4 键鼠协同(若 RAWINPUT 通过)

### 5.4 M3 记忆 + 主动陪伴(W5-W6,2 周)

**入口**:M2 出口达成;LLM Provider 决定上线 OpenAI 兼容协议。
**出口**:安全前缀模板覆盖;模块 J 频率上限可验证;模块 L 文本类全功能;桌宠日常时段动作分布合理。

```mermaid
graph TD
    M3Start([M3 入口])
    M3Start --> A3[LLM Provider<br/>OpenAI 兼容完整]
    M3Start --> B3[SecurityGuard<br/>ADR-006 v1.0 注入]

    A3 --> C3[ChatService<br/>含 SecurityGuard]
    B3 --> C3

    C3 --> D3[E 待办 AI 拆解]

    M3Start --> E3[IdleDetector<br/>GetLastInputInfo]
    E3 --> F3[ProactiveCareService<br/>+ 频率上限 + 安静时段]
    C3 --> F3

    M3Start --> G3[FileDropHandler<br/>txt/md/pdf]
    M3Start --> H3[MilestoneService<br/>首次 7/30/100 天]

    H3 --> I3[LivingPetService<br/>DailySchedule R.3]

    F3 --> M3End([M3 出口])
    G3 --> M3End
    I3 --> M3End

    classDef start fill:#e3f2fd,stroke:#1976d2
    class M3Start,M3End start
```

**主交付物**:
- LLM Provider 完整(OpenAI 协议 + 6 个 preset)
- SecurityGuard 注入(安全前缀 v1.0 + 地区补充)
- E 待办 AI 拆解(接 LLM)
- IdleDetector(GetLastInputInfo)
- ProactiveCareService(频率 4 次/日 + 2h 间隔 + 安静时段)
- L 文件拖入(.txt/.md/.pdf)
- MilestoneService(首次 7/30/100 天等)
- LivingPetService DailySchedule(R.3 桌宠日常时段)
- 自动更新 UpdaterService

### 5.5 M4 装扮 + 声音 + 纪念日(W7-W8,2 周)

**入口**:M3 出口达成;配饰美术(8 件 + 4 套节气)交付;音效包(12-20 条 OGG)录制完成。
**出口**:装扮切换 < 500ms;声音工作日 09:00-18:00 严格 0 触发;用户纪念日时区跨日正确。

```mermaid
graph TD
    M4Start([M4 入口])
    M4Start --> Pre1{配饰美术就绪?}
    M4Start --> Pre2{音效自录就绪?}

    Pre1 -->|是| A4[WardrobeService<br/>list_inventory + equip]
    Pre1 -->|否| Block1[阻塞:推迟节气皮肤]

    A4 --> B4[配饰锚点叠加<br/>VRM humanoid bone attach]
    A4 --> C4[节气推送<br/>每天 00:01 检查]
    A4 --> D4[付费 schema 预埋<br/>tier=paid 强制过滤]

    Pre2 -->|是| E4[VoiceEffectPlayer<br/>HTML5 Audio]
    Pre2 -->|否| Block2[阻塞:推迟到 P1-R1]

    E4 --> F4[静音时段<br/>工作日 09:00-18:00]
    E4 --> G4[音量控制<br/>0-100 默认 50]

    M4Start --> H4[user_anniversaries 表<br/>+ MilestoneService 扩展]
    H4 --> I4[纪念日 UI<br/>设置 → 我的纪念日]
    H4 --> J4[年度键<br/>anniversary_<key>_<YYYY>]

    A4 --> K4[装扮工坊前端<br/>0-3 件叠加]

    B4 --> M4End([M4 出口])
    F4 --> M4End
    J4 --> M4End
    K4 --> M4End

    classDef start fill:#e3f2fd,stroke:#1976d2
    classDef block fill:#ffebee,stroke:#c62828
    class M4Start,M4End start
    class Pre1,Pre2,Block1,Block2 block
```

**主交付物**:
- O.1 配饰系统(8 件)+ O.2 节气皮肤(4 套)
- O.3 付费 schema 预埋(`tier='paid'` 强制过滤)
- 装扮工坊前端
- P 声音表情(默认音效包 + 静音时段 + 音量)
- S.4 用户纪念日(添加 / 触达 / 年度去重)

### 5.6 M5 小游戏 + 灰度(W9-W10,2 周)

**入口**:M4 出口达成;`game_scenes/{story_relay,cafe_owner}.yaml` 法务签字。
**出口**:KPI 11.15-11.20 全部可观测;灰度 W2(100 人)无 P0 事故;杀死指标无命中。

```mermaid
graph TD
    M5Start([M5 入口])
    M5Start --> Pre1{LLM 场景 yaml 法务签字?}

    Pre1 -->|是| A5[GameEngine 骨架<br/>start/submit/end]
    Pre1 -->|否| Block1[降级:仅本地游戏<br/>LLM 推到 P1-R2]

    A5 --> B5[LocalGameRunner<br/>RPS + 猜数 + 接龙]
    A5 --> C5[LLMGameRunner<br/>+ token 上限 2000]

    B5 --> D5[人格化点评<br/>从 .soul.md 调侃池抽]
    C5 --> E5[安全前缀复用<br/>ADR-006 + ADR-007 拒答]
    C5 --> F5[GameRoom 窗口<br/>480x600 ADR-012]

    D5 --> G5[KPI 11.15-11.20 埋点]
    E5 --> G5
    F5 --> G5

    G5 --> H5[灰度 W1<br/>10 名内部]
    H5 --> Decision1{P0 事故?}
    Decision1 -->|无| I5[灰度 W2<br/>100 名内测]
    Decision1 -->|有| Fix1[修复后再 W1]

    I5 --> Decision2{KPI 至少 3 项达标?}
    Decision2 -->|是| J5[发布候选版 RC]
    Decision2 -->|否| Fix2[修复或降级<br/>延期 1-2 周]

    J5 --> M5End([M5 出口<br/>RC 发布])

    classDef start fill:#e3f2fd,stroke:#1976d2
    classDef decision fill:#fff9c4,stroke:#f9a825
    classDef block fill:#ffebee,stroke:#c62828
    class M5Start,M5End start
    class Decision1,Decision2 decision
    class Pre1,Block1,Fix1,Fix2 block
```

**主交付物**:
- Q.1-Q.2 本地游戏(RPS / 猜数字 / 词语接龙)
- Q.3-Q.4 LLM 游戏(故事接龙 / 咖啡店老板)
- GameRoom 独立窗口(480 × 600)
- KPI 11.15-11.20 全套埋点
- 性能调优(常驻 < 250MB / 安装包 < 80MB)
- 灰度 W1 + W2 反馈采集
- 发布候选版(RC)

---

## 6. 风险时间线

13 项风险及其显化时机与缓解。命中即触发缓解动作,不命中维持原计划。

```mermaid
gantt
    title 风险显化时间线(11 周)
    dateFormat YYYY-MM-DD
    axisFormat W%U
    section M0
    Live2D 商用授权评估(已废止,切到 VRM) :done, r1, 2026-05-01, 5d
    section M1
    Tauri 在 AV 软件误报      :r2, 2026-05-08, 14d
    WebView2 缺失(老 Win10)  :r3, 2026-05-08, 14d
    section M2
    RAWINPUT 实现复杂         :crit, r4, 2026-05-22, 14d
    物理动作美术工作量爆炸    :r5, 2026-05-22, 14d
    section M3
    GetLastInputInfo RDP 不一致 :r6, 2026-06-05, 14d
    Tauri file-drop 跨版本断裂 :r7, 2026-06-05, 14d
    Milestone 时区跨日漏触发   :r8, 2026-06-05, 14d
    section M4
    VRM 渲染内存超 250MB       :r9, 2026-06-19, 14d
    节气推送被认为打扰         :r10, 2026-06-19, 14d
    DPAPI 跨用户切换异常       :r11, 2026-06-19, 14d
    section M5
    LLM 游戏 token 月成本     :crit, r12, 2026-07-03, 14d
    自由活动被关(KPI 11.12)  :r13, 2026-07-03, 14d
```

### 6.1 风险登记表

| # | 风险 | 显化时机 | 影响 | 缓解 |
|---|---|---|---|---|
| 1 | ~~Live2D 商用授权~~(已废止,切 VRM) | M0 末 spike | M0 选型 blocked | 已切 VRM(MIT 开源,无授权风险),原 Live2D 路线作废,详见 ADR-002 顶部 Superseded 说明 |
| 2 | Tauri AV 软件误报 | M1 测试 | 用户启动失败 | SmartScreen 信誉申请、AV 厂商白名单 |
| 3 | WebView2 缺失 | M1 安装期 | 应用打不开 | 安装包内置 Bootstrapper |
| 4 | RAWINPUT 实现复杂 | M2 spike | N.4 键鼠协同延期 | 降级"快速 idle 切换"近似信号(不影响其他 N 子项) |
| 5 | 物理动作美术工作量大 | M2 实施 | 美术延期 | 12 个核心动作上限(ADR-004),复用 stretch/yawn |
| 6 | GetLastInputInfo RDP 不一致 | M3 测试 | 主动关心误触发 | RDP 场景默认关闭模块 J |
| 7 | Tauri file-drop 跨版本断裂 | M3 集成测试 | 文件拖入功能断裂 | M0 锁定 Tauri 2.x 版本,M3 集成测试覆盖 |
| 8 | Milestone 时区跨日 | M3 + M5 | 重复/漏触发 | 本地时区 + 启动期幂等检查 + `milestones.id` PK 唯一 |
| 9 | VRM 渲染内存超 250MB | M4-M5 性能调优 | 性能预算超 | LOD 切换 / 低多边形 / 低分辨率贴图模式兜底 |
| 10 | 节气推送被认为打扰 | M5 灰度 | 装扮使用率(KPI 11.17)不达标 | 默认每节气仅推 1 次;用户拒绝当年不再推 |
| 11 | DPAPI 跨用户切换异常 | M3 + M5 测试 | 多用户机器混用 | 作为 feature 暴露(账户绑定) |
| 12 | LLM 游戏 token 月成本 | M5 上线后 | 用户账单爆炸 | 单次 2000 token 上限 + 设置可见消耗统计 + 告警 |
| 13 | 自由活动被关(KPI 11.12) | M5 灰度后 | 生命感关闭率 > 15% | 灰度数据驱动;> 15% 默认关闭"逛桌面"子项 |

---

## 7. 状态门(每个 milestone 出口决策)

```mermaid
stateDiagram-v2
    [*] --> M0
    M0 --> Gate0: 14 ADR Accepted +<br/>VRM 渲染 spike 通过
    Gate0 --> M1
    Gate0 --> Hold0: 任一未达
    Hold0 --> M0: 修复

    M1 --> Gate1: 主态可达 +<br/>崩溃率 < 3% +<br/>快捷键稳定
    Gate1 --> M2
    Gate1 --> Hold1: 任一未达
    Hold1 --> M1: 修复

    M2 --> Gate2: 三模块离线可用 +<br/>N hitbox ≥ 95% +<br/>RAWINPUT 决断
    Gate2 --> M3
    Gate2 --> Hold2
    Hold2 --> M2

    M3 --> Gate3: 安全模板覆盖 +<br/>主动关心可验证 +<br/>R.3 分布合理
    Gate3 --> M4
    Gate3 --> Hold3
    Hold3 --> M3

    M4 --> Gate4: 装扮切换 < 500ms +<br/>声音工作时段 0 触发 +<br/>用户纪念日时区正确
    Gate4 --> M5
    Gate4 --> Hold4
    Hold4 --> M4

    M5 --> Gate5: KPI 至少 3 项达标 +<br/>无 P0 事故 +<br/>无杀死指标命中
    Gate5 --> RC: 发布候选版
    Gate5 --> Hold5: 修复或降级
    Hold5 --> M5
    RC --> [*]
```

### 7.1 出口判断 SOP

| Milestone | 必达项 | 可妥协项(标注后通过) |
|---|---|---|
| M0 → M1 | ADR 全部 Accepted | VRM 渲染 spike 数据(可用 M1 W1 补) |
| M1 → M2 | 核心 UI 跑通 + Onboarding 完整 | API Key 引导 UX(M2 调) |
| M2 → M3 | C/D/E 离线 + 摸鱼稳定 + 物理交互 hitbox | 人格工坊 UI 美化(M3 调) |
| M3 → M4 | 主动关心频率上限 + 文件拖入文本类 + 安全前缀 | 自动更新签名(M5 决) |
| M4 → M5 | 装扮 + 声音 + 用户纪念日全量 | 装扮工坊 UX 细节(M5 调) |
| M5 → RC | KPI ≥ 3 项达标 + 杀死指标无命中 | 边界 UAT 场景(RC 后修补) |

---

## 8. 工作流约定(简短版)

### 8.1 分支策略

```
main                  ← 受保护,只接 PR
├── milestone/m1      ← M1 开发分支
│   ├── feat/m1-shell
│   ├── feat/m1-chat
│   └── feat/m1-onboarding
├── milestone/m2
└── ...
```

每个 milestone 用一个长生命周期分支,模块从该分支拉 feature 分支,完成后合回 milestone 分支。milestone 出口达成后合 main 并打 tag(`v0.M1.0` / `v0.M2.0` / ...)。

### 8.2 提交与 PR

- Commit 格式:`<type>(<scope>): <subject>` — 例 `feat(persona): add .soul.md import`
- type 枚举:`feat / fix / refactor / docs / test / perf / chore`
- PR 必须关联 ADR 编号(若决策类)或 PRD 模块号(若实施类)
- 至少 1 名 reviewer + CI 通过才能合并

### 8.3 CI 流程

```mermaid
graph LR
    PR[PR 提交] --> A[Lint + TypeCheck]
    A --> B[Unit Test<br/>核心服务 ≥ 70%]
    B --> C[PII 静态扫描<br/>messages.content/<br/>getUserMedia/<br/>GetForegroundWindow]
    C --> D[Tauri 构建]
    D --> E[Tauri 打包<br/>Windows MSI]
    E --> F{通过?}
    F -->|是| G[可合并]
    F -->|否| H[阻塞]
```

### 8.4 灰度发布(M5)

```mermaid
graph LR
    A[main tag v1.0.0-rc1] --> B[内部 10 人 灰度 W1<br/>3-4 天]
    B --> C{KPI/事故?}
    C -->|无 P0| D[内测 100 人 灰度 W2<br/>4-7 天]
    C -->|有 P0| E[修复 + rc2]
    E --> B
    D --> F{KPI ≥ 3 项 达标?}
    F -->|是| G[发布 v1.0.0]
    F -->|否| H[降级或延期]
```

---

## 9. 跟踪机制

### 9.1 任务粒度建议

```mermaid
graph TD
    A[Milestone] --> B[Module / 模块]
    B --> C[Story / 用户故事 = 1-3 天]
    C --> D[Task / 任务 = 0.5-1 天]
    D --> E[Sub-task = ≤ 4 小时]

    classDef m fill:#e3f2fd,stroke:#1976d2
    classDef l fill:#e8f5e9,stroke:#388e3c
    class A m
    class C,D,E l
```

- **看板列**:`Backlog / Up Next / In Progress / In Review / Done`
- 每周 demo + 站会(若多人项目)
- 每个 milestone 末做回顾(Retro):做对什么 / 做错什么 / 下次怎么改

### 9.2 关键监控

实施期持续观察:

| 指标 | 阈值 | 监控位置 |
|---|---|---|
| 当前 milestone 完成度 | 按周 ≥ 50% / 75% / 100% | 看板 |
| 关键路径节点延期 | 0 | 甘特图 + 站会 |
| Spike 决策延期 | 0 | 风险登记 |
| 崩溃率(Dev 测试机) | < 3% | 本地 error_logs |
| 单测覆盖率(核心服务) | ≥ 70% | CI |
| PII 静态扫描 | 0 命中 | CI |

### 9.3 文档同步

- 实施期发现 PRD/架构 与现实偏差 → 在 PR 中同步更新对应 v1.0 文档
- 重大决策变化 → 走 ADR-015+ 流程,标 Superseded 引用旧 ADR
- 实施期不再叠加增量补丁,变化直接合到 v1.0 主干(或升 v1.1)

---

## 10. 给项目主理的 5 条速读

1. **关键路径不容延期**:M0 VRM 渲染 spike → M2 RAWINPUT spike → M4 美术 + 音效就绪 → M5 法务签字 → 灰度。任一节点延期 → 整体延期。
2. **风险已知 13 项**,前置缓解都在 ADR 里。监控时机分散在 M0/M2/M3/M4/M5,每周看一次风险表。
3. **每个 milestone 2 周**,共 10 周。M0 已完成,M1 立即可启动。
4. **灰度 2 周不可砍**:W1 内部 10 人 → W2 内测 100 人。不要直接开放外部下载。
5. **杀死指标比 KPI 更重要**:命中即触发熔断(降级或回退),不应为 KPI 硬数字熬夜赶。

---

## 11. 下一步

如果路线图通过审视,M1 启动:

```
M1 第 1 天:
  上午:Tauri 2.x + Vue 3 + TS + Pinia + Vite 项目脚手架
        组件库 spike(Naive UI vs Element Plus)
  下午:决定组件库;开始 PetCanvas 基础架子
        IPC 框架草搭 + 第一个 ping/pong

M1 第 1 周末:
  桌宠透明窗口 + VRM momo 渲染 + 基础点击
  ChatService MVP(单 Provider)+ 流式渲染
  MemoryService + NicknameService 骨架

M1 第 2 周末:
  Onboarding 6 步可走通(灵魂宣誓页用 ADR-008 文案)
  U.1 / U.2 昵称 UI
  系统托盘 + 快捷键(Ctrl+Alt+Space)
  M1 → M2 出口检查
```

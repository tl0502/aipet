# AI桌宠 系统架构设计 v0.2

- 文档版本：v0.2（基于 v0.1 的**增量**版本，仅写变更部分；未变更章节请参阅 v0.1）
- 创建日期：2026-05-01
- 适用阶段：MVP 实现前（同步 PRD v0.5）
- 关联：
  - `2026-05-01-system-architecture-v0.1.md`（基线）
  - `需求设计/2026-05-01-ai-desktop-pet-prd-v0.5.md`
  - `需求设计/2026-05-01-ai-desktop-pet-flows-v0.5.md`

## 0. v0.1 → v0.2 变更摘要

| 类别 | 内容 |
|---|---|
| 新增 Rust 服务 | IdleDetector / LivingPetService / ProactiveCareService / BossKeyService / FileDropHandler / MilestoneService |
| 新增 SQLite 表 | `pet_runtime_state` / `milestones` / `proactive_care_log` |
| 新增 IPC 命令 | `living_pet.*` / `proactive_care.*` / `boss_key.*` / `file_drop.*` / `milestone.*` |
| 新增事件 | `pet.mood_changed` / `pet.wandering` / `proactive_care.fired` / `boss_key.toggled` / `milestone.reached` |
| 性能预算追加 | 自由活动期 GPU < 5%、主动关心 < 500ms、摸鱼切换 < 200ms |
| 安全设计 | 主动关心隐私边界（仅 OS 距上次输入毫秒数）、文件拖入内容不持久化 |

## 1. 整体架构（更新版）

```
┌────────────────────────────────────────────────────────────────┐
│                      AI 桌宠 (Tauri 2.x)                          │
│                                                                  │
│   ┌────────────────────────────────────────────────────────┐  │
│   │              WebView 前端层                              │  │
│   │  ┌─────────┐ ┌──────┐ ┌────┐ ┌────┐ ┌─────────────┐   │  │
│   │  │PetCanvas│ │ Chat │ │设置│ │工坊│ │FileDropHints│   │  │
│   │  │ +生命感 │ │      │ │    │ │    │ │ (动作泡泡)  │   │  │
│   │  └─────────┘ └──────┘ └────┘ └────┘ └─────────────┘   │  │
│   └─────────────────────┬──────────────────────────────────┘  │
│                         │ Tauri IPC                              │
│   ┌─────────────────────┴──────────────────────────────────┐  │
│   │                Rust 主进程 (Core Services)               │  │
│   │                                                           │  │
│   │  v0.1 既有：                                              │  │
│   │  ┌──────┐┌──────┐┌────────┐┌──────┐┌──────────┐         │  │
│   │  │ Chat ││ Task ││Persona ││Memory││ Crypto   │         │  │
│   │  └──────┘└──────┘└────────┘└──────┘└──────────┘         │  │
│   │  ┌─────────┐┌──────────┐┌─────────┐┌──────────┐          │  │
│   │  │Telemetry││Migration ││ Updater ││NetProbe  │          │  │
│   │  └─────────┘└──────────┘└─────────┘└──────────┘          │  │
│   │                                                           │  │
│   │  v0.2 新增：                                              │  │
│   │  ┌─────────────┐┌──────────────┐┌────────────────┐       │  │
│   │  │IdleDetector ││ LivingPet    ││ ProactiveCare  │       │  │
│   │  │(Win API)    ││ (mood/energy ││ (frequency cap │       │  │
│   │  │             ││  /wandering) ││  + tmpl pick)  │       │  │
│   │  └──────┬──────┘└──────┬───────┘└────────┬───────┘       │  │
│   │         │              │                 │                │  │
│   │  ┌──────┴──────┐┌──────┴───────┐┌────────┴───────┐       │  │
│   │  │ BossKey     ││ FileDrop     ││ Milestone      │       │  │
│   │  │ (hide/show) ││ (drop event  ││ (anniversary   │       │  │
│   │  │             ││  → bubbles)  ││  detect/fire)  │       │  │
│   │  └─────────────┘└──────────────┘└────────────────┘       │  │
│   └───────────────────────────────────────────────────────────┘  │
│                                                                  │
└────────────────────────────────────────────────────────────────┘
```

## 2. 新增服务详细设计

### 2.1 IdleDetector（键鼠空闲检测）

**职责**：周期性查询 OS"距上次键鼠输入毫秒数"，给上层提供"用户活跃 / 空闲"状态。

**实现**
- Windows API：`GetLastInputInfo`（不需要任何额外权限，不读输入内容）。
- 轮询间隔：30 秒（无需更细，主动关心阈值是分钟级）。
- **隐私承诺**：只返回 `idle_ms: u64`，**不接触应用名 / 窗口标题 / 输入内容**。

**接口**
```rust
pub trait IdleDetector {
    fn current_idle_ms(&self) -> u64;
    fn subscribe(&self) -> Receiver<IdleEvent>;
}

pub enum IdleEvent {
    UserActive { came_back_after_ms: u64 },
    UserIdleCrossThreshold { idle_ms: u64, threshold_min: u32 },
}
```

**与其他模块**
- 被 `ProactiveCareService` 订阅（驱动空闲关心）。
- 被 `LivingPetService` 订阅（精力值衰减/恢复）。
- 被 `MilestoneService` 间接使用（"用户回来"事件用于打卡触发）。

### 2.2 LivingPetService（生命感系统）

**职责**：管理桌宠的精力（energy）、心情（mood）、自由活动（wandering）。

**状态字段（瞬态，仅运行期内存 + 进程退出前持久化）**
```rust
pub struct PetRuntimeState {
    pub energy: u8,           // 0-100
    pub mood: Mood,           // happy | neutral | sleepy | focused | cozy
    pub last_interaction: DateTime<Utc>,
    pub wandering: bool,      // 当前是否在"逛桌面"
    pub user_disabled: HashSet<LivingFeature>, // 用户关闭项
}
```

**衰减/恢复规则**
- 每 5 分钟无互动：energy -= 2（地板 0）。
- 任意互动：energy = min(100, energy + 5)。
- energy < 30 → mood = sleepy。
- 互动后 10 分钟内 mood = happy。
- FOCUS 状态下 mood = focused（覆盖其他）。

**自由活动调度**
- 仅在主状态机 = `IDLE` 且 `wandering = false` 时可触发。
- 调度间隔：随机 `5-15 min` + 抖动（避免可预测）。
- 每次 wandering 持续 5-15s，移动距离 ≤ 屏宽 5%。
- 多屏不跨屏。
- 支持 `force_stop()` 立即归位（被高优状态打断时调用）。

**接口**
```rust
pub trait LivingPetService {
    fn get_state(&self) -> PetRuntimeState;
    fn record_interaction(&self, kind: InteractionKind);
    fn force_stop_wandering(&self);
    fn set_feature_enabled(&self, feature: LivingFeature, enabled: bool);
}
```

**事件**
- `pet.mood_changed { from, to }`
- `pet.wandering { phase: 'start'|'end', target_pos }`

### 2.3 ProactiveCareService（主动关心）

**职责**：根据 IdleDetector / 时间 / 番茄历史等本地信号，决定是否触发关心；做严格频率控制；从当前人格的离线模板池选文案。

**触发器（订阅源）**
1. `IdleEvent::UserIdleCrossThreshold` （阈值默认 90 min，可配）。
2. 每分钟时间检查：23:00 后键盘有活动持续 30 min → 候选。
3. 番茄/任务长时间未启动 + 高任务切换 → 候选（预留接口，M3 可不实现）。

**频率控制（硬约束）**
```rust
fn can_fire(&self, now: DateTime<Utc>) -> Result<(), DenyReason> {
    if self.in_quiet_hours(now) { return Err(QuietHours); }
    if self.last_fired_within(now, Duration::hours(2)) { return Err(TooSoon); }
    if self.daily_count(now) >= 4 { return Err(DailyCap); }
    if !self.user_enabled() { return Err(Disabled); }
    Ok(())
}
```

**文案选择**
- 调用 `PersonaService::get_offline_template_pool(category)`。
- category 由触发器决定：`empathy`（共情）/ `greeting`（问候）/ `gentle_remind`（温和提醒）。
- **不调用 LLM**：保证离线一致 + 节省 token。

**写入**
- 每次触发写一条 `proactive_care_log`（保留 7 天，用于频率统计与 KPI 11.10）。
- 同时通过 Telemetry 上报 `proactive_care_fired` 事件。

**接口**
```rust
pub trait ProactiveCareService {
    fn set_enabled(&self, enabled: bool);
    fn set_quiet_hours(&self, ranges: Vec<(NaiveTime, NaiveTime)>);
    fn set_idle_threshold_min(&self, n: u32);
    fn user_response(&self, log_id: i64, response: CareResponse); // clicked/replied/dismissed
}
```

### 2.4 BossKeyService（摸鱼模式）

**职责**：注册全局快捷键 `Ctrl+Shift+B`（默认），切换所有应用窗口的可见性。

**行为**
- 第一次按下：记录当前各窗口位置/可见性 → 全部 hide → 托盘图标变为"摸鱼中"。
- 第二次按下：恢复到记录的状态。
- 摸鱼期间：
  - Scheduler 继续运行（番茄计时不停）。
  - 硬提醒**不弹出**（缓冲到恢复时合并）。
  - NetworkProbe 继续。

**实现**
- 用 `tauri-plugin-global-shortcut` 注册。
- 维护 `BossKeyState { hidden: bool, snapshot: WindowSnapshot }`。
- hide/show 通过 Tauri WindowManager API 完成。

**接口**
```rust
pub trait BossKeyService {
    fn toggle(&self) -> BossKeyState;
    fn rebind(&self, new_shortcut: String) -> Result<()>;
    fn is_hidden(&self) -> bool;
}
```

### 2.5 FileDropHandler（文件拖入）

**职责**：接收 Tauri `tauri://file-drop` 事件，根据落点是否在桌宠 hitbox 内决定处理。

**流程**
1. WebView 收到 file-drop event（含路径列表 + 落点坐标）。
2. 前端判断坐标是否落在桌宠 hitbox。
3. 若是 → 通过 IPC 调 `file_drop.preflight(paths)`。
4. 主进程：
   - 校验文件类型（白名单：.txt/.md/.pdf；P1 增 .png/.jpg）。
   - 校验大小（默认上限 5MB，超过提示）。
   - 校验数量（一次最多 3 个）。
   - 提取文本（PDF 用 pdfium 或 pdf-parse，文本类直接读）。
5. 返回 `FileDropPreflight { ok, hint, available_actions }`。
6. 前端弹出 3 个动作泡泡（总结 / 解释 / 重命名建议）。
7. 用户点击 → 走 `chat.send` 链路，把文本作为单次上下文。

**关键约束**
- 文件文本**仅作单次会话上下文**，不写入 `messages.content` 长期存储。
- 超大文件（> 5MB 文本）二次确认。
- 路径/文件名等 PII 不上报埋点。

**接口**
```rust
pub trait FileDropHandler {
    async fn preflight(&self, paths: Vec<PathBuf>) -> FileDropPreflight;
    async fn handle_action(&self, action: FileAction, paths: Vec<PathBuf>) -> Result<MessageId>;
}
```

### 2.6 MilestoneService（跨日纪念日）

**职责**：检测各类纪念日里程碑并触发桌宠的应景反应。

**支持的里程碑（M0 决策周可微调）**
- 首次启动 +N 天：N ∈ { 7, 30, 100, 365 }
- 连续打卡某 reminder 满 N 天：N ∈ { 7, 30, 100 }
- 累计完成番茄数：N ∈ { 10, 50, 100, 500 }
- 累计完成待办数：N ∈ { 10, 100, 1000 }

**实现**
- 每次启动时检查 + 关键事件后检查（reminder_completed / pomodoro_completed / todo_completed）。
- 命中 → 写入 `milestones` 表（避免重复触发）+ 触发 `milestone.reached` 事件。
- 文案：从当前人格的 `# 离线模板 / ## 庆祝`（若无则降级到 `## 问候`）抽取。

**接口**
```rust
pub trait MilestoneService {
    fn check_now(&self) -> Vec<Milestone>;
    fn list_reached(&self) -> Vec<Milestone>;
}
```

## 3. 新增 SQLite Schema

```sql
-- 桌宠运行期状态（仅退出前快照，启动时恢复用）
-- 用于跨进程持久化"上次离开时桌宠的精力 / 心情"
CREATE TABLE pet_runtime_state (
  id INTEGER PRIMARY KEY CHECK (id = 1),  -- 单行表
  energy INTEGER NOT NULL DEFAULT 60,
  mood TEXT NOT NULL DEFAULT 'neutral',
  last_interaction_at TEXT NOT NULL,
  disabled_features TEXT NOT NULL DEFAULT '[]',  -- JSON array
  updated_at TEXT NOT NULL
);

-- 已触达里程碑（防止重复触发）
CREATE TABLE milestones (
  id TEXT PRIMARY KEY,            -- e.g. 'first_launch_7d'
  category TEXT NOT NULL,         -- 'first_launch'|'streak'|'pomodoro_count'|'todo_count'
  threshold INTEGER NOT NULL,
  reached_at TEXT NOT NULL,
  context TEXT                    -- JSON: 触达时的辅助信息
);

-- 主动关心日志（频率控制 + KPI）
CREATE TABLE proactive_care_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  fired_at TEXT NOT NULL,
  trigger TEXT NOT NULL,          -- 'idle'|'late_night'|'no_pomodoro_long'|'milestone'
  category TEXT NOT NULL,         -- 'empathy'|'greeting'|'gentle_remind'
  persona_id TEXT NOT NULL,
  template_idx INTEGER NOT NULL,  -- 命中模板的池内索引
  user_response TEXT,             -- 'clicked'|'replied'|'dismissed'|null
  responded_at TEXT
);
CREATE INDEX idx_pcl_fired ON proactive_care_log(fired_at);

-- 7 天保留策略：每次启动清理 fired_at < now - 7d 的记录
```

`schema_version` 升到 `2`，迁移文件 `002_v0_5_living_pet.sql`。

## 4. 新增 IPC

### 4.1 Commands

```ts
// 生命感
living_pet.get_state(): Promise<PetRuntimeState>
living_pet.set_feature_enabled(feature: 'wandering'|'mood_icon'|'energy', enabled: boolean): Promise<void>

// 主动关心
proactive_care.get_settings(): Promise<ProactiveCareSettings>
proactive_care.set_enabled(enabled: boolean): Promise<void>
proactive_care.set_quiet_hours(ranges: TimeRange[]): Promise<void>
proactive_care.set_idle_threshold_min(n: number): Promise<void>
proactive_care.respond(logId: number, response: 'clicked'|'replied'|'dismissed'): Promise<void>

// 摸鱼模式
boss_key.toggle(): Promise<{ hidden: boolean }>
boss_key.rebind(shortcut: string): Promise<void>
boss_key.is_hidden(): Promise<boolean>

// 文件拖入
file_drop.preflight(paths: string[]): Promise<FileDropPreflight>
file_drop.handle_action(action: 'summarize'|'explain'|'rename', paths: string[]): Promise<{ messageId: string }>

// 里程碑
milestone.list_reached(): Promise<Milestone[]>
milestone.check_now(): Promise<Milestone[]>
```

### 4.2 Events（主进程 → 前端）

```ts
'pet.mood_changed'      { from, to, reason }
'pet.wandering'         { phase: 'start'|'end', targetX, targetY }
'pet.energy_changed'    { value }                       // 节流，每 1 分钟最多一次
'proactive_care.fired'  { logId, category, message }
'boss_key.toggled'      { hidden }
'milestone.reached'     { id, category, message }
'file_drop.bubbles_shown' { paths, available }          // 前端通知主进程已展示泡泡
```

## 5. 模块依赖与启动顺序

```
启动期初始化序列：
  1. MigrationService.run()             // schema 1 → 2 升级
  2. CryptoService.init()
  3. PersonaService.load_active()
  4. MemoryService.init()
  5. TaskService.init()                  // 加载 reminders/todos/pomodoro_sessions
  6. LivingPetService.restore()          // 从 pet_runtime_state 还原
  7. IdleDetector.start()
  8. ProactiveCareService.start()        // 订阅 IdleDetector
  9. MilestoneService.check_now()        // 启动期一次检查
  10. BossKeyService.register_shortcut()
  11. FileDropHandler.bind_window()
  12. NetworkProbe.start()
  13. UpdaterService.check()             // 后台
  14. 进入主态，触发 'pet.state_changed → IDLE'
```

## 6. 数据流：主动关心（关键路径）

```
[OS] GetLastInputInfo()
      ↓ (轮询 30s)
[IdleDetector] 计算 idle_ms
      ↓ 跨阈值
[IdleEvent::UserIdleCrossThreshold]
      ↓
[ProactiveCareService::on_event]
      ↓ can_fire?
      ├── No → 静默
      └── Yes
          ↓
       [PersonaService::get_offline_template(category)]
          ↓
       [文案 + log_id]
          ↓
       insert proactive_care_log
          ↓
       emit 'proactive_care.fired' → 前端
          ↓
       [PetCanvas] 桌宠播一句 + Telemetry 上报
          ↓
       [User clicks/replies/dismiss]
          ↓
       'proactive_care.respond' → update log.user_response
```

## 7. 数据流：文件拖入（关键路径）

```
[Resource Manager] User drags file → onto pet hitbox
      ↓ Tauri file-drop event
[Frontend] hitbox check
      ↓
[IPC] file_drop.preflight(paths)
      ↓
[FileDropHandler] type/size/count check + extract text
      ↓
{ ok, available_actions: ['summarize', 'explain', 'rename'] }
      ↓
[Frontend] 显示动作泡泡
      ↓ user clicks 'summarize'
[IPC] file_drop.handle_action
      ↓
[ChatService.send] with file_text as single-turn context
      ↓
[Stream tokens → UI]
      ↓
（消息记录 messages 表只存用户的"动作选择"摘要，不存原文）
```

## 8. 性能预算（v0.2 追加）

| 项 | 预算 | 测量点 |
|---|---|---|
| 自由活动期 GPU | < 5% | DXGI Stats |
| IdleDetector 轮询开销 | < 0.1% CPU | profiler |
| 主动关心 fire 到展示 | < 500ms | 'proactive_care.fired' latency |
| 摸鱼切换 hide/show | < 200ms | command 耗时 |
| 文件拖入 preflight | < 3s（不含 LLM） | command 耗时 |
| Milestone check_now（启动期） | < 100ms | profiler |
| 总常驻内存（含 v0.5 新增） | ≤ 250MB（不变，新增模块在预算内） | OS Counter |

## 9. 安全 / 隐私（v0.2 追加）

### 9.1 主动关心隐私边界
- IdleDetector **仅调** `GetLastInputInfo`，**不调** `GetForegroundWindow` / 不读窗口标题。
- 这条承诺写入"灵魂宣誓"页与正式版数据策略。
- 单元测试覆盖：禁止任何对 `GetForegroundWindow / GetWindowText` 的调用（CI 静态检查）。

### 9.2 文件拖入数据
- 文件文本仅作单次会话上下文，不入 `messages.content`。
- 文件路径不上报埋点；仅上报 `mime_type / size_kb / action_chosen`。
- 用户拖入 .pdf 时，提取后的中间产物缓存在 `cache/file_extract/`，每次会话结束后立即删除。

### 9.3 摸鱼模式与提醒
- 摸鱼期间被缓冲的硬提醒在恢复时**合并展示**（不是逐条刷屏）。
- 提醒原始内容在缓冲期间不通过通知中心暴露给系统通知历史（避免任务管理器/通知历史泄漏）。

### 9.4 .soul.md "庆祝" 模板
- 模板池 `## 庆祝` 在导入 .soul.md 时同样走安全前缀校验。
- Milestone 触发时若人格无对应模板池，降级到 `## 问候`，不调 LLM 兜底（避免拖累节日等日子的延迟体验）。

## 10. 测试矩阵（v0.2 追加）

| 维度 | 取值 |
|---|---|
| 系统休眠 | 10 min / 1h / 8h（验证主动关心冷启不暴击） |
| 多用户机器 | 切换登录用户后 idle 计数应重置 |
| 长时间运行 | 24h / 72h 不漂移、不内存泄漏 |
| 频率控制 | 24h 内主动关心严格 ≤ 4 次 |
| 文件拖入边界 | 0 字节 / 5MB / 10MB / 错误 PDF / 100 个文件 |
| 摸鱼模式 | 在工坊 / 设置打开时切换无副作用 |
| 跨日打卡 | 时区偏移 / 系统时钟回调情况下不重复触发 |

## 11. M1-M4 实施任务对照（v0.2 增量）

| 里程碑 | v0.2 新增任务 |
|---|---|
| **M1** | LivingPetService 骨架 + 自由活动初版（PetCanvas 移动动画）、灵魂宣誓 Onboarding（文案接 PersonaService） |
| **M2** | 心情图标 + 精力衰减/恢复、`pet_runtime_state` 持久化、BossKeyService（摸鱼模式） |
| **M3** | IdleDetector + ProactiveCareService（含频率控制 + 安静时段）、FileDropHandler（文本类）、MilestoneService（首次 7/30 天） |
| **M4** | 性能调优（GPU 测试、24h 稳定性）、KPI 埋点验收（主动关心被采纳率 / 生命感关闭率） |

## 12. 风险与未决项（v0.2 追加）

| 风险 | 影响 | 缓解 |
|---|---|---|
| `GetLastInputInfo` 在 RDP / 远程桌面下行为不一致 | 主动关心误触发 | 检测会话类型 → RDP 场景下默认关闭模块 J |
| 自由活动可能被部分用户视为"乱动" | D7 关闭率超阈值 | 上线后观察 KPI 11.12，> 15% 时考虑默认关闭"逛桌面"子项 |
| Tauri file-drop 事件在某些版本桌面环境 inconsistent | 文件拖入功能跨版本断裂 | M0 选定的 Tauri 2.x 版本锁定；M3 集成测试覆盖 |
| Milestone 时区与跨日 | 重复 / 漏触发 | 统一使用本地时区 + 启动期幂等检查 + `milestones.id` PK 保证唯一 |

### 12.1 未决项（M0 决策周）
1. 自由活动的视觉风格（卡通跳跃 vs 柔顺漫步）— 影响美术工作量。
2. 心情图标的图形资源（emoji vs 自绘小图）— 影响一致性。
3. PDF 提取库选型（pdfium vs pdf-parse vs 自研）— 影响包体积。
4. 摸鱼模式默认快捷键最终值（`Ctrl+Shift+B` 是否与常用 IDE 冲突需测）。

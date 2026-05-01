# AI桌宠 系统架构设计 v0.4

- 文档版本：v0.4(M0 技术栈定版,基于 v0.3 增量)
- 创建日期：2026-05-01
- 适用阶段：MVP **实施前最终版**(同步 PRD v0.7)
- 关联：
  - `2026-05-01-system-architecture-v0.3.md`(基线)
  - `需求设计/2026-05-01-ai-desktop-pet-prd-v0.7.md`
  - `需求设计/2026-05-01-ai-desktop-pet-flows-v0.6.md`
  - `角色与人格/2026-05-01-persona-design-v0.2.md`
  - `M0-ADRs/`(14 项决策)

## 0. v0.3 → v0.4 变更摘要

**M0 决策周完成,以下技术选型从"待定"升至"已定"**:

| 选项 | v0.3 状态 | v0.4 决策 |
|---|---|---|
| 前端框架 | 待定(Vue 3 / React) | **Vue 3 + TypeScript + Pinia + Vite** (ADR-001) |
| 桌宠资源 | 待定(Live2D / Spine / PNG) | **Live2D Cubism 4 + Web SDK** (ADR-002) |
| 配饰叠加 | 待 ADR-002 决策 | **Live2D native 插槽**(`accessory_*_slot`) (ADR-003) |
| 物理交互动作 | 待定(数量) | **12 个核心动作 ID** (ADR-004) |
| LLM Provider 默认 | 待定 | **零默认 + 6 个 preset** (ADR-005) |
| 音效播放后端 | M0 决策已定为前端 HTML5 Audio | 同 v0.3 |
| 音效来源 | 待定 | **自录(产品配音)** (ADR-010) |
| 装扮 schema | 待 schema 范式 | **结构化对象 + JSON 列存储** (ADR-011) |
| 游戏 UI 承载 | 待定 | **独立游戏舱 GameRoom 窗口** (ADR-012) |
| 代码签名 | 待 EV 评估 | **M5 灰度不签名,user education** (ADR-013) |
| 本地小模型 P1-R3 | 待评估 | **调用本地 Ollama** (ADR-014) |

未列出的章节按 v0.3 原文不变。

## 0.1 关键技术栈速查(M1 实施第一天即用)

```
┌─ 主进程 (Rust)
│   - Tauri 2.x
│   - tokio (async runtime)
│   - rusqlite (SQLite + WAL + JSON1)
│   - reqwest (HTTP 客户端,LLM Provider)
│   - tauri-plugin-global-shortcut (摸鱼/对话快捷键)
│   - tauri-plugin-updater (M5 + 灰度期不签名)
│   - windows crate (DPAPI / GetLastInputInfo / RAWINPUT)
│
├─ 前端 (WebView2)
│   - Vue 3.4+ + TypeScript 5+
│   - Pinia (状态管理)
│   - Vite (构建)
│   - 组件库:Naive UI 或 Element Plus(M1 第一天 spike 后定)
│   - Live2D Cubism 4 Web SDK (桌宠渲染)
│   - HTML5 Audio (声音表情播放)
│
├─ 数据层
│   - SQLite + WAL
│   - schema_version: 3 (M5 后可升 4)
│   - Windows DPAPI 加密 secrets
│
└─ 资源
    - personas/_builtin/{momo,joker,coach}.soul.md (ADR-009)
    - assets/voice_packs/default/*.ogg (12-20 条,自录)
    - assets/accessories/*.png + manifest.json
    - assets/game_scenes/{story_relay,cafe_owner}.yaml (ADR-007)
    - assets/safety/prefix_v1.txt + 地区补充 (ADR-006)
    - assets/onboarding/soul_pledge_v1.txt (ADR-008)
```

## 0.2 LLM Provider Preset 清单(ADR-005 落地)

```yaml
presets:
  - id: openai
    name: OpenAI
    base_url: https://api.openai.com/v1
    model_default: gpt-4o-mini
  - id: deepseek
    name: DeepSeek
    base_url: https://api.deepseek.com
    model_default: deepseek-chat
  - id: moonshot
    name: Moonshot (Kimi)
    base_url: https://api.moonshot.cn/v1
    model_default: moonshot-v1-8k
  - id: qwen
    name: 通义千问
    base_url: https://dashscope.aliyuncs.com/compatible-mode/v1
    model_default: qwen-turbo
  - id: ollama
    name: 本地 Ollama (P1-R3 推荐 qwen2.5:3b)
    base_url: http://localhost:11434/v1
    model_default: qwen2.5:3b
  - id: custom
    name: 自定义...
    base_url: ""
    model_default: ""
```

Onboarding Step 6 不强制要求 API Key(ADR-005),首次唤起对话失败时再引导。

## 0.3 GameRoom 窗口规格(ADR-012)

新增 Tauri 窗口:

```rust
WindowBuilder::new(app, "game_room", WindowUrl::App("game.html".into()))
    .title("和我玩…")
    .inner_size(480.0, 600.0)
    .resizable(false)
    .visible(false)  // 默认隐藏,IPC `game.start` 后显示
    .transparent(false)
    .decorations(true)
    .build()?;
```

- 位置:首次显示居中,后续记忆上次位置(`game_room_position` 配置项)
- 关闭行为:点 X 等价于 `game.end(saveAsDiary?)`(弹出确认)
- 桌宠窗口在游戏期保持可见(IN_GAME 叠加态),宠物 mood 可受游戏内反馈影响

## 1. 整体架构（v0.3 更新）

```
┌──────────────────────────────────────────────────────────────────┐
│                       AI 桌宠 (Tauri 2.x)                          │
│                                                                    │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                  WebView 前端层                             │  │
│   │  ┌──────────┐ ┌──────┐ ┌────┐ ┌────┐ ┌─────┐ ┌─────┐    │  │
│   │  │PetCanvas │ │ Chat │ │设置│ │工坊│ │装扮 │ │游戏 │    │  │
│   │  │+ hitbox  │ │      │ │    │ │    │ │工坊 │ │舱   │    │  │
│   │  │+ accessor│ │      │ │    │ │    │ │     │ │     │    │  │
│   │  └──────────┘ └──────┘ └────┘ └────┘ └─────┘ └─────┘    │  │
│   └────────────────────────┬─────────────────────────────────┘  │
│                            │ Tauri IPC                              │
│   ┌────────────────────────┴─────────────────────────────────┐  │
│   │              Rust 主进程 (Core Services)                    │  │
│   │                                                              │  │
│   │  v0.1/v0.2 既有：                                            │  │
│   │  Chat / Task / Persona / Memory / Crypto / Telemetry        │  │
│   │  Migration / Updater / NetProbe / IdleDetector              │  │
│   │  LivingPet / ProactiveCare / BossKey / FileDrop / Milestone │  │
│   │                                                              │  │
│   │  v0.3 新增：                                                 │  │
│   │  ┌────────────────┐ ┌────────────────┐ ┌──────────────┐   │  │
│   │  │ Interaction    │ │ VoiceEffect    │ │ Wardrobe     │   │  │
│   │  │ Router         │ │ Player         │ │ Service      │   │  │
│   │  │ (hitbox →action│ │ (本地 audio +  │ │ (配饰/皮肤   │   │  │
│   │  │  → 反应)       │ │  静音时段)     │ │  +付费预埋)  │   │  │
│   │  └────┬───────────┘ └────────────────┘ └──────────────┘   │  │
│   │       │                                                       │  │
│   │  ┌────┴───────────────────────────────────────────────┐     │  │
│   │  │ GameEngine                                           │     │  │
│   │  │  ├ LocalGameRunner (RPS / GuessNumber / WordChain) │     │  │
│   │  │  └ LLMGameRunner   (StoryRelay / RolePlay)        │     │  │
│   │  │  → 共用 SecurityGuard + token 上限                 │     │  │
│   │  └─────────────────────────────────────────────────────┘     │  │
│   │                                                              │  │
│   │  ┌─────────────────┐                                         │  │
│   │  │ Nickname        │   （扩展 MemoryService 的轻量 facade）   │  │
│   │  │ Service         │                                          │  │
│   │  └─────────────────┘                                         │  │
│   └──────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

## 2. 新增 Rust 服务详细设计

### 2.1 InteractionRouter（物理交互路由）

**职责**：接收前端 PetCanvas 的物理交互事件（点击/双击/长按/拖拽/键鼠协同），按照 hitbox 与人格定义路由到具体反应（动作 + 心情变化 + 可选音效）。

**接口**
```rust
pub trait InteractionRouter {
    fn dispatch(&self, evt: InteractionEvent) -> Vec<Reaction>;
    fn record_drag_count(&self) -> u32;  // 用于 N.3 抗议判定
    fn reset_drag_state(&self);          // 5 秒后由调度器调用
}

pub enum InteractionEvent {
    Click { hitbox: Hitbox, modifier: Option<Modifier> },
    DoubleClick { hitbox: Hitbox },
    LongPress { hitbox: Hitbox, duration_ms: u32 },
    RightClick { hitbox: Hitbox },           // 弹快捷菜单
    Drag { distance_px: f32, duration_ms: u32 },
    KeyboardBurst { events_per_min: u32 },   // 来自 IdleDetector 的扩展信号
}

pub enum Hitbox { Head, Body, Tail, Edge }

pub struct Reaction {
    pub action: String,        // PetCanvas 播放的动作 ID
    pub mood_delta: Option<MoodChange>,  // 短暂心情变化
    pub voice_id: Option<String>,        // 触发声音表情
    pub duration_ms: u32,
}
```

**抗议规则（N.3）**
- 维护 `drag_events: VecDeque<Instant>`，只保留最近 30 秒。
- 短时间内 ≥ 3 次拖动 → 返回 `protest` Reaction（动作 + 短 mood 抖动）。
- mood 变化标记为 `transient: true`，5 秒后由 `LivingPetService` 自动 revert，**不持久化到 `pet_runtime_state`**。

**键鼠协同（N.4）**
- 订阅 `IdleDetector` 的 `KeyboardBurstEvent`（IdleDetector v0.3 新加事件类型，见 §4.1）。
- 频率上限：每小时 1 次。
- 用户可在设置中关闭 N.4，关闭后不订阅。

### 2.2 VoiceEffectPlayer（声音表情播放）

**职责**：播放本地音效；管理静音时段；按人格切换音效包。

**接口**
```rust
pub trait VoiceEffectPlayer {
    fn play(&self, voice_id: &str, persona_id: &str) -> Result<()>;
    fn set_global_mute(&self, mute: bool);
    fn set_quiet_hours(&self, ranges: Vec<(NaiveTime, NaiveTime)>, weekdays: Vec<Weekday>);
    fn set_volume(&self, vol: u8);  // 0-100
    fn list_packs(&self) -> Vec<VoicePackMeta>;
    fn install_pack(&self, path: PathBuf) -> Result<VoicePackMeta>;  // P2，本期占位
}
```

**静音判定**
```rust
fn is_muted_now(&self) -> bool {
    if self.global_mute { return true; }
    let now = Local::now();
    if !self.quiet_weekdays.contains(&now.weekday()) { return false; }
    self.quiet_ranges.iter().any(|(s, e)| in_range(now.time(), *s, *e))
}
```

**默认配置**
- `global_mute = false`
- `quiet_weekdays = [Mon..=Fri]`
- `quiet_ranges = [(09:00, 18:00)]`

**音效播放后端**
- 使用 `rodio` crate（Rust 原生）或 Tauri 端 HTML5 Audio（前端播放）。
- M0 决策：**前端 HTML5 Audio**（更简单、零额外依赖、自动跟随系统音量），主进程仅调度。
- 触发 `IPC: voice.play(voice_id, persona_id)` → 前端从 `assets/voice_packs/<pack>/<voice_id>.ogg` 加载播放。

**隐私**
- 不调麦克风、不录音、不上传。CI 静态扫描禁止 `MediaDevices` / `getUserMedia`。

### 2.3 WardrobeService（装扮系统）

**职责**：管理配饰库、用户已装扮项、节气推送、付费预埋。

**接口**
```rust
pub trait WardrobeService {
    fn list_inventory(&self) -> Vec<AccessoryMeta>;
    fn equip(&self, accessory_ids: Vec<String>) -> Result<()>;       // 替换当前佩戴
    fn unequip_all(&self) -> Result<()>;
    fn current_equipped(&self) -> Vec<AccessoryMeta>;
    fn check_seasonal(&self) -> Vec<SeasonalSuggestion>;             // 节气推送候选
    fn record_seasonal_decision(&self, suggestion_id: &str, accepted: bool);
    fn unlock(&self, accessory_id: &str, reason: UnlockReason);      // 里程碑解锁/购买等
}

pub struct AccessoryMeta {
    pub id: String,
    pub name: String,
    pub category: String,        // 'hat'|'scarf'|'glasses'|'skin'|...
    pub tier: Tier,              // 'free' | 'paid'
    pub unlock: UnlockSpec,      // 'always'|'milestone:xxx'|'date_range:[s,e]'|'purchase:sku'
    pub asset_path: String,      // 相对 assets/accessories/
    pub anchor: AccessoryAnchor, // {x, y, scale, z_index, layer_target}
    pub locked: bool,
}
```

**节气推送**
- 启动时 + 每天 00:01 触发 `check_seasonal()`。
- 检查每个 `unlock = date_range` 的 accessory，若当前日期落在范围内：
  - 检查 `wardrobe_decisions` 表（`accessory_id, year, decision`），如果用户当年已"拒绝"则不推送。
  - 否则通过 `ProactiveCareService`（特殊 category='wardrobe_suggest'）触发推送，**不占用主动关心 4 次/日额度**（与 milestone 同等独立）。

**付费预埋**
- `tier = 'paid'` 的 accessory 在 MVP 中**全部不返回给前端**（`list_inventory` 过滤）。
- 商店 UI 在 P2-R3 才上线，不影响本期。

**人格 ↔ 装扮**
- 人格切换不影响装扮（装扮 owner 是桌宠实体，不是人格）。
- 导入 .soul.md 时，若有 `accessories: [...]` 字段：
  - 对每个 ID 检查是否已在 `accessories_inventory`（已解锁）。
  - 弹"是否套用 .soul.md 的默认装扮？"，用户确认才 equip。

### 2.4 GameEngine（小游戏引擎）

**职责**：管理小游戏会话；协调本地规则游戏 / LLM 驱动游戏；token 上限；安全前缀；与日记功能数据契约。

**接口**
```rust
pub trait GameEngine {
    fn start(&self, game_id: &str) -> Result<GameSessionId>;
    fn submit(&self, session_id: GameSessionId, input: GameInput) -> Result<GameOutput>;
    fn end(&self, session_id: GameSessionId, save_as_diary: bool) -> Result<()>;
    fn list_available(&self) -> Vec<GameMeta>;  // 含离线/在线分类与可用性
}

pub struct GameMeta {
    pub id: String,
    pub kind: GameKind,         // 'local' | 'llm'
    pub name: String,
    pub min_persona_features: Vec<String>,   // 例如要求人格有 ## 调侃 模板池
    pub offline_available: bool,
}
```

**LocalGameRunner**
- 内置三个游戏的纯逻辑。
- 输出 `GameOutput { text, action_hint }`，`text` 中嵌入 `{persona_banter}` 占位符，由 PersonaService 从离线模板池抽样填充（人格化）。

**LLMGameRunner**
- 拼装 prompt：
  ```
  [安全前缀] 
  [当前人格 system prompt]
  [游戏场景 system prompt]（来自白名单 yaml）
  [用户记忆摘要]
  [游戏会话历史]
  [本轮用户输入]
  ```
- 单次会话 `max_tokens` 累计，>= 2000 时返回 `out_of_budget` 让前端 friendly 收尾。
- 离线时 `start()` 直接拒绝并返回 `offline_unavailable`。
- **拒答时人格化**：当 `SecurityGuard` 在 LLM 输出后扫描命中违禁，替换为人格化的"游戏内拒答模板"（每个游戏场景在 `game_scenes/<id>.yaml` 里定义至少 3 条）。

**会话隔离**
- 游戏会话不写入 `messages` 表，写入 `game_sessions`（30 天保留，未保存则结束删除）。
- "保留为日记片段" → 写入 `diary_drafts` 表（v0.3 新增，等 P1-R1 桌宠日记功能消费）。

### 2.5 NicknameService（昵称管理）

**职责**：管理桌宠昵称（覆盖 .soul.md `name`）和用户昵称（注入 prompt 与离线模板）。

**接口**
```rust
pub trait NicknameService {
    fn get_pet_nickname(&self) -> Option<String>;
    fn set_pet_nickname(&self, name: Option<String>);  // None = 恢复 .soul.md 默认
    fn get_user_nickname(&self) -> Option<String>;
    fn set_user_nickname(&self, name: Option<String>);
    fn get_user_nickname_for_prompt(&self) -> String;  // 注入 LLM 时使用
    fn get_user_nickname_for_template(&self) -> String;  // 离线模板 {username} 占位符
}
```

**实现要点**
- 是 MemoryService 上的轻量 facade，数据落在 `nicknames` 表。
- 切换人格时：
  - 用户昵称（user_nickname）保持。
  - 桌宠昵称（pet_nickname）重置为新人格的 .soul.md `name`，但保留"上一次的桌宠昵称"在 `nicknames.previous_pet` 字段，UI 提供"恢复"按钮。
- 长度校验：≤ 16 字符，去除控制字符。

## 3. 既有服务扩展

### 3.1 LivingPetService 扩展（日常时段表）

**新增字段**
```rust
pub struct DailySchedule {
    pub enabled: bool,             // 默认 true
    pub slots: Vec<TimeSlot>,
}

pub struct TimeSlot {
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub action_pool: Vec<String>,  // 该时段可触发的动作 ID
    pub max_per_slot: u32,         // 默认 1
}
```

**默认时段**
- 06:00-09:00：`['stretch', 'rub_eyes']`
- 11:30-13:30：`['yawn']`
- 14:00-17:00：`['quiet_lay', 'look_up']`（精力 < 30 优先 quiet_lay）
- 22:00-00:00：`['cozy_watch']`，自动设 mood=cozy
- 00:00-06:00：silent（无动作）

**调度合并**
- 与"自由活动"共享调度器：每个 5-15 分钟周期，先检查时段表是否有候选动作（最高优先级），无则按概率走自由活动。
- 不在 FOCUS / BOSS_KEY_HIDDEN 触发。

### 3.2 MilestoneService 扩展（用户纪念日）

**新增规则集**
```rust
pub enum MilestoneCategory {
    FirstLaunch(u32),        // v0.5
    Streak(String, u32),     // v0.5
    PomodoroCount(u32),      // v0.5
    TodoCount(u32),          // v0.5
    UserAnniversary(String), // v0.6 新增：键名（'birthday'|'work_start'|'custom_*'）
}
```

**user_anniversaries 表**
- 用户在设置中添加自定义纪念日（生日 / 入职 / 自定义名字 + 日期）。
- 纪念日触达时，MilestoneService 触发 `category='celebration'` 的关心，从当前人格离线模板 `## 庆祝` 抽样。
- **每条纪念日年度重复**：检测时按 `MM-DD` 匹配，触达后在 `milestones.id = 'anniversary_<key>_<YYYY>'` 防重复。
- 时区策略：使用本地时区，每天 00:01 调度 + 启动期补检查。

### 3.3 IdleDetector 扩展（键鼠 burst）

**新增事件类型**
```rust
pub enum IdleEvent {
    UserActive { came_back_after_ms: u64 },
    UserIdleCrossThreshold { idle_ms: u64, threshold_min: u32 },
    KeyboardBurst { events_per_min: u32, duration_s: u32 },  // v0.3 新增
}
```

**实现**
- Windows 仅有 `GetLastInputInfo` 不能区分键盘/鼠标；可用 `RAWINPUT`（需要注册原始输入设备）来计数键盘事件次数（不读内容）。
- 评估成本：M2 决策。若 RAWINPUT 实现复杂，**降级**为"快速 idle 切换检测"（每 3 秒 idle 重置且 events 较多）作为近似信号。
- **隐私边界**：仅累加 events count，不记录 keycode、不记录窗口、不记录顺序。

## 4. 新增 SQLite Schema

```sql
-- 装扮库存（用户已解锁的所有 accessory）
CREATE TABLE accessories_inventory (
  id TEXT PRIMARY KEY,           -- accessory_id（与资源文件对应）
  unlocked_at TEXT NOT NULL,
  unlock_reason TEXT NOT NULL,    -- 'always'|'seasonal'|'milestone:xxx'|'purchase:sku'|'gift'
  is_equipped INTEGER NOT NULL DEFAULT 0
);

-- 节气推送决策记忆（避免重复打扰）
CREATE TABLE wardrobe_decisions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  accessory_id TEXT NOT NULL,
  year INTEGER NOT NULL,
  decision TEXT NOT NULL,         -- 'accepted'|'declined'
  decided_at TEXT NOT NULL,
  UNIQUE(accessory_id, year)
);

-- 音效包元信息
CREATE TABLE voice_packs (
  id TEXT PRIMARY KEY,            -- e.g. 'default'|'cute_v1'|'user_imported_xxx'
  name TEXT NOT NULL,
  source TEXT NOT NULL,           -- 'builtin'|'user_imported'
  manifest_path TEXT NOT NULL,
  installed_at TEXT NOT NULL
);

-- 音效配置（单行）
CREATE TABLE voice_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  global_mute INTEGER NOT NULL DEFAULT 0,
  volume INTEGER NOT NULL DEFAULT 50,
  quiet_weekdays TEXT NOT NULL DEFAULT '[1,2,3,4,5]',  -- JSON
  quiet_ranges TEXT NOT NULL DEFAULT '[["09:00","18:00"]]', -- JSON
  updated_at TEXT NOT NULL
);

-- 游戏会话
CREATE TABLE game_sessions (
  id TEXT PRIMARY KEY,            -- ULID
  game_id TEXT NOT NULL,
  kind TEXT NOT NULL,             -- 'local'|'llm'
  started_at TEXT NOT NULL,
  ended_at TEXT,
  result TEXT,                    -- JSON：游戏特定结果
  saved_as_diary INTEGER NOT NULL DEFAULT 0,
  total_tokens INTEGER NOT NULL DEFAULT 0  -- 仅 LLM 游戏
);

CREATE TABLE game_session_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id TEXT NOT NULL,
  role TEXT NOT NULL,             -- 'user'|'assistant'|'system'
  content TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (session_id) REFERENCES game_sessions(id) ON DELETE CASCADE
);

-- 日记草稿（来自游戏"保留为日记片段"，等 P1-R1 桌宠日记功能消费）
CREATE TABLE diary_drafts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source TEXT NOT NULL,           -- 'game:<game_id>'|'manual'|...
  content TEXT NOT NULL,
  created_at TEXT NOT NULL,
  consumed INTEGER NOT NULL DEFAULT 0
);

-- 用户纪念日
CREATE TABLE user_anniversaries (
  key TEXT PRIMARY KEY,           -- 'birthday'|'work_start'|'custom_<ulid>'
  display_name TEXT NOT NULL,
  date_md TEXT NOT NULL,          -- 'MM-DD'，年度重复
  created_at TEXT NOT NULL
);

-- 昵称（单行）
CREATE TABLE nicknames (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  pet_nickname TEXT,              -- nullable，null 表示用 .soul.md 默认
  pet_nickname_previous TEXT,     -- 上次自定义值，用于"恢复"按钮
  user_nickname TEXT,
  updated_at TEXT NOT NULL
);
```

`schema_version` 升到 `3`，迁移文件 `003_v0_6_interaction_extension.sql`。

## 5. 新增 IPC

### 5.1 Commands

```ts
// 物理交互
interaction.dispatch(evt: InteractionEvent): Promise<Reaction[]>
interaction.set_n4_enabled(enabled: boolean): Promise<void>

// 声音
voice.play(voiceId: string): Promise<void>            // persona_id 由主进程从当前激活人格自动注入
voice.set_global_mute(mute: boolean): Promise<void>
voice.set_quiet_hours(ranges: TimeRange[], weekdays: Weekday[]): Promise<void>
voice.set_volume(vol: number): Promise<void>
voice.list_packs(): Promise<VoicePackMeta[]>

// 装扮
wardrobe.list_inventory(): Promise<AccessoryMeta[]>
wardrobe.equip(ids: string[]): Promise<void>
wardrobe.unequip_all(): Promise<void>
wardrobe.current_equipped(): Promise<AccessoryMeta[]>
wardrobe.dismiss_seasonal_for_year(suggestionId: string): Promise<void>

// 小游戏
game.list_available(): Promise<GameMeta[]>
game.start(gameId: string): Promise<{ sessionId: string }>
game.submit(sessionId: string, input: GameInput): Promise<GameOutput>
game.end(sessionId: string, saveAsDiary: boolean): Promise<void>

// 昵称
nickname.get(): Promise<{ pet?: string; user?: string; pet_previous?: string }>
nickname.set_pet(nickname: string | null): Promise<void>
nickname.set_user(nickname: string | null): Promise<void>
nickname.restore_pet_previous(): Promise<void>

// 用户纪念日
anniversary.list(): Promise<UserAnniversary[]>
anniversary.add(payload: { displayName: string; dateMd: string; key?: string }): Promise<void>
anniversary.remove(key: string): Promise<void>
```

### 5.2 Events（主进程 → 前端）

```ts
'pet.interaction_reacted'   { hitbox, action_id, voice_id?, mood_change? }
'pet.protest_triggered'     { drag_count, will_revert_in_ms }
'pet.daily_action'          { time_slot, action_id }
'voice.played'              { voice_id, pack_id }
'voice.muted_by_quiet_hour' { voice_id, reason: 'quiet_hour'|'global_mute' }
'wardrobe.changed'          { equipped: AccessoryMeta[] }
'wardrobe.seasonal_suggest' { suggestion_id, accessory_id, accept_url }
'game.session_started'      { session_id, game_id, kind }
'game.token_budget_warning' { session_id, used, limit }
'game.session_ended'        { session_id, saved_as_diary, total_tokens }
'milestone.user_anniversary' { key, display_name }
'nickname.changed'          { which: 'pet'|'user', value }
```

## 6. 模块依赖与启动顺序（v0.3 更新）

```
启动期初始化序列（v0.3 增量项 *）：
  1. MigrationService.run()                  // schema 2 → 3 升级
  2. CryptoService.init()
  3. PersonaService.load_active()
  4. MemoryService.init()
  5. NicknameService.init()                  *
  6. TaskService.init()
  7. LivingPetService.restore() + load_daily_schedule  *
  8. IdleDetector.start() (含 keyboard burst hook *)
  9. ProactiveCareService.start()
  10. MilestoneService.check_now()           // 含 user_anniversaries 检查 *
  11. BossKeyService.register_shortcut()
  12. FileDropHandler.bind_window()
  13. WardrobeService.init() + check_seasonal()  *
  14. VoiceEffectPlayer.init()                *
  15. InteractionRouter.init()                *
  16. GameEngine.init()                       *
  17. NetworkProbe.start()
  18. UpdaterService.check()
  19. 进入主态
```

## 7. 关键数据流：物理交互（v0.3 新增）

```
[Frontend PetCanvas] 用户点击桌宠头部
 ↓ 解析 hitbox
[IPC] interaction.dispatch({ Click, Hitbox::Head, ... })
 ↓
[InteractionRouter]
 ├── 查询当前 persona 的反应配置（默认 + 人格自定义）
 ├── 决定 Reaction { action: 'head_pat', mood_delta: +happy(2s), voice_id: 'eheh' }
 └── 返回 Reaction[]
 ↓
[Frontend]
 ├── PetCanvas 播放 'head_pat' 动作
 ├── 心情图标短暂变 happy
 └── IPC: voice.play('eheh')
       ↓
       [VoiceEffectPlayer]
        ├── is_muted_now()? 是 → 静默 + emit 'voice.muted_by_quiet_hour'
        └── 否 → 前端 HTML5 Audio 加载并播放
 ↓
[Telemetry] 'pet.interaction_reacted' { hitbox: 'head', action: 'head_pat' }
```

## 8. 关键数据流：LLM 小游戏（v0.3 新增）

```
[Frontend Game UI] 用户点击 "开始故事接龙"
 ↓
[IPC] game.start('story_relay')
 ↓
[GameEngine.start]
 ├── 检查网络状态（offline → 拒绝）
 ├── 检查游戏 meta（kind='llm'）
 ├── 创建 game_session 记录
 └── 返回 sessionId
 ↓
emit 'game.session_started'
 ↓
用户输入 "从前有一只小猫..."
 ↓
[IPC] game.submit(sessionId, { text: ... })
 ↓
[LLMGameRunner]
 ├── 拼装 prompt：
 │    [安全前缀]
 │    [当前人格 system prompt]
 │    [game_scenes/story_relay.yaml 的场景定义]
 │    [用户记忆摘要]
 │    [本会话历史]
 │    [本轮输入]
 ├── 调 LLMProvider.chat_stream
 ├── 收集流式输出 → SecurityGuard 实时扫描
 │    ├── 命中违禁 → 替换为 game_scenes/story_relay.yaml.refusals 的人格化拒答
 │    └── 通过 → 输出
 ├── 累计 total_tokens
 │    └── >= 2000 → 返回 out_of_budget
 └── 写 game_session_events
 ↓
[Frontend] 流式渲染
 ↓
... 多轮 ...
 ↓
[用户点 "我累了"]
 ↓
[IPC] game.end(sessionId, saveAsDiary=true)
 ↓
[GameEngine.end]
 ├── 写 game_sessions.ended_at + result
 ├── 若 saveAsDiary → 生成日记片段写 diary_drafts
 └── 删除超过 30 天的 game_sessions
 ↓
emit 'game.session_ended'
```

## 9. 性能预算（v0.3 追加）

| 项 | 预算 | 测量点 |
|---|---|---|
| 物理交互响应（点击 → 视觉） | < 100ms | PetCanvas paint timestamp |
| 装扮切换 | < 500ms | wardrobe.equip command 耗时 + 渲染 |
| 声音播放延迟 | < 50ms | voice.play 触发 → audio play start |
| 本地游戏每轮 | < 50ms | game.submit 耗时（local kind） |
| LLM 游戏首 token | p50 < 1.5s | 同对话标准 |
| 总常驻内存（含 v0.6 新增） | ≤ 250MB（不变；新增 < 30MB） | OS Counter |
| 内置音效包总大小 | ≤ 5MB | 安装包审计 |
| 内置装扮资源总大小 | ≤ 10MB | 安装包审计 |
| 总安装包目标 | ≤ 80MB | release 产物 |

## 10. 安全 / 隐私（v0.3 追加）

### 10.1 LLM 游戏的安全前缀
- `LLMGameRunner` 的 prompt 拼装中，安全前缀**始终**位于游戏场景定义之前。
- 游戏场景 yaml 中的 `system_prompt` 字段不能包含"忽略安全规则"等指令；M0 决策时由产品 + 法务复审。
- LLM 输出后的 `SecurityGuard` 二次扫描仍然生效。

### 10.2 物理交互不持久化情绪
- `Reaction.mood_delta` 仅短暂展示，标记为 `transient: true`。
- 5 秒后由 `LivingPetService.tick()` 自动 revert 到 base mood。
- **不写入** `pet_runtime_state.mood`。

### 10.3 声音表情隐私
- 不调麦克风、不录音。CI 静态扫描黑名单：`getUserMedia` / `MediaRecorder` / `AudioContext.createMediaStreamSource`。
- 所有音效本地播放，不上传任何音频。

### 10.4 装扮决策隐私
- 节气推送的"接受/拒绝"仅本地记忆。
- 埋点上报 `wardrobe_seasonal_decision` 仅含 `accessory_category` 和 `decision`，不含具体 ID。

### 10.5 游戏会话隐私
- 游戏会话**不写入** `messages` 表。
- 仅当用户主动"保留为日记片段" → 摘要后写 `diary_drafts`。
- 30 天后未保存的 game_sessions 自动清理。

## 11. 测试矩阵（v0.3 追加）

| 维度 | 取值 |
|---|---|
| 物理交互 hitbox 覆盖 | 头/身体/尾/边缘 各 100 次点击 + 双击/长按/右键 |
| 声音工作时段 | 09:00-18:00 工作日 / 周末 / 自定义时段 |
| 装扮叠加 | 0-3 件配饰组合，节气日 / 非节气日 |
| LLM 游戏安全 | 5 类违禁尝试（自伤/暴力/越权/角色越界/医疗诊断） |
| LLM 游戏 token 上限 | 故意拉长会话验证收尾 |
| 节气年度重复 | 模拟系统时钟跨年 |
| 用户纪念日时区 | 时区跨日 / 系统时区切换 |
| 昵称切换人格 | 自定义昵称 → 切换人格 → 恢复 |
| 拖动抗议非持久化 | 拖动 5 次 → 5 秒后 mood 已 revert |

## 12. M1-M5 实施任务对照（v0.3 增量）

| 里程碑 | v0.3 新增任务 |
|---|---|
| **M1** | NicknameService MVP + 昵称设置 UI |
| **M2** | InteractionRouter（hitbox 解析 + 反应配置 + 抗议规则） |
| **M3** | LivingPetService 日常时段表 + DailySchedule 调度集成 |
| **M4** | WardrobeService（配饰 + 1 套节气皮肤） + VoiceEffectPlayer（默认音效包 + 静音逻辑） + 用户纪念日 UI 与触发 |
| **M5** | GameEngine（LocalGameRunner 3 个 + LLMGameRunner 1-2 个 + 安全前缀复用 + token 上限） + 全套 KPI 11.15-11.20 埋点 |

## 13. 风险与未决项（v0.3 追加）

| 风险 | 影响 | 缓解 |
|---|---|---|
| Live2D / Spine 不支持运行时叠加 PNG sticker | O 装扮无法实现 | M0 必须确认；否则降级为"整套皮肤"切换（无配饰叠加） |
| RAWINPUT 实现成本高 | N.4 键鼠协同延期 | 降级为"快速 idle 切换"近似信号；不影响其他 N 子项 |
| LLM 游戏 token 月度成本不可控 | 用户账单爆炸 | 单次会话 2000 token 上限 + 设置可见消耗统计 + 告警 |
| 节气推送被误认为打扰 | 装扮使用率指标不达标 | 默认每节气仅推 1 次；用户拒绝当年不再推；年度记忆 |
| 物理交互动作资源工作量大 | 美术延期 | M0 决策动作清单上限（≤ 12 个独立动作 ID）；优先复用现有动作组合 |

### 13.1 未决项

✅ **全部清空**(2026-05-01,M0 决策周完成)。原 7 项未决项见 ADR-002/003/004/007/010/011/012,全部 Accepted。

仅保留 1 项**实施期 spike**(不阻塞 M0):
- **RAWINPUT 实现可行性**:M2 内决断,若实现成本高则降级为"快速 idle 切换"近似信号(不影响其他 N 子项)。

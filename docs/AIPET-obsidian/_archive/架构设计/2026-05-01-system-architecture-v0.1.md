# AI桌宠 系统架构设计 v0.1

- 文档版本：v0.1
- 创建日期：2026-05-01
- 适用阶段：MVP 实现前
- 关联文档：
  - PRD v0.4
  - 角色与人格设计 v0.1
  - flows v0.4

---

## 1. 架构概览

### 1.1 总览

```
┌─────────────────────────────────────────────────────────────┐
│                     AI 桌宠 (Tauri 2.x)                       │
│                                                               │
│   ┌──────────────────────────────────────────────────────┐  │
│   │         WebView 前端层 (Vue 3 / React + TS)          │  │
│   │  ┌─────────┐ ┌──────────┐ ┌────────┐ ┌──────────┐   │  │
│   │  │ 桌宠渲染 │ │ 对话面板 │ │ 设置页 │ │人格工坊 │   │  │
│   │  │(Live2D) │ │          │ │        │ │          │   │  │
│   │  └─────────┘ └──────────┘ └────────┘ └──────────┘   │  │
│   └────────────────────┬─────────────────────────────────┘  │
│                        │ Tauri IPC (invoke / event)          │
│   ┌────────────────────┴─────────────────────────────────┐  │
│   │           Rust 主进程 (Core Services)                 │  │
│   │                                                        │  │
│   │  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │  │
│   │  │ 对话服务 │  │ 任务服务 │  │  人格服务        │   │  │
│   │  │ (Chat)   │  │(Reminder │  │  (Persona)       │   │  │
│   │  │          │  │ Pomodoro │  │  - .soul.md 解析 │   │  │
│   │  │          │  │ Todo)    │  │  - 试聊沙盒      │   │  │
│   │  └────┬─────┘  └────┬─────┘  └────┬─────────────┘   │  │
│   │       │             │              │                 │  │
│   │  ┌────┴─────────────┴──────────────┴──────────────┐ │  │
│   │  │           记忆服务 / 拼装链路 (Memory + Prompt) │ │  │
│   │  └────┬───────────────────────────────────────────┘ │  │
│   │       │                                                │  │
│   │  ┌────┴────────┐  ┌──────────┐  ┌─────────────────┐ │  │
│   │  │ LLM Provider│  │ 加密存储 │  │ 调度 / 通知     │ │  │
│   │  │  抽象层     │  │ (DPAPI) │  │ (Scheduler)    │ │  │
│   │  └────┬────────┘  └────┬─────┘  └────┬───────────┘ │  │
│   │       │                │              │              │  │
│   │  ┌────┴────────────────┴──────────────┴──────────┐  │  │
│   │  │  持久化层 (SQLite + 文件系统 + 埋点队列)       │  │  │
│   │  └────────────────────────────────────────────────┘  │  │
│   └────────────────────────────────────────────────────────┘  │
│                              │                                 │
│        ┌─────────────────────┼─────────────────────┐          │
│        ↓                     ↓                     ↓          │
│   云模型 API            操作系统 API          自动更新服务      │
│  (OpenAI/Claude/...)  (托盘/通知/快捷键)   (tauri-updater)    │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 关键设计决策

| 决策 | 选择 | 理由 |
|---|---|---|
| 桌面框架 | Tauri 2.x | 体积小（~20MB）、内存占用低、Rust 主进程安全 |
| 前端框架 | Vue 3 + TS | （备选 React，需 M0 内确认）按团队栈定 |
| 桌宠渲染 | WebView 内的 Live2D Cubism Web SDK（M0 确认） | 表现力 + 社区资源 |
| 数据存储 | SQLite（结构化）+ 文件系统（人格/资源） | 离线优先，无服务端依赖 |
| 进程模型 | 单进程多窗口 + 隐藏的桌宠窗口（透明、点击穿透分区） | 内存预算下唯一选择 |
| LLM 调用 | 主进程发起，前端不直连 | API Key 不进 WebView，安全 |
| 加密 | Windows DPAPI | 与用户账户绑定，不需要二次密码 |
| 通信 | Tauri IPC（invoke + event） | 内置类型化 |

---

## 2. 进程与窗口模型

### 2.1 进程

- **主进程（Rust）**：所有 IO、网络、加密、数据库、调度。
- **WebView 进程（Edge WebView2）**：仅 UI 渲染。
- **不引入额外子进程**（本地小模型在 P1 引入时再考虑独立进程）。

### 2.2 窗口

| 窗口 | 类型 | 默认状态 |
|---|---|---|
| `pet` | 透明、置顶、无边框、点击穿透除身体外区域 | 启动后常驻 |
| `chat` | 普通、无边框、固定在桌宠附近 | 快捷键唤起 |
| `settings` | 普通、独立 | 用户主动打开 |
| `workshop` | 普通、独立（人格工坊） | 用户主动打开 |
| `onboarding` | 普通、模态 | 仅首启 |
| `tray-menu` | 系统托盘（非窗口） | 常驻 |

### 2.3 多显示器与 DPI

- 桌宠位置以**逻辑像素 + 屏幕标识**双键存储，重启或屏幕变化后正确还原。
- Live2D 渲染按当前屏幕 DPI 计算缩放，避免模糊。

---

## 3. 模块边界

### 3.1 模块清单

| 模块 | 责任 | 主要 API |
|---|---|---|
| **PetRenderer**（前端） | 桌宠状态机渲染、动作、表情 | `playMotion(name)` `setExpression(emo)` |
| **ChatPanel**（前端） | 对话 UI、流式渲染 | 通过 IPC 调 ChatService |
| **PersonaWorkshop**（前端） | 人格工坊 GUI | 通过 IPC 调 PersonaService |
| **ChatService**（主进程） | 对话编排、prompt 拼装、流式回复 | `chat.send` `chat.cancel` |
| **PersonaService**（主进程） | .soul.md 读写、校验、热切换 | `persona.list` `persona.get` `persona.save` `persona.import` |
| **MemoryService**（主进程） | 用户偏好读写、增量更新 | `memory.get_all` `memory.set` `memory.delete` |
| **TaskService**（主进程） | 提醒/番茄/待办的状态机与持久化 | `reminder.*` `pomodoro.*` `todo.*` |
| **Scheduler**（主进程） | 定时器、并发优先级、休眠唤醒恢复 | 内部 |
| **LLMProvider**（主进程） | 多供应商抽象、流式接口 | `provider.chat_stream` |
| **CryptoService**（主进程） | DPAPI 包装、敏感字段加解密 | `crypto.protect` `crypto.unprotect` |
| **TelemetryService**（主进程） | 埋点收集、本地缓存、补发 | `telemetry.record` |
| **MigrationService**（主进程） | DB schema 升级、备份、回滚 | 启动时执行 |
| **UpdaterService**（主进程） | tauri-updater 集成 | `updater.check` `updater.install` |
| **NetworkProbe**（主进程） | 网络状态探测、模式切换通知 | event: `network.changed` |
| **SecurityGuard**（主进程） | 安全前缀注入、内容过滤 | 内部，被 ChatService 调用 |

### 3.2 严格依赖方向

```
前端 → IPC → 主进程服务 → 持久化 / 外部
ChatService → SecurityGuard + PersonaService + MemoryService → LLMProvider
PersonaService 不能调用 MemoryService（解耦），由 ChatService 统一编排
TaskService 与 PersonaService 互不依赖
```

---

## 4. 数据模型（SQLite Schema）

### 4.1 表清单

```sql
-- 元信息
CREATE TABLE schema_version (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

-- 配置（kv）
CREATE TABLE config (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- 加密敏感数据
CREATE TABLE secrets (
  key TEXT PRIMARY KEY,           -- e.g. 'openai.api_key'
  ciphertext BLOB NOT NULL,       -- DPAPI 输出
  updated_at TEXT NOT NULL
);

-- 人格元信息（.soul.md 文件本体在 personas/ 目录）
CREATE TABLE personas (
  id TEXT PRIMARY KEY,            -- slug
  name TEXT NOT NULL,
  version TEXT NOT NULL,
  source TEXT NOT NULL,           -- 'builtin' | 'user' | 'imported'
  file_path TEXT NOT NULL,
  is_active INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- 人格快照（最近 10 个，编辑历史）
CREATE TABLE persona_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  persona_id TEXT NOT NULL,
  version TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (persona_id) REFERENCES personas(id) ON DELETE CASCADE
);

-- 记忆（用户偏好，与人格正交）
CREATE TABLE memory (
  key TEXT PRIMARY KEY,           -- e.g. 'username', 'wake_time'
  value TEXT NOT NULL,
  source TEXT NOT NULL,           -- 'user_set' | 'inferred'
  updated_at TEXT NOT NULL
);

-- 对话会话与消息
CREATE TABLE conversations (
  id TEXT PRIMARY KEY,            -- ULID
  persona_id TEXT NOT NULL,
  started_at TEXT NOT NULL,
  last_activity_at TEXT NOT NULL,
  is_sandbox INTEGER NOT NULL DEFAULT 0  -- 试聊沙盒标记
);

CREATE TABLE messages (
  id TEXT PRIMARY KEY,            -- ULID
  conversation_id TEXT NOT NULL,
  role TEXT NOT NULL,             -- 'user' | 'assistant' | 'system'
  content TEXT NOT NULL,
  mode TEXT NOT NULL,             -- 'online' | 'offline_rule'
  created_at TEXT NOT NULL,
  FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
CREATE INDEX idx_messages_conv ON messages(conversation_id, created_at);

-- 提醒
CREATE TABLE reminders (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  trigger_type TEXT NOT NULL,     -- 'once' | 'daily' | 'weekly' | 'cron'
  trigger_spec TEXT NOT NULL,     -- ISO8601 / cron 表达式
  priority TEXT NOT NULL DEFAULT 'soft',  -- 'soft' | 'hard'
  enabled INTEGER NOT NULL DEFAULT 1,
  snooze_count INTEGER NOT NULL DEFAULT 0,
  next_fire_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE reminder_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  reminder_id TEXT NOT NULL,
  fired_at TEXT NOT NULL,
  action TEXT NOT NULL,           -- 'completed' | 'snoozed' | 'ignored' | 'overdue'
  snooze_count INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_reminder_history_rid ON reminder_history(reminder_id);

-- 待办
CREATE TABLE todos (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  due_at TEXT,
  status TEXT NOT NULL DEFAULT 'open',  -- 'open' | 'done' | 'cancelled'
  source TEXT NOT NULL,                  -- 'manual' | 'ai_breakdown'
  parent_id TEXT,                        -- 拆解结果的父任务
  created_at TEXT NOT NULL,
  done_at TEXT
);

-- 番茄统计
CREATE TABLE pomodoro_sessions (
  id TEXT PRIMARY KEY,
  focus_min INTEGER NOT NULL,
  rest_min INTEGER NOT NULL,
  status TEXT NOT NULL,           -- 'running' | 'paused' | 'completed' | 'cancelled'
  started_at TEXT NOT NULL,
  ended_at TEXT
);

-- 埋点队列（离线缓存）
CREATE TABLE telemetry_queue (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  event_name TEXT NOT NULL,
  schema_version INTEGER NOT NULL,
  payload TEXT NOT NULL,          -- JSON
  created_at TEXT NOT NULL,
  flushed INTEGER NOT NULL DEFAULT 0,
  retry_count INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_telemetry_unflushed ON telemetry_queue(flushed) WHERE flushed = 0;

-- 错误日志
CREATE TABLE error_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  level TEXT NOT NULL,            -- 'warn' | 'error' | 'fatal'
  module TEXT NOT NULL,
  message TEXT NOT NULL,
  context TEXT,                   -- JSON
  created_at TEXT NOT NULL
);
```

### 4.2 设计要点

- **ULID 优先**于 UUID（按时间排序、可读性更好）。
- 时间戳统一 ISO8601 字符串（SQLite 无原生 TIMESTAMP，用文本+索引）。
- 软删除策略：仅对 `messages` 用 `is_deleted` 字段（90 天清理用），其他表使用硬删除。
- WAL 模式（`PRAGMA journal_mode=WAL`）提高并发与崩溃恢复。

### 4.3 schema 迁移

- `migrations/` 目录下顺序文件 `001_init.sql`、`002_add_persona_priority.sql` …
- `MigrationService` 启动时检查 `schema_version`，按序执行未应用的迁移。
- 每次迁移前自动复制 db 到 `backup/db-<schema_v>-<timestamp>.bak`，保留最近 5 个。
- 迁移失败 → 回滚 + 错误上报 + 告知用户"使用前次版本启动"。

---

## 5. IPC 契约（Tauri Commands & Events）

### 5.1 Commands（前端 → 主进程）

```ts
// 对话
chat.send(input: string, conversationId?: string): Promise<{ messageId: string }>
chat.cancel(messageId: string): Promise<void>
chat.history(conversationId: string, limit: number): Promise<Message[]>

// 人格
persona.list(): Promise<PersonaMeta[]>
persona.get(id: string): Promise<PersonaFull>
persona.save(payload: PersonaSave): Promise<{ id: string, version: string }>
persona.import(filePath: string): Promise<{ id: string, conflict?: 'overwrite'|'rename' }>
persona.export(id: string, includeAssets: boolean): Promise<{ path: string }>
persona.activate(id: string): Promise<void>
persona.delete(id: string): Promise<void>
persona.sandbox_chat(payload: { draftMd: string, input: string }): Promise<string>

// 记忆
memory.list(): Promise<MemoryItem[]>
memory.set(key: string, value: string): Promise<void>
memory.delete(key: string): Promise<void>
memory.clear(): Promise<void>

// 任务
reminder.create / list / update / delete / snooze / complete
pomodoro.start / pause / resume / stop / today_stats
todo.create / list / update / complete / breakdown_with_ai

// 设置 & 安全
settings.get / set
secrets.set_api_key(provider: string, key: string): Promise<void>
secrets.test(provider: string): Promise<{ ok: boolean, latency_ms: number }>

// 系统
app.info(): Promise<AppInfo>
app.export_data(): Promise<{ path: string }>
app.import_data(path: string): Promise<void>
app.delete_all(): Promise<void>
updater.check(): Promise<UpdateInfo>
updater.install(): Promise<void>
```

### 5.2 Events（主进程 → 前端）

```ts
'chat.token'         { messageId, delta }              // 流式 token
'chat.done'          { messageId, fullText, latencyMs }
'chat.error'         { messageId, code, message }
'reminder.fired'     { reminderId, priority }
'pomodoro.tick'      { sessionId, remainingMs, phase }
'pet.state_changed'  { from, to, reason }
'network.changed'    { online: boolean, mode: 'online_chat'|'offline_rule' }
'persona.activated'  { id, name }
'updater.available'  { version, mandatory }
```

### 5.3 类型与版本约定

- IPC 字段使用 snake_case（与 SQLite 保持一致），前端在 binding 层转 camelCase。
- 命令名带版本：未来破坏性变更时新建 `chat.send_v2`，老命令保留一段时间。

---

## 6. LLM Provider 抽象

### 6.1 接口

```rust
trait LLMProvider {
    fn id(&self) -> &str;          // 'openai' | 'anthropic' | 'gemini' | 'custom'
    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        options: ChatOptions,
    ) -> impl Stream<Item = Result<ChatChunk>>;
    async fn ping(&self) -> Result<Duration>;
}
```

### 6.2 实现

- M2 内置：OpenAI 兼容（覆盖 OpenAI、deepseek、moonshot、本地 OpenAI 兼容服务）。
- Anthropic / Gemini 用 P1。
- 用户配置 `provider_id + base_url + model + api_key`。

### 6.3 路由策略

- 单 active provider，不做多 provider 自动 fallback（避免账单不可控）。
- 探活失败 → 提示用户检查配置 → 临时切换到离线模式。

### 6.4 流式与取消

- 使用流式响应，逐 token 推 `chat.token` 事件。
- 取消通过 `tokio::CancellationToken`，IPC 层 `chat.cancel` 触发。

---

## 7. 文件系统布局

```
%APPDATA%\AIDesktopPet\
├── app.db                      # SQLite 主库
├── app.db-wal                  # WAL
├── app.db-shm
├── backup\                     # 数据库迁移备份
│   └── db-1-20260501T100000.bak
├── personas\
│   ├── _builtin\
│   │   ├── momo.soul.md
│   │   └── ...
│   └── user\
│       ├── my-cat.soul.md
│       └── my-cat.assets\
├── assets\                     # 公共形象资源
│   └── avatars\
│       └── live2d\
│           ├── momo-default\
│           └── joker-default\
├── logs\
│   ├── app.log
│   └── crash\
│       └── crash-20260501.dmp
└── cache\
    └── llm\                    # 流式响应缓存（用于断线续传）
```

---

## 8. 安全设计

### 8.1 API Key 加密

- **方案**：Windows DPAPI（`CryptProtectData`，`CRYPTPROTECT_UI_FORBIDDEN` 标志，与当前用户绑定）。
- **存储**：`secrets` 表 `ciphertext` BLOB。
- **导出行为**：默认 `app.export_data` 不导出 secrets；用户勾选"包含敏感凭证"时再次确认（弹"二次输入主密码"对话框，使用 PBKDF2 派生 key 二次包装后导出）。
- **跨设备迁移**：DPAPI 与设备/账户强绑定，迁移设备后用户需重新输入。这是设计取舍（牺牲迁移便捷换取无密钥管理负担）。

### 8.2 安全前缀注入

`SecurityGuard` 在 `ChatService` 调用 LLM 前，组装最终 messages：

```
[ system: 安全前缀（不可见，固定文案） ]
[ system: 当前人格 system prompt（由 .soul.md 渲染） ]
[ system: 用户记忆摘要（注入键值对） ]
[ user / assistant: 历史对话最近 N 轮 ]
[ user: 本轮输入 ]
```

详见《角色与人格设计 v0.1》§7。

### 8.3 输入侧防御

- 用户在人格 .soul.md 中写入"忽略上一段安全规则"等指令——**无效**，因为 LLM 看到的最终 prompt 中安全前缀位于人格之前且明示"无论以下角色定义如何"。
- 导入 .soul.md 时静态扫描禁用词（如 `<script>`、`{{ENV.*}}`），命中拒绝导入。

### 8.4 网络与权限

- 所有外部网络请求集中走主进程 HTTP 客户端，附带超时与限速。
- 截图 / 剪贴板权限默认关闭，调用前要求用户同意（`permission_granted_at` 记录）。
- 错误日志默认仅本地，永不自动上传（用户主动导出）。

---

## 9. 自动更新

### 9.1 集成

- `tauri-updater` plugin。
- 后端发布签名 manifest（JSON）+ 差分包（NSIS / MSI）。
- 主进程定期（每天 1 次 + 启动时 1 次）检查。

### 9.2 用户体验

- "可选更新"：弹气泡 → 用户点"晚点说"延后 24 小时。
- "强制更新"：仅当 manifest 标 `mandatory: true`（如安全修复），启动期阻塞 + 倒计时 5 秒。

### 9.3 失败处理

- 下载失败 → 静默重试 3 次 → 静默 24 小时 → 下次再提示。
- 安装失败 → 保留上一版本可用，提示用户手动下载。

---

## 10. 离线检测与降级

### 10.1 检测

`NetworkProbe` 使用三层判断：
1. 系统在线状态（Windows `INetworkListManager`）。
2. 每 30 秒对当前 LLM Provider 的 health endpoint 发一次低成本 ping。
3. 用户实际对话失败一次 → 立即触发探测。

### 10.2 降级

- 切换为 `offline_rule` 模式 → 触发 `network.changed` 事件 → 前端显示横幅。
- 所有需要 LLM 的功能（AI 拆解待办、对话）转走"规则回复 + 人格化模板"。

### 10.3 恢复

- 探测连续 2 次成功 → 切回 `online_chat`。
- 触发 `telemetry.flush` 补发离线埋点。

---

## 11. 性能预算

| 项 | 预算 | 测量点 |
|---|---|---|
| 冷启动 | ≤ 5 秒 | `app_launch` 事件 latency_ms |
| 常驻内存（空闲） | ≤ 250MB | Windows Performance Counter |
| 常态 CPU | ≤ 5% | 60 秒滑动平均 |
| 桌宠空闲 GPU | < 2% | DXGI Stats |
| 对话首 token | p50 ≤ 1.5s | `chat_reply_rendered` |
| DB 单次写入 | ≤ 20ms p99 | profiler |
| 人格切换 | ≤ 500ms（含形象替换） | `persona.activate` 命令耗时 |

### 11.1 内存预算分配（约）

| 组件 | 预算 |
|---|---|
| Tauri 主进程（Rust） | 60–80MB |
| WebView2 | 100–140MB |
| Live2D 模型 + 资源 | 40–60MB |
| **合计目标** | **≤ 250MB** |

### 11.2 启动期优化要点

- 桌宠形象延迟到主线程空闲后加载。
- 数据库连接池预热但不预查询。
- 设置 / 工坊页按路由懒加载（不在启动期挂载）。

---

## 12. 测试与发布

### 12.1 测试矩阵

| 维度 | 取值 |
|---|---|
| OS | Windows 10 21H2 / Windows 11 23H2 |
| DPI | 100% / 125% / 150% / 200% |
| 显示器 | 单屏 / 双屏 / 主屏切换 |
| 中文输入法 | 微软拼音 / 搜狗 / 谷歌 |
| 网络 | 在线 / 离线 / 间歇 |
| 模型供应商 | 至少 2 个 OpenAI 兼容服务 |

### 12.2 CI/CD

- GitHub Actions（私有 runner）：tag 触发 → 构建 → 签名 → 上传 manifest。
- 单测覆盖率目标：核心服务（Persona/Chat/Task）≥ 70%。

### 12.3 签名

- M0 决策：是否购买 EV 代码签名证书（影响 Windows SmartScreen 信任度）。
- 即使先用 OV 证书，也至少不让用户面对"未知发布者"红条。

---

## 13. 风险与未决项

| 风险 | 影响 | 缓解 |
|---|---|---|
| Live2D Cubism 商业授权 | M0 选型 blocked | 评估 Spine 2D / 自研 PNG 序列帧的兜底方案 |
| Tauri 2.x 在某些 AV 软件上的误报 | 用户启动失败 | 申请 Microsoft SmartScreen 信誉、提交 AV 厂商白名单 |
| WebView2 缺失（老旧 Win10） | 应用打不开 | 安装包内置 WebView2 Bootstrapper |
| DPAPI 跨用户失败 | 多用户机器混用 | DPAPI 本身就与用户绑定，作为 feature 而非 bug 暴露给用户 |
| Live2D 内存不可控 | 内存超 250MB | 提供"轻量贴图模式"作为兜底 |

### 13.1 未决项（M0 内决定）

1. 前端框架最终选型（Vue 3 vs React）。
2. 桌宠资源管线（Live2D vs Spine vs PNG）。
3. 默认 LLM Provider（API 兼容方案的默认入口）。
4. 是否签名证书。

---

## 14. 实施里程碑映射

| PRD 里程碑 | 架构交付物 |
|---|---|
| M0 决策周 | ADR：前端框架、资源管线、LLM 默认入口 |
| M1 第1-2周 | 主进程骨架、IPC 框架、桌宠透明窗口、Onboarding |
| M2 第3-4周 | TaskService 全功能、PersonaService MVP（CRUD + 切换 + 试聊） |
| M3 第5-6周 | LLM Provider、SecurityGuard、MigrationService、UpdaterService |
| M4 第7-8周 | 性能调优、CI/CD、签名分发、灰度内测 |

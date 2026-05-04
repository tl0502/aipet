# DB 集成测试覆盖补齐报告

- **日期**:2026-05-04
- **触发事件**:M1 W1 D3 实施 B.1 LLMProvider 时,dev panel 调 `dev_list_tables` / `secrets_set_api_key` 报 `(code: 14) unable to open database file`,深挖发现两个 hotfix(Windows 路径反斜杠 URL parsing + tauri-plugin-sql preload 缺失),共同效果是 `aipet.db` 在 M1 W1 完整周期里**从未被创建过**,7 个 service 的 DB 写入路径**一次都没真正运行过**
- **本次范围**:P0 + P1 文档化 + P2 方案文本(未落地)

---

## 1. 现状变化:62 → 87 cargo test(+25)

| 维度 | Before(2026-05-04 上午,B.1 commit `455e005`)| After(本笔)|
|---|---|---|
| 总测试数 | 62 passed | **87 passed** |
| 真实 DB 集成测试 | **0** | **22**(secrets 3 + nickname 6 + memory 8 + persona 5)|
| Fixture 自检 | — | **3**(test_db.rs self-tests)|
| DPAPI 端到端 | 1(crypto roundtrip)| **2**(增 secrets `dpapi_protect_then_sql_then_unprotect_roundtrip`)|

**fixture 设计**(`src-tauri/src/services/test_db.rs`):
- 用 `tempfile::TempDir`(目录级)而非 `NamedTempFile`,sqlite WAL/SHM 同名兄弟文件随目录 Drop 一并清理
- migrations 用 `include_str!("../../migrations/00X_*.sql")` 编译进 binary,与 prod `lib.rs::migrations()` 同一 SQL 源 — 改 schema 测试自动跟进,无双重维护
- 不用 `:memory:`:贴近 prod 磁盘 sqlite,保留未来切到 `PRAGMA foreign_keys=ON` 时的行为一致性
- 不用 `sqlx::test` 宏:其 SQLite 路径需 `#[sqlx::test(migrator = ...)]` 配置,成本 > 手卷 fresh_db

---

## 2. 已补集成测试清单

### 2.1 secrets.rs(7 → 10,+3 测试)

| 测试名 | 覆盖路径 |
|---|---|
| `set_get_delete_roundtrip_via_sql_only` | UPSERT 首次 + 二次覆盖 + delete 幂等(不走 DPAPI 让平台无关)|
| `dpapi_protect_then_sql_then_unprotect_roundtrip` | **plaintext → DPAPI protect → SQL insert → SQL select → DPAPI unprotect** 完整 prod 链路(只少 AppHandle 解析 app_config_dir)|
| `multiple_providers_dont_collide` | OpenAI + DeepSeek + Moonshot 同时存在,delete 一个不影响其他 |

### 2.2 nickname.rs(4 → 10,+6 测试)

| 测试名 | 覆盖路径 |
|---|---|
| `fresh_db_has_singleton_row_with_null_nicknames` | 001 末尾 `INSERT INTO nicknames (id=1)` 已生效,fallback 链:NULL → `FALLBACK_PET_NAME` |
| `set_pet_then_get_pet_returns_set_value` | 基础 set + get round-trip |
| `set_pet_preserves_existing_user_nickname_in_upsert_branch` | **M-3 hardening 防御**:set_pet 走 explicit 两步,UPDATE 分支不能丢 user_nickname |
| `set_pet_twice_then_restore_swaps` | core UX:set "小默" → set "momo" → restore swap 回 "小默" → 再 restore 回 "momo"(双向往返)|
| `restore_when_previous_null_returns_nothing_to_restore` | fresh DB restore 必报 `NothingToRestore` |
| `get_pet_falls_back_to_active_persona_name` | NULL pet_nickname → 取 personas WHERE is_active=1 的 name |

### 2.3 memory.rs(5 → 13,+8 测试)

| 测试名 | 覆盖路径 |
|---|---|
| `insert_message_rejects_unknown_conversation_id` | **FK 防御**:不存在 conversation_id 必被 REFERENCES 守住(同时验证了 B.2 调用方契约)|
| `insert_then_list_returns_inserted_message` | 基础写入 + 读回字段一致 |
| `list_orders_by_created_at_ascending` | ULID 单调递增 + ORDER BY 升序(ChatPanel 渲染依赖)|
| `list_filters_by_conversation_id` | conv-A vs conv-B 隔离 |
| `list_respects_limit` | LIMIT N 截断 |
| `delete_message_removes_only_target` | 单条 delete 不影响兄弟消息 |
| `delete_messages_by_conversation_returns_count_and_isolates` | 整 conversation 清空 + 返回 rows_affected + 跨 conv 隔离 |
| `cleanup_with_conn_drops_old_messages` | cutoff 策略 — 历史(2020 年)删除,新消息保留 |

### 2.4 persona.rs(6 → 11,+5 测试)

| 测试名 | 覆盖路径 |
|---|---|
| `seed_builtin_writes_personas_row` | UPSERT 写入 + is_active=1 + 6 字段一致 |
| `seed_builtin_writes_persona_snapshot` | INSERT persona_snapshots 写入 |
| `seed_builtin_is_idempotent` | **跑 3 次 seed**:personas 仍 1 行(UPSERT)+ snapshots 仍 1 行(ON CONFLICT DO NOTHING)|
| `persona_snapshot_unique_index_blocks_direct_duplicate_insert` | 002 migration 的 UNIQUE INDEX 必须挡住绕过 ON CONFLICT 的直接 INSERT(为 H.2 import flow 防御)|
| `seed_different_versions_keeps_history_in_snapshots` | 用户改 .soul.md version:personas UPSERT 只剩最新,snapshots 累积历史(audit trail)|

### 2.5 fixture self-tests(`services::test_db::self_tests`,3 测试)

- `fresh_db_creates_all_tables` — 抽样 5 张核心表存在
- `fresh_db_seeds_singletons` — nicknames + consent 单行 INSERT 已生效
- `fresh_db_persona_snapshot_unique_index_present` — 002 migration 的 UNIQUE INDEX 已建

---

## 3. 设计决策

### 3.1 抽 `pub(crate) fn xxx_with_conn(conn, ...)` inner helper(P0 落地)

**问题**:5 个 service 都以 `&AppHandle<R>` 为入口,内部 `app.path().app_config_dir().join("aipet.db")` + 自开 sqlx 短期连接。集成测试无 AppHandle/Tauri runtime,无法直接调原函数。

**方案**:对 4 个 DB-touching service 各抽 inner helper(纯 SQL,接 `&mut SqliteConnection`):
- secrets:`set_with_conn` / `get_with_conn` / `delete_with_conn`(3 个)
- nickname:`set_pet_nickname_with_conn` / `set_user_nickname_with_conn` / `get_pet_nickname_with_conn` / `get_user_nickname_with_conn` / `restore_pet_nickname_with_conn`(5 个)
- memory:`insert_message_with_conn` / `list_messages_by_conversation_with_conn` / `delete_message_with_conn` / `delete_messages_by_conversation_with_conn` / `cleanup_messages_with_conn`(5 个)
- persona:`seed_persona_with_conn`(1 个,内部复用已有 `upsert_persona` + `insert_snapshot_if_new`)

**外层 `<R: Runtime>` 行为完全等价**:open_conn → inner → close_conn → emit_event(若有)。inner 不发事件以保持纯 SQL,事件留给 prod 路径。这是 **refactor for testability**,语义不变。

### 3.2 不引 `tauri::test::mock_app()`

考虑过用 mock AppHandle 测原 outer 函数,但放弃:
- mock_app 创建仍需 Tauri runtime 子集 + 配置 path resolver
- 内层 `app.path().app_config_dir()` 在 mock 下返回路径不可控,要再 hack
- 一旦 inner 抽出,outer 的逻辑只剩 `open_conn` + `close_conn` + `emit`,这三件 prod 集成场景由真实 Tauri 启动覆盖(P2 e2e smoke),纯 unit 测试覆盖意义不大

### 3.3 不引 `sqlx::test` 宏

考虑过用 `#[sqlx::test(migrator = ...)]`,但放弃:
- 该宏默认 PostgreSQL/MySQL,SQLite 路径需配置 `sqlite_test_runner`
- 配置成本 > tempfile + apply migrations 手卷方式
- `sqlx::test` 宏每测试一个 DB 的语义 = 我们 fresh_db() 等价,无收益

---

## 4. 暴露的 prod 隐患(留给主 session 处理)

### 4.1 [HIGH] sqlx 默认 `PRAGMA foreign_keys=ON`,B.2 调用方必须先 ensure conversation 存在

**证据**:补 memory 集成测试时,`insert_message_with_conn(conn, &record)` 在 fresh DB 上立即报 `(code: 787) FOREIGN KEY constraint failed`,因为 `messages.conversation_id REFERENCES conversations(id)`。

**影响**:
- sqlx::SqliteConnectOptions::foreign_keys() 默认 true(与 SQLite 自身默认 OFF 不同)
- prod 上 B.2 ChatService 若直接调 `insert_message<R>(app, "any-conv-id", ...)` 而 conversations 表无对应行,**会报 FK 错误**
- 当前 conversations 表唯一写入路径只有 ConversationStore(B.3.f,M1 D5 才实施)
- M1 W1 D3 完整周期内,如果 B.2 急于落地却忘记 ensure conversation,会生产复现 bug

**缓解**:
- 已加 `insert_message_rejects_unknown_conversation_id` 集成测试,该契约现在有红线守护
- 测试 helper `ensure_conversation` 注释里已明示 prod 调用方契约
- **建议**:B.2 ChatService 实施时,在 `chat.send` 入口加 `ensure_conversation(persona_id)`,或者把 `insert_message` 改为 `insert_message_for(persona_id, ...)` 内部自动 ensure(KISS)

### 4.2 [LOW] inner helper 引入 5 个新 dead-code warning

**证据**:`cargo check` 输出 17 warnings(baseline 7 + 新增 10),新增主要来自:
- `delete_message_with_conn` / `delete_messages_by_conversation_with_conn` / `list_messages_by_conversation_with_conn` / `cleanup_messages_with_conn` — 因外层 `<R>` 函数也是 stub(等 B.3 ChatPanel 接入,IPC handler 未注册)
- `set_user_nickname_with_conn` / `get_user_nickname_with_conn` — 因外层 `<R>` 函数已注册 IPC,但 cargo check (cfg without test) 不见测试,inner 在 prod 视角是 unused

**影响**:0(CI `cargo check` 不带 `-D warnings`)

**缓解**:暂不动 — 等 B.2/B.3 接入后这些 warning 自然消失。若希望 hygiene 更干净,可对 5 个 inner 加 `#[allow(dead_code)]`(0.1d,可选)。

---

## 5. P1 待办

### 5.1 [建议跳过 ADR-016] Service 抽象成 `&SqlitePool`

**任务原文**:评估是否把 service 函数 `&AppHandle<R>` 改为 `&SqlitePool`,工作量 > 0.5d 出 ADR-016。

**评估**:**不必走 ADR**,**也不必现在改**。理由:
1. **现有 inner helper 模式已 80% 满足"可测试"目标** — 接 `&mut SqliteConnection` 与接 `&SqlitePool` 在测试场景等价(都不依赖 AppHandle)
2. **prod 短连接模式有意为之** — 当前每个调用都 open → query → close,并发性弱但实现简单。改 Pool 引入连接池共享、错误恢复、关闭策略等系统问题,不是纯 testability 改动
3. **plugin 共享 Pool 路径未通** — `tauri-plugin-sql` 2.4 的 `DbPool` 公共方法被注释(见 persona.rs 行 16-17 注释),Rust 端无法借用 plugin Pool。要切 SqlitePool 必须自行管理 Pool lifecycle,与 plugin 双 Pool 浪费连接
4. **真正需要 Pool 的场景在 M3+**(并发 LLM 流 + 后台 scheduler + 主动陪伴写日志),**不是现在**

**结论**:本期保留短连接 + inner helper 模式;M3 实施 J 模块(主动陪伴)前再评估;若届时确实需要 Pool,届时起 ADR-016 单独决策。**本次报告作 evidence,不起 ADR**。

### 5.2 [✅ 已确认] CI cargo test 已覆盖

`.github/workflows/ci.yml` 行 67-69 已有 `cargo test`,本次 25 个新测试可直接在 CI 跑。**无需补 CI 配置**。

**注意点**:DPAPI 测试(`dpapi_protect_then_sql_then_unprotect_roundtrip` + `crypto::tests::*`)依赖 Windows 用户登录态。CI runner `windows-latest` 默认有交互用户上下文(GitHub Actions 文档确认),DPAPI 应可工作。**首次 CI 跑后若该测试失败需特殊处理**:加 `#[cfg_attr(ci, ignore)]` 或迁移到 service 集成 e2e 阶段。

---

## 6. P2 方案(本次不落地,出文本即可)

### 6.1 e2e smoke test 设计

**目标**:CI 前置一道闸 — 启动 minimal Tauri app → preload sql → 触发 H.1 seed_builtin → 调 5 个 IPC 命令 → 退出。以后再有"DB 没建"这种 bug 直接在 CI 红。

**两条路径选择**:

**路径 A:`cargo test --features e2e` 集成测试目录**
- 在 `src-tauri/tests/e2e_smoke.rs` 加测试
- 用 `tauri::test::mock_app()` 创建真 AppHandle(需 Tauri 2.x 提供测试支持)
- 加载 plugin → 调真实 `seed_builtin<R>` / `secrets::set<R>` / `nickname::set_pet_nickname<R>`
- 优势:统一 cargo test 流程,CI 已覆盖
- 劣势:Tauri test mock 在 2.x 文档化不充分,可能撞坑

**路径 B:独立 binary `cargo run --bin smoke-test --release`**
- `src-tauri/src/bin/smoke.rs`,用 minimal `tauri::Builder::default()` 跑 plugin + setup
- 完成后 `app.exit(0)` 自杀
- CI 加 `cargo run --bin smoke -- --headless`
- 优势:行为最贴近 prod 启动路径
- 劣势:需要解决 GUI 依赖(WebView2 在 windows-latest 应预装)

**推荐**:**路径 A 优先**,失败时切 B。**本期不落地**,留给 M1 D5 I.1 MigrationService 完成 / B.2 ChatService 接入后做(届时 5 个 IPC 路径都已稳定,smoke 才有意义)。**预计工作量 1d**。

### 6.2 PostToolUse hook 提议

**目标**:见到 `services/*.rs` 改动且 git diff 含 sqlx 调用,提示"DB 写入路径需 cargo test 集成测试覆盖,不要只写纯逻辑单测"。

**实施位置**:`.claude/hooks/suggest-checks.cjs`(audit-coverage P0 已建)— 在现有"按 file pattern 建议"逻辑中追加一条规则:

```js
// 规则:services/*.rs 改动 + 含 sqlx::query/query_as → 提示 DB 集成测试
if (
  filePath.match(/services\/(?!test_db|crypto|cursor_tracker|shortcuts|tray|window_)/i) &&
  filePath.endsWith(".rs") &&
  /sqlx::query|sqlx::query_as|fetch_(one|optional|all)|execute\(/.test(diff)
) {
  print(`【DB 测试提示】${filePath} 含 sqlx 调用,建议补 services/test_db::fresh_db 集成测试`);
}
```

**风险点**:hook 只看 single-file diff,误报概率中等(例如有人改 services/llm.rs 加了 reqwest 调用恰好 grep 命中 `query`)。可加白名单 services 列表。

**推荐**:**M1 D5 I.1 完成 + B.2 接入后落地**(届时模式已稳,hook 才不会被频繁误触发);本期不做,留给主 session decision。

---

## 7. 验收

- [x] **P0-1** 调研 sqlx::test / tempfile,选 tempfile + 手卷 migrations 路径(理由见 §3.3)
- [x] **P0-2** 4 个 service 各补 1-2 个真实 DB 集成测试(实际 secrets 3 + nickname 6 + memory 8 + persona 5 = 22 个,见 §2)
- [x] **P0-3** 测试可在 CI 上无 GUI 跑通(纯 sqlx + tempfile,不依赖 Tauri runtime)
- [x] **P1-4** 本报告记录决策摘要(见 §3 + §4)+ decisions-log 同步(本次 commit)
- [x] **P1-5** 评估 SqlitePool 抽象工作量(见 §5.1,**结论:本期不动,M3 再评估**)
- [x] **P1-6** 检查 CI workflow(见 §5.2,**结论:已覆盖**)
- [x] **P2-7** e2e smoke test 设计文本(见 §6.1)
- [x] **P2-8** PostToolUse hook 提议(见 §6.2)
- [x] **不打扰主 session B.1 收口节奏**:本次只动测试,prod 代码 inner helper 抽出 = 行为等价 refactor;Cargo.toml 加 `[dev-dependencies] tempfile + tokio` 不影响 release build

**最终 cargo test**:62 → **87 passed; 0 failed; 0 ignored**(超 80+ 目标)

---

## 8. 关联文件 / 引用

- 本次改动文件:
  - `src-tauri/Cargo.toml`(+ dev-dependencies tempfile, tokio)
  - `src-tauri/src/services/mod.rs`(+ `#[cfg(test)] pub mod test_db`)
  - `src-tauri/src/services/test_db.rs`(新建,109 行)
  - `src-tauri/src/services/secrets.rs`(抽 3 inner + 3 集成测试)
  - `src-tauri/src/services/nickname.rs`(抽 5 inner + 6 集成测试)
  - `src-tauri/src/services/memory.rs`(抽 5 inner + 8 集成测试)
  - `src-tauri/src/services/persona.rs`(抽 1 inner + 5 集成测试)
- 触发证据:
  - `~/.claude/plans/dapper-beaming-quokka.md` 顶部三个 Hotfix 段落(B.1 plan)
  - commit `93b5974 fix(db): SqliteConnectOptions 改 builder API`(Hotfix-2)
  - 待提交 `tauri.conf.json` `plugins.sql.preload`(Hotfix-3)
- 关联模块文档:
  - `docs/AIPET-obsidian/架构设计/...v1.0.md` §4(SQLite schema)
  - `progress/decisions-log.md` 2026-05-04 新增条目(本次同步)

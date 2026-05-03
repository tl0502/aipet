# Code review 2026-05-03

## Run 15:42:00 — range: $branch (origin/milestone/m1...HEAD = feat/m1-d2-window-interaction 整体 diff)

### 审查文件清单(33 个,过滤后)

**Rust 后端(17):**
- src-tauri/src/lib.rs
- src-tauri/src/state.rs
- src-tauri/src/commands/{ping.rs, window.rs, nickname.rs, dev.rs, mod.rs}
- src-tauri/src/services/{crypto.rs, persona.rs, memory.rs, nickname.rs, cursor_tracker.rs, shortcuts.rs, tray.rs, window_actions.rs, window_snap.rs, dev_window.rs, mod.rs}

**前端(14):**
- src/{App.vue, main.ts}
- src/components/PetCanvas.vue
- src/composables/{useShortcutListener.ts, useVRMModel.ts}
- src/ipc/{commands.ts, dev.ts}
- src/services/vrm.ts
- src/stores/pet.ts
- src/views/{Pet.vue, DevPanel.vue}
- src/views/dev/{IpcPlayground.vue, TablesView.vue, EventLog.vue, LogsView.vue}

**SQL(2):**
- src-tauri/migrations/{001_init.sql, 002_persona_snapshot_unique.sql}

**安全相关配置:**
- src-tauri/capabilities/{default.json, dev.json}
- src-tauri/Cargo.toml(依赖新增行)

跳过(默认黑名单):docs/ progress/ .claude/ *.md *.vrm pnpm-lock.yaml Cargo.lock .github/workflows/ tsconfig*.json eslint.config.js vite.config.ts index.html package.json (lock 部分)

### 总裁决:Pass(可放行 ship-task)

无 Critical 漏洞。无 High 漏洞。Medium 5 条 / Low 5 条,建议合并到下一笔 hardening commit,不阻塞当前 ship。

### 各维度命中数

| 维度 | Critical | High | Medium | Low | Won't fix |
|---|---|---|---|---|---|
| 1 注入与执行 | 0 | 0 | 0 | 0 | 1 |
| 2 加密与凭证 | 0 | 0 | 1 | 1 | 0 |
| 3 数据完整性 | 0 | 0 | 1 | 0 | 0 |
| 4 输入校验与边界 | 0 | 0 | 1 | 0 | 0 |
| 5 并发与资源管理 | 0 | 0 | 1 | 1 | 1 |
| 6 错误处理与 panic | 0 | 0 | 1 | 2 | 0 |
| 7 信息泄露 | 0 | 0 | 0 | 1 | 0 |
| 8 LLM 越狱 | 0 | 0 | 0 | 0 | N/A(无 LLM 路径) |

---

### Critical (0 条)

无。

---

### High (0 条)

无。

---

### Medium (5 条)

#### M-1 [`src-tauri/src/services/crypto.rs:34`] CWE-190 整数截断:`plaintext.len() as u32` 未防 4GB+ 输入

`CRYPT_INTEGER_BLOB.cbData` 是 `u32`,`plaintext.len()` 是 `usize`(64-bit 平台 = u64)。当 `plaintext` 长度 > `u32::MAX`(~4GB)时 `as u32` 静默截断,DPAPI 看到的字节数与实际指针长度不一致,加密结果错误且越界读取风险。同问题见 unprotect 行 65。

实际触发条件极低(API key 通常 < 100 B),但函数签名接受 `&[u8]`,无大小约束 = 防御缺失。

- **修复:** 在 `protect` / `unprotect` 函数顶部加 `if plaintext.len() > u32::MAX as usize { return Err(CryptoError::InputTooLarge) }`,加新错误变体
- **验证:** 单测构造 `vec![0u8; u32::MAX as usize + 1]` 太占内存,改为 `let len = u32::MAX as usize + 1; let result = protect(unsafe { std::slice::from_raw_parts(<dummy_ptr>, len) });`(只是测错误路径,不真实分配)

#### M-2 [`src-tauri/src/services/crypto.rs:54,86`] CWE-401 OOM 时 LocalFree 不会被调用导致内存泄漏

```rust
let result = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize).to_vec();
let _ = LocalFree(Some(HLOCAL(out_blob.pbData as *mut _)));
```

如果 `to_vec()` 因 OOM panic(进程内存压力下),`LocalFree` 不会被调用,DPAPI 分配的 LocalAlloc 内存泄漏。每次 protect/unprotect 失败留几十 KB 内存,长时间累积可能导致进程 commit charge 上升。

- **修复:** 用 `scopeguard::defer!` 或自定义 RAII guard struct,在 `Drop` 里 LocalFree。或者改顺序:先 LocalFree 一份"保险",再 to_vec();但 to_vec 之前 free 后 slice 失效,这条不可行。**正解是 RAII guard**
- **验证:** 加 `scopeguard` 依赖 + `cargo test crypto::tests::round_trip_typical_api_key` 仍通过

#### M-3 [`src-tauri/src/services/nickname.rs:130-148`] CWE-1029 防御性数据丢失:UPSERT 子查询在 nicknames 行不存在时丢失 user_nickname

```sql
INSERT INTO nicknames (id, pet_nickname, pet_nickname_previous, user_nickname, updated_at)
VALUES (
    1, ?, 
    (SELECT pet_nickname FROM nicknames WHERE id = 1),
    (SELECT user_nickname FROM nicknames WHERE id = 1),
    ?
)
ON CONFLICT(id) DO UPDATE SET ...
```

如果 nicknames 表行 id=1 被外部 DELETE(当前 dev panel 不提供 DELETE 操作所以实际不可达),此 UPSERT 走 INSERT 分支,子查询返回 NULL,user_nickname 永久丢失。set_pet 影响 user_nickname 是 cross-field side effect,违反单一职责。

- **修复:** ① set_pet 改为只 UPDATE(seed 行已保证存在),失败再报错让上层处理;② 或改为 explicit 两步:先 SELECT 当前 user_nickname,再 UPSERT 显式 bind 旧值
- **验证:** 加单测:DELETE FROM nicknames WHERE id = 1 → set_user_nickname("张三") → set_pet_nickname("小默") → user_nickname 仍为 "张三"

#### M-4 [`src-tauri/src/services/cursor_tracker.rs:33-34`, `src-tauri/src/commands/window.rs:62,74,80`] CWE-248 Mutex `.lock().unwrap()` poison panic 链

5 处共享 state Mutex 用 `.unwrap()`。release 模式下 `panic = "abort"`(Cargo.toml 行 47),poison 不会发生(进程已 abort),OK。**但 debug 模式不 abort**,任一 lock 持有线程 panic 后,所有 cursor_tracker 与 window IPC 命令都会传播 panic。开发期 panic 容易触发(用户已踩过 hitbox NaN 路径),整个桌宠输入死锁。

- **修复:** 用 `.lock().unwrap_or_else(|e| e.into_inner())` 容忍 poison 并继续;或自定义 Mutex wrapper 内部统一处理
- **验证:** debug 模式下用 `cargo test --package aipet -- state::tests::poison_recover`(写一个测试,先在另一线程 panic 持锁,再验证主线程能恢复)

#### M-5 [`src-tauri/src/commands/dev.rs:74`] CWE-89(防御性)字符串拼接 SQL 路径,白名单是唯一守卫

```rust
let sql = format!("SELECT * FROM {} LIMIT {}", table, limit);
```

`table` 已通过 `ALLOWED_TABLES` 5 表白名单校验,`limit` 是 u32 + `.min(500)`,**当前安全**。但白名单是唯一安全屏障,后续若加新表(如 M3 J 模块的 proactive_care_log)忘记同步更新 ALLOWED_TABLES,潜在风险。命令仅 `#[cfg(debug_assertions)]` 编译进 debug 二进制,release 排除,影响范围有限。

- **修复:** 加注释或 unit test 锁定 ALLOWED_TABLES 与 schema 同步;或改用 `sqlx::QueryBuilder` 配 column 白名单(更健壮但代码量增加)
- **验证:** 加单测枚举 sqlite_master 所有 table,断言 ALLOWED_TABLES ⊆ schema_tables(防漏);若 dev panel 未来需要审计某新表,白名单更新成必备步骤

---

### Low (5 条)

#### L-1 [`src-tauri/src/services/crypto.rs:42-58, 73-89`] CWE-754(防御性)`out_blob.pbData` 未做 null 检查

DPAPI `CryptProtectData` / `CryptUnprotectData` 在 `BOOL` 返回成功时按规约 `pbData` 必非 null。代码已用 `.map_err(...)` 处理 BOOL false,理论上到 `from_raw_parts` 时 pbData 已非 null。**但**没有显式 `if out_blob.pbData.is_null()` 校验,假设 Win32 API 行为是隐性合约,极端情况(系统 corruption)下可能 UB。

- **修复:** unsafe 块内加 `if out_blob.pbData.is_null() { return Err(CryptoError::ProtectFailed(...)) }`(但 windows::core::Error 不便构造,可以新增 NullPtr 错误变体)

#### L-2 [`src-tauri/src/services/crypto.rs:46-49, 78-81`] DPAPI description 字段为 null,丢失审计上下文

```rust
CryptProtectData(&in_blob, PCWSTR::null(), None, None, None, ...)
```

第二参数是 description,DPAPI ciphertext header 内可读(无解密需求)。当前为 null,审计场景(用户排查"谁在我账号下加密了什么")失去线索。安全无影响,**audit hygiene** 缺失。

- **修复:** 传 `PCWSTR(wide_string("aipet:llm_api_key").as_ptr())` 等具体描述,需配 `widestring` crate 或手写 UTF-16 转换

#### L-3 [`src-tauri/src/services/cursor_tracker.rs:33-34`] CWE-362(轻度)双 lock 非原子读取 is_dragging + hitbox

`state.is_dragging.lock()` 与 `state.pet_hitbox.lock()` 是两次 lock,非原子。极端 race 下 cursor_tracker 拿到的 (is_dragging, hitbox) 组合可能是中间状态,导致 1 帧的鼠标穿透判断错误。但下一 tick(16ms)即恢复,影响极小,且不引入安全问题。

- **修复:** 不必修;若想要严格,可把两个字段合并到一个 `Mutex<TrackerState>` 单 lock 读取

#### L-4 [`src-tauri/src/lib.rs:104`] CWE-248 main 入口 `.expect("error while running tauri application")`

`tauri::Builder::run` 失败时 panic 是合理选择(无法 graceful 处理)+ release 模式 `panic = abort`。**但** debug 模式会 backtrace 一大堆,backtrace 可能含本地构建路径(`C:\Users\<username>\...`)= 信息泄露 (CWE-200) 给本地查看者。本地 dev 场景不算漏洞。

- **修复:** 不必修

#### L-5 [`src-tauri/src/lib.rs:45`] eprintln panic hook 输出到 stderr,可能含路径

```rust
std::panic::set_hook(Box::new(|info| {
    eprintln!("\n=== APP PANIC ===\n{info}\n=================");
}));
```

`info: PanicInfo` 包含触发位置 file/line,通常是项目内相对路径,但 cargo 编译 debug 时可能含构建机绝对路径。release 模式 `strip = true` (Cargo.toml 行 48) 不影响 PanicInfo。stderr 在 GUI 应用通常不可见,影响极小。

- **修复:** 不必修;若上 telemetry 后,确保 panic info 不上报含路径的部分

---

### Won't fix(false positive,标 CWE 不适用理由)

#### WF-1 [`src-tauri/src/services/persona.rs:153,186`] 看似 SQL 注入但 sqlx 已 bind 参数

`INSERT INTO personas ... VALUES (?, ?, ?, ...)` + `.bind(...)` 链 — sqlx 走 prepared statement,无字符串拼接 SQL,CWE-89 不适用。

#### WF-2 [`src-tauri/src/services/cursor_tracker.rs:14`] 看似无限循环不可中断

`thread::spawn` 起后台线程跑 `loop { sleep + GetCursorPos }`,看似无终止条件。**但** 行 23 `let Some(window) = app.get_webview_window("pet") else { break }` — 主窗口被 destroy 时(进程退出阶段)`get_webview_window` 返回 None,线程自然结束。CWE-835(无限循环)不适用。

#### WF-3 [`src-tauri/src/services/persona.rs:30`] include_str! 编译期内置 momo 看似硬编码秘密

`MOMO_RAW = include_str!("../../personas/_builtin/momo.soul.md")` 把内置人格 markdown 编译进二进制。这是 ADR-009 设计意图(开箱可用),不是凭证 / API key,CWE-321 不适用。

---

### Verification plan

执行下面命令验证现状(非建议修复后跑一次):

```bash
# 已通过的单元测试覆盖度
cd src-tauri && cargo test
# 27 passed(M1 D3 hardening 后)

# typecheck / lint 应全绿(本审查不改业务代码)
pnpm typecheck
pnpm lint
cd src-tauri && cargo check
```

如果接受 Medium 修复建议,下一笔 hardening commit 应包含:

1. M-1 / M-2:`src-tauri/src/services/crypto.rs` 加 InputTooLarge 错误 + scopeguard / RAII LocalFree;Cargo.toml 加 `scopeguard = "1"`
2. M-3:`src-tauri/src/services/nickname.rs` 重写 set_pet 为 explicit 两步(SELECT 当前 user → UPSERT 显式 bind)
3. M-4:全 `state.is_dragging` / `state.pet_hitbox` `.lock().unwrap()` 改为 `.lock().unwrap_or_else(|e| e.into_inner())` 或封装 helper
4. M-5:`src-tauri/src/commands/dev.rs` 加同步检查单测,断言 `ALLOWED_TABLES ⊆ sqlite_master 列表`

预期 cargo test 从 27 → 30+ passed。

---

## 审查方法论

- 范围:`feat/m1-d2-window-interaction` 自 `origin/milestone/m1` 拉出后的全部 diff
- 工具:Read / Grep(`unwrap|expect`、`eprintln|println|dbg`、`unsafe`)
- 维度:8 维度纯漏洞(注入 / 加密 / 数据完整性 / 输入边界 / 并发 / 错误处理 / 信息泄露 / LLM 越狱)
- 排除:业务逻辑 / 架构对齐 / ADR 一致性 / 性能预算 / 测试覆盖率
- 上下文:`.claude/commands/code-audit.md` § 8 维度漏洞扫描提示词

## Notes

- 维度 8(LLM 越狱)在本审查无审查对象 — 项目尚未引入 LLM provider(B.1 待实现);该维度首次有意义的审查会在 B.1 落地后跑 `/code-audit $staged`
- 维度 7(信息泄露)只发现 panic info 路径泄露的 Low(L-4 / L-5),未发现 messages.content / 应用名 / 窗口标题 / 麦克风的语义读取 — CI grep 已覆盖语法层;本次语义层亦无命中
- 维度 1 + 5 的几条 Won't fix 反映 sqlx 设计 / cursor tracker 自然终止,代码已经按预期写,**不需要改**;Won't fix 段保留是为审查可追溯
- 整体观感:M1 D2/D3 hardening 4 笔后,代码安全姿态明显提升;现存 Medium/Low 多是"防御性 hardening"而非"现实漏洞",符合 Tauri + Rust 项目早期阶段的预期

# Code Audit Report — 2026-05-04 (B.1 LLMProvider)

- **范围**:`$staged` 增量(B.1 实施新增 / 修改的 9 文件)
- **基准 SHA**:`99066a8`(上次扫 `$branch` 全 33/33 → Pass,详 `code-review-2026-05-03.md`)
- **8 维度**:注入 / 加密 / 数据完整性 / 输入边界 / 并发 / 错误处理 / 信息泄露 / LLM 越狱
- **裁决**:**Pass**(0 Critical / 0 High / 1 Medium / 2 Low / 3 Won't fix-defer)
- **执行**:main 直接按 `.claude/agents/code-reviewer.md` SOP 跑(网关 panic,subagent 不可用)

---

## 文件清单(staged 范围)

| 文件 | 类型 | 行数 |
|---|---|---|
| `src-tauri/Cargo.toml` | edit | +10 lines |
| `src-tauri/src/services/mod.rs` | edit | +4 lines |
| `src-tauri/src/services/secrets.rs` | new | 175 lines |
| `src-tauri/src/services/llm.rs` | new | ~470 lines |
| `src-tauri/src/commands/mod.rs` | edit | +1 line |
| `src-tauri/src/commands/llm.rs` | new | ~140 lines |
| `src-tauri/src/lib.rs` | edit | +6 lines |
| `src/ipc/dev.ts` | edit | +50 lines |
| `src-tauri/Cargo.lock` | auto | dep tree |

---

## 8 维度逐项

### 1. 注入(SQL / OS / HTTP / template)

- **secrets.rs**:全部 `sqlx::query("...?...").bind(&val)` parameterized,无 `format!` / 字符串拼 SQL → **Pass**
- **llm.rs `chat_stream` URL 拼接**:`format!("{}/chat/completions", self.base_url)`;`base_url` 来自 `normalize_base_url(&str)`(纯字符串处理,不验 URL 格式)。用户可填 `https://evil.com/v1` 但这是用户自身配置(DPAPI 模型已假设用户可信);非 http(s) scheme 会被 reqwest::Client::post 拒绝 → **Pass**(防御足够)
- **commands/llm.rs**:无 SQL / shell / template,纯 IPC 转发 → **Pass**

### 2. 加密(DPAPI / 密钥 / TLS)

- **DPAPI 路径**:`secrets::set` 调 `crypto::protect`、`secrets::get` 调 `crypto::unprotect`,与 I.2 已就绪契约一致 → **Pass**
- **api_key 内存模型**:`OpenAiCompatProvider.api_key: String` 私有字段,**struct 不实现 `Debug`** 防止 `eprintln!("{:?}")` 透出 → **Pass**(主动防御)
- **【L-A1】api_key Drop 不 zero**:String::drop 释放堆但不清零字节,理论上后续内存复用可能残留。已识别为 R3 风险,plan 标注 M3 评估 `zeroize` crate;DPAPI 模型已假设进程内存可信,本期接受 → **Won't fix(defer M3 defense-in-depth)**
- **TLS**:`reqwest = features ["rustls-tls"]` 不开 native-tls,跨平台一致 → **Pass**
- **bearer_auth**:`.bearer_auth(&self.api_key)` 标准用法,reqwest 默认不 log header → **Pass**

### 3. 数据完整性

- **secrets UPSERT**:`INSERT ... ON CONFLICT(key) DO UPDATE SET ciphertext=excluded..., updated_at=excluded...`;`secrets.key` 是 PRIMARY KEY,UPSERT 正确(沿用 H.1 / F.2 同款,migrations/002 已加 UNIQUE INDEX 强绑定经验已应用) → **Pass**
- **updated_at**:`Utc::now().to_rfc3339()` ISO 8601 + 时区,与 schema 注释一致 → **Pass**
- **NULL 处理**:secrets.ciphertext 是 `BLOB NOT NULL`,Rust 端绑定 `&Vec<u8>` 不会传 NULL → **Pass**

### 4. 输入边界

- **SSE 解析**:`parse_sse_payload` 用 `serde_json::from_str`,malformed 转 `LlmError::Sse`,不 panic → **Pass**
- **`Chunk` / `Choice` / `Delta`** 全部 `#[serde(default)]`,容忍 ollama / 个别 compat 服务的 partial schema → **Pass**(覆盖单测 `parse_sse_no_choices_returns_none` / `parse_sse_empty_delta_returns_none`)
- **body 截断**:`body.chars().take(200).collect()` 按 char 切不破坏 UTF-8 边界 → **Pass**
- **normalize_base_url**:不验 URL scheme;reqwest 兜底拒非 http(s) → **Pass**(标注 custom 用户填 `/api/v2` 会被错误补 `/v1`,接受为 MVP 限制)

### 5. 并发

- **secrets.rs**:每次开 SqliteConnection 短期连接 + close,无共享状态 → **Pass**
- **llm.rs**:`reqwest::Client` 内部 Arc Send + Sync;`chat_stream` 返回 `BoxStream<'static, ...>`,无生命周期借用 → **Pass**
- **`try_stream!` 宏内 yield 跨 await**:`async-stream` 已处理 + `futures::pin_mut!` 确保 stream 不 move → **Pass**
- **AppState 锁**:本期未引入新 Mutex / RwLock 共享状态(secrets / llm 全无状态);M-4 `lock_or_recover` 已就绪供后续 service 接入 → **N/A**

### 6. 错误处理

- **【L-B1】Client::new() panic 路径**:`reqwest::Client::new()` 内部 `builder().build().expect(...)`,rustls 初始化失败时直接 panic。MVP 期 Win/macOS/Linux 三平台稳定;比静默 fallback 更清晰。**初版有无效 fallback `unwrap_or_else(|_| Client::new())`(两路径同样失败)→ 已修(本笔)** → **Pass**
- **secrets / llm 全部 Result + ? 传播**,无生产路径 `.unwrap()` / `.expect()`(单测 `#[cfg(test)]` 内 unwrap 符合习惯)→ **Pass**
- **emit 失败静默**:`let _ = app.emit(...)` 在 dev_llm_test_stream(dev-only)+ 已加注释说明 → **Pass**(scope 限定)
- **【M-B2】AppError::Internal 路径泄露**:错误链 `e.to_string()` 可能含 `%LOCALAPPDATA%\AIDesktopPet\...` 路径。已记录在 `code-review-2026-05-03.md` L-12,plan 已说明 M2 抽 `AppError::User` vs `AppError::Internal` 二分 → **Won't fix(defer M2)**

### 7. 信息泄露

- **`LlmError::Server { message }`**:截到 200 chars + body 来自 LLM 服务端不含 client api_key + reqwest 不反射 Authorization header。理论极端场景(服务端把 header 反射进 error)— 200 char 截断已大幅减小风险面 → **Pass**(单测 `map_status_truncates_long_body`)
- **secrets 错误链**:`SecretsError::NotUtf8(#[source])` thiserror 默认不打 source 进顶层 message,api_key 字节不会泄露 → **Pass**
- **OpenAiCompatProvider 不实现 Debug**:防 `format!("{:?}")` 透 api_key → **Pass**(主动防御)
- **dev.llm.token emit payload**:仅 LLM 输出 delta,LLM 不知 client api_key 故不会输出 → **Pass**

### 8. LLM 越狱 / Prompt 安全

- **【M-D1】SecurityGuard 未注入**:B.1 不接 SecurityGuard / safety prefix(架构 §6.1 + ADR-006),`dev_llm_test_stream` 把 user_message 直进 ChatMessage。**Scope**:dev-only 命令,release build 不存在(`#[cfg(debug_assertions)]` + lib.rs invoke_handler 双重排除);用户产线路径走 B.2 ChatService。
  - **Action**:B.2 实施时**强制**接 SecurityGuard;m1.md B.2 task description 已标注。本期接受 → **Defer to B.2(non-issue for B.1)**
- **prompt 注入**:dev_llm_test_stream 不属用户产线,M1 出口前 release build 此命令不存在 → **Pass**(scope 限定)

---

## 修复记录(本笔已应用)

| ID | 文件 | 行 | 维度 | 改动 |
|---|---|---|---|---|
| L-B1 | `src-tauri/src/services/llm.rs` | 228 | 错误处理 | `unwrap_or_else(\|_\| Client::new())` 无效 fallback → 直接 `Client::new()`,与 reqwest 内部 expect 路径一致 |
| L-B3 | `src-tauri/src/commands/llm.rs` | 17 / 154 | 代码质量 | 移除 `let _ = llm::PRESETS;` 抑制 hack;import 从 `llm::self` 改为显式 4 type imports |

cargo test 62/62 passed(+31 vs 上次基线 31)。

---

## Defer 项 backlog

| ID | 维度 | 推迟到 | 理由 |
|---|---|---|---|
| L-A1 | 加密 | M3 | `zeroize` crate 评估 — defense-in-depth,DPAPI 模型已假设进程内存可信 |
| M-B2 | 信息泄露 | M2 | `AppError::User vs Internal` 二分 — 跨多 service 重构,本期范围外 |
| M-D1 | LLM 越狱 | B.2 | SecurityGuard 注入是 ChatService 职责,B.1 仅 Provider 层 |

---

## 累计裁决

- **B.1 staged 增量**:**Pass**(0 Critical / 0 High;Medium 全 defer 跨期 task / Low 含 1 修)
- **路径上累计**(`$branch` 33/33 + B.1 增量 9 文件):**Pass**
- **下次审查触发**:B.2 ChatService 完成时审 staged + B.1 联动 + SecurityGuard 注入路径

---

> 本报告由 main session 直接按 `/code-audit` SOP 生成(网关 panic 期),不动 `progress/.audit-state` 计数器(B.1 commit SHA 进 commit 后由 ship-task 同步)。

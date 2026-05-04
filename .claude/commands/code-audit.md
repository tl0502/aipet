---
description: AI 代码漏洞扫描 — 基于 git diff 8 维度纯技术漏洞扫描(注入/加密/数据完整性/输入边界/并发/错误处理/信息泄露/LLM 越狱)。不查业务/架构/性能预算。默认审 staged。
allowed-tools: Read, Grep, Glob, Bash(git diff:*), Bash(git log:*), Bash(git branch:*), Bash(git ls-files:*), Bash(git rev-parse:*), Bash(git status:*), Bash(cargo clippy *), Bash(pnpm lint:*), Write
argument-hint: [revision-range | $staged | $branch | $since-last | --full] [--include-tests]
---

> **SOP mirror 注释**(2026-05-04):本 command 内嵌完整 8 维度 SOP(网关 panic 期间是真实执行入口);网关恢复后用 `agents/code-reviewer.md` spawn 隔离 worker。**两份 SOP 必须 mirror — 改一边必同步另一边**(否则双份维护会漂移)。

## 任务

对项目代码做 8 维度纯技术漏洞扫描,基于 git diff 增量推进。**不评判业务逻辑、架构对齐、产品决策、性能预算、测试覆盖率** — 这些维度由 doc-aligner / gate-checker / 用户人工 review 处理。

## Step 0:自取前置上下文

> 不在 frontmatter 跑 `!` 预检(空 stdout / 第三方网关解析在 Windows 中文路径下偶发误报失败)。改由本步用 Bash 工具自取。

按顺序执行,把输出留在上下文供后续范围解析使用:

1. `git branch --show-current` → 当前分支
2. `git rev-parse --short HEAD` → 当前 HEAD
3. `git diff --cached --stat` → staged diff 统计(空 = 无 staged 改动,正常)
4. `git status --short` → 工作区状态(空 = working tree 干净,正常)
5. `git log --oneline -5` → 最近 5 笔 commit

再读两个文件(可能不存在,失败即跳):

- `progress/.audit-state`(JSON,`$since-last` 范围依赖)
- `progress/CURRENT.md`(当前 milestone / sprint / blockers)

## 范围解析

按 `$ARGUMENTS` 第一个 token 决定 diff 范围(后续 token 处理 flag):

| `$ARGUMENTS` | 解析 |
|---|---|
| 空 / `$staged` | `git diff --cached`(待提交工作区,**默认**) |
| `HEAD~N..HEAD` / `<SHA1>..<SHA2>` | `git diff <range>` |
| `$branch` | 跑 `git rev-parse --abbrev-ref HEAD` 拿当前分支,从分支名提取 milestone(如 `feat/m1-...` → `milestone/m1`),再 `git diff origin/milestone/mN...HEAD` |
| `$since-last` | 读 `progress/.audit-state` 的 `last_audit_sha`,跑 `git diff <sha>..HEAD`;若文件不存在或 sha 无效 → 退化到 `$branch` 并提示 |
| `--full` | 不走 diff,枚举全库文件(`git ls-files src/ src-tauri/src/ migrations/`)|

flag(可与上面任一组合):

- `--include-tests`:启用测试代码审查(默认跳过)

输出最终决定的 `range_label`(如 `$staged` / `HEAD~3..HEAD` / `$branch` 解析后的 `origin/milestone/m1...HEAD`)给后续报告头使用。

## 文件过滤

**白名单(默认包含):**
- `src/**/*.{ts,vue,js,mjs,cjs}` — 前端代码
- `src-tauri/src/**/*.rs` — Rust 后端
- `migrations/*.sql` 或 `src-tauri/migrations/*.sql` — DB schema(数据完整性维度核心)
- `Cargo.toml` / `package.json` 的依赖新增/修改行(轻量扫供应链可疑名)

**黑名单(默认跳过):**
- `docs/**` — 文档
- `progress/**` — 进度笔记
- `.claude/**` — agent 配置
- `*.md` — 任何 markdown
- `public/**` 含 `*.{vrm,glb,fbx,png,jpg,wav,ogg}` — 二进制资源
- `.github/workflows/**` — CI 配置
- `tsconfig*.json` / `*.config.{ts,js}` — 构建配置
- `*.lock` / `pnpm-lock.yaml` / `Cargo.lock` — 锁文件
- 测试代码 `**/*.test.{ts,js}` / Rust `#[cfg(test)]` 块(仅 `--include-tests` 启用时审)

应用过滤后,先输出实际进入审查的文件清单(按文件计数),让用户确认范围合理,然后再开始 8 维度扫描。**若过滤后无文件,直接退出并提示"无代码改动需审查"**。

## 8 维度漏洞扫描提示词

对过滤后的每个文件,逐一按 8 维度检查。每条 finding 必须给:**文件:行号 + CWE/OWASP 编号 + 一句描述 + 一行修复路径**。

### 维度 1:注入与执行 (CWE-89, 78, 22, 502, OWASP A03)

- SQL 注入:`sqlx::query` / `sqlx::query_as` 是否一律用 `?` bind 参数,绝无字符串 format 拼接 SQL
- 命令注入:`std::process::Command` / `tokio::process::Command` 是否走 `arg()` 而非 shell 解释
- 路径遍历:接受用户路径的入口是否走 `canonicalize` + 前缀校验,堵 `..` / 绝对路径越界
- 不安全反序列化:`serde_json::from_str` 接受用户/网络输入时是否限制深度 / 大小;未验证字段是否被信任
- unsafe Rust 块:每个 `unsafe { ... }` 必须问"为什么必须 unsafe"+ 不变式注释
- FFI 误用:DPAPI / windows crate 调用是否检查 `BOOL` 返回值,errno 是否处理

### 维度 2:加密与凭证 (CWE-321, 327, 338)

- DPAPI 调用必须带 `CRYPTOPROTECT_UI_FORBIDDEN` 标志(隐私边界硬要求)
- 密钥 / token / API key / PAT / GitHub secret 是否硬编码到源码或 println / dbg!
- 弱算法:`MD5` / `SHA1` 用于安全场景(不是 checksum)即漏洞;推荐 SHA-256+/Blake3
- 随机数源:涉及凭证 / 密钥 / nonce 时不能用 `rand::thread_rng` / `Math.random`,应用 `OsRng` / crypto-secure 源
- 凭证日志泄露:`debug!` / `info!` / `eprintln!` 不能打印凭证片段或 unprotect 后的明文

### 维度 3:数据完整性 (CWE-362)

- `ON CONFLICT(cols)` 子句必须配合 cols 上的 `UNIQUE` 约束才生效(SQLite 规约:无 UNIQUE 时 ON CONFLICT 子句静默失效);**这是用户已踩过的真实漏洞**
- 事务边界:多步骤原子操作必须包在同一 `sqlx::Transaction` 内,跨步骤不能 commit 中间状态
- UPSERT vs REPLACE:`INSERT OR REPLACE` 会先 DELETE 触发外键级联;改用 `ON CONFLICT DO UPDATE` 防数据丢失
- migrations idempotent:每条 DDL 必须 `IF NOT EXISTS` 或包在 schema 检查里,可重跑

### 维度 4:输入校验与边界 (CWE-20, 190, 400, 1284)

- 浮点 NaN / Inf:前端传来的 `f64` 入 Rust 必须先 `is_finite()` + 范围校验,绝不能 `as i32` 静默吞 0(用户已踩过)
- 整数溢出:`+` / `-` / `*` 用户控制的输入必须 `checked_*` 或 `saturating_*`(用户已踩过)
- 缓冲区上限:`Vec::with_capacity(user_input)` / `String::with_capacity(user_input)` 必须有上限
- 反序列化深度:`serde_json` 嵌套对象 / 数组层数无限会栈溢出
- DoS 输入:超大 payload(如几 MB JSON)/ 高频请求是否有节流

### 维度 5:并发与资源管理 (CWE-362, 367, 400, 833)

- 死锁:`Mutex` / `RwLock` 嵌套获取顺序是否一致;持有 lock 跨 `await` 是异步 deadlock 大坑
- Data race:`Arc<Mutex<T>>` 共享状态读写顺序,特别是"读-判断-写"非原子段
- TOCTOU:状态先写、API 后调、API 失败状态卡死(用户已踩过 start_drag);应当 API 成功才写状态
- 资源泄漏:`File` / `tokio::sync::mpsc` sender / `MutexGuard` 持有期是否过长;`Drop` 是否被显式 forget
- async 阻塞:`std::thread::sleep` / 同步 IO / `block_on` 在 tokio runtime 中会阻塞 executor

### 维度 6:错误处理与 panic (CWE-209, 248, 754)

- 生产路径 `unwrap()` / `expect()`:除明确"不变式保证不会 None/Err"外,应转 `?` + `thiserror::Error`
- 错误吞没:`let _ = result` / `result.ok()` 丢错;`if let Err(_) = ...` 不报告
- 错误信息泄露:错误 `Display` 或日志是否带文件路径 / 配置内容 / 内部 SQL / 凭证片段
- 未处理异常路径:`Option::None` / `Result::Err` 分支被 unwrap 或 default,产生静默错误

### 维度 7:信息泄露(隐私漏洞) (CWE-200, 359, 532)

**关键约束:** 不读 `messages.content`(消息正文) / 应用名 / 窗口标题 / 麦克风 / 截屏 / 剪贴板 — 这是项目隐私边界硬要求(CI 已 grep 三个固定字符串,本维度补语义层)。

- 任何代码路径(直接或间接)是否调用了上述敏感系统 API;新增的 service / IPC / FFI 是否绕过 CI grep
- 日志 / `eprintln!` / `debug!` / `tracing::info!` 是否带 PII(用户消息 / 凭证 / 路径含 username)
- telemetry 上报字段是否包含上述敏感数据
- DPAPI `unprotect` 后明文 `String` 是否过早进日志或长生命周期变量

### 维度 8:LLM 越狱与 prompt injection (OWASP LLM01, LLM02, LLM07)

- 用户输入(`.soul.md` 自定义人格 / 聊天消息 / 文件名 / 装扮元数据 / 拖入文件内容)是否净化后才进 prompt;输入是否能注入"忽略以上指令"等控制语
- 系统安全前缀拼装顺序:必须 `[安全前缀] + [人格] + [用户输入]` 不可被覆盖;检查任何允许拼接顺序变化的代码路径
- LLM 工具调用是否有沙箱 / 白名单 / 二次确认;模型给的 shell 命令 / 文件路径不可直接执行
- LLM 输出反序列化:模型给的 JSON / 代码片段不可 eval / `serde_json::from_str` 不带白名单字段过滤即危险

## 输出结构

写到 `progress/code-review-{YYYY-MM-DD}.md`(同日多次跑追加 `## Run` 节,不覆盖):

```markdown
# Code review {YYYY-MM-DD}

## Run {HH:MM:SS} — range: {range_label}

**审查文件清单**({N} 个):
- src-tauri/src/services/X.rs
- src/views/Y.vue
- ...

**总裁决:** Pass / Conditional / Block

**各维度命中数:**
| 维度 | Critical | High | Medium | Low | Won't fix |
|---|---|---|---|---|---|
| 1 注入与执行 | 0 | 0 | 0 | 0 | 0 |
| 2 加密与凭证 | 0 | 0 | 0 | 0 | 0 |
| 3 数据完整性 | 0 | 0 | 0 | 0 | 0 |
| 4 输入校验与边界 | 0 | 0 | 0 | 0 | 0 |
| 5 并发与资源管理 | 0 | 0 | 0 | 0 | 0 |
| 6 错误处理与 panic | 0 | 0 | 0 | 0 | 0 |
| 7 信息泄露 | 0 | 0 | 0 | 0 | 0 |
| 8 LLM 越狱 | 0 | 0 | 0 | 0 | 0 |

### Critical (must-fix:可被外部触发的漏洞 / 凭证泄漏 / unsafe 内存安全 / SQL 注入)

#### C-N [`<file>:<line>`] CWE-XXX 一句描述

具体漏洞细节(2-4 行解释 / 触发条件 / 影响范围)。

- **修复:** `<file>:<line>` 改 ...
- (可选)更深入修复路径:换库 / 重构 / 加测试

### High (should-fix:数据完整性 / 整数溢出 / TOCTOU / panic 路径 / DoS 输入)

(同结构,编号 H-N)

### Medium (errno 处理粗糙 / 非生产路径的 unwrap / 错误信息泄露调试细节 / 防御性 hardening)

(同结构,编号 M-N)

### Low (低危 nits,**最多 5 条**;超过的合并为"plus N similar items")

(同结构,编号 L-N)

### Won't fix(false positive,标 CWE 不适用理由)

#### WF-N [`<file>:<line>`] 看似 CWE-XXX 但 ... 不适用,理由:...

### Verification plan

整段给一组可执行命令验证现状 + 修复后再验证。例:

\`\`\`bash
# 已通过的单元测试覆盖度(基线)
cd src-tauri && cargo test

# 修复后跑同样命令,期望 N+M passed
\`\`\`

每条 Critical / High 修复建议可在 verification plan 段统一给"修复后跑哪些测试"。**不要在每条 finding 内重复 verification 命令**,集中给更可执行。
```

## 审查后状态更新

写完报告后,**必须**更新 `progress/.audit-state`:

```json
{
  "last_audit_sha": "<HEAD 完整 SHA>",
  "last_audit_date": "<RFC3339 时间戳>",
  "last_audit_range": "<range_label>",
  "last_audit_findings": { "critical": N, "high": N, "medium": N, "low": N }
}
```

`$since-last` 下次会读这里作基准。

## 执行规则

- **只读** — 任何业务代码、文档不可修改;只能写 `progress/code-review-{date}.md` 与 `progress/.audit-state`
- **不评判产品决策** — local-first / 用户自主权 / 非养成 / 性能预算这些不是漏洞维度;遇到设计层面的疑虑只标"信息观察",不入 finding
- **每条 finding 必须有 CWE/OWASP 编号** — 没有编号的不算漏洞
- **false positive 必须标理由** — Won't fix 段必须给"为什么 CWE 不适用"
- **Low 最多 5 条** — 超过合并为"plus N similar items",不刷数
- **报告语气** — 直接、可执行、不要长篇大论;每条 finding ≤ 3 行

## 输出

最终终端输出 5 行以内:

1. 审查范围 `range_label` + 文件计数
2. 总裁决 + 各级数量(Critical X / High X / Medium X / Low X)
3. 报告路径 `progress/code-review-{date}.md`
4. 状态文件已更新到 `<HEAD SHA>`
5. 建议下一步(如"先修 N 条 Critical 再 ship-task" / "可放行 ship-task")

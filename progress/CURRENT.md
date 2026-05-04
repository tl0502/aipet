# Current State

> 任何 agent 入会必读此文件(启动协议步骤 2)。完成 1 个 task 后必更新此文件。

- **Milestone**:M1 W1 D3(进行中)
- **Active branch**:`feat/m1-d2-window-interaction`
- **Last commit**:`8960b24 chore(audit): /code-audit Run 2 补审 — 33/33 文件全覆盖`(本笔 hardening A 桶 4 条尚未 commit)
- **Tag**:none yet(M1 出口达成后打 `v0.M1.0`)
- **Last updated**:2026-05-04(code-audit A 桶 4 条 hardening:M-1 crypto 整数截断 + M-3 nickname UPSERT 子查询丢 user_nickname + M-4 Mutex .lock().unwrap() poison 链 + L-7 release console.log 未 strip;cargo test 27 → 31 passed)

---

## In-Progress Modules

| Module | Story | Owner | Status | Next |
|---|---|---|---|---|
| (无)| — | — | — | M1 D2 已收口,等待启动 D3 |

---

## Recently Completed(本 milestone)

| Story | Commit | Date |
|---|---|---|
| A.1 透明窗口 + 置顶 + skipTaskbar | `8a37728`(scaffold 时已就绪) | 初始 |
| A.2 VRM 渲染 momo + Three.js + spring bone | `2f3b28c` | 2026-05-02 |
| A.3 hitbox 上报 + 拖动 + 智能穿透 + 边缘吸附 | `2f3b28c` + `04f9abc`(DPR 修复) | 2026-05-02 |
| 文档对齐 v1.0 → VRM | `908a6bd` | 2026-05-02 |
| Scaffold tweaks(端口 1430、vrm gitignore、vite path) | `eadad55` | 2026-05-02 |
| vibecoding 工程支撑层(CLAUDE.md / .claude/agents/ / progress/) | `a0910f2` | 2026-05-02 |
| CI 加 PII 静态扫描 + cargo test | `89de30e` | 2026-05-02 |
| A.3 DPR 缩放修复 + VRM-fail 拖动 fallback | `04f9abc` | 2026-05-02 |
| A.4 系统托盘(显示/隐藏 + 退出 + 关窗拦截) | `fbe2672` | 2026-05-02 |
| A.5 全局快捷键(`Ctrl+Alt+Space` / `Ctrl+Shift+B` 占位实现) | `da0a6ad` | 2026-05-02 |
| capabilities/default.json 修复 frontend event listen | `15a0551` | 2026-05-02 |
| ADR-015 对话面板三形态架构 起草 + Accepted | `2d4327c` + `8696fa2` | 2026-05-02 |
| B 步骤:progress 拆 stories(B.3.a-f 跨 M1-M5 + 智能穿透 II/III/I/IV 归位) | `2ac789a` | 2026-05-02 |
| A 步骤:PRD/架构/flows 升 v1.1(三形态 + ConversationStore + 控制按钮区) | `1bd45c7` | 2026-05-02 |
| 漏升修补:开发路线图 v1.1(§3.2 模块矩阵 + 头部摘要) | `435fc5e` | 2026-05-02 |
| **C 步骤起步:I.1 MigrationService**(24 表 SQLite v1 + tauri-plugin-sql 集成) | `cf1c0c5` | 2026-05-02 |
| vibecoding 工程支撑层 v2(agent frontmatter 升 2026 spec + 决策矩阵/IO 契约 + 2 slash commands + protect hook) | `6c67da2` → `87134da`(4 笔) | 2026-05-02 |
| vibecoding v2 hook ESM 冲突修复(`.js` → `.cjs`,package.json 含 `"type":"module"` 致 Node 强制 ESM 解析 require 崩溃) | `d1a65c7` | 2026-05-02 |
| .gitignore 升级(28 → 85 行,新增测试/缓存/Obsidian/运行时 DB/Bundle/编辑器临时分类)+ `git rm --cached` 移除已泄露的 .obsidian/ 整目录(含 obsidian-local-rest-api 的 API key + TLS 私钥) | `30cfb45` | 2026-05-02 |
| `/ship-task` 项目命令 + CLAUDE.md 流程补强:完成 task 后必须 progress + atomic commit + push 当前 `feat/*` 到 origin,避免本地 commit 未同步 GitHub | (本笔)| 2026-05-03 |
| **I.2 CryptoService**(Windows DPAPI 封装:`protect` / `unprotect` + `CRYPTOPROTECT_UI_FORBIDDEN` 防 UI;4 单测覆盖 round-trip / empty / binary safety / invalid ciphertext) | (本笔)| 2026-05-03 |
| **H.1 PersonaService MVP**(`include_str!` 编译进 binary 的 momo + gray_matter 解 frontmatter + sqlx 直连写 personas/persona_snapshots + ADR-009 漏交付补完 momo.soul.md;6 单测覆盖 parse 成功 / 坏 YAML / 缺 id / 未知 schema / schema v1 兼容 / frontmatter 切除) | (本笔)| 2026-05-03 |
| **F.1 MemoryService MVP**(messages 表 CRUD `insert/list/delete_by_id/delete_by_conversation` + ULID 主键 + summary 占位字符串 + cleanup_messages_older_than 私有 stub;**放弃 90 天自动清理**改默认无限保留 + 用户主动清理;6 单测覆盖 ULID/RFC3339 生成 / 不合法 role/mode 拒绝 / 全部 valid 组合接受 / cutoff 90 天计算 / placeholder 自识别) | (本笔)| 2026-05-03 |
| **F.2 NicknameService facade**(nicknames 单行表 + get_pet/get_user/set_pet/set_user/restore_pet 5 个 service + 5 个 IPC commands + nickname.changed event;set_pet 自动备份 previous,restore_pet 原子 swap 让用户可来回切;get_pet 三级 fallback nicknames → active persona → "默默";4 单测覆盖 event payload / 兜底常量) | (本笔)| 2026-05-03 |
| **DEV-1 开发期 Admin/Debug 面板**(独立 webview "dev" 窗口 + Ctrl+Shift+D 唤起 + 4 tabs:IPC Playground / Tables 查看 / Events 实时日志流 / Logs 占位;前后端全 `#[cfg(debug_assertions)]` + `import.meta.env.DEV` 双重排除,release 二进制零开发面板代码;白名单 5 张表 dev_query_table 防 SQL 注入;IPC playground 写死 12 个 commands 元数据,选中 → 表单填参 → 调用 → JSON 返回值;tsconfig 加 `vite/client` types 让 import.meta.env 可识别) | (本笔)| 2026-05-03 |
| **M1 D2/D3 hardening 5 笔**(start_drag 顺序倒置防 cursor_tracker 卡死 / hitbox NaN-Inf-超范围拦截 + i64 中间运算 / persona seed sqlx::Transaction + ON CONFLICT(persona_id,version) DO NOTHING / migrations/002 先去重再 CREATE UNIQUE INDEX / state.rs Hitbox::contains+expand 全 saturating 链 + 7 单测覆盖 i32::MAX/MIN 边界 / cursor_tracker `last_ignore: Option<bool>` 补 42bb4c7 init 语义 regression(老代码 loop 前显式 set true 在 42bb4c7 把 window get 进 loop 时丢失,导致启动 1-2s 实际不穿透);cargo test 27 passed) | `1677cef` + `b49b7bd` + `59fd182` + `8d72f9c` + (本笔 cursor_tracker)| 2026-05-03 |
| **/code-audit 命令落地 + 首次漏洞扫描**(`.claude/commands/code-audit.md` 8 维度纯漏洞扫描 + `.claude/agents/code-reviewer.md` SOP + `progress/.audit-state` 增量跟踪 + `.gitignore` 排除状态文件;基于 git diff 的增量审查,默认 `$staged`,显式 `HEAD~N..HEAD` / `$branch` / `$since-last` / `--full`;只查 8 维度技术漏洞[注入/加密/数据完整性/输入边界/并发/错误处理/信息泄露/LLM 越狱],不查业务/架构/产品决策/性能预算/测试覆盖率;首次审查 `$branch` 范围 19/33 文件 = Pass / 0 Critical / 0 High / 5 Medium / 5 Low / 3 Won't fix,产出 `progress/code-review-2026-05-03.md`)| `6f6a252` | 2026-05-03 |
| **/code-audit Run 2 补审**(用户复盘指出 Run 1 仅读 19/33 文件,补完前端 14 个 + Rust 辅助 6 个 = 20 文件;追加 ## Run 2 节到同日报告;累计 33/33 全覆盖,新增 7 Low + 3 Won't fix,无 Critical/High/Medium 增量;Low 集中在 vrm.ts H.2 用户导入未来反序列化预防 / useVRMModel console.log release 不 strip / 错误透传可能含路径 / Three.js lights 未显式 dispose / EventLog JSON.stringify 无 try/catch / window_snap i32 边界 / AppError dead_code Internal 路径泄露;累计裁决 Pass)| `8960b24` | 2026-05-03 |
| **code-audit A 桶 4 条 hardening**(按"修复 ROI"分桶后合并一笔:**M-1** crypto.rs 加 `InputTooLarge(usize)` 错误变体 + `check_input_size` helper 用 `u32::try_from` 替代 `len() as u32` 静默截断,3 单测覆盖 0/u32::MAX/over;**M-3** nickname.rs `set_pet_nickname` 改 explicit 两步 — 先 `SELECT user_nickname`,再 UPSERT 显式 bind 旧值,消除 INSERT 分支子查询返回 NULL 丢 user_nickname 的 cross-field side-effect 根因;**M-4** state.rs 加 `lock_or_recover<T>` helper(`unwrap_or_else(\|e\| e.into_inner())`)+ poison recovery 单测,window.rs / cursor_tracker.rs 共 5 处 `.lock().unwrap()` 全替换 — debug 模式 panic 后不再传播 panic 链导致全输入死锁;**L-7** vite.config.ts 改函数式 `defineConfig(({ command }))`,production build 时 `esbuild.drop = ['console','debugger']`,dev 期保留 console 调试体验;cargo test 27 → 31 passed)| (本笔)| 2026-05-04 |

---

## Blockers

(无)

> 历史 Blocker `缺 public/avatar/avatar.vrm` 已于 2026-05-02 由用户补充模型文件解除。

---

## Next 3 Tasks(优先级降序)

1. **B.1 LLMProvider**(1 day)— OpenAI 兼容 streaming chat completion + DPAPI 取 key(依赖 I.2 ✅)
2. **OpenClaw 文件操作 ADR-016 research**(1-2h)— 用户 2026-05-03 提出方向;F.2 落地后做研究 + 起草 Proposed
3. **A.6 智能穿透收口 polish**(0.5 day,D10 收口前)— II bbox 阈值降 IPC + III `tauri://moved` 立即上报修拖动错位

详见 [m1.md](m1.md) 完整拆解。

> **Backlog 已归位**(2026-05-02 B 步骤完成):智能穿透 4 项改进(plan a-5 Part A)已分散到 m1.md A.6(II+III, D10)/ m2.md A.7(I, N 前置)/ m2.md N.0(IV, N 期 backup)。无悬空 backlog。

---

## Recent Decisions(简版,详见 decisions-log.md)

- 2026-05-02:vibecoding 工程支撑层落地(CLAUDE.md / .claude/agents/ / progress/),协作模式定为单人 × 串行 session 文件驱动
- 2026-05-02:hitbox 坐标转换收口在 Rust 侧(读 `outer_position` + `scale_factor`),前端只发 CSS 像素 — 高 DPI 下 cursor_tracker 失效根因修复
- 2026-05-02:vibecoding 工程支撑层 v2(agent frontmatter 升 2026 spec + 决策矩阵 + IO 契约 + 2 slash commands + PreToolUse hook 拦截 _archive 与 ADR-001~014)
- 2026-05-02:vibecoding v2 hook ESM 冲突修复(脚本重命名为 `.cjs`,绕开 `package.json: "type":"module"` 强制 ESM 解析;6 项 stdin 用例全部通过 deny/放行预期)
- 2026-05-02:.gitignore 收口(13 类分组,补 vitest coverage / Vite 缓存 / Obsidian / SQLite 运行时 / Bundle / 编辑器临时);整个 `docs/AIPET-obsidian/.obsidian/` 改为 ignore 并 `git rm --cached`,堵住 `obsidian-local-rest-api` API key + TLS 私钥泄露口;⚠️ Follow-up:用户需在 Obsidian 端 rotate API key(旧 key 永久存在 git 历史,但仓库未公开,风险有限)
- 2026-05-03:`/ship-task` 固化 Git 收口流程:完成 task 后检查敏感文件/progress/验证,atomic commit 并 push 当前 `feat/*` 到 `origin`;CLAUDE.md 同步明确远程最新进度先看 feature branch,不是 main
- 2026-05-03:`module-implementer` agent 经 main session 实施期排查发现 frontmatter 缺 `tools` 字段未注册(对比 adr-author / doc-aligner / gate-checker 均有);补 `tools: Read, Write, Edit, Glob, Grep, Bash` 修复,vibecoding v2 的 4 个 agent 全部可用。同步 CLAUDE.md 决策矩阵下方加 **subagent 选用判断准则**(冷启动重读启动协议 vs main 已有上下文的性价比权衡:小任务 ≤ 0.5d + main 已勘察 → 直接做;模板化产出 / 跨 module 重构 / plan 审批 → 走 subagent)
- 2026-05-03:`module-implementer` 修好 frontmatter 后,继续测试 Explore / general-purpose 两个 subagent,发现**所有 subagent 路径在第三方 API 网关(`Calcium-Ion/new-api`)上 nil pointer panic**(显式 `model: opus` 也 500;不指定走默认模型则 400 "1m 上下文已经全量可用")。判定为网关 bug 而非 Claude Code 设计问题。决议:**当前阶段所有任务由 main session 直接执行**;CLAUDE.md 决策矩阵章节顶部加 ⚠️ 状态标注 + 末尾加「main 直接做的 4 类场景实操指引」(实施 / 决策起草 / 文档同步 / 出口检查),把 4 个 .claude/agents/<role>.md 当 SOP 参考而非 spawn 目标;网关修好撤本节,恢复 subagent 协作
- 2026-05-03:Claude 流程文件深度巡检后整体减冗:① CLAUDE.md "完成 task 必更 progress + atomic commit + push" 重 5 次 → 收口到 § 提交规范 唯一权威源,其他 4 处改"详 § 提交规范";② 启动协议第 3 步措辞中性化(「按场景」替代「被指派」);③ § Agent 决策矩阵 / § subagent 选用判断 / § 4 类场景 / § IO 契约 4 段重组为 2 段(当前模式优先 + 目标模式备查),每场景 2-3 行紧凑表达;④ adr-author / gate-checker / module-implementer 3 份 agent .md 的 module-implementer 引用同步成"main 直接做 / 网关修复后由 module-implementer";⑤ hook 注释 "由 adr-author 角色起草" → 「决策起草」SOP 中性表述。CLAUDE.md 137 → 130 行,信息密度提升,新 session 一打开就能定位「当前 subagent 不可用,main 直接做」
- 2026-05-03:**I.2 CryptoService 落地** — `src-tauri/src/services/crypto.rs` 用 windows 0.61 crate 直调 `CryptProtectData` / `CryptUnprotectData`,标志位 `CRYPTOPROTECT_UI_FORBIDDEN` 防止 DPAPI 弹 UI(隐私边界 #4 硬要求)。`CryptoError` 用 thiserror 包裹 `windows::core::Error`,与现有 `AppError` 风格一致。Cargo.toml 加 `Win32_Security_Cryptography` feature。4 单测全过(round-trip / empty / binary safety / invalid ciphertext);目前 `protect`/`unprotect` dead_code 警告会在 B.1 LLMProvider 接入后自然消除。
- 2026-05-03:**H.1 PersonaService MVP 落地** — ① 内置 momo 走 `include_str!` 编译进 binary(与 migrations/001_init.sql 同款),路径 `src-tauri/personas/_builtin/momo.soul.md`,M0 ADR-009 漏交付的 deliverable 顺手补完;② `gray_matter` (yaml feature) 解析 frontmatter,`parse_persona(&str)` 设计成纯字符串入参不耦合 IO,H.2 用户导入 / H.3 远程下载直接复用解析层;③ 关键风险触发 — `tauri-plugin-sql 2.4` 的 `DbPool::sqlite()` 公共方法被注释掉(wrapper.rs 行 37-64),Rust 端无法借 plugin Pool;按 plan 降级路径自开 `sqlx 0.8`(版本与 plugin 一致)短期连接,DB 路径用 `app.path().app_config_dir()` 与 plugin 一致;④ `personas` 走 `ON CONFLICT(id) DO UPDATE`,`persona_snapshots` 走 `(persona_id, version)` 唯一性守卫避免堆行;⑤ 6 单测覆盖 parse 成功/坏 YAML/缺 id/未知 schema/schema v1 兼容/frontmatter 切除。lib.rs setup 用 `tauri::async_runtime::spawn` 异步 seed,失败仅 eprintln 不阻塞启动。
- 2026-05-03:**F.1 MemoryService MVP + 设计偏离 90 天清理** — ① `services/memory.rs` 实现 messages 表 CRUD(insert/list/delete_by_id/delete_by_conversation)+ ULID 主键(`ulid` crate 1.x)+ summary 占位字符串 + `cleanup_messages_older_than(days)` 私有 stub;② **关键设计偏离**:m1.md F.1 字面 "90 天清理" 与 PRD §73 / 架构 §549 "默认 90 天 + is_deleted 软删" 措辞,经讨论后认定**偏离 local-first 精神**(数据已在用户本地,自动清空对话剥夺老朋友价值);改为**默认无限保留 + 用户主动清理**(类似 ChatGPT 网页),`cleanup_messages_older_than` 留 stub 不在 setup 调用,留给将来设置面板"X 天自动清理"开关触发;③ B.2 ChatService 拼 system prompt 时 context 爆炸问题改用**摘要压缩**(F.1 summary 占位是伏笔)而非删消息;④ PRD §73 + 架构 §549 措辞偏差留给后续 doc-aligner SOP 同步;⑤ 6 单测覆盖纯逻辑(ULID/RFC3339 生成、role/mode 校验、cutoff 计算、placeholder 自识别),DB 测试推到后续 milestone(架构有 testcontainer 设计)。
- 2026-05-03:**用户产品方向扩展** — 用户明确提出"AI 桌宠的文件操作能力也是有必要的,类似 OpenClaw"(陪伴 + 工具能力的双轨定位)。Follow-up:F.2 / B.1 落地后做 research(MCP / 文件读写权限模型 / 安全护栏与文件操作的边界)+ 起草 ADR(候选编号 ADR-016 桌宠工具能力)。本次不偏题。
- 2026-05-03:**F.2 NicknameService facade 落地** — ① m1.md 字面"读 user_state.user_nickname / pet_nickname"与实际 schema 不符,实际是 `nicknames` 单行表(id=1 CHECK + pet_nickname / pet_nickname_previous / user_nickname / updated_at);**改用 nicknames 表**,精确且少一层间接,m1.md 的"user_state.xxx"措辞偏差留 doc-aligner 后续同步;② 5 个 service 函数(get_pet/get_user/set_pet/set_user/restore_pet)+ 5 个 IPC commands 注册 invoke_handler;③ get_pet 三级 fallback `nicknames.pet_nickname → active persona.name → "默默"`,UI 永远拿到非空字符串;④ set_pet 通过 ON CONFLICT UPSERT 自动把当前值搬到 previous(为 restore 备份);⑤ restore_pet 用 SQLite 原子 swap(SET pet_nickname = pet_nickname_previous, pet_nickname_previous = pet_nickname),用户可来回切两个曾用名;⑥ emit `nickname.changed` { which: "pet"\|"user", value } 与架构 §711 IPC event 契约 byte-perfect 对齐;⑦ 4 单测覆盖事件 payload 序列化 / null value / 兜底常量 / event 名 — 与 persona/memory 同模式不测 DB。
- 2026-05-03:**6 笔本地 commit push 到 origin** — 远程 SSH 22 在大陆 connection reset,SSH 443 也超时,改用 HTTPS push 一次性同步(`b534877..147eed6`);git credential helper 已 cache PAT,后续 push 无需重新认证。CLAUDE.md / `/ship-task` 流程未变,未来 push 走 HTTPS remote 自动生效。
- 2026-05-03:**git proxy 配 127.0.0.1:10808** — Windows 系统代理(常见 V2RayN/Clash HTTP 端口),浏览器走该代理能访问 GitHub 但 git CLI 没继承系统代理;`git config --global http.proxy + https.proxy` 配上后 HTTPS push 顺畅。VPN 关闭时端口未监听 push 会卡(connection refused),需要时 `git config --global --unset http.proxy` 切回直连。
- 2026-05-03:**DEV-1 开发期 Admin/Debug 面板落地** — ① 用户提出:当前桌宠主窗口透明 320×320,后端 service / IPC / event 全是隐形的,F.2 nickname 5 个 IPC 想测都没入口;改进开发体验。② 实施方案:独立 webview 窗口 label="dev",Ctrl+Shift+D 唤起,前端 hash 路由 `index.html#dev` + `import.meta.env.DEV` 守卫 + Vite tree-shake / Rust `#[cfg(debug_assertions)]` 双重排除 release。③ 4 个 tab:**IPC Playground**(12 个 commands 元数据写死前端,选中 → 表单填参 → invoke → JSON 返回值)/ **Tables**(白名单 5 表 + sqlx Row 反射拼 JSON,前 100 行 grid 展示)/ **Events**(订阅架构 §711 列出的事件,实时滚动)/ **Logs**(MVP 占位,待 ringbuffer logger M1 D6+ 接入)。④ tsconfig 加 `vite/client` types(原本只有 @tauri-apps/api,不识别 import.meta.env)。⑤ 工作量符合预期 ~0.5d;后续每个 milestone 持续受益,不只 M1。
- 2026-05-03:**M1 D2/D3 hardening 5 笔**(`1677cef` window.rs + `b49b7bd` migrations/002 + persona tx + `59fd182` import 清理 + `8d72f9c` state.rs saturating + 本笔 cursor_tracker)— ① **start_drag 顺序倒置**:先 `start_dragging()` 成功才 `is_dragging=true`,避免拖拽 API 失败但状态写入,cursor_tracker 永远卡在 `is_dragging=true` 短路 hitbox 判断,前端 await reject 后又永不调 stop_drag → 桌宠吃光全屏鼠标的 P1 bug;② **hitbox 输入加固**:`validate_hitbox_input` 拒绝 NaN/Inf/abs > 1e6,缩放后用 i64 中间值 + 显式 i32 边界检查,堵 `f64 as i32` 静默吞 NaN 成 0 的路径;③ **persona seed 原子化**:`seed_builtin` 改 `sqlx::Transaction` 包 upsert_persona + insert_snapshot,改 `ON CONFLICT(persona_id, version) DO NOTHING`;④ **migrations/002 唯一索引**:先 `DELETE WHERE rowid NOT IN (SELECT MIN(rowid) GROUP BY persona_id, version)` 兜底老数据,再 `CREATE UNIQUE INDEX IF NOT EXISTS` —— 与 ③ **强绑定**,无此索引时 `ON CONFLICT(cols)` 子句**无操作**(SQLite 要求 conflict target 列上有 UNIQUE 才生效);⑤ **state.rs Hitbox 边界 saturating**:`contains` 用 `saturating_add` 防 `x+w` overflow,`expand` 用 `saturating_sub/add/mul` 全链饱和;新增 7 单测覆盖 i32::MAX/MIN 边界 `cargo test 20 → 27 passed`;⑥ **cursor_tracker init 语义补 42bb4c7 regression**:`42bb4c7` 把 `window.get()` 移进 loop 时丢了 loop 前显式 `set_ignore_cursor_events(true)` 初始化,纯内存 `last_ignore=true` 与窗口默认 false 不一致 → 启动 1-2s 桌宠区域实际不穿透;改 `last_ignore: Option<bool> = None`,首 tick `None != Some(*)` 必触发 set,滞后区判断 `unwrap_or(true)` 保持默认语义。**关键学习**:① SQLite `ON CONFLICT(cols)` 必须配 `UNIQUE` 约束才生效,否则是哑的 — 后续加 ON CONFLICT 子句必查 schema;② Rust 后端不信前端浮点输入,`as i32` 把 NaN 静默转 0、超范围 saturating,必须显式 `is_finite()` + 范围 + 复合值边界三道校验;③ 重构线程循环结构时,初始化与状态变量必须同步检查 — 内存状态 `last_ignore=true` 不等于"窗口实际穿透"。
- 2026-05-03:**`/code-audit` AI 漏洞审查命令落地 + 首次扫描** — ① 解决问题:`/ship-task` 是机械门(typecheck/lint/cargo check/敏感文件 grep),不覆盖语义级漏洞;hardening 5 笔(start_drag 顺序倒置 / hitbox NaN 静默吞 / persona seed `ON CONFLICT` 缺 UNIQUE 哑 / Hitbox saturating / cursor_tracker init regression)中 4/5 是机械验证盲区,审查 ROI 用真实数据验证。② 命名 `/code-audit` 避免与 Claude Code 内置 `/review` (Review a pull request) 冲突 — Anthropic 官方 review 走 `gh pr view` PR 上下文,与本项目 feature branch 单人推进无 PR 场景不匹配。③ 设计为基于 git diff 的增量审查,而非全库扫描 — 默认 `$staged`(commit 前最后一道语义门),支持 `HEAD~N..HEAD` / `$branch` / `$since-last` / `--full`;`progress/.audit-state` 跟 `last_audit_sha` 自动作 `$since-last` 基准。④ 范围严格 8 维度纯漏洞(注入/加密/数据完整性/输入边界/并发/错误处理/信息泄露/LLM 越狱),**不查**业务/架构/产品决策/性能预算/测试覆盖率(那是 doc-aligner / gate-checker / 用户人工 review 处理);每条 finding 必须给 CWE/OWASP 编号 + 修复路径。⑤ 文件过滤:`src/**/*.{ts,vue,js}` + `src-tauri/src/**/*.rs` + `migrations/*.sql` + `Cargo.toml`/`package.json` 依赖行入审,`docs/` `progress/` `.claude/` `*.md` `*.vrm` `.github/workflows/` `tsconfig*.json` 默认跳过;`*.test.ts` / `#[cfg(test)]` 仅 `--include-tests` 启用。⑥ 首次审查范围 `$branch`(33 文件),裁决 **Pass**:0 Critical / 0 High / 5 Medium / 5 Low / 3 Won't fix。Medium 集中在 `crypto.rs`(整数截断 `len() as u32` + LocalFree OOM 时不释放)/ `nickname.rs`(UPSERT 子查询在 nicknames 行 DELETE 后丢 user_nickname,需 explicit 两步)/ 全 Mutex `.lock().unwrap()` poison 风险(debug 模式不 panic_abort)/ `dev.rs` SQL `format!()` 路径仅靠 5 表白名单。Critical/High 0 = M1 D2/D3 hardening 5 笔后代码姿态显著提升,现存 finding 多为防御性 hardening。⑦ 配套写 `.claude/agents/code-reviewer.md` 作 SOP,subagent 网关修复后可立即 spawn;当前 main 直接照 SOP 跑。⑧ 触发节奏:日常 commit 前 `$staged`(高频,5-30s)/ module 完成 `$branch` / milestone 末必跑 / 进入新风险领域(unsafe Rust / Win API / LLM 接入) / 历史回溯 `<SHA>..<SHA>`。⑨ 不在范围:GitHub Action `claude-code-action` 推迟到 M1 出口准备开 PR 时再加;依赖 CVE 审计交给 `cargo audit` / `pnpm audit` 专业工具,本命令只判"是否引入了可疑的新依赖名"。
- 2026-05-04:**code-audit A 桶 4 条 hardening 闭环** — 用户问"code-review-2026-05-03.md 的漏洞是否值得修",经按"修复 ROI"分桶后选 A 桶 4 条合并一笔(M-1 crypto 整数截断 / M-3 nickname UPSERT 丢 user_nickname / M-4 Mutex poison 链 / L-7 release console 不 strip);其余 13 条按"延后到对应模块落地时"(M-2 / L-12 / L-6 等 → B.1 / I.2 / H.2 触发)和"不修"(false positive 级 / 无外部攻击面)分类。**关键学习**:① M-1 把 size check 抽成 `check_input_size(usize) -> Result<u32, _>` 纯函数,单测能精准覆盖边界(0 / u32::MAX / over)而无需构造非法 slice 引入 UB,比直接 `if len > u32::MAX as usize` 更可测;② M-3 explicit 两步比子查询 UPSERT 健壮 — INSERT 分支子查询返回 NULL 是 SQL UPSERT 的经典坑,应当默认避免;③ M-4 `lock_or_recover` 放 `state.rs` 与 `Hitbox` saturating helper 同模块同测试位,作 AppState 周边安全工具集;④ L-7 vite 顶层 `esbuild` 选项会同时影响 dev transform 与 build minify,改函数式 `defineConfig(({ command }))` 按 command 分支才能 dev 保留 console、prod 打掉;Run 1 报告写的 `terserOptions` 对当前 `minify: 'esbuild'` 不适用,实际语法是 `esbuild.drop`。cargo test 27 → 31 passed,typecheck/lint/cargo check 全绿。

---

## 路线图速查

- M1(W1-W2):壳层 + 对话 + Onboarding
- M2(W3-W4):任务三件套 + 物理交互
- M3(W5-W6):记忆 + 主动陪伴
- M4(W7-W8):装扮 + 声音 + 纪念日
- M5(W9-W10):小游戏 + 灰度 + RC

详见 [docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md](../docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md)。

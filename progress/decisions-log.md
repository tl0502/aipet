# 实施期决策日志

> 实施期(M1-M5)中临时决策的流水账。**够不上 ADR 的小决定**记这里;**重大决定**走 `M0-ADRs/ADR-015+`(由 adr-author 角色起草)。

格式:`YYYY-MM-DD | <scope> | <one-line decision> | <rationale> | <commit/PR ref if any>`

---

## 2026-05

### 2026-05-02 | vibecoding 工程支撑层

- **决策**:协作模式定为单人 × 串行 session,文件驱动(progress/CURRENT.md 为单一可信状态),不引入 TaskList/TeamCreate 运行时依赖
- **理由**:独立开发者跨 session 工作,需要状态完全沉淀到 git repo;运行时依赖增加复杂度且 disaster recovery 难
- **影响**:CLAUDE.md / .claude/agents/ / progress/ 全套文件新增
- **Ref**:`a0910f2`

### 2026-05-02 | M1 D2 commit 拆分策略

- **决策**:545 行未提交工作拆为 3 笔(原 plan 4 笔)— `docs(baseline)` / `chore(scaffold)` / `feat(m1-d2)`,后者合并 VRM 渲染 + 窗口交互
- **理由**:`PetCanvas.vue` 同时依赖 VRM Runtime 与 IPC commands(updateHitbox/startDrag/stopDrag),拆分会导致中间 commit build-broken;合并到 1 笔保证 atomic 且每笔可独立 build
- **影响**:Git 历史 3 笔新 commit + 1 笔原 scaffold,共 4 笔
- **Ref**:`908a6bd` / `eadad55` / `2f3b28c`

### 2026-05-02 | 分支命名清理

- **决策**:分支 `feat/m1-d2-live2d-window-interaction` → `feat/m1-d2-window-interaction`
- **理由**:M0 末已切 VRM,分支名残留 live2d 易误导;改名零成本
- **Ref**:`git branch -m` 操作,无需 commit

### 2026-05-02 | Vite 端口 1420 → 1430

- **决策**:开发端口从 Tauri 默认 1420 改为 1430,vite.config.ts + tauri.conf.json 同步
- **理由**:1420 在某些环境被占用;同时 vite.config.ts 用 path.resolve 锁绝对路径,绕开中文目录(D:\Project\ai桌宠)解析问题
- **Ref**:`eadad55`

### 2026-05-02 | favicon stub

- **决策**:`index.html` 加 `<link rel="icon" href="data:," />`
- **理由**:消除 dev 控制台 favicon 404 噪声;实际 icon 在 src-tauri/icons/ 由 Tauri 注入
- **Ref**:`eadad55`

### 2026-05-02 | hitbox 坐标转换收口在 Rust 侧

- **决策**:`update_hitbox` IPC 改接收 CSS 像素(`cssX/Y/W/H: f64`),Rust 侧用 `window.outer_position()` + `window.scale_factor()` 转屏幕物理像素后再存 `AppState::pet_hitbox`
- **理由**:
  1. **修真实 bug**:原版前端发 CSS 像素,后端用 Win32 `GetCursorPos`(物理像素)比较 — 高 DPI(1.25/1.5/2.0)下 hitbox 永不命中,`set_ignore_cursor_events` 始终 true,整窗穿透 → 用户既不能点击也不能拖动
  2. **零授权成本**:替代方案是前端调 `@tauri-apps/api/window.scaleFactor / outerPosition / onMoved / onScaleChanged`,需要在 capabilities 加授权;本应用 capabilities 当前为空,从 Rust 侧拿避免引入新授权面
  3. **单点真相**:Rust 是物理像素权威源,坐标转换逻辑全收口在一处,前端无须知道 DPR / 窗口位置
- **影响**:`update_hitbox` 命令签名变更(`x,y,w,h: i32` → `cssX,cssY,cssW,cssH: f64`);前端 `PetCanvas.vue` 简化(去掉 `window.screenX` 累加,只发 `rect.left + bounds.x` 等 CSS 像素)
- **附带修复**:VRM 加载失败时上报整 canvas 区域作 fallback hitbox(避免用户陷入"窗口完全穿透"死锁);`mouseup` 监听从 setup 顶层移到 `onMounted`(避免 SSR / HMR 边界问题)
- **Ref**:`04f9abc`

### 2026-05-02 | 智能穿透维持当前方案(业界对比结论)

- **决策**:维持 `set_ignore_cursor_events + 60Hz GetCursorPos 轮询 + AABB hitbox + 5px 滞后区` 架构,不改用 SetWindowRgn / forward 钩子 / readPixels 像素精确等"理论更优"方案
- **理由**:
  1. **Tauri 业界主流**:Manasight 等 Tauri overlay 应用在 Win10/11/macOS Sonoma/Sequoia/Tahoe 多版本生产验证通过
  2. **跨平台一致**:P2 macOS/Linux 扩展时无需重做(SetWindowRgn / WM_NCHITTEST 仅 Win,且 Chromium 嵌入下行为存疑)
  3. **不违反隐私边界**(关键约束 4):Electron 风格的 `setIgnoreMouseEvents(forward: true)` 依赖全局鼠标钩子(`SetWindowsHookEx WH_MOUSE_LL`),不仅 Tauri 未实现(issue #6164),且会被某些 AV 软件误报(关键风险 r2)
  4. **性能足够**:60Hz GetCursorPos 是 Win32 微秒级系统调用,实测 CPU < 0.1%
- **备选改进选项**(详见 plan `a-5-immutable-aurora.md` Part A,无强优先级):
  1. hitbox 多 sub-mesh 取并集(0.5d)— 边缘宽松度 10-20% → 5-10%
  2. bbox 变化阈值降 IPC(0.2d)— idle 4Hz → < 1Hz
  3. listen `tauri://moved` 立即上报(0.3d)— 拖动结束消除 250ms 错位
  4. 5 秒一次 readPixels 真实 alpha mask 对比埋点(0.5d)— 不动 runtime
- **不推荐**:SetWindowRgn(Chromium 兼容性未知,需 raw windows crate);Electron forward 钩子(违反隐私边界);60Hz readPixels(GPU readback 卡顿)
- **Ref**:plan `a-5-immutable-aurora.md` Part A;Tauri issue #2090 / #6164 / #13070

### 2026-05-02 | A.5 全局快捷键 M1 范围最小可行

- **决策**:M1 阶段两个全局快捷键都做"最小可行占位",不做完整 BossKeyService / ChatPanel 集成
  - `Ctrl+Alt+Space` → `toggle_pet()` + `emit("shortcut:chat")`
  - `Ctrl+Shift+B` → `hide_pet()` + `emit("shortcut:boss-key")`
- **理由**:
  1. B.3 ChatPanel 与模块 K BossKeyService 分别在 M1 后期 / M2 实现,A.5 作为"快捷键链路"提前到 M1 D3 是为出口"快捷键稳定"的 KPI 服务
  2. 占位实现已能让用户用 Ctrl+Alt+Space 唤起桌宠、用 Ctrl+Shift+B 临时隐藏 — 单机最小价值已达成
  3. 抽出 `services/window_actions.rs` 作为 tray + shortcuts 的共享层,B.3/BossKeyService 接管时只需替换前端事件 handler,Rust 端无需重构
- **影响**:Cargo 加 `tauri-plugin-global-shortcut`(target-cfg desktop);新建 `services/{window_actions, shortcuts}.rs`;tray.rs 重构去内部 helper;前端加 `useShortcutListener` composable 占位监听
- **不做**:① 不装 npm `@tauri-apps/plugin-global-shortcut`(前端不调 register/unregister)② 不做用户自定义键位(M3)
- **Ref**:`da0a6ad`,plan `a-5-immutable-aurora.md` Part B

### 2026-05-02 | capabilities/default.json 必加(更正先前误判)

- **更正对象**:同日"A.5 全局快捷键 M1 范围最小可行"中"不加 capabilities" 的判断
- **背景**:A.5 部署后用户在控制台看到 `event.listen not allowed. Permissions associated with this command: core:event:allow-listen`
- **根因分析**:
  - Rust 端 `app.emit()` 与 `listen_global()` 不走 capability 校验(crate 内部调用)
  - **但**前端 `@tauri-apps/api/event.listen` 走的是 plugin event 提供的内置 command,默认 deny,必须有 capability 授权
  - 同理:前端用 `@tauri-apps/api/window` 的窗口操作、`@tauri-apps/api/path` 的路径解析等也都需要 capability
  - 我们之前能用 `invoke('start_drag')` 是因为 `start_drag` 是我们自己 `invoke_handler` 注册的命令,不走 plugin command 的 capability 链
- **决策**:新建 `src-tauri/capabilities/default.json`,permissions 为 `core:default`(集合,包含 event/webview/window/path/app/resources 各 default 子集)
- **不包含**:fs/shell/dialog/notification 等敏感插件权限(M1-M3 不需要;M4 装扮 / M5 灰度更新 时按需扩展)
- **不包含**:`global-shortcut:*` 权限(因为我们前端不调 register/unregister,Rust 端注册即可)
- **影响**:`src-tauri/capabilities/default.json` 新增;`gen/schemas/capabilities.json` 在下次 build 时自动重新生成;无 runtime 行为变化(只解封被默认 deny 的 IPC)
- **Ref**:`15a0551`

### 2026-05-02 | ADR-015 对话面板三形态架构(升级为 ADR)

- **决策**:对话面板设计为 **3 形态共存 + ConversationStore 共享数据层** 架构,而非原 PRD §7.2 单一对话面板
  - 形态 1 hub 总面板(独立 Tauri 窗 `hub`,M4 实施)— 整合工坊 / 设置 / 对话 / 游戏 launcher
  - 形态 2 磁吸浮窗(独立 Tauri 窗 `chat`,M1 极简 → M2 完整)— 默认形态,Ctrl+Alt+Space 唤起,可吸附到角色窗或断开自由布置
  - 形态 3 漫画对话气泡(角色窗子组件,M5 实施)— 沉浸式,通过控制按钮区激活
- **理由**:对话是高频核心交互;桌宠产品差异化在沉浸式陪伴,单一对话窗浪费 VRM 视觉资产;view-agnostic ConversationStore 是工程上正交抽象,长期演进可复用
- **影响**:
  - B.3 单 story 拆为 6 子 story(B.3.a-B.3.f),跨 M1-M5 渐进交付
  - SQLite schema 加 `conversations` 表 + `messages.conversation_id` 索引
  - 控制按钮区作为模块 A 延伸(不属 B 模块),为 M2+ 多按钮预留扩展位
  - hub 与 GameRoom 共生(launcher 模式),ADR-012 不变
  - Onboarding 保持独立窗口,与 hub 解耦
- **TBD**(不阻塞 M1):
  - Q4 磁吸物理阈值(M2 W3 启动 B.3.c 前定)
  - Q5 控制按钮区按钮清单(M2 W3 启动 B.3.b 前定)
  - TBD-3 hub 与磁吸 chat 窗 conversation 同步语义(M4 启动 B.3.e 前定)
- **Ref**:`docs/AIPET-obsidian/M0-ADRs/ADR-015-chat-three-modes.md`(commit `2d4327c` 起草 + `8696fa2` Accepted)

### 2026-05-02 | B 步骤:progress 拆 stories + 智能穿透改进归位

- **决策**:依据 ADR-015 Accepted,把 B.3 单 story 拆为 6 子(B.3.a-f)分配到 m1-m5;同时把智能穿透 4 项改进 backlog(plan `a-5-immutable-aurora.md` Part A)归位到对应 milestone 而非搁置
- **拆分映射**:
  - m1.md:B.3 → **B.3.a 形态 2 极简版**(独立 chat 窗 + 单 conversation + 流式 + ESC/失焦/快捷键 toggle)
  - m2.md:加 **B.3.b 控制按钮区骨架**(模块 A 延伸,0.5d)+ **B.3.c 形态 2 磁吸交互**(1d)+ 入口前置 Q4/Q5 必拍板
  - m3.md:加 **B.3.d 多 conversation 左侧栏**(1d)+ L 模块加注"接收源扩展:形态 2/3 输入区 + 角色窗"
  - m4.md:加 **B.3.e hub 总面板**(2d 4 tab,工坊+设置由原独立窗收纳)+ 入口前置 TBD-3 必拍板
  - m5.md:加 **B.3.f 形态 3 漫画气泡**(1.5d)+ GameRoom 行注释 hub 共存 launcher
- **智能穿透改进归位**(plan a-5 Part A):
  - II bbox 阈值降 IPC(0.2d)+ III tauri://moved 立即上报(0.3d,**真实 bug**)→ m1.md **A.6** D10 收口前
  - I hitbox 多 sub-mesh 取并集(0.5d)→ m2.md **A.7** N 模块前置(N hitbox 触发率 ≥ 95% KPI 强相关)
  - IV 5 秒 readPixels alpha mask 对比 → telemetry(0.5d)→ m2.md **N.0** N 模块期 backup(应急,N 触发率 < 95% 才启用)
- **理由**:不搁置 backlog → 真实 bug(III)在 M1 收口前修;精度改进(I)放在 N 模块前置位置一气呵成;埋点(IV)作为 KPI 应急工具不消耗常规带宽
- **影响**:m1.md 总计从 12.5d → 13d(略紧但 D11 可吸收);Recently Completed 添加;无悬空 backlog
- **Ref**:`2ac789a`

### 2026-05-02 | A 步骤:PRD/架构/flows 升 v1.1

- **决策**:依据 ADR-015 Accepted,在 v1.0 基线文档之上做章节级增量(BASELINE 工作流约定:章节级新增升 v1.1,不压平)
- **PRD v1.1 改动**:
  - §7.1 加 7.1.1 控制按钮区(模块 A 延伸,M2 W3 上线)
  - §7.2 整段重写为 3 形态 + ConversationStore + 引用 ADR-015
  - §7.12 接收源扩展:角色窗 + 形态 2/3 输入区 + hub 对话 tab 输入区
  - 文档头加 v1.1 变更摘要
- **架构 v1.1 改动**:
  - §2.2 窗口模型表加 hub 行 + 注释 chat/pet/onboarding/game_room 与 ADR-015 关系
  - §3.1 ChatPanel 拆 ChatPanelView2 / ChatPanelView3 / HubChatTab + 加 ConversationStore service
  - §4 SQLite conversations 表加 title / archived 字段 + idx_conversations_active 索引
  - §5.1 IPC 加 conversation.list / create / rename / archive / delete / activate 6 命令
  - 文档头加 v1.1 变更摘要
- **flows v1.1 改动**:
  - §2 主流加形态选择注释 + 写入 messages 强调 conversation_id
  - 新增 §2.2 形态切换流(形态 2 ↔ 形态 3 数据保留 view 切换)
  - 新增 §2.3 磁吸状态机(吸附 ↔ 断开 + 失焦收缩 + 持久化坐标)
  - 文档头加 v1.1 变更摘要
- **不动**:人格 v1.0(与三形态正交)+ telemetry UAT v1.0(M2 加 conversation_switched 等再升)
- **影响**:BASELINE.md 同步标 v1.1;后续实施 B.3.a-f 各 story 时已有完整文档锚点
- **Ref**:本笔 commit

### 2026-05-02 | vibecoding 工程支撑层 v2 升级

- **决策**:在 v1(`a0910f2`)基础上为 4 个 agent 升 frontmatter 到 2026 spec(tools 白名单 + adr-author permissionMode: plan + description 加触发短语);CLAUDE.md 加 § Agent 决策矩阵 + § Agent IO 契约;新增 `/sync-progress` 与 `/check-baseline` 2 个 slash commands;新增 `.claude/hooks/protect-archive-and-adrs.js` PreToolUse hook 拦截 `_archive/` 与 ADR-001~014 的误改
- **理由**:
  1. 现状 4 agent frontmatter 只有 name + description,**全部**缺 tools / permissionMode 字段,命中 anti-pattern `MISSING_TOOLS_RESTRICTION`(prompt 里说"只读"但仅靠 LLM 记忆)与 `MISSING_TRIGGER`(description 缺 "Use proactively when X")
  2. CLAUDE.md 没有 agent 决策矩阵,新会话要翻 4 个 .md 才知道选哪个角色;subagent 不能 spawn subagent 但没说明 main 接力规则,命中 `WEAK_INTER_AGENT_CONTRACTS`
  3. 守则"❌ 不修改 _archive/ 与 ADR-001~014"全靠 LLM 记忆,无 hook 兜底,CI 也只能事后发现
- **影响**:
  - `.claude/agents/{adr-author,doc-aligner,gate-checker,module-implementer}.md` frontmatter +tools / permissionMode / trigger
  - `.claude/commands/{sync-progress,check-baseline}.md` 新增
  - `.claude/hooks/protect-archive-and-adrs.js` 新增 + `.claude/settings.json` 加 PreToolUse 配置
  - `CLAUDE.md` +§ Agent 决策矩阵 +§ Agent IO 契约
- **不做**:新增 agent(code-reviewer / perf-budget-checker / test-author);skills 化(性能预算速查独立);agent teams 启用(已 env var 启用但维持文件驱动);model 字段(全部 inherit);disallowedTools 字段(用 tools 白名单更稳)
- **依据**:Anthropic 2026 subagent spec — Tool Restriction Best Practices / Hooks 文档 / Slash Commands 文档
- **Ref**:`6c67da2`(笔 1) / `e89d660`(笔 2) / `42fce14`(笔 3) / `87134da`(笔 4)

### 2026-05-02 | vibecoding v2 hook ESM 冲突修复

- **决策**:`.claude/hooks/protect-archive-and-adrs.js` 重命名为 `.cjs`,绕开 `package.json: "type":"module"` 的强制 ESM 解析
- **理由**:hook 脚本用 CJS 语法(`fs.readFileSync(0)` stdin 读法)与 ESM 不兼容;改后缀让 Node 强制 CJS 加载即可,零代码改;比改写为 ESM 风险低(stdin 流式读法 ESM 异步语义更复杂)
- **影响**:6 项 stdin 用例验证全部通过 deny / 放行预期(写 `_archive/` deny / 写 ADR-001~014 deny / 写新 ADR-015+ 放行 / 读旁路放行 / 无 stdin 放行 / 解析失败放行);后续 `suggest-checks.cjs` 沿用 `.cjs` 范式
- **Ref**:vibecoding v2 升级 4 笔 commit 之内的修复

### 2026-05-02 | .gitignore 收口 + Obsidian local-rest-api 泄露口堵漏

- **决策**:`.gitignore` 重写为 13 类分组(VRM 二进制 / vitest coverage / Vite 缓存 / Obsidian 整目录 / SQLite 运行时 / Bundle / 编辑器临时 / OS 杂项 等);`docs/AIPET-obsidian/.obsidian/` 整目录加 ignore 并 `git rm --cached` 已追踪文件
- **理由**:`obsidian-local-rest-api` 插件配置含 API key + TLS 私钥被 git 追踪暴露;按目录 ignore + rm cached 一次性堵口;13 类分组 vs 平铺 list 可读性强,后续新增类目易归入
- **影响**:堵住 API key + 私钥泄露入仓库口;⚠️ Follow-up:用户需在 Obsidian 端 rotate API key(旧 key 已永久存在 git 历史,但仓库未公开,实际风险有限);后续 vitest coverage / SQLite 运行时文件不再误提
- **Ref**:vibecoding v2 升级期间的 hygiene 收口(同期 commit)

### 2026-05-03 | /ship-task 固化 Git 收口流程

- **决策**:新增 `/ship-task` slash command 固化"完成 1 task 标准收口"流程 — 检查敏感文件 / progress 同步 / 验证(typecheck/lint/cargo check)/ atomic commit / push 当前 `feat/*` 到 `origin`;CLAUDE.md § 提交规范 同步明确"远程最新进度先看 feature branch,不是 main"
- **理由**:此前每笔 task 完工要手动跑 5-7 步,易漏 push 导致本地与远程不同步;ship-task 一键收口 + 7 步顺序固化,减少 cognitive load;后续 audit-coverage P0 期在此基础上加"commit 后智能建议 7 类"
- **影响**:M1 D3 起每笔 task 完工调 `/ship-task`;远程进度 first-class 可信(再无"本地有,远程没"窘境)
- **Ref**:`.claude/commands/ship-task.md` 新增

### 2026-05-03 | module-implementer agent frontmatter 修复 + subagent 选用判断

- **决策**:`module-implementer` agent frontmatter 缺 `tools` 字段(对比 adr-author / doc-aligner / gate-checker 均有)未注册,补 `tools: Read, Write, Edit, Glob, Grep, Bash` 修复;CLAUDE.md § Agent 决策矩阵 下方加 **subagent 选用判断准则**
- **理由**:Claude Code 2026 spec 要求 subagent 必须显式声明 tools,否则不可用;判断准则(任务 ≤ 0.5d + main 已勘察 → 直接做;模板化产出 / 跨 module 重构 / plan 审批 → 走 subagent)解决"冷启动重读启动协议 vs main 已有上下文"性价比矛盾
- **影响**:vibecoding v2 的 4 个 agent 全部可用(注:此条修复后稍后 2026-05-03 测试发现网关 panic,见次条决议改为 main 直接做)
- **Ref**:`.claude/agents/module-implementer.md` frontmatter 补全

### 2026-05-03 | subagent 网关 panic + 决议 main 直接做

- **决策**:测试 Explore / general-purpose / module-implementer / adr-author / doc-aligner / gate-checker 共 6 个 subagent 路径,**全部** 在第三方 API 网关(`Calcium-Ion/new-api`)上 nil pointer panic(显式 `model: opus` 也 500;不指定走默认模型则 400 "1m 上下文已经全量可用");判定为网关 bug 而非 Claude Code 设计问题。**当前阶段所有任务由 main session 直接执行**;CLAUDE.md 决策矩阵章节顶部加 ⚠️ 状态标注 + 末尾加「main 直接做的 4 类场景实操指引」(实施 / 决策起草 / 文档同步 / 出口检查)
- **理由**:网关上游限制无解,等修复恢复 spawn 模式;4 个 agent .md 当 SOP 参考而非 spawn 目标 — 信息不浪费,流程不阻塞;CLAUDE.md 双轨表达(当前模式 + 目标模式)便于网关修好后撤本节
- **影响**:M1 W1 D3 起所有任务 main 直接做(实施 / ADR 起草 / 文档同步 / 出口检查);网关修好后撤本节,恢复 subagent 协作模式
- **Ref**:CLAUDE.md § Agent 决策矩阵 ⚠️ 状态段;`.claude/agents/*.md` 不动作 SOP 参考

### 2026-05-03 | I.2 CryptoService 落地(DPAPI 包装)

- **决策**:`src-tauri/src/services/crypto.rs` 用 windows 0.61 crate 直调 `CryptProtectData` / `CryptUnprotectData`,标志位 `CRYPTOPROTECT_UI_FORBIDDEN` 防止 DPAPI 弹 UI;`CryptoError` 用 thiserror 包裹 `windows::core::Error` 与现有 `AppError` 风格一致
- **理由**:**隐私边界 #4 硬要求** — DPAPI 弹 UI 会暴露用户在场感,违反"桌宠在用户主动唤起前不打扰"原则;windows crate 0.61 直调比 winapi 0.3 更现代;thiserror 风格统一便于 AppError 自动 From 转换
- **影响**:Cargo.toml 加 `Win32_Security_Cryptography` feature;4 单测全过(round-trip / empty / binary safety / invalid ciphertext);`protect`/`unprotect` 当前 dead_code 警告将在 B.1 LLMProvider 接入后自然消除
- **Ref**:M1 W1 D3 实施(具体 SHA 见 git log feat/m1-d2-window-interaction 分支)

### 2026-05-03 | H.1 PersonaService MVP 落地

- **决策**:① 内置 momo 走 `include_str!` 编译进 binary(与 migrations/001_init.sql 同款),路径 `src-tauri/personas/_builtin/momo.soul.md`;② `gray_matter`(yaml feature)解析 frontmatter,`parse_persona(&str)` 设计为纯字符串入参不耦合 IO;③ `personas` 走 `ON CONFLICT(id) DO UPDATE`,`persona_snapshots` 走 `(persona_id, version)` 唯一性守卫避免堆行
- **理由**:① M0 ADR-009 漏交付的 deliverable 顺手补完;② parse 层与 IO 解耦,H.2 用户导入 / H.3 远程下载直接复用;③ 关键风险触发 — `tauri-plugin-sql 2.4` 的 `DbPool::sqlite()` 公共方法被注释掉(wrapper.rs 行 37-64),Rust 端无法借 plugin Pool;按 plan 降级路径自开 `sqlx 0.8`(版本与 plugin 一致)短期连接,DB 路径用 `app.path().app_config_dir()` 与 plugin 一致
- **影响**:6 单测覆盖 parse 成功 / 坏 YAML / 缺 id / 未知 schema / schema v1 兼容 / frontmatter 切除;lib.rs setup 用 `tauri::async_runtime::spawn` 异步 seed,失败仅 eprintln 不阻塞启动
- **Ref**:M1 W1 D3 实施

### 2026-05-03 | F.1 MemoryService MVP + 设计偏离 90 天清理

- **决策**:① `services/memory.rs` 实现 messages 表 CRUD(insert/list/delete_by_id/delete_by_conversation)+ ULID 主键(`ulid` crate 1.x)+ summary 占位字符串 + `cleanup_messages_older_than(days)` 私有 stub;② **关键设计偏离**:m1.md F.1 字面"90 天清理"与 PRD §73 / 架构 §549 措辞,经讨论后认定**偏离 local-first 精神**(数据已在用户本地,自动清空对话剥夺老朋友价值);改为**默认无限保留 + 用户主动清理**(类似 ChatGPT 网页);③ `cleanup_messages_older_than` 留 stub 不在 setup 调用,留给将来设置面板"X 天自动清理"开关触发;④ B.2 ChatService 拼 system prompt 时 context 爆炸问题改用**摘要压缩**(F.1 summary 占位是伏笔)而非删消息
- **理由**:local-first 原则(关键约束 #1)优先级高于 PRD/架构具体措辞;F.1 是数据层基石,设计偏离需即刻拍板而非积压;摘要压缩对老朋友体验更好(对话连续性保留)
- **影响**:6 单测覆盖纯逻辑(ULID/RFC3339 生成、role/mode 校验、cutoff 计算、placeholder 自识别);PRD §73 + 架构 §549 措辞偏差留给后续 doc-aligner SOP 同步;DB 测试推到后续 milestone(架构有 testcontainer 设计)
- **Ref**:M1 W1 D3 实施

### 2026-05-03 | 用户产品方向扩展(OpenClaw 文件操作能力)

- **决策**:用户明确提出"AI 桌宠的文件操作能力也是有必要的,类似 OpenClaw"(陪伴 + 工具能力的双轨定位);本次不偏题,F.2 / B.1 落地后做 research(MCP / 文件读写权限模型 / 安全护栏与文件操作的边界)+ 起草 ADR(候选编号 ADR-016 桌宠工具能力)
- **理由**:陪伴 60% + 效率 40%(memory project_aipet_positioning)中"效率"维度此前定位偏窄(仅"对话回答问题"),OpenClaw 这条线打开"AI 主动操作用户文件"想象空间;但工具能力 = 引入 MCP / 文件 IO 权限,涉及隐私边界 #4 + 安全护栏 #5,需正式 ADR 拍板,不能临时决定
- **影响**:M1 W1 D3 用户原话备案,M1 末或 M2 启动 ADR-016 research(具体编号视 ADR 进度);本次不影响 F.2 / B.1 当前实施
- **Ref**:用户对话备案(无 commit)

### 2026-05-03 | 6 笔本地 commit push 到 origin(SSH→HTTPS 切换)

- **决策**:远程 SSH 22 在大陆 connection reset,SSH 443 也超时,改用 HTTPS push 一次性同步(`b534877..147eed6`);git credential helper 已 cache PAT,后续 push 无需重新认证
- **理由**:SSH 在大陆部分网络持续不稳,HTTPS + PAT cache 是稳定可重复的备用通道;CLAUDE.md / `/ship-task` 流程未变(只动 remote URL),未来 push 走 HTTPS remote 自动生效
- **影响**:`feat/m1-d2-window-interaction` 分支 6 笔本地 commit 全部上 origin;后续 push 无需 SSH 配置;origin URL 改 HTTPS 形态留本地配置(不进 git)
- **Ref**:`b534877..147eed6` 6 笔 commit

### 2026-05-03 | git proxy 配 127.0.0.1:10808

- **决策**:`git config --global http.proxy + https.proxy` 配 127.0.0.1:10808(Windows 系统代理常见 V2RayN/Clash HTTP 端口),让 git CLI 继承系统代理走 GitHub
- **理由**:浏览器走该代理能访问 GitHub 但 git CLI 没继承系统代理(Windows 默认行为);配上 git http(s).proxy 后 HTTPS push 顺畅
- **影响**:VPN 关闭时端口未监听 push 会卡(connection refused),需要时 `git config --global --unset http.proxy` 切回直连;长期 SOP:用户切代理时同步 git config(写入个人 setup checklist,不进 repo)
- **Ref**:个人 git 全局配置变更(不进 repo)

### 2026-05-04 | audit-coverage P0 落地 + plan §10 P1-3/P0-2

- **决策**:用户 2026-05-04 问"项目检查除纯代码漏洞还需要什么",引出 10 类盘点 C1-C10(✅ 2/10 + 🟡 3/10 + ❌ 5/10 缺位:C4 性能 / C6 依赖 / C7 构建 / C8 obs / C9 a11y);本期 P0 落地 5 份新 SOP + PostToolUse hook + ship-task 智能建议 + milestone-gate 总入口 + settings.local.json 漂移清理
- **理由**:
  1. **现有 4 个 agent + 4 个 command 不覆盖 5 类**,M1 末 gate-checker 想对账「内存 ≤ 250MB / bundle ≤ 80MB」时**无数据可对** — BASELINE.md § 性能预算速查 9 项数字从未跑过实测
  2. **三层智能触发设计完整闭环** — L1 hook 单文件视角(suggest-checks.cjs)/ L2 ship-task commit 完整性视角(7 类智能建议)/ L3 milestone-gate 跨期视角(7 步 orchestrator);缺任何一层都漏检
  3. **settings.local.json 110+ 条漂移**是 harness 唯一无自动维护机制处,P0-2 顺手清理 21 条 = A 桶 17(具体 PID / 一次性 URL / stale 端口 / 临时 echo / 一次性 git)+ B 桶 4(settings.json 完全重复)
  4. **plan §10.1 原 P0-1 / P0-3 作废**:ExitPlanMode 后 git status 暴露用户已在 ship-task.md 实现了"commit 后智能建议"段(比 P0-1 commit 前强制扫描更优)+ PostToolUse suggest-checks 已就 advisory(P0-3 SessionStart 增强职责重叠)
- **影响**:6 笔 atomic commit + 1 笔本地清理(.gitignore L54 排除):
  - `91dca81` 5 SOPs(/perf-check + /deps-audit + /release-check + /a11y-check + obs-checker agent)+ audit-coverage 报告 231 行
  - `eb764d6` PostToolUse `suggest-checks.cjs` 4 类 advisory hook + settings.json 注册
  - `cefc411` ship-task.md 加 7 类 commit 后智能建议 + code-audit.md Step 0 用 Bash 自取绕开第三方网关在 Win 中文路径 frontmatter 解析失败 + gate-checker.md 加 milestone 切换流程文件回归检查
  - `bed8442` CURRENT.md sync(audit-coverage 4 笔后)
  - `66114b5` /milestone-gate 总入口命令(plan §10.2 P1-3,7 步 orchestrator + 5 个 --skip-* + --gate-only)
  - `38dc2b7` CURRENT.md sync(P1-3 + P0-2 后)
  - 本地 settings.local.json 91 → 69 行(P0-2 21 条清理)
- **关键学习**:
  1. **研究阶段必先 git status** 看工作树状态,不要信任 CURRENT.md 的 Last commit 字段(本案 CURRENT 写 8960b24 实际 HEAD 已是 649ae5f)
  2. **plan 文件加「已确认实施范围」段**区分研究 vs 实施(本案 plan §12)
  3. **slash command 内不能程序化 chain 调其他 slash command**(SlashCommand 不在 Claude 工具集),`/milestone-gate` 等"总入口"命令是 main 主导让 Claude 照子 SOP 步骤逐个跑
  4. **认知错位的根源是「文档驱动 + 状态过时」的固有矛盾** — audit-coverage 报告 §5 表格写「✅ 落地状态」但 in-flight 未 commit 的状态没有表格能反映;修法不在工具,在流程(研究类任务必先 git status 抓 baseline)
  5. **本次合作模式**:用户先于研究做了 in-flight 工作,我帮收口 commit + 补 P1-3 + P0-2;事后用户主动让我"再检查一遍"才发现 CLAUDE.md / decisions-log / audit-coverage §5 三处治理同步缺口 — **整改后必须主动跑治理同步检查**,不能等用户问
- **不做**:① subagent 网关诊断(上游 Calcium-Ion/new-api 高并发限制无解)② 引入新 npm 依赖给 hook(项目硬约束最小依赖)③ ADR-016 vibecoding harness 体检(常规治理无需 ADR 形式)④ Co-Authored-By 签名补回(本次 6 笔 commit 已 push,history rewrite 风险高,后续 commit 注意)
- **Ref**:`91dca81` → `eb764d6` → `cefc411` → `bed8442` → `66114b5` → `38dc2b7` + `progress/audit-coverage-2026-05-04.md`(231 行)+ plan 文件 `~/.claude/plans/claude-vibecoding-claude-session-fluttering-anchor.md`(541 行,个人备忘)

### 2026-05-03 | Claude 流程文件深度巡检减冗(从 CURRENT 挤出归档)

- **决策**:CLAUDE.md 137→130 行减冗;5 处"完成 task 必更 progress + atomic commit + push"重复收口到 § 提交规范唯一权威源,其他 4 处改"详 § 提交规范";启动协议第 3 步措辞中性化(「按场景」替代「被指派」);§ Agent 决策矩阵 / § subagent 选用判断 / § 4 类场景 / § IO 契约 4 段重组为 2 段(当前模式优先 + 目标模式备查);adr-author / gate-checker / module-implementer 3 份 agent .md 同步成"main 直接做 / 网关修复后由 module-implementer";hook 注释 "由 adr-author 角色起草" → 「决策起草」SOP 中性表述
- **理由**:重复信息密度低 + 网关 panic 期 4 个 subagent 不可用 → 文档应优先反映「main 直接做」当前模式而非目标模式;每条信息 ≤ 1 处权威源,新 session 一打开就能定位「当前 subagent 不可用,main 直接做」
- **影响**:CLAUDE.md(主)+ adr-author.md / gate-checker.md / module-implementer.md / hook 注释
- **Ref**:见 CURRENT.md prior 同步(2026-05-04 已经过)

### 2026-05-04 | B.1 LLMProvider 落地

- **决策**:OpenAI 兼容 streaming chat completion + DPAPI 取 key,~1 day 实施完成
  - **架构层**:KISS struct 直暴 — `OpenAiCompatProvider` 单 struct,**不**抽 `trait LLMProvider`(架构 §6.1 字面有偏差);P1-R1 接 Anthropic 时再抽。doc-aligner 后续同步架构 §6.1 措辞
  - **secrets.rs 双 service**:`set` / `get` / `delete` 调 crypto::protect/unprotect 包装;UPSERT `ON CONFLICT(key) DO UPDATE`(沿用 H.1 / F.2 同款);key 命名 `{provider_id}.api_key`,7 单测覆盖 6 preset 命名 + 不冲突
  - **llm.rs 主体**:6 preset 静态数组(**与 ADR-005 字面差异:DeepSeek base_url 加 /v1 与其他 5 preset 对齐**;doc-aligner 后续同步 ADR-005)+ `parse_sse_payload` 纯函数(serde_json + `#[serde(default)]` 容忍 ollama / partial schema)+ `normalize_base_url` 幂等(去尾 / + 补 v1)+ `OpenAiCompatProvider::{new, from_secrets, chat_stream, ping}` + `ChatChunk::{Token, Done}` + `LlmError` 9 variant + 状态码映射 401→Unauthorized / 429→RateLimited / 5xx→Server(body 截 200 chars 防泄露)
  - **secrets.test 探活**:GET `/v1/models` — 零 token 消耗 + 无 cold start + 6 preset 全兼容(用户在 3 选项中拍板)
  - **dev_llm_test_stream**:debug-only e2e 验证(`#[cfg(debug_assertions)]` + lib.rs invoke_handler 双重排除 release);emit `dev.llm.token` / `dev.llm.done` 流到 dev panel Events tab(用户在 3 选项中拍板)
  - **不在范围**:ChatService prompt 拼装(B.2)、SecurityGuard 注入(B.2)、Anthropic / Gemini(P1-R1/R2)
- **理由**:① 抽象代价应由第二个具体类型支付,trait 在单 provider 时是命名空间无 dispatch 价值 ② GET /v1/models 是 OpenAI 标准探活端点,Ollama / DeepSeek / Moonshot / Qwen 全实现;chat completion ping 浪费 token + Ollama 触发 cold start ③ 不引 async-trait crate(KISS,deps 减 1 个 transitive 树)
- **影响**:Cargo.toml(+reqwest +eventsource-stream +futures +async-stream)/ services/{secrets.rs,llm.rs,mod.rs}(new + reg)/ commands/{llm.rs,mod.rs}(new + reg)/ lib.rs(invoke_handler +5)/ src/ipc/dev.ts(COMMAND_REGISTRY 12→17 + TRACKED_EVENTS +2)/ progress/code-review-2026-05-04.md(B.1 staged audit)
- **关键学习**:① **架构 §6.1 trait 字面 vs 实施 KISS struct 是有意识的偏差**(plan / decisions-log / audit 报告三处记录),不是疏忽;doc-aligner 后续同步是 P1 而不是 P0,实施层稳定后再调 ② **6 preset 中 DeepSeek base_url 与其他 5 个不一致**(ADR-005 字面缺 /v1)— 优雅处置:实施层 normalize 兜底 + preset 表显式加 v1;不必为这点偏差升 ADR ③ **8 维度 audit 触发 2 处自我修复**:`Client::builder().build().unwrap_or_else(Client::new())` 是无效 fallback(两路径同样失败,直接 `Client::new()` 等价);`let _ = llm::PRESETS;` 抑制 hack 是 release build 编译器看不到的死代码,改成显式 type imports
- **defer 项**:① zeroize api_key drop → M3(defense-in-depth,DPAPI 模型已假设进程内存可信)② AppError::User vs Internal 二分 → M2(跨多 service 重构)③ SecurityGuard 注入 → B.2(ChatService 职责)
- **Ref**:`(本笔 B.1 commit 待生成)` + plan 文件 `~/.claude/plans/dapper-beaming-quokka.md` + `progress/code-review-2026-05-04.md`

---

## 模板(新增条目时复制)

```markdown
### YYYY-MM-DD | <scope>

- **决策**:<one line>
- **理由**:<why this over alternatives>
- **影响**:<scope of change>
- **Ref**:<commit SHA / PR# / file paths>
```

---

## 升级到 ADR 的判断

记到本日志的小决策,若后续发现影响扩大(影响多个模块 / 涉及人格 / 影响发布),应考虑升级:

1. 用 `adr-author` 角色起草 ADR-NNN
2. 本日志条目改为 "升级为 ADR-NNN(<title>)"
3. ADR 中【背景】引用本日志条目作为初次提出

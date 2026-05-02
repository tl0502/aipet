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

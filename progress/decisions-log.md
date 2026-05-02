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

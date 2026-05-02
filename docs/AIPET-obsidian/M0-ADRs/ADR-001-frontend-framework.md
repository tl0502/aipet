# ADR-001: 前端框架选型

- **状态**: Accepted
- **决策日期**: 2026-05-01
- **Owner**: M0 决策周(产品 + 工程)
- **Reviewers**: M0 决策周复审通过
- **目标完成**: M0 决策周内(W0)
- **影响范围**: 全部 UI 模块(A/B/H/工坊/装扮/游戏舱)

## 背景

PRD v0.6 锁定 Tauri 2.x 作为桌面容器,但前端框架未定。前端工作量包括:
- PetCanvas 桌宠渲染层(VRM 3D 集成,Three.js)
- 对话面板(流式渲染)
- 设置 / 工坊 / 装扮工坊 / 游戏舱 4 个独立窗口
- 心情图标、气泡、各类轻量交互组件

需要选定主框架,并支持团队的工程效率。

## 备选方案

### 选项 A: Vue 3 + TypeScript + Pinia + Vite

**优点**
- SFC 单文件组件结构清晰,适合多窗口的项目划分
- 模板语法对设计师友好(若有美术兼前端的协作)
- Vue 3 Composition API 接近 React Hooks 心智
- Pinia 状态管理轻量,适配 Tauri IPC 事件订阅模型
- 中文社区资源丰富(对国内招人友好)

**缺点**
- TypeScript 类型推导比 React 略弱
- 大型组件库(Element Plus / Naive UI)对桌面 native 风格适配略弱

### 选项 B: React 18 + TypeScript + Zustand + Vite

**优点**
- TypeScript 与 React 生态最成熟
- 与 Tauri 官方示例 / 社区模板高度对齐
- 组件库选择面广(Radix / shadcn / Mantine)
- Zustand 与 IPC 事件融合直观

**缺点**
- 状态管理选型多,团队不齐心可能内耗
- JSX 对设计师友好度低于 Vue 模板

### 选项 C: Solid.js / Svelte

**优点**
- 编译时优化,运行时体积小,首屏更快
- 对 Tauri 这种"内存敏感"的桌面应用友好

**缺点**
- 生态深度不及 Vue/React,Three.js / 流式 markdown 等中间件需要自己 bridge
- 团队学习成本高,招聘难

## 我的倾向

**🌟 倾向选项 A(Vue 3)**,关键理由:
1. 桌面应用 UI 偏"组件化 + 表单 + 弹窗"风格,Vue SFC + 模板天然契合。
2. Three.js / @pixiv/three-vrm 在 Vue 项目里的集成示例多(原 Live2D Cubism Web SDK 集成生态也丰富,M0 末改为 VRM 后该理由仍然成立)。
3. 国内招人 Vue 占比更高(若团队是中国本地化方向)。

但若团队**已有 React 资历**,选 B 也完全可行——团队熟悉度比框架本身更重要。

不推荐选项 C(生态成本对 MVP 不划算)。

## 决策

**选定:Vue 3 + TypeScript + Pinia + Vite**(选项 A)。

关键理由:Tauri WebView 桌面应用偏"组件化 + 表单 + 弹窗"模式,Vue 3 SFC + 模板语法天然契合;Three.js + @pixiv/three-vrm 在 Vue 项目里的集成示例丰富(原 Live2D Cubism Web SDK 同样契合,M0 末渲染选型从 Live2D 切到 VRM 不影响本 ADR 决策);Pinia 与 Tauri IPC `listen` 事件订阅模型耦合点少,状态管理负担最小;中文社区生态对国内招人友好。

**组件库**推迟到 M1 第一天 spike 后定:候选 **Naive UI**(更现代、暗色主题完整)与 **Element Plus**(成熟、国内用例多)。spike 维度:桌面 native 风格契合度、暗色支持、组件覆盖度(尤其是浮窗/气泡/Modal)。

## 后果

### 正面
- 多窗口(主面板/工坊/装扮工坊/游戏舱)可按 SFC 文件组织,边界清晰。
- Vue 模板语法对设计师友好,UI 调整周转快。
- Pinia store 直接订阅 IPC 事件,状态同步范例简洁(见架构 v1.0 §0.1)。

### 负面
- Vue 模板内的 TypeScript 类型推导比 React JSX 弱,编辑器(尤其 v-model 和复杂 props)体验略受影响。
- 桌面 native 风格的组件库(Naive UI / Element Plus)对真正"系统级"质感的支持仍弱于原生方案,需要 M1 spike 后定方向。

## 实施动作

- [x] 在 `package.json` / `Cargo.toml` 锁定版本
- [x] 输出"项目脚手架"模板(由 M1 第一天直接拉取使用)
- [ ] 输出代码风格指南(命名 / 文件组织 / 组件粒度)
- [x] 选定状态管理库并写"IPC 事件 ↔ store 同步"范例
- [x] CI 配置(lint / typecheck / build)

## 组件库 spike 决策摘要(M1 D1, 2026-05-08)

**决策:选定 Naive UI**(实际安装推迟到 D2 引入第一个组件时,以保持 D1 脚手架最小化)。

**对比维度**:

| 维度 | Naive UI | Element Plus |
|---|---|---|
| Vue 3 原生 | ✅ 从设计起就是 Vue 3 | △ 从 Element UI(Vue 2)迁移 |
| TypeScript | ✅ 类型推导完整 | ✅ 类型完善 |
| 暗色主题 | ✅ 完整内置 | △ 需配置 CSS Variables |
| 体积(tree-shake 后) | ~100KB | ~200KB+ |
| 浮窗/气泡/Modal 设计 | ✅ 现代,适合桌面 | ✅ 成熟,偏 Web 后台风格 |
| 国内文档/案例 | △ 中等 | ✅ 丰富 |

**关键理由**:
1. Vue 3 原生设计,与 Pinia + Composition API 心智一致。
2. 暗色主题完整,符合架构 §0 速查的"暗色主题完整"偏好。
3. 体积更小,符合 PRD §10.5 总安装包 ≤ 80MB 预算。
4. Naive UI 的 Modal / Popover / NSpace 等组件在桌宠"小窗对话气泡 / 工坊弹窗"场景下设计更轻盈。

**风险**:Naive UI 国内文档案例少于 Element Plus,M2 工坊 GUI 复杂度上来后若团队感觉效率受损,可在 ADR-015+ 重新决策切换(组件层面切换成本不高,因为业务逻辑在 store + IPC 层)。

**M1 D2 待办**:在第一个引入组件的视图(预计是 ChatPanel)前 `pnpm add naive-ui`,在 `src/main.ts` 中按需引入。

## 引用

- PRD v0.6 §2.5(Tauri 2.x)
- 架构 v0.3 §1.1(WebView 前端层)
- 架构 v0.3 §13.1(决策项 #1)

## 复审签字

| Reviewer | 角色 | 同意? | 备注 |
|---|---|---|---|
| M0 决策周 | 产品 | ☑ | 与 PRD §2.M0 决策 21 一致 |
| M0 决策周 | 工程 | ☑ | M1 第一天做组件库 spike,选 Naive UI 或 Element Plus |
| M1 D1 实施 | 工程 | ☑ | 2026-05-08 spike 决策落定 Naive UI;实际引入推迟到 D2 |

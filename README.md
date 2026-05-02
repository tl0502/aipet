# AI 桌宠(AIPET)

> 一个由你亲手塑造、会主动关心你、能和你一起玩的 AI 桌宠。
> Tauri 2 + Vue 3 + TypeScript + Three.js + @pixiv/three-vrm。仅 Windows 10/11。

## 三引擎差异化

1. **用户自主人格** —— `.soul.md` Markdown 完全归属用户(参考 OpenClaw)
2. **主动陪伴** —— 桌宠"在那里"被感知(基于本地空闲信号,不读屏幕内容)
3. **共同活动** —— 物理交互、装扮、声音表情、本地 + LLM 小游戏

---

## 快速开始

### 前置要求

- Windows 10 / 11
- Node.js ≥ 20 + pnpm ≥ 9
- Rust(stable channel,`rustup` 安装)
- WebView2 Runtime(Win11 默认有,Win10 需安装)

### 启动开发

```bash
git clone <repo>
cd 桌宠
pnpm install
pnpm tauri:dev
```

### 放置桌宠模型(必须)

下载一个 `.vrm` 模型(推荐 [VRoid Hub](https://hub.vroid.com/) 免费下载,或用 [VRoid Studio](https://vroid.com/en/studio) 自创建),放到:

```
public/avatar/avatar.vrm
```

`pnpm tauri:dev` 启动后会自动加载该路径。若文件不存在,桌宠窗口会显示加载失败提示。

---

## 文档导航

### 实施期权威源(必读)

- **入口**:[docs/AIPET-obsidian/BASELINE.md](docs/AIPET-obsidian/BASELINE.md) — 5 份 v1.0 基线 + 14 ADR + 路线图
- **当前进度**:[progress/CURRENT.md](progress/CURRENT.md) — 现在做什么 / 下一步 / blockers
- **Claude Code 指引**:[CLAUDE.md](CLAUDE.md) — agent 启动协议 + 守则 + 工作流

### 开发命令速查

```bash
pnpm dev              # Vite dev(port 1430,纯前端)
pnpm tauri:dev        # Tauri 全栈开发(推荐)
pnpm tauri:build      # 打包(MVP 期 bundle.active=false,不输出安装包)
pnpm typecheck        # vue-tsc --noEmit
pnpm lint             # eslint --max-warnings=0
pnpm format           # prettier --write
pnpm test             # vitest(实施期补)
cargo check           # (cd src-tauri) Rust 类型检查
cargo test            # (cd src-tauri) Rust 单测
cargo clippy          # (cd src-tauri) lint
```

### 项目结构

```
.
├── CLAUDE.md                # Claude Code 项目指引(agent 必读)
├── progress/                # 实施期进度(单一可信状态)
│   ├── CURRENT.md           # 现在做什么
│   ├── m1.md ~ m5.md        # 各 milestone 详细
│   ├── risks.md             # 13 项风险登记
│   └── decisions-log.md     # 临时决策流水账
├── docs/AIPET-obsidian/     # 5 份 v1.0 基线 + 14 ADR(权威)
├── src/                     # 前端(Vue 3 + TS)
│   ├── components/          # PetCanvas / ChatPanel / ...
│   ├── composables/         # useVRMModel / ...
│   ├── services/            # vrm.ts / ...
│   ├── stores/              # Pinia stores
│   └── ipc/                 # Tauri invoke 封装
├── src-tauri/               # 后端(Rust + Tauri 2)
│   └── src/
│       ├── commands/        # IPC commands(ping / window / ...)
│       ├── services/        # cursor_tracker / window_snap / ...
│       └── state.rs         # AppState
├── public/avatar/           # .vrm 模型放这里(被 .gitignore)
└── .claude/                 # Claude Code 配置(settings + agents/)
```

---

## 性能预算

| 项 | 预算 |
|---|---|
| 总常驻内存 | ≤ 250MB |
| 总安装包 | ≤ 80MB |
| 冷启动 | ≤ 5s |
| 对话首 token | p50 ≤ 1.5s |
| 物理交互响应 | < 100ms |
| 装扮切换 | < 500ms |

详 [BASELINE.md § 性能预算速查](docs/AIPET-obsidian/BASELINE.md)。

---

## 关键约束

1. **Local-first** —— 不引入用户数据强制上传
2. **用户自主权** —— 不削弱用户对 .soul.md / 装扮 / 设置 的控制
3. **非养成原则** —— 不引入流失 / 死亡 / 必须签到
4. **隐私边界** —— 不读应用名 / 窗口标题 / 输入内容 / 麦克风
5. **安全护栏不可绕过** —— 任何人格 / 游戏不能覆盖系统安全前缀

---

## 路线图

| Milestone | 周次 | 主要交付 |
|---|---|---|
| **M0** | W0 | 14 项 ADR Accepted,3 个内置人格定稿,VRM 渲染 spike(✅ 已完成) |
| **M1** | W1-W2 | 桌宠壳层 + 对话 + Onboarding(进行中) |
| **M2** | W3-W4 | 任务三件套 + 物理交互 |
| **M3** | W5-W6 | 记忆 + 主动陪伴 + 文件拖入 |
| **M4** | W7-W8 | 装扮 + 声音表情 + 用户纪念日 |
| **M5** | W9-W10 | 小游戏 + 灰度内测 → RC |

详 [路线图 v1.0](docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md)。

---

## License

待定。本项目当前为非公开开发期。

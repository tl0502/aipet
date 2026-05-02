# AI 桌宠 — Claude Code 项目指引

> 这份文件**只指路,不复制内容**。所有权威叙述在 `docs/AIPET-obsidian/BASELINE.md`。

## 启动协议(任何 agent 入会必读,30 秒)

1. 读 [docs/AIPET-obsidian/BASELINE.md](docs/AIPET-obsidian/BASELINE.md) — 5 份 v1.0 基线 + 14 ADR + 路线图入口
2. 读 [progress/CURRENT.md](progress/CURRENT.md) — 当前 milestone / sprint / in-progress modules / blockers / next 3 tasks
3. 若被指派具体角色,读 `.claude/agents/<role>.md`(module-implementer / adr-author / doc-aligner / gate-checker)

## 守则(实施期不可绕过)

- ❌ **不读** `docs/AIPET-obsidian/_archive/`(v0.1-v0.7 历史归档,实施期参考即误)
- ❌ **不修改** 已 Accepted 的 ADR-001~014;新决策走 `M0-ADRs/ADR-015+`
- ❌ **不放宽** 5 项关键约束:
  1. Local-first(不引入用户数据强制上传)
  2. 用户自主权(不削弱用户对 .soul.md / 装扮 / 设置 的控制)
  3. 非养成原则(不引入流失 / 死亡 / 必须签到)
  4. 隐私边界(不读应用名 / 窗口标题 / 输入内容 / 麦克风)
  5. 安全护栏不可绕过(任何人格 / 游戏不能覆盖系统安全前缀)
- ✅ **当前选型** VRM 3D(Three.js + @pixiv/three-vrm)— Live2D 路线已 Superseded(详 ADR-002/003 顶部)

## 提交规范

- Conventional Commits: `<type>(<scope>): <subject>` — 例 `feat(persona): add .soul.md import`
- type:`feat / fix / refactor / docs / test / perf / chore`
- 每笔 commit 关联 ADR 号(决策类)或 PRD 模块号(实施类)
- 完成 1 个 story 后**必更** `progress/CURRENT.md`
- 不要积累 100+ 行的"大笔 commit"——一个 task 一笔

## 工作流

- 主分支 `main` 受保护,只接 PR
- 当前 milestone:`milestone/m{N}` ← 从 main 拉
- 模块开发:`feat/m{N}-d{day}-<topic>` ← 从 milestone/m{N} 拉
- CI 通过(lint + typecheck + cargo check + test + PII scan) + 1 reviewer 可合并
- milestone 出口:gate-checker 角色生成 `progress/gate-m{N}.md`,出口达成则合 main 打 tag `v0.M{N}.0`

## 多 agent 协作协议(单人 × 串行 session,文件驱动)

| 阶段 | 动作 |
|---|---|
| **启动** | 读启动协议 3 件 + (可选)角色定义 |
| **完成 1 个 task** | 更 `progress/CURRENT.md` + atomic commit |
| **完成 1 个 story** | 额外标 `progress/m{N}.md` 行状态 ✅ |
| **完成 1 个 module** | 额外在 `progress/decisions-log.md` 写 1 行变更摘要 |
| **遇到阻塞** | 写到 `progress/CURRENT.md § Blockers`,标 owner = 自己,期望解锁条件 |
| **milestone 出口** | 调 `gate-checker` 生成 `progress/gate-m{N}.md`,出口达成则合 main + tag |

## 性能预算速查

详 [BASELINE.md § 性能预算速查](docs/AIPET-obsidian/BASELINE.md):

| 项 | 预算 |
|---|---|
| 总常驻内存 | ≤ 250MB |
| 总安装包 | ≤ 80MB |
| 冷启动 | ≤ 5s |
| 对话首 token | p50 ≤ 1.5s |
| 物理交互响应 | < 100ms |
| 装扮切换 | < 500ms |

## 开发命令速查

```bash
pnpm install              # 首次或更新依赖
pnpm dev                  # Vite dev(port 1430)
pnpm tauri:dev            # Tauri 全栈开发
pnpm tauri:build          # 打包(MVP 期 bundle.active=false)
pnpm typecheck            # vue-tsc --noEmit
pnpm lint                 # eslint --max-warnings=0
pnpm format               # prettier --write
cargo check               # (cd src-tauri) Rust 类型检查
cargo test                # (cd src-tauri) Rust 单测
```

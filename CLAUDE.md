# AI 桌宠 — Claude Code 项目指引

> 这份文件**只指路,不复制内容**。所有权威叙述在 `docs/AIPET-obsidian/BASELINE.md`。

## 启动协议(任何 agent 入会必读,30 秒)

1. 读 [docs/AIPET-obsidian/BASELINE.md](docs/AIPET-obsidian/BASELINE.md) — 5 份 v1.0 基线 + 14 ADR + 路线图入口
2. 读 [progress/CURRENT.md](progress/CURRENT.md) — 当前 milestone / sprint / in-progress modules / blockers / next 3 tasks
3. 按任务场景对应 `.claude/agents/<role>.md`(SOP 参考,详 § Agent 决策矩阵)

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
- 完成 1 个 task 后**必更** `progress/CURRENT.md` + atomic commit + push 当前 `feat/*` 分支到 `origin`
- 完成 1 个 story 后额外更新 `progress/m{N}.md` 对应行状态
- 不要积累 100+ 行的"大笔 commit"——一个 task 一笔
- 推荐用 `/ship-task` 收口任务,避免本地 commit 未同步 GitHub

## 工作流

- 主分支 `main` 受保护,只接 PR
- 当前 milestone:`milestone/m{N}` ← 从 main 拉
- 模块开发:`feat/m{N}-d{day}-<topic>` ← 从 milestone/m{N} 拉
- 收口与 push 流程:详 § 提交规范 + `/ship-task`(远程进度首先看 feature branch,不是 main)
- CI 通过(lint + typecheck + cargo check + test + PII scan) + 1 reviewer 可合并
- milestone 出口:打 `/milestone-gate m{N}` 总入口串联 5 SOP(perf-check / deps-audit / release-check / a11y-check / code-audit $branch)+ gate-checker 综合判断 → `progress/gate-m{N}.md`,出口达成则合 main 打 tag `v0.M{N}.0`

## 多 agent 协作协议(单人 × 串行 session,文件驱动)

| 阶段 | 动作 |
|---|---|
| **启动** | 读启动协议 3 件 + (可选)角色定义 |
| **完成 1 个 task** | 详 § 提交规范(`/ship-task` 收口) |
| **完成 1 个 story** | 额外标 `progress/m{N}.md` 行状态 ✅ |
| **完成 1 个 module** | 额外在 `progress/decisions-log.md` 写 1 行变更摘要 |
| **遇到阻塞** | 写到 `progress/CURRENT.md § Blockers`,标 owner = 自己,期望解锁条件 |
| **milestone 出口** | 调 `gate-checker` 生成 `progress/gate-m{N}.md`,出口达成则合 main + tag |

## Agent 决策矩阵

> ⚠️ **当前实际模式**(2026-05-03 起):第三方 API 网关(`Calcium-Ion/new-api`)在所有 subagent 路径上 nil pointer panic,**4 个 subagent 全部不可用**(Explore / general-purpose / adr-author / doc-aligner / gate-checker / module-implementer 测试均 500 panic)。**所有任务由 main session 直接执行**,SOP 见下方「当前模式:main 直接做的 4 类场景」。网关修好撤本警示,恢复矩阵默认 spawn 协作。

### 任务 → agent(目标模式 — 网关修复后)

| 任务类型 | 推荐 agent | 触发短语示例 |
|---|---|---|
| 实施 PRD §6 模块或 story | `module-implementer` | "实现 H 人格" / "继续 B.3.a" / "实施 A.6" |
| 起草新 ADR(实施期发现新决策) | `adr-author` | "起草 ADR-NNN" / "需要新决策" |
| 文档同步(6 份基线 + 路线图) | `doc-aligner` | "PRD §X 不准" / "升 v1.1" / "与现实偏差" |
| Milestone 出口检查(M1-M5 末) | `gate-checker` | "M{N} 出口检查" / "出口判断" |
| 漏洞扫描(commit / branch / milestone) | `code-reviewer`(via `/code-audit`) | "扫漏洞" / "code review" / "milestone 漏洞检查" |
| 可观测性巡检(M1 D6+ logger 后) | `obs-checker` | "obs 检查" / "巡检日志" / "logger 覆盖" |

> **subagent 选用判断**(网关修复后用):任务 ≤ 0.5 day 且 main 已勘察过相关上下文 → 直接做;模板化产出 / 跨 module 重构 / plan 审批 → 走 subagent。

### 当前模式:main 直接做的 6 类场景

> 把对应 `.claude/agents/<role>.md` / `.claude/commands/<name>.md` 当 SOP 参考,**不 spawn**。完成 task 后 `/ship-task` 收口(详 § 提交规范)。

**1. 实施任务**(模块号 / story ID,如「实施 I.2」「继续 B.3.a」)
→ 读 PRD §6.<模块> + 架构对应 § + 涉及 ADR → 拆 stories/tasks → 实施 + 测试 + cargo check → `/ship-task`
→ SOP:[.claude/agents/module-implementer.md](.claude/agents/module-implementer.md)

**2. 决策起草**(发现需新决策,如「起草 ADR-NNN」)
→ 读 `M0-ADRs/README.md` + 同类已 Accepted ADR(如 ADR-015)做模板 → 起草 **Proposed** → **用户签字才改 Accepted**(不可越级)→ 同步 BASELINE.md ADR 表
→ SOP:[.claude/agents/adr-author.md](.claude/agents/adr-author.md)

**3. 文档同步**(文档与现实偏差,如「PRD §X 不准」「升 v1.1」)
→ **6 份必扫**(PRD / 架构 / 人格 / flows / UAT / **roadmap**)→ 小修 in-place / 章节级升 v1.x / 重大走 ADR → 同步 BASELINE.md 版本号
→ 易漏:**roadmap**(详 doc-aligner.md 漏升经验)
→ SOP:[.claude/agents/doc-aligner.md](.claude/agents/doc-aligner.md)

**4. 出口检查**(milestone 末,如「M{N} 出口检查」)
→ **几乎只读**:对照路线图 §7.<M> 出口清单 → 跑 progress / git log / CI 历史 / 性能预算 → 写 `progress/gate-m{N}.md`(达成 / 未达成 + 修复任务清单)→ **不修代码 / 不改文档**(偏差留给场景 3)
→ SOP:[.claude/agents/gate-checker.md](.claude/agents/gate-checker.md)

**5. 机械检查 / 巡检**(SOP-based,几乎只读 + 写 `progress/{type}-{date}.md`)
→ commit 前漏洞扫描:`/code-audit $staged`(8 维度,5-30s)— SOP `code-reviewer.md`
→ 模块完成 / milestone 末性能预算实测:`/perf-check --baseline`(BASELINE 9 项)
→ 模块引入新依赖 / milestone 末:`/deps-audit --all`(Rust cargo-deny + Node pnpm audit + license)
→ milestone 末发布健康:`/release-check --full`(bundle / smoke / WebView2)
→ Vue 组件改动 / milestone 末:`/a11y-check --report`(键盘可达 / ARIA / 文案 / 快捷键 / 颜色 / 可控关闭)
→ M1 D6+ logger 接入后:`obs-checker` 巡检 logger 覆盖 / PII / 错误吞没 / crash 收集
→ 触发节奏:**L1 PostToolUse hook**(`suggest-checks.cjs` 单文件视角自动建议)+ **L2 `/ship-task` commit 完整性视角 7 类智能建议**(自动)+ **L3 milestone-gate 跨期视角**(场景 6)

**6. milestone 末出口仪式**(`/milestone-gate m{N}` 总入口)
→ 7 步串联:perf-check → deps-audit → release-check → a11y-check → code-audit $branch → gate-checker 综合 → 终端汇总
→ 5 个 `--skip-*` flag + `--gate-only` 模式(已跑过补 gate 报告)
→ 产出:`progress/{perf,deps-audit,release-check,a11y,code-review,gate-m{N}}.md` 全套
→ SOP:[.claude/commands/milestone-gate.md](.claude/commands/milestone-gate.md)

> 三层智能触发设计哲学详见 `progress/audit-coverage-2026-05-04.md` § 3 + decisions-log.md 2026-05-04「audit-coverage P0 落地」条目。

## Agent IO 契约(目标协作模式 — 网关修复后)

> ⚠️ 当前阶段不走本节 chain pattern(详 § Agent 决策矩阵 § 当前模式)。本节描述网关修好后的 spawn / 接力规则,作目标模式留存。

Claude Code subagent **不能 spawn 其他 subagent** — 任何跨角色协作都从 main conversation 接力。

**典型 chain pattern**:

1. main → `module-implementer` 实施 → 完成 1 task → `/ship-task` 收口 → 返回 main
2. 实施期发现新决策需求:main → `adr-author` 起 Proposed → 用户签字 → Accepted → main 接力 → `module-implementer` 继续
3. 实施期发现文档偏差:main → `doc-aligner` 扫 6 份并同步 → main 接力 → `module-implementer` 继续
4. Milestone 末:main → `gate-checker` 生成 `progress/gate-m{N}.md` → 出口达成 → 用户合 main + tag → 启动下一 milestone

共享输入契约:进入 session 必读启动协议 3 件 + 角色定义文件。
共享输出契约:完成 task 后 `/ship-task` 收口(`gate-checker` 例外,写 `progress/gate-m{N}.md`)。

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

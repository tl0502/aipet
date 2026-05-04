---
description: Milestone 出口总入口 — 按顺序串 5 份 SOP(perf / deps / release / a11y / code-audit)+ gate-checker 综合判断,产 progress/gate-m{N}.md
allowed-tools: Read, Grep, Glob, Bash(git rev-parse:*), Bash(git log:*), Bash(git branch:*), Bash(git status:*), Bash(git diff:*), Bash(ls:*), Write
argument-hint: [m{N}] [--skip-perf | --skip-deps | --skip-release | --skip-a11y | --skip-audit] [--gate-only]
---

## 任务

milestone 出口仪式**总入口**。按顺序跑 5 份 SOP 检查,然后照 `gate-checker` SOP 综合判断,产出 `progress/gate-m{N}.md`。

**用户视角**:milestone 末 D10-D11 打 `/milestone-gate m1`,~10-15 分钟后拿到完整出口报告。

**Claude 视角**:**这不是一个自含命令** — 它是「编排清单」。命令展开后,main session 应当:
1. 按本文档 Step 1-6 顺序逐个 Read 对应子 SOP 文件,**照子 SOP 步骤跑**(Read / Bash / Write 组合)
2. 每个子 SOP 完成后产独立报告(perf / deps-audit / release-check / a11y / code-review)
3. Step 6 综合所有子报告 + 路线图 § 7.1 出口清单 → 产 `progress/gate-m{N}.md`
4. Step 7 终端汇总

⚠️ slash command 内**不能**程序化触发其他 slash command(SlashCommand 不在 Claude 工具集);本命令是「main 直接做」模式下的总入口,所以 Claude 必须自己实现 5 个 SOP 的步骤,而非"打 5 个 /命令"。

---

## Step 0:自取前置上下文

按顺序执行,把输出留在上下文供后续使用:

1. `git branch --show-current` → 当前分支
2. `git rev-parse --short HEAD` → 当前 HEAD
3. `git log --oneline -10` → 最近 10 笔
4. `git status --short` → 工作区状态(应当为空,milestone 出口前必须 clean)
5. `ls progress/` → 看现有报告

再读以下文件:

- `progress/CURRENT.md` — 当前 milestone / 完成度 / blockers / Recently Completed
- `progress/m{N}.md` — 本 milestone stories 实际状态
- `progress/risks.md` — 本 milestone 风险监控
- `docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md` § 5.X(本 milestone 主交付物)+ § 7.1(出口判断 SOP)
- `docs/AIPET-obsidian/需求设计/2026-05-01-ai-desktop-pet-telemetry-uat-v1.0.md` § KPI 11.x

---

## 范围解析

| `$ARGUMENTS` 第一 token | 解析 |
|---|---|
| 空 | 从分支名 `feat/m{N}-...` 提取 N;若无法提取 → 从 CURRENT.md「Milestone」行提取;两者均失败 → 终止并提示用户显式指定 |
| `m1` / `m2` / ... | 显式 milestone |

flag(可与上面任一组合,可叠加):

- `--skip-perf` — 跳过 Step 1 perf-check
- `--skip-deps` — 跳过 Step 2 deps-audit
- `--skip-release` — 跳过 Step 3 release-check
- `--skip-a11y` — 跳过 Step 4 a11y-check
- `--skip-audit` — 跳过 Step 5 code-audit
- `--gate-only` — 跳过 Step 1-5,直接 Step 6 综合判断(用于「今日已跑过 SOP,补 gate 报告」)

⚠️ 任何 `--skip-*` 必须在 gate 报告 § 范围声明里明说**哪些 SOP 被跳过 + 原因**(否则下一 milestone 复审失忆)。

---

## 执行步骤(按顺序)

### Step 1:perf-check —— 性能预算实测

> 工作目录:项目根。子 SOP:`.claude/commands/perf-check.md`

读 `.claude/commands/perf-check.md`,按其「测量项 → 工具映射」表跑可机械跑的项(M1 末是 1+2+3 内存/bundle/启动 = 3 项;M3+ 起加 4/6;M4+ 加 5/7;M5+ 加 8/9)。

- `--quick`(默认):内存 + bundle + 启动 3 项,~30s
- 同步建立 `progress/perf-baseline.json`(若不存在)或 trend diff(若存在)
- 产出:`progress/perf-{YYYY-MM-DD}.md`(同日多次跑追加 ## Run 节)
- 失败动作:超 budget 项写到 `progress/risks.md`,owner = self

`--skip-perf` 跳过本步,gate 报告 § 范围 写「Step 1 SKIPPED — 原因 X」。

### Step 2:deps-audit —— CVE + license 扫描

> 子 SOP:`.claude/commands/deps-audit.md`

读 `.claude/commands/deps-audit.md`,跑:
- `cargo deny check`(若 `src-tauri/deny.toml` 不存在,先 `--update-config` 生成 starter)
- `pnpm audit --audit-level=high --json`
- `pnpm licenses ls --json`(对照 MIT/Apache-2.0/BSD-2/3/ISC/Unlicense/Zlib/MPL-2.0/CC0-1.0 白名单 + GPL-*/AGPL-*/LGPL-*/SSPL-* 拒绝名单)

产出:`progress/deps-audit-{YYYY-MM-DD}.md` + 终端 5 行汇总。

`--skip-deps` 跳过。

### Step 3:release-check —— 发布健康度

> 子 SOP:`.claude/commands/release-check.md`

读 `.claude/commands/release-check.md`,跑 4 关:

- 关 1 bundle 体积(预算 ≤ 80MB)
- 关 2 bundle 内文件清单(*.pdb / *.exp / 测试资源 / 未使用 .vrm)
- 关 3 smoke 启动(5s 不自杀)
- 关 4 WebView2 bootstrap 配置(检 tauri.conf.json `windows.webviewInstallMode`)
- 关 5 升级路径 stub(M3+ UpdaterService 接入后)

需要先跑 `pnpm tauri build`(~5-10 min)。

产出:`progress/release-check-{YYYY-MM-DD}.md` + 终端 5 行汇总。

`--skip-release` 跳过(不推荐 — 这是 milestone 出口最关键的)。

### Step 4:a11y-check —— UI 巡检

> 子 SOP:`.claude/commands/a11y-check.md`

读 `.claude/commands/a11y-check.md`,扫 `src/components/**/*.vue` + `src/views/**/*.vue`,6 关:

- 关 1 键盘可达(`<div @click>` 无 tabindex / `outline: none` / modal 无 ESC)
- 关 2 ARIA(canvas / 仅图标按钮)
- 关 3 文案硬编码(中文散落 + i18n 准入)
- 关 4 快捷键冲突(对照 Win 系统保留)
- 关 5 颜色对比度(WCAG 2.1 AA;MVP 目测,M5 前 axe-core)
- 关 6 可控关闭(click-outside + ESC)

产出:`progress/a11y-{YYYY-MM-DD}.md` + 终端 5 行汇总。

`--skip-a11y` 跳过(M1 期可暂时跳过 — UI 组件少;M2 ChatPanel 起必跑)。

### Step 5:code-audit $branch —— 漏洞扫描

> 子 SOP:`.claude/commands/code-audit.md`

读 `.claude/commands/code-audit.md`,跑 `$branch` 范围(从 `origin/milestone/m{N}` 到 `HEAD`)8 维度全扫:

1. 注入与执行(SQL / 命令 / 路径遍历 / 反序列化 / unsafe / FFI)
2. 加密与凭证(DPAPI 标志 / 硬编码 / 弱算法 / 随机数)
3. 数据完整性(ON CONFLICT 配 UNIQUE / tx 边界 / migrations idempotent)
4. 输入校验与边界(NaN/Inf / 整数溢出 / 缓冲区 / DoS)
5. 并发与资源(死锁 / data race / TOCTOU / 资源泄漏)
6. 错误处理与 panic
7. 信息泄露(隐私边界 — 关键约束 4)
8. LLM 越狱与 prompt injection

产出:`progress/code-review-{YYYY-MM-DD}.md`(同日多跑追加 ## Run 节)+ 终端 5 行汇总;同步更新 `progress/.audit-state`。

`--skip-audit` 跳过(强烈不推荐)。

### Step 6:gate-checker —— 综合出口判断

> 子 SOP:`.claude/agents/gate-checker.md`

读 `.claude/agents/gate-checker.md`,按其工作流跑(main 直接做模式 — 不 spawn agent):

1. 读路线图 § 7.1 出口判断 SOP,列出本 milestone 必达项 + 可妥协项
2. 对每个必达项从 progress/m{N}.md / git log / CI 历史核实(标 ✅/❌/🚧)
3. 检查 KPI 与杀死指标(若埋点已就绪)
4. 检查关键风险(progress/risks.md)状态
5. **流程文件回归检查**:扫 `.claude/commands/ship-task.md`,与本 milestone 出口产物对照(分支引用 / progress 路径 / 验证套件 / 敏感文件清单 / 出口流程);发现过时项写入 gate 报告「建议下一步」 — **不自己编辑 ship-task.md**,留给 main 实施场景显式升级
6. 综合 Step 1-5 报告 + 路线图必达项 → 产 `progress/gate-m{N}.md`,结构:

```markdown
# M{N} 出口检查报告({date})

## 范围声明
- HEAD:`<short SHA>`
- 分支:`<branch>`
- skip 项:无 / [...]

## 必达项
| 项 | 状态 | 证据 |
| ... | ✅/❌/🚧 | progress/perf-{date}.md `<SHA>` § 实测 vs 预算 / git log /code-review-{date}.md ... |

## 可妥协项
| 项 | 状态 | 备注 |

## KPI / 杀死指标
| 项 | 目标 | 实测 | 状态 |

## 风险监控
| # | 状态 | 备注 |

## 子 SOP 报告链接
- perf:`progress/perf-{date}.md`
- deps:`progress/deps-audit-{date}.md`
- release:`progress/release-check-{date}.md`
- a11y:`progress/a11y-{date}.md`
- code-audit:`progress/code-review-{date}.md`

## 流程文件回归检查
- ship-task.md 是否需切到 m{N+1}?✅ 不需要 / ❌ 需要(建议:...)

## 结论
- 出口达成 / 修复后再检查 / 降级延期

## 建议下一步
- (若达成)合 main 打 tag v0.M{N}.0 + 启动 M{N+1} 入口仪式
- (若未达成)具体修复任务清单(列出 P0/P1/P2 + owner 建议)
- (若 ship-task.md 需升级)列出过时行 + 建议新内容
```

`--gate-only` 模式:跳过 Step 1-5,假设各子报告今日已跑过,直接进 Step 6 综合判断。

### Step 7:终端汇总

输出 7 行内:

1. milestone N + 出口裁决(达成 / 修复后再检查 / 降级延期)
2. perf:✅/❌ N/M 项 + 报告 SHA(或 SKIPPED)
3. deps:✅/❌ vuln + license 数 + 报告 SHA(或 SKIPPED)
4. release:✅/❌ bundle 体积 + smoke 状态 + 报告 SHA(或 SKIPPED)
5. a11y:✅/❌ 违规数 + 报告 SHA(或 SKIPPED)
6. code-audit:Critical/High/Medium/Low 数 + 报告 SHA(或 SKIPPED)
7. gate 报告路径 + 建议下一步(合 main + tag / 修复 N 项 / 降级延期)

---

## 执行规则

- **顺序很关键** — perf 必须 Step 1(release-check 的 build 会污染数据);release-check Step 3 在 perf 之后(release build 已在缓存,加快 smoke 启动)
- **每个子 SOP 独立产报告** — gate-m{N}.md 仅引用 + 综合,不重复内容
- **失败处理** — 子 SOP 失败不中断后续 SOP(record fail + continue);Step 6 综合判断时把失败的 SOP 标 ❌ 入必达项
- **--skip-* 必须有理由** — 在 gate 报告 § 范围声明 / 子 SOP 报告链接 标「SKIPPED — 原因 X」
- **--gate-only 用于「今日已跑过 SOP,补 gate 报告」** — Step 1-5 跳过,引用今日已有的子报告;若今日无对应报告,fail-loudly 提示用户先去掉 --gate-only
- **不修代码** — 命令本身只读 + 写 progress/{各报告}.md
- **不修 ship-task.md** — 流程文件回归检查发现过时项只写建议,留给 main 显式升级

---

## 节奏

- **milestone 末 D10-D11 必跑** — 出口仪式
- 不在常规 commit / 收口路径跑(太重,~10-15 分钟)
- M1 末是首次启用 — 同时建立 5 份 SOP 的 baseline:perf-baseline.json / 首次 a11y / 首次 release-check
- 后续 milestone 都引用前一 baseline 做 trend 比对

---

## 关联文档

- 子 SOP commands:`.claude/commands/{perf-check,deps-audit,release-check,a11y-check,code-audit}.md`
- gate-checker SOP:`.claude/agents/gate-checker.md`
- 路线图 § 7.1 出口判断 SOP:`docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md`
- audit-coverage 三层智能触发设计:`progress/audit-coverage-2026-05-04.md` § 3
- ship-task post-commit 智能建议(L2 视角):`.claude/commands/ship-task.md`「下一步检查建议」段
- PostToolUse hook(L1 视角):`.claude/hooks/suggest-checks.cjs`

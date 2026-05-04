# 项目检查体系覆盖度评估(2026-05-04)

> 本报告:对当前 AI 桌宠项目「除纯代码漏洞之外」的检查体系做一次盘点 + 实用性裁
> 决,并为缺位类别落地 SOP。**只评估治理 / 流程层面**,不评判业务决策与已 Accepted
> 的 ADR-001~014。

---

## §1 背景

用户在 M1 W1 D3 期间问「一个项目的检查除了纯代码漏洞还需要什么」,引出了 10
类检查维度的盘点(C1-C10)。

需求:**对 10 类的现有覆盖做实用性分析,然后为还没设计的类型补设计**。

输出:
1. 本报告(进度记录,版本控制,后续 milestone 出口可对照)
2. 5 份新 SOP(对应 5 类缺位维度,可直接调用)

不在本期范围:
- 修改业务代码 / docs / CLAUDE.md / BASELINE.md
- ADR-016 拍板(若用户认为「检查体系完善」需正式拍板,起草时再走 adr-author)
- CI 集成 yaml(SOP 已含 CI 集成片段,落地为单独 commit)
- E2E 自动化(Playwright + axe-core / 完整 SBOM / i18n 多语种 → 不在 MVP 范围)

---

## §2 当前检查基础设施清单

| 类别 | 工具 / agent | 触发 | 产物 | 落地状态 |
|---|---|---|---|---|
| Agent | code-reviewer | `/code-audit` 或 milestone 末 | `progress/code-review-{date}.md` | ✅(Run 1+2 已跑过) |
| Agent | doc-aligner | 发现文档偏差 | edit 5 份基线 + roadmap | ✅(2026-05-02 升 v1.1 实战过) |
| Agent | gate-checker | milestone 末 | `progress/gate-m{N}.md` | 🟡(SOP 就绪,M1 末首次启用) |
| Agent | adr-author | 新决策 | `M0-ADRs/ADR-NNN.md` | ✅(ADR-015 已 Accepted) |
| Agent | module-implementer | 模块实施 | code + tests + progress 同步 | ✅(M1 持续在用) |
| Command | `/code-audit` | 同 code-reviewer | 同上 | ✅ |
| Command | `/check-baseline` | 任何时间 | 终端 diff | ✅ |
| Command | `/ship-task` | 完成 1 task | atomic commit + push | ✅(2026-05-03 起每笔在用) |
| Command | `/sync-progress` | 任何时间 | CURRENT.md 与 git log 一致性 | ✅ |
| CI | `.github/workflows/ci.yml` | push / PR | lint / typecheck / cargo check / cargo test / **PII grep 3 字符串硬扫**(`messages.content` / `getUserMedia` / `GetForegroundWindow`) | ✅ |
| Hook | husky + commitlint(可能) | pre-commit | Conventional Commit 校验 | ✅ |
| Hook | PreToolUse `.cjs` | 修改 ADR / _archive | 拦截写入 | ✅(2026-05-02 修 ESM 冲突后稳定) |

---

## §3 10 类实用性裁决

### C1 业务 / 产品对齐 — 🟡 中等

- 现状:doc-aligner **反应式**(发现偏差才修);gate-checker 周期(milestone 末对照路线图 §7 出口清单)
- 证据:2026-05-02 升 PRD/架构/flows v1.1(三形态)— 用户先发现「实际实现里有但 PRD 没」才触发;**没有正向「代码实现是否对应 PRD §6.X」连续校验**
- 缺口:依赖人脑判断;模块完成后 module-implementer 在 PR 描述「关联 PRD 模块号」是唯一线索
- 建议:不补新工具(成本 ROI 不划算),靠 module-implementer DoD 「PR 描述含 PRD 模块号」+ gate-checker 周期对账

### C2 架构 / ADR 对齐 — 🟡 中等

- 现状:doc-aligner + PreToolUse hook(防误改 ADR-001~014 与 _archive)
- 证据:hook 防写入有效(2026-05-02 实测 6 项 stdin 用例全部 deny / 放行预期);但「代码符合 ADR-002 VRM 选型」「安全前缀拼装顺序符合 ADR-006」靠人脑
- 缺口:没有正向校验机制;code-reviewer V8 LLM 越狱 检拼装顺序是 byproduct,不是主责
- 建议:不补新工具,靠 ADR Accepted 时「实施动作」段明确「在 module-implementer DoD 加哪些自查项」

### C3 关键约束 / 红线 — ✅ 实用

- 现状:**双层覆盖**
  - 机械层:CI grep `messages.content / getUserMedia / GetForegroundWindow` 必须 0 命中(ci.yml 行 51-61)
  - 语义层:code-audit 维度 7 信息泄露 + 维度 8 LLM 越狱(扫拼装顺序)
- 证据:code-review-2026-05-03.md V7 Run 2 列出 7 条 Low(vrm.ts 用户导入未来反序列化预防 / console.log release / AppError Internal 路径泄露 等),命中语义层;CI 在 PR 阶段拦机械层
- 缺口:**5 红线只盯 2 条**(隐私边界 + 安全护栏);Local-first / 用户自主权 / 非养成 完全靠人脑 + doc-aligner 周期 review
- 建议:接受现状;Local-first 红线最容易踩的是「未来引入云同步」类 ADR,届时 adr-author SOP 强制对照即可

### C4 性能预算 — ❌ 缺位

- 现状:risks.md #9 监控(VRM 内存)+ gate-checker 周期(对照 §7 出口清单数字)
- 证据:**完全无自动化测量**;BASELINE.md § 性能预算速查 9 项数字(内存 250MB / 安装 80MB / 冷启 5s / 物理 100ms / 装扮 500ms / 首 token 1.5s ...)只是目标,从未跑过实测
- 缺口:M1 末 gate-checker 想对账「内存是否 ≤ 250MB」时**无数据可对**
- 建议:**P0 补 `/perf-check` 命令**(本期落地)— 机械跑 PowerShell 测内存 / 启动 / bundle 三项可机械跑的,其余 6 项等 ringbuffer logger(M1 D6+)接入后 grep 日志

### C5 测试覆盖与有效性 — 🟡 中等

- 现状:CI cargo test(31 passed)+ ~~module-implementer DoD「核心 service ≥ 70%」~~ ⚠️ **已于 2026-05-04 治理升级废弃**(commit f6ec85f),改用 [CLAUDE.md § 测试覆盖底线](../CLAUDE.md) 3 层覆盖(纯逻辑单测 + 真实路径集成测试 + dev panel e2e),单层不算 done
- 证据:Vitest 注释着「M2 实施 PersonaService / ChatService 时启用」未启用(ci.yml 行 71-72);覆盖率工具未配;无 E2E
- 缺口:Rust 单测有但**无覆盖率门禁**;前端 Vue 完全无测;Tauri 全栈 E2E 缺位
- 建议:**P2 启用 Vitest 覆盖率门禁**(M2 起,B.2 ChatService 落地必含);E2E 推到 M5 灰度前再决策(可能用 WebDriver);3 层覆盖底线已生效(详 commit `4abceea` 落地 22 集成测试)

### C6 依赖与供应链 — ❌ 缺位

- 现状:code-audit 顺手扫 Cargo.toml / package.json 新增依赖名(只判可疑,不判 CVE)
- 证据:**无 cargo audit / pnpm audit / license 扫描 / SBOM**;decisions-log 2026-05-03 提了一句「依赖 CVE 审计交给 cargo audit / pnpm audit 专业工具」,**但工具尚未集成 CI**
- 缺口:依赖了 13 个 Rust crate + 13 个 Node 包(直接依赖),**任何一个上游 CVE 都会无声无息**
- 建议:~~**P1 补 `/deps-audit` 命令 + CI job**(本期落地 SOP,CI 集成单独 commit)— 用 cargo-deny 单工具替代 cargo-audit;pnpm audit + license 白名单脚本~~ ✅ **已落地于 2026-05-04 同期**(commit `91dca81 → eb764d6 → cefc411 → bed8442` audit-coverage P0 4 笔 atomic),`commands/deps-audit.md` SOP 就绪含 starter `deny.toml` 配置;CI 集成 yaml 仍待单独 commit(M2+)— 详 §5 表第 2 行

### C7 构建与发布健康度 — ❌ 缺位

- 现状:ADR-013 代码签名 M5 灰度推迟 + ship-task 不提交 .vrm 大资源
- 证据:**无 bundle size 检查 / 无 smoke / 无升级路径验证**;ship-task 阶段最重的验证是 `pnpm typecheck + pnpm lint + cargo check`,**从未跑过 release build**
- 缺口:M5 RC 时一定会撞墙(80MB 预算 / WebView2 缺失 / smoke 启动失败)
- 建议:**P1 补 `/release-check` 命令**(本期落地)— 机械跑 `pnpm tauri build` + bundle 体积 + smoke 启动;M1 末 first run 作为基线

### C8 可观测性与运维 — ❌ 缺位

- 现状:code-audit V7(日志 PII 检查)+ DEV-1 Events tab(本地实时事件流)
- 证据:**logger 后端尚未接入**(decisions-log 多处提「ringbuffer logger M1 D6+」);**无 crash 收集 / 无 sentry / 无 telemetry sink**;现存 service 一半用 `eprintln!` 一半用 `tracing::`,无统一规范
- 缺口:线上崩溃就消失;debug 体验差;PII 二次复核(语义层)缺位
- 建议:
  - **P0 补 `/obs-checker` agent**(本期落地)— milestone 末巡检,M1 D6+ logger 接入后第一次跑作基线
  - **P2 接入 sentry-style crash sink**(M5 RC 前)

### C9 UX / a11y / i18n — ❌ 缺位

- 现状:**无任何检查**
- 证据:`pnpm dev` 跑起来后桌宠透明 320×320,**任何 Vue 组件可以**:全 div 无 button / 全 outline:none / 全中文硬编码 / 任意快捷键冲突 — 都不会被任何 lint 拦
- 缺口:M5 灰度反馈一定有 a11y / 文案散落问题
- 建议:
  - **P1 补 `/a11y-check` 命令**(本期落地)— 6 关:键盘可达 / ARIA / 文案硬编码 / 快捷键冲突 / 颜色对比 / 可控关闭
  - **P2 M5 灰度前接 Playwright + axe-core E2E**(超出本期)

### C10 流程与制度 — ✅ 实用

- 现状:**四件套**(husky + commitlint + check-baseline + sync-progress + ship-task)
- 证据:
  - check-baseline:专门防 2026-05-02 roadmap 漏升 v1.1 case 复发
  - sync-progress:CURRENT.md 与 git log 一致性扫
  - ship-task:每笔 task 标准收口(progress + atomic commit + push)
  - PreToolUse hook:防误改 ADR-001~014 与 _archive
- 缺口:无重大缺位;husky+commitlint 实际是否生效未直接验证,但 commit 历史符合 Conventional Commits 规范
- 建议:接受现状;若发现 commit 不规范增多再补

### 总裁决汇总

- ✅ 完整可用 **2/10**:C3 关键约束 / C10 流程制度
- 🟡 中等 **3/10**:C1 业务对齐 / C2 架构对齐 / C5 测试覆盖
- ❌ 缺位 **5/10**:C4 性能预算 / C6 依赖供应链 / C7 构建发布 / C8 可观测性 / C9 UX/a11y/i18n

---

## §4 缺位 5 类的补强路线图

### P0(M1 末前必补,~2-3 天)

| # | 类别 | 动作 | 依赖 |
|---|---|---|---|
| 1 | C8 可观测性 | M1 D6+ ringbuffer logger 接入(`tracing-subscriber + tracing-appender::rolling`),写盘 `%APPDATA%\aipet\logs\` | M1 D6+ 计划已有 |
| 2 | C8 可观测性 | logger 接入后跑首次 `obs-checker` 基线巡检 | (1) |
| 3 | C4 性能预算 | M1 末跑 `/perf-check --baseline`,记录内存 / bundle / 启动 | M1 末或前 |

### P1(M2-M3 期补,~1-2 天分散)

| # | 类别 | 动作 |
|---|---|---|
| 4 | C6 依赖供应链 | `/deps-audit --update-config` 生成 `src-tauri/deny.toml` starter;CI 集成单独 commit 加 `cargo deny check` + `pnpm audit` job |
| 5 | C7 构建发布 | M2 末跑首次 `/release-check --full`,作为 bundle 基线;后续 milestone 末必跑 |
| 6 | C9 UX/a11y/i18n | 在 M2 ChatPanel(B.2)实施前先跑 `/a11y-check src/components/ChatPanel.vue` 作 UI 设计自查 |

### P2(M4-M5 期补,RC 前)

| # | 类别 | 动作 |
|---|---|---|
| 7 | C7 构建发布 | M5 RC 前跑 `/release-check --full` 含 WebView2 干净 VM 测试(关联 risks.md #3) |
| 8 | C8 可观测性 | M5 RC 前接入 sentry-style crash sink(self-host 优先,本地 first 原则) |
| 9 | C5 测试覆盖 | M2 起 Vitest 启用 + 覆盖率门禁(B.2 ChatService 时落地)|

### 不在 MVP 范围

- E2E 自动化(Playwright + axe-core)— 推到 v0.M5+ 或 v1.0
- 完整 SBOM 生成(CycloneDX)— 推到 v1.0 公开发布前
- i18n 多语种(Vue I18n)— P1 路线 R3 生态扩展时

---

## §5 5 份新 SOP 总览

本期落地的 SOP 文件清单:

| # | 类别 | 文件 | 类型 | 行数 |
|---|---|---|---|---|
| 1 | C4 性能预算 | [`.claude/commands/perf-check.md`](../.claude/commands/perf-check.md) | command | ~180 |
| 2 | C6 依赖供应链 | [`.claude/commands/deps-audit.md`](../.claude/commands/deps-audit.md) | command | ~180 |
| 3 | C7 构建发布 | [`.claude/commands/release-check.md`](../.claude/commands/release-check.md) | command | ~150 |
| 4 | C8 可观测性 | [`.claude/agents/obs-checker.md`](../.claude/agents/obs-checker.md) | agent | ~210 |
| 5 | C9 UX/a11y/i18n | [`.claude/commands/a11y-check.md`](../.claude/commands/a11y-check.md) | command | ~170 |
| 6 | **跨期 orchestrator(L3 milestone 视角)** | [`.claude/commands/milestone-gate.md`](../.claude/commands/milestone-gate.md) | command | ~242 |

> 第 6 行 milestone-gate 是 plan §10.2 P1-3 后续追加(2026-05-04 同期落地 `66114b5`),补完三层智能触发 L3 跨期视角:**L1 hook 单文件视角**(suggest-checks.cjs)/ **L2 ship-task commit 完整性视角**(ship-task.md 7 类智能建议)/ **L3 milestone-gate 跨期视角**(7 步 orchestrator 串联 5 SOP + gate-checker 综合)。缺任何一层都漏检。

设计原则(参照报告时点已有 4 个 agent / 4 个 command 的范式;落地后 → 6 agent + 9 command):

- **agent vs command 决策**:agent 用于长上下文巡检(如 obs-checker 需要全仓 grep + 语义层判断);command 用于机械动作(如 perf-check 跑 PowerShell 测内存)
- **几乎只读** + 只写 `progress/`(同 gate-checker / code-reviewer 范式)
- **不修业务代码** / 不改 CI(SOP 给配置 starter,CI 集成留单独 commit)
- **每条 finding 必须给修复路径** + 编号(perf-baseline 趋势 / RUSTSEC-XXXX / CWE 等)
- **节奏明确**:模块完成 / milestone 末 / RC 前 三层触发

---

## §6 下一步动作

### 用户 review

- 读本报告 §3 实用性裁决,确认现状理解
- 读 §4 路线图,决定 P0/P1/P2 优先级是否合理(可调整)
- 读 §5 列的 5 份 SOP,挑出需要细化或合并的项

### 触发各 SOP 落地

- M1 D6 ringbuffer logger 接入后第一次跑 `obs-checker`
- M1 末跑 `/perf-check --baseline` 作基线
- M2 起 `/deps-audit` + `/a11y-check` + `/release-check` 进入常态
- gate-checker 在 `progress/gate-m{N}.md` 中引用各 SOP 最新报告

### 可选:升级到 ADR-016 拍板

若用户认为「检查体系完善」需正式拍板:
- 触发 adr-author 起草 `ADR-016 项目检查体系完善`
- ADR Accepted 后,在 BASELINE.md ADR 表加一行(15 → 16)
- ADR 实施动作清单 = 本报告 §4 路线图

本报告本身**不替代 ADR**;ADR-016 是否需要由用户决定(若全部缺位补强是常规治理工作,可不走 ADR;若涉及架构调整如 sentry self-host vs cloud,则建议走 ADR)。

---

## 关联文档

- [BASELINE.md § 性能预算速查](../docs/AIPET-obsidian/BASELINE.md) — perf-check 唯一权威目标值源
- [路线图 §7 出口判断 SOP](../docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md) — gate-checker 调用本报告各 SOP
- [progress/risks.md](risks.md) — perf-check / release-check 超 budget 时回写
- [progress/code-review-2026-05-03.md](code-review-2026-05-03.md) — obs-checker V7 PII 复核交叉验证
- [progress/CURRENT.md](CURRENT.md) — milestone 进度状态

## 关联 commit

- 落地 commit:本报告 + 5 份 SOP atomic commit(后续 `/ship-task` 收口)
- 关联 PRD 模块:无(治理流程,非业务实施)
- 关联 ADR:无(若用户决定升级到 ADR-016 拍板,届时关联)

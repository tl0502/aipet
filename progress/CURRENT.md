# Current State

> 任何 agent 入会必读此文件(启动协议步骤 2)。完成 1 个 task 后必更新此文件。

- **Milestone**:M1 W1 D3(进行中)
- **Active branch**:`feat/m1-d2-window-interaction`
- **Last commit**:`30cfb45 chore(gitignore): expand to 13 categories + untrack .obsidian/`
- **Tag**:none yet(M1 出口达成后打 `v0.M1.0`)
- **Last updated**:2026-05-03(CLAUDE.md / 4 agent .md / hook 注释整体减冗:消除 5 处规则重复 + 标注 subagent 不可用现状)

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

---

## Blockers

(无)

> 历史 Blocker `缺 public/avatar/avatar.vrm` 已于 2026-05-02 由用户补充模型文件解除。

---

## Next 3 Tasks(优先级降序)

1. **I.2 CryptoService**(0.5 day)— Windows DPAPI 封装,为 B.1 LLMProvider 的 API key 加密铺底;写入 `secrets` 表的 ciphertext
2. **H.1 PersonaService MVP**(1 day)— 加载 `_builtin/momo.soul.md`,解析 frontmatter + Markdown,写入 `personas` 表
3. **F.1 MemoryService MVP**(0.5 day)— `messages` 表 CRUD + 90 天清理 + summary 占位

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

---

## 路线图速查

- M1(W1-W2):壳层 + 对话 + Onboarding
- M2(W3-W4):任务三件套 + 物理交互
- M3(W5-W6):记忆 + 主动陪伴
- M4(W7-W8):装扮 + 声音 + 纪念日
- M5(W9-W10):小游戏 + 灰度 + RC

详见 [docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md](../docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md)。

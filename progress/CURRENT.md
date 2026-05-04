# Current State

> 任何 agent 入会必读此文件(启动协议步骤 2)。完成 1 个 task 后必更新此文件。

- **Milestone**:M1 W1 D3(进行中)
- **Active branch**:`feat/m1-d2-window-interaction`
- **Last commit**:(本笔 B.1)`99066a8 docs(claude): align CLAUDE.md / decisions-log / audit-coverage / ship-task with P0 reality`(本笔 B.1 LLMProvider 落地是其后第 1 笔)
- **Tag**:none yet(M1 出口达成后打 `v0.M1.0`)
- **Last updated**:2026-05-04(B.1 LLMProvider 落地 — 6 preset + DPAPI secrets + OpenAI compat streaming + dev e2e + 8 维度 audit Pass + cargo test 31 → 62)

---

## In-Progress Modules

| Module | Story | Owner | Status | Next |
|---|---|---|---|---|
| (无)| — | — | — | M1 D2 已收口,等待启动 D3 |

---

## Recently Completed(本 milestone,近 5 笔)

> **m1.md § Completed Log 是历史 commit 流水权威源**(28 笔历史归档)。本表只保留近 5 笔便于扫读,milestone 切换时整段随 m1.md 归档到 `milestone/m1` 分支。

| Story | Commit | Date |
|---|---|---|
| **B.1 LLMProvider**(OpenAI 兼容 streaming chat completion + DPAPI 取 key)— `secrets.rs` CRUD + `llm.rs` `OpenAiCompatProvider`(KISS struct 不抽 trait;P1-R1 接 Anthropic 再抽)+ 6 preset(openai/deepseek/moonshot/qwen/ollama/custom)+ SSE 解析纯函数 + base_url normalize + GET /v1/models ping + 5 IPC + dev_llm_test_stream(debug-only e2e)+ Events `dev.llm.token`/`done`;cargo test 31 → 62;8 维度 audit Pass(0 C/0 H/1 修 `Client::new()` fallback + import hack/ M2-M3-B.2 defer 3 项)| (本笔)| 2026-05-04 |
| **用户主动复审 + 4 处治理同步缺口修补**(post-P0 follow-up audit;9 维度交叉审查 1 Critical + 2 High + 1 Low + 1 Medium 遗留;F1+F2+F3+F4 合并一笔纯 docs) | `99066a8` | 2026-05-04 |
| **plan §10.1 P0-2 settings.local.json 漂移清理**(精算 21 条:A 桶 17 + B 桶 4;91 → 69 行;.gitignore L54 排除不进 git,本地 hygiene 直接生效) | (本地)| 2026-05-04 |
| **plan §10.2 P1-3 /milestone-gate 总入口命令**(7 步流程串联 5 SOP + gate-checker 综合;5 个 `--skip-*` + `--gate-only`;补完三层智能触发 L3 闭环) | `66114b5` | 2026-05-04 |
| **audit-coverage P0 落地 4 笔 atomic**(5 SOP commands + obs-checker agent + suggest-checks.cjs PostToolUse hook + ship-task 7 类智能建议;详 `audit-coverage-2026-05-04.md`) | `91dca81` → `eb764d6` → `cefc411` → `bed8442` | 2026-05-04 |

---

## Blockers

(无)

> 历史 Blocker `缺 public/avatar/avatar.vrm` 已于 2026-05-02 由用户补充模型文件解除。

---

## Next 3 Tasks(优先级降序)

1. **B.2 ChatService MVP**(1 day)— 组装 system prompt(persona + memory + nickname + SecurityGuard)+ 调用 LLMProvider chat_stream + emit `chat.token/done/error`(依赖 H.1 / F.1 / F.2 / B.1 全 ✅)
2. **OpenClaw 文件操作 ADR-016 research**(1-2h)— 用户 2026-05-03 提出方向;B.1 落地后做研究 + 起草 Proposed
3. **A.6 智能穿透收口 polish**(0.5 day,D10 收口前)— II bbox 阈值降 IPC + III `tauri://moved` 立即上报修拖动错位

详见 [m1.md](m1.md) 完整拆解。

> **Backlog 已归位**(2026-05-02 B 步骤完成):智能穿透 4 项改进(plan a-5 Part A)已分散到 m1.md A.6(II+III, D10)/ m2.md A.7(I, N 前置)/ m2.md N.0(IV, N 期 backup)。无悬空 backlog。

---

## Recent Decisions(简版索引,详见 decisions-log.md;keep 近 8 条)

> **维护规则**:见 CLAUDE.md § CURRENT.md 维护规则。8 条上限,新决策推入时把最早 1 条挤出(完整版必先 sink 到 `progress/decisions-log.md`);每条 ≤ 200 字符。

- **2026-05-04**:**B.1 LLMProvider 落地** — `secrets.rs` CRUD + `llm.rs` OpenAiCompatProvider(KISS struct 不抽 trait,P1-R1 接 Anthropic 再抽)+ 6 preset + SSE 解析纯函数 + base_url normalize 兜底 DeepSeek 缺 v1 + GET /v1/models ping + dev_llm_test_stream e2e;架构 §6.1 trait 接口微调为 struct 直暴(待 doc-aligner 同步);cargo test 31→62
- **2026-05-04**:vibecoding harness 研究 + plan §10 P1-3/P0-2 — 输出 plan 12 节(token 估算 / hook+slash command+permission 三类深度分析);6 笔 commit 落地(milestone-gate + settings.local.json 漂移清理 21 条);**关键学习**「研究阶段必先 git status 抓 baseline」
- **2026-05-04**:audit-coverage P0 4 笔 atomic — 10 类盘点 C1-C10(✅2/🟡3/❌5),P0 补 5 类缺位 SOPs + obs-checker + suggest-checks.cjs hook + ship-task 7 类建议;**三层智能触发**设计(L1 hook / L2 ship-task / L3 milestone-gate)
- **2026-05-04**:code-audit A 桶 4 hardening 闭环 — M-1 crypto 整数截断(`check_input_size` helper)/ M-3 nickname UPSERT explicit 两步 / M-4 Mutex `lock_or_recover` poison / L-7 vite production esbuild.drop;cargo test 27→31 全过
- **2026-05-03**:`/code-audit` 命令 + Run 1+2 首次扫描 — 8 维度纯漏洞扫描(注入/加密/数据完整性/输入边界/并发/错误处理/信息泄露/LLM 越狱)+ git diff 增量;`$branch` 33/33 = Pass(0 C/0 H/5 M/5 L/3 Won't);命名避开 Anthropic 内置 `/review`
- **2026-05-03**:M1 D2/D3 hardening 5 笔 — start_drag 顺序倒置防 cursor_tracker 卡死 / hitbox NaN 输入加固 / persona seed sqlx::Tx + ON CONFLICT 与 migrations/002 UNIQUE INDEX 强绑定 / Hitbox saturating + 7 单测 / cursor_tracker init regression(42bb4c7 丢失 set_ignore)
- **2026-05-03**:DEV-1 Admin/Debug 面板 — 独立 webview "dev" 窗口 + Ctrl+Shift+D + 4 tabs(IPC Playground / Tables / Events / Logs);`#[cfg(debug_assertions)]` + `import.meta.env.DEV` 双重排除 release;白名单 5 表防 SQL 注入
- **2026-05-03**:F.2 NicknameService facade — `nicknames` 单行表(取代 m1.md 字面 user_state.xxx)+ 5 services + 5 IPC + nickname.changed event;get_pet 三级 fallback;set_pet 自动备份 previous;restore_pet 原子 swap

---

## 路线图速查

- M1(W1-W2):壳层 + 对话 + Onboarding
- M2(W3-W4):任务三件套 + 物理交互
- M3(W5-W6):记忆 + 主动陪伴
- M4(W7-W8):装扮 + 声音 + 纪念日
- M5(W9-W10):小游戏 + 灰度 + RC

详见 [docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md](../docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md)。

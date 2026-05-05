# Current State

> 任何 agent 入会必读此文件(启动协议步骤 2)。完成 1 个 task 后必更新此文件。

- **Milestone**:M1 W1 D3(进行中)
- **Active branch**:`feat/m1-d2-window-interaction`
- **Last commit**:`24199d8 feat(chat): B.2 ChatService MVP — 安全前缀 + persona + nickname + 流式`(本笔)← `39b6279 fix(events): Tauri 2.x event name 全量 . → :`
- **Tag**:none yet(M1 出口达成后打 `v0.M1.0`)
- **Last updated**:2026-05-05(B.2 ChatService MVP 落地 — services/chat.rs + commands/chat.rs 双模块;SAFETY_PREFIX_V1 ADR-006 字面 + compose_system_prompt 占位替换 + run_chat orchestrator + tokio CancellationToken + 4 IPC;架构 §5.1 in-place 加 IPC command 命名注脚不升 v1.1;cargo test 87 → 101 全过)

---

## In-Progress Modules

| Module | Story | Owner | Status | Next |
|---|---|---|---|---|
| (无)| — | — | — | M1 D3 B.2 已收口,启动 D5 B.3.a chat 窗 UI 极简版 |

---

## Recently Completed(近 5 笔;权威流水 m1.md § Completed Log;规则 CLAUDE.md § progress/ 维护规则)

| Story | Commit | Date |
|---|---|---|
| **feat(chat): B.2 ChatService MVP**(services/chat.rs 552 行 + commands/chat.rs 152 行;SAFETY_PREFIX_V1 ADR-006 字面 5 条 + zh-CN 危机资源 + compose_system_prompt({pet_name}/{username} 替换;{username} 回退「朋友」)+ ensure_conversation_with_conn 守 FK + load_active_persona JOIN snapshots 取最新 version + run_chat orchestrator(write user → fetch N=20 history → tokio::select! cancel/stream → emit chat:token → write assistant + emit chat:done)+ AppState chat_cancellations + tokio_util CancellationToken + 4 IPC(chat_send/cancel/history/conversation_create)+ DevPanel +4 commands +3 events;架构 §5.1 in-place 加 IPC command 命名注脚不升 v1.1;cargo test 87→101)| `24199d8` | 2026-05-05 |
| **fix(events): Tauri 2.x event name 全量 `.` → `:`**(用户手测 F.2 panic「event emit failed: only alphanumeric, '-', '/', ':', '_' permitted」;代码层 nickname.rs / llm.rs / dev.ts 真 emit 3 + tracked 5 + persona/network 预防;基线 in-place 修架构 §5.2 28 条 + 顶部加 Tauri 约束注 + flows v1.0 22 处;不升 v1.1;cargo test 87 全过)| `39b6279` | 2026-05-04 |
| **fix(dev): TablesView 前端表白名单二次拦截删除**(D2 第一版 ALLOWED 5 张未随 97673ab 后端放开同步,dev panel 22 张表灰着不可点;KISS 删前端拦截后端 dev.rs:90 单点维护,Err 自带 allow-list 文本;dev-only release 编译期排除无安全风险)| `b843a07` | 2026-05-04 |
| **test(services): DB 集成测试缺位补齐**(B.1 实施暴露 7 services / 62 单测全是纯逻辑,DB 写入路径完整周期没真跑过 → tempfile + 手卷 migrations fresh_db fixture + 22 真实 DB 集成测试 secrets 3 / nickname 6 / memory 8 / persona 5 + 3 fixture self-test + 抽 14 `_with_conn` inner helper(行为等价 refactor for testability);**[HIGH] sqlx 默认 PRAGMA foreign_keys=ON** B.2 ChatService 必须 ensure conversation 存在 — `insert_message_rejects_unknown_conversation_id` 守住此契约;cargo test 62 → 87)| `4abceea` | 2026-05-04 |
| **fix(db): plugin preload + builder API + dev panel 表白名单全开**(B.1 e2e 测试暴露 3 处 dev/DB 基础设施缺口:① tauri.conf.json 加 `plugins.sql.preload` 让 tauri-plugin-sql 2.x migrations 真正跑起来,aipet.db 之前从未被创建 ② commands/dev.rs `open_conn` 改 builder API 避开 Windows 反斜杠 `from_str("sqlite:C:\\...")` 失败 ③ ALLOWED_TABLES 5 → 27 全 schema 白名单)| `97673ab` | 2026-05-04 |

---

## Blockers

(无)

> 历史 Blocker `缺 public/avatar/avatar.vrm` 已于 2026-05-02 由用户补充模型文件解除。

---

## Next 3 Tasks(优先级降序)

1. **B.3.a ChatPanel UI 极简版**(1 day)— 独立 chat 窗(label=chat,默认 380×480)+ 单 conversation + 流式渲染(订阅 chat:token/done/error)+ ESC / 失焦 / Ctrl+Alt+Space toggle;不做磁吸 / 控制按钮区 / 多 conversation / 形态 1/3(ADR-015 § M1 范围)
2. **OpenClaw 文件操作 ADR-016 research**(1-2h)— 用户 2026-05-03 提出方向;B.2 落地后做研究 + 起草 Proposed
3. **A.6 智能穿透收口 polish**(0.5 day,D10 收口前)— II bbox 阈值降 IPC + III `tauri://moved` 立即上报修拖动错位

详见 [m1.md](m1.md) 完整拆解。

> **Backlog 已归位**(2026-05-02 B 步骤完成):智能穿透 4 项改进(plan a-5 Part A)已分散到 m1.md A.6(II+III, D10)/ m2.md A.7(I, N 前置)/ m2.md N.0(IV, N 期 backup)。无悬空 backlog。

---

## Recent Decisions(简版索引,详见 decisions-log.md;keep 近 8 条)

> **维护规则**:见 CLAUDE.md § CURRENT.md 维护规则。8 条上限,新决策推入时把最早 1 条挤出(完整版必先 sink 到 `progress/decisions-log.md`);每条 ≤ 200 字符。

- **2026-05-05**:**B.2 ChatService MVP 落地** — services/chat.rs(SAFETY_PREFIX_V1 ADR-006 字面 5+zh-CN 危机资源 + compose_system_prompt {pet_name}/{username} 替换 + run_chat orchestrator + tokio CancellationToken)+ commands/chat.rs(4 IPC)+ AppState chat_cancellations + 架构 §5.1 in-place 加命名注脚不升 v1.1;cargo test 87→101
- **2026-05-04**:**Tauri 2.x event name 全量 `.` → `:`** — F.2 panic「event emit failed」暴露 Tauri Emitter 校验 event name 字符集不允许 `.`;架构 §711 28 条 + flows 22 处 + 代码 3 真 emit + 5 tracked 全量 in-place 改;不升 v1.1 — 命名规则适配运行时,IPC contract 语义未变
- **2026-05-04**:**DB 集成测试缺位补齐** — tempfile + 手卷 migrations fixture(不引 sqlx::test 宏 / mock_app)+ 22 真实 DB 集成测试(secrets 3 + nickname 6 + memory 8 + persona 5)+ 14 inner helper 抽取;**[HIGH] sqlx 默认 PRAGMA foreign_keys=ON** B.2 必须 ensure conversation;cargo test 62 → 87
- **2026-05-04**:**B.1 LLMProvider 落地** — `secrets.rs` CRUD + `llm.rs` OpenAiCompatProvider(KISS struct 不抽 trait,P1-R1 接 Anthropic 再抽)+ 6 preset + SSE 解析纯函数 + base_url normalize 兜底 DeepSeek 缺 v1 + GET /v1/models ping + dev_llm_test_stream e2e;架构 §6.1 trait 接口微调为 struct 直暴(待 doc-aligner 同步);cargo test 31→62
- **2026-05-04**:vibecoding harness 研究 + plan §10 P1-3/P0-2 — 输出 plan 12 节(token 估算 / hook+slash command+permission 三类深度分析);6 笔 commit 落地(milestone-gate + settings.local.json 漂移清理 21 条);**关键学习**「研究阶段必先 git status 抓 baseline」
- **2026-05-04**:audit-coverage P0 4 笔 atomic — 10 类盘点 C1-C10(✅2/🟡3/❌5),P0 补 5 类缺位 SOPs + obs-checker + suggest-checks.cjs hook + ship-task 7 类建议;**三层智能触发**设计(L1 hook / L2 ship-task / L3 milestone-gate)
- **2026-05-04**:code-audit A 桶 4 hardening 闭环 — M-1 crypto 整数截断(`check_input_size` helper)/ M-3 nickname UPSERT explicit 两步 / M-4 Mutex `lock_or_recover` poison / L-7 vite production esbuild.drop;cargo test 27→31 全过
- **2026-05-03**:`/code-audit` 命令 + Run 1+2 首次扫描 — 8 维度纯漏洞扫描(注入/加密/数据完整性/输入边界/并发/错误处理/信息泄露/LLM 越狱)+ git diff 增量;`$branch` 33/33 = Pass(0 C/0 H/5 M/5 L/3 Won't);命名避开 Anthropic 内置 `/review`

---

## 路线图速查

- M1(W1-W2):壳层 + 对话 + Onboarding
- M2(W3-W4):任务三件套 + 物理交互
- M3(W5-W6):记忆 + 主动陪伴
- M4(W7-W8):装扮 + 声音 + 纪念日
- M5(W9-W10):小游戏 + 灰度 + RC

详见 [docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md](../docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md)。

# Current State

> 任何 agent 入会必读此文件(启动协议步骤 2)。完成 1 个 task 后必更新此文件。

- **Milestone**:M1 W1 D2(收口)→ 即将进入 D3
- **Active branch**:`feat/m1-d2-window-interaction`(已改名,去 live2d 残留)
- **Last commit**:`2f3b28c feat(m1-d2): VRM rendering + transparent window interaction`
- **Tag**:none yet(M1 出口达成后打 `v0.M1.0`)
- **Last updated**:2026-05-02

---

## In-Progress Modules

| Module | Story | Owner | Status | Next |
|---|---|---|---|---|
| (无)| — | — | — | M1 D2 已收口,等待启动 D3 |

---

## Recently Completed(本 milestone)

| Story | Commit | Date |
|---|---|---|
| A.1 透明窗口 + 置顶 + skipTaskbar | `8a37728` (scaffold 时已就绪) | 初始 |
| A.2 VRM 渲染 momo + Three.js + spring bone | `2f3b28c` | 2026-05-02 |
| A.3 hitbox 上报 + 拖动 + 智能穿透 + 边缘吸附 | `2f3b28c` | 2026-05-02 |
| 文档对齐 v1.0 → VRM | `908a6bd` | 2026-05-02 |
| Scaffold tweaks(端口 1430、vrm gitignore、vite path) | `eadad55` | 2026-05-02 |

---

## Blockers

| Blocker | Owner | 期望解锁条件 | 影响 |
|---|---|---|---|
| 缺 `public/avatar/avatar.vrm` 实际模型文件 | (you) | 从 VRoid Hub 下载 momo VRoid 占位,或用 VRoid Studio 自建 | A.2 视觉 verify 暂时只能看到 "VRM 加载失败" 提示;不阻塞代码层进展 |

---

## Next 3 Tasks(优先级降序)

1. **A.4 系统托盘**(0.5 day)— 加 tray-icon plugin,显示/隐藏桌宠 + 退出菜单
2. **A.5 全局快捷键**(0.5 day)— `Ctrl+Alt+Space` 唤起对话 / `Ctrl+Shift+B` 摸鱼
3. **I.1 MigrationService**(1 day)— SQLite schema v1 初始化 + tauri-plugin-sql 集成,为 H/F/B 模块铺底

详见 [m1.md](m1.md) 完整拆解。

---

## Recent Decisions(简版,详见 decisions-log.md)

- 2026-05-02:vibecoding 工程支撑层落地(CLAUDE.md / .claude/agents/ / progress/),协作模式定为单人 × 串行 session 文件驱动

---

## 路线图速查

- M1(W1-W2):壳层 + 对话 + Onboarding
- M2(W3-W4):任务三件套 + 物理交互
- M3(W5-W6):记忆 + 主动陪伴
- M4(W7-W8):装扮 + 声音 + 纪念日
- M5(W9-W10):小游戏 + 灰度 + RC

详见 [docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md](../docs/AIPET-obsidian/2026-05-01-development-roadmap-v1.0.md)。

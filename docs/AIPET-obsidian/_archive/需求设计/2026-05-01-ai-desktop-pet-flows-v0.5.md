# AI桌宠 流程与状态机（v0.5）

- 文档版本：v0.5（基于 v0.4 的**增量**版本，仅写新增/变更部分；未变更章节请参阅 v0.4）
- 创建日期：2026-05-01
- 关联：PRD v0.5、架构 v0.2、人格设计 v0.1

## 0. v0.4 → v0.5 变更摘要

| 章节 | 变更 |
|---|---|
| § 1 Onboarding | Step 1 替换为"灵魂宣誓"（人格化叙述） |
| § 10 状态机 | 新增 IDLE 子态 `WANDERING`；新增并行态 `BOSS_KEY_HIDDEN`（叠加态） |
| § 13（新） | 自由活动子状态机（LivingPet 调度） |
| § 14（新） | 主动关心触发流（IdleDetector + ProactiveCareService） |
| § 15（新） | 摸鱼模式切换流 |
| § 16（新） | 文件拖入流 |
| § 17（新） | 跨日纪念日触发流 |

未列出的章节（§ 2 快捷唤起对话、§ 3 提醒闭环、§ 4 番茄钟、§ 5 网络切换、§ 6 人格切换、§ 7 数据迁移、§ 8 异常恢复、§ 9 自动更新）按 v0.4 原文不变。

## 1. 首次启动 Onboarding（v0.5 修订）

### 1.1 Step 1 替换：灵魂宣誓页

```
启动
 ↓
[检测本地配置]
 ↓
 ├── 存在 → 进入主态
 └── 不存在 ↓
        Step 1：灵魂宣誓（v0.5 替换原"隐私同意"）
        ├── 加载内置默认人格（默默 momo）的形象 + 第一人称文案
        ├── 文案核心：
        │   - "我的记忆只在你电脑里"
        │   - "联网聊天前我会告诉你"
        │   - "截图、剪贴板默认是关的"
        │   - "这些不是法务条款，是我的承诺"
        ├── 右下角"查看完整数据策略"链接 → 模态展开正式版条款
        ├── 底部按钮："我懂了，一起开始" / "再看一眼条款" / "退出"
        └── 用户点"我懂了" → 视为同意，继续
        ↓
        Step 2-6：沿用 v0.4
        ↓
[创建默认配置 + 标记 onboarded=true]
 ↓
[进入主态，桌宠出现，状态 IDLE]
```

### 1.2 等价性保证
- "我懂了"按钮在数据库与日志中等价记录为 `consent.granted=true`，附带 `consent.method='soul_pledge'`、`consent.version=<文案版本号>`。
- 文案版本号在 M0 决策周由产品+法务双签字定版后写入应用资源，不可在线热更。
- 如未来法务要求更新，文案版本号 +1，下次启动用户需要再次确认（次数 >1 时弹"内容已更新"提示）。

## 13. 自由活动子状态机（v0.5 新增）

### 13.1 主状态机变更

`IDLE` 内部新增子状态 `WANDERING`：

```
[IDLE]
  ├── (默认) STILL：桌宠在原地待命
  └── WANDERING：桌宠正在小幅"逛桌面"
```

进入 `FOCUS` / `REMIND` / `UPDATING` / `ERROR` 时立即 `force_stop_wandering()`，回到 STILL。

### 13.2 触发流

```
[LivingPetService 调度器]
 ↓ 每 5-15 分钟随机抖动
[检查前置条件]
 ├── 主状态是否 == IDLE? 否 → 不触发
 ├── BOSS_KEY_HIDDEN? 是 → 不触发
 ├── 用户已关闭"逛桌面"? 是 → 不触发
 └── 全部通过 ↓
       [选择目标位置]（不超过屏宽 5%，不跨屏）
        ↓
       emit 'pet.wandering' { phase: 'start', targetX, targetY }
        ↓
       前端 PetCanvas 播放路径动画（5-15s）
        ↓
       到达 → emit 'pet.wandering' { phase: 'end' }
        ↓
       回到 STILL
```

### 13.3 中断分支
- FOCUS 启动 / 提醒触发 / 摸鱼模式切换 / 用户拖动桌宠：立即 `force_stop_wandering()` → 桌宠原地停下并回到合适位置。
- 进程被强杀 → 下次启动从 `pet_runtime_state.last_position` 还原。

## 14. 主动关心触发流（v0.5 新增）

### 14.1 总览

```
[OS 时钟] 每 30 秒
 ↓
[IdleDetector] 调 GetLastInputInfo()
 ↓
[计算 idle_ms]
 ↓
 ├── 跨过阈值（默认 90 min）→ 触发候选
 └── 跨阈值 + 用户回来 → emit UserActive
            ↓ 用于 LivingPet 精力恢复（不触发关心）

[ProactiveCareService::on_event]
 ↓
[频率/安静时段/启用状态检查]
 ├── 任一不通过 → 静默
 └── 全部通过 ↓
        [PersonaService.get_offline_template('empathy')]
         ↓ 抽样 1 条
        [写入 proactive_care_log]
         ↓
        emit 'proactive_care.fired' { logId, message }
         ↓
        [前端] 桌宠头顶气泡 + 心情图标短暂变化
         ↓
        [用户响应]
         ├── 点击气泡 → 弹出对话面板，预填关心文案
         │   → IPC: proactive_care.respond(clicked)
         ├── 主动回复 → 进入正常对话流
         │   → IPC: proactive_care.respond(replied)
         └── 8 秒未操作 → 气泡淡出
             → IPC: proactive_care.respond(dismissed)
```

### 14.2 各触发器（独立但共享频率池）

| 触发器 | 条件 | 文案 category |
|---|---|---|
| 长时间空闲 | idle_ms ≥ 90min | `empathy`（"还好吗"） |
| 深夜工作 | 23:00 后键盘活动 ≥ 30min | `gentle_remind` |
| 长时间未启动番茄 | 当日累计活动 > 2h 且无番茄 | `gentle_remind` |
| 跨日纪念日 | MilestoneService 推送 | `celebration`（无则降级 `greeting`） |

频率池是**共享的**：所有触发器加起来不超过 4 次/日、间隔 ≥ 2 小时。

### 14.3 边界情况

| 情况 | 处理 |
|---|---|
| 系统休眠后唤醒，idle_ms 突然爆表 | IdleDetector 检测到"上次轮询到现在的间隔 > 5 min" → 视为"休眠期" → 唤醒后 30 秒内不触发关心 |
| 多显示器 / RDP 远程会话 | 检测会话类型为 RDP → 模块 J 默认关闭（避免远程操作时被骚扰） |
| 用户在勿扰时段 | 不触发；勿扰结束后不"补提"（避免堆积爆炸） |
| 当前主状态是 FOCUS | 不触发（避免打断专注） |
| 用户连续 dismiss 3 次 | 自动调高阈值 +30min（自适应；可在设置看到提示并手动恢复） |

### 14.4 自适应（v0.5 内做轻量版，重度版本 P1-R2）
- 短期窗口（最近 24h）内 dismiss ≥ 3 次 → 阈值 +30 min。
- 短期窗口内 clicked / replied ≥ 2 次 → 阈值 -15 min（不低于 60 min）。
- 任何调整都通过 telemetry 上报，便于回归。

## 15. 摸鱼模式切换流（v0.5 新增）

### 15.1 切换为隐藏

```
[Global Shortcut] Ctrl+Shift+B
 ↓
[BossKeyService::toggle]
 ↓ 当前是显示态
[拍快照]
 - 当前各窗口位置 / 可见性
 - 桌宠当前 mood / energy / wandering 状态
 ↓
[force_stop_wandering] (如果在逛)
 ↓
[依次 hide]
 - pet 窗口
 - chat 面板（如打开）
 - workshop / settings 窗口（如打开）
 ↓
[托盘图标变更]"摸鱼中"
 ↓
[BossKeyState.hidden = true]
 ↓
emit 'boss_key.toggled' { hidden: true }
```

### 15.2 隐藏期间的事件处理

| 事件 | 行为 |
|---|---|
| 软提醒触发 | 缓冲到 `boss_key_pending_reminders` 队列，不弹通知 |
| 硬提醒触发 | 同上缓冲（v0.5 决策：摸鱼时硬提醒也缓冲，因为用户的语义就是"现在不能见人"）|
| 番茄钟到点 | 计时正常，REST 也正常进入，但桌宠形象不显示；恢复后桌宠在 REST 状态 |
| 主动关心触发 | 跳过（不计入今日 4 次额度） |
| 自由活动调度 | 跳过（条件不通过） |
| 用户从其他渠道拖入文件 | 不响应（无 hitbox 可达） |
| 自动更新可用 | 不弹气泡，等恢复后再说 |

### 15.3 切换为显示

```
[Global Shortcut] Ctrl+Shift+B
 ↓
[BossKeyService::toggle]
 ↓ 当前是隐藏态
[读取快照]
 ↓
[依次 show 各窗口到原位置]
 ↓
[处理缓冲队列]
 ├── 缓冲提醒 ≥ 2 条 → 桌宠用一句话合并提示
 │   "回来了？刚才我留了 N 条提醒在这"
 └── = 1 条 → 直接展示该提醒
 ↓
[BossKeyState.hidden = false]
 ↓
emit 'boss_key.toggled' { hidden: false }
```

### 15.4 失败/异常分支

| 分支 | 处理 |
|---|---|
| 快捷键注册失败 | 启动期提示用户改键，允许从托盘菜单手动切换 |
| 隐藏过程中桌宠窗口已被外力关闭 | hide 命令静默忽略，恢复期跳过该窗口 |
| 摸鱼期间应用崩溃重启 | 启动期检测到上次未正常关闭且 `bosskey_pending=true` → 默认恢复显示态（用户看见所有窗口正常） |

## 16. 文件拖入流（v0.5 新增）

### 16.1 路径

```
[Resource Manager / 桌面] User drags file(s)
 ↓ 拖到桌宠 hitbox
[Tauri file-drop event] 含 paths + cursor x,y
 ↓
[Frontend] hitbox check
 ├── 不在 hitbox → 不响应（默认行为）
 └── 在 hitbox ↓
        [IPC: file_drop.preflight(paths)]
         ↓
        [FileDropHandler]
          ├── 类型/数量/大小校验
          │   ├── 不通过 → 返回 { ok: false, hint }
          │   └── 通过 ↓
          ├── 提取文本（PDF 用 pdfium）
          │   ├── 失败 → { ok: false, hint: 'PDF 解析失败' }
          │   └── 成功 ↓
          └── 决定 available_actions（在线/离线下不同）
                 ↓
        返回 { ok: true, file_text_cached: '<cache_key>', available_actions }
         ↓
        [前端] 桌宠头顶展开 3 个动作泡泡
        [Telemetry: file_drop.bubbles_shown]
         ↓
        [User clicks 'summarize']
         ↓
        [IPC: file_drop.handle_action('summarize', cache_key)]
         ↓
        [ChatService.send] with file_text 作为单次上下文
         ↓
        [流式回复]
         ↓
        [写 messages 表]
         - role: user
         - content: 仅记录"已请求总结 <文件名>"，**不记录文件原文**
         - role: assistant
         - content: 完整回复（用户看到的）
         ↓
        [清理缓存] 会话窗口关闭后立即删 cache/file_extract/<cache_key>
```

### 16.2 离线分支
- preflight 返回 `available_actions = ['rename']`（仅"重命名建议"用本地规则）。
- "总结 / 解释" 灰显并提示"等联网了再聊"。

### 16.3 大文件保护
- 文本 > 5MB 或 PDF 提取后 > 5MB → preflight 返回 `confirm_required=true`。
- 前端弹出"文件较大，可能消耗较多 token，是否继续"。

### 16.4 多文件
- 一次拖入最多 3 个；超过提示"请分批"。
- 多文件时动作泡泡变为：总结全部 / 解释每个 / 重命名每个。

## 17. 跨日纪念日触发流（v0.5 新增）

### 17.1 检查时机

```
触发点：
1. 启动期（MigrationService 后）
2. reminder_completed 事件
3. pomodoro_completed 事件
4. todo_completed 事件
5. 每日凌晨 00:01 的定时唤醒检查
```

### 17.2 流程

```
[触发点]
 ↓
[MilestoneService::check_now]
 ↓
[依次评估每条规则]
 ├── first_launch_7d / 30d / 100d / 365d
 ├── reminder_streak_7 / 30 / 100
 ├── pomodoro_count_10 / 50 / 100 / 500
 └── todo_count_10 / 100 / 1000
 ↓
[查询 milestones 表过滤已触达]
 ↓
[新触达的逐条处理]
 ↓
 ├── insert into milestones (id, ...)
 ├── 通过 ProactiveCareService 触发关心（category='celebration'）
 │   - 注意：里程碑触发不受频率上限限制（每日最多 4 次的"自然关心"额度独立）
 │   - 但同一时间点多个里程碑命中 → 合并为一条庆祝消息
 ├── emit 'milestone.reached' { id, message }
 └── Telemetry 上报
 ↓
[前端] 桌宠播一句应景的话 + 视情况展示一次特殊动作（如撒花、鞠躬）
```

### 17.3 时区与时钟回调

| 情况 | 处理 |
|---|---|
| 用户系统时区变更 | 已触达里程碑不撤销；新触发以变更后时区为准 |
| 系统时钟被往前调（用户作弊） | "首次启动 +N 天"类规则要求 `now - first_launch_at >= N天 且 now > first_launch_at + 1天` 双条件 |
| 系统时钟被往后调（修复时间） | 已触达不重新触发（`milestones.id` PK 唯一） |
| 跨日切换时未启动 | 凌晨 00:01 唤醒不可靠 → 改在下次启动时统一检查（已包含） |

## 18. 状态机总图（v0.5 重写）

### 18.1 主状态（含 v0.5 子态）

```
                              ┌─────────────┐
                              │   BOOTING   │
                              └─────┬───────┘
                                    │ 配置就绪
                       ┌────────────┴──────────────┐
                       │                            │
                       ▼                            ▼
                ┌────────────┐              ┌──────────────────────┐
                │ ONBOARDING │  Step 6 完成 │     IDLE             │◀──┐
                │ (含 Soul   │ ───────────► │  ┌──────────────┐    │   │
                │  Pledge)   │              │  │ STILL        │    │   │
                └────────────┘              │  │ WANDERING(子)│    │   │
                                            │  └──────────────┘    │   │
                                            └─┬─────┬──────────────┘   │
                                              │     │                   │
                                              │     │ 番茄钟开始         │
                          完成提醒 / 忽略 / 稍后│     ▼                   │
                                              │   ┌─────┐               │
                                              │   │FOCUS│               │
                                              │   └──┬──┘               │
                                              │      │ 倒计时完成        │
                                              │      ▼                  │
                                              │   ┌─────┐               │
                                              │   │REST │───────────────┘
                                              │   └──┬──┘ 休息结束
                                              │      │
                                              │      │ 硬提醒
                                              ▼      ▼
                                          ┌────────────┐
                                          │   REMIND   │
                                          └────────────┘

       ┌──────────┐           ┌──────────┐
       │ UPDATING │           │  ERROR   │
       └──────────┘           └──────────┘
        （任意态可入：用户触发更新 / 致命错误）

【叠加态】BOSS_KEY_HIDDEN：可与上述任意非 ERROR 态并存
   - 在此叠加态下：UI 全部不可见、提醒缓冲、自由活动跳过、主动关心跳过
   - 退出叠加态时合并展示缓冲事件
```

### 18.2 子状态：IDLE 内部

```
IDLE
 ├── STILL（默认）
 └── WANDERING（LivingPet 调度，5-15 分钟随机进入；持续 5-15 秒）
```

### 18.3 心情（独立维度，与主状态正交）

| Mood | 触发条件 | 视觉 |
|---|---|---|
| `happy` | 互动后 10 分钟内 | 头顶 ✨ 等正向符号 |
| `sleepy` | energy < 30 | 头顶 zZ 符号，动作迟缓 |
| `focused` | 主状态 = FOCUS | 头顶 🎯 符号（自动覆盖其他） |
| `cozy` | 23:00 后或周末早上 | 头顶 🌙 / ☕ |
| `neutral` | 默认 | 无符号 |

### 18.4 状态迁移规则补充表

| From → To | 条件 | 备注 |
|---|---|---|
| `IDLE.STILL → IDLE.WANDERING` | LivingPet 调度命中 + 全部前置通过 | v0.5 新增 |
| `IDLE.WANDERING → IDLE.STILL` | 路径走完 / 被打断 | v0.5 新增 |
| `* → BOSS_KEY_HIDDEN`（叠加） | 用户按摸鱼快捷键 | v0.5 新增；叠加态，主状态不变 |
| `BOSS_KEY_HIDDEN → 取消` | 用户再按 / 通过托盘恢复 | v0.5 新增 |
| `IDLE / REST → REMIND（庆祝）` | MilestoneService 触发 | v0.5 新增；走主动关心通道而非传统 reminder 通道 |

## 19. UAT 关键场景对应（v0.5 增补）

| 场景 | 对应章节 |
|---|---|
| 灵魂宣誓页等价于隐私同意 | § 1.2 |
| 自由活动不在 FOCUS 期触发 | § 13.1, § 13.2 |
| 主动关心 24 小时严格 ≤ 4 次 | § 14.2 |
| 安静时段不触发任何主动关心 | § 14.3 |
| 摸鱼模式期间硬提醒被合并而非丢失 | § 15.2-15.3 |
| 文件拖入超大文件二次确认 | § 16.3 |
| 文件原文不写入对话历史 | § 16.1（messages 表） |
| 跨日纪念日不重复触发 | § 17.3 |
| 系统休眠唤醒不暴击主动关心 | § 14.3 |
| RDP 远程下模块 J 自动关闭 | § 14.3 |

## 20. 实施提示（v0.5 追加）

1. **状态机扩展**：Rust 主进程的 `PetState` enum 升级为带子态的结构 `{ main: PetMainState, sub: Option<IdleSubState>, overlay: HashSet<OverlayState> }`，编译期穷尽检查避免遗漏。
2. **频率池统一管理**：所有"主动出现的桌宠互动"（关心 / 庆祝 / 自由活动）都查同一个 ProactiveCareService 频率池，避免分散造成超额。
3. **自由活动与心情图标解耦**：自由活动只负责位置，心情图标只反映状态，不互相驱动。
4. **状态变迁全打埋点**：所有 PetState 变化、子态进入/退出、叠加态切换都触发 `pet.state_changed` 事件用于回归。
5. **测试 fixture**：建立"模拟时钟 + 模拟 idle_ms"测试 fixture，用于 UAT 频率控制场景的可重复验证。

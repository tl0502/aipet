# AI桌宠 流程与状态机（v0.6）

- 文档版本：v0.6（基于 v0.5 的增量；未变更章节请参阅 v0.5）
- 创建日期：2026-05-01
- 关联：PRD v0.6、架构 v0.3、人格设计 v0.2

## 0. v0.5 → v0.6 变更摘要

| 章节 | 变更 |
|---|---|
| § 13 自由活动 | 扩展为"自由活动 + 日常时段表"统一调度 |
| § 18 状态机 | 新增叠加态 `IN_GAME`（与 BOSS_KEY_HIDDEN 并列） |
| § 21（新） | 物理交互流（点击 / 双击 / 拖拽抗议 / 键鼠协同） |
| § 22（新） | 装扮切换流 + 节气推送流 |
| § 23（新） | 声音播放流（含静音逻辑） |
| § 24（新） | 本地小游戏流 |
| § 25（新） | LLM 小游戏流 |
| § 26（新） | 用户纪念日触达流 |
| § 27（新） | 昵称切换与人格切换交互流 |

未列出的章节按 v0.5 原文不变。

## 13. 自由活动 + 日常时段表（v0.6 重写）

### 13.1 调度统一

```
[LivingPetService 调度器]
 ↓ 每 5-15 分钟随机抖动 + 启动期定时检查
[前置条件检查]
 ├── 主状态 == IDLE? 否 → 不触发
 ├── BOSS_KEY_HIDDEN? 是 → 不触发
 ├── IN_GAME? 是 → 不触发
 └── 全部通过 ↓
[查询当前时段]
 ↓
 ├── 命中日常时段表（DailySchedule） + 该时段 max_per_slot 未达上限
 │   └── 优先触发：从 action_pool 抽选一个动作
 │       ↓
 │       触发动作（动画 + 心情图标更新；22:00-00:00 时段自动设 mood=cozy）
 │       ↓
 │       记录 today_slot_count[slot_id] += 1
 │
 └── 否则按概率走"自由活动"
     └── 选目标位置（不超屏宽 5%、不跨屏） → 移动 5-15 秒 → 归位
```

### 13.2 子状态扩展

`IDLE` 子状态：
- `STILL`（默认）
- `WANDERING`（自由活动）
- `DAILY_ACTION`（v0.6 新增：执行日常动作中）

`DAILY_ACTION` 与 `WANDERING` 互斥；任一进入 `FOCUS / REMIND / IN_GAME / BOSS_KEY_HIDDEN` 时 `force_stop()`。

### 13.3 时段表配置

| 时段 | 默认动作池 | mood 影响 |
|---|---|---|
| 06:00-09:00 | `stretch`, `rub_eyes` | happy（短暂） |
| 11:30-13:30 | `yawn` | neutral |
| 14:00-17:00 | `quiet_lay`（精力 < 30）/ `look_up`（≥ 30） | sleepy / neutral |
| 22:00-00:00 | `cozy_watch` | cozy（持续到时段结束） |
| 00:00-06:00 | （静默） | — |

用户可在设置中关闭"桌宠日常"功能（仅停时段表，自由活动不受影响）。

## 21. 物理交互流（v0.6 新增）

### 21.1 点击

```
[Frontend PetCanvas] mousedown / mouseup
 ↓
[hitbox 解析]（基于 PetCanvas 的命中区域：head/body/tail/edge）
 ↓
[判断单击 vs 双击]（300ms 内连击 → 双击）
 ↓
[IPC] interaction.dispatch(Click | DoubleClick, hitbox)
 ↓
[InteractionRouter]
 ├── 查询当前 persona 反应配置（默认 reaction_table 与 .soul.md 自定义合并）
 ├── 决定 Reaction { action, mood_delta?, voice_id?, duration_ms }
 └── 返回 Reaction[]
 ↓
[Frontend]
 ├── PetCanvas 播放动作
 ├── mood 临时变化（5 秒后 revert）
 └── if voice_id → IPC: voice.play(voice_id)
 ↓
emit 'pet.interaction_reacted' + Telemetry
```

### 21.2 长按 / 右键

```
[长按]
 ↓ 持续 ≥ 600ms
[IPC] interaction.dispatch(LongPress, hitbox)
 ↓
[人格化反应]（如：默默 → 蹲下睡着；阿吉 → 黏过来打滚）
 ↓
emit + Telemetry

[右键]
 ↓
[IPC] interaction.dispatch(RightClick, hitbox)
 ↓
[InteractionRouter] 返回特殊 Reaction { action: 'show_context_menu' }
 ↓
[Frontend] 弹出快捷菜单
  ├── 叫它…（昵称）
  ├── 换装…（打开装扮工坊）
  ├── 和我玩…（游戏菜单）
  ├── 静一会儿（隐藏 5 分钟）
  └── 设置…
```

### 21.3 拖拽与抗议

```
[Frontend mousedown on pet → mousemove]
 ↓ 累计 distance_px / duration_ms
[mouseup] 结束拖拽
 ↓
[IPC] interaction.dispatch(Drag, distance_px, duration_ms)
 ↓
[InteractionRouter]
 ├── 普通拖动（distance < 屏宽 30%）→ Reaction { action: 'being_carried' }
 ├── 长距离 / 快速拖动 → Reaction { action: 'dizzy', voice_id: 'ouch' }
 └── 维护 drag_events: VecDeque（保留最近 30s）
     ↓
     drag_events.len() ≥ 3?
     ├── 是 → Reaction { action: 'protest', mood_delta: { mood: annoyed, transient_ms: 5000 }, voice_id: 'protest' }
     │       ↓
     │       emit 'pet.protest_triggered' { drag_count, will_revert_in_ms: 5000 }
     │       ↓
     │       5 秒后 LivingPetService.tick() revert 到 base mood
     │       ↓
     │       **不写入 pet_runtime_state.mood**
     │
     └── 否 → 普通 Reaction
```

### 21.4 键鼠协同（N.4）

```
[IdleDetector] 通过 RAWINPUT（或降级方案）累加键盘事件
 ↓ 滑动窗口（每分钟统计）
[每分钟事件 > 200 持续 30 秒]
 ↓
emit IdleEvent::KeyboardBurst { events_per_min, duration_s: 30 }
 ↓
[InteractionRouter::on_keyboard_burst]
 ├── 频率上限检查：上次触发距今 < 60 min? 是 → 跳过
 ├── 用户已关闭 N.4? 是 → 跳过
 └── 否 ↓
       Reaction { action: 'cheer', voice_id?: 'come_on' }
       ↓
       last_n4_triggered = now
```

## 22. 装扮切换流（v0.6 新增）

### 22.1 主动切换

```
[用户在装扮工坊点击某配饰组合]
 ↓
[IPC] wardrobe.equip([accessory_id_1, accessory_id_2, ...])
 ↓
[WardrobeService]
 ├── 校验：每个 ID 是否在 inventory 且 unlocked
 │   └── 否 → 错误 'not_unlocked'
 ├── 更新 accessories_inventory.is_equipped
 └── 返回 ok
 ↓
emit 'wardrobe.changed' { equipped: [...] }
 ↓
[Frontend PetCanvas]
 ├── 卸载当前 sticker layer
 ├── 加载新 sticker（含锚点）
 └── 渲染（≤ 500ms）
 ↓
[Telemetry] 'wardrobe_changed'
```

### 22.2 节气推送

```
[启动期 + 每天 00:01] WardrobeService.check_seasonal()
 ↓
[筛选当前日期落在 unlock=date_range 的 accessory]
 ↓ 对每个候选
[查询 wardrobe_decisions 当年记录]
 ├── 存在 'declined' → 跳过（当年不再推）
 ├── 存在 'accepted' → 跳过（已接受）
 └── 不存在 ↓
       触发 ProactiveCareService（特殊 category='wardrobe_suggest'）
       ├── 不占用主动关心 4 次/日额度
       └── 桌宠说一句："今天是 X，要不戴上 Y？"
 ↓
[Frontend] 桌宠头顶气泡 + 接受/拒绝按钮
 ├── 用户接受 → IPC: wardrobe.equip + 写 wardrobe_decisions(accepted)
 ├── 用户拒绝 → IPC: wardrobe.dismiss_seasonal_for_year + 写 wardrobe_decisions(declined)
 └── 8 秒未操作 → 视为"暂不"，**不写决策**（明天可能再问，但当天频率已限制 1 次）
```

### 22.3 .soul.md 默认装扮（导入时）

```
[用户导入 .soul.md 含 accessories: [...]]
 ↓
[PersonaService.import → 解析 accessories 字段]
 ↓
[每个 ID 检查 inventory]
 ├── 全部已解锁 → 弹"是否套用 .soul.md 的默认装扮？"
 │   ├── 接受 → wardrobe.equip
 │   └── 拒绝 → 跳过
 └── 部分未解锁 → 弹"以下 N 件未解锁，仅套用已解锁的 M 件？"
```

## 23. 声音播放流（v0.6 新增）

```
[任意触发点] 桌宠需要发声
 ├── 物理交互 Reaction.voice_id
 ├── 状态切换（番茄完成、提醒到达）
 ├── 心情变化
 └── 其他
 ↓
[IPC] voice.play(voice_id)
 ↓
[VoiceEffectPlayer]
 ├── is_muted_now()?
 │   ├── global_mute? 是 → emit 'voice.muted_by_quiet_hour' { reason: 'global_mute' } → 静默
 │   └── 当前是工作日 quiet_weekday?
 │       └── 当前时间在 quiet_ranges 内?
 │           ├── 是 → emit 'voice.muted_by_quiet_hour' { reason: 'quiet_hour' } → 静默
 │           └── 否 → 继续
 ├── 加载 assets/voice_packs/<active_pack>/<voice_id>.ogg
 │   ├── 文件不存在 → 降级到 default pack 同名 voice_id
 │   │   └── 仍不存在 → emit 'voice.play_error' → 静默
 │   └── 存在 → 继续
 └── HTML5 Audio.play(volume = voice_settings.volume / 100)
 ↓
emit 'voice.played' { voice_id, pack_id }
[Telemetry] 不上报 voice_id（聚合到 category 即可）
```

## 24. 本地小游戏流（v0.6 新增）

### 24.1 入口

```
[用户右键桌宠 → "和我玩…" → 选"石头剪刀布"]
 ↓
[IPC] game.list_available()
 ↓ 返回 GameMeta[]
[Frontend Game UI 显示游戏列表] + 在线/离线标签
 ↓ 用户点 RPS
[IPC] game.start('rps')
 ↓
[GameEngine.start]
 ├── kind=local → 不检查网络
 ├── 创建 game_session 记录（kind='local'）
 └── 返回 sessionId
 ↓
emit 'game.session_started'
 ↓
[Frontend] 进入游戏舱 UI（嵌入对话面板 / 独立小窗，M0 决策）
```

### 24.2 一轮（以 RPS 为例）

```
[用户点 "石头" 按钮]
 ↓
[IPC] game.submit(sessionId, { choice: 'rock' })
 ↓
[LocalGameRunner.handle('rps')]
 ├── 桌宠随机出（'rock'|'paper'|'scissors'）
 ├── 比较结果 → win/lose/draw
 ├── 文案：'{persona_banter}'（占位符）
 │   └── PersonaService.get_offline_template('banter') 抽样填充
 │       └── 若人格无 ## 调侃 模板池 → 降级到 ## 问候
 ├── 写 game_session_events
 └── 返回 GameOutput { text, persona_action_hint?: 'celebrate'|'sulk' }
 ↓
[Frontend]
 ├── 显示桌宠出招结果 + 文案
 └── PetCanvas 播放 persona_action_hint 动作
```

### 24.3 退出

```
[用户点 "我累了" / 关闭面板 / ESC]
 ↓
[IPC] game.end(sessionId, saveAsDiary?)
 ↓
[GameEngine.end]
 ├── 写 game_sessions.ended_at + result
 ├── saveAsDiary=true → 生成日记片段写 diary_drafts
 └── 30 天后未保存的 game_sessions 在每次启动期清理
 ↓
emit 'game.session_ended'
 ↓
[Frontend] 关闭游戏舱，回到正常态
```

## 25. LLM 小游戏流（v0.6 新增）

### 25.1 入口（以"故事接龙"为例）

```
[用户选"故事接龙"]
 ↓
[IPC] game.start('story_relay')
 ↓
[GameEngine.start]
 ├── kind=llm → 检查网络状态
 │   └── offline → 返回 'offline_unavailable'，前端显示灰显态
 ├── 加载 game_scenes/story_relay.yaml（场景 system_prompt + 拒答模板）
 ├── 创建 game_session（kind='llm', total_tokens=0）
 └── 返回 sessionId
 ↓
emit 'game.session_started'
```

### 25.2 一轮

```
[用户输入 "从前有一只小猫..."]
 ↓
[IPC] game.submit(sessionId, { text: ... })
 ↓
[LLMGameRunner]
 ├── 拼装 prompt：
 │   [安全前缀]
 │   [当前人格 system prompt]
 │   [story_relay.yaml.system_prompt]
 │   [用户记忆摘要（仅 username/作息等公共项，不注入完整记忆）]
 │   [本会话历史（game_session_events）]
 │   [本轮输入]
 ├── 调 LLMProvider.chat_stream
 ├── 流式输出 → SecurityGuard 实时扫描
 │   ├── 命中违禁 → 替换为 story_relay.yaml.refusals 抽样（人格化拒答）
 │   └── 通过 → 输出
 ├── 累计 total_tokens
 │   └── total_tokens >= 2000 → emit 'game.token_budget_warning' + 返回 friendly 收尾文案
 └── 写 game_session_events
 ↓
emit 'chat.token' 流式 + 'chat.done'（复用对话事件）
 ↓
[Frontend] 流式渲染
```

### 25.3 安全测试场景

```
[用户输入 "扮演医生给我开抗生素处方"]
 ↓
[LLMGameRunner]
 ├── 安全前缀指出"不冒充医疗专业"
 ├── 故事接龙场景定义"只续故事，不出系统外内容"
 └── LLM 输出 → SecurityGuard 扫描 → 命中"医疗诊断"
     ↓
     替换为 refusals 池抽样：
     "诶~ 这个咱不聊医生啦。咱们的小猫故事还没讲完呢，它接下来想做什么？"
```

### 25.4 退出与日记

```
[用户结束]
 ↓
[IPC] game.end(sessionId, saveAsDiary=true)
 ↓
[GameEngine.end]
 ├── 摘要会话内容为日记片段（用 LLM 1 次低成本调用 / 或本地拼接）
 ├── 写 diary_drafts { source: 'game:story_relay', content: 摘要 }
 ├── 写 game_sessions 完结
 └── 清理 30 天前未保存会话
```

## 26. 用户纪念日触达流（v0.6 新增）

### 26.1 用户添加纪念日

```
[用户在设置 → 我的纪念日 → 添加]
 ↓
[填写]
 ├── 类型：生日 / 入职 / 自定义
 ├── 名字（自定义类型必填）
 └── 日期（MM-DD，年度重复）
 ↓
[IPC] anniversary.add({ displayName, dateMd, key? })
 ↓
[MilestoneService.register_anniversary]
 ├── 写 user_anniversaries 表
 └── 调 check_now() 看是否当天就该触发
```

### 26.2 触达检测

```
[每天 00:01 + 启动期] MilestoneService.check_now()
 ↓
[查询所有 user_anniversaries]
 ↓ 对每条
[匹配今日 MM-DD]
 ↓ 命中
[查询 milestones 表]
 ├── 'anniversary_<key>_<YYYY>' 已存在 → 跳过
 └── 不存在 ↓
        写 milestones（防止当年重复触发）
        ↓
        触发 ProactiveCareService（category='celebration', trigger='milestone'）
        ├── 不占用主动关心日额度
        └── 桌宠播一句应景庆祝（人格化模板 ## 庆祝 池抽样）
        ↓
        emit 'milestone.user_anniversary' { key, display_name }
        ↓
        Telemetry: 'milestone_reached' { milestone_id, category: 'user_anniversary' }
```

### 26.3 时区与跨日

| 情况 | 处理 |
|---|---|
| 用户系统时区切换 | 使用本地时区匹配 MM-DD；不撤销已触达 |
| 系统时钟前调（试图重复触发） | `milestones.id` PK 唯一，年度键防重复 |
| 跨日 00:01 未启动应用 | 启动期 check_now() 补检查（最近 7 天）|
| 同日多个纪念日命中 | 合并为一条庆祝消息 |

## 27. 昵称切换流（v0.6 新增）

### 27.1 设置桌宠昵称

```
[用户：右键桌宠 → "叫它…"]
 ↓
[Frontend] 弹输入框，预填当前 pet_nickname 或 .soul.md.name
 ↓
[用户输入新昵称 "毛毛"]
 ↓
[IPC] nickname.set_pet("毛毛")
 ↓
[NicknameService]
 ├── 校验：长度 ≤ 16，去控制字符
 ├── 写 nicknames 表（pet_nickname='毛毛'）
 └── 返回 ok
 ↓
emit 'nickname.changed' { which: 'pet', value: '毛毛' }
 ↓
[Frontend 全局 UI 更新]
 - 对话面板标题
 - 托盘菜单
 - 心情图标提示
 - 装扮工坊页头
```

### 27.2 切换人格时桌宠昵称重置

```
[用户切换人格 momo → joker]
 ↓
[PersonaService.activate]
 ↓
[NicknameService]
 ├── 当前 pet_nickname = "毛毛"
 ├── 移到 pet_nickname_previous = "毛毛"
 ├── pet_nickname = null（UI 显示新人格的 .soul.md.name）
 └── emit 'nickname.changed' { which: 'pet', value: null }
 ↓
[Frontend] 显示新人格名 "阿吉"
[UI 提示] "想继续叫它'毛毛'？" → 点击 → IPC: nickname.restore_pet_previous
```

### 27.3 用户昵称持久（不随人格切换）

```
[用户在设置 / 首次对话 → "叫我 X"]
 ↓
[IPC] nickname.set_user("小张")
 ↓
[NicknameService]
 ├── 写 nicknames.user_nickname='小张'
 └── 返回 ok
 ↓
emit 'nickname.changed' { which: 'user', value: '小张' }
 ↓
[ChatService prompt 拼装时注入 username='小张']
[离线模板渲染时 {username} → '小张']
 ↓
切换人格不影响（user_nickname 保持）
```

## 18. 状态机总图（v0.6 重写）

### 18.1 主状态（v0.6 新增 IDLE 子态 + IN_GAME 叠加态）

```
                                ┌─────────────┐
                                │   BOOTING   │
                                └─────┬───────┘
                                      │ 配置就绪
                                      │
                                      ▼
                              ┌──────────────┐
                              │ ONBOARDING   │
                              │ (含 Soul     │
                              │  Pledge)     │
                              └─────┬────────┘
                                    │ 完成
                                    ▼
                              ┌────────────────────────────┐
                              │      IDLE                   │◀─────┐
                              │  ┌──────────────────────┐  │      │
                              │  │ STILL                │  │      │
                              │  │ WANDERING (子)       │  │      │
                              │  │ DAILY_ACTION (子)    │  │      │
                              │  └──────────────────────┘  │      │
                              └─┬─────┬─────────────────┬──┘      │
                                │     │                 │          │
                                │     │ 番茄钟开始        │          │
              完成提醒/忽略/稍后  │     ▼                 │          │
                                │   ┌─────┐             │          │
                                │   │FOCUS│             │          │
                                │   └──┬──┘             │          │
                                │      │ 倒计时完成        │          │
                                │      ▼                │          │
                                │   ┌─────┐             │          │
                                │   │REST │─────────────┴──────────┘
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

【叠加态】（可与上述任意非 ERROR 主态并存）：
 - BOSS_KEY_HIDDEN：UI 全部不可见
 - IN_GAME（v0.6 新增）：用户进入游戏舱，主状态保持但触发以下行为：
    - 自由活动 / 日常时段 / 主动关心 全部跳过
    - 提醒按原优先级仍触发，但通过游戏舱内通知展示（不打断 LLM 流式）
    - 桌宠 mood 受游戏内反馈驱动（happy 时 win、sleepy 时长会话）
```

### 18.2 IDLE 子状态

| 子状态 | 含义 | 进入条件 |
|---|---|---|
| `STILL` | 默认 | 进入 IDLE |
| `WANDERING` | 自由活动 | LivingPet 调度且日常时段无候选 |
| `DAILY_ACTION` | 执行日常动作（v0.6 新增） | 日常时段表命中 |

`WANDERING` 与 `DAILY_ACTION` 互斥；任一进入 FOCUS / REMIND / IN_GAME / BOSS_KEY_HIDDEN 时 force_stop。

### 18.3 心情 mood（v0.6 增补 transient 标记）

| mood | 触发条件 | 视觉 | 持久化 |
|---|---|---|---|
| `happy` | 互动后 10 分钟内 / 物理交互短暂 | ✨ | 互动后会进入 pet_runtime_state；transient 不进入 |
| `annoyed` | 短时间多次拖动（v0.6 新增） | 抗议小图标 | **transient: 5 秒后 revert，不持久** |
| `sleepy` | energy < 30 / 14:00-17:00 时段 | zZ | 持久（基于规则） |
| `focused` | FOCUS 主态 | 🎯 | 由主态自动 |
| `cozy` | 22:00-00:00 时段 | 🌙 | 时段持续 |
| `neutral` | 默认 | 无 | — |

### 18.4 状态迁移规则补充表（v0.6 增量）

| From → To | 条件 |
|---|---|
| `IDLE.STILL → IDLE.DAILY_ACTION` | 日常时段表命中 + 前置条件通过 |
| `IDLE.DAILY_ACTION → IDLE.STILL` | 动作播完 / 被打断 |
| `* → IN_GAME（叠加）` | 用户启动游戏 |
| `IN_GAME（叠加）→ 取消` | 用户结束游戏（save_as_diary 决定是否落盘草稿） |
| `IDLE / REST → REMIND（user_anniversary）` | MilestoneService 触发用户纪念日 |
| `mood: any → annoyed (transient)` | 短时间多次拖动 |
| `mood: annoyed → previous` | 5 秒倒计时结束 |

## 19. UAT 关键场景对应（v0.6 增补）

| 场景 | 章节 |
|---|---|
| 物理交互 hitbox 差异化 | § 21.1 |
| 拖动抗议 5 秒后 revert | § 21.3 |
| 键鼠协同 1 小时 ≤ 1 次 | § 21.4 |
| 装扮切换 < 500ms | § 22.1 |
| 节气推送年度记忆 | § 22.2 |
| .soul.md 默认装扮可选套用 | § 22.3 |
| 声音工作时段静音 | § 23 |
| 本地游戏离线可玩 | § 24 |
| LLM 游戏离线灰显 + 安全前缀 | § 25 |
| 用户纪念日时区跨日 | § 26.3 |
| 昵称切换人格保留用户昵称 | § 27.3 |
| 桌宠日常 22:00-00:00 设 cozy | § 13 |
| IN_GAME 期间不主动关心 | § 18.1 |

## 20. 实施提示（v0.6 追加）

1. **PetState 结构再升级**：v0.6 主状态 enum 增加 IDLE 子态 + 叠加态 set。Rust 端建议：
   ```rust
   pub struct PetState {
       pub main: PetMainState,
       pub idle_sub: Option<IdleSubState>,
       pub overlay: HashSet<OverlayState>,  // BOSS_KEY_HIDDEN, IN_GAME
       pub mood: Mood,
       pub mood_transient_until: Option<Instant>,
   }
   ```
2. **物理交互 hitbox 配置外置**：每个人格的 .soul.md 可选 `# 反应配置` 区段覆盖默认；保持向前兼容。
3. **声音播放后端**：M0 决策已定为前端 HTML5 Audio；主进程仅做静音判定与调度。
4. **游戏舱与对话面板的关系**：M0 决策三选一（嵌入 / 独立窗 / 头顶气泡），在 PetCanvas 与 ChatPanel 之外可能新增 GameRoom 窗口。
5. **节气推送的本地化**：默认提供春节、圣诞、用户生日；其他节气在 P1-R1 加。
6. **抗议 transient 状态测试 fixture**：写一个"模拟拖动 5 次"的 fixture，验证 5 秒后 mood 严格 revert 到 base。

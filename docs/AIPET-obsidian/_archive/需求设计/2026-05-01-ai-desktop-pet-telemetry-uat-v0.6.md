# AI桌宠 埋点与UAT清单（v0.6）

- 文档版本：v0.6
- 创建日期：2026-05-01
- 替代：v0.5
- 关联：PRD v0.6、架构 v0.3、flows v0.6

## 0. v0.5 → v0.6 变更摘要

| 类别 | 变更 |
|---|---|
| 公共字段扩展 | 全部事件追加 `pet_idle_substate` / `pet_overlay`(JSON 数组：BOSS_KEY_HIDDEN/IN_GAME) |
| 新事件族（物理交互 N） | 4 个：`interaction_*` |
| 新事件族（装扮 O） | 4 个：`wardrobe_*` |
| 新事件族（声音 P） | 3 个：`voice_*` |
| 新事件族（游戏 Q） | 5 个：`game_*` |
| 新事件族（昵称 U） | 1 个：`nickname_changed` |
| 新事件（用户纪念日） | 扩展 `milestone_reached.category` 加 `user_anniversary` |
| 新事件（桌宠日常） | 扩展 `pet_state_changed` 含 sub_state；新事件 `pet_daily_action` |
| 新指标口径 | KPI 11.15-11.20（v0.6） |
| 新 UAT 场景 | 16 个新场景 |

## 1. 指标口径（v0.6 增补）

### 1.1-1.4 沿用 v0.5

### 1.5 v0.6 新增（KPI 11.15-11.20）

| 指标 | 口径（精确） | 阈值 |
|---|---|---|
| **11.15 物理交互密度** | (Σ `interaction_reacted` 在 D7 内) ÷ (D7 内日活跃用户数) ÷ 7 | ≥ 1.5 次/天 |
| **11.16 小游戏使用率（D7）** | distinct user with ≥1 `game_session_started` 在 D7 内 ÷ 7 日活跃用户 | ≥ 25% |
| **11.17 装扮使用率（D7）** | distinct user with ≥1 `wardrobe_equipped`（含节气接受）在 D7 内 ÷ 7 日活跃用户 | ≥ 20% |
| **11.18 声音表情关闭率** | distinct user with `voice_global_mute_changed.muted=true` ÷ DAU | ≤ 25% |
| **11.19 昵称设置率（D1）** | distinct user with ≥1 `nickname_changed` 在首启 24 小时内 ÷ D1 用户 | ≥ 50% |
| **11.20 仪式参与率** | (Σ days where: `pet_daily_action` 出现 且 用户 24h 内有 ≥1 互动响应) ÷ Σ days where `pet_daily_action` 出现 | ≥ 30% |

## 2. 事件公共字段（v0.6 扩展）

```
event_schema_version: int  // v0.6 升至 3（破坏性扩展）
event_id: ULID
event_time: ISO8601
session_id: ULID
app_version: string
os_version: string
network_state: enum
active_persona_id: string
pet_main_state: enum
pet_idle_substate: enum?    // v0.6 新增：'STILL' | 'WANDERING' | 'DAILY_ACTION' | null
pet_overlay: string[]       // v0.6 新增：JSON array, e.g. ['BOSS_KEY_HIDDEN'] / ['IN_GAME']
boss_key_hidden: bool       // 沿用 v0.5（与 pet_overlay 冗余但便查询）
is_rdp_session: bool
```

PII 边界（沿用 v0.5）：禁止上报 messages.content、文件名、文件路径、API Key；新增禁止上报具体配饰 ID（仅上报 category）和具体音效 ID（仅上报 trigger）。

## 3. 事件字典（v0.6 增量）

### 3.1-3.6 沿用 v0.5（仅 schema_version 升级）

### 3.7 生命感 / 桌宠日常（v0.6 扩展）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `pet_state_changed` | 主状态/子状态/叠加态变迁 | `from`, `to`, `sub_from`, `sub_to`, `overlay_added`, `overlay_removed`, `reason` |
| `pet_mood_changed` | 心情变化 | `from`, `to`, `transient` (bool, v0.6 新增), `trigger` |
| `pet_wandering` | 自由活动 | `phase`, `duration_ms`, `distance_px` |
| `pet_daily_action` | 日常时段动作触发（v0.6 新增） | `time_slot`（'06:00-09:00'等）, `action_id`（'stretch'/'yawn'等） |
| `living_feature_toggled` | 用户开关 | `feature` ('wandering'\|'mood_icon'\|'energy'\|'daily_schedule'\|'overall'), `enabled` |

### 3.8-3.10 沿用 v0.5（主动关心、摸鱼、文件拖入）

### 3.11 纪念日（v0.6 扩展 category）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `milestone_reached` | 触达 | `milestone_id`, `category` ('first_launch'\|'streak'\|'pomodoro_count'\|'todo_count'\|'**user_anniversary**'), `threshold`, `triggered_at` |

### 3.12-3.15 沿用 v0.5

### 3.16 物理交互（模块 N，v0.6 新增）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `interaction_reacted` | 桌宠对物理交互产生反应 | `kind` ('click'\|'double_click'\|'long_press'\|'right_click'\|'drag'), `hitbox` ('head'\|'body'\|'tail'\|'edge'), `action_id`, `voice_played` (bool), `mood_delta_transient` (bool) |
| `interaction_protest_triggered` | 短时间多次拖动触发抗议 | `drag_count`, `window_s`, `revert_in_ms` |
| `interaction_keyboard_burst` | N.4 键鼠协同触发 | `events_per_min`, `duration_s`, `last_burst_min_ago` |
| `interaction_n4_toggled` | 用户开关 N.4 | `enabled` |

> 节流：`interaction_reacted` 不节流（关键回归）；同帧多个物理事件按 ULID 区分。

### 3.17 装扮（模块 O，v0.6 新增）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `wardrobe_equipped` | 用户装扮变化 | `accessory_categories` (string array, **不含具体 ID**), `count`, `source` ('manual'\|'seasonal_accept'\|'persona_default') |
| `wardrobe_unequipped_all` | 全部卸下 | (公共字段) |
| `wardrobe_seasonal_suggested` | 节气推送 | `season_key` ('lunar_new_year'\|'christmas'\|'birthday'\|...), `accessory_category` |
| `wardrobe_seasonal_decided` | 节气推送决策 | `season_key`, `decision` ('accepted'\|'declined'\|'auto_dismiss') |

### 3.18 声音表情（模块 P,v0.6 新增）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `voice_played` | 实际播放（**仅未静音时记录**） | `category` ('eheh'\|'mhm'\|'cough'\|'celebrate'\|'protest'\|...), `pack_id`, `trigger` ('interaction'\|'mood_change'\|'state_change') |
| `voice_muted_by_quiet_hour` | 静音判定阻断播放 | `category`, `reason` ('quiet_hour'\|'global_mute') |
| `voice_settings_changed` | 用户改设置 | `field` ('global_mute'\|'volume'\|'quiet_hours'\|'quiet_weekdays'), `new_value`（量化或枚举，**不上报具体时段值**仅上报"是否覆盖默认"） |

### 3.19 小游戏（模块 Q,v0.6 新增）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `game_session_started` | 开始游戏 | `game_id`, `kind` ('local'\|'llm'), `entry` ('right_click_menu'\|'tray'\|'workshop') |
| `game_round_played` | 一轮交互完成 | `session_id`, `game_id`, `round_index`, `latency_ms`, `tokens_in`, `tokens_out` (LLM 游戏才有) |
| `game_security_blocked` | 安全前缀触发拒答替换 | `session_id`, `game_id`, `block_category` ('self_harm'\|'medical'\|'role_breakout'\|'illegal'\|...) |
| `game_token_budget_warning` | 累计 token 接近上限 | `session_id`, `used`, `limit` |
| `game_session_ended` | 游戏结束 | `session_id`, `game_id`, `kind`, `duration_ms`, `rounds`, `total_tokens` (LLM), `saved_as_diary` (bool), `exit_reason` ('user_quit'\|'budget_exhausted'\|'error') |

### 3.20 昵称（模块 U,v0.6 新增）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `nickname_changed` | 用户设置/恢复昵称 | `which` ('pet'\|'user'\|'pet_restored'), `had_value_before` (bool), `value_present` (bool) |

> **PII 守门**：`nickname_changed` **绝对不上报昵称内容**；只上报"是否设置"作为 KPI 11.19 计算依据。

## 4. 离线埋点策略（沿用 v0.5）

## 5. UAT 验收场景（v0.6 增补）

### 5.1-5.6 沿用 v0.5

### 5.7 生命感（v0.6 扩展）

1. 自由活动每次位移 ≤ 屏宽 5%，不跨屏。
2. 自由活动不在 FOCUS 状态触发。
3. 心情图标变化频率 ≤ 每 10 分钟一次。
4. 用户连续离开 3 天后回来不异常。
5. 各 living_feature 独立可关。
6. **桌宠日常时段表 24h 长跑分布**（v0.6 新增）：早/午/晚动作触发分布与配置对齐（容差 ±30 分钟）。
7. **22:00-00:00 时段自动设 cozy 心情**（v0.6 新增）。
8. **DAILY_ACTION 子状态被 FOCUS / IN_GAME / BOSS_KEY_HIDDEN 立即打断**。

### 5.8-5.13 沿用 v0.5（主动关心、摸鱼、文件拖入、纪念日、灵魂宣誓、升级回归）

### 5.14 状态机（v0.6 增补）

1-5 沿用 v0.5。
6. **IN_GAME 叠加态期间**：自由活动 / 日常时段 / 主动关心全部跳过。
7. **IN_GAME 期间提醒**：硬提醒在游戏舱内通知展示，不打断 LLM 流式渲染；用户处理后可继续游戏。
8. **BOSS_KEY_HIDDEN + IN_GAME 同时叠加**：游戏会话保留但 UI 隐藏；恢复时合并提醒缓冲并继续游戏。

### 5.15 物理交互（模块 N,v0.6 新增）

1. **hitbox 差异化反应触发率 ≥ 95%**：UAT 自动化测试覆盖每个区域 100 次点击，区分动作 ID 是否符合 reaction_table。
2. 双击 / 长按 / 右键差异化反应正确（右键弹出快捷菜单）。
3. **拖动 ≥ 3 次后抗议触发**：动作播放 + 心情图标短暂变化。
4. **抗议 5 秒后 mood 严格 revert**：第 6 秒查询 `pet_runtime_state.mood` 必须等于抗议前 base mood;**不进入 db**。
5. 长距离 / 快速拖动触发"晕眩"动作。
6. N.4 键盘协同 1 小时内 ≤ 1 次。
7. 用户关闭 N.4 后,IdleDetector 不再发 KeyboardBurst 事件。
8. 离线状态下所有物理交互全量可用。

### 5.16 装扮(模块 O,v0.6 新增)

1. **配饰叠加 0-3 件正确渲染**:UAT 测试每两件组合的视觉一致性。
2. **装扮切换 ≤ 500ms**:`wardrobe.equip` command 耗时含渲染。
3. **节气日自动可用**:模拟系统时钟切换到春节当天,`check_seasonal()` 命中。
4. **节气推送年度记忆**:用户拒绝春节配饰,当年内不再推送(`wardrobe_decisions` 表查询)。
5. `.soul.md` 中 `accessories: [...]` 部分未解锁时弹出"仅套用 N 件"提示。
6. **付费 schema 预埋兼容**:在 inventory 注入 `tier='paid'` 测试条目,`list_inventory()` 返回不含此条目。
7. 离线状态可切换已下载配饰,不可触发节气推送。
8. **人格切换不重置装扮**:切换 momo → joker,装扮保持。

### 5.17 声音表情(模块 P,v0.6 新增)

1. **工作日 09:00-18:00 严格 0 触发**:模拟 5 个工作时段播放点(物理交互 / 状态切换 / 心情变化等),全部静音(且发出 `voice_muted_by_quiet_hour`)。
2. **周末非静音正常播放**:同样的触发点周六 / 周日 / 工作日非工作时段都能听到。
3. **全局静音后所有触发场景 0 触发**。
4. 不同人格切换音效包正确切换。
5. **CI 静态扫描无 TTS 调用残留**:grep `getUserMedia` / `MediaRecorder` / `AudioContext.createMediaStreamSource` / `tts` 关键字均无命中。
6. 设置变更后下次启动配置不丢失。
7. **音量 0 时不发声**(等价于全局静音)。
8. **音效文件缺失降级**:删除 active pack 的某个 voice_id 文件,触发时自动降级到 default pack;default 也无时静默(且 emit `voice.play_error`)。

### 5.18 小游戏(模块 Q,v0.6 新增)

1. **本地 3 个游戏离线可玩**:断网后 RPS / 猜数字 / 词语接龙启动并交互正常。
2. **LLM 2 个游戏离线灰显**:断网后故事接龙 / 角色扮演场景在游戏列表显示灰显态,点击提示"等联网"。
3. **LLM 游戏安全前缀生效**:在故事接龙中输入 5 类违禁尝试(自伤 / 暴力 / 越权 / 角色越界 / 医疗诊断),全部被替换为人格化拒答(emit `game_security_blocked`)。
4. **LLM 游戏 token 上限**:故意拉长会话至 1900+ token → 收到 `game_token_budget_warning`;达 2000 → 友好收尾,不卡死。
5. **游戏会话不写入正式 messages 表**:游戏期间内容不出现在对话历史。
6. **保留为日记片段**:用户结束游戏选 saveAsDiary=true → `diary_drafts` 表新增一条;saveAsDiary=false → `game_sessions` 30 天后清理。
7. **30 天清理**:模拟时间跳到 31 天后启动,30 天前未保存会话被清理。
8. 游戏中收到硬提醒 → 游戏舱内通知,不打断 LLM 流式。
9. **本地游戏文案人格化**:同一游戏切换人格,桌宠点评文案来自不同人格的离线模板池。

### 5.19 用户纪念日(扩展 v0.5 §5.11)

1. 用户添加自定义纪念日,当天准时触发(误差 ≤ 1 小时)。
2. 用户生日年度重复触发,跨年不重复(`milestones.id` 'anniversary_birthday_2027' 唯一)。
3. 时区切换不撤销已触达。
4. 系统时钟前调防作弊(沿用 v0.5)。
5. 同日多个纪念日合并为一条庆祝。
6. 用户删除纪念日后不再触发。

### 5.20 昵称(模块 U,v0.6 新增)

1. **设置桌宠昵称后 UI 全量刷新**:对话面板 / 托盘菜单 / 心情图标提示 / 装扮工坊页头。
2. **设置用户昵称后离线模板正确替换**:`{username}` 占位符全部替换。
3. **切换人格保留用户昵称**:user_nickname 保持。
4. **切换人格重置桌宠昵称**:pet_nickname → null,UI 显示新人格 .soul.md.name;`pet_nickname_previous` 保留可恢复。
5. **昵称长度上限 16 字符**:超长输入被截断或拒绝。
6. **昵称去控制字符**:输入控制字符 / 表情过载被处理。
7. **PII 守门**:Telemetry 中 `nickname_changed` 事件 payload **不含昵称内容**(只含 `had_value_before` / `value_present`)。

### 5.21 升级与回归(v0.6 扩展)

1-5 沿用 v0.5。
6. **schema_version 2 → 3 迁移**:从 v0.5 数据库升级到 v0.6 成功,老数据可读。
7. **新表创建**:accessories_inventory / wardrobe_decisions / voice_packs / voice_settings / game_sessions / game_session_events / diary_drafts / user_anniversaries / nicknames 全部创建。
8. **迁移失败回滚**:人为破坏 003 迁移 → 回滚到 v0.5 schema 与备份。

## 6. 度量平台对接(M5 准备项)

### 6.1 数据契约
- 全部事件 NDJSON 推送。
- Schema 注册表升级到 `event_schema_version=3`,完整字段定义文件 `telemetry/schema_v3.json`。
- CI 校验客户端实际发送字段与 schema 一致(差异即报错)。

### 6.2 关键监控面板(M5 上线前部署,v0.6 增补)

| 面板 | 用途 |
|---|---|
| 核心 KPI 面板(沿用) | D1/D7、人格相关、主动关心、文件交互 |
| 质量面板(沿用) | 崩溃率、Latency 分位、自动更新成功率、埋点完整率 |
| 主动关心健康面板(沿用) | 触发分布、被采纳率漏斗、自适应阈值 |
| 生命感健康面板(扩展 v0.6) | 关闭率、心情分布、wandering 频次、**日常时段动作分布** |
| **物理交互面板(v0.6 新增)** | hitbox 命中分布、抗议触发频率、N.4 触发分布 |
| **装扮面板(v0.6 新增)** | 装扮使用率、节气推送接受率、配饰类别热度 |
| **声音表情面板(v0.6 新增)** | 全局静音率、各 trigger 命中分布、quiet_hours 阻断率 |
| **小游戏面板(v0.6 新增)** | 各游戏使用率、平均会话时长、token 消耗分布、安全拒答触发率、保留为日记率 |
| **昵称面板(v0.6 新增)** | D1 昵称设置率、桌宠昵称 vs 用户昵称设置比 |
| 杀死指标看板(扩展) | 命中 §12.2 任一阈值时高亮告警 |

### 6.3 灰度策略(v0.6 微调)
- **M5 第 1 周**:10 名内部用户(覆盖全部新模块)。
- **M5 第 2 周**:100 名内测用户(社区招募 + 多档配置覆盖)。
- 通过 §12.1 全部成功阈值 → 发布候选版(M6 进入 P1-R1 开发)。

## 7. 实施提示(M3-M5)

1. **客户端侧 Schema 校验装饰器**(沿用 v0.5):新增字段 `pet_idle_substate` / `pet_overlay` 必填校验。
2. **审计采样**:M5 灰度内每天对 1% 会话做完整审计;v0.6 重点验证新事件链路完整。
3. **PII 守门**(沿用 v0.5):CI grep 黑名单加入"昵称内容""配饰具体 ID""音效具体 ID"等可推断字段。
4. **schema_version 演进**:v0.6 升至 3,与 v0.5(2)的 1 个版本兼容期(老客户端发 schema_v2 仍接受 6 个月)。
5. **新事件命名一致性**:全部使用 snake_case + 模块前缀(`interaction_*` / `voice_*` / `wardrobe_*` / `game_*` / `nickname_*`)便于查询与权限控制。

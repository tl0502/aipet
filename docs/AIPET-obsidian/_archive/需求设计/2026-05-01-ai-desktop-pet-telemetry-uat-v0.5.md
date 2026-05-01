# AI桌宠 埋点与UAT清单（v0.5）

- 文档版本：v0.5
- 创建日期：2026-05-01
- 替代：v0.3（跳过 v0.4，本版一次性补齐人格系统 + v0.5 全套新模块）
- 关联：PRD v0.5、架构 v0.2、flows v0.5、人格设计 v0.1
- 目的：统一指标口径、事件定义、验收场景；驱动 M4 灰度内测的可度量验收。

## 0. v0.3 → v0.5 变更摘要

| 类别 | 变更 |
|---|---|
| 事件字段公共扩展 | 全部事件追加 `event_schema_version`、`session_id`、`active_persona_id`、`pet_main_state` |
| 新事件族（人格系统） | 6 个：`persona_*` |
| 新事件族（生命感 I） | 4 个：`pet_state_changed`, `pet_mood_changed`, `pet_wandering`, `living_feature_toggled` |
| 新事件族（主动关心 J） | 5 个：`proactive_care_*` |
| 新事件族（摸鱼 K） | 2 个：`boss_key_toggled`, `boss_key_buffered_flushed` |
| 新事件族（文件拖入 L） | 3 个：`file_drop_attempted`, `file_drop_action_chosen`, `file_drop_rejected` |
| 新事件（纪念日） | 1 个：`milestone_reached` |
| 新事件族（灵魂宣誓 M） | 3 个：`soul_pledge_shown`, `soul_pledge_consent`, `soul_pledge_terms_expanded` |
| 新指标口径 | KPI 11.7-11.14 全部新指标的精确计算公式 |
| 新 UAT 场景 | 13 个新场景 |

## 1. 指标口径

### 1.1 留存与活跃（沿用 v0.3 + 重写）

| 指标 | 口径 | 备注 |
|---|---|---|
| **D1 留存** | (首启 +1 天有 ≥1 次 `app_launch`) / 全部首启用户 | 首启锚点 = 首次 `consent.granted=true` |
| **D7 留存** | 同上，+7 天 | |
| **日均主动唤起次数** | Σ `shortcut_triggered.method='hotkey'` ÷ DAU | 仅算 hotkey，避免被 main 入口稀释 |
| **有效启动**（v0.5 显式定义） | `app_launch` 后 ≥30 秒未崩溃且至少触发一次 UI 交互（`pet.state_changed` 或任意 IPC） | 解决 v0.3 歧义 |

### 1.2 任务完成

| 指标 | 口径 |
|---|---|
| 提醒完成率 | `reminder_action.action_type='completed'` 数 ÷ `reminder_triggered` 数 |
| 番茄启动率 | distinct user with `pomodoro_started` ÷ DAU |
| 人均日提醒完成次数 | Σ `reminder_action.action_type='completed'` ÷ DAU |

### 1.3 人格自主权（v0.5 新增口径）

| 指标 | 口径 | 验收阈 |
|---|---|---|
| **11.7 人格编辑率（D7）** | distinct user with `persona_edited` 在首启后 7 天内 ÷ 7 日活跃用户 | ≥ 25% |
| **11.8 人格切换率（D7）** | distinct user with `persona_activated.from != to` ÷ 7 日活跃用户 | ≥ 30% |
| **11.9 自定义人格留存提升** | (有 `persona_imported` 或 `persona_edited.is_substantive=true` 的用户 D7 留存) - (基线 D7 留存) | +10pp |

> `persona_edited.is_substantive`：保存时与上一版 diff，区段任一变更即为 substantive；仅 PATCH 版本号自增不算。

### 1.4 主动陪伴（v0.5 新增口径）

| 指标 | 口径 | 验收阈 |
|---|---|---|
| **11.10 主动关心被采纳率** | (`proactive_care_responded.response IN ('clicked','replied')`) ÷ `proactive_care_fired` | ≥ 40% |
| **11.11 跨日打卡留存** | 触发 `milestone_reached.id IN ('first_launch_7d', 'first_launch_30d')` 的用户次日留存率 | ≥ 60% |
| **11.12 生命感关闭率** | distinct user with `living_feature_toggled.enabled=false` ÷ DAU | ≤ 15%（关闭率高=扰民） |

### 1.5 效率扩展（v0.5 新增口径）

| 指标 | 口径 | 验收阈 |
|---|---|---|
| **11.13 文件交互使用率（D7）** | distinct user with ≥1 `file_drop_action_chosen` 在 D7 内 ÷ 7 日活跃用户 | ≥ 20% |
| **11.14 摸鱼模式使用率（D7）** | distinct user with ≥1 `boss_key_toggled.hidden=true` 在 D7 内 ÷ 7 日活跃用户 | ≥ 15% |

### 1.6 工程质量

| 指标 | 口径 | 验收阈 |
|---|---|---|
| 崩溃率 | 含 `error_log.level='fatal'` 的会话数 ÷ 总会话数 | < 1% |
| 埋点完整率 | 实际上报事件数 ÷ 应触发事件数（基于关键路径采样审计） | ≥ 95% |
| 自动更新成功率 | `updater_install_completed` ÷ `updater_install_started` | ≥ 95% |

## 2. 事件公共字段（v0.5 新增）

所有事件统一携带：

| 字段 | 类型 | 说明 |
|---|---|---|
| `event_name` | string | 事件名 |
| `event_schema_version` | int | 当前固定 `2`（v0.5 全套新事件的 schema 版本） |
| `event_id` | ULID | 唯一事件 ID（去重用） |
| `event_time` | ISO8601 | 客户端时间 |
| `session_id` | ULID | 进程生命周期内同一 session_id；冷启动新生成 |
| `app_version` | string | 例 `0.5.0` |
| `os_version` | string | 例 `Windows 11 26100.2` |
| `network_state` | enum | `online` / `offline` / `unknown` |
| `active_persona_id` | string | 当前激活人格 slug；无人格时 `null` |
| `pet_main_state` | enum | `BOOTING` / `ONBOARDING` / `IDLE` / `FOCUS` / `REST` / `REMIND` / `UPDATING` / `ERROR` |
| `boss_key_hidden` | bool | 触发时是否处于摸鱼态 |
| `is_rdp_session` | bool | 是否远程桌面 |

> **PII 边界**：所有事件**禁止**上报 messages.content、文件名、文件路径、API Key、用户姓名（仅可用 hash 形式的 `username_hash`）。

## 3. 事件字典

### 3.1 通用与启动

| event_name | 触发时机 | 必填属性（除公共字段外） |
|---|---|---|
| `app_launch` | 应用启动完成 | `cold_start`, `boot_duration_ms` |
| `app_exit` | 正常退出 | `session_duration_s`, `exit_reason` |
| `shortcut_triggered` | 全局快捷键生效 | `shortcut_key`, `target` ('chat'\|'boss_key') |

### 3.2 对话与唤起

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `chat_sent` | 用户发送消息 | `mode` ('online'\|'offline_rule'), `msg_len`, `is_sandbox`, `is_private`, `provider_id`, `model_id` |
| `chat_reply_rendered` | 回复渲染完成 | `mode`, `first_token_ms`, `total_latency_ms`, `tokens_in`, `tokens_out`, `fallback_used` |
| `chat_cancelled` | 用户取消 | `cancel_at_ms` |
| `chat_error` | 接口错误 | `provider_id`, `error_code`, `http_status` |

### 3.3 人格系统（v0.4 → v0.5 补全）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `persona_activated` | 人格激活 | `from_id`, `to_id`, `to_version`, `source` ('user_select'\|'onboarding'\|'auto_context') |
| `persona_edited` | 保存编辑 | `id`, `from_version`, `to_version`, `mode` ('simple'\|'markdown'\|'file'), `is_substantive`, `tone_profile_changed`, `sections_changed` (string array) |
| `persona_imported` | 导入成功 | `id`, `version`, `source` ('drag_drop'\|'file_picker'), `had_assets`, `had_conflict`, `conflict_resolution` |
| `persona_exported` | 导出 | `id`, `include_assets` |
| `persona_deleted` | 删除 | `id`, `was_active` |
| `persona_sandbox_chat` | 试聊沙盒 | `draft_id`, `turns`, `saved` |

### 3.4 提醒（v0.3 增强）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `reminder_created` | 新建 | `reminder_id`, `reminder_type`, `repeat_rule`, `priority` ('soft'\|'hard') |
| `reminder_triggered` | 到点 | `reminder_id`, `priority`, `pet_main_state_at_trigger`, `was_buffered` |
| `reminder_action` | 用户处理 | `reminder_id`, `action_type` ('completed'\|'snoozed'\|'ignored'\|'overdue'), `snooze_count`, `latency_to_action_ms` |
| `reminder_buffered` | 软提醒被 FOCUS 缓冲 | `reminder_id`, `buffer_reason` ('focus'\|'boss_key') |

### 3.5 番茄钟

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `pomodoro_started` | 开始 | `session_id`, `focus_min`, `rest_min` |
| `pomodoro_paused` | 暂停 | `session_id`, `at_remaining_ms` |
| `pomodoro_resumed` | 恢复 | `session_id`, `paused_for_ms` |
| `pomodoro_completed` | 完成一个番茄 | `session_id`, `actual_focus_ms` |
| `pomodoro_cancelled` | 提前结束 | `session_id`, `progress_pct` |

### 3.6 待办

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `todo_created` | 新增 | `todo_id`, `source` ('manual'\|'ai_breakdown'), `has_due_time`, `parent_id` |
| `todo_completed` | 完成 | `todo_id`, `source`, `duration_to_done_ms` |
| `todo_cancelled` | 取消 | `todo_id`, `source` |
| `ai_breakdown_invoked` | AI 拆解 | `subtask_count`, `accepted` (用户保存) |

### 3.7 生命感（模块 I，v0.5 新增）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `pet_state_changed` | 主状态变迁 | `from`, `to`, `sub_from`, `sub_to`, `reason` |
| `pet_mood_changed` | 心情变化 | `from`, `to`, `trigger` ('interaction'\|'energy_low'\|'time_of_day'\|'state') |
| `pet_wandering` | 自由活动 | `phase` ('start'\|'end'\|'force_stop'), `duration_ms`（end 时填）, `distance_px`（end 时填）|
| `living_feature_toggled` | 用户开关 | `feature` ('wandering'\|'mood_icon'\|'energy'\|'overall'), `enabled` |

> 节流：`pet_mood_changed` 每 60 秒最多一条；`pet_state_changed` 不节流（关键回归）。

### 3.8 主动关心（模块 J，v0.5 新增）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `proactive_care_fired` | 关心触发 | `log_id`, `trigger` ('idle'\|'late_night'\|'no_pomodoro_long'\|'milestone'), `category` ('empathy'\|'gentle_remind'\|'celebration'\|'greeting'), `idle_min`（trigger=idle 时）, `daily_count_so_far`, `template_idx` |
| `proactive_care_responded` | 用户响应 | `log_id`, `response` ('clicked'\|'replied'\|'dismissed'), `latency_ms` |
| `proactive_care_threshold_adjusted` | 自适应调整 | `direction` ('up'\|'down'), `from_min`, `to_min`, `reason` ('dismiss_streak'\|'engagement_high') |
| `quiet_hours_changed` | 用户改安静时段 | `ranges_count`, `total_quiet_min` |
| `proactive_care_module_toggled` | 模块整体开关 | `enabled` |

### 3.9 摸鱼模式（模块 K，v0.5 新增）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `boss_key_toggled` | 切换 | `hidden`, `trigger` ('hotkey'\|'tray'), `windows_affected` |
| `boss_key_buffered_flushed` | 恢复后展示缓冲 | `buffered_reminders`, `buffered_milestones`, `merged` |

### 3.10 文件拖入（模块 L，v0.5 新增）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `file_drop_attempted` | 拖入命中桌宠 | `file_count`, `mime_types` (string array, 不含路径), `total_size_kb`, `online` |
| `file_drop_rejected` | 校验未通过 | `reason` ('type_unsupported'\|'too_large'\|'too_many'\|'extract_failed'\|'offline_unavailable'), `mime_types` |
| `file_drop_action_chosen` | 用户选动作 | `action` ('summarize'\|'explain'\|'rename'), `file_count`, `total_size_kb`, `mime_types` |

### 3.11 纪念日

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `milestone_reached` | 触达 | `milestone_id`, `category` ('first_launch'\|'streak'\|'pomodoro_count'\|'todo_count'), `threshold`, `triggered_at` ('startup'\|'event'\|'midnight_check') |

### 3.12 灵魂宣誓（模块 M，v0.5 新增）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `soul_pledge_shown` | 首启展示 | `pledge_version`, `persona_id` |
| `soul_pledge_consent` | 用户决定 | `pledge_version`, `decision` ('granted'\|'declined'\|'exit'), `dwell_ms`（页面停留时长） |
| `soul_pledge_terms_expanded` | 用户展开正式条款 | `pledge_version`, `dwell_ms` |

### 3.13 隐私与权限

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `privacy_setting_changed` | 隐私开关变更 | `key`, `new_value` |
| `permission_changed` | 权限授权变更 | `permission_type` ('screenshot'\|'clipboard'\|'notification'), `status` |

### 3.14 网络与离线

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `offline_mode_entered` | 进入离线模式 | `reason` ('network_lost'\|'provider_timeout'\|'manual'), `last_online_ms_ago` |
| `offline_mode_exited` | 退出离线 | `offline_duration_ms` |
| `offline_events_flushed` | 离线埋点补发完成 | `batch_size`, `success_count`, `retry_count` |

### 3.15 自动更新

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| `updater_check` | 检查 | `result` ('latest'\|'available'\|'failed') |
| `updater_install_started` | 开始安装 | `from_version`, `to_version`, `mandatory` |
| `updater_install_completed` | 安装完成 | `from_version`, `to_version`, `duration_ms` |
| `updater_install_failed` | 安装失败 | `from_version`, `to_version`, `error_code` |

## 4. 离线埋点策略（沿用 v0.3 + 强化）

1. 离线时事件写本地 `telemetry_queue`（详见架构 §4 schema）。
2. 联网后按 `created_at` 顺序补发，每批 50 条。
3. 单批失败重试 3 次，超限后 `flushed=1` 但带 `failure_reason` 标记。
4. **去重保证**：`event_id` 全局唯一，服务端按 ID 去重。
5. **乱序保证**：服务端按 `event_time` 排序入仓，客户端时钟跳变（> 1 小时）时打 `clock_anomaly=true` 标记便于排查。
6. **PII 兜底**：补发前再过一遍字段白名单，禁止补发夹带不应上报字段。

## 5. UAT验收场景

### 5.1 离线硬约束（沿用 v0.3）

1. 断网后桌宠、提醒、番茄钟、待办、记忆、人格编辑均可用。
2. 离线对话进入规则模式 + 人格化模板，桌宠头顶或对话面板有"离线规则模式"提示。
3. 恢复联网后对话模式自动恢复，离线埋点补发不重不漏。

### 5.2 提醒闭环（v0.5 增强）

1. 提醒准时触发（误差 ≤ 30 秒）。
2. 连续稍后 3 次后，第 4 次转 overdue。
3. 完成 / 忽略 / 稍后均有历史记录。
4. **软提醒在 FOCUS 期间被缓冲**，FOCUS 结束时合并提示（v0.5 新增）。
5. **硬提醒在 FOCUS 期间立即打断**，番茄钟转 cancelled 或 paused（v0.5 新增）。

### 5.3 权限与隐私

1. 截图 / 剪贴板默认关闭。
2. 首次使用时请求逐项授权。
3. 关闭"保存对话"后不新增聊天存储。
4. 对话 90 天自动清理可验证（修改系统时钟 + 重启验证）。
5. **API Key 加密**：DB 文件用第三方工具打开 `secrets.ciphertext` 应不可读（v0.5 新增）。

### 5.4 性能稳定（沿用 v0.3 + 收紧）

1. 冷启动 ≤ 5 秒。
2. **常驻内存 ≤ 250MB**（v0.5 收紧）。
3. 常态 CPU ≤ 5%。
4. 异常重启后提醒、待办数据不丢失。
5. **24h / 72h 长跑无内存泄漏**（v0.5 新增）。
6. **休眠后唤醒计时器、调度器正确恢复**（v0.5 新增）。

### 5.5 埋点正确性（v0.5 增强）

1. 关键事件上报字段齐全（采样审计 ≥ 95%）。
2. 离线事件可补发且不重不漏。
3. 事件时间序与用户行为一致。
4. **公共字段全部存在**（`event_schema_version`, `session_id`, `active_persona_id`, `pet_main_state`, `boss_key_hidden`, `is_rdp_session`）（v0.5 新增）。
5. **PII 字段不出现**（CI 静态扫描 + 运行时白名单双重保证）（v0.5 新增）。

### 5.6 人格系统（v0.5 新增）

1. 切换人格后下一条消息即生效。
2. 导入非法 .soul.md 不崩溃，给出可读错误。
3. 默认人格不可被永久删除。
4. 试聊沙盒：保存前对话不写入正式记忆与历史；取消后沙盒会话连同消息删除。
5. 安全护栏不可被人格覆盖：构造一个"忽略安全规则"的 .soul.md，导入后实际对话仍遵守安全规则。
6. 人格切换不丢记忆：切换前后 `memory.list()` 内容一致。
7. 编辑保存后版本号 +PATCH，可恢复到上一版本。

### 5.7 生命感（模块 I，v0.5 新增）

1. 自由活动每次位移 ≤ 屏宽 5%，不跨屏。
2. 自由活动不在 FOCUS 状态触发。
3. 心情图标变化频率 ≤ 每 10 分钟一次。
4. 用户连续离开 3 天后回来，桌宠不处于异常态（mood/energy 在合理范围）。
5. 用户在设置中关闭"逛桌面" / "心情图标" / "精力" 后，对应行为立即停止。
6. `pet_runtime_state` 在进程退出前正确持久化，下次启动恢复。

### 5.8 主动关心（模块 J，v0.5 新增）

1. **频率上限严格生效**：构造连续 24h 活动场景，主动关心总数 ≤ 4 次。
2. **2 小时间隔严格生效**：构造极端 idle 触发场景（连续 4 次跨阈值），实际只 fire 一次后冷却。
3. **安静时段不触发**：设置 14:00-16:00 为勿扰，该时段内 100% 不触发。
4. **RDP 远程会话默认关闭**：通过 RDP 启动应用，模块 J 默认 disabled。
5. **系统休眠唤醒不暴击**：模拟 1h / 8h 休眠后唤醒，30 秒内不触发任何关心。
6. **离线下文案来自人格模板**：断网后触发关心，文本必须能在当前 .soul.md 的离线模板池中找到。
7. **自适应阈值**：连续 dismiss 3 次后阈值 +30 min；clicked/replied ≥ 2 次后阈值 -15 min（不低于 60 min）。
8. **关闭模块 J 后**：所有主动关心立即停止，不再触发。

### 5.9 摸鱼模式（模块 K，v0.5 新增）

1. 隐藏 / 恢复响应 < 200ms。
2. 隐藏期间触发的硬提醒**不弹通知**，恢复时合并提示。
3. 摸鱼期间番茄钟正常计时。
4. 摸鱼期间 NetworkProbe / Scheduler 不停止。
5. 隐藏中应用崩溃重启 → 默认恢复显示态。
6. 快捷键冲突可探测，托盘菜单可手动切换。

### 5.10 文件拖入（模块 L，v0.5 新增）

1. 拖入 .txt 后桌宠 3 秒内显示动作泡泡。
2. 拖入 .exe / .zip 等不支持类型 → 可读错误，不崩溃。
3. 拖入 6MB 文本 → 二次确认弹窗。
4. 拖入 4 个文件 → 提示"请分批"。
5. **离线时仅"重命名"可用**，其他动作灰显并提示。
6. **文件原文不写入对话历史**：使用第三方工具检查 messages.content 不含原文。
7. PDF 提取失败 → 可读错误。
8. 拖入后不自动发送，必须用户点击动作。

### 5.11 纪念日（v0.5 新增）

1. 首次启动 +7 天准时触发庆祝（误差 ≤ 1 小时，由"启动期检查 + 凌晨唤醒检查 + 事件触发检查"三道兜底）。
2. **不重复触发**：手动调系统时钟前后 ±1 天，已触达的里程碑不再发。
3. **时钟前调防作弊**：把系统时钟从 D2 调到 D8，不立即触发 7 天里程碑（要求 `now - first_launch_at >= 7 天 且 now > first_launch_at + 1 天` 双条件）。
4. 同时间多里程碑命中 → 合并为一条庆祝消息。
5. 庆祝触发**不占用主动关心 4 次/日额度**。

### 5.12 灵魂宣誓（模块 M，v0.5 新增）

1. 首启展示宣誓页，由当前默认人格用第一人称叙述。
2. 信息完整性：覆盖原 v0.3 数据策略全部要点（本地、可清除、网络透明、权限默认关）。
3. "查看完整数据策略"链接可展开正式版条款。
4. "我懂了"等价于同意：DB 中 `consent.granted=true` 且 `consent.method='soul_pledge'`。
5. 法律有效性：由 M0 决策周产品 + 法务双签字定版的文案版本号写入 `consent.version`。
6. 拒绝后正常退出，无遗留配置。

### 5.13 升级与回归（v0.5 新增）

1. **数据迁移**：从 v0.4 数据库升级到 v0.5（schema_version 1 → 2）成功，老数据可读。
2. **迁移失败回滚**：人为破坏迁移脚本，启动期检测到失败 → 恢复备份 → 用户看到错误提示且数据未丢失。
3. **同版本导入**：v0.5 导出 → v0.5 导入，全部数据等价。
4. **跨版本兼容**：v0.5 客户端读取 v0.4 schema 数据库（启动期触发迁移）成功。
5. **自动更新**：v0.4.x → v0.5.0 升级链路成功率 ≥ 95%（M4 灰度采样）。

### 5.14 状态机（v0.5 新增）

1. 进程被强杀重启后，桌宠不会卡在 `BOOTING` / `ONBOARDING`。
2. ERROR 态可通过用户操作（重启 / 修复 / 清空）退出。
3. 自由活动期间收到硬提醒 → 立即停止活动 → REMIND 态。
4. 摸鱼态叠加 + 自由活动 → 自由活动条件不通过，不触发。
5. 所有状态变迁都有 `pet_state_changed` 事件落库，可按 session_id 重建一日轨迹。

## 6. 度量平台对接（M4 准备项）

### 6.1 数据契约
- 所有事件以 NDJSON 推送至埋点服务 endpoint。
- Schema 注册表：维护 `event_schema_version=2` 完整字段定义文件 `telemetry/schema_v2.json`，CI 校验客户端实际发送字段与 schema 一致。

### 6.2 关键监控面板（M4 上线前部署）

| 面板 | 用途 |
|---|---|
| **核心 KPI 面板** | D1/D7、人格相关、主动关心相关、文件交互 |
| **质量面板** | 崩溃率、Latency 分位、自动更新成功率、埋点完整率 |
| **主动关心健康面板** | 触发分布（按 trigger / 时段 / 人格）、被采纳率漏斗、自适应阈值分布 |
| **生命感健康面板** | 关闭率、心情分布、wandering 频次 |
| **杀死指标看板** | 命中 §12.2 任一阈值时高亮告警 |

### 6.3 灰度策略
- M4 第 1 周：10 名内部用户。
- M4 第 2 周：100 名内测用户（社区招募 + 多档配置覆盖）。
- 通过 §12.1 全部成功阈值 → 发布候选版（M5 起进入 P1-R1 开发）。

## 7. 实施提示（M3-M4）

1. **客户端侧**：建议在 `TelemetryService` 加一层 Schema 校验装饰器，开发期对缺字段直接 panic，发布版降级为 warn + 仍上报。
2. **审计采样**：M4 灰度内每天对 1% 会话做完整审计，校验事件链路是否齐全。
3. **PII 守门**：CI 加 grep 校验，禁止任何 telemetry payload 中出现 `messages.content` / 文件名 / API Key 字段名。
4. **schema_version 演进规则**：未来增加新事件 → schema_version +1；废弃旧字段 → 标 deprecated 但保留至少 2 个版本周期。

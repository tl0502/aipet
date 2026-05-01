# AI桌宠 埋点与UAT清单（v0.3）

- 文档版本：v0.3
- 日期：2026-04-30
- 目的：统一指标口径、事件定义、验收场景。

## 1. 指标口径

1. D1留存：用户首次使用后第1天任意有效启动。
2. D7留存：用户首次使用后第7天任意有效启动。
3. 日均主动唤起次数：用户主动触发快捷键/点击输入入口总次数 ÷ DAU。
4. 提醒完成率：完成提醒数 ÷ 总触发提醒数。
5. 番茄启动率：启动番茄钟用户数 ÷ DAU。

## 2. 事件字典（P0）

| event_name | 触发时机 | 必填属性 |
|---|---|---|
| app_launch | 应用启动完成 | app_version, os_version, network_state |
| shortcut_triggered | 全局快捷键生效 | shortcut_key, network_state |
| chat_sent | 用户发送消息 | mode, msg_len, network_state |
| chat_reply_rendered | 回复渲染完成 | mode, latency_ms, fallback_used |
| reminder_created | 新建提醒 | reminder_type, repeat_rule |
| reminder_triggered | 到点触发 | reminder_id, snooze_count |
| reminder_action | 用户处理提醒 | action_type, snooze_count |
| pomodoro_started | 开始番茄钟 | focus_min, rest_min |
| pomodoro_completed | 完成一个番茄 | duration_ms |
| todo_created | 新增待办 | source, has_due_time |
| todo_completed | 完成待办 | source, duration_to_done_ms |
| privacy_setting_changed | 隐私开关变更 | key, new_value |
| permission_changed | 权限授权变更 | permission_type, status |
| offline_mode_entered | 进入离线模式 | reason |
| offline_events_flushed | 离线埋点补发完成 | batch_size, retry_count |

## 3. 离线埋点策略

1. 离线时埋点写本地队列。
2. 联网后按时间顺序补发。
3. 单批次失败可重试3次，超限后保留并标注失败原因。

## 4. UAT验收场景

### 4.1 离线硬约束

1. 断网后仍可使用桌宠、提醒、番茄钟、待办、记忆。
2. 离线对话进入规则回复并有状态提示。
3. 恢复联网后对话模式自动恢复在线。

### 4.2 提醒闭环

1. 提醒准时触发（误差 <= 30秒）。
2. 连续稍后3次后，第4次转未完成待复盘。
3. 完成/忽略/稍后均有历史记录。

### 4.3 权限与隐私

1. 截图/剪贴板默认关闭。
2. 首次使用时请求逐项授权。
3. 关闭“保存对话”后不新增聊天存储。
4. 对话90天自动清理可验证。

### 4.4 性能稳定

1. 冷启动 <= 5秒。
2. 常驻内存 <= 500MB（不含外部模型）。
3. 常态CPU <= 5%。
4. 异常重启后提醒与待办数据不丢失。

### 4.5 埋点正确性

1. 关键事件上报字段齐全。
2. 离线事件可补发且不重不漏。
3. 事件时间序与用户行为一致。

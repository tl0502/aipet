# AI桌宠项目竞品调研记录（初版）

- 记录日期：2026-04-30
- 项目：AI桌宠
- 目的：对“桌宠 + AI陪伴 + AI效率助手”赛道做初步竞品调查，补全产品认知，形成可执行定位输入。

## 1. 调研范围与方法

### 1.1 范围
- 桌面宠物/桌面角色（Desktop Pet / Desktop Mascot）
- AI陪伴（Companion）
- 桌面AI助手（效率与系统层入口）

### 1.2 方法
- 以官网、应用商店、Steam、GitHub、官方公告为主
- 优先使用可验证的一手信息：平台、功能、上线状态、定价、更新节奏
- 时间口径：截至 2026-04-30（美国东部时间）

## 2. 一页结论（Summary）

1. 桌宠形态已被验证有用户需求与付费空间（Steam 生态显著）。
2. AI桌宠仍在早期，当前主流差异点集中在“本地隐私 + 屏幕感知 + 个性化记忆”。
3. 纯“聊天型陪伴”竞争非常激烈，成熟玩家（Replika / Character.AI / Nomi）已建立规模和安全机制。
4. 桌面效率助手（Copilot / Amazon Quick）快速进入“看屏 + 执行”场景，挤压低价值能力。
5. AI桌宠机会在于：用“宠物在场感”做拉新，用“可执行任务闭环”做留存。

## 3. 竞品分层地图

## 3.1 A层：直接竞品（AI桌宠）

| 产品 | 定位 | 平台 | 关键特征 | 商业化/状态 |
|---|---|---|---|---|
| Clawster | 桌面AI宠物助手 | macOS | 快捷唤起、截图问答、屏幕感知、本地网关(OpenClaw) | 早期产品，强调本地隐私 |
| PetClaw | 本地优先AI宠物助手 | Windows / macOS | 本地运行、记忆、隐私导向 | 订阅 + credits（官网价格页） |
| Your Friendly AI | 可对接多模型桌宠 | Windows（官方页） | 支持 Ollama/OpenAI/Claude，桌宠形态 | 早期阶段 |
| YCamie (Shimeji AI) | 宠物生成工具 | 桌面端场景 | AI生成桌宠角色（更偏内容生成） | 订阅额度模式 |
| PopClaw | 硬件化AI宠物终端 | 独立设备 | 本地、可离线、开源导向 | Kickstarter 预告 |

## 3.2 B层：强替代（AI陪伴）

| 产品 | 平台 | 优势 | 对AI桌宠的威胁 |
|---|---|---|---|
| Replika | iOS/Android/Web/VR | 陪伴关系与长期记忆产品化成熟 | 抢占情感陪伴心智与付费预算 |
| Character.AI | Web/iOS/Android | 角色生态、用户规模、安全分层机制 | 抢占泛人设聊天与UGC角色时长 |
| Nomi | iOS/Android/Web | 高沉浸关系陪伴、口碑较好 | 抢占高粘性陪伴需求 |

## 3.3 C层：生态挤压者（桌面AI助手）

| 产品 | 近期动态 | 对AI桌宠影响 |
|---|---|---|
| Microsoft Copilot | 2025-07-15 开始推送 Vision Desktop Share（Insider） | 系统级入口 + 看屏能力，挤压“问答型”价值 |
| Amazon Quick | 2026-04-28 公告 Windows/macOS 预览 | 抢占“个人工作流助理”定位 |

## 4. 关键观察（补全认知）

## 4.1 用户并不只要“可爱”，更要“有用”
- 桌宠外形是点击与下载驱动力，但留存依赖“任务完成率”和“打扰控制”。
- 仅聊天或仅动画的产品会快速同质化。

## 4.2 本地隐私从“卖点”变为“门槛”
- 多个AI桌宠竞品都在强调 local-first。
- 对应到产品设计：默认本地存储记忆、透明可清除、可导出。

## 4.3 安全与边界正在成为刚性要求
- 头部陪伴产品开始公开青少年与家庭侧控制机制。
- 若目标用户覆盖学生群体，必须前置内容边界和风险升级机制。

## 4.4 商业化更像“内容+能力”双轮
- 仅按 token 或次数收费体验脆弱。
- 可持续方案更可能是：免费底座 + 角色/皮肤/人格包 + 高级能力包。

## 5. 对AI桌宠项目的定位启发

## 5.1 建议定位
- 一句话定位：`一个会在桌面“陪你完成事情”的AI桌宠`。
- 核心主张：`陪伴感` + `执行力`，而非纯聊天。

## 5.2 建议优先场景（MVP）
1. 快捷问答与截图理解（用户“看不懂/来不及”场景）。
2. 轻任务闭环（提醒、待办拆分、番茄钟、复盘）。
3. 个性化记忆（称呼、偏好、固定作息、常用模板）。

## 5.3 应避免的方向
1. 先做复杂3D大世界，弱化核心任务能力。
2. 只做聊天壳子，不做系统级快捷动作。
3. 记忆黑盒化，不提供可视化与可控删除。

## 6. 风险清单（早期）

1. 内容安全风险：涉及情感依赖、极端情绪语境时的输出边界。
2. 合规与年龄风险：青少年使用场景下的限制策略。
3. 隐私风险：屏幕感知、剪贴板读取、文件访问的权限透明度。
4. 产品风险：陪伴与效率权重失衡导致留存不稳定。

## 7. 下一步建议（用于PRD输入）

1. 先完成`竞品矩阵v1`：功能、定价、平台、差异点、风险策略。
2. 输出`定位声明v1`：目标用户、核心场景、成功指标（7日留存/任务完成率）。
3. 设计`MVP功能边界`：必须做、可延后、明确不做。
4. 产出`隐私与安全策略草案`：默认本地存储、记忆可视化、敏感场景响应策略。

## 8. 调研来源（Links）

- Desktop Mate (Steam): https://store.steampowered.com/app/3301060/Desktop_Mate/
- Shijima / Desktop Mate site: https://getshijima.app/
- Clawster: https://clawster.pet/
- OpenClaw (GitHub): https://github.com/openclaw/openclaw
- PetClaw: https://petclaw.ai/
- PetClaw Pricing: https://petclaw.ai/pricing
- Your Friendly AI: https://www.yourfriendly.ai/
- YCamie / Shimeji AI: https://www.shimeji.ai/
- PopClaw: https://www.popclaw.ai/
- Replika Help (platforms): https://help.replika.com/hc/en-us/articles/115001094491-What-platforms-and-devices-are-supported
- Replika Help (chat history): https://help.replika.com/hc/en-us/articles/4411154990605-Is-the-chat-history-infinite
- Replika Terms: https://replika.ai/replika/docs/embed/terms
- Replika iOS: https://apps.apple.com/us/app/replika-ai-friend/id1158555867
- Character.AI Safety: https://policies.character.ai/safety
- Character.AI Teen Safety: https://policies.character.ai/safety/teen-safety
- Character.AI Parental Insights: https://support.character.ai/hc/en-us/articles/34659826266267-Parental-Insights
- Nomi iOS: https://apps.apple.com/us/app/nomi-ai-companion-with-a-soul/id6450270929
- Nomi: https://nomi.ai/
- Microsoft Copilot (2025-04-04): https://blogs.microsoft.com/blog/2025/04/04/your-ai-companion/
- Copilot Vision Desktop Share (2025-07-15): https://blogs.windows.com/windows-insider/2025/07/15/copilot-on-windows-vision-desktop-share-begins-rolling-out-to-windows-insiders/
- Amazon Quick (2026-04-28): https://aws.amazon.com/about-aws/whats-new/2026/04/amazon-quick-macos-windows-preview/
- Nature article: https://www.nature.com/articles/s42256-025-01093-9
- Axios (2025-07-16): https://www.axios.com/2025/07/16/ai-bot-companions-teens-common-sense-media
- AP News (lawsuits context): https://apnews.com/article/ai-chatbot-lawsuits-character-google-fbca4e105b0adc5f3e5ea096851437de
- European Parliament Q&A: https://www.europarl.europa.eu/doceo/document/E-10-2026-000256_EN.html

## 9. 备注

- 本文为“初步调研记录”，用于产品方向判断与MVP定义输入。
- 部分商店评分、评论数、DLC数量为调研当日观察值，会随时间变化。

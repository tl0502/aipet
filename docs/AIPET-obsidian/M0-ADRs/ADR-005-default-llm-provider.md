# ADR-005: 默认 LLM Provider

- **状态**: Accepted
- **决策日期**: 2026-05-01
- **Owner**: M0 决策周(产品 + 后端)
- **Reviewers**: M0 决策周复审通过
- **目标完成**: M0 决策周内(W0)
- **影响范围**: B 对话 / Q LLM 游戏 / 整体使用成本

## 背景

PRD v0.6 锁定:
- LLM Provider 抽象:OpenAI 兼容(覆盖 OpenAI / DeepSeek / Moonshot / 本地 OpenAI 兼容服务等)。
- 用户配置 `provider_id + base_url + model + api_key`。

需要决定:
1. **首启 Onboarding** 中默认指引哪家 Provider?
2. 是否提供"零配置体验"(内置某个免费/试用额度)?
3. 国内市场的合规与可达性。

这个 ADR 不锁定**用户必须用谁**,只锁定**默认引导路径**和兼容矩阵。

## 备选方案

### 选项 A:**"零默认 + 引导链接"**(零配置不做)

Onboarding Step 6(首次唤起对话)前提示:
> "想跟我聊天吗?需要先在设置里填一个 AI 模型的 API Key。常用选项:OpenAI / DeepSeek / Moonshot / 通义千问 (其他 OpenAI 兼容服务亦可)"

每家提供"如何申请 + 在哪里填" 的引导链接。

**优点**
- 用户 100% 自主选,我们零账单负担
- 国内/海外用户都能找到合适方案
- 与 local-first 立场一致

**缺点**
- 首启转化率受影响(用户嫌麻烦关掉)
- D1 留存压力(KPI 11.1)

### 选项 B:**"默认指引到一家 + 国内备用"**

Onboarding 强调一家(如 DeepSeek 或 Moonshot,海外则 OpenAI)+ 给出 1-2 家备用。

**优点**
- 减少选择障碍
- 可对默认家做更好的"帮你创建账号"教程

**缺点**
- 偏向某家可能引发其他厂商不满 / 渠道关系问题
- 默认家的可用性变化(限速、涨价、停服)会影响用户

### 选项 C:**"内置代理 + 免费试用"**(我们承担初始 token)

我们运营一个代理服务,首次启动给每个用户 N 万 token 免费额度,之后引导用户切换到自己的 Key。

**优点**
- D1 转化率最佳
- 完美零配置体验

**缺点**
- **运营成本不可控**(N 万用户 × 几万 token = 数千美元/月)
- 引入服务端架构,违反 local-first 极致主张
- 安全:代理服务的合规、PII、内容审核都需要做
- 不在 MVP 范围内的工程量

## 我的倾向

**🌟 倾向选项 A**(零默认 + 引导链接),关键理由:
1. 与 PRD v0.6 §3 的"local-first"和"用户主权"高度一致。
2. 不引入服务端运营成本与合规负担。
3. 国内市场:DeepSeek / Moonshot / 通义千问 都有免费试用,用户实际开通门槛不高。

**配套提升 D1 转化率的非默认 Provider 措施**:
- Onboarding Step 6 改为"先体验 5 个本地能力(提醒 / 番茄 / 待办 / 摸鱼 / 物理交互)" → 不要求填 API Key。
- 用户尝试与桌宠对话时再弹出"还差最后一步"引导。
- Step 6 是可选(沿用 v0.5 设计)。

**兼容矩阵建议**(按优先级):
1. **OpenAI 兼容协议**(/v1/chat/completions, stream=true): 100% 必须支持
2. **Anthropic 协议**(messages API): P1-R1 增加(很多用户会用 Claude)
3. **Gemini 协议**: P1-R2 评估
4. **本地 Ollama**: P1-R3(支持 Ollama 时同时打开本地小模型路径,见 ADR-014)

**默认配置预设建议**(放在 dropdown 里方便用户选):
```yaml
presets:
  - id: openai
    name: OpenAI
    base_url: https://api.openai.com/v1
    model_default: gpt-4o-mini
  - id: deepseek
    name: DeepSeek
    base_url: https://api.deepseek.com
    model_default: deepseek-chat
  - id: moonshot
    name: Moonshot (Kimi)
    base_url: https://api.moonshot.cn/v1
    model_default: moonshot-v1-8k
  - id: qwen
    name: 通义千问
    base_url: https://dashscope.aliyuncs.com/compatible-mode/v1
    model_default: qwen-turbo
  - id: ollama
    name: 本地 Ollama
    base_url: http://localhost:11434/v1
    model_default: qwen2.5:3b
  - id: custom
    name: 自定义...
```

## 决策

**选定:选项 A — "零默认 + 引导链接"** + 6 个 preset。

Onboarding Step 6 不强制 API Key,首启 5 个本地能力(提醒/番茄/待办/摸鱼/物理交互)即可使用;首次唤起对话失败时再引导。设置 → 模型页提供 6 个 preset:**OpenAI / DeepSeek / Moonshot / 通义千问 / 本地 Ollama / 自定义**(完整配置见架构 v1.0 §0.2)。

**MVP 兼容矩阵**:OpenAI 兼容协议 100%(P0 必备);Anthropic messages API → P1-R1;Gemini → P1-R2。

## 后果

### 正面
- 零运营成本(我们不承担用户 LLM token 账单),与 local-first 立场一致。
- 国内/海外用户都能自由选择(DeepSeek/Moonshot/通义 在国内有免费试用,海外 OpenAI 兼容广)。
- API Key 通过 Windows DPAPI 加密(用户账户绑定),不引入服务端密钥管理负担。

### 负面
- 首启转化率受影响(用户嫌"还要去申请 API Key"麻烦关掉),D1 留存(KPI 11.1)有压力。
- 缓解措施:Onboarding 改为"先体验 5 个本地能力" + 桌宠首次需要联网时再弹引导;提升 Onboarding 完成率独立于对话 Provider 配置。

## 实施动作

- [ ] 输出 Provider preset 列表(JSON 资源)
- [ ] 设置 → 模型页 UI 设计稿(支持 preset 选择 + 自定义)
- [ ] 实现"测试连接"按钮(调用模型一次轻量请求验证)
- [ ] 错误码映射:401/429/500 → 友好提示
- [ ] M3 增加 Anthropic 协议支持(P1-R1 范畴,本期占位)
- [ ] 与 ADR-006(安全前缀)联动:不同 Provider 的 system role 拼装方式

## 引用

- PRD v0.6 §7.7(模块 G 设置)
- 架构 v0.3 §6(LLM Provider 抽象)
- 流程 v0.5 §1.2(Onboarding Step 6)

## 复审签字

| Reviewer | 角色 | 同意? | 备注 |
|---|---|---|---|
| M0 决策周 | 产品 | ☑ | 与 PRD §2.M0 决策 25 一致 |
| M0 决策周 | 后端 | ☑ | DPAPI 加密 + 测试连接按钮在 M3 设置页落地 |

// B.1 LLMProvider — OpenAI 兼容 streaming chat completion
//
// 范围(plan B.1 T2):
// - 6 preset 静态数组(架构 §0.2 + ADR-005;DeepSeek base_url 已统一含 /v1,与其他 5 preset 对齐)
// - OpenAiCompatProvider struct(KISS,不抽 trait;P1-R1 接 Anthropic 时再抽,见 decisions-log)
// - chat_stream:reqwest POST + SSE 解析 → Stream<Result<ChatChunk, LlmError>>
// - ping:GET {base_url}/models,200 即 ok(零 token 消耗 + 6 preset 全兼容)
// - 错误码映射:401 → Unauthorized / 429 → RateLimited / 5xx & 其他非 200 → Server
//
// 不在范围:
// - chat.send / chat.cancel / chat.history IPC(B.2 ChatService)
// - SecurityGuard 注入 / persona prompt 拼装(B.2)
// - tokio::CancellationToken 取消(B.2 实现 — Stream drop 自带早终止;cancel 是把 stream 主动 abort)
// - Anthropic / Gemini 协议(P1-R1 / R2)
//
// 设计决定:
// - api_key 字段 private,struct 不实现 Debug 透露(防误打日志)
// - base_url 在构造时做 normalize_base_url,保证一律含 /v1 不带尾部 /
// - SSE 解析抽 parse_sse_payload(&str) -> Result<Option<ChatChunk>, LlmError> 纯函数,
//   单测可精准覆盖 token / done / malformed / empty delta / finish_reason 5 种边界
// - ChatChunk::Done.latency_ms 是首字节到 [DONE] 的总耗时(M1 不区分 first_token vs total)
//
// 验收:
// - cargo test 覆盖 normalize_base_url + parse_sse_payload + map_status 共 12+ case
// - 真实 streaming 端到端在 dev panel `dev_llm_test_stream` 命令验证(T4)

use async_stream::try_stream;
use eventsource_stream::Eventsource;
use futures::stream::{BoxStream, StreamExt};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use thiserror::Error;

use crate::services::crypto::CryptoError;
use crate::services::secrets::{self, SecretsError};
use tauri::{AppHandle, Runtime};

// ============================================================================
// 6 Preset(架构 §0.2 + ADR-005)
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct ProviderPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub model_default: &'static str,
}

/// 6 个内置 preset。
///
/// **与 ADR-005 字面差异**:DeepSeek base_url 此处统一加 `/v1` 后缀,
/// 与其他 5 preset(openai/moonshot/qwen/ollama)对齐。代码侧 `normalize_base_url` 也做幂等兜底,
/// 即使用户填了不带 v1 的 DeepSeek base_url,运行时也会被自动补全。
/// doc-aligner 后续把 ADR-005 § preset 表 DeepSeek 改成带 v1(纯文档同步,不影响实现)。
pub const PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        id: "openai",
        name: "OpenAI",
        base_url: "https://api.openai.com/v1",
        model_default: "gpt-4o-mini",
    },
    ProviderPreset {
        id: "deepseek",
        name: "DeepSeek",
        base_url: "https://api.deepseek.com/v1",
        model_default: "deepseek-chat",
    },
    ProviderPreset {
        id: "moonshot",
        name: "Moonshot (Kimi)",
        base_url: "https://api.moonshot.cn/v1",
        model_default: "moonshot-v1-8k",
    },
    ProviderPreset {
        id: "qwen",
        name: "通义千问",
        base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
        model_default: "qwen-turbo",
    },
    ProviderPreset {
        id: "ollama",
        name: "本地 Ollama",
        base_url: "http://localhost:11434/v1",
        model_default: "qwen2.5:3b",
    },
    ProviderPreset {
        id: "custom",
        name: "自定义...",
        base_url: "",
        model_default: "",
    },
];

// ============================================================================
// 类型契约(架构 §6.1 微调:struct 直暴而非 trait,见 plan / decisions-log)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 'system' | 'user' | 'assistant'
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Default)]
pub struct ChatOptions {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum ChatChunk {
    /// SSE delta.content 增量 token
    Token(String),
    /// 流结束 — `[DONE]` 哨兵 / finish_reason / 上游正常关闭流
    Done {
        latency_ms: u64,
    },
}

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("missing api key for provider {0}")]
    MissingKey(String),
    #[error("auth failed (401): bad api key")]
    Unauthorized,
    #[error("rate limited (429)")]
    RateLimited,
    #[error("server error {status}: {message}")]
    Server { status: u16, message: String },
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("malformed sse: {0}")]
    Sse(String),
    #[error("crypto: {0}")]
    Crypto(#[from] CryptoError),
    #[error("secrets: {0}")]
    Secrets(#[from] SecretsError),
}

// ============================================================================
// base_url 归一化
// ============================================================================

/// 把任意用户/preset 输入归一化为 `<host>/v1`(无尾部 `/`)。
///
/// 幂等:`https://api.openai.com/v1/` → `https://api.openai.com/v1`
///       `https://api.deepseek.com`    → `https://api.deepseek.com/v1`
///       `https://api.deepseek.com/v1` → `https://api.deepseek.com/v1`
///
/// 安全性:不做 URL 解析,仅字符串处理 — 6 preset + custom 路径都是 OpenAI 兼容,统一末段 `/v1`。
/// 反例(custom 用户填了非 /v1 的 path 比如 `/api/v2`)会被错误地补 `/v1`。MVP 期接受这个限制 —
/// custom 用户配错时 secrets.test 会立即返回 401/404 反馈,不会静默错误。
pub fn normalize_base_url(s: &str) -> String {
    let trimmed = s.trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}

// ============================================================================
// HTTP 状态码 → LlmError 映射
// ============================================================================

/// 把非 200 状态码 + body 映射成具体 LlmError variant。
fn map_status(status: u16, body: &str) -> LlmError {
    match status {
        401 => LlmError::Unauthorized,
        429 => LlmError::RateLimited,
        s => LlmError::Server {
            status: s,
            // body 截到 200 字符避免日志爆炸 + 防误打 api_key 进 server error 链(本期 body 来自 LLM 服务端,
            // 不会含 client 侧 key,但仍保守截断)
            message: body.chars().take(200).collect(),
        },
    }
}

// ============================================================================
// SSE payload 解析
// ============================================================================

/// 解析 OpenAI 兼容 SSE 单条 payload。
///
/// 输入:`data:` 后面的 payload 字符串(已被 eventsource-stream 切好,不含 `data: ` 前缀和 `\n\n`)
///
/// 返回:
/// - `Ok(Some(ChatChunk::Token(s)))`:有效 token delta
/// - `Ok(Some(ChatChunk::Done { .. }))`:`[DONE]` 哨兵或 finish_reason 触发
/// - `Ok(None)`:空 delta(LLM 偶尔会下发 `{"delta":{}}` 心跳),调用方跳过
/// - `Err(LlmError::Sse(_))`:JSON 解析失败 / schema 不符
pub fn parse_sse_payload(payload: &str, latency_ms: u64) -> Result<Option<ChatChunk>, LlmError> {
    let trimmed = payload.trim();
    if trimmed == "[DONE]" {
        return Ok(Some(ChatChunk::Done { latency_ms }));
    }

    // OpenAI streaming response schema:
    //   { "id":"...", "choices":[{"delta":{"content":"Hi"},"finish_reason":null}] }
    // 我们只关心 choices[0].delta.content + choices[0].finish_reason
    // 防御:choices / delta 都用 #[serde(default)] —— Ollama / 个别 compat 服务的第一帧
    // 可能下发 `{"id":"..."}` 没有 choices,视为心跳跳过
    #[derive(Deserialize)]
    struct Chunk {
        #[serde(default)]
        choices: Vec<Choice>,
    }
    #[derive(Deserialize)]
    struct Choice {
        #[serde(default)]
        delta: Delta,
        #[serde(default)]
        finish_reason: Option<String>,
    }
    #[derive(Deserialize, Default)]
    struct Delta {
        #[serde(default)]
        content: Option<String>,
    }

    let parsed: Chunk =
        serde_json::from_str(trimmed).map_err(|e| LlmError::Sse(format!("json: {e}")))?;
    let Some(choice) = parsed.choices.into_iter().next() else {
        // 无 choices(罕见,ollama / 个别 compat 服务在第一帧),视为心跳
        return Ok(None);
    };

    if choice.finish_reason.is_some() {
        return Ok(Some(ChatChunk::Done { latency_ms }));
    }

    match choice.delta.content {
        Some(s) if !s.is_empty() => Ok(Some(ChatChunk::Token(s))),
        // 空 delta 心跳:`{"delta":{}}` 或 `{"delta":{"content":""}}`
        _ => Ok(None),
    }
}

// ============================================================================
// OpenAiCompatProvider
// ============================================================================

/// OpenAI 兼容 LLM Provider(覆盖 OpenAI / DeepSeek / Moonshot / Qwen / Ollama / Custom)。
///
/// **不实现 Debug**:防止 api_key 字段被误打进日志(如 `eprintln!("{provider:?}")`)。
/// 调试需要时打印 `provider_id` / `base_url` / `model` 三个 pub 字段足够。
pub struct OpenAiCompatProvider {
    pub provider_id: String,
    pub base_url: String, // 已 normalize:含 /v1,无尾部 /
    pub model: String,
    api_key: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct ChatRequestBody<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

impl OpenAiCompatProvider {
    /// 直接用明文 api_key 构造(测试 / dev panel 的 dev_llm_test_stream 用)。
    pub fn new(provider_id: &str, base_url: &str, model: &str, api_key: String) -> Self {
        // reqwest::Client::new() 默认带 rustls-tls + system proxy(我们的 features 已开),
        // 国内访问 OpenAI 需依赖系统代理 — 不显式 .no_proxy()。
        // 失败仅在 rustls 初始化异常(MVP 期 Win / macOS / Linux 三平台均稳定),让其 panic 比静默
        // fallback 更清晰(reqwest::Client::new() 内部就是 builder().build().expect(...))。
        let client = reqwest::Client::new();
        Self {
            provider_id: provider_id.to_string(),
            base_url: normalize_base_url(base_url),
            model: model.to_string(),
            api_key,
            client,
        }
    }

    /// 从 secrets 表取 api_key 构造 provider。
    /// secrets 不存在 → `LlmError::MissingKey(provider_id)`,UI 引导用户去设置页填 key。
    pub async fn from_secrets<R: Runtime>(
        app: &AppHandle<R>,
        provider_id: &str,
        base_url: &str,
        model: &str,
    ) -> Result<Self, LlmError> {
        let api_key = secrets::get(app, provider_id)
            .await?
            .ok_or_else(|| LlmError::MissingKey(provider_id.to_string()))?;
        Ok(Self::new(provider_id, base_url, model, api_key))
    }

    /// 探活:GET `{base_url}/models`,200 即 ok。
    /// 6 preset 全部支持 GET /v1/models(OpenAI 标准端点;Ollama 在 1.x 也实现了)。
    pub async fn ping(&self) -> Result<Duration, LlmError> {
        let url = format!("{}/models", self.base_url);
        let started = Instant::now();
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(map_status(status, &body));
        }
        // 不读 body — 节省带宽,模型列表在 secrets.test 上下文不需要返回前端
        Ok(started.elapsed())
    }

    /// 流式 chat completion。
    ///
    /// 调用方(B.2 ChatService)责任:
    /// - fold token 拼成 fullText
    /// - emit `chat.token` / `chat.done` / `chat.error` 事件
    /// - 用 tokio::select! + CancellationToken 实现 chat.cancel(drop stream 即可早停)
    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        options: ChatOptions,
    ) -> Result<BoxStream<'static, Result<ChatChunk, LlmError>>, LlmError> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = ChatRequestBody {
            model: if options.model.is_empty() {
                &self.model
            } else {
                &options.model
            },
            messages: &messages,
            stream: true,
            temperature: options.temperature,
            max_tokens: options.max_tokens,
        };

        let started = Instant::now();
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .header("Accept", "text/event-stream")
            .json(&body)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(map_status(status, &body));
        }

        // bytes_stream → eventsource-stream:把 chunk 重新组装成 SSE event(处理跨包断行)
        let event_stream = resp.bytes_stream().eventsource();

        let stream = try_stream! {
            futures::pin_mut!(event_stream);
            let mut emitted_done = false;

            while let Some(ev) = event_stream.next().await {
                let ev = ev.map_err(|e| LlmError::Sse(format!("eventsource: {e}")))?;
                let latency_ms = started.elapsed().as_millis() as u64;
                match parse_sse_payload(&ev.data, latency_ms)? {
                    Some(chunk @ ChatChunk::Done { .. }) => {
                        emitted_done = true;
                        yield chunk;
                        break;
                    }
                    Some(chunk) => yield chunk,
                    None => continue,
                }
            }

            // 上游正常关闭流但没下发 [DONE](Ollama 偶现)— 主动补 Done
            if !emitted_done {
                yield ChatChunk::Done {
                    latency_ms: started.elapsed().as_millis() as u64,
                };
            }
        };

        Ok(stream.boxed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== normalize_base_url =====

    #[test]
    fn normalize_already_v1() {
        assert_eq!(
            normalize_base_url("https://api.openai.com/v1"),
            "https://api.openai.com/v1"
        );
    }

    #[test]
    fn normalize_strips_trailing_slash() {
        assert_eq!(
            normalize_base_url("https://api.openai.com/v1/"),
            "https://api.openai.com/v1"
        );
    }

    #[test]
    fn normalize_appends_v1_when_missing() {
        // DeepSeek 在 ADR-005 字面是 https://api.deepseek.com (无 /v1)
        // — preset 表已修正,但用户手填可能仍然不带,normalize 兜底
        assert_eq!(
            normalize_base_url("https://api.deepseek.com"),
            "https://api.deepseek.com/v1"
        );
    }

    #[test]
    fn normalize_strips_then_appends() {
        assert_eq!(
            normalize_base_url("https://api.deepseek.com/"),
            "https://api.deepseek.com/v1"
        );
    }

    #[test]
    fn normalize_localhost_ollama() {
        assert_eq!(
            normalize_base_url("http://localhost:11434/v1"),
            "http://localhost:11434/v1"
        );
    }

    #[test]
    fn normalize_qwen_compatible_mode() {
        assert_eq!(
            normalize_base_url("https://dashscope.aliyuncs.com/compatible-mode/v1"),
            "https://dashscope.aliyuncs.com/compatible-mode/v1"
        );
    }

    // ===== parse_sse_payload =====

    #[test]
    fn parse_sse_token() {
        let payload = r#"{"choices":[{"delta":{"content":"Hi"},"finish_reason":null}]}"#;
        let r = parse_sse_payload(payload, 100).unwrap();
        match r {
            Some(ChatChunk::Token(s)) => assert_eq!(s, "Hi"),
            other => panic!("expected Token, got {other:?}"),
        }
    }

    #[test]
    fn parse_sse_done_sentinel() {
        let r = parse_sse_payload("[DONE]", 200).unwrap();
        assert!(matches!(r, Some(ChatChunk::Done { latency_ms: 200 })));
    }

    #[test]
    fn parse_sse_done_via_finish_reason() {
        let payload = r#"{"choices":[{"delta":{},"finish_reason":"stop"}]}"#;
        let r = parse_sse_payload(payload, 300).unwrap();
        assert!(matches!(r, Some(ChatChunk::Done { latency_ms: 300 })));
    }

    #[test]
    fn parse_sse_empty_delta_returns_none() {
        // 空 delta 心跳 — 调用方跳过不发 token
        let payload = r#"{"choices":[{"delta":{}}]}"#;
        let r = parse_sse_payload(payload, 100).unwrap();
        assert!(r.is_none(), "empty delta should be heartbeat (None)");
    }

    #[test]
    fn parse_sse_empty_string_content_returns_none() {
        // OpenAI 第一帧通常是 `{"delta":{"role":"assistant","content":""}}` —
        // content 为空字符串也视为心跳,不推 Token
        let payload = r#"{"choices":[{"delta":{"role":"assistant","content":""}}]}"#;
        let r = parse_sse_payload(payload, 100).unwrap();
        assert!(r.is_none(), "empty string content should not push token");
    }

    #[test]
    fn parse_sse_no_choices_returns_none() {
        // ollama 偶尔下发 `{"id":"..."}` 没 choices —— 视为心跳
        let payload = r#"{"id":"chatcmpl-123"}"#;
        let r = parse_sse_payload(payload, 100).unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn parse_sse_malformed_json_errors() {
        let r = parse_sse_payload("not json at all", 100);
        assert!(matches!(r, Err(LlmError::Sse(_))));
    }

    #[test]
    fn parse_sse_with_whitespace_padding() {
        // eventsource-stream 应该已经 trim,但 parse 自己也 trim 防御
        let payload = "  [DONE]  ";
        let r = parse_sse_payload(payload, 100).unwrap();
        assert!(matches!(r, Some(ChatChunk::Done { .. })));
    }

    // ===== map_status =====

    #[test]
    fn map_status_401_unauthorized() {
        assert!(matches!(
            map_status(401, "Invalid API key"),
            LlmError::Unauthorized
        ));
    }

    #[test]
    fn map_status_429_rate_limited() {
        assert!(matches!(map_status(429, "slow down"), LlmError::RateLimited));
    }

    #[test]
    fn map_status_500_server_error() {
        match map_status(500, "boom") {
            LlmError::Server { status, message } => {
                assert_eq!(status, 500);
                assert_eq!(message, "boom");
            }
            other => panic!("expected Server, got {other:?}"),
        }
    }

    #[test]
    fn map_status_truncates_long_body() {
        // 防御性截断 — 防 server error 链日志爆炸
        let long_body = "x".repeat(1000);
        match map_status(503, &long_body) {
            LlmError::Server { message, .. } => {
                assert!(message.chars().count() <= 200, "body must be truncated");
            }
            other => panic!("expected Server, got {other:?}"),
        }
    }

    // ===== PRESETS 完整性 =====

    #[test]
    fn presets_have_six_entries() {
        assert_eq!(PRESETS.len(), 6);
    }

    #[test]
    fn presets_ids_are_unique() {
        let ids: std::collections::HashSet<_> = PRESETS.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), PRESETS.len(), "preset ids must be unique");
    }

    #[test]
    fn presets_known_ids_present() {
        let ids: Vec<_> = PRESETS.iter().map(|p| p.id).collect();
        for expected in ["openai", "deepseek", "moonshot", "qwen", "ollama", "custom"] {
            assert!(ids.contains(&expected), "missing preset: {expected}");
        }
    }

    #[test]
    fn presets_non_custom_have_normalized_base_url() {
        // 防御:除 custom 外,其他 5 preset 的 base_url 应已经过 normalize_base_url 后保持不变
        // (即:含 /v1,无尾部 /);R2 风险的 doc-aligner 同步一致性守卫
        for preset in PRESETS.iter().filter(|p| p.id != "custom") {
            let normalized = normalize_base_url(preset.base_url);
            assert_eq!(
                preset.base_url, normalized,
                "preset {} base_url not normalized",
                preset.id
            );
        }
    }

    // ===== ChatRequestBody 序列化(防 stream/temperature 字段名漂移) =====

    #[test]
    fn chat_request_body_serializes_stream_true() {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "hi".to_string(),
        }];
        let body = ChatRequestBody {
            model: "gpt-4o-mini",
            messages: &messages,
            stream: true,
            temperature: Some(0.7),
            max_tokens: Some(64),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains(r#""stream":true"#));
        assert!(json.contains(r#""model":"gpt-4o-mini""#));
        assert!(json.contains(r#""temperature":0.7"#));
        assert!(json.contains(r#""max_tokens":64"#));
    }

    #[test]
    fn chat_request_body_skips_none_options() {
        let body = ChatRequestBody {
            model: "x",
            messages: &[],
            stream: true,
            temperature: None,
            max_tokens: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(!json.contains("temperature"));
        assert!(!json.contains("max_tokens"));
    }
}

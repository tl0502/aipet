// B.1 LLMProvider IPC commands(架构 §5.1 secrets.* + dev_llm_test_stream)
//
// MVP 暴露 5 个 commands:
//   - secrets_set_api_key  (架构 §656 secrets.set_api_key)
//   - secrets_delete_api_key  (B.1 加 — 用户撤回 key 用)
//   - secrets_test  (架构 §657 secrets.test → GET /v1/models)
//   - llm_list_presets  (Onboarding/G 设置页 dropdown 用,不依赖 DB)
//   - dev_llm_test_stream  (debug-only,端到端 streaming 验证;release 不存在)
//
// 不在范围:chat.send / chat.cancel / chat.history(B.2 ChatService)
//
// 错误透传:NicknameError 同款 — `.map_err(|e| e.to_string())` 转 String,
// IPC 层不暴露具体 LlmError variant 给前端(避免类型耦合,前端按 message 分类即可)

use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::services::llm::{
    ChatChunk, ChatMessage, ChatOptions, OpenAiCompatProvider, ProviderPreset, PRESETS,
};
use crate::services::secrets;

// ============================================================================
// secrets.* — set / delete / test
// ============================================================================

#[tauri::command]
pub async fn secrets_set_api_key(
    app: AppHandle,
    provider: String,
    key: String,
) -> Result<(), String> {
    secrets::set(&app, &provider, &key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn secrets_delete_api_key(app: AppHandle, provider: String) -> Result<(), String> {
    secrets::delete(&app, &provider)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub ok: bool,
    pub latency_ms: u64,
    /// 失败时携带 friendly message(401 / 网络 / 5xx 已分类);成功时为 None
    pub message: Option<String>,
}

#[tauri::command]
pub async fn secrets_test(
    app: AppHandle,
    provider: String,
    base_url: String,
    model: String,
) -> Result<TestResult, String> {
    // 1) 从 secrets 表取 api_key 构造 provider
    let provider_inst =
        match OpenAiCompatProvider::from_secrets(&app, &provider, &base_url, &model).await {
            Ok(p) => p,
            Err(e) => {
                return Ok(TestResult {
                    ok: false,
                    latency_ms: 0,
                    message: Some(e.to_string()),
                })
            }
        };

    // 2) ping → GET /v1/models
    match provider_inst.ping().await {
        Ok(d) => Ok(TestResult {
            ok: true,
            latency_ms: d.as_millis() as u64,
            message: None,
        }),
        Err(e) => Ok(TestResult {
            ok: false,
            latency_ms: 0,
            message: Some(e.to_string()),
        }),
    }
}

// ============================================================================
// llm_list_presets — preset dropdown
// ============================================================================

#[tauri::command]
pub fn llm_list_presets() -> Vec<ProviderPreset> {
    PRESETS.to_vec()
}

// ============================================================================
// dev_llm_test_stream — debug-only 端到端 streaming 验证
// ============================================================================
//
// 前端调用流程(IPC playground / Events tab):
//   1. invoke('dev_llm_test_stream', { provider, base_url, model, user_message })
//   2. 后端 chat_stream 逐 token 通过 emit('dev.llm.token', { delta }) 推送
//   3. 流结束后命令返回完整 fullText(供 IPC 调用方对照)
//
// release build 完全不编译此命令(双重排除:#[cfg(debug_assertions)] + lib.rs invoke_handler 注册处)

#[cfg(debug_assertions)]
#[tauri::command]
pub async fn dev_llm_test_stream<R: Runtime>(
    app: AppHandle<R>,
    provider: String,
    base_url: String,
    model: String,
    user_message: String,
) -> Result<String, String> {
    use futures::StreamExt;
    use tauri::Emitter;

    let provider_inst = OpenAiCompatProvider::from_secrets(&app, &provider, &base_url, &model)
        .await
        .map_err(|e| e.to_string())?;

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: user_message,
    }];
    let options = ChatOptions {
        model: model.clone(),
        temperature: Some(0.7),
        max_tokens: Some(256), // dev 期限 256 防止 token 飘走
    };

    let mut stream = provider_inst
        .chat_stream(messages, options)
        .await
        .map_err(|e| e.to_string())?;

    let mut full_text = String::new();
    while let Some(chunk_result) = stream.next().await {
        match chunk_result.map_err(|e| e.to_string())? {
            ChatChunk::Token(delta) => {
                full_text.push_str(&delta);
                // emit 失败不中断流(dev panel 可能未订阅)
                let _ = app.emit("dev.llm.token", &delta);
            }
            ChatChunk::Done { latency_ms } => {
                let _ = app.emit(
                    "dev.llm.done",
                    serde_json::json!({ "latency_ms": latency_ms, "full_text_len": full_text.len() }),
                );
                break;
            }
        }
    }

    Ok(full_text)
}

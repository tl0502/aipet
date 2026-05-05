// B.2 ChatService IPC commands(架构 §5.1 chat.* + conversation.*)
//
// 4 个 commands 暴露给前端:
//   - chat_send         立即返 message_id + conversation_id;后台 spawn run_chat 流式 emit events
//   - chat_cancel       查 AppState.chat_cancellations + .cancel() 触发 select! 早终止
//   - chat_history      转发 memory::list_messages_by_conversation
//   - conversation_create  最小 helper(B.3.a 启动新对话用);完整 CRUD(list/rename/archive/delete/activate)
//                          留 M3 B.3.d
//
// 命名约定:Tauri 2.x command name 仅允许 [a-zA-Z0-9_],故用 snake_case `chat_send` 等。
// 架构 §5.1 文档形式 `chat.send` 是 IPC 调用方约定的逻辑分组,前端 binding 层做映射 — 见 T7 注脚。
//
// 取消注册顺序(避免 race):
//   1. spawn 前先在 AppState 注册 (message_id, token)
//   2. spawn task 内 run_chat finally 反注册(无论 success / error / cancelled)
//   3. chat_cancel 查表 .cancel() 即可,反注册由 spawn task 自己负责
// 这种顺序保证 chat_cancel 在 spawn task 拿到 token 之前调用也能命中(token 已在表里)。

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

use crate::services::chat;
use crate::services::memory;
use crate::state::{lock_or_recover, AppState};

// ============================================================================
// chat_send
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ChatSendResponse {
    pub message_id: String,
    pub conversation_id: String,
}

/// 启动一轮对话:立即返 message_id + conversation_id,流式 token 通过 events 推送。
///
/// 调用契约(前端):
/// 1. 先 `listen('chat:token')` / `listen('chat:done')` / `listen('chat:error')`
/// 2. 再 `invoke('chat_send', { ... })` — 否则可能丢首个 token
/// 3. 用返回的 message_id 关联 events.payload.message_id
///
/// `conversation_id` 为 None → 自动 ULID 新建一个 conversation(persona_id = 当前激活)。
#[tauri::command]
pub async fn chat_send(
    app: AppHandle,
    state: State<'_, AppState>,
    conversation_id: Option<String>,
    provider: String,
    base_url: String,
    model: String,
    user_message: String,
) -> Result<ChatSendResponse, String> {
    let conv_id = conversation_id.unwrap_or_else(|| Ulid::new().to_string());
    let message_id = Ulid::new().to_string();

    // 注册 cancel token(在 spawn 前,保证 chat_cancel 总能命中)
    let token = CancellationToken::new();
    {
        let mut map = lock_or_recover(&state.chat_cancellations);
        map.insert(message_id.clone(), token.clone());
    }

    let app_clone = app.clone();
    let conv_clone = conv_id.clone();
    let msg_clone = message_id.clone();

    // 后台 spawn — IPC 立即返回,run_chat 内部 emit 流式 events
    tauri::async_runtime::spawn(async move {
        let result = chat::run_chat(
            app_clone.clone(),
            conv_clone,
            provider,
            base_url,
            model,
            user_message,
            msg_clone.clone(),
            token,
        )
        .await;

        // 反注册 cancel token(无论成功 / 失败 / cancelled)
        if let Some(state) = app_clone.try_state::<AppState>() {
            let mut map = lock_or_recover(&state.chat_cancellations);
            map.remove(&msg_clone);
        }

        if let Err(e) = result {
            // run_chat 内部已 emit chat:error;此处仅 eprintln 用于 dev 排查
            eprintln!("[chat_send] run_chat task ended with error: {e}");
        }
    });

    Ok(ChatSendResponse {
        message_id,
        conversation_id: conv_id,
    })
}

// ============================================================================
// chat_cancel
// ============================================================================

#[tauri::command]
pub async fn chat_cancel(state: State<'_, AppState>, message_id: String) -> Result<(), String> {
    let map = lock_or_recover(&state.chat_cancellations);
    if let Some(token) = map.get(&message_id) {
        token.cancel();
        Ok(())
    } else {
        // 已完成 / 已取消 / 不存在 — 都视为幂等成功
        Ok(())
    }
}

// ============================================================================
// chat_history
// ============================================================================

#[tauri::command]
pub async fn chat_history(
    app: AppHandle,
    conversation_id: String,
    limit: Option<u32>,
) -> Result<Vec<memory::MessageRecord>, String> {
    memory::list_messages_by_conversation(&app, &conversation_id, limit.or(Some(50)))
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// conversation_create
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ConvCreated {
    pub id: String,
}

#[tauri::command]
pub async fn conversation_create(
    app: AppHandle,
    persona_id: String,
    title: Option<String>,
) -> Result<ConvCreated, String> {
    let id = Ulid::new().to_string();
    let app_config = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("config dir: {e}"))?;
    let db_path = app_config.join("aipet.db");

    use sqlx::sqlite::SqliteConnectOptions;
    use sqlx::ConnectOptions;
    let mut conn = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false)
        .connect()
        .await
        .map_err(|e| format!("db connect: {e}"))?;

    chat::ensure_conversation_with_conn(&mut conn, &id, &persona_id, title.as_deref())
        .await
        .map_err(|e| e.to_string())?;
    use sqlx::Connection;
    conn.close().await.map_err(|e| format!("db close: {e}"))?;

    Ok(ConvCreated { id })
}

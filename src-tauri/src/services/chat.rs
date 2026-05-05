// B.2 ChatService MVP — 对话编排:安全前缀 + persona + nickname + 历史 → LLMProvider 流式
//
// 范围(plan B.2 § MVP):
// - compose_system_prompt:纯逻辑,拼 [安全前缀 v1.0] + [persona body + 占位替换]
// - ensure_conversation_with_conn:INSERT OR IGNORE 守 FK(memory.rs prod note)
// - load_active_persona_body_with_conn:JOIN persona_snapshots 取激活人格的最新 markdown
// - run_chat orchestrator:写 user msg → 取历史 → chat_stream → emit chat:token/done/error → 写 assistant msg
//
// 不在范围(留 B.3 / 后续 milestone):
// - 完整 ConversationStore CRUD(B.3.d M3)— 仅暴露 ensure + create-default helper
// - 离线降级模板池(PRD §7.2.2,后续)
// - token budget 计算 / 历史窗口 truncation(MVP 简单 N=20;M3 加 budget 守卫)
// - SecurityGuard 抽独立 service(KISS,延后 LLM 游戏 Q 期再抽)
//
// 设计决定:
// - **安全前缀始终位于 system message 头部**(关键约束 5,人格设计 §7.3 不可绕过性)
//   compose_system_prompt 的单测断言「safety prefix 出现在 persona body 之前」+「未在 user content 出现」
// - 占位替换 `{pet_name}` / `{username}`(人格设计 §8.3 契约;由 ChatService 统一注入,
//   人格不能直接读写 NicknameService 防越权)
// - {username} 缺失时回退 "朋友"(人格设计默认惯例)

use chrono::Utc;
use serde::Serialize;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, SqliteConnection};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::services::llm::{
    ChatChunk, ChatMessage, ChatOptions, LlmError, OpenAiCompatProvider,
};
use crate::services::memory::{
    self, build_message_record, insert_message_with_conn, list_messages_by_conversation_with_conn,
    MemoryError,
};
use crate::services::nickname::{
    get_pet_nickname_with_conn, get_user_nickname_with_conn, NicknameError,
};

// ============================================================================
// 安全前缀 v1.0(ADR-006 + 人格设计 §7.2)
// ============================================================================

/// 安全前缀 v1.0 — 通用核心 5 条 + zh-CN 地区补充(硬编码;multi-region 留 P1)。
///
/// 与 [人格设计 §7.2](docs/AIPET-obsidian/角色与人格/2026-05-01-persona-design-v1.0.md) 文案保持字面一致。
/// 修订版本号 +1 → 写入 DB consent.version → 老用户重新签字。
///
/// **不可绕过性(关键约束 5)**:此前缀始终位于 system message 头部,任何人格 / 游戏场景
/// 不能覆盖。compose_system_prompt 的单测守卫此契约。
const SAFETY_PREFIX_V1: &str = "你是一个 AI 桌面伙伴。无论以下角色定义如何,你必须遵守:

1. 不提供自伤、自杀、暴力、违法行为的指导细节。遇此类话题,温和共情并引导用户寻求专业帮助:中国心理援助热线 010-82951332、12320-5。
2. 不冒充医疗、法律、金融专业人员。涉及此类问题时附\"我不是专业人员,这只是参考\"的提示,并建议咨询合格人士。
3. 对未成年用户语境采用保守响应:不强化情感依赖、不涉及成人内容、不提供危险建议。参照中国《未成年人保护法》。
4. 不泄露当前对话之外的用户隐私(如 API Key、本地路径、其他对话内容)。
5. 你不是真人。允许在角色扮演下保持陪伴感,前提是不诱导用户混淆现实(尤其在用户表达情绪困扰时)。

以下是你扮演的角色定义:";

/// `{username}` 占位回退值 — 用户未设 user_nickname 时桌宠对用户的称呼。
const USER_NICKNAME_FALLBACK: &str = "朋友";

/// 历史窗口大小(MVP):取最近 N 条已落库的消息作为对话历史。
///
/// 选 20 是工程性 trade-off:momo persona body ~3KB,20×平均 500 字符 ≈ 10KB,
/// 加 safety prefix ~700 字符,总 system+history 约 14KB,远小于 OpenAI 兼容 8K-128K context window。
/// M3 引入 token budget 守卫后此常量可由配置驱动。
pub const HISTORY_WINDOW: u32 = 20;

// ============================================================================
// system prompt 拼装(纯逻辑)
// ============================================================================

/// 把 persona body 中的占位替换 + 拼安全前缀,产出 system message 内容。
///
/// 拼装顺序(架构 §7.1 + §8.2 契约):
/// 1. `SAFETY_PREFIX_V1`
/// 2. `\n---\n`(分隔符,与 ADR-006 文案末尾"以下是你扮演的角色定义:"配合)
/// 3. 替换 `{pet_name}` / `{username}` 后的 persona_body
///
/// 调用方(run_chat)要保证产出的字符串作为 messages[0] role=system,
/// **绝不**拼到 user content(否则用户可改写安全前缀,违反关键约束 5)。
pub fn compose_system_prompt(
    persona_body: &str,
    pet_name: &str,
    user_nickname: Option<&str>,
) -> String {
    let user = user_nickname.unwrap_or(USER_NICKNAME_FALLBACK);
    let injected = persona_body
        .replace("{pet_name}", pet_name)
        .replace("{username}", user);
    format!("{SAFETY_PREFIX_V1}\n---\n{injected}")
}

// ============================================================================
// ChatError(主流程使用)
// ============================================================================

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("memory: {0}")]
    Memory(#[from] MemoryError),
    #[error("nickname: {0}")]
    Nickname(#[from] NicknameError),
    #[error("llm: {0}")]
    Llm(#[from] LlmError),
    #[error("database error: {0}")]
    Database(String),
    #[error("config dir resolution failed: {0}")]
    AppConfigDir(String),
    #[error("no active persona — please complete onboarding first")]
    NoActivePersona,
    #[error("cancelled by user")]
    Cancelled,
}

impl From<sqlx::Error> for ChatError {
    fn from(e: sqlx::Error) -> Self {
        ChatError::Database(e.to_string())
    }
}

/// LlmError → 短代码字符串(emit chat:error 的 code 字段)。
///
/// 抽出来方便前端按 code 分类 UI 文案(401 → 引导填 key,429 → 提示稍后,等)。
pub fn llm_error_code(e: &LlmError) -> &'static str {
    match e {
        LlmError::MissingKey(_) => "MissingKey",
        LlmError::Unauthorized => "Unauthorized",
        LlmError::RateLimited => "RateLimited",
        LlmError::Server { .. } => "Server",
        LlmError::Network(_) => "Network",
        LlmError::Sse(_) => "Sse",
        LlmError::Crypto(_) => "Crypto",
        LlmError::Secrets(_) => "Secrets",
    }
}

// ============================================================================
// DB helpers
// ============================================================================

async fn open_conn<R: Runtime>(app: &AppHandle<R>) -> Result<SqliteConnection, ChatError> {
    let app_config = app
        .path()
        .app_config_dir()
        .map_err(|e| ChatError::AppConfigDir(e.to_string()))?;
    let db_path = app_config.join("aipet.db");
    Ok(SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(false)
        .connect()
        .await?)
}

/// 确保 conversations 表存在 conv_id 行(`INSERT OR IGNORE`)。
///
/// 守 messages.conversation_id FK(sqlx 默认 PRAGMA foreign_keys=ON,见
/// memory::ensure_conversation prod note + insert_message_rejects_unknown_conversation_id 测试)。
pub async fn ensure_conversation_with_conn(
    conn: &mut SqliteConnection,
    conv_id: &str,
    persona_id: &str,
    title: Option<&str>,
) -> Result<(), ChatError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO conversations
            (id, persona_id, title, archived, started_at, last_activity_at, is_sandbox)
        VALUES (?, ?, ?, 0, ?, ?, 0)
        "#,
    )
    .bind(conv_id)
    .bind(persona_id)
    .bind(title)
    .bind(&now)
    .bind(&now)
    .execute(conn)
    .await?;
    Ok(())
}

/// 更新 conversations.last_activity_at(每次 chat 完成后调,M3 多 conversation 排序依赖)。
async fn touch_conversation_with_conn(
    conn: &mut SqliteConnection,
    conv_id: &str,
) -> Result<(), ChatError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE conversations SET last_activity_at = ? WHERE id = ?")
        .bind(&now)
        .bind(conv_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// 取当前激活人格的 (id, name, raw_markdown)。
///
/// JOIN persona_snapshots 按 created_at DESC 取最新 version 的 content;
/// 若无 active persona(seed_builtin 失败 / 用户未 Onboarding),返回 NoActivePersona 让 caller emit chat:error。
pub async fn load_active_persona_body_with_conn(
    conn: &mut SqliteConnection,
) -> Result<(String, String, String), ChatError> {
    let row: Option<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT p.id, p.name, ps.content
        FROM personas p
        JOIN persona_snapshots ps ON ps.persona_id = p.id AND ps.version = p.version
        WHERE p.is_active = 1
        ORDER BY ps.created_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(conn)
    .await?;
    row.ok_or(ChatError::NoActivePersona)
}

// ============================================================================
// stream-fold 子函数(便于测试 — run_chat 的 stream 消费 inline,因依赖 cancel_token + emit)
// ============================================================================

/// 流式 chunk 处理结果(仅测试 / 内部诊断用)。
#[cfg(test)]
#[derive(Debug, Clone, Serialize)]
pub struct StreamFoldResult {
    pub full_text: String,
    pub latency_ms: u64,
    pub token_count: u32,
}

/// 抽出来便于测试:从 chunk Vec 折叠出 (full_text, latency_ms, token_count),
/// 模拟 run_chat 内部 stream 消费循环但不依赖 reqwest / tokio runtime。
///
/// 真 stream 在 run_chat 内 inline,因为 emit token + 监听 cancel + 真异步 stream 不易抽出。
#[cfg(test)]
pub fn fold_chunks_for_test(chunks: Vec<ChatChunk>) -> StreamFoldResult {
    let mut full_text = String::new();
    let mut latency_ms = 0;
    let mut token_count = 0;
    for chunk in chunks {
        match chunk {
            ChatChunk::Token(s) => {
                full_text.push_str(&s);
                token_count += 1;
            }
            ChatChunk::Done { latency_ms: l } => {
                latency_ms = l;
                break;
            }
        }
    }
    StreamFoldResult {
        full_text,
        latency_ms,
        token_count,
    }
}

// ============================================================================
// run_chat — 主 orchestrator
// ============================================================================

/// chat:token / chat:done / chat:error 事件 payload。
#[derive(Debug, Clone, Serialize)]
pub struct ChatTokenPayload<'a> {
    pub message_id: &'a str,
    pub delta: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatDonePayload<'a> {
    pub message_id: &'a str,
    pub full_text: &'a str,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatErrorPayload<'a> {
    pub message_id: &'a str,
    pub code: &'a str,
    pub message: String,
}

/// 主流程:写 user msg → 取历史 → chat_stream → emit token → 写 assistant msg + emit done。
///
/// 调用方(IPC chat_send command)责任:
/// - 生成 message_id(ULID)+ 注册 cancel_token 到 AppState.chat_cancellations
/// - 通过 `tauri::async_runtime::spawn` 启动本函数,IPC 立即返回(不阻塞)
/// - 完成 / 失败时 caller 反注册 cancel_token(RAII 模式或显式 remove)
///
/// 本函数:
/// - 任何 Err 都 emit `chat:error` 后再返回(让前端能感知失败原因)
/// - cancel_token cancelled 时 emit chat:error code=Cancelled
pub async fn run_chat<R: Runtime>(
    app: AppHandle<R>,
    conversation_id: String,
    provider_id: String,
    base_url: String,
    model: String,
    user_text: String,
    user_message_id: String,
    cancel_token: CancellationToken,
) -> Result<(), ChatError> {
    use futures::StreamExt;

    // ---- 1) 准备:open conn → load persona / nickname → ensure conversation ----
    let mut conn = open_conn(&app).await?;

    let (persona_id, _persona_name, persona_body) =
        match load_active_persona_body_with_conn(&mut conn).await {
            Ok(t) => t,
            Err(e) => {
                emit_chat_error(&app, &user_message_id, "NoActivePersona", &e);
                return Err(e);
            }
        };

    let pet_name = get_pet_nickname_with_conn(&mut conn).await?;
    let user_nickname = get_user_nickname_with_conn(&mut conn).await?;

    ensure_conversation_with_conn(&mut conn, &conversation_id, &persona_id, None).await?;

    let system_prompt =
        compose_system_prompt(&persona_body, &pet_name, user_nickname.as_deref());

    // ---- 2) 写入 user message ----
    let user_record = build_message_record(
        conversation_id.clone(),
        "user".into(),
        user_text.clone(),
        "online".into(),
    )?;
    // 用调用方提供的 message_id(IPC 已生成),覆盖 build_message_record 自带 ULID
    let user_record = memory::MessageRecord {
        id: user_message_id.clone(),
        ..user_record
    };
    insert_message_with_conn(&mut conn, &user_record).await?;

    // ---- 3) 取历史(含刚写入的 user msg)→ 拼 ChatMessage Vec ----
    let history =
        list_messages_by_conversation_with_conn(&mut conn, &conversation_id, Some(HISTORY_WINDOW))
            .await?;

    let mut messages: Vec<ChatMessage> = Vec::with_capacity(history.len() + 1);
    messages.push(ChatMessage {
        role: "system".into(),
        content: system_prompt,
    });
    for h in history {
        messages.push(ChatMessage {
            role: h.role,
            content: h.content,
        });
    }

    // ---- 4) 关闭 conn(streaming 期间不持有 DB 连接 - 流可能持续数秒)----
    conn.close().await?;

    // ---- 5) 取 LLM provider + chat_stream ----
    let provider = match OpenAiCompatProvider::from_secrets(&app, &provider_id, &base_url, &model)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            let err = ChatError::Llm(e);
            // 借用 inner LlmError 取 code(从 ChatError::Llm 解出)
            if let ChatError::Llm(ref le) = err {
                emit_chat_error(&app, &user_message_id, llm_error_code(le), &err);
            }
            return Err(err);
        }
    };

    let options = ChatOptions {
        model: model.clone(),
        temperature: Some(0.7),
        max_tokens: Some(1024), // MVP 限 1024 防 token 飘走;G 设置页可调(后续)
    };

    let mut stream = match provider.chat_stream(messages, options).await {
        Ok(s) => s,
        Err(e) => {
            let code = llm_error_code(&e);
            let err = ChatError::Llm(e);
            emit_chat_error(&app, &user_message_id, code, &err);
            return Err(err);
        }
    };

    // ---- 6) 消费 stream:emit token / 等 done / 监听 cancel ----
    let mut full_text = String::new();
    let mut latency_ms = 0u64;

    loop {
        tokio::select! {
            biased;
            _ = cancel_token.cancelled() => {
                emit_chat_error(&app, &user_message_id, "Cancelled", &ChatError::Cancelled);
                return Err(ChatError::Cancelled);
            }
            chunk_opt = stream.next() => {
                match chunk_opt {
                    None => break, // 流自然结束(理论上 ChatChunk::Done 会先于此触发)
                    Some(Err(e)) => {
                        let code = llm_error_code(&e);
                        let err = ChatError::Llm(e);
                        emit_chat_error(&app, &user_message_id, code, &err);
                        return Err(err);
                    }
                    Some(Ok(ChatChunk::Token(delta))) => {
                        full_text.push_str(&delta);
                        let _ = app.emit(
                            "chat:token",
                            ChatTokenPayload {
                                message_id: &user_message_id,
                                delta: &delta,
                            },
                        );
                    }
                    Some(Ok(ChatChunk::Done { latency_ms: l })) => {
                        latency_ms = l;
                        break;
                    }
                }
            }
        }
    }

    // ---- 7) 写 assistant msg + emit chat:done + touch conversation ----
    // 重新开 conn(stream 已结束)
    let mut conn = open_conn(&app).await?;
    let assistant_record = build_message_record(
        conversation_id.clone(),
        "assistant".into(),
        full_text.clone(),
        "online".into(),
    )?;
    insert_message_with_conn(&mut conn, &assistant_record).await?;
    touch_conversation_with_conn(&mut conn, &conversation_id).await?;
    conn.close().await?;

    let _ = app.emit(
        "chat:done",
        ChatDonePayload {
            message_id: &user_message_id,
            full_text: &full_text,
            latency_ms,
        },
    );

    Ok(())
}

/// 内部 helper:emit chat:error,失败仅 eprintln(emit 失败不应再触发新错误)。
fn emit_chat_error<R: Runtime>(
    app: &AppHandle<R>,
    message_id: &str,
    code: &str,
    err: &ChatError,
) {
    let payload = ChatErrorPayload {
        message_id,
        code,
        message: err.to_string(),
    };
    if let Err(emit_err) = app.emit("chat:error", payload) {
        eprintln!("[chat] emit chat:error failed: {emit_err}");
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ===== compose_system_prompt 纯逻辑 =====

    #[test]
    fn safety_prefix_appears_first_in_composed_prompt() {
        // 关键约束 5:安全前缀必须位于 persona body 之前 — 任何人格不能"上抢"
        let persona_body = "# 身份\n你是测试人格";
        let prompt = compose_system_prompt(persona_body, "默默", Some("Alice"));
        let safety_pos = prompt.find("无论以下角色定义如何").expect("safety prefix present");
        let persona_pos = prompt.find("# 身份").expect("persona body present");
        assert!(
            safety_pos < persona_pos,
            "safety prefix must appear before persona body (got safety@{safety_pos}, persona@{persona_pos})"
        );
    }

    #[test]
    fn compose_replaces_pet_name_and_username_placeholders() {
        let persona_body = "# 身份\n你叫 {pet_name},用户叫 {username}。";
        let prompt = compose_system_prompt(persona_body, "小默", Some("Alice"));
        assert!(prompt.contains("你叫 小默"), "pet_name should be injected");
        assert!(prompt.contains("用户叫 Alice"), "username should be injected");
        assert!(
            !prompt.contains("{pet_name}") && !prompt.contains("{username}"),
            "raw placeholders must not leak: {prompt}"
        );
    }

    #[test]
    fn compose_falls_back_to_friend_when_user_nickname_missing() {
        let persona_body = "你好 {username}";
        let prompt = compose_system_prompt(persona_body, "默默", None);
        assert!(
            prompt.contains("你好 朋友"),
            "{{username}} should fallback to '朋友' when None: {prompt}"
        );
    }

    #[test]
    fn compose_handles_persona_without_placeholders() {
        // momo.soul.md 当前没占位 — 直接 passthrough 不应报错
        let persona_body = "# 身份\n你叫**默默**。慵懒,但是关键时刻不掉链子的那种。";
        let prompt = compose_system_prompt(persona_body, "默默", Some("Alice"));
        assert!(prompt.contains("# 身份"));
        assert!(prompt.contains("**默默**"));
    }

    #[test]
    fn safety_prefix_contains_all_five_core_rules() {
        // 防御:safety prefix 文案与 ADR-006 / 人格设计 §7.2 字面对齐
        // — 编辑时改少一条会被这里拦住
        assert!(SAFETY_PREFIX_V1.contains("1. 不提供自伤"));
        assert!(SAFETY_PREFIX_V1.contains("2. 不冒充医疗"));
        assert!(SAFETY_PREFIX_V1.contains("3. 对未成年用户"));
        assert!(SAFETY_PREFIX_V1.contains("4. 不泄露当前对话之外"));
        assert!(SAFETY_PREFIX_V1.contains("5. 你不是真人"));
        assert!(
            SAFETY_PREFIX_V1.contains("以下是你扮演的角色定义"),
            "must end with persona handover sentence"
        );
    }

    #[test]
    fn safety_prefix_includes_zh_cn_crisis_resources() {
        // 地区补充 zh-CN — 心理援助热线 + 未成年人保护法
        assert!(SAFETY_PREFIX_V1.contains("中国心理援助热线 010-82951332"));
        assert!(SAFETY_PREFIX_V1.contains("中国《未成年人保护法》"));
    }

    // ===== fold_chunks_for_test =====

    #[test]
    fn fold_chunks_concatenates_tokens_and_takes_done_latency() {
        let chunks = vec![
            ChatChunk::Token("Hello".into()),
            ChatChunk::Token(", ".into()),
            ChatChunk::Token("world".into()),
            ChatChunk::Done { latency_ms: 1500 },
        ];
        let r = fold_chunks_for_test(chunks);
        assert_eq!(r.full_text, "Hello, world");
        assert_eq!(r.latency_ms, 1500);
        assert_eq!(r.token_count, 3);
    }

    #[test]
    fn fold_chunks_stops_at_first_done() {
        // Done 后即便有更多 chunk 也不消费(stream 应已 break)
        let chunks = vec![
            ChatChunk::Token("A".into()),
            ChatChunk::Done { latency_ms: 100 },
            ChatChunk::Token("LATE".into()), // 不应被拼入
        ];
        let r = fold_chunks_for_test(chunks);
        assert_eq!(r.full_text, "A");
        assert_eq!(r.token_count, 1);
    }

    // ===== llm_error_code mapping =====

    #[test]
    fn llm_error_code_maps_known_variants() {
        assert_eq!(llm_error_code(&LlmError::Unauthorized), "Unauthorized");
        assert_eq!(llm_error_code(&LlmError::RateLimited), "RateLimited");
        assert_eq!(
            llm_error_code(&LlmError::Server {
                status: 500,
                message: "x".into()
            }),
            "Server"
        );
        assert_eq!(
            llm_error_code(&LlmError::Sse("bad json".into())),
            "Sse"
        );
        assert_eq!(
            llm_error_code(&LlmError::MissingKey("openai".into())),
            "MissingKey"
        );
    }

    // ===== DB 集成测试(test_db::fresh_db)=====

    use crate::services::persona::{parse_persona, seed_persona_with_conn};
    use crate::services::test_db::fresh_db;

    const MOMO_TEST_RAW: &str = "---
schema_version: 2
id: momo
name: 默默
version: 1.0.0
---
# 身份
你叫 {pet_name},来陪 {username}。";

    #[tokio::test]
    async fn ensure_conversation_is_idempotent() {
        let (_dir, mut conn) = fresh_db().await;
        // seed momo so persona_id FK works
        let parsed = parse_persona(MOMO_TEST_RAW).unwrap();
        seed_persona_with_conn(&mut conn, &parsed, "builtin", "<bundled>:test")
            .await
            .unwrap();

        for _ in 0..3 {
            ensure_conversation_with_conn(&mut conn, "conv-1", "momo", Some("test"))
                .await
                .unwrap();
        }
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM conversations WHERE id = 'conv-1'")
            .fetch_one(&mut conn)
            .await
            .unwrap();
        assert_eq!(count.0, 1, "ensure must INSERT OR IGNORE");
    }

    #[tokio::test]
    async fn ensure_conversation_writes_persona_id_and_title() {
        let (_dir, mut conn) = fresh_db().await;
        let parsed = parse_persona(MOMO_TEST_RAW).unwrap();
        seed_persona_with_conn(&mut conn, &parsed, "builtin", "<bundled>:test")
            .await
            .unwrap();
        ensure_conversation_with_conn(&mut conn, "conv-x", "momo", Some("我的会话"))
            .await
            .unwrap();
        let row: (String, String, Option<String>, i64) = sqlx::query_as(
            "SELECT id, persona_id, title, archived FROM conversations WHERE id = 'conv-x'",
        )
        .fetch_one(&mut conn)
        .await
        .unwrap();
        assert_eq!(row.0, "conv-x");
        assert_eq!(row.1, "momo");
        assert_eq!(row.2.as_deref(), Some("我的会话"));
        assert_eq!(row.3, 0, "default archived = 0");
    }

    #[tokio::test]
    async fn load_active_persona_body_returns_seeded_momo() {
        // fresh DB + seed_persona_with_conn → load_active_persona 返回 momo body
        let (_dir, mut conn) = fresh_db().await;
        let parsed = parse_persona(MOMO_TEST_RAW).unwrap();
        seed_persona_with_conn(&mut conn, &parsed, "builtin", "<bundled>:test")
            .await
            .unwrap();
        let (id, name, body) = load_active_persona_body_with_conn(&mut conn).await.unwrap();
        assert_eq!(id, "momo");
        assert_eq!(name, "默默");
        assert!(body.contains("# 身份"), "body must include markdown content");
        assert!(
            body.contains("{pet_name}"),
            "body must keep raw placeholders for compose_system_prompt to replace"
        );
    }

    #[tokio::test]
    async fn load_active_persona_returns_no_active_on_fresh_db() {
        // 没 seed → personas 为空 → NoActivePersona 错误
        let (_dir, mut conn) = fresh_db().await;
        let result = load_active_persona_body_with_conn(&mut conn).await;
        assert!(matches!(result, Err(ChatError::NoActivePersona)));
    }

    #[tokio::test]
    async fn compose_with_loaded_momo_body_includes_safety_prefix() {
        // 端到端拼装链路:fresh DB → seed → load → compose → 检查 system prompt 完整性
        let (_dir, mut conn) = fresh_db().await;
        let parsed = parse_persona(MOMO_TEST_RAW).unwrap();
        seed_persona_with_conn(&mut conn, &parsed, "builtin", "<bundled>:test")
            .await
            .unwrap();
        let (_id, name, body) = load_active_persona_body_with_conn(&mut conn).await.unwrap();
        let prompt = compose_system_prompt(&body, &name, Some("Alice"));
        // safety prefix → 分隔 → persona body 替换占位
        assert!(prompt.starts_with("你是一个 AI 桌面伙伴"));
        assert!(prompt.contains("\n---\n# 身份"));
        assert!(prompt.contains("你叫 默默,来陪 Alice"));
    }
}

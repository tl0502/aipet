// F.1 MemoryService MVP — messages 表 CRUD + summary 占位
//
// 范围(plan F.1,选项 A):
// - messages 表 CRUD(insert / list / delete by id / delete by conversation)
// - summary 占位:summarize_conversation 返回固定字符串,M3 引入摘要算法时再填实
// - cleanup_messages_older_than(days) 私有 stub:不在 setup 调,留给将来设置面板触发
//
// 设计决定(2026-05-03 与用户对齐):
// - **不做 90 天自动清理** — 偏离 local-first 精神;桌宠核心价值是"老朋友",自动清空对话反价值
// - **默认无限保留** + 用户主动清理(类似 ChatGPT 网页);UI 入口由后续模块 B.3 / 设置面板提供
// - PRD §73 / 架构 §549 的"90 天默认 + is_deleted 软删"措辞与本实现偏差,留给 doc-aligner 后续对齐
// - messages.id 走 ULID(schema 注释明写;时间序利于 (conversation_id, created_at) 索引)
// - DB 连接同 persona.rs 模式:自开 sqlx 短期连接,DB 路径用 app.path().app_config_dir()

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, FromRow, SqliteConnection};
use std::str::FromStr;
use tauri::{AppHandle, Manager, Runtime};
use thiserror::Error;
use ulid::Ulid;

const VALID_ROLES: &[&str] = &["user", "assistant", "system"];
const VALID_MODES: &[&str] = &["online", "offline_rule"];

const SUMMARY_PLACEHOLDER: &str = "[摘要功能将于 M3 引入;此处为占位符,详见 progress/decisions-log.md]";

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageRecord {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub mode: String,
    pub created_at: String,
}

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("invalid role: got '{0}', want one of {1:?}")]
    InvalidRole(String, &'static [&'static str]),
    #[error("invalid mode: got '{0}', want one of {1:?}")]
    InvalidMode(String, &'static [&'static str]),
    #[error("database error: {0}")]
    Database(String),
    #[error("config dir resolution failed: {0}")]
    AppConfigDir(String),
}

impl From<sqlx::Error> for MemoryError {
    fn from(e: sqlx::Error) -> Self {
        MemoryError::Database(e.to_string())
    }
}

/// 纯逻辑构造:校验 role / mode + 生成 ULID + RFC3339 时间戳。
///
/// 抽出来便于测试 — insert_message 内部组装时 wrap 这个函数。
pub fn build_message_record(
    conversation_id: String,
    role: String,
    content: String,
    mode: String,
) -> Result<MessageRecord, MemoryError> {
    if !VALID_ROLES.contains(&role.as_str()) {
        return Err(MemoryError::InvalidRole(role, VALID_ROLES));
    }
    if !VALID_MODES.contains(&mode.as_str()) {
        return Err(MemoryError::InvalidMode(mode, VALID_MODES));
    }
    Ok(MessageRecord {
        id: Ulid::new().to_string(),
        conversation_id,
        role,
        content,
        mode,
        created_at: Utc::now().to_rfc3339(),
    })
}

/// 计算 N 天之前的 RFC3339 时间戳 — cleanup_messages_older_than 用。
fn cutoff_for_days(days: u32) -> String {
    (Utc::now() - chrono::Duration::days(days as i64)).to_rfc3339()
}

async fn open_conn<R: Runtime>(app: &AppHandle<R>) -> Result<SqliteConnection, MemoryError> {
    let app_config = app
        .path()
        .app_config_dir()
        .map_err(|e| MemoryError::AppConfigDir(e.to_string()))?;
    let db_path = app_config.join("aipet.db");
    let db_url = format!("sqlite:{}", db_path.display());
    Ok(SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(false)
        .connect()
        .await?)
}

/// 写入一条消息(ChatService B.2 streaming 完成后调用)。
pub async fn insert_message<R: Runtime>(
    app: &AppHandle<R>,
    conversation_id: String,
    role: String,
    content: String,
    mode: String,
) -> Result<MessageRecord, MemoryError> {
    let record = build_message_record(conversation_id, role, content, mode)?;

    let mut conn = open_conn(app).await?;
    sqlx::query(
        r#"
        INSERT INTO messages (id, conversation_id, role, content, mode, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&record.id)
    .bind(&record.conversation_id)
    .bind(&record.role)
    .bind(&record.content)
    .bind(&record.mode)
    .bind(&record.created_at)
    .execute(&mut conn)
    .await?;
    conn.close().await?;

    Ok(record)
}

/// 按 conversation_id 列消息,按 created_at 升序;limit=None 时返回全部。
///
/// ChatService 拼 system prompt + recent messages 时调用;UI 翻历史也用。
pub async fn list_messages_by_conversation<R: Runtime>(
    app: &AppHandle<R>,
    conversation_id: &str,
    limit: Option<u32>,
) -> Result<Vec<MessageRecord>, MemoryError> {
    let mut conn = open_conn(app).await?;

    let records: Vec<MessageRecord> = match limit {
        Some(n) => sqlx::query_as::<_, MessageRecord>(
            r#"
            SELECT id, conversation_id, role, content, mode, created_at
            FROM messages
            WHERE conversation_id = ?
            ORDER BY created_at ASC
            LIMIT ?
            "#,
        )
        .bind(conversation_id)
        .bind(n)
        .fetch_all(&mut conn)
        .await?,
        None => sqlx::query_as::<_, MessageRecord>(
            r#"
            SELECT id, conversation_id, role, content, mode, created_at
            FROM messages
            WHERE conversation_id = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(conversation_id)
        .fetch_all(&mut conn)
        .await?,
    };

    conn.close().await?;
    Ok(records)
}

/// 按消息 ID 删除(用户主动 "删除某条" 用)。
pub async fn delete_message<R: Runtime>(
    app: &AppHandle<R>,
    id: &str,
) -> Result<(), MemoryError> {
    let mut conn = open_conn(app).await?;
    sqlx::query("DELETE FROM messages WHERE id = ?")
        .bind(id)
        .execute(&mut conn)
        .await?;
    conn.close().await?;
    Ok(())
}

/// 按 conversation_id 清空全部消息(用户在 ChatPanel 点 "清空对话" 时调用 backing)。
/// 返回删除行数。
pub async fn delete_messages_by_conversation<R: Runtime>(
    app: &AppHandle<R>,
    conversation_id: &str,
) -> Result<u64, MemoryError> {
    let mut conn = open_conn(app).await?;
    let result = sqlx::query("DELETE FROM messages WHERE conversation_id = ?")
        .bind(conversation_id)
        .execute(&mut conn)
        .await?;
    conn.close().await?;
    Ok(result.rows_affected())
}

/// 摘要占位 — M3 引入压缩算法时填实(LLM 摘要 / 抽取式摘要等)。
pub async fn summarize_conversation<R: Runtime>(
    _app: &AppHandle<R>,
    _conversation_id: &str,
) -> Result<String, MemoryError> {
    Ok(SUMMARY_PLACEHOLDER.to_string())
}

/// 私有 stub:删除 N 天前的消息。F.1 不在启动时调,留给将来设置面板"自动清理"开关触发。
///
/// 设计意图:用户主动选择"开启 N 天清理"时才生效,默认无限保留(local-first 精神)。
#[allow(dead_code)]
pub(crate) async fn cleanup_messages_older_than<R: Runtime>(
    app: &AppHandle<R>,
    days: u32,
) -> Result<u64, MemoryError> {
    let cutoff = cutoff_for_days(days);
    let mut conn = open_conn(app).await?;
    let result = sqlx::query("DELETE FROM messages WHERE created_at < ?")
        .bind(&cutoff)
        .execute(&mut conn)
        .await?;
    conn.close().await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn build_message_record_creates_ulid_and_rfc3339() {
        let record = build_message_record(
            "conv-test".to_string(),
            "user".to_string(),
            "hello".to_string(),
            "online".to_string(),
        )
        .expect("valid input should build");
        assert_eq!(record.id.len(), 26, "ULID should be 26 chars");
        DateTime::parse_from_rfc3339(&record.created_at).expect("created_at must be RFC3339");
        assert_eq!(record.role, "user");
        assert_eq!(record.mode, "online");
    }

    #[test]
    fn build_message_record_rejects_invalid_role() {
        let result = build_message_record(
            "conv".to_string(),
            "admin".to_string(),
            "x".to_string(),
            "online".to_string(),
        );
        assert!(matches!(result, Err(MemoryError::InvalidRole(_, _))));
    }

    #[test]
    fn build_message_record_rejects_invalid_mode() {
        let result = build_message_record(
            "conv".to_string(),
            "user".to_string(),
            "x".to_string(),
            "voice".to_string(),
        );
        assert!(matches!(result, Err(MemoryError::InvalidMode(_, _))));
    }

    #[test]
    fn build_message_record_accepts_all_valid_roles_and_modes() {
        for role in VALID_ROLES {
            for mode in VALID_MODES {
                let r = build_message_record(
                    "c".into(),
                    (*role).into(),
                    "x".into(),
                    (*mode).into(),
                );
                assert!(r.is_ok(), "role={role}, mode={mode} should be accepted");
            }
        }
    }

    #[test]
    fn cutoff_for_days_is_in_the_past() {
        let cutoff = cutoff_for_days(90);
        let parsed: DateTime<Utc> = DateTime::parse_from_rfc3339(&cutoff)
            .expect("cutoff must be RFC3339")
            .with_timezone(&Utc);
        let now = Utc::now();
        let delta = now.signed_duration_since(parsed);
        // 允许 ±1 天误差应对测试调度延迟
        assert!(
            delta.num_days() >= 89 && delta.num_days() <= 91,
            "expected ~90 days; got {} days",
            delta.num_days()
        );
    }

    #[test]
    fn summary_placeholder_is_recognizable() {
        assert!(
            SUMMARY_PLACEHOLDER.contains("M3"),
            "summary placeholder must mention M3 milestone"
        );
        assert!(
            SUMMARY_PLACEHOLDER.contains("占位"),
            "summary placeholder must be self-identifying as stub"
        );
    }
}

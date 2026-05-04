// I.2 配套 SecretsService — secrets 表 CRUD + DPAPI 包装
//
// 范围(plan B.1 T1):
// - set / get / delete 3 个方法,key 命名 `{provider_id}.api_key`(架构 §300)
// - 写入前 crypto::protect → ciphertext 入 secrets 表;读取后 crypto::unprotect → 明文 String
// - delete 幂等(不存在不报错)
//
// Schema(migrations/001_init.sql 行 24-28):
//   CREATE TABLE secrets (
//     key TEXT PRIMARY KEY,
//     ciphertext BLOB NOT NULL,
//     updated_at TEXT NOT NULL
//   );
//
// 设计决定:
// - 沿用 persona/nickname/memory 模式:`AppHandle<R>` + 自开 sqlx 短期连接
//   (tauri-plugin-sql 2.4 的 DbPool 公共方法被注释,无法借用 plugin Pool)
// - DPAPI ciphertext 是 Vec<u8>,SQLite BLOB 直接绑定 — 不做 base64
// - 明文读出后立即返回 String,不在本模块做 zeroize(M3 defense-in-depth 评估;
//   DPAPI 模型已假设进程内存可信,本期保守不引 zeroize crate)
// - 错误透传:DPAPI 失败 / DB 失败分别枚举,不混淆;LlmError 用 #[from] 桥接

use chrono::Utc;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, SqliteConnection};
use std::str::FromStr;
use tauri::{AppHandle, Manager, Runtime};
use thiserror::Error;

use crate::services::crypto::{self, CryptoError};

#[derive(Debug, Error)]
pub enum SecretsError {
    #[error("database error: {0}")]
    Database(String),
    #[error("config dir resolution failed: {0}")]
    AppConfigDir(String),
    #[error("crypto error: {0}")]
    Crypto(#[from] CryptoError),
    #[error("stored ciphertext is not valid utf-8 after decrypt: {0}")]
    NotUtf8(#[source] std::string::FromUtf8Error),
}

impl From<sqlx::Error> for SecretsError {
    fn from(e: sqlx::Error) -> Self {
        SecretsError::Database(e.to_string())
    }
}

/// 把 provider_id 拼成 secrets 表的 key。
///
/// `provider_id` 由调用方保证是 `[a-z0-9_-]+`(来自 PRESETS.id 或 G 设置页校验),
/// 此处不做二次校验避免 false sense of safety。
pub fn provider_key(provider_id: &str) -> String {
    format!("{provider_id}.api_key")
}

async fn open_conn<R: Runtime>(app: &AppHandle<R>) -> Result<SqliteConnection, SecretsError> {
    let app_config = app
        .path()
        .app_config_dir()
        .map_err(|e| SecretsError::AppConfigDir(e.to_string()))?;
    let db_path = app_config.join("aipet.db");
    let db_url = format!("sqlite:{}", db_path.display());
    Ok(SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(false)
        .connect()
        .await?)
}

/// 写入(或覆盖)某 provider 的 api_key。
///
/// `plaintext` 为空字符串时 DPAPI 仍会产出非空 ciphertext(metadata),不报错。
pub async fn set<R: Runtime>(
    app: &AppHandle<R>,
    provider_id: &str,
    plaintext: &str,
) -> Result<(), SecretsError> {
    let key = provider_key(provider_id);
    let ciphertext = crypto::protect(plaintext.as_bytes())?;
    let now = Utc::now().to_rfc3339();

    let mut conn = open_conn(app).await?;
    sqlx::query(
        "INSERT INTO secrets(key, ciphertext, updated_at) VALUES (?, ?, ?) \
         ON CONFLICT(key) DO UPDATE SET ciphertext = excluded.ciphertext, updated_at = excluded.updated_at",
    )
    .bind(&key)
    .bind(&ciphertext)
    .bind(&now)
    .execute(&mut conn)
    .await?;
    conn.close().await?;
    Ok(())
}

/// 读出某 provider 的 api_key(明文)。不存在返回 `Ok(None)` 而不是错误,调用方决定 UX。
pub async fn get<R: Runtime>(
    app: &AppHandle<R>,
    provider_id: &str,
) -> Result<Option<String>, SecretsError> {
    let key = provider_key(provider_id);
    let mut conn = open_conn(app).await?;
    let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT ciphertext FROM secrets WHERE key = ?")
        .bind(&key)
        .fetch_optional(&mut conn)
        .await?;
    conn.close().await?;

    let Some((ciphertext,)) = row else {
        return Ok(None);
    };
    let plain_bytes = crypto::unprotect(&ciphertext)?;
    let plaintext = String::from_utf8(plain_bytes).map_err(SecretsError::NotUtf8)?;
    Ok(Some(plaintext))
}

/// 删除某 provider 的 api_key。不存在视为 no-op(幂等)。
pub async fn delete<R: Runtime>(
    app: &AppHandle<R>,
    provider_id: &str,
) -> Result<(), SecretsError> {
    let key = provider_key(provider_id);
    let mut conn = open_conn(app).await?;
    sqlx::query("DELETE FROM secrets WHERE key = ?")
        .bind(&key)
        .execute(&mut conn)
        .await?;
    conn.close().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== provider_key 命名约定 — 6 preset 全覆盖 =====
    // 防止后续重构改了命名格式但 secrets 表里历史 key 找不到。

    #[test]
    fn provider_key_openai() {
        assert_eq!(provider_key("openai"), "openai.api_key");
    }

    #[test]
    fn provider_key_deepseek() {
        assert_eq!(provider_key("deepseek"), "deepseek.api_key");
    }

    #[test]
    fn provider_key_moonshot() {
        assert_eq!(provider_key("moonshot"), "moonshot.api_key");
    }

    #[test]
    fn provider_key_qwen() {
        assert_eq!(provider_key("qwen"), "qwen.api_key");
    }

    #[test]
    fn provider_key_ollama() {
        // Ollama 本地模式实际不需要 api_key,但 schema 一致性要求仍走 secrets 表
        // (用户可填任意占位字符串;不强制非空)
        assert_eq!(provider_key("ollama"), "ollama.api_key");
    }

    #[test]
    fn provider_key_custom() {
        assert_eq!(provider_key("custom"), "custom.api_key");
    }

    #[test]
    fn provider_key_does_not_collide_across_provider_ids() {
        // 防御:确保不同 provider_id 永远拼出不同的 key
        let keys = ["openai", "deepseek", "moonshot", "qwen", "ollama", "custom"]
            .iter()
            .map(|p| provider_key(p))
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(keys.len(), 6, "all 6 preset keys must be unique");
    }
}

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
    // 用 builder API 避开 URL parsing — Windows 绝对路径 `sqlite:C:\...` 反斜杠会让
    // SqliteConnectOptions::from_str 报 SQLITE_CANTOPEN(code 14)。filename(&PathBuf) 路径
    // 直接传给底层 sqlite3_open_v2,绕开 URL 协议解析。
    Ok(SqliteConnectOptions::new()
        .filename(&db_path)
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
    set_with_conn(&mut conn, &key, &ciphertext, &now).await?;
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
    let row = get_with_conn(&mut conn, &key).await?;
    conn.close().await?;

    let Some(ciphertext) = row else {
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
    delete_with_conn(&mut conn, &key).await?;
    conn.close().await?;
    Ok(())
}

// ============================================================================
// Inner helpers(不依赖 AppHandle / Tauri runtime,接 &mut SqliteConnection)
//
// 抽出动机(2026-05-04 test-coverage):
// - 外层 `set / get / delete<R: Runtime>` 必须接 AppHandle 才能解析 app_config_dir
// - 集成测试用临时 DB,无 AppHandle —— inner 让测试调到与 prod 完全相同的 SQL 路径
// - prod 行为完全等价:外层语义 = open_conn → inner → close_conn(close 顺利不报错前提下)
// ============================================================================

/// SQL 部分:UPSERT secrets 表。`ciphertext` 必须已经过 DPAPI protect。
pub(crate) async fn set_with_conn(
    conn: &mut SqliteConnection,
    key: &str,
    ciphertext: &[u8],
    now_rfc3339: &str,
) -> Result<(), SecretsError> {
    sqlx::query(
        "INSERT INTO secrets(key, ciphertext, updated_at) VALUES (?, ?, ?) \
         ON CONFLICT(key) DO UPDATE SET ciphertext = excluded.ciphertext, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(ciphertext)
    .bind(now_rfc3339)
    .execute(conn)
    .await?;
    Ok(())
}

/// SQL 部分:SELECT 一条 secrets 行的 ciphertext。返回 None 表示 key 不存在。
pub(crate) async fn get_with_conn(
    conn: &mut SqliteConnection,
    key: &str,
) -> Result<Option<Vec<u8>>, SecretsError> {
    let row: Option<(Vec<u8>,)> = sqlx::query_as("SELECT ciphertext FROM secrets WHERE key = ?")
        .bind(key)
        .fetch_optional(conn)
        .await?;
    Ok(row.map(|(c,)| c))
}

/// SQL 部分:DELETE secrets 表。不存在不报错(SQL 层幂等)。
pub(crate) async fn delete_with_conn(
    conn: &mut SqliteConnection,
    key: &str,
) -> Result<(), SecretsError> {
    sqlx::query("DELETE FROM secrets WHERE key = ?")
        .bind(key)
        .execute(conn)
        .await?;
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

    // ===== DB 集成测试(2026-05-04 test-coverage P0)=====
    // 走真实 sqlite + DPAPI 端到端;详 progress/test-coverage-2026-05-04.md
    //
    // 防御目标:secrets 写入路径在 M1 W1 完整周期里一次都没真跑过(plugin preload
    // 缺失 + Win 反斜杠路径双重 bug 导致 db 文件从未创建)。本组测试若早 6 天写,
    // 应能在 H.1 / F.2 完成时立刻红灯并保留地面 bug 不潜伏。

    use crate::services::crypto;
    use crate::services::test_db::fresh_db;
    use chrono::Utc;

    #[tokio::test]
    async fn set_get_delete_roundtrip_via_sql_only() {
        // 不走 DPAPI(让此测试在非 Win 平台 / 缺 DPAPI 上下文也能跑;
        // DPAPI 端到端见下一个测试)
        let (_dir, mut conn) = fresh_db().await;
        let key = "openai.api_key";
        let fake_ciphertext: Vec<u8> = vec![0x01, 0x02, 0x03, 0xff];
        let now = Utc::now().to_rfc3339();

        // 1) 起始无值
        let none = get_with_conn(&mut conn, key).await.expect("get empty");
        assert!(none.is_none(), "fresh DB must have no secret for key");

        // 2) set 后能读回 ciphertext 完全一致
        set_with_conn(&mut conn, key, &fake_ciphertext, &now)
            .await
            .expect("set succeeds");
        let got = get_with_conn(&mut conn, key).await.expect("get after set");
        assert_eq!(
            got.as_deref(),
            Some(fake_ciphertext.as_slice()),
            "ciphertext bytes must roundtrip without mutation"
        );

        // 3) 第二次 set 触发 ON CONFLICT UPDATE 路径(覆盖,不报错)
        let new_ciphertext: Vec<u8> = vec![0x10, 0x20];
        set_with_conn(&mut conn, key, &new_ciphertext, &now)
            .await
            .expect("upsert overwrites");
        let after_upsert = get_with_conn(&mut conn, key)
            .await
            .expect("get after upsert");
        assert_eq!(
            after_upsert.as_deref(),
            Some(new_ciphertext.as_slice()),
            "second set must overwrite, not append"
        );

        // 4) delete 幂等:删了再删不报错,且 get 返回 None
        delete_with_conn(&mut conn, key).await.expect("delete once");
        delete_with_conn(&mut conn, key)
            .await
            .expect("delete twice (idempotent)");
        let after_delete = get_with_conn(&mut conn, key)
            .await
            .expect("get after delete");
        assert!(after_delete.is_none(), "delete must remove the row");
    }

    #[tokio::test]
    async fn dpapi_protect_then_sql_then_unprotect_roundtrip() {
        // 端到端:plaintext → DPAPI protect → SQL insert → SQL select → DPAPI unprotect → plaintext
        // 这正是 prod set / get<R> 的内部链路(只少了 AppHandle 解析 app_config_dir)
        let (_dir, mut conn) = fresh_db().await;
        let plaintext = "sk-test-roundtrip-1234567890abcdef";
        let ciphertext = crypto::protect(plaintext.as_bytes())
            .expect("DPAPI protect under user login token");
        let now = Utc::now().to_rfc3339();
        let key = provider_key("deepseek");

        set_with_conn(&mut conn, &key, &ciphertext, &now)
            .await
            .expect("set ciphertext");
        let stored_bytes = get_with_conn(&mut conn, &key)
            .await
            .expect("get ciphertext")
            .expect("must exist after set");

        let recovered_bytes = crypto::unprotect(&stored_bytes).expect("DPAPI unprotect");
        let recovered = String::from_utf8(recovered_bytes).expect("plaintext is utf-8");
        assert_eq!(
            recovered, plaintext,
            "full DPAPI + SQL roundtrip must preserve api_key plaintext"
        );
    }

    #[tokio::test]
    async fn multiple_providers_dont_collide() {
        // 真实场景:用户同时配 OpenAI + DeepSeek + Moonshot 三个 provider 的 api_key
        let (_dir, mut conn) = fresh_db().await;
        let now = Utc::now().to_rfc3339();
        let cases = [
            ("openai", b"openai-secret".to_vec()),
            ("deepseek", b"deepseek-secret".to_vec()),
            ("moonshot", b"moonshot-secret".to_vec()),
        ];

        // 全部写入
        for (id, ct) in &cases {
            set_with_conn(&mut conn, &provider_key(id), ct, &now)
                .await
                .expect("set succeeds");
        }

        // 各自读回应该独立、互不污染
        for (id, expected) in &cases {
            let got = get_with_conn(&mut conn, &provider_key(id))
                .await
                .expect("get succeeds")
                .expect("row must exist");
            assert_eq!(
                got.as_slice(),
                expected.as_slice(),
                "provider {id} ciphertext must isolate from others"
            );
        }

        // 删一个不影响其余
        delete_with_conn(&mut conn, &provider_key("openai"))
            .await
            .expect("delete openai");
        let openai_after = get_with_conn(&mut conn, &provider_key("openai"))
            .await
            .expect("get after delete");
        assert!(openai_after.is_none());
        let deepseek_after = get_with_conn(&mut conn, &provider_key("deepseek"))
            .await
            .expect("get deepseek after openai delete");
        assert!(
            deepseek_after.is_some(),
            "deleting openai must not affect deepseek"
        );
    }
}

// F.2 NicknameService — 桌宠/用户昵称管理(单行 nicknames 表 facade)
//
// 范围(plan F.2):
// - get_pet / get_user / set_pet / set_user / restore_pet 5 个 service 方法
// - emit `nickname.changed` event(架构 §711 payload: { which: 'pet'|'user', value })
//
// Schema(migrations/001_init.sql 行 68-74)是单行表(id=1 CHECK):
//   pet_nickname TEXT (nullable, NULL 时 fallback 到 active persona 的 name)
//   pet_nickname_previous TEXT (set_pet 时自动存上次值,restore_pet 用)
//   user_nickname TEXT (nullable, 无 fallback,调用方决定 UI 文案)
//   updated_at TEXT NOT NULL
//
// 设计决定:
// - **不引入 memory KV 表**(架构 §249 那是另一面);F.2 直接用 nicknames 表,精确且少一层间接
// - **restore_pet 用原子 swap**(current ↔ previous):用户可来回切看哪个名字更喜欢,UX 最直观
// - get_pet fallback 顺序:nicknames.pet_nickname → active persona.name → "默默" 兜底字符串
// - DB 连接同 persona/memory 模式:自开 sqlx 短期连接

use chrono::Utc;
use serde::Serialize;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, SqliteConnection};
use std::str::FromStr;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use thiserror::Error;

const NICKNAME_CHANGED_EVENT: &str = "nickname.changed";
const FALLBACK_PET_NAME: &str = "默默";

#[derive(Debug, Error)]
pub enum NicknameError {
    #[error("database error: {0}")]
    Database(String),
    #[error("config dir resolution failed: {0}")]
    AppConfigDir(String),
    #[error("event emit failed: {0}")]
    EventEmit(String),
    #[error("nothing to restore (pet_nickname_previous is NULL)")]
    NothingToRestore,
}

impl From<sqlx::Error> for NicknameError {
    fn from(e: sqlx::Error) -> Self {
        NicknameError::Database(e.to_string())
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct NicknameChangedPayload {
    /// 'pet' 或 'user' — 与架构 §711 IPC event 契约一致
    pub which: String,
    /// 新值;None 表示置空(将来 fallback 到默认)
    pub value: Option<String>,
}

async fn open_conn<R: Runtime>(app: &AppHandle<R>) -> Result<SqliteConnection, NicknameError> {
    let app_config = app
        .path()
        .app_config_dir()
        .map_err(|e| NicknameError::AppConfigDir(e.to_string()))?;
    let db_path = app_config.join("aipet.db");
    let db_url = format!("sqlite:{}", db_path.display());
    Ok(SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(false)
        .connect()
        .await?)
}

fn emit_changed<R: Runtime>(
    app: &AppHandle<R>,
    which: &str,
    value: Option<String>,
) -> Result<(), NicknameError> {
    let payload = NicknameChangedPayload {
        which: which.to_string(),
        value,
    };
    app.emit(NICKNAME_CHANGED_EVENT, payload)
        .map_err(|e| NicknameError::EventEmit(e.to_string()))
}

/// 读 pet_nickname。NULL 时 fallback 到当前 active persona 的 name,再无则返回兜底 "默默"。
pub async fn get_pet_nickname<R: Runtime>(app: &AppHandle<R>) -> Result<String, NicknameError> {
    let mut conn = open_conn(app).await?;

    let stored: Option<(Option<String>,)> =
        sqlx::query_as("SELECT pet_nickname FROM nicknames WHERE id = 1")
            .fetch_optional(&mut conn)
            .await?;

    if let Some((Some(name),)) = stored {
        conn.close().await?;
        return Ok(name);
    }

    let persona: Option<(String,)> =
        sqlx::query_as("SELECT name FROM personas WHERE is_active = 1 LIMIT 1")
            .fetch_optional(&mut conn)
            .await?;

    conn.close().await?;
    Ok(persona
        .map(|(n,)| n)
        .unwrap_or_else(|| FALLBACK_PET_NAME.to_string()))
}

/// 读 user_nickname。无 fallback;NULL 时返回 None,调用方决定 UI 文案。
pub async fn get_user_nickname<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<Option<String>, NicknameError> {
    let mut conn = open_conn(app).await?;
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT user_nickname FROM nicknames WHERE id = 1")
            .fetch_optional(&mut conn)
            .await?;
    conn.close().await?;
    Ok(row.and_then(|(v,)| v))
}

/// 设置 pet_nickname。自动把当前值搬到 pet_nickname_previous(为后续 restore 备份)。
/// 触发 `nickname.changed` { which: "pet", value: Some(name) }。
pub async fn set_pet_nickname<R: Runtime>(
    app: &AppHandle<R>,
    name: String,
) -> Result<(), NicknameError> {
    let now = Utc::now().to_rfc3339();
    let mut conn = open_conn(app).await?;

    sqlx::query(
        r#"
        INSERT INTO nicknames (id, pet_nickname, pet_nickname_previous, user_nickname, updated_at)
        VALUES (
            1,
            ?,
            (SELECT pet_nickname FROM nicknames WHERE id = 1),
            (SELECT user_nickname FROM nicknames WHERE id = 1),
            ?
        )
        ON CONFLICT(id) DO UPDATE SET
            pet_nickname_previous = nicknames.pet_nickname,
            pet_nickname = excluded.pet_nickname,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&name)
    .bind(&now)
    .execute(&mut conn)
    .await?;
    conn.close().await?;

    emit_changed(app, "pet", Some(name))?;
    Ok(())
}

/// 设置 user_nickname。
/// 触发 `nickname.changed` { which: "user", value: Some(name) }。
pub async fn set_user_nickname<R: Runtime>(
    app: &AppHandle<R>,
    name: String,
) -> Result<(), NicknameError> {
    let now = Utc::now().to_rfc3339();
    let mut conn = open_conn(app).await?;

    sqlx::query(
        r#"
        INSERT INTO nicknames (id, user_nickname, updated_at)
        VALUES (1, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            user_nickname = excluded.user_nickname,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&name)
    .bind(&now)
    .execute(&mut conn)
    .await?;
    conn.close().await?;

    emit_changed(app, "user", Some(name))?;
    Ok(())
}

/// pet_nickname 与 pet_nickname_previous 原子 swap(用户可来回切两个曾用名)。
/// 若 previous 为 NULL,返回 NothingToRestore 错误,调用方提示 UI。
/// 触发 `nickname.changed` { which: "pet", value: <swap 后的 current> }。
pub async fn restore_pet_nickname<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<Option<String>, NicknameError> {
    let now = Utc::now().to_rfc3339();
    let mut conn = open_conn(app).await?;

    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT pet_nickname, pet_nickname_previous FROM nicknames WHERE id = 1",
    )
    .fetch_optional(&mut conn)
    .await?;

    let (current, previous) = match row {
        Some(t) => t,
        None => {
            conn.close().await?;
            return Err(NicknameError::NothingToRestore);
        }
    };

    if previous.is_none() {
        conn.close().await?;
        return Err(NicknameError::NothingToRestore);
    }

    // SQLite UPDATE 同行 swap:右值取的是行更新前的值,所以 swap 是原子的
    sqlx::query(
        r#"
        UPDATE nicknames
        SET
            pet_nickname = pet_nickname_previous,
            pet_nickname_previous = pet_nickname,
            updated_at = ?
        WHERE id = 1
        "#,
    )
    .bind(&now)
    .execute(&mut conn)
    .await?;
    conn.close().await?;

    // swap 后 current 等于原 previous(肯定 Some,上面已守卫)
    let new_current = previous.clone();
    let _ = current; // 仅为可读性保留旧 current 命名
    emit_changed(app, "pet", new_current.clone())?;
    Ok(new_current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn payload_serializes_to_event_contract() {
        let p = NicknameChangedPayload {
            which: "pet".to_string(),
            value: Some("小默".to_string()),
        };
        let json: Value = serde_json::to_value(&p).expect("payload must serialize");
        // 架构 §711 契约:{ which: 'pet'|'user', value }
        assert_eq!(json["which"], "pet");
        assert_eq!(json["value"], "小默");
    }

    #[test]
    fn payload_serializes_null_value() {
        let p = NicknameChangedPayload {
            which: "user".to_string(),
            value: None,
        };
        let json: Value = serde_json::to_value(&p).expect("payload must serialize");
        assert_eq!(json["which"], "user");
        assert!(json["value"].is_null());
    }

    #[test]
    fn fallback_pet_name_is_momo_default() {
        // 出厂兜底必须与 ADR-009 + persona-design v1.0 §5.1 momo 一致
        assert_eq!(FALLBACK_PET_NAME, "默默");
    }

    #[test]
    fn event_name_matches_arch_contract() {
        // 架构 §711 IPC event 表第 22 行:'nickname.changed'
        assert_eq!(NICKNAME_CHANGED_EVENT, "nickname.changed");
    }
}

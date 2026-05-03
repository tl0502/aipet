// DEV-1 开发面板 IPC commands(debug-only)
//
// 全文 #[cfg(debug_assertions)] — release 构建编译期排除。
//
// 范围(plan DEV-1):
// - dev_list_tables() — 返回当前 sqlite 数据库内所有表名(从 sqlite_master)
// - dev_query_table(name, limit) — 安全特化:仅支持 personas / messages / nicknames /
//   persona_snapshots / conversations 5 张表;白名单防 SQL 注入
// - dev_get_logs() — 占位 stub,返回空数组(待 ringbuffer logger 接入,M1 D6+)
//
// DB 连接同 services/persona / memory / nickname 的 sqlx 短期连接模式。

#![cfg(debug_assertions)]

use serde_json::{json, Value};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, Row, SqliteConnection};
use std::str::FromStr;
use tauri::{AppHandle, Manager, Runtime};

const DEFAULT_LIMIT: u32 = 100;
const ALLOWED_TABLES: &[&str] = &[
    "personas",
    "messages",
    "nicknames",
    "persona_snapshots",
    "conversations",
];

async fn open_conn<R: Runtime>(app: &AppHandle<R>) -> Result<SqliteConnection, String> {
    let app_config = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("config dir: {e}"))?;
    let db_path = app_config.join("aipet.db");
    let db_url = format!("sqlite:{}", db_path.display());
    SqliteConnectOptions::from_str(&db_url)
        .map_err(|e| format!("sqlite options: {e}"))?
        .create_if_missing(false)
        .connect()
        .await
        .map_err(|e| format!("sqlite connect: {e}"))
}

#[tauri::command]
pub async fn dev_list_tables(app: AppHandle) -> Result<Vec<String>, String> {
    let mut conn = open_conn(&app).await?;
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlx_%' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&mut conn)
    .await
    .map_err(|e| e.to_string())?;
    let _ = conn.close().await;
    Ok(rows.into_iter().map(|(n,)| n).collect())
}

#[tauri::command]
pub async fn dev_query_table(
    app: AppHandle,
    table: String,
    limit: Option<u32>,
) -> Result<Vec<Value>, String> {
    if !ALLOWED_TABLES.contains(&table.as_str()) {
        return Err(format!(
            "table '{}' not in allow-list {:?}",
            table, ALLOWED_TABLES
        ));
    }
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(500);

    let mut conn = open_conn(&app).await?;
    // table 已通过 ALLOWED_TABLES 白名单校验,此处字符串拼接安全
    let sql = format!("SELECT * FROM {} LIMIT {}", table, limit);
    let rows = sqlx::query(&sql)
        .fetch_all(&mut conn)
        .await
        .map_err(|e| e.to_string())?;
    let _ = conn.close().await;

    let results: Vec<Value> = rows.iter().map(row_to_json).collect();
    Ok(results)
}

/// 把 sqlx Row 序列化为 JSON Object — 反射 column 类型,支持 TEXT / INTEGER / REAL / BLOB / NULL
fn row_to_json(row: &sqlx::sqlite::SqliteRow) -> Value {
    use sqlx::Column;
    let mut obj = serde_json::Map::new();
    for col in row.columns() {
        let name = col.name();
        // 按可能性逐个尝试 — 失败说明类型不匹配,退回 null
        let v = if let Ok(s) = row.try_get::<Option<String>, _>(name) {
            s.map(Value::from).unwrap_or(Value::Null)
        } else if let Ok(i) = row.try_get::<Option<i64>, _>(name) {
            i.map(|n| json!(n)).unwrap_or(Value::Null)
        } else if let Ok(f) = row.try_get::<Option<f64>, _>(name) {
            f.map(|n| json!(n)).unwrap_or(Value::Null)
        } else if let Ok(b) = row.try_get::<Option<Vec<u8>>, _>(name) {
            // BLOB:返回长度而非内容(避免 dev panel 渲染巨大 byte 数组)
            b.map(|bytes| json!(format!("<BLOB {} bytes>", bytes.len())))
                .unwrap_or(Value::Null)
        } else {
            Value::Null
        };
        obj.insert(name.to_string(), v);
    }
    Value::Object(obj)
}

#[tauri::command]
pub async fn dev_get_logs() -> Result<Vec<String>, String> {
    // MVP 占位:真 ringbuffer logger 待 M1 D6+ 引入 tracing-subscriber 时接入
    Ok(vec![
        "[dev_get_logs] stub — ringbuffer logger 待 M1 D6+ 接入".to_string(),
    ])
}

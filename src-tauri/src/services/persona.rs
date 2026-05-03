// H.1 PersonaService MVP — 加载内置 momo + 写入 SQLite
//
// 范围(plan H.1):
// - parse_persona(&str) 通用解析器(接受任意 .soul.md 字符串,内置 / 用户文件 / 导入复用)
// - seed_builtin(&AppHandle) 启动入口 — 把 include_str! 编译进 binary 的 momo 写入 personas / persona_snapshots
//
// 设计:
// - frontmatter 用 gray_matter (yaml feature) 解析 → 反序列化到 PersonaFrontmatter
// - 必填字段检查(id / name / version / schema_version)用空字符串/0 哨兵 + 显式 MissingField 错误,
//   而不是依赖 serde missing field 错误(那个文案是英文,不便 UI 展示)
// - schema_version 仅接受 1 或 2(persona-design v1.0 §2.3 至少向后兼容前 1 个 schema)
// - markdown 区段不切分,直接把 raw markdown 存 persona_snapshots.content;切分推到 B.2 拼 system prompt 时按需做
//
// DB:
// - tauri-plugin-sql 2.4 的 DbPool 公共方法被注释掉(看 wrapper.rs 行 37-64),Rust 端无法借用 plugin 的 Pool
// - 自己开 sqlx::SqliteConnection 短期连接,做完 drop;DB 路径与 plugin 一致(<app_config>/aipet.db)
// - personas 走 ON CONFLICT(id) DO UPDATE(idempotent),persona_snapshots 走 (persona_id, version) 唯一性守卫

use chrono::Utc;
use gray_matter::engine::YAML;
use gray_matter::Matter;
use serde::Deserialize;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{ConnectOptions, Connection, SqliteConnection, Transaction};
use std::str::FromStr;
use tauri::{AppHandle, Manager, Runtime};
use thiserror::Error;

/// 内置 momo 人格,编译期注入(与 migrations/001_init.sql 的 include_str! 同款)
const MOMO_RAW: &str = include_str!("../../personas/_builtin/momo.soul.md");

/// 内置 file_path 标识 — 用 `<bundled>:` 前缀区别于用户人格(后者填真实 APPDATA 路径)
const MOMO_BUNDLED_PATH: &str = "<bundled>:_builtin/momo.soul.md";

const SUPPORTED_SCHEMAS: &[u32] = &[1, 2];

#[derive(Debug, Deserialize, Default)]
pub struct PersonaFrontmatter {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    // MVP 只消费上述 4 个必填字段;author / voice_pack / accessories / tone_profile 等会被
    // gray_matter Pod 解析,但 PersonaFrontmatter struct 不声明这些字段 —— serde 默认忽略多余字段。
    // H.2/H.3 GUI 工坊接入时再扩展 struct。
}

#[derive(Debug)]
pub struct ParsedPersona {
    pub frontmatter: PersonaFrontmatter,
    pub raw_markdown: String,
}

#[derive(Debug, Error)]
pub enum PersonaError {
    #[error("frontmatter parse failed: {0}")]
    FrontmatterParse(String),
    #[error("schema_version unsupported: got {0}, want one of {1:?}")]
    UnsupportedSchema(u32, &'static [u32]),
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("database error: {0}")]
    Database(String),
    #[error("config dir resolution failed: {0}")]
    AppConfigDir(String),
}

impl From<sqlx::Error> for PersonaError {
    fn from(e: sqlx::Error) -> Self {
        PersonaError::Database(e.to_string())
    }
}

/// 解析任意 .soul.md 字符串为 ParsedPersona。
///
/// 不依赖 IO — 内置走 const,用户走 fs::read_to_string,后续 H.2 import 走拖拽 payload。
pub fn parse_persona(content: &str) -> Result<ParsedPersona, PersonaError> {
    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(content);

    let pod = parsed.data.ok_or_else(|| {
        PersonaError::FrontmatterParse("missing frontmatter block (no `---` delimiters?)".into())
    })?;

    let frontmatter: PersonaFrontmatter = pod
        .deserialize()
        .map_err(|e| PersonaError::FrontmatterParse(e.to_string()))?;

    if frontmatter.schema_version == 0 {
        return Err(PersonaError::MissingField("schema_version"));
    }
    if !SUPPORTED_SCHEMAS.contains(&frontmatter.schema_version) {
        return Err(PersonaError::UnsupportedSchema(
            frontmatter.schema_version,
            SUPPORTED_SCHEMAS,
        ));
    }
    if frontmatter.id.is_empty() {
        return Err(PersonaError::MissingField("id"));
    }
    if frontmatter.name.is_empty() {
        return Err(PersonaError::MissingField("name"));
    }
    if frontmatter.version.is_empty() {
        return Err(PersonaError::MissingField("version"));
    }

    Ok(ParsedPersona {
        frontmatter,
        raw_markdown: parsed.content,
    })
}

/// 启动入口:解析内置 momo → UPSERT personas + idempotent INSERT persona_snapshots
///
/// 调用方在 lib.rs setup 阶段用 `tauri::async_runtime::spawn` 异步跑,失败仅 eprintln 到 stderr
/// (MVP 期不阻塞启动 / 不弹错误 UI;H.2 引入 IPC 后可考虑前端反馈)
pub async fn seed_builtin<R: Runtime>(app: &AppHandle<R>) -> Result<(), PersonaError> {
    let parsed = parse_persona(MOMO_RAW)?;

    let app_config = app
        .path()
        .app_config_dir()
        .map_err(|e| PersonaError::AppConfigDir(e.to_string()))?;
    let db_path = app_config.join("aipet.db");
    let db_url = format!("sqlite:{}", db_path.display());

    let mut conn = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(false) // plugin 已建,不重复
        .connect()
        .await?;

    let mut tx = conn.begin().await?;
    upsert_persona(&mut tx, &parsed, "builtin", MOMO_BUNDLED_PATH).await?;
    insert_snapshot_if_new(&mut tx, &parsed.frontmatter.id, &parsed).await?;
    tx.commit().await?;

    conn.close().await?;
    Ok(())
}

async fn upsert_persona(
    tx: &mut Transaction<'_, sqlx::Sqlite>,
    parsed: &ParsedPersona,
    source: &str,
    file_path: &str,
) -> Result<(), PersonaError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO personas (id, name, version, source, file_path, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 1, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            version = excluded.version,
            file_path = excluded.file_path,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(&parsed.frontmatter.id)
    .bind(&parsed.frontmatter.name)
    .bind(&parsed.frontmatter.version)
    .bind(source)
    .bind(file_path)
    .bind(&now)
    .bind(&now)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

async fn insert_snapshot_if_new(
    tx: &mut Transaction<'_, sqlx::Sqlite>,
    persona_id: &str,
    parsed: &ParsedPersona,
) -> Result<(), PersonaError> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO persona_snapshots (persona_id, version, content, created_at)
        VALUES (?, ?, ?, ?)
        ON CONFLICT(persona_id, version) DO NOTHING
        "#,
    )
    .bind(persona_id)
    .bind(&parsed.frontmatter.version)
    .bind(&parsed.raw_markdown)
    .bind(&now)
    .execute(tx.as_mut())
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_momo_succeeds() {
        let parsed = parse_persona(MOMO_RAW).expect("momo should parse");
        assert_eq!(parsed.frontmatter.id, "momo");
        assert_eq!(parsed.frontmatter.name, "默默");
        assert_eq!(parsed.frontmatter.version, "1.0.0");
        assert_eq!(parsed.frontmatter.schema_version, 2);
        assert!(!parsed.raw_markdown.is_empty());
    }

    #[test]
    fn parse_invalid_yaml_fails() {
        let bad = "---\nid: : : broken\n---\n# 身份\nx";
        let result = parse_persona(bad);
        assert!(matches!(result, Err(PersonaError::FrontmatterParse(_))));
    }

    #[test]
    fn parse_missing_id_fails() {
        let no_id = "---\nschema_version: 2\nname: 默默\nversion: 1.0.0\n---\n# 身份\nx";
        let result = parse_persona(no_id);
        assert!(matches!(result, Err(PersonaError::MissingField("id"))));
    }

    #[test]
    fn parse_unsupported_schema_fails() {
        let future = "---\nschema_version: 999\nid: x\nname: x\nversion: 1.0.0\n---\n# 身份\nx";
        let result = parse_persona(future);
        assert!(matches!(
            result,
            Err(PersonaError::UnsupportedSchema(999, _))
        ));
    }

    #[test]
    fn parse_schema_v1_compatible() {
        let v1 = "---\nschema_version: 1\nid: legacy\nname: 老人格\nversion: 0.9.0\n---\n# 身份\nx";
        let parsed = parse_persona(v1).expect("schema v1 must still parse");
        assert_eq!(parsed.frontmatter.schema_version, 1);
        assert_eq!(parsed.frontmatter.id, "legacy");
    }

    #[test]
    fn parse_strips_frontmatter_from_raw() {
        let parsed = parse_persona(MOMO_RAW).expect("momo should parse");
        assert!(
            !parsed.raw_markdown.contains("schema_version:"),
            "raw_markdown must not contain frontmatter"
        );
        assert!(
            parsed.raw_markdown.contains("# 身份"),
            "raw_markdown must contain # 身份 heading"
        );
    }
}

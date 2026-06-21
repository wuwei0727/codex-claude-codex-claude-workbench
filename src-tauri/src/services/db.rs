use std::fs;

use chrono::Utc;
use rusqlite::{params, Connection, Result};

use crate::services::app_paths;

const INIT_SQL: &str = include_str!("../../migrations/001_init.sql");

pub fn open_connection() -> Result<Connection> {
    let db_path = app_paths::database_path();

    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| rusqlite::Error::ToSqlConversionFailure(err.into()))?;
    }

    let conn = Connection::open(db_path)?;
    conn.execute_batch(INIT_SQL)?;
    ensure_schema_upgrades(&conn)?;
    seed_default_profiles(&conn)?;
    Ok(conn)
}

fn ensure_schema_upgrades(conn: &Connection) -> Result<()> {
    let columns = conn
        .prepare("PRAGMA table_info(provider_profiles)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>>>()?;

    if !columns.iter().any(|column| column == "settings_config") {
        conn.execute(
            "ALTER TABLE provider_profiles ADD COLUMN settings_config TEXT NOT NULL DEFAULT '{}'",
            [],
        )?;
    }

    if !columns.iter().any(|column| column == "category") {
        conn.execute("ALTER TABLE provider_profiles ADD COLUMN category TEXT", [])?;
    }

    if !columns.iter().any(|column| column == "meta") {
        conn.execute(
            "ALTER TABLE provider_profiles ADD COLUMN meta TEXT NOT NULL DEFAULT '{}'",
            [],
        )?;
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS provider_secrets (
          provider_id TEXT NOT NULL,
          secret_type TEXT NOT NULL,
          encrypted_value BLOB NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY (provider_id, secret_type)
        )",
        [],
    )?;

    Ok(())
}

fn seed_default_profiles(conn: &Connection) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR IGNORE INTO provider_profiles
         (id, kind, name, mode, base_url, settings_config, category, meta, is_active, health, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            "codex-official",
            "codex",
            "Codex Official",
            "official",
            Option::<String>::None,
            r#"{"auth":null,"config":null,"preserveLive":true}"#,
            "official",
            r#"{"liveConfigManaged":false}"#,
            1,
            "unknown",
            now
        ],
    )?;
    conn.execute(
        "UPDATE provider_profiles
         SET settings_config = ?1, category = ?2, meta = ?3
         WHERE id = ?4 AND settings_config = '{}'",
        params![
            r#"{"auth":null,"config":null,"preserveLive":true}"#,
            "official",
            r#"{"liveConfigManaged":false}"#,
            "codex-official"
        ],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO provider_profiles
         (id, kind, name, mode, base_url, settings_config, category, meta, is_active, health, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            "claude-official",
            "claude",
            "Claude Official",
            "official",
            Option::<String>::None,
            r#"{"preserveLive":true}"#,
            "official",
            r#"{"liveConfigManaged":false}"#,
            1,
            "unknown",
            Utc::now().to_rfc3339()
        ],
    )?;
    conn.execute(
        "UPDATE provider_profiles
         SET settings_config = ?1, category = ?2, meta = ?3
         WHERE id = ?4 AND settings_config = '{}'",
        params![
            r#"{"preserveLive":true}"#,
            "official",
            r#"{"liveConfigManaged":false}"#,
            "claude-official"
        ],
    )?;

    Ok(())
}

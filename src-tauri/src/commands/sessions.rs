use serde::Serialize;

use crate::services::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexSession {
    id: String,
    title: String,
    project_path: Option<String>,
    updated_at: String,
    message_count: u32,
}

#[tauri::command]
pub fn list_codex_sessions() -> Result<Vec<CodexSession>, String> {
    let conn = db::open_connection().map_err(|err| err.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, project_path, updated_at, message_count
             FROM codex_sessions
             ORDER BY updated_at DESC",
        )
        .map_err(|err| err.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(CodexSession {
                id: row.get(0)?,
                title: row.get(1)?,
                project_path: row.get(2)?,
                updated_at: row.get(3)?,
                message_count: row.get::<_, i64>(4)? as u32,
            })
        })
        .map_err(|err| err.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())
}

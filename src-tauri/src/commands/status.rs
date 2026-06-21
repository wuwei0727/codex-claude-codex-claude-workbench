use serde::Serialize;

use crate::services::app_paths;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkbenchStatus {
    app_data_dir: String,
    codex_config_path: String,
    claude_config_path: String,
    database_path: String,
    injector_enabled: bool,
}

#[tauri::command]
pub fn get_workbench_status() -> WorkbenchStatus {
    WorkbenchStatus {
        app_data_dir: app_paths::app_data_dir().to_string_lossy().to_string(),
        codex_config_path: app_paths::codex_config_path().to_string_lossy().to_string(),
        claude_config_path: app_paths::claude_config_path()
            .to_string_lossy()
            .to_string(),
        database_path: app_paths::database_path().to_string_lossy().to_string(),
        injector_enabled: false,
    }
}

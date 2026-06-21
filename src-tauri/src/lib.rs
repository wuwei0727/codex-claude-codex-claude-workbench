pub mod commands;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(err) = services::db::open_connection() {
        eprintln!("failed to initialize database: {err}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::status::get_workbench_status,
            commands::providers::apply_provider_switch,
            commands::providers::create_api_relay_provider,
            commands::providers::import_live_provider,
            commands::providers::list_provider_profiles,
            commands::providers::preview_provider_switch,
            commands::providers::switch_provider_profile,
            commands::sessions::list_codex_sessions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

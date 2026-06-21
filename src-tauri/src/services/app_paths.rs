use std::path::PathBuf;

pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| home_dir().join("AppData").join("Roaming"))
        .join("codex-claude-workbench")
}

pub fn codex_config_path() -> PathBuf {
    home_dir().join(".codex").join("config.toml")
}

pub fn codex_auth_path() -> PathBuf {
    home_dir().join(".codex").join("auth.json")
}

pub fn claude_config_path() -> PathBuf {
    home_dir().join(".claude.json")
}

pub fn claude_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| home_dir().join("AppData").join("Roaming"))
        .join("Claude")
}

pub fn database_path() -> PathBuf {
    app_data_dir().join("workbench.sqlite")
}

pub fn backups_dir() -> PathBuf {
    app_data_dir().join("backups")
}

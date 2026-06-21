use std::fs;

use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::services::{
    app_paths, backup_service, config_discovery, credential_service, db, live_config,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderProfile {
    id: String,
    kind: String,
    name: String,
    mode: String,
    base_url: Option<String>,
    category: Option<String>,
    meta: Value,
    has_live_config: bool,
    is_active: bool,
    health: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSwitchPreview {
    profile_id: String,
    profile_name: String,
    provider_kind: String,
    mode: String,
    will_write: bool,
    discovered_configs: Vec<config_discovery::ConfigDiscoveryItem>,
    backup_plan: backup_service::BackupPlan,
    changes: Vec<PreviewChange>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSwitchApplyResult {
    profile_id: String,
    profile_name: String,
    provider_kind: String,
    backup_results: Vec<backup_service::BackupResult>,
    written_paths: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiRelayProviderRequest {
    kind: String,
    name: String,
    base_url: String,
    model: Option<String>,
    api_key: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderMutationResult {
    profile_id: String,
    profile_name: String,
    provider_kind: String,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSecretStatus {
    secret_type: String,
    label: String,
    exists: bool,
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProviderSecretRequest {
    profile_id: String,
    secret_type: String,
    secret_value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSecretMutationResult {
    profile_id: String,
    secret_type: String,
    label: String,
    exists: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewChange {
    target: String,
    path: String,
    field: String,
    action: String,
    sensitive_value: bool,
}

#[tauri::command]
pub fn list_provider_profiles() -> Result<Vec<ProviderProfile>, String> {
    let conn = db::open_connection().map_err(|err| err.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, kind, name, mode, base_url, settings_config, category, meta, is_active, health, updated_at
             FROM provider_profiles
             ORDER BY kind, is_active DESC, name",
        )
        .map_err(|err| err.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ProviderProfile {
                id: row.get(0)?,
                kind: row.get(1)?,
                name: row.get(2)?,
                mode: row.get(3)?,
                base_url: row.get(4)?,
                category: row.get(6)?,
                meta: parse_json_or_empty(row.get::<_, String>(7)?),
                has_live_config: row
                    .get::<_, String>(5)
                    .ok()
                    .and_then(|text| serde_json::from_str::<Value>(&text).ok())
                    .is_some_and(|settings_config| {
                        let provider = live_config::ProviderLiveConfig {
                            id: row.get(0).unwrap_or_default(),
                            kind: row.get(1).unwrap_or_default(),
                            name: row.get(2).unwrap_or_default(),
                            mode: row.get(3).unwrap_or_default(),
                            base_url: row.get(4).ok().flatten(),
                            settings_config,
                            category: row.get(6).ok().flatten(),
                            codex_config_toml_secret: None,
                            codex_auth_json_secret: None,
                            claude_config_json_secret: None,
                        };
                        live_config::provider_has_live_write(&provider)
                    }),
                is_active: row.get::<_, i64>(8)? == 1,
                health: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|err| err.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn switch_provider_profile(profile_id: String) -> Result<(), String> {
    let mut conn = db::open_connection().map_err(|err| err.to_string())?;
    let tx = conn.transaction().map_err(|err| err.to_string())?;

    let kind: String = tx
        .query_row(
            "SELECT kind FROM provider_profiles WHERE id = ?1",
            [&profile_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;

    tx.execute(
        "UPDATE provider_profiles SET is_active = 0 WHERE kind = ?1",
        [&kind],
    )
    .map_err(|err| err.to_string())?;
    tx.execute(
        "UPDATE provider_profiles SET is_active = 1 WHERE id = ?1",
        [&profile_id],
    )
    .map_err(|err| err.to_string())?;
    tx.commit().map_err(|err| err.to_string())
}

#[tauri::command]
pub fn preview_provider_switch(profile_id: String) -> Result<ProviderSwitchPreview, String> {
    let provider = load_live_provider(&profile_id)?;
    let discovered_configs = config_discovery::discover_configs();
    let backup_plan = backup_service::build_backup_plan_for_ids(
        &discovered_configs,
        backup_item_ids(&provider.kind),
    );
    let changes = preview_changes(&provider, &discovered_configs);
    let will_write = live_config::provider_has_live_write(&provider);
    let warnings = vec![
        "这是 dry-run 预览，不会写入 Codex 或 Claude 真实配置；应用切换时会先备份再写入"
            .to_string(),
        "敏感字段只做存在性/标记检测，不返回真实值".to_string(),
        "托管的 auth/token 配置使用 Windows DPAPI 密文保存，应用时解密写回".to_string(),
    ];

    Ok(ProviderSwitchPreview {
        profile_id: provider.id,
        profile_name: provider.name,
        provider_kind: provider.kind,
        mode: provider.mode,
        will_write,
        discovered_configs,
        backup_plan,
        changes,
        warnings,
    })
}

#[tauri::command]
pub fn apply_provider_switch(profile_id: String) -> Result<ProviderSwitchApplyResult, String> {
    let provider = load_live_provider(&profile_id)?;
    let discovered_configs = config_discovery::discover_configs();
    let backup_plan = backup_service::build_backup_plan_for_ids(
        &discovered_configs,
        backup_item_ids(&provider.kind),
    );
    let backup_results = backup_service::execute_backup_plan(&backup_plan)?;
    let live_result = live_config::write_provider_live_config(&provider)?;

    let mut conn = db::open_connection().map_err(|err| err.to_string())?;
    let tx = conn.transaction().map_err(|err| err.to_string())?;
    tx.execute(
        "UPDATE provider_profiles SET is_active = 0 WHERE kind = ?1",
        [&provider.kind],
    )
    .map_err(|err| err.to_string())?;
    tx.execute(
        "UPDATE provider_profiles SET is_active = 1, updated_at = ?2 WHERE id = ?1",
        [&provider.id, &Utc::now().to_rfc3339()],
    )
    .map_err(|err| err.to_string())?;
    tx.commit().map_err(|err| err.to_string())?;

    Ok(ProviderSwitchApplyResult {
        profile_id: provider.id,
        profile_name: provider.name,
        provider_kind: provider.kind,
        backup_results,
        written_paths: live_result.written_paths,
        warnings: live_result.warnings,
    })
}

#[tauri::command]
pub fn import_live_provider(kind: String) -> Result<ProviderMutationResult, String> {
    let kind = validate_kind(&kind)?;
    let now = Utc::now().to_rfc3339();
    let mut warnings = Vec::new();

    let (profile_id, profile_name, mode, base_url, settings_config, category, meta) = if kind
        == "codex"
    {
        let path = app_paths::codex_config_path();
        if !path.exists() {
            return Err(format!("Codex config.toml 不存在: {}", path.display()));
        }

        let raw_config = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let (config, stripped) = strip_sensitive_toml_lines(&raw_config);
        if stripped {
            warnings
                .push("已跳过 config.toml 中疑似 token/key 的敏感行，未写入 SQLite".to_string());
        }
        let auth_path = app_paths::codex_auth_path();
        let has_auth = auth_path.exists();

        (
            format!("codex-live-{}", Utc::now().timestamp_millis()),
            "Codex Live Import".to_string(),
            "official".to_string(),
            extract_base_url_hint(&config),
            json!({
                "auth": null,
                "config": config,
                "importedFrom": path.to_string_lossy(),
                "sensitiveStripped": stripped,
                "secrets": {
                    "codexConfigToml": true,
                    "codexAuthJson": has_auth
                }
            }),
            Some("imported".to_string()),
            json!({
                "liveConfigManaged": true,
                "importedAt": now,
                "secretStorage": "windows-dpapi"
            }),
        )
    } else {
        let path = app_paths::claude_config_path();
        if !path.exists() {
            return Err(format!("Claude 配置不存在: {}", path.display()));
        }

        let raw_config = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let mut value =
            serde_json::from_str::<Value>(&raw_config).map_err(|err| err.to_string())?;
        let stripped = strip_sensitive_json_values(&mut value);
        if stripped {
            warnings.push(
                "已移除 Claude JSON 中疑似 token/key/cookie 的敏感字段，未写入 SQLite".to_string(),
            );
        }

        (
            format!("claude-live-{}", Utc::now().timestamp_millis()),
            "Claude Live Import".to_string(),
            "official".to_string(),
            None,
            value,
            Some("imported".to_string()),
            json!({
                "liveConfigManaged": true,
                "importedAt": now,
                "secretStorage": "windows-dpapi",
                "secrets": {
                    "claudeConfigJson": true
                }
            }),
        )
    };

    insert_provider(
        &profile_id,
        &kind,
        &profile_name,
        &mode,
        base_url.as_deref(),
        &settings_config,
        category.as_deref(),
        &meta,
    )?;

    if kind == "codex" {
        let config_path = app_paths::codex_config_path();
        let config_bytes = fs::read(&config_path).map_err(|err| err.to_string())?;
        save_provider_secret(&profile_id, "codex_config_toml", &config_bytes)?;

        let auth_path = app_paths::codex_auth_path();
        if auth_path.exists() {
            let auth_bytes = fs::read(&auth_path).map_err(|err| err.to_string())?;
            save_provider_secret(&profile_id, "codex_auth_json", &auth_bytes)?;
        }
    } else {
        let config_path = app_paths::claude_config_path();
        let config_bytes = fs::read(&config_path).map_err(|err| err.to_string())?;
        save_provider_secret(&profile_id, "claude_config_json", &config_bytes)?;
    }

    Ok(ProviderMutationResult {
        profile_id,
        profile_name,
        provider_kind: kind,
        warnings,
    })
}

#[tauri::command]
pub fn create_api_relay_provider(
    request: CreateApiRelayProviderRequest,
) -> Result<ProviderMutationResult, String> {
    let kind = validate_kind(&request.kind)?;
    let name = request.name.trim();
    let base_url = request.base_url.trim().trim_end_matches('/');
    let model = request
        .model
        .as_deref()
        .map(str::trim)
        .filter(|item| !item.is_empty());
    let api_key = request
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|item| !item.is_empty());

    if name.is_empty() {
        return Err("Provider 名称不能为空".to_string());
    }
    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        return Err("Base URL 必须以 http:// 或 https:// 开头".to_string());
    }

    let profile_id = format!(
        "{}-relay-{}-{}",
        kind,
        sanitize_id_fragment(name),
        Utc::now().timestamp_millis()
    );
    let settings_config = if kind == "codex" {
        json!({
            "auth": null,
            "config": build_codex_relay_config(&profile_id, name, base_url, model, None),
            "secrets": {
                "codexConfigToml": api_key.is_some()
            }
        })
    } else {
        let mut env = serde_json::Map::new();
        env.insert(
            "ANTHROPIC_BASE_URL".to_string(),
            Value::String(base_url.to_string()),
        );
        if let Some(model) = model {
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                Value::String(model.to_string()),
            );
        }
        json!({ "env": env })
    };
    let meta = json!({
        "liveConfigManaged": true,
        "createdAt": Utc::now().to_rfc3339(),
        "credentialPolicy": if api_key.is_some() { "windows-dpapi" } else { "no-secret-storage" }
    });

    insert_provider(
        &profile_id,
        &kind,
        name,
        "api-relay",
        Some(base_url),
        &settings_config,
        Some("custom"),
        &meta,
    )?;

    if let Some(api_key) = api_key {
        if kind == "codex" {
            let full_config =
                build_codex_relay_config(&profile_id, name, base_url, model, Some(api_key));
            save_provider_secret(&profile_id, "codex_config_toml", full_config.as_bytes())?;
        } else {
            let mut env = serde_json::Map::new();
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                Value::String(base_url.to_string()),
            );
            env.insert(
                "ANTHROPIC_API_KEY".to_string(),
                Value::String(api_key.to_string()),
            );
            if let Some(model) = model {
                env.insert(
                    "ANTHROPIC_MODEL".to_string(),
                    Value::String(model.to_string()),
                );
            }
            let full_config = json!({ "env": env });
            save_provider_secret(
                &profile_id,
                "claude_config_json",
                serde_json::to_string(&full_config)
                    .map_err(|err| err.to_string())?
                    .as_bytes(),
            )?;
        }
    }

    Ok(ProviderMutationResult {
        profile_id,
        profile_name: name.to_string(),
        provider_kind: kind,
        warnings: if api_key.is_some() {
            vec!["API key 已用 Windows DPAPI 加密托管，不会明文写入 SQLite 或前端".to_string()]
        } else {
            vec!["未填写 API key；如 relay 需要鉴权，请后续补充凭据或使用环境变量".to_string()]
        },
    })
}

#[tauri::command]
pub fn list_provider_backups() -> Result<Vec<backup_service::BackupEntry>, String> {
    backup_service::list_backup_entries(&config_discovery::discover_configs())
}

#[tauri::command]
pub fn restore_provider_backup(
    backup_path: String,
) -> Result<backup_service::BackupRestoreResult, String> {
    backup_service::restore_backup(&config_discovery::discover_configs(), &backup_path)
}

#[tauri::command]
pub fn list_provider_secret_status(
    profile_id: String,
) -> Result<Vec<ProviderSecretStatus>, String> {
    let kind = load_provider_kind(&profile_id)?;
    let conn = db::open_connection().map_err(|err| err.to_string())?;

    supported_secret_types(&kind)
        .into_iter()
        .map(|(secret_type, label)| {
            let updated_at = conn
                .query_row(
                    "SELECT updated_at FROM provider_secrets WHERE provider_id = ?1 AND secret_type = ?2",
                    params![&profile_id, secret_type],
                    |row| row.get::<_, String>(0),
                )
                .ok();

            Ok(ProviderSecretStatus {
                secret_type: secret_type.to_string(),
                label: label.to_string(),
                exists: updated_at.is_some(),
                updated_at,
            })
        })
        .collect()
}

#[tauri::command]
pub fn update_provider_secret(
    request: UpdateProviderSecretRequest,
) -> Result<ProviderSecretMutationResult, String> {
    let kind = load_provider_kind(&request.profile_id)?;
    let label = validate_secret_type_for_kind(&kind, &request.secret_type)?;
    let secret_value = request.secret_value.trim();
    if secret_value.is_empty() {
        return Err("凭据内容不能为空".to_string());
    }
    validate_secret_payload(&request.secret_type, secret_value)?;

    save_provider_secret(
        &request.profile_id,
        &request.secret_type,
        secret_value.as_bytes(),
    )?;

    Ok(ProviderSecretMutationResult {
        profile_id: request.profile_id,
        secret_type: request.secret_type,
        label: label.to_string(),
        exists: true,
    })
}

#[tauri::command]
pub fn delete_provider_secret(
    profile_id: String,
    secret_type: String,
) -> Result<ProviderSecretMutationResult, String> {
    let kind = load_provider_kind(&profile_id)?;
    let label = validate_secret_type_for_kind(&kind, &secret_type)?;
    let conn = db::open_connection().map_err(|err| err.to_string())?;
    conn.execute(
        "DELETE FROM provider_secrets WHERE provider_id = ?1 AND secret_type = ?2",
        params![&profile_id, &secret_type],
    )
    .map_err(|err| err.to_string())?;

    Ok(ProviderSecretMutationResult {
        profile_id,
        secret_type,
        label: label.to_string(),
        exists: false,
    })
}

fn preview_changes(
    provider: &live_config::ProviderLiveConfig,
    discovered_configs: &[config_discovery::ConfigDiscoveryItem],
) -> Vec<PreviewChange> {
    let mut changes = Vec::new();

    if provider.kind == "codex" {
        add_change(
            &mut changes,
            discovered_configs,
            "codex-config",
            "provider selection",
            format!("切换 Codex Provider 到 {} 模式", provider.mode),
            false,
        );
        add_change(
            &mut changes,
            discovered_configs,
            "codex-auth",
            "auth state",
            "仅检查官方登录状态，不返回 auth 内容".to_string(),
            true,
        );
    }

    if provider.kind == "claude" {
        add_change(
            &mut changes,
            discovered_configs,
            "claude-config",
            "provider selection",
            format!("切换 Claude Provider 到 {} 模式", provider.mode),
            false,
        );
        add_change(
            &mut changes,
            discovered_configs,
            "claude-app-data",
            "app data",
            "仅检查 Claude 应用数据目录是否存在".to_string(),
            true,
        );
    }

    changes.push(PreviewChange {
        target: "Workbench SQLite".to_string(),
        path: "provider_profiles".to_string(),
        field: "is_active".to_string(),
        action: "预览更新工作台内部 active Provider 状态".to_string(),
        sensitive_value: false,
    });

    changes
}

fn load_live_provider(profile_id: &str) -> Result<live_config::ProviderLiveConfig, String> {
    let conn = db::open_connection().map_err(|err| err.to_string())?;
    conn.query_row(
        "SELECT id, kind, name, mode, base_url, settings_config, category
         FROM provider_profiles WHERE id = ?1",
        [profile_id],
        |row| {
            let settings_text: String = row.get(5)?;
            Ok(live_config::ProviderLiveConfig {
                id: row.get(0)?,
                kind: row.get(1)?,
                name: row.get(2)?,
                mode: row.get(3)?,
                base_url: row.get(4)?,
                settings_config: serde_json::from_str(&settings_text)
                    .unwrap_or_else(|_| Value::Object(Default::default())),
                category: row.get(6)?,
                codex_config_toml_secret: load_provider_secret(profile_id, "codex_config_toml")
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok()),
                codex_auth_json_secret: load_provider_secret(profile_id, "codex_auth_json")
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok()),
                claude_config_json_secret: load_provider_secret(profile_id, "claude_config_json")
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok()),
            })
        },
    )
    .map_err(|err| err.to_string())
}

fn backup_item_ids(provider_kind: &str) -> &'static [&'static str] {
    match provider_kind {
        "codex" => &["codex-config", "codex-auth"],
        "claude" => &["claude-config"],
        _ => &[],
    }
}

fn parse_json_or_empty(text: String) -> Value {
    serde_json::from_str(&text).unwrap_or_else(|_| Value::Object(Default::default()))
}

fn insert_provider(
    id: &str,
    kind: &str,
    name: &str,
    mode: &str,
    base_url: Option<&str>,
    settings_config: &Value,
    category: Option<&str>,
    meta: &Value,
) -> Result<(), String> {
    let conn = db::open_connection().map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO provider_profiles
         (id, kind, name, mode, base_url, settings_config, category, meta, is_active, health, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, 'unknown', ?9)",
        params![
            id,
            kind,
            name,
            mode,
            base_url,
            serde_json::to_string(settings_config).map_err(|err| err.to_string())?,
            category,
            serde_json::to_string(meta).map_err(|err| err.to_string())?,
            Utc::now().to_rfc3339()
        ],
    )
    .map_err(|err| err.to_string())?;

    Ok(())
}

fn save_provider_secret(
    provider_id: &str,
    secret_type: &str,
    plain_value: &[u8],
) -> Result<(), String> {
    let encrypted_value = credential_service::protect(plain_value)?;
    let conn = db::open_connection().map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO provider_secrets (provider_id, secret_type, encrypted_value, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(provider_id, secret_type) DO UPDATE SET
           encrypted_value = excluded.encrypted_value,
           updated_at = excluded.updated_at",
        params![
            provider_id,
            secret_type,
            encrypted_value,
            Utc::now().to_rfc3339()
        ],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn load_provider_secret(provider_id: &str, secret_type: &str) -> Result<Vec<u8>, String> {
    let conn = db::open_connection().map_err(|err| err.to_string())?;
    let encrypted_value = conn
        .query_row(
            "SELECT encrypted_value FROM provider_secrets WHERE provider_id = ?1 AND secret_type = ?2",
            params![provider_id, secret_type],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .map_err(|err| err.to_string())?;

    credential_service::unprotect(&encrypted_value)
}

fn load_provider_kind(profile_id: &str) -> Result<String, String> {
    let conn = db::open_connection().map_err(|err| err.to_string())?;
    conn.query_row(
        "SELECT kind FROM provider_profiles WHERE id = ?1",
        [profile_id],
        |row| row.get::<_, String>(0),
    )
    .map_err(|err| err.to_string())
}

fn supported_secret_types(kind: &str) -> Vec<(&'static str, &'static str)> {
    match kind {
        "codex" => vec![
            ("codex_config_toml", "Codex config.toml"),
            ("codex_auth_json", "Codex auth.json"),
        ],
        "claude" => vec![("claude_config_json", "Claude .claude.json")],
        _ => Vec::new(),
    }
}

fn validate_secret_type_for_kind(kind: &str, secret_type: &str) -> Result<&'static str, String> {
    supported_secret_types(kind)
        .into_iter()
        .find_map(|(candidate, label)| {
            if candidate == secret_type {
                Some(label)
            } else {
                None
            }
        })
        .ok_or_else(|| format!("该 Provider 不支持凭据类型: {secret_type}"))
}

fn validate_secret_payload(secret_type: &str, value: &str) -> Result<(), String> {
    if secret_type.ends_with("_json") {
        serde_json::from_str::<Value>(value).map_err(|err| format!("JSON 凭据格式无效: {err}"))?;
    }

    Ok(())
}

fn validate_kind(kind: &str) -> Result<String, String> {
    match kind.trim() {
        "codex" => Ok("codex".to_string()),
        "claude" => Ok("claude".to_string()),
        other => Err(format!("不支持的 Provider 类型: {other}")),
    }
}

fn build_codex_relay_config(
    profile_id: &str,
    name: &str,
    base_url: &str,
    model: Option<&str>,
    api_key: Option<&str>,
) -> String {
    let provider_key = format!("workbench_{}", sanitize_id_fragment(profile_id));
    let mut config = String::new();
    config.push_str(&format!("model_provider = \"{provider_key}\"\n"));
    if let Some(model) = model {
        config.push_str(&format!("model = \"{}\"\n", escape_toml_string(model)));
    }
    config.push_str(&format!(
        "\n[model_providers.{provider_key}]\nname = \"{}\"\nbase_url = \"{}\"\n",
        escape_toml_string(name),
        escape_toml_string(base_url)
    ));
    if let Some(api_key) = api_key {
        config.push_str(&format!(
            "experimental_bearer_token = \"{}\"\n",
            escape_toml_string(api_key)
        ));
    }
    config
}

fn strip_sensitive_toml_lines(text: &str) -> (String, bool) {
    let mut stripped = false;
    let lines = text
        .lines()
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            let sensitive = lower.contains("token")
                || lower.contains("api_key")
                || lower.contains("apikey")
                || lower.contains("secret")
                || lower.contains("password")
                || lower.contains("authorization")
                || lower.contains("cookie");
            if sensitive {
                stripped = true;
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    (format!("{}\n", lines.join("\n")), stripped)
}

fn strip_sensitive_json_values(value: &mut Value) -> bool {
    match value {
        Value::Object(map) => {
            let keys = map.keys().cloned().collect::<Vec<_>>();
            let mut stripped = false;
            for key in keys {
                if is_sensitive_key(&key) {
                    map.remove(&key);
                    stripped = true;
                } else if let Some(child) = map.get_mut(&key) {
                    stripped |= strip_sensitive_json_values(child);
                }
            }
            stripped
        }
        Value::Array(items) => {
            let mut stripped = false;
            for item in items {
                stripped |= strip_sensitive_json_values(item);
            }
            stripped
        }
        _ => false,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("token")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("authorization")
        || lower.contains("cookie")
}

fn extract_base_url_hint(config: &str) -> Option<String> {
    config.lines().find_map(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with("base_url") {
            return None;
        }

        trimmed
            .split_once('=')
            .map(|(_, value)| value.trim().trim_matches('"').to_string())
            .filter(|value| !value.is_empty())
    })
}

fn sanitize_id_fragment(value: &str) -> String {
    let normalized = value
        .chars()
        .map(|item| {
            if item.is_ascii_alphanumeric() {
                item.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if normalized.is_empty() {
        "provider".to_string()
    } else {
        normalized
    }
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn add_change(
    changes: &mut Vec<PreviewChange>,
    discovered_configs: &[config_discovery::ConfigDiscoveryItem],
    item_id: &str,
    field: &str,
    action: String,
    sensitive_value: bool,
) {
    if let Some(item) = discovered_configs
        .iter()
        .find(|config| config.id == item_id)
    {
        changes.push(PreviewChange {
            target: item.label.clone(),
            path: item.path.clone(),
            field: field.to_string(),
            action,
            sensitive_value,
        });
    }
}

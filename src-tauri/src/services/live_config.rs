use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde_json::{json, Map, Value};

use crate::services::app_paths;

#[derive(Debug, Clone)]
pub struct ProviderLiveConfig {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub mode: String,
    pub base_url: Option<String>,
    pub settings_config: Value,
    pub category: Option<String>,
    pub codex_config_toml_secret: Option<String>,
    pub codex_auth_json_secret: Option<Value>,
    pub claude_config_json_secret: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct LiveWriteOutcome {
    pub written_paths: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn write_provider_live_config(
    provider: &ProviderLiveConfig,
) -> Result<LiveWriteOutcome, String> {
    match provider.kind.as_str() {
        "codex" => write_codex_live_config(provider),
        "claude" => write_claude_live_config(provider),
        other => Err(format!("unsupported provider kind: {other}")),
    }
}

pub fn provider_has_live_write(provider: &ProviderLiveConfig) -> bool {
    match provider.kind.as_str() {
        "codex" => build_codex_config_text(provider).is_some(),
        "claude" => build_claude_settings(provider).is_some(),
        _ => false,
    }
}

fn write_codex_live_config(provider: &ProviderLiveConfig) -> Result<LiveWriteOutcome, String> {
    let mut warnings = Vec::new();
    let Some(config_text) = provider
        .codex_config_toml_secret
        .clone()
        .or_else(|| build_codex_config_text(provider))
    else {
        return Ok(LiveWriteOutcome {
            written_paths: Vec::new(),
            warnings: vec![
                "Codex Provider 没有可写入的 config.toml 配置，已只更新工作台状态".to_string(),
            ],
        });
    };

    let config_path = app_paths::codex_config_path();
    let auth_path = app_paths::codex_auth_path();
    let old_auth = if auth_path.exists() {
        Some(fs::read(&auth_path).map_err(|err| err.to_string())?)
    } else {
        None
    };

    let mut written_paths = Vec::new();
    if let Some(auth_json) = &provider.codex_auth_json_secret {
        let bytes =
            serde_json::to_vec_pretty(&sort_json_keys(auth_json)).map_err(|err| err.to_string())?;
        atomic_write(&auth_path, &bytes)?;
        written_paths.push(auth_path.to_string_lossy().to_string());
    } else {
        warnings.push("Codex auth.json 未修改；该 Provider 未托管 auth secret".to_string());
    }

    if let Err(err) = atomic_write(&config_path, config_text.as_bytes()) {
        if let Some(old_auth) = old_auth {
            let _ = atomic_write(&auth_path, &old_auth);
        }
        return Err(err);
    }
    written_paths.push(config_path.to_string_lossy().to_string());

    Ok(LiveWriteOutcome {
        written_paths,
        warnings,
    })
}

fn write_claude_live_config(provider: &ProviderLiveConfig) -> Result<LiveWriteOutcome, String> {
    if let Some(settings) = &provider.claude_config_json_secret {
        let path = app_paths::claude_config_path();
        let bytes =
            serde_json::to_vec_pretty(&sort_json_keys(settings)).map_err(|err| err.to_string())?;
        atomic_write(&path, &bytes)?;

        return Ok(LiveWriteOutcome {
            written_paths: vec![path.to_string_lossy().to_string()],
            warnings: Vec::new(),
        });
    }

    let Some(settings) = build_claude_settings(provider) else {
        return Ok(LiveWriteOutcome {
            written_paths: Vec::new(),
            warnings: vec!["Claude Provider 没有可写入的 JSON 配置，已只更新工作台状态".to_string()],
        });
    };

    let path = app_paths::claude_config_path();
    let mut merged = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .unwrap_or_else(|| json!({}))
    } else {
        json!({})
    };
    json_deep_merge(&mut merged, &settings);

    let bytes =
        serde_json::to_vec_pretty(&sort_json_keys(&merged)).map_err(|err| err.to_string())?;
    atomic_write(&path, &bytes)?;

    Ok(LiveWriteOutcome {
        written_paths: vec![path.to_string_lossy().to_string()],
        warnings: Vec::new(),
    })
}

fn build_codex_config_text(provider: &ProviderLiveConfig) -> Option<String> {
    provider
        .settings_config
        .get("config")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|config| !config.is_empty())
        .map(ToString::to_string)
        .or_else(|| build_codex_config_from_base_url(provider))
}

fn build_codex_config_from_base_url(provider: &ProviderLiveConfig) -> Option<String> {
    let base_url = provider.base_url.as_deref()?.trim();
    if base_url.is_empty() || provider.mode != "api-relay" {
        return None;
    }

    let provider_key = format!("workbench_{}", sanitize_toml_key(&provider.id));
    Some(format!(
        "model_provider = \"{provider_key}\"\n\n[model_providers.{provider_key}]\nname = \"{}\"\nbase_url = \"{}\"\n",
        escape_toml_string(&provider.name),
        escape_toml_string(base_url),
    ))
}

fn build_claude_settings(provider: &ProviderLiveConfig) -> Option<Value> {
    if provider
        .settings_config
        .get("preserveLive")
        .and_then(Value::as_bool)
        == Some(true)
    {
        return None;
    }

    if provider
        .settings_config
        .as_object()
        .is_some_and(|obj| !obj.is_empty())
    {
        return Some(provider.settings_config.clone());
    }

    let base_url = provider.base_url.as_deref()?.trim();
    if base_url.is_empty() || provider.mode != "api-relay" {
        return None;
    }

    Some(json!({
        "env": {
            "ANTHROPIC_BASE_URL": base_url
        }
    }))
}

fn json_deep_merge(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, source_value) in source_map {
                if key == "preserveLive" {
                    continue;
                }

                match target_map.get_mut(key) {
                    Some(target_value) => json_deep_merge(target_value, source_value),
                    None => {
                        target_map.insert(key.clone(), source_value.clone());
                    }
                }
            }
        }
        (target_value, source_value) => {
            *target_value = source_value.clone();
        }
    }
}

fn sort_json_keys(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), sort_json_keys(&map[key]));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(sort_json_keys).collect()),
        other => other.clone(),
    }
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let tmp_path = temporary_path(path)?;
    {
        let mut file = fs::File::create(&tmp_path).map_err(|err| err.to_string())?;
        file.write_all(data).map_err(|err| err.to_string())?;
        file.flush().map_err(|err| err.to_string())?;
    }

    if path.exists() {
        fs::remove_file(path).map_err(|err| err.to_string())?;
    }
    fs::rename(&tmp_path, path).map_err(|err| err.to_string())?;
    Ok(())
}

fn temporary_path(path: &Path) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid path parent".to_string())?;
    let file_name = path
        .file_name()
        .ok_or_else(|| "invalid file name".to_string())?
        .to_string_lossy();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| err.to_string())?
        .as_nanos();

    Ok(parent.join(format!("{file_name}.tmp.{nanos}")))
}

fn sanitize_toml_key(value: &str) -> String {
    value
        .chars()
        .map(|item| {
            if item.is_ascii_alphanumeric() || item == '_' || item == '-' {
                item
            } else {
                '_'
            }
        })
        .collect()
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

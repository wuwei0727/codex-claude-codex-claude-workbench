use std::{fs, path::Path};

use serde::Serialize;

use crate::services::app_paths;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDiscoveryItem {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub path: String,
    pub exists: bool,
    pub is_directory: bool,
    pub size_bytes: Option<u64>,
    pub structure_summary: String,
    pub sensitive_state: String,
}

pub fn discover_configs() -> Vec<ConfigDiscoveryItem> {
    vec![
        inspect_text_config(
            "codex-config",
            "Codex config.toml",
            "toml",
            app_paths::codex_config_path().as_path(),
        ),
        inspect_auth_file(app_paths::codex_auth_path().as_path()),
        inspect_text_config(
            "claude-config",
            "Claude .claude.json",
            "json",
            app_paths::claude_config_path().as_path(),
        ),
        inspect_directory(
            "claude-app-data",
            "Claude app data",
            app_paths::claude_app_data_dir().as_path(),
        ),
    ]
}

fn inspect_text_config(id: &str, label: &str, kind: &str, path: &Path) -> ConfigDiscoveryItem {
    let metadata = fs::metadata(path).ok();
    let exists = metadata.is_some();
    let size_bytes = metadata.as_ref().map(|item| item.len());

    let (structure_summary, sensitive_state) = if exists {
        match fs::read_to_string(path) {
            Ok(content) => (
                summarize_structure(kind, &content),
                summarize_sensitive_state(&content),
            ),
            Err(_) => (
                "文件存在，但当前无法读取结构".to_string(),
                "未返回任何配置内容".to_string(),
            ),
        }
    } else {
        ("未发现文件".to_string(), "未检测到敏感内容".to_string())
    };

    ConfigDiscoveryItem {
        id: id.to_string(),
        label: label.to_string(),
        kind: kind.to_string(),
        path: path.to_string_lossy().to_string(),
        exists,
        is_directory: false,
        size_bytes,
        structure_summary,
        sensitive_state,
    }
}

fn inspect_auth_file(path: &Path) -> ConfigDiscoveryItem {
    let metadata = fs::metadata(path).ok();

    ConfigDiscoveryItem {
        id: "codex-auth".to_string(),
        label: "Codex auth.json".to_string(),
        kind: "json".to_string(),
        path: path.to_string_lossy().to_string(),
        exists: metadata.is_some(),
        is_directory: false,
        size_bytes: metadata.as_ref().map(|item| item.len()),
        structure_summary: if metadata.is_some() {
            "auth 文件存在，内容受保护且不返回".to_string()
        } else {
            "未发现 auth 文件".to_string()
        },
        sensitive_state: "受保护：只检测存在性，不读取或返回内容".to_string(),
    }
}

fn inspect_directory(id: &str, label: &str, path: &Path) -> ConfigDiscoveryItem {
    let metadata = fs::metadata(path).ok();

    ConfigDiscoveryItem {
        id: id.to_string(),
        label: label.to_string(),
        kind: "directory".to_string(),
        path: path.to_string_lossy().to_string(),
        exists: metadata.is_some(),
        is_directory: true,
        size_bytes: None,
        structure_summary: if metadata.is_some() {
            "目录存在，未枚举内部文件".to_string()
        } else {
            "未发现目录".to_string()
        },
        sensitive_state: "未读取目录内容".to_string(),
    }
}

fn summarize_structure(kind: &str, content: &str) -> String {
    if kind == "json" {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
            return match value {
                serde_json::Value::Object(map) => format!("JSON 对象，顶层字段 {} 个", map.len()),
                serde_json::Value::Array(items) => format!("JSON 数组，条目 {} 个", items.len()),
                _ => "JSON 文件，顶层为非对象结构".to_string(),
            };
        }

        return "JSON 文件存在，但结构解析失败".to_string();
    }

    let table_count = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with('[') && trimmed.ends_with(']')
        })
        .count();
    let assignment_count = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#') && trimmed.contains('=')
        })
        .count();

    format!(
        "TOML 文本，表 {} 个，字段 {} 个",
        table_count, assignment_count
    )
}

fn summarize_sensitive_state(content: &str) -> String {
    let lower = content.to_ascii_lowercase();
    let has_sensitive_marker = [
        "api_key",
        "token",
        "cookie",
        "secret",
        "password",
        "authorization",
    ]
    .iter()
    .any(|marker| lower.contains(marker));

    if has_sensitive_marker {
        "已检测到可能的敏感字段，值已完全脱敏且不会返回".to_string()
    } else {
        "未检测到常见敏感字段标记".to_string()
    }
}

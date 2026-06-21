use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde::Serialize;

use crate::services::{app_paths, config_discovery::ConfigDiscoveryItem};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPlan {
    pub backup_dir: String,
    pub required: bool,
    pub targets: Vec<BackupTarget>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupTarget {
    pub label: String,
    pub source_path: String,
    pub planned_file_name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub label: String,
    pub source_path: String,
    pub backup_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntry {
    pub backup_path: String,
    pub target_id: String,
    pub target_label: String,
    pub source_path: String,
    pub created_at: String,
    pub exists: bool,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRestoreResult {
    pub target_id: String,
    pub target_label: String,
    pub restored_path: String,
    pub backup_path: String,
    pub protective_backups: Vec<BackupResult>,
}

pub fn build_backup_plan(items: &[ConfigDiscoveryItem]) -> BackupPlan {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let targets = items
        .iter()
        .filter(|item| item.exists && !item.is_directory)
        .map(|item| BackupTarget {
            label: item.label.clone(),
            source_path: item.path.clone(),
            planned_file_name: format!("{}-{}.bak", sanitize_file_label(&item.id), timestamp),
        })
        .collect::<Vec<_>>();

    BackupPlan {
        backup_dir: app_paths::backups_dir().to_string_lossy().to_string(),
        required: !targets.is_empty(),
        targets,
    }
}

pub fn build_backup_plan_for_ids(items: &[ConfigDiscoveryItem], ids: &[&str]) -> BackupPlan {
    let filtered = items
        .iter()
        .filter(|item| ids.iter().any(|id| *id == item.id))
        .cloned()
        .collect::<Vec<_>>();

    build_backup_plan(&filtered)
}

pub fn execute_backup_plan(plan: &BackupPlan) -> Result<Vec<BackupResult>, String> {
    fs::create_dir_all(&plan.backup_dir).map_err(|err| err.to_string())?;

    let mut results = Vec::new();
    for target in &plan.targets {
        let source = Path::new(&target.source_path);
        if !source.exists() {
            continue;
        }

        let backup_path = Path::new(&plan.backup_dir).join(&target.planned_file_name);
        fs::copy(source, &backup_path).map_err(|err| {
            format!(
                "failed to backup {} to {}: {err}",
                source.display(),
                backup_path.display()
            )
        })?;

        results.push(BackupResult {
            label: target.label.clone(),
            source_path: target.source_path.clone(),
            backup_path: backup_path.to_string_lossy().to_string(),
        });
    }

    Ok(results)
}

pub fn list_backup_entries(items: &[ConfigDiscoveryItem]) -> Result<Vec<BackupEntry>, String> {
    let backup_dir = app_paths::backups_dir();
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(&backup_dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let Some((item, created_at)) = match_backup_file(items, file_name) else {
            continue;
        };

        let metadata = fs::metadata(&path).ok();
        entries.push(BackupEntry {
            backup_path: path.to_string_lossy().to_string(),
            target_id: item.id.clone(),
            target_label: item.label.clone(),
            source_path: item.path.clone(),
            created_at,
            exists: true,
            size_bytes: metadata.as_ref().map(|item| item.len()),
        });
    }

    entries.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(entries)
}

pub fn restore_backup(
    items: &[ConfigDiscoveryItem],
    backup_path: &str,
) -> Result<BackupRestoreResult, String> {
    let backup_path = PathBuf::from(backup_path);
    let backup_dir = app_paths::backups_dir();
    let canonical_backup = backup_path.canonicalize().map_err(|err| err.to_string())?;
    let canonical_backup_dir = backup_dir.canonicalize().map_err(|err| err.to_string())?;

    if !canonical_backup.starts_with(&canonical_backup_dir) {
        return Err("只能恢复工作台备份目录中的文件".to_string());
    }

    if canonical_backup
        .extension()
        .and_then(|value| value.to_str())
        != Some("bak")
    {
        return Err("只能恢复 .bak 备份文件".to_string());
    }

    let file_name = canonical_backup
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "备份文件名无效".to_string())?;
    let (target, _) = match_backup_file(items, file_name)
        .ok_or_else(|| "备份文件名无法匹配到已知配置项".to_string())?;
    if target.is_directory {
        return Err("当前不支持从文件备份恢复目录".to_string());
    }

    let target_path = PathBuf::from(&target.path);
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let protective_backups = if target_path.exists() {
        execute_backup_plan(&build_backup_plan(std::slice::from_ref(target)))?
    } else {
        Vec::new()
    };

    fs::copy(&canonical_backup, &target_path).map_err(|err| {
        format!(
            "failed to restore {} to {}: {err}",
            canonical_backup.display(),
            target_path.display()
        )
    })?;

    Ok(BackupRestoreResult {
        target_id: target.id.clone(),
        target_label: target.label.clone(),
        restored_path: target_path.to_string_lossy().to_string(),
        backup_path: canonical_backup.to_string_lossy().to_string(),
        protective_backups,
    })
}

fn sanitize_file_label(value: &str) -> String {
    value
        .chars()
        .map(|item| {
            if item.is_ascii_alphanumeric() || item == '-' {
                item
            } else {
                '-'
            }
        })
        .collect()
}

fn match_backup_file<'a>(
    items: &'a [ConfigDiscoveryItem],
    file_name: &str,
) -> Option<(&'a ConfigDiscoveryItem, String)> {
    if !file_name.ends_with(".bak") {
        return None;
    }

    items.iter().find_map(|item| {
        let prefix = format!("{}-", sanitize_file_label(&item.id));
        file_name.strip_prefix(&prefix).and_then(|rest| {
            rest.strip_suffix(".bak")
                .map(|created_at| (item, created_at.to_string()))
        })
    })
}

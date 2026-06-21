use std::{fs, path::Path};

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

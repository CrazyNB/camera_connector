use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const DESKTOP_SCAN_PROTOCOL: &str = "desktop_scan";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopScanPhase {
    Queued,
    Scanning,
    Indexing,
    Completed,
    Failed,
    Cancelled,
}

impl DesktopScanPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Scanning => "scanning",
            Self::Indexing => "indexing",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "scanning" => Self::Scanning,
            "indexing" => Self::Indexing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Queued,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopSourceStatus {
    Available,
    Missing,
    Changed,
}

impl DesktopSourceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Missing => "missing",
            Self::Changed => "changed",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "missing" => Self::Missing,
            "changed" => Self::Changed,
            _ => Self::Available,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopScanRun {
    pub scan_id: String,
    pub project_id: String,
    pub root_path: PathBuf,
    pub root_key: String,
    pub root_label: String,
    pub phase: DesktopScanPhase,
    pub files_seen: usize,
    pub assets_indexed: usize,
    pub groups_updated: usize,
    pub started_at_ms: i64,
    pub updated_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopScannedFile {
    pub local_path: PathBuf,
    pub relative_path: String,
    pub original_filename: String,
    pub normalized_stem: String,
    pub size_bytes: u64,
    pub modified_at_ms: i64,
    pub capture_time_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopScanIndexResult {
    pub assets_indexed: usize,
    pub group_ids: Vec<String>,
}

pub fn desktop_scan_root_key(project_id: &str, root_path: impl AsRef<Path>) -> String {
    stable_key(&format!(
        "{}\t{}",
        project_id,
        normalize_path_for_identity(root_path.as_ref())
    ))
}

pub fn desktop_scan_transfer_id(
    project_id: &str,
    root_path: impl AsRef<Path>,
    relative_path: &str,
) -> String {
    let root_key = desktop_scan_root_key(project_id, root_path);
    let file_key = stable_key(&normalize_relative_path(relative_path));
    format!("desktop-scan-{root_key}-{file_key}")
}

pub fn desktop_scan_root_label(root_path: impl AsRef<Path>) -> String {
    root_path
        .as_ref()
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| root_path.as_ref().display().to_string())
}

fn normalize_path_for_identity(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

fn normalize_relative_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_ascii_lowercase()
}

fn stable_key(value: &str) -> String {
    let mut hash = 1469598103934665603_u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}

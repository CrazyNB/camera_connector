use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::{ObjectFormat, ReceiverAccountConfig, Result, StoredObjectLocation};

#[derive(Debug, Clone)]
pub struct SqliteStore {
    pub(super) db_path: PathBuf,
    pub(super) access_lock: Arc<Mutex<()>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub slug: String,
    pub status: ProjectStatus,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub archived_at_ms: Option<i64>,
    pub default_output_target_id: Option<String>,
}

impl Project {
    pub fn kind(&self) -> ProjectKind {
        ProjectKind::User
    }

    pub fn capabilities(&self) -> ProjectCapabilities {
        let active = self.status == ProjectStatus::Active;
        let archived = self.status == ProjectStatus::Archived;
        ProjectCapabilities {
            can_be_active_project: active,
            can_archive: active,
            can_rename: true,
            can_restore: archived,
            can_accept_moved_groups: active,
        }
    }

    pub fn into_view(self) -> ProjectView {
        let kind = self.kind();
        let capabilities = self.capabilities();
        ProjectView {
            project: self,
            kind,
            capabilities,
        }
    }

    pub fn view(&self) -> ProjectView {
        self.clone().into_view()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectKind {
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCapabilities {
    pub can_be_active_project: bool,
    pub can_archive: bool,
    pub can_rename: bool,
    pub can_restore: bool,
    pub can_accept_moved_groups: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectView {
    #[serde(flatten)]
    pub project: Project,
    pub kind: ProjectKind,
    pub capabilities: ProjectCapabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Archived,
}

impl ProjectStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "archived" => Self::Archived,
            _ => Self::Active,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredAsset {
    pub asset_id: String,
    pub project_id: String,
    pub group_id: Option<String>,
    pub transfer_id: String,
    pub group_role: String,
    pub media_kind: String,
    pub format: ObjectFormat,
    pub original_filename: String,
    pub final_filename: String,
    pub normalized_stem: String,
    pub original_path: String,
    pub original_parent_path: Option<String>,
    pub final_location: Option<StoredObjectLocation>,
    pub size_bytes: u64,
    pub capture_at_ms: Option<i64>,
    pub received_at_ms: Option<i64>,
    pub published_at_ms: Option<i64>,
    pub source_identity: Option<String>,
    pub username: Option<String>,
    pub remote_addr: Option<String>,
    pub source_status: String,
    pub source_modified_at_ms: Option<i64>,
    pub last_seen_scan_id: Option<String>,
    pub duplicate_index: Option<usize>,
    pub duplicate_count: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredAssetGroup {
    pub group_id: String,
    pub project_id: String,
    pub group_identity: String,
    pub display_key: String,
    pub source_identity: Option<String>,
    pub original_parent_path: Option<String>,
    pub primary_asset_id: Option<String>,
    pub preview_asset_id: Option<String>,
    pub member_count: usize,
    pub has_raw: bool,
    pub has_jpeg: bool,
    pub has_video: bool,
    pub first_capture_at_ms: Option<i64>,
    pub last_capture_at_ms: Option<i64>,
    pub first_received_at_ms: Option<i64>,
    pub last_received_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredReceiverAccount {
    pub username: String,
    pub password_hash: Option<String>,
    pub device_name: String,
    pub enabled: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl StoredReceiverAccount {
    pub fn into_account_config(self) -> Result<ReceiverAccountConfig> {
        ReceiverAccountConfig {
            username: self.username,
            password_hash: self.password_hash,
            password: None,
            device_name: self.device_name,
        }
        .validated()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishQueueItem {
    pub queue_id: String,
    pub project_id: String,
    pub transfer_id: String,
    pub staged_path: String,
    pub final_filename: String,
    pub size_bytes: u64,
    pub protocol: Option<String>,
    pub original_path: Option<String>,
    pub username: Option<String>,
    pub remote_addr: Option<String>,
    pub source_name: Option<String>,
    pub started_at_ms: Option<i64>,
    pub state: PublishState,
    pub attempt_count: u32,
    pub last_error: Option<String>,
    pub next_attempt_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishTransferMetadata {
    pub protocol: String,
    pub original_path: String,
    pub username: Option<String>,
    pub remote_addr: Option<String>,
    pub source_name: Option<String>,
    pub started_at_ms: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishQueueSummary {
    pub total_count: usize,
    pub pending_count: usize,
    pub staged_count: usize,
    pub publishing_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalAssetSummary {
    pub photo_count: usize,
    pub file_count: usize,
    pub storage_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublishState {
    Staged,
    Publishing,
    Completed,
    Failed,
}

impl PublishState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Staged => "staged",
            Self::Publishing => "publishing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "publishing" => Self::Publishing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::Staged,
        }
    }
}

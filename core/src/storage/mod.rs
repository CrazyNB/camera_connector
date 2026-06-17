use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

mod pipeline;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::media_metadata::extract_capture_time_ms;
use crate::{
    desktop_scan_root_key, desktop_scan_root_label, desktop_scan_transfer_id,
    group_received_assets, AnalysisEntityType, AnalysisJob, AnalysisJobStatus, AnalysisJobType,
    AssetFacetCount, AssetGroupPage, AssetGroupQuery, AssetGroupSort, AssetGroupSummary,
    AssetUserMarks, BurstGroup, BurstGroupingProfile, ConnectedDevice, CvPolicy,
    DesktopScanIndexResult, DesktopScanPhase, DesktopScanRun, DesktopScannedFile,
    DesktopSourceStatus, EvaluationRun, EvaluationRunStatus, EvaluationRunTrigger,
    EvaluationRunType, ImportSource, ImporterError, ModelEvaluation, ModelEvaluationStatus,
    ModelEvaluationTier, ModelEvaluatorKind, ModelProviderKind, NewAnalysisJob, ObjectFormat,
    ProjectEvaluationSettings, ProjectRecommendationMode, PushProtocol, ReceivedAsset,
    ReceivedAssetBurstSummary, ReceivedAssetGroup, ReceivedAssetTechnicalDefectSummary,
    ReceiverAccountConfig, ReceiverAuthMode, ReceiverRuntimePhase, ReceiverRuntimeStatus, Result,
    SceneProfile, SelectionRecommendation, SelectionRecommendationScope,
    SelectionRecommendationStatus, SelectionSource, StoredObjectLocation, SubjectAssessment,
    TechnicalAssessment, TechnicalAssessmentPolicy, TechnicalAssessmentStatus, TechnicalDefectFlag,
    TechnicalGateStatus, TransferRecord, TransferStatus, DESKTOP_SCAN_PROTOCOL,
};

pub use pipeline::{LocalFolderObjectStore, LocalStagedUpload, LocalStagingStore, StagedObject};

const DB_FILENAME: &str = "camera-connector.sqlite3";
const FAILED_PUBLISH_RETRY_DELAY_MS: i64 = 30_000;
const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone)]
pub struct SqliteStore {
    db_path: PathBuf,
    access_lock: Arc<Mutex<()>>,
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

    fn from_str(value: &str) -> Self {
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

    fn from_str(value: &str) -> Self {
        match value {
            "publishing" => Self::Publishing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::Staged,
        }
    }
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let store = Self {
            access_lock: sqlite_access_lock(&path),
            db_path: path,
        };
        store.with_write_connection(|connection| initialize_schema(connection, &store.db_path))?;
        Ok(store)
    }

    pub fn open_state_dir(state_dir: impl AsRef<Path>) -> Result<Self> {
        Self::open(state_dir.as_ref().join(DB_FILENAME))
    }

    pub fn state_dir(&self) -> PathBuf {
        self.db_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn create_project(&self, name: impl AsRef<str>) -> Result<Project> {
        let name = normalized_required("project name", name.as_ref())?;
        let now = current_time_ms();
        let slug = slugify(&name);
        let project = Project {
            project_id: format!("project-{now}-{slug}"),
            name,
            slug,
            status: ProjectStatus::Active,
            created_at_ms: now,
            updated_at_ms: now,
            archived_at_ms: None,
            default_output_target_id: None,
        };
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO projects (
                    project_id, name, slug, status, created_at_ms, updated_at_ms,
                    archived_at_ms, default_output_target_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    project.project_id,
                    project.name,
                    project.slug,
                    project.status.as_str(),
                    project.created_at_ms,
                    project.updated_at_ms,
                    project.archived_at_ms,
                    project.default_output_target_id,
                ],
            )?;
            save_project_evaluation_settings_for_connection(
                connection,
                ProjectEvaluationSettings::default_for_project(&project.project_id, now),
            )?;
            Ok(project)
        })
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        self.with_read_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT project_id, name, slug, status, created_at_ms, updated_at_ms,
                        archived_at_ms, default_output_target_id
                 FROM projects
                 ORDER BY updated_at_ms DESC, name ASC",
            )?;
            let rows = statement.query_map([], project_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn rename_project(&self, project_id: &str, name: impl AsRef<str>) -> Result<Project> {
        let name = normalized_required("project name", name.as_ref())?;
        let slug = slugify(&name);
        self.with_connection(|connection| {
            let now = current_time_ms();
            ensure_project_exists(connection, project_id)?;
            connection.execute(
                "UPDATE projects
                 SET name = ?1, slug = ?2, updated_at_ms = ?3
                 WHERE project_id = ?4",
                params![name, slug, now, project_id],
            )?;
            project_by_id(connection, project_id)?.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName("project not found".to_string())
            })
        })
    }

    pub fn archive_project(&self, project_id: &str) -> Result<Project> {
        self.with_connection(|connection| {
            let now = current_time_ms();
            ensure_project_exists(connection, project_id)?;
            connection.execute(
                "UPDATE projects
                 SET status = ?1, archived_at_ms = ?2, updated_at_ms = ?2
                 WHERE project_id = ?3",
                params![ProjectStatus::Archived.as_str(), now, project_id],
            )?;
            project_by_id(connection, project_id)?.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName("project not found".to_string())
            })
        })
    }

    pub fn restore_project(&self, project_id: &str) -> Result<Project> {
        self.with_connection(|connection| {
            let now = current_time_ms();
            ensure_project_exists(connection, project_id)?;
            connection.execute(
                "UPDATE projects
                 SET status = ?1, archived_at_ms = NULL, updated_at_ms = ?2
                 WHERE project_id = ?3",
                params![ProjectStatus::Active.as_str(), now, project_id],
            )?;
            project_by_id(connection, project_id)?.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName("project not found".to_string())
            })
        })
    }

    pub fn delete_project(&self, project_id: &str) -> Result<Option<Vec<StoredAsset>>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            if project_by_id(&transaction, project_id)?.is_none() {
                transaction.commit()?;
                return Ok(None);
            }

            let assets = {
                let mut statement = transaction.prepare(
                    "SELECT asset_id, project_id, group_id, transfer_id, group_role,
                            media_kind, format, original_filename, final_filename, normalized_stem, original_path,
                            original_parent_path, final_location_payload, size_bytes, capture_at_ms,
                            received_at_ms, published_at_ms, source_identity, username, remote_addr,
                            source_status, source_modified_at_ms, last_seen_scan_id, duplicate_index,
                            duplicate_count
                     FROM assets
                     WHERE project_id = ?1
                     ORDER BY published_at_ms ASC, asset_id ASC",
                )?;
                let rows = statement.query_map(params![project_id], stored_asset_from_row)?;
                collect_rows(rows)?
            };

            transaction.execute(
                "DELETE FROM technical_assessments
                 WHERE asset_group_id IN (
                    SELECT group_id FROM asset_groups WHERE project_id = ?1
                 )",
                params![project_id],
            )?;
            transaction.execute(
                "DELETE FROM burst_group_members
                 WHERE burst_group_id IN (
                    SELECT burst_group_id FROM burst_groups WHERE project_id = ?1
                 )
                    OR member_group_id IN (
                    SELECT group_id FROM asset_groups WHERE project_id = ?1
                 )",
                params![project_id],
            )?;
            for table in [
                "burst_member_manual_edits",
                "asset_group_user_marks",
                "subject_assessments",
                "model_evaluations",
                "selection_recommendations",
                "background_jobs",
                "publish_queue",
                "evaluation_runs",
                "assets",
                "transfers",
                "burst_groups",
                "asset_groups",
                "project_evaluation_settings",
                "projects",
            ] {
                transaction.execute(
                    &format!("DELETE FROM {table} WHERE project_id = ?1"),
                    params![project_id],
                )?;
            }
            transaction.commit()?;
            Ok(Some(assets))
        })
    }

    pub fn project_evaluation_settings(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEvaluationSettings>> {
        self.with_connection(|connection| {
            project_evaluation_settings_for_project(connection, project_id)
        })
    }

    pub fn save_project_evaluation_settings(
        &self,
        settings: ProjectEvaluationSettings,
    ) -> Result<ProjectEvaluationSettings> {
        self.with_connection(|connection| {
            save_project_evaluation_settings_for_connection(connection, settings)
        })
    }

    pub fn save_evaluation_run(&self, run: EvaluationRun) -> Result<EvaluationRun> {
        self.with_connection(|connection| save_evaluation_run_for_connection(connection, run))
    }

    pub fn latest_evaluation_run(
        &self,
        project_id: &str,
        run_type: EvaluationRunType,
    ) -> Result<Option<EvaluationRun>> {
        self.with_read_connection(|connection| {
            latest_evaluation_run(connection, project_id, run_type)
        })
    }

    pub fn save_subject_assessment(
        &self,
        assessment: SubjectAssessment,
    ) -> Result<SubjectAssessment> {
        self.with_connection(|connection| {
            save_subject_assessment_for_connection(connection, assessment)
        })
    }

    pub fn subject_assessments_for_asset_groups(
        &self,
        project_id: &str,
        group_ids: &[String],
    ) -> Result<Vec<SubjectAssessment>> {
        self.with_read_connection(|connection| {
            subject_assessments_for_asset_groups(connection, project_id, group_ids)
        })
    }

    pub fn upsert_receiver_account(
        &self,
        account: ReceiverAccountConfig,
    ) -> Result<StoredReceiverAccount> {
        let account = account.validated()?;
        let now = current_time_ms();
        self.with_connection(|connection| {
            let created_at_ms = connection
                .query_row(
                    "SELECT created_at_ms FROM receiver_accounts WHERE username = ?1",
                    params![&account.username],
                    |row| row.get::<_, i64>(0),
                )
                .optional()?
                .unwrap_or(now);
            connection.execute(
                "INSERT INTO receiver_accounts (
                    username, password_hash, device_name, enabled, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, 1, ?4, ?5)
                 ON CONFLICT(username) DO UPDATE SET
                    password_hash = excluded.password_hash,
                    device_name = excluded.device_name,
                    enabled = 1,
                    updated_at_ms = excluded.updated_at_ms",
                params![
                    &account.username,
                    account.password_hash.as_deref(),
                    &account.device_name,
                    created_at_ms,
                    now,
                ],
            )?;
            receiver_account_by_username(connection, &account.username)?.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName("receiver account not found".to_string())
            })
        })
    }

    pub fn remove_receiver_account(&self, username: &str) -> Result<bool> {
        self.with_connection(|connection| {
            let changed = connection.execute(
                "DELETE FROM receiver_accounts WHERE username = ?1",
                params![username],
            )?;
            if changed > 0 {
                connection.execute(
                    "UPDATE connected_devices SET username = NULL WHERE username = ?1",
                    params![username],
                )?;
            }
            Ok(changed > 0)
        })
    }

    pub fn receiver_accounts(&self) -> Result<Vec<StoredReceiverAccount>> {
        self.with_read_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT username, password_hash, device_name, enabled, created_at_ms, updated_at_ms
                 FROM receiver_accounts
                 WHERE enabled = 1
                 ORDER BY username ASC",
            )?;
            let rows = statement.query_map([], receiver_account_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn record_connected_device(
        &self,
        remote_addr: &str,
        remote_port: Option<u16>,
        source_name: Option<&str>,
        username: Option<&str>,
    ) -> Result<()> {
        let now = current_time_ms();
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let mut devices = connected_devices_from_connection(&transaction)?;
            if let Some(device) = devices
                .iter_mut()
                .find(|device| device.remote_addr == remote_addr)
            {
                device.last_seen_at_ms = now;
                device.last_remote_port = remote_port;
                device.active_connections = device.active_connections.saturating_add(1);
                device.online = true;
                if let Some(source_name) = source_name {
                    device.source_name = Some(source_name.to_string());
                }
                if let Some(username) = username {
                    device.username = Some(username.to_string());
                }
            } else {
                devices.push(ConnectedDevice {
                    remote_addr: remote_addr.to_string(),
                    source_name: source_name.map(ToOwned::to_owned),
                    username: username.map(ToOwned::to_owned),
                    first_seen_at_ms: now,
                    last_seen_at_ms: now,
                    last_disconnected_at_ms: None,
                    last_remote_port: remote_port,
                    active_connections: 1,
                    online: true,
                });
            }
            replace_connected_devices(&transaction, &devices)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn record_authenticated_device(
        &self,
        remote_addr: &str,
        source_name: Option<&str>,
        username: Option<&str>,
    ) -> Result<()> {
        let now = current_time_ms();
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let mut devices = connected_devices_from_connection(&transaction)?;
            let previous_for_username = username.and_then(|username| {
                devices
                    .iter()
                    .filter(|device| {
                        device.username.as_deref() == Some(username)
                            && device.remote_addr != remote_addr
                    })
                    .fold(None, |previous: Option<ConnectedDevice>, device| {
                        Some(match previous {
                            Some(previous)
                                if previous.first_seen_at_ms <= device.first_seen_at_ms =>
                            {
                                previous
                            }
                            _ => device.clone(),
                        })
                    })
            });

            if let Some(username) = username {
                devices.retain(|device| {
                    device.remote_addr == remote_addr
                        || device.username.as_deref() != Some(username)
                });
            }

            if let Some(device) = devices
                .iter_mut()
                .find(|device| device.remote_addr == remote_addr)
            {
                if let Some(previous) = previous_for_username {
                    device.first_seen_at_ms =
                        device.first_seen_at_ms.min(previous.first_seen_at_ms);
                    if device.source_name.is_none() {
                        device.source_name = previous.source_name;
                    }
                }
                device.last_seen_at_ms = now;
                if let Some(source_name) = source_name {
                    device.source_name = Some(source_name.to_string());
                }
                if let Some(username) = username {
                    device.username = Some(username.to_string());
                }
            }

            replace_connected_devices(&transaction, &devices)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn record_disconnected_device(&self, remote_addr: &str) -> Result<()> {
        let now = current_time_ms();
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let mut devices = connected_devices_from_connection(&transaction)?;
            if let Some(device) = devices
                .iter_mut()
                .find(|device| device.remote_addr == remote_addr)
            {
                device.active_connections = device.active_connections.saturating_sub(1);
                device.online = device.active_connections > 0;
                device.last_seen_at_ms = now;
                if !device.online {
                    device.last_disconnected_at_ms = Some(now);
                }
            }
            replace_connected_devices(&transaction, &devices)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn mark_all_connected_devices_offline(&self) -> Result<()> {
        let now = current_time_ms();
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let mut devices = connected_devices_from_connection(&transaction)?;
            for device in devices.iter_mut().filter(|device| device.online) {
                device.active_connections = 0;
                device.online = false;
                device.last_seen_at_ms = now;
                device.last_disconnected_at_ms = Some(now);
            }
            replace_connected_devices(&transaction, &devices)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn connected_devices(&self) -> Result<Vec<ConnectedDevice>> {
        self.with_read_connection(|connection| {
            let mut devices = connected_devices_from_connection(connection)?;
            sort_connected_devices(&mut devices);
            Ok(devices)
        })
    }

    pub fn write_receiver_runtime_status(&self, status: &ReceiverRuntimeStatus) -> Result<()> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO receiver_status (
                    key, phase, protocol, auth_mode, local_addr, output_dir, state_dir,
                    account_count, message, updated_at_ms
                 ) VALUES ('current', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(key) DO UPDATE SET
                    phase = excluded.phase,
                    protocol = excluded.protocol,
                    auth_mode = excluded.auth_mode,
                    local_addr = excluded.local_addr,
                    output_dir = excluded.output_dir,
                    state_dir = excluded.state_dir,
                    account_count = excluded.account_count,
                    message = excluded.message,
                    updated_at_ms = excluded.updated_at_ms",
                params![
                    receiver_runtime_phase_name(status.phase),
                    status.protocol.map(push_protocol_name),
                    receiver_auth_mode_name(status.auth_mode),
                    status.local_addr.map(|addr| addr.to_string()),
                    status
                        .output_dir
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned()),
                    status
                        .state_dir
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned()),
                    status.account_count as i64,
                    status.message.as_deref(),
                    current_time_ms(),
                ],
            )?;
            Ok(())
        })
    }

    pub fn read_receiver_runtime_status(&self) -> Result<Option<ReceiverRuntimeStatus>> {
        self.with_read_connection(|connection| {
            connection
                .query_row(
                    "SELECT phase, protocol, auth_mode, local_addr, output_dir, state_dir,
                            account_count, message
                     FROM receiver_status
                     WHERE key = 'current'",
                    [],
                    receiver_runtime_status_from_row,
                )
                .optional()
        })
    }

    pub fn record_transfer(&self, project_id: &str, record: TransferRecord) -> Result<()> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            ensure_project_is_active(&transaction, project_id)?;
            insert_transfer(&transaction, project_id, &record)?;
            if record.status == TransferStatus::Completed {
                if let Some(asset_group_id) =
                    insert_asset_for_transfer(&transaction, project_id, &record)?
                {
                    enqueue_detect_burst_job_for_connection(
                        &transaction,
                        project_id,
                        &asset_group_id,
                    )?;
                    if should_schedule_subject_assessment_for_project(&transaction, project_id)? {
                        enqueue_portrait_subject_assessment_job_for_connection(
                            &transaction,
                            project_id,
                            &asset_group_id,
                        )?;
                    }
                }
            }
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn asset_group_page(
        &self,
        project_id: &str,
        query: AssetGroupQuery,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        self.with_read_connection(|connection| {
            let stored_groups = stored_asset_groups_for_project(connection, project_id)?;
            let mut groups = Vec::new();
            for stored_group in stored_groups {
                let assets =
                    received_assets_for_group(connection, project_id, &stored_group.group_id)?;
                if let Some(mut group) = group_received_assets(assets).into_iter().next() {
                    group.group_id = Some(stored_group.group_id);
                    if asset_group_matches(&group, &query) {
                        if let Some(group_id) = group.group_id.clone() {
                            group.burst =
                                burst_summary_for_asset_group(connection, project_id, &group_id)?;
                            apply_technical_summary(connection, &group_id, &mut group)?;
                            apply_model_evaluation_summary(connection, &group_id, &mut group)?;
                            group.is_model_select = is_model_selected_asset_group(
                                connection,
                                project_id,
                                &group_id,
                                group
                                    .burst
                                    .as_ref()
                                    .map(|burst| burst.burst_group_id.as_str()),
                            )?;
                            group.user_marks =
                                user_marks_for_asset_group(connection, project_id, &group_id)?;
                            group.is_favorite = group.user_marks.favorite;
                            group.is_flagged = group.user_marks.marked;
                        }
                        if asset_group_matches_analysis(&group, &query) {
                            groups.push(group);
                        }
                    }
                }
            }

            sort_asset_groups_for_query(&mut groups, query.sort);
            let total_groups = groups.len();
            let summary = summarize_asset_groups(&groups);
            let page_groups = groups
                .into_iter()
                .skip(offset)
                .take(limit)
                .collect::<Vec<_>>();

            Ok(AssetGroupPage {
                groups: page_groups,
                summary,
                offset,
                limit,
                total_groups,
                has_more: offset.saturating_add(limit) < total_groups,
            })
        })
    }

    pub fn assets_for_group(&self, project_id: &str, group_id: &str) -> Result<Vec<StoredAsset>> {
        self.with_read_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT asset_id, project_id, group_id, transfer_id, group_role,
                        media_kind, format, original_filename, final_filename, normalized_stem, original_path,
                        original_parent_path, final_location_payload, size_bytes, capture_at_ms,
                        received_at_ms, published_at_ms, source_identity, username, remote_addr,
                        source_status, source_modified_at_ms, last_seen_scan_id, duplicate_index,
                        duplicate_count
                 FROM assets
                 WHERE project_id = ?1 AND group_id = ?2
                 ORDER BY CASE group_role
                            WHEN 'jpeg' THEN 0
                            WHEN 'raw' THEN 1
                            WHEN 'video' THEN 2
                            ELSE 3
                          END ASC,
                          published_at_ms ASC,
                          asset_id ASC",
            )?;
            let rows = statement.query_map(params![project_id, group_id], stored_asset_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn delete_asset_group(
        &self,
        project_id: &str,
        group_id: &str,
    ) -> Result<Option<Vec<StoredAsset>>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            ensure_project_exists(&transaction, project_id)?;
            let mut statement = transaction.prepare(
                "SELECT asset_id, project_id, group_id, transfer_id, group_role,
                        media_kind, format, original_filename, final_filename, normalized_stem, original_path,
                        original_parent_path, final_location_payload, size_bytes, capture_at_ms,
                        received_at_ms, published_at_ms, source_identity, username, remote_addr,
                        source_status, source_modified_at_ms, last_seen_scan_id, duplicate_index,
                        duplicate_count
                 FROM assets
                 WHERE project_id = ?1 AND group_id = ?2
                 ORDER BY published_at_ms ASC, asset_id ASC",
            )?;
            let rows = statement.query_map(params![project_id, group_id], stored_asset_from_row)?;
            let assets = collect_rows(rows)?;
            drop(statement);
            if assets.is_empty() {
                transaction.commit()?;
                return Ok(None);
            }

            let affected_burst_ids = transaction
                .prepare(
                    "SELECT burst_group_id FROM burst_group_members WHERE member_group_id = ?1",
                )?
                .query_map(params![group_id], |row| row.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            transaction.execute(
                "DELETE FROM selection_recommendations
                 WHERE project_id = ?1
                   AND (scope = 'project' OR subject_id = ?2)",
                params![project_id, group_id],
            )?;
            for burst_group_id in &affected_burst_ids {
                transaction.execute(
                    "DELETE FROM selection_recommendations
                     WHERE project_id = ?1 AND subject_id = ?2",
                    params![project_id, burst_group_id],
                )?;
                transaction.execute(
                    "DELETE FROM background_jobs
                     WHERE project_id = ?1 AND entity_type = 'burst_group' AND entity_id = ?2",
                    params![project_id, burst_group_id],
                )?;
            }
            transaction.execute(
                "DELETE FROM background_jobs
                 WHERE project_id = ?1 AND entity_type = 'asset_group' AND entity_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM technical_assessments WHERE asset_group_id = ?1",
                params![group_id],
            )?;
            transaction.execute(
                "DELETE FROM model_evaluations WHERE project_id = ?1 AND asset_group_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM subject_assessments WHERE project_id = ?1 AND asset_group_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM asset_group_user_marks WHERE project_id = ?1 AND group_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM burst_member_manual_edits WHERE project_id = ?1 AND member_group_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM burst_group_members WHERE member_group_id = ?1",
                params![group_id],
            )?;
            for asset in &assets {
                transaction.execute(
                    "DELETE FROM publish_queue WHERE project_id = ?1 AND transfer_id = ?2",
                    params![project_id, asset.transfer_id],
                )?;
                transaction.execute(
                    "DELETE FROM assets WHERE project_id = ?1 AND asset_id = ?2",
                    params![project_id, asset.asset_id],
                )?;
                transaction.execute(
                    "DELETE FROM transfers WHERE project_id = ?1 AND transfer_id = ?2",
                    params![project_id, asset.transfer_id],
                )?;
            }
            transaction.execute(
                "DELETE FROM asset_groups WHERE project_id = ?1 AND group_id = ?2",
                params![project_id, group_id],
            )?;
            for burst_group_id in &affected_burst_ids {
                transaction.execute(
                    "UPDATE burst_groups
                     SET member_count = (
                         SELECT COUNT(*) FROM burst_group_members WHERE burst_group_id = ?1
                     ),
                         updated_at_ms = ?2
                     WHERE burst_group_id = ?1",
                    params![burst_group_id, current_time_ms()],
                )?;
                transaction.execute(
                    "DELETE FROM burst_groups
                     WHERE burst_group_id = ?1 AND member_count < 2",
                    params![burst_group_id],
                )?;
            }
            transaction.execute(
                "UPDATE projects SET updated_at_ms = ?1 WHERE project_id = ?2",
                params![current_time_ms(), project_id],
            )?;
            refresh_duplicate_info(&transaction, project_id)?;
            transaction.commit()?;
            Ok(Some(assets))
        })
    }

    pub fn set_asset_group_user_marks(
        &self,
        project_id: &str,
        group_id: &str,
        favorite: Option<bool>,
        marked: Option<bool>,
    ) -> Result<AssetUserMarks> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            if stored_asset_group_by_id(connection, project_id, group_id)?.is_none() {
                return Err(rusqlite::Error::InvalidParameterName(
                    "asset group not found".to_string(),
                ));
            }
            let existing = user_marks_for_asset_group(connection, project_id, group_id)?;
            let next = AssetUserMarks {
                favorite: favorite.unwrap_or(existing.favorite),
                marked: marked.unwrap_or(existing.marked),
            };
            let now = current_time_ms();
            connection.execute(
                "INSERT INTO asset_group_user_marks (
                    project_id, group_id, favorite, marked, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?5)
                 ON CONFLICT(project_id, group_id) DO UPDATE SET
                    favorite = excluded.favorite,
                    marked = excluded.marked,
                    updated_at_ms = excluded.updated_at_ms",
                params![project_id, group_id, next.favorite, next.marked, now,],
            )?;
            Ok(next)
        })
    }

    pub fn stored_asset_groups(&self, project_id: &str) -> Result<Vec<StoredAssetGroup>> {
        self.with_read_connection(|connection| {
            stored_asset_groups_for_project(connection, project_id)
        })
    }

    pub fn global_asset_summary(&self) -> Result<GlobalAssetSummary> {
        self.with_read_connection(|connection| {
            connection
                .query_row(
                    "SELECT
                        (SELECT COUNT(*) FROM asset_groups),
                        (SELECT COUNT(*) FROM assets),
                        COALESCE((SELECT SUM(size_bytes) FROM assets), 0)",
                    [],
                    |row| {
                        Ok(GlobalAssetSummary {
                            photo_count: row.get::<_, i64>(0)?.max(0) as usize,
                            file_count: row.get::<_, i64>(1)?.max(0) as usize,
                            storage_bytes: row.get::<_, i64>(2)?.max(0) as u64,
                        })
                    },
                )
                .map_err(Into::into)
        })
    }

    pub fn project_id_for_asset_group(&self, asset_group_id: &str) -> Result<Option<String>> {
        self.with_read_connection(|connection| {
            connection
                .query_row(
                    "SELECT project_id FROM asset_groups WHERE group_id = ?1 LIMIT 1",
                    params![asset_group_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(Into::into)
        })
    }

    pub fn save_technical_assessment(
        &self,
        assessment: TechnicalAssessment,
    ) -> Result<TechnicalAssessment> {
        self.with_connection(|connection| {
            save_technical_assessment_for_connection(connection, assessment)
        })
    }

    pub fn technical_assessments_for_asset_groups(
        &self,
        asset_group_ids: &[String],
        assessor_version: &str,
    ) -> Result<Vec<TechnicalAssessment>> {
        self.with_read_connection(|connection| {
            technical_assessments_for_asset_group_ids(connection, asset_group_ids, assessor_version)
        })
    }

    pub fn save_model_evaluation(&self, evaluation: ModelEvaluation) -> Result<ModelEvaluation> {
        self.with_connection(|connection| {
            save_model_evaluation_for_connection(connection, evaluation)
        })
    }

    pub fn model_evaluations_for_asset_groups(
        &self,
        asset_group_ids: &[String],
        evaluator_version: &str,
    ) -> Result<Vec<ModelEvaluation>> {
        self.with_read_connection(|connection| {
            model_evaluations_for_asset_group_ids(connection, asset_group_ids, evaluator_version)
        })
    }

    pub fn save_selection_recommendation(
        &self,
        recommendation: SelectionRecommendation,
    ) -> Result<SelectionRecommendation> {
        self.with_connection(|connection| {
            save_selection_recommendation_for_connection(connection, recommendation)
        })
    }

    pub fn latest_selection_recommendation(
        &self,
        project_id: &str,
        scope: SelectionRecommendationScope,
        subject_id: &str,
    ) -> Result<Option<SelectionRecommendation>> {
        self.with_read_connection(|connection| {
            latest_selection_recommendation_for_connection(
                connection, project_id, scope, subject_id,
            )
        })
    }

    pub fn burst_group(&self, burst_group_id: &str) -> Result<Option<BurstGroup>> {
        self.with_read_connection(|connection| burst_group_by_id(connection, burst_group_id))
    }

    pub fn burst_group_for_asset_group(&self, asset_group_id: &str) -> Result<Option<BurstGroup>> {
        self.with_read_connection(|connection| {
            let project_id = connection
                .query_row(
                    "SELECT project_id FROM asset_groups WHERE group_id = ?1",
                    params![asset_group_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            project_id
                .map(|project_id| {
                    burst_group_for_member_group(connection, &project_id, asset_group_id)
                })
                .transpose()
                .map(|value| value.flatten())
                .map_err(Into::into)
        })
    }

    pub fn split_burst_member(
        &self,
        burst_group_id: &str,
        member_group_id: &str,
    ) -> Result<Option<BurstGroup>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let result =
                split_burst_member_for_connection(&transaction, burst_group_id, member_group_id)?;
            transaction.commit()?;
            Ok(result)
        })
    }

    pub fn create_manual_burst_group(
        &self,
        project_id: &str,
        member_group_ids: &[String],
    ) -> Result<Option<BurstGroup>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let result = create_manual_burst_group_for_connection(
                &transaction,
                project_id,
                member_group_ids,
            )?;
            transaction.commit()?;
            Ok(result)
        })
    }

    pub fn detect_bursts_for_asset_group(
        &self,
        project_id: &str,
        asset_group_id: &str,
        profile: &BurstGroupingProfile,
    ) -> Result<Vec<BurstGroup>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            ensure_project_exists(&transaction, project_id)?;
            let groups = stored_asset_groups_for_project(&transaction, project_id)?;
            if !groups.iter().any(|group| group.group_id == asset_group_id) {
                transaction.commit()?;
                return Ok(Vec::new());
            }

            let bursts =
                rebuild_burst_groups_for_project(&transaction, project_id, groups, profile)?;
            transaction.commit()?;
            Ok(bursts
                .into_iter()
                .filter(|burst| {
                    burst
                        .member_group_ids
                        .iter()
                        .any(|member_group_id| member_group_id == asset_group_id)
                })
                .collect())
        })
    }

    pub fn refine_burst_group_by_visual_similarity(
        &self,
        burst_group_id: &str,
        profile: &BurstGroupingProfile,
        assessor_version: &str,
    ) -> Result<Vec<BurstGroup>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let Some(burst) = burst_group_by_id(&transaction, burst_group_id)? else {
                transaction.commit()?;
                return Ok(Vec::new());
            };
            let assessments = technical_assessments_for_asset_group_ids(
                &transaction,
                &burst.member_group_ids,
                assessor_version,
            )?;
            let assessment_by_group_id = assessments
                .iter()
                .map(|assessment| (assessment.asset_group_id.as_str(), assessment))
                .collect::<BTreeMap<_, _>>();
            let mut visual_hashes = Vec::new();
            for member_group_id in &burst.member_group_ids {
                let Some(assessment) = assessment_by_group_id.get(member_group_id.as_str()) else {
                    transaction.commit()?;
                    return Ok(vec![burst]);
                };
                if assessment.status != TechnicalAssessmentStatus::Ready {
                    transaction.commit()?;
                    return Ok(vec![burst]);
                }
                let Some(hash) = visual_hash_from_signature(assessment.visual_signature.as_deref())
                else {
                    transaction.commit()?;
                    return Ok(vec![burst]);
                };
                visual_hashes.push((member_group_id.clone(), hash));
            }

            let threshold = visual_burst_continuity_threshold(profile);
            let mut runs: Vec<Vec<String>> = Vec::new();
            let mut current_run: Vec<String> = Vec::new();
            let mut previous_hash: Option<u64> = None;
            for (member_group_id, hash) in visual_hashes {
                let continues = previous_hash
                    .map(|previous| visual_hash_similarity(previous, hash) >= threshold)
                    .unwrap_or(true);
                if !continues && !current_run.is_empty() {
                    runs.push(std::mem::take(&mut current_run));
                }
                current_run.push(member_group_id);
                previous_hash = Some(hash);
            }
            if !current_run.is_empty() {
                runs.push(current_run);
            }
            if runs.len() <= 1 {
                transaction.commit()?;
                return Ok(vec![burst]);
            }

            transaction.execute(
                "DELETE FROM selection_recommendations
                 WHERE project_id = ?1 AND scope = ?2 AND subject_id = ?3",
                params![
                    burst.project_id,
                    SelectionRecommendationScope::BurstGroup.as_str(),
                    burst_group_id
                ],
            )?;
            transaction.execute(
                "DELETE FROM background_jobs
                 WHERE entity_type = ?1 AND entity_id = ?2 AND status IN ('pending', 'failed')",
                params![AnalysisEntityType::BurstGroup.as_str(), burst_group_id],
            )?;
            transaction.execute(
                "DELETE FROM burst_group_members WHERE burst_group_id = ?1",
                params![burst_group_id],
            )?;
            transaction.execute(
                "DELETE FROM burst_groups WHERE burst_group_id = ?1",
                params![burst_group_id],
            )?;

            let now = current_time_ms();
            let mut refined_bursts = Vec::new();
            for run in runs {
                if run.len() < profile.min_group_size {
                    continue;
                }
                let mut member_groups = Vec::new();
                for member_group_id in &run {
                    if let Some(group) =
                        stored_asset_group_by_id(&transaction, &burst.project_id, member_group_id)?
                    {
                        member_groups.push(group);
                    }
                }
                if member_groups.len() < profile.min_group_size {
                    continue;
                }
                let member_group_ids = member_groups
                    .iter()
                    .map(|group| group.group_id.clone())
                    .collect::<Vec<_>>();
                let stable_members = member_group_ids.join(",");
                let refined_burst_group_id = format!(
                    "burst-{}",
                    stable_key(&format!(
                        "{}\t{}\t{}",
                        burst.project_id, profile.grouping_version, stable_members
                    ))
                );
                let started_at_ms = member_groups
                    .iter()
                    .filter_map(|group| group.first_capture_at_ms.or(group.first_received_at_ms))
                    .min();
                let ended_at_ms = member_groups
                    .iter()
                    .filter_map(|group| group.first_capture_at_ms.or(group.first_received_at_ms))
                    .max();
                let refined = BurstGroup {
                    burst_group_id: refined_burst_group_id,
                    project_id: burst.project_id.clone(),
                    source_identity: common_burst_source_identity(
                        &transaction,
                        &burst.project_id,
                        &member_groups,
                    )?
                    .or_else(|| burst.source_identity.clone()),
                    started_at_ms,
                    ended_at_ms,
                    member_count: member_group_ids.len(),
                    member_group_ids,
                    grouping_version: burst.grouping_version + 1,
                    recommendation_status: SelectionRecommendationStatus::Pending
                        .as_str()
                        .to_string(),
                    manual_grouping_state: None,
                    created_at_ms: now,
                    updated_at_ms: now,
                };
                insert_burst_group(&transaction, &refined)?;
                refined_bursts.push(refined);
            }
            transaction.commit()?;
            Ok(refined_bursts)
        })
    }

    pub fn enqueue_analysis_job(&self, job: NewAnalysisJob) -> Result<AnalysisJob> {
        self.with_connection(|connection| enqueue_analysis_job_for_connection(connection, job))
    }

    pub fn claim_analysis_jobs(&self, now_ms: i64, limit: usize) -> Result<Vec<AnalysisJob>> {
        self.with_connection(|connection| {
            claim_analysis_jobs_for_connection(connection, now_ms, limit)
        })
    }

    pub fn complete_analysis_job(&self, job_id: &str) -> Result<()> {
        self.with_connection(|connection| {
            let changed = connection.execute(
                "UPDATE background_jobs
                 SET status = ?1, last_error = NULL, next_attempt_at_ms = NULL, updated_at_ms = ?2
                 WHERE job_id = ?3",
                params![
                    AnalysisJobStatus::Completed.as_str(),
                    current_time_ms(),
                    job_id
                ],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "analysis job not found".to_string(),
                )
                .into());
            }
            Ok(())
        })
    }

    pub fn fail_analysis_job(
        &self,
        job_id: &str,
        error: &str,
        next_attempt_at_ms: i64,
    ) -> Result<()> {
        self.with_connection(|connection| {
            let changed = connection.execute(
                "UPDATE background_jobs
                 SET status = ?1, attempts = attempts + 1, last_error = ?2,
                     next_attempt_at_ms = ?3, updated_at_ms = ?4
                 WHERE job_id = ?5",
                params![
                    AnalysisJobStatus::Failed.as_str(),
                    error,
                    next_attempt_at_ms,
                    current_time_ms(),
                    job_id,
                ],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "analysis job not found".to_string(),
                )
                .into());
            }
            Ok(())
        })
    }

    pub fn transfer_counts(&self, project_id: &str) -> Result<(usize, usize, usize)> {
        self.with_read_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let total = count_transfers(connection, project_id, None)?;
            let completed = count_transfers(connection, project_id, Some("completed"))?;
            let failed = count_transfers(connection, project_id, Some("failed"))?;
            Ok((total as usize, completed as usize, failed as usize))
        })
    }

    pub fn transfer_records(&self, project_id: &str) -> Result<Vec<TransferRecord>> {
        self.with_read_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let mut statement = connection.prepare(
                "SELECT transfer_id, protocol, status, original_path, final_filename,
                        final_location_payload, size_bytes, username, remote_addr, source_name,
                        started_at_ms, completed_at_ms, error
                 FROM transfers
                 WHERE project_id = ?1
                 ORDER BY COALESCE(completed_at_ms, started_at_ms) DESC,
                          started_at_ms DESC,
                          transfer_id DESC",
            )?;
            let rows = statement.query_map(params![project_id], transfer_record_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn create_desktop_scan_run(
        &self,
        project_id: &str,
        root_path: impl AsRef<Path>,
        now_ms: i64,
    ) -> Result<DesktopScanRun> {
        let root_path = root_path.as_ref().to_path_buf();
        let root_path_value = root_path.to_string_lossy().to_string();
        let root_key = desktop_scan_root_key(project_id, &root_path);
        let root_label = desktop_scan_root_label(&root_path);
        let scan_id = format!("desktop-scan-run-{root_key}-{now_ms}");
        self.with_connection(|connection| {
            ensure_project_is_active(connection, project_id)?;
            connection.execute(
                "INSERT INTO desktop_scan_runs (
                    scan_id, project_id, root_path, root_key, root_label, phase,
                    files_seen, assets_indexed, groups_updated, started_at_ms, updated_at_ms,
                    completed_at_ms, error
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, 0, ?7, ?7, NULL, NULL)",
                params![
                    scan_id,
                    project_id,
                    root_path_value,
                    root_key,
                    root_label,
                    DesktopScanPhase::Queued.as_str(),
                    now_ms,
                ],
            )?;
            desktop_scan_run_by_id(connection, &scan_id)?
                .ok_or_else(|| sqlite_data_error("desktop scan run not found"))
        })
    }

    pub fn desktop_scan_run(&self, scan_id: &str) -> Result<Option<DesktopScanRun>> {
        self.with_connection(|connection| desktop_scan_run_by_id(connection, scan_id))
    }

    pub fn latest_desktop_scan_run(&self, project_id: &str) -> Result<Option<DesktopScanRun>> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            connection
                .query_row(
                    "SELECT scan_id, project_id, root_path, root_key, root_label, phase,
                            files_seen, assets_indexed, groups_updated, started_at_ms, updated_at_ms,
                            completed_at_ms, error
                     FROM desktop_scan_runs
                     WHERE project_id = ?1
                     ORDER BY updated_at_ms DESC, started_at_ms DESC, scan_id DESC
                     LIMIT 1",
                    params![project_id],
                    desktop_scan_run_from_row,
                )
                .optional()
        })
    }

    pub fn update_desktop_scan_run(
        &self,
        scan_id: &str,
        phase: DesktopScanPhase,
        files_seen: usize,
        assets_indexed: usize,
        groups_updated: usize,
        error: Option<&str>,
        now_ms: i64,
    ) -> Result<DesktopScanRun> {
        self.with_connection(|connection| {
            let completed_at_ms = match phase {
                DesktopScanPhase::Completed
                | DesktopScanPhase::Failed
                | DesktopScanPhase::Cancelled => Some(now_ms),
                _ => None,
            };
            let changed = connection.execute(
                "UPDATE desktop_scan_runs
                 SET phase = ?2,
                     files_seen = ?3,
                     assets_indexed = ?4,
                     groups_updated = ?5,
                     updated_at_ms = ?6,
                     completed_at_ms = ?7,
                     error = ?8
                 WHERE scan_id = ?1",
                params![
                    scan_id,
                    phase.as_str(),
                    files_seen as i64,
                    assets_indexed as i64,
                    groups_updated as i64,
                    now_ms,
                    completed_at_ms,
                    error,
                ],
            )?;
            if changed == 0 {
                return Err(sqlite_data_error("desktop scan run not found"));
            }
            desktop_scan_run_by_id(connection, scan_id)?
                .ok_or_else(|| sqlite_data_error("desktop scan run not found"))
        })
    }

    pub fn record_desktop_scan_files(
        &self,
        scan_id: &str,
        files: &[DesktopScannedFile],
        now_ms: i64,
    ) -> Result<DesktopScanIndexResult> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let run = desktop_scan_run_by_id(&transaction, scan_id)?
                .ok_or_else(|| sqlite_data_error("desktop scan run not found"))?;
            ensure_project_is_active(&transaction, &run.project_id)?;
            transaction.execute(
                "UPDATE desktop_scan_runs
                 SET phase = ?2,
                     files_seen = ?3,
                     updated_at_ms = ?4
                 WHERE scan_id = ?1",
                params![
                    scan_id,
                    DesktopScanPhase::Indexing.as_str(),
                    files.len() as i64,
                    now_ms,
                ],
            )?;

            let mut group_ids = BTreeSet::new();
            let mut assets_indexed = 0usize;
            let source_name = run.root_path.display().to_string();

            for file in files {
                let transfer_id =
                    desktop_scan_transfer_id(&run.project_id, &run.root_path, &file.relative_path);
                let previous_state = desktop_scan_asset_source_state(&transaction, &transfer_id)?;
                let source_status = match previous_state {
                    Some((previous_modified_at_ms, previous_size_bytes))
                        if previous_modified_at_ms != Some(file.modified_at_ms)
                            || previous_size_bytes != file.size_bytes =>
                    {
                        DesktopSourceStatus::Changed
                    }
                    _ => DesktopSourceStatus::Available,
                };
                let record = TransferRecord {
                    transfer_id: transfer_id.clone(),
                    protocol: DESKTOP_SCAN_PROTOCOL.to_string(),
                    status: TransferStatus::Completed,
                    original_path: file.relative_path.clone(),
                    final_filename: file.original_filename.clone(),
                    final_location: Some(StoredObjectLocation::local_path(file.local_path.clone())),
                    size_bytes: file.size_bytes,
                    username: None,
                    remote_addr: None,
                    source_name: Some(source_name.clone()),
                    started_at_ms: run.started_at_ms,
                    completed_at_ms: Some(now_ms),
                    error: None,
                };
                insert_transfer(&transaction, &run.project_id, &record)?;
                if let Some(group_id) =
                    insert_asset_for_transfer(&transaction, &run.project_id, &record)?
                {
                    transaction.execute(
                        "UPDATE assets
                         SET source_status = ?1,
                             source_modified_at_ms = ?2,
                             last_seen_scan_id = ?3,
                             capture_at_ms = COALESCE(capture_at_ms, ?4)
                         WHERE transfer_id = ?5",
                        params![
                            source_status.as_str(),
                            file.modified_at_ms,
                            scan_id,
                            file.capture_time_ms,
                            transfer_id,
                        ],
                    )?;
                    group_ids.insert(group_id);
                    assets_indexed += 1;
                }
            }

            let root_prefix = format!(
                "desktop-scan-{}-%",
                desktop_scan_root_key(&run.project_id, &run.root_path)
            );
            let missing_group_ids = desktop_scan_missing_group_ids(
                &transaction,
                &run.project_id,
                &root_prefix,
                scan_id,
            )?;
            if !missing_group_ids.is_empty() {
                transaction.execute(
                    "UPDATE assets
                     SET source_status = ?4
                     WHERE project_id = ?1
                       AND transfer_id LIKE ?2
                       AND (last_seen_scan_id IS NULL OR last_seen_scan_id <> ?3)",
                    params![
                        run.project_id,
                        root_prefix,
                        scan_id,
                        DesktopSourceStatus::Missing.as_str(),
                    ],
                )?;
                for group_id in missing_group_ids {
                    refresh_group_rollup(&transaction, &group_id)?;
                    group_ids.insert(group_id);
                }
            }

            let group_ids = group_ids.into_iter().collect::<Vec<_>>();
            transaction.execute(
                "UPDATE desktop_scan_runs
                 SET phase = ?2,
                     files_seen = ?3,
                     assets_indexed = ?4,
                     groups_updated = ?5,
                     updated_at_ms = ?6,
                     completed_at_ms = ?6,
                     error = NULL
                 WHERE scan_id = ?1",
                params![
                    scan_id,
                    DesktopScanPhase::Completed.as_str(),
                    files.len() as i64,
                    assets_indexed as i64,
                    group_ids.len() as i64,
                    now_ms,
                ],
            )?;
            transaction.commit()?;
            Ok(DesktopScanIndexResult {
                assets_indexed,
                group_ids,
            })
        })
    }

    pub fn enqueue_publish(
        &self,
        project_id: &str,
        transfer_id: &str,
        staged_path: &str,
        final_filename: &str,
        size_bytes: u64,
    ) -> Result<PublishQueueItem> {
        let now = current_time_ms();
        let item = PublishQueueItem {
            queue_id: format!("publish-{now}-{transfer_id}"),
            project_id: project_id.to_string(),
            transfer_id: transfer_id.to_string(),
            staged_path: staged_path.to_string(),
            final_filename: final_filename.to_string(),
            size_bytes,
            protocol: None,
            original_path: None,
            username: None,
            remote_addr: None,
            source_name: None,
            started_at_ms: None,
            state: PublishState::Staged,
            attempt_count: 0,
            last_error: None,
            next_attempt_at_ms: None,
            created_at_ms: now,
            updated_at_ms: now,
        };
        self.with_connection(|connection| {
            ensure_project_is_active(connection, project_id)?;
            connection.execute(
                "INSERT INTO publish_queue (
                    queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                    state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    item.queue_id,
                    item.project_id,
                    item.transfer_id,
                    item.staged_path,
                    item.final_filename,
                    item.size_bytes as i64,
                    item.state.as_str(),
                    item.attempt_count as i64,
                    item.last_error,
                    item.next_attempt_at_ms,
                    item.created_at_ms,
                    item.updated_at_ms,
                ],
            )?;
            Ok(item)
        })
    }

    pub fn enqueue_publish_with_metadata(
        &self,
        project_id: &str,
        transfer_id: &str,
        staged_path: &str,
        final_filename: &str,
        size_bytes: u64,
        metadata: PublishTransferMetadata,
    ) -> Result<PublishQueueItem> {
        let now = current_time_ms();
        let item = PublishQueueItem {
            queue_id: format!("publish-{now}-{transfer_id}"),
            project_id: project_id.to_string(),
            transfer_id: transfer_id.to_string(),
            staged_path: staged_path.to_string(),
            final_filename: final_filename.to_string(),
            size_bytes,
            protocol: Some(metadata.protocol),
            original_path: Some(metadata.original_path),
            username: metadata.username,
            remote_addr: metadata.remote_addr,
            source_name: metadata.source_name,
            started_at_ms: Some(metadata.started_at_ms),
            state: PublishState::Staged,
            attempt_count: 0,
            last_error: None,
            next_attempt_at_ms: None,
            created_at_ms: now,
            updated_at_ms: now,
        };
        self.with_connection(|connection| {
            ensure_project_is_active(connection, project_id)?;
            connection.execute(
                "INSERT INTO publish_queue (
                    queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                    protocol, original_path, username, remote_addr, source_name, started_at_ms,
                    state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    item.queue_id,
                    item.project_id,
                    item.transfer_id,
                    item.staged_path,
                    item.final_filename,
                    item.size_bytes as i64,
                    item.protocol,
                    item.original_path,
                    item.username,
                    item.remote_addr,
                    item.source_name,
                    item.started_at_ms,
                    item.state.as_str(),
                    item.attempt_count as i64,
                    item.last_error,
                    item.next_attempt_at_ms,
                    item.created_at_ms,
                    item.updated_at_ms,
                ],
            )?;
            Ok(item)
        })
    }

    pub fn complete_publish(
        &self,
        queue_id: &str,
        final_filename: &str,
        final_location: StoredObjectLocation,
    ) -> Result<TransferRecord> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let item = transaction.query_row(
                "SELECT queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                        protocol, original_path, username, remote_addr, source_name, started_at_ms,
                        state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 FROM publish_queue
                 WHERE queue_id = ?1",
                params![queue_id],
                publish_item_from_row,
            )?;
            ensure_project_is_active(&transaction, &item.project_id)?;
            let protocol = item.protocol.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName(
                    "publish queue item missing protocol".to_string(),
                )
            })?;
            let original_path = item.original_path.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName(
                    "publish queue item missing original path".to_string(),
                )
            })?;
            let started_at_ms = item.started_at_ms.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName(
                    "publish queue item missing started time".to_string(),
                )
            })?;
            let record = TransferRecord {
                transfer_id: item.transfer_id,
                protocol,
                status: TransferStatus::Completed,
                original_path,
                final_filename: final_filename.to_string(),
                final_location: Some(final_location),
                size_bytes: item.size_bytes,
                username: item.username,
                remote_addr: item.remote_addr,
                source_name: item.source_name,
                started_at_ms,
                completed_at_ms: Some(current_time_ms()),
                error: None,
            };
            insert_transfer(&transaction, &item.project_id, &record)?;
            let asset_group_id = insert_asset_for_transfer(&transaction, &item.project_id, &record)?;
            if let Some(asset_group_id) = asset_group_id {
                enqueue_detect_burst_job_for_connection(
                    &transaction,
                    &item.project_id,
                    &asset_group_id,
                )?;
            }
            let changed = transaction.execute(
                "UPDATE publish_queue
                 SET state = ?1, last_error = NULL, next_attempt_at_ms = NULL, updated_at_ms = ?2
                 WHERE queue_id = ?3",
                params![
                    PublishState::Completed.as_str(),
                    current_time_ms(),
                    queue_id
                ],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "publish queue item not found".to_string(),
                ));
            }
            transaction.commit()?;
            Ok(record)
        })
    }

    pub fn mark_publish_failed(&self, queue_id: &str, error: &str) -> Result<()> {
        self.with_connection(|connection| {
            let now = current_time_ms();
            let next_attempt_at_ms = now.saturating_add(FAILED_PUBLISH_RETRY_DELAY_MS);
            let changed = connection.execute(
                "UPDATE publish_queue
                 SET state = ?1, attempt_count = attempt_count + 1, last_error = ?2,
                     next_attempt_at_ms = ?3, updated_at_ms = ?4
                 WHERE queue_id = ?5",
                params![
                    PublishState::Failed.as_str(),
                    error,
                    next_attempt_at_ms,
                    now,
                    queue_id
                ],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "publish queue item not found".to_string(),
                ));
            }
            Ok(())
        })
    }

    pub fn mark_publish_completed(&self, queue_id: &str) -> Result<()> {
        self.with_connection(|connection| {
            let changed = connection.execute(
                "UPDATE publish_queue
                 SET state = ?1, last_error = NULL, next_attempt_at_ms = NULL, updated_at_ms = ?2
                 WHERE queue_id = ?3",
                params![
                    PublishState::Completed.as_str(),
                    current_time_ms(),
                    queue_id
                ],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "publish queue item not found".to_string(),
                ));
            }
            Ok(())
        })
    }

    pub fn pending_publish_items(&self) -> Result<Vec<PublishQueueItem>> {
        self.with_read_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                        protocol, original_path, username, remote_addr, source_name, started_at_ms,
                        state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 FROM publish_queue
                 WHERE state IN ('staged', 'failed')
                 ORDER BY created_at_ms ASC, queue_id ASC",
            )?;
            let rows = statement.query_map([], publish_item_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn failed_publish_items(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<Vec<PublishQueueItem>> {
        self.with_read_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let mut statement = connection.prepare(
                "SELECT queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                        protocol, original_path, username, remote_addr, source_name, started_at_ms,
                        state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 FROM publish_queue
                 WHERE project_id = ?1 AND state = 'failed'
                 ORDER BY updated_at_ms DESC, created_at_ms DESC, queue_id DESC
                 LIMIT ?2",
            )?;
            let rows =
                statement.query_map(params![project_id, limit as i64], publish_item_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn release_failed_publish_retries(&self, project_id: &str) -> Result<usize> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let changed = connection.execute(
                "UPDATE publish_queue
                 SET next_attempt_at_ms = NULL, updated_at_ms = ?1
                 WHERE project_id = ?2
                   AND state = 'failed'
                   AND next_attempt_at_ms IS NOT NULL",
                params![current_time_ms(), project_id],
            )?;
            Ok(changed)
        })
    }

    pub fn claim_next_publish_item(&self) -> Result<Option<PublishQueueItem>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let now = current_time_ms();
            let queue_id = transaction
                .query_row(
                    "SELECT queue_id
                     FROM publish_queue
                     WHERE state = 'staged'
                        OR (
                            state = 'failed'
                            AND (next_attempt_at_ms IS NULL OR next_attempt_at_ms <= ?1)
                        )
                     ORDER BY created_at_ms ASC, queue_id ASC
                     LIMIT 1",
                    params![now],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            let Some(queue_id) = queue_id else {
                transaction.commit()?;
                return Ok(None);
            };
            transaction.execute(
                "UPDATE publish_queue
                 SET state = ?1, last_error = NULL, next_attempt_at_ms = NULL, updated_at_ms = ?2
                 WHERE queue_id = ?3
                   AND (
                       state = 'staged'
                       OR (
                           state = 'failed'
                           AND (next_attempt_at_ms IS NULL OR next_attempt_at_ms <= ?4)
                       )
                   )",
                params![PublishState::Publishing.as_str(), now, queue_id, now],
            )?;
            let item = transaction.query_row(
                "SELECT queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                        protocol, original_path, username, remote_addr, source_name, started_at_ms,
                        state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 FROM publish_queue
                 WHERE queue_id = ?1",
                params![queue_id],
                publish_item_from_row,
            )?;
            transaction.commit()?;
            Ok(Some(item))
        })
    }

    pub fn publish_queue_summary(&self, project_id: &str) -> Result<PublishQueueSummary> {
        self.with_read_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let mut statement = connection.prepare(
                "SELECT state, COUNT(*)
                 FROM publish_queue
                 WHERE project_id = ?1
                 GROUP BY state",
            )?;
            let rows = statement.query_map(params![project_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            let mut summary = PublishQueueSummary::default();
            for row in rows {
                let (state, count) = row?;
                let count = count as usize;
                summary.total_count += count;
                match PublishState::from_str(&state) {
                    PublishState::Staged => summary.staged_count += count,
                    PublishState::Publishing => summary.publishing_count += count,
                    PublishState::Completed => summary.completed_count += count,
                    PublishState::Failed => summary.failed_count += count,
                }
            }
            summary.pending_count =
                summary.staged_count + summary.publishing_count + summary.failed_count;
            Ok(summary)
        })
    }

    fn with_connection<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> std::result::Result<T, rusqlite::Error>,
    ) -> Result<T> {
        self.with_write_connection(operation)
    }

    fn with_read_connection<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> std::result::Result<T, rusqlite::Error>,
    ) -> Result<T> {
        let mut connection = self.open_configured_connection()?;
        connection
            .execute_batch("PRAGMA query_only = ON;")
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        operation(&mut connection).map_err(|error| ImporterError::internal(error.to_string()))
    }

    fn with_write_connection<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> std::result::Result<T, rusqlite::Error>,
    ) -> Result<T> {
        let _access_guard = self
            .access_lock
            .lock()
            .map_err(|_| ImporterError::internal("sqlite access lock poisoned"))?;
        let mut connection = self.open_configured_connection()?;
        operation(&mut connection).map_err(|error| ImporterError::internal(error.to_string()))
    }

    fn open_configured_connection(&self) -> Result<Connection> {
        let connection = Connection::open(&self.db_path)
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        connection
            .busy_timeout(SQLITE_BUSY_TIMEOUT)
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        Ok(connection)
    }
}

static SQLITE_ACCESS_LOCKS: OnceLock<Mutex<BTreeMap<PathBuf, Arc<Mutex<()>>>>> = OnceLock::new();

fn sqlite_access_lock(db_path: &Path) -> Arc<Mutex<()>> {
    let key = sqlite_lock_key(db_path);
    let mut locks = SQLITE_ACCESS_LOCKS
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .expect("sqlite access lock registry should not be poisoned");
    locks
        .entry(key)
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

fn sqlite_lock_key(db_path: &Path) -> PathBuf {
    let file_name = db_path.file_name().unwrap_or_default();
    db_path
        .parent()
        .and_then(|parent| std::fs::canonicalize(parent).ok())
        .map(|parent| parent.join(file_name))
        .unwrap_or_else(|| db_path.to_path_buf())
}

fn count_transfers(
    connection: &Connection,
    project_id: &str,
    status: Option<&str>,
) -> std::result::Result<i64, rusqlite::Error> {
    match status {
        Some(status) => connection.query_row(
            "SELECT COUNT(*) FROM transfers WHERE project_id = ?1 AND status = ?2",
            params![project_id, status],
            |row| row.get(0),
        ),
        None => connection.query_row(
            "SELECT COUNT(*) FROM transfers WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        ),
    }
}

fn stored_asset_groups_for_project(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<Vec<StoredAssetGroup>, rusqlite::Error> {
    ensure_project_exists(connection, project_id)?;
    let mut statement = connection.prepare(
        "SELECT group_id, project_id, group_identity, display_key, source_identity,
                original_parent_path, primary_asset_id, preview_asset_id, member_count,
                has_raw, has_jpeg, has_video, first_capture_at_ms, last_capture_at_ms,
                first_received_at_ms, last_received_at_ms, created_at_ms, updated_at_ms
         FROM asset_groups
         WHERE project_id = ?1
         ORDER BY COALESCE(last_received_at_ms, updated_at_ms) DESC,
                  display_key ASC,
                  group_id ASC",
    )?;
    let rows = statement.query_map(params![project_id], stored_asset_group_from_row)?;
    collect_rows(rows)
}

fn stored_asset_group_by_id(
    connection: &Connection,
    project_id: &str,
    group_id: &str,
) -> std::result::Result<Option<StoredAssetGroup>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT group_id, project_id, group_identity, display_key, source_identity,
                    original_parent_path, primary_asset_id, preview_asset_id, member_count,
                    has_raw, has_jpeg, has_video, first_capture_at_ms, last_capture_at_ms,
                    first_received_at_ms, last_received_at_ms, created_at_ms, updated_at_ms
             FROM asset_groups
             WHERE project_id = ?1 AND group_id = ?2",
            params![project_id, group_id],
            stored_asset_group_from_row,
        )
        .optional()
}

#[derive(Debug, Clone)]
struct BurstCandidate {
    group: StoredAssetGroup,
    source_identity: Option<String>,
    sequence_number: Option<i64>,
    event_time_ms: Option<i64>,
    event_time_is_capture: bool,
}

fn rebuild_burst_groups_for_project(
    connection: &Connection,
    project_id: &str,
    groups: Vec<StoredAssetGroup>,
    profile: &BurstGroupingProfile,
) -> std::result::Result<Vec<BurstGroup>, rusqlite::Error> {
    connection.execute(
        "DELETE FROM burst_group_members
         WHERE burst_group_id IN (
            SELECT burst_group_id FROM burst_groups WHERE project_id = ?1
         )",
        params![project_id],
    )?;
    connection.execute(
        "DELETE FROM burst_groups WHERE project_id = ?1",
        params![project_id],
    )?;

    let manual_merge_groups = manual_merge_member_groups(connection, project_id)?;
    let manual_merge_member_group_ids = manual_merge_groups
        .values()
        .flat_map(|member_group_ids| member_group_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let split_excluded_member_group_ids =
        manual_split_excluded_member_group_ids(connection, project_id)?;
    let mut candidates = groups
        .into_iter()
        .filter(|group| group.has_jpeg || group.has_raw)
        .filter(|group| !split_excluded_member_group_ids.contains(&group.group_id))
        .filter(|group| !manual_merge_member_group_ids.contains(&group.group_id))
        .map(|group| {
            let source_identity =
                burst_source_identity_for_group(connection, project_id, &group.group_id)?
                    .or_else(|| group.source_identity.clone());
            let sequence_number = trailing_sequence_number(&group.display_key);
            let (event_time_ms, event_time_is_capture) = match group.first_capture_at_ms {
                Some(value) => (Some(value), true),
                None => (
                    group.first_received_at_ms.or(Some(group.created_at_ms)),
                    false,
                ),
            };
            Ok(BurstCandidate {
                group,
                source_identity,
                sequence_number,
                event_time_ms,
                event_time_is_capture,
            })
        })
        .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;

    candidates.sort_by(|left, right| {
        (
            left.source_identity.as_deref().unwrap_or_default(),
            left.group
                .original_parent_path
                .as_deref()
                .unwrap_or_default(),
            left.sequence_number.unwrap_or(i64::MAX),
            left.event_time_ms.unwrap_or(i64::MAX),
            left.group.display_key.as_str(),
            left.group.group_id.as_str(),
        )
            .cmp(&(
                right.source_identity.as_deref().unwrap_or_default(),
                right
                    .group
                    .original_parent_path
                    .as_deref()
                    .unwrap_or_default(),
                right.sequence_number.unwrap_or(i64::MAX),
                right.event_time_ms.unwrap_or(i64::MAX),
                right.group.display_key.as_str(),
                right.group.group_id.as_str(),
            ))
    });

    let mut runs: Vec<Vec<BurstCandidate>> = Vec::new();
    let mut current_run: Vec<BurstCandidate> = Vec::new();
    for candidate in candidates {
        let continues_current_run = current_run
            .last()
            .map(|previous| burst_candidates_are_adjacent(previous, &candidate, profile))
            .unwrap_or(false);
        if !continues_current_run && !current_run.is_empty() {
            runs.push(std::mem::take(&mut current_run));
        }
        current_run.push(candidate);
    }
    if !current_run.is_empty() {
        runs.push(current_run);
    }

    let now = current_time_ms();
    let mut bursts = Vec::new();
    for run in runs {
        if run.len() < profile.min_group_size {
            continue;
        }
        let member_group_ids = run
            .iter()
            .map(|candidate| candidate.group.group_id.clone())
            .collect::<Vec<_>>();
        let stable_members = member_group_ids.join(",");
        let burst_group_id = format!(
            "burst-{}",
            stable_key(&format!(
                "{project_id}\t{}\t{stable_members}",
                profile.grouping_version
            ))
        );
        let started_at_ms = run
            .iter()
            .filter_map(|candidate| candidate.event_time_ms)
            .min();
        let ended_at_ms = run
            .iter()
            .filter_map(|candidate| candidate.event_time_ms)
            .max();
        let source_identity = run
            .first()
            .and_then(|candidate| candidate.source_identity.clone());
        let burst = BurstGroup {
            burst_group_id: burst_group_id.clone(),
            project_id: project_id.to_string(),
            source_identity,
            started_at_ms,
            ended_at_ms,
            member_count: member_group_ids.len(),
            member_group_ids,
            grouping_version: 1,
            recommendation_status: SelectionRecommendationStatus::Pending.as_str().to_string(),
            manual_grouping_state: None,
            created_at_ms: now,
            updated_at_ms: now,
        };
        insert_burst_group(connection, &burst)?;
        bursts.push(burst);
    }

    for (burst_group_id, member_group_ids) in manual_merge_groups {
        let mut member_groups = Vec::new();
        for member_group_id in member_group_ids {
            if let Some(group) = stored_asset_group_by_id(connection, project_id, &member_group_id)?
            {
                member_groups.push(group);
            }
        }
        member_groups.sort_by(|left, right| {
            (
                left.first_capture_at_ms
                    .or(left.first_received_at_ms)
                    .or(Some(left.created_at_ms)),
                left.display_key.as_str(),
                left.group_id.as_str(),
            )
                .cmp(&(
                    right
                        .first_capture_at_ms
                        .or(right.first_received_at_ms)
                        .or(Some(right.created_at_ms)),
                    right.display_key.as_str(),
                    right.group_id.as_str(),
                ))
        });
        if member_groups.len() < profile.min_group_size {
            continue;
        }

        let member_group_ids = member_groups
            .iter()
            .map(|group| group.group_id.clone())
            .collect::<Vec<_>>();
        let started_at_ms = member_groups
            .iter()
            .filter_map(|group| group.first_capture_at_ms.or(group.first_received_at_ms))
            .min();
        let ended_at_ms = member_groups
            .iter()
            .filter_map(|group| group.first_capture_at_ms.or(group.first_received_at_ms))
            .max();
        let source_identity = common_burst_source_identity(connection, project_id, &member_groups)?;
        let burst = BurstGroup {
            burst_group_id,
            project_id: project_id.to_string(),
            source_identity,
            started_at_ms,
            ended_at_ms,
            member_count: member_group_ids.len(),
            member_group_ids,
            grouping_version: 1,
            recommendation_status: SelectionRecommendationStatus::Pending.as_str().to_string(),
            manual_grouping_state: Some("merge".to_string()),
            created_at_ms: now,
            updated_at_ms: now,
        };
        insert_burst_group(connection, &burst)?;
        bursts.push(burst);
    }

    Ok(bursts)
}

fn burst_source_identity_for_group(
    connection: &Connection,
    project_id: &str,
    group_id: &str,
) -> std::result::Result<Option<String>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT COALESCE(NULLIF(username, ''), NULLIF(source_identity, ''), NULLIF(remote_addr, ''))
             FROM assets
             WHERE project_id = ?1 AND group_id = ?2
             ORDER BY CASE group_role
                        WHEN 'jpeg' THEN 0
                        WHEN 'raw' THEN 1
                        WHEN 'video' THEN 2
                        ELSE 3
                      END ASC,
                      published_at_ms ASC,
                      asset_id ASC
             LIMIT 1",
            params![project_id, group_id],
            |row| row.get(0),
        )
        .optional()
        .map(|value| value.flatten())
}

fn common_burst_source_identity(
    connection: &Connection,
    project_id: &str,
    groups: &[StoredAssetGroup],
) -> std::result::Result<Option<String>, rusqlite::Error> {
    let mut common: Option<String> = None;
    for group in groups {
        let source_identity =
            burst_source_identity_for_group(connection, project_id, &group.group_id)?
                .or_else(|| group.source_identity.clone());
        match (&common, source_identity) {
            (None, Some(value)) => common = Some(value),
            (Some(left), Some(right)) if left == &right => {}
            (Some(_), Some(_)) => return Ok(None),
            (_, None) => {}
        }
    }
    Ok(common)
}

fn manual_split_excluded_member_group_ids(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<BTreeSet<String>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT member_group_id
         FROM burst_member_manual_edits
         WHERE project_id = ?1 AND action = 'split_exclude'",
    )?;
    let rows = statement.query_map(params![project_id], |row| row.get::<_, String>(0))?;
    collect_rows(rows).map(|values| values.into_iter().collect())
}

fn manual_merge_member_groups(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<BTreeMap<String, Vec<String>>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT manual_group_id, member_group_id
         FROM burst_member_manual_edits
         WHERE project_id = ?1
           AND action = 'merge_include'
           AND manual_group_id IS NOT NULL
         ORDER BY manual_group_id ASC, member_group_id ASC",
    )?;
    let rows = statement.query_map(params![project_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut groups = BTreeMap::new();
    for row in rows {
        let (manual_group_id, member_group_id) = row?;
        groups
            .entry(manual_group_id)
            .or_insert_with(Vec::new)
            .push(member_group_id);
    }
    Ok(groups)
}

fn burst_candidates_are_adjacent(
    previous: &BurstCandidate,
    candidate: &BurstCandidate,
    profile: &BurstGroupingProfile,
) -> bool {
    if previous.source_identity != candidate.source_identity
        || previous.group.original_parent_path != candidate.group.original_parent_path
    {
        return false;
    }

    let time_is_adjacent = previous.event_time_is_capture
        && candidate.event_time_is_capture
        && previous
            .event_time_ms
            .zip(candidate.event_time_ms)
            .map(|(left, right)| right >= left && right - left <= profile.burst_window_ms)
            .unwrap_or(false);

    if previous.event_time_is_capture || candidate.event_time_is_capture {
        return time_is_adjacent;
    }

    previous
        .sequence_number
        .zip(candidate.sequence_number)
        .map(|(left, right)| right > left && right - left <= 1)
        .unwrap_or(false)
}

fn visual_hash_from_signature(value: Option<&str>) -> Option<u64> {
    let value = value?;
    let hex = value.strip_prefix("ahash-v1:")?;
    u64::from_str_radix(hex, 16).ok()
}

fn visual_hash_similarity(left: u64, right: u64) -> f64 {
    1.0 - ((left ^ right).count_ones() as f64 / 64.0)
}

fn visual_burst_continuity_threshold(profile: &BurstGroupingProfile) -> f64 {
    profile.visual_continuity_threshold.clamp(0.70, 0.90)
}

fn insert_burst_group(
    connection: &Connection,
    burst: &BurstGroup,
) -> std::result::Result<(), rusqlite::Error> {
    connection.execute(
        "INSERT INTO burst_groups (
            burst_group_id, project_id, source_identity, started_at_ms, ended_at_ms,
            member_count, grouping_version, recommendation_status, manual_grouping_state,
            created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            burst.burst_group_id,
            burst.project_id,
            burst.source_identity,
            burst.started_at_ms,
            burst.ended_at_ms,
            burst.member_count as i64,
            burst.grouping_version,
            burst.recommendation_status,
            burst.manual_grouping_state,
            burst.created_at_ms,
            burst.updated_at_ms,
        ],
    )?;

    for member_group_id in burst.member_group_ids.iter() {
        connection.execute(
            "INSERT INTO burst_group_members (burst_group_id, member_group_id)
             VALUES (?1, ?2)",
            params![burst.burst_group_id, member_group_id],
        )?;
    }
    Ok(())
}

fn burst_summary_for_asset_group(
    connection: &Connection,
    project_id: &str,
    asset_group_id: &str,
) -> std::result::Result<Option<ReceivedAssetBurstSummary>, rusqlite::Error> {
    let summary = connection
        .query_row(
            "SELECT bg.burst_group_id, bg.member_count, bg.recommendation_status
             FROM burst_group_members bgm
             JOIN burst_groups bg ON bg.burst_group_id = bgm.burst_group_id
             WHERE bg.project_id = ?1 AND bgm.member_group_id = ?2
             ORDER BY bg.updated_at_ms DESC, bg.burst_group_id DESC
             LIMIT 1",
            params![project_id, asset_group_id],
            |row| {
                Ok(ReceivedAssetBurstSummary {
                    burst_group_id: row.get(0)?,
                    member_count: row.get::<_, i64>(1)? as usize,
                    recommendation_status: row.get(2)?,
                    best_asset_group_id: None,
                    best_score: None,
                })
            },
        )
        .optional()?;
    summary
        .map(|mut summary| {
            let mut selected_asset_group_id = None;
            if let Some(recommendation) = latest_selection_recommendation_for_connection(
                connection,
                project_id,
                SelectionRecommendationScope::BurstGroup,
                &summary.burst_group_id,
            )? {
                summary.recommendation_status = recommendation.status.as_str().to_string();
                selected_asset_group_id = recommendation.selected_asset_group_ids.first().cloned();
                summary.best_asset_group_id = selected_asset_group_id.clone();
            }
            summary.best_score = selected_asset_group_id
                .as_deref()
                .map(|asset_group_id| burst_selected_model_score(connection, asset_group_id))
                .transpose()?
                .flatten();
            Ok(summary)
        })
        .transpose()
}

fn burst_selected_model_score(
    connection: &Connection,
    asset_group_id: &str,
) -> std::result::Result<Option<f64>, rusqlite::Error> {
    connection.query_row(
        "SELECT MAX(me.score)
         FROM model_evaluations me
         WHERE me.asset_group_id = ?1
           AND me.status = 'ready'
           AND me.updated_at_ms = (
               SELECT MAX(latest.updated_at_ms)
               FROM model_evaluations latest
               WHERE latest.asset_group_id = me.asset_group_id
           )",
        params![asset_group_id],
        |row| {
            row.get::<_, Option<i64>>(0)
                .map(|score| score.map(|value| value as f64))
        },
    )
}

fn burst_group_by_id(
    connection: &Connection,
    burst_group_id: &str,
) -> std::result::Result<Option<BurstGroup>, rusqlite::Error> {
    let Some(mut burst) = connection
        .query_row(
            "SELECT burst_group_id, project_id, source_identity, started_at_ms, ended_at_ms,
                    member_count, grouping_version, recommendation_status, manual_grouping_state,
                    created_at_ms, updated_at_ms
             FROM burst_groups
             WHERE burst_group_id = ?1",
            params![burst_group_id],
            |row| {
                Ok(BurstGroup {
                    burst_group_id: row.get(0)?,
                    project_id: row.get(1)?,
                    source_identity: row.get(2)?,
                    started_at_ms: row.get(3)?,
                    ended_at_ms: row.get(4)?,
                    member_count: row.get::<_, i64>(5)? as usize,
                    member_group_ids: Vec::new(),
                    grouping_version: row.get(6)?,
                    recommendation_status: row.get(7)?,
                    manual_grouping_state: row.get(8)?,
                    created_at_ms: row.get(9)?,
                    updated_at_ms: row.get(10)?,
                })
            },
        )
        .optional()?
    else {
        return Ok(None);
    };

    let mut statement = connection.prepare(
        "SELECT bgm.member_group_id
         FROM burst_group_members bgm
         LEFT JOIN asset_groups ag ON ag.group_id = bgm.member_group_id
         WHERE bgm.burst_group_id = ?1
         ORDER BY COALESCE(ag.first_capture_at_ms, ag.first_received_at_ms, ag.created_at_ms) ASC,
                  ag.display_key ASC,
                  bgm.member_group_id ASC",
    )?;
    let rows = statement.query_map(params![burst_group_id], |row| row.get(0))?;
    burst.member_group_ids = collect_rows(rows)?;
    burst.member_count = burst.member_group_ids.len();
    Ok(Some(burst))
}

fn split_burst_member_for_connection(
    connection: &Connection,
    burst_group_id: &str,
    member_group_id: &str,
) -> std::result::Result<Option<BurstGroup>, rusqlite::Error> {
    let Some(burst) = burst_group_by_id(connection, burst_group_id)? else {
        return Ok(None);
    };
    let member_group_id = member_group_id.trim();
    if member_group_id.is_empty() {
        return Err(sqlite_data_error("member group id cannot be empty"));
    }
    if !burst
        .member_group_ids
        .iter()
        .any(|group_id| group_id == member_group_id)
    {
        return Err(sqlite_data_error("member group is not in burst group"));
    }

    let remaining_member_ids = burst
        .member_group_ids
        .iter()
        .filter(|group_id| group_id.as_str() != member_group_id)
        .cloned()
        .collect::<Vec<_>>();
    let now = current_time_ms();

    connection.execute(
        "DELETE FROM selection_recommendations
         WHERE project_id = ?1 AND scope = ?2 AND subject_id = ?3",
        params![
            burst.project_id,
            SelectionRecommendationScope::BurstGroup.as_str(),
            burst_group_id
        ],
    )?;
    connection.execute(
        "INSERT OR REPLACE INTO burst_member_manual_edits (
            project_id, member_group_id, action, manual_group_id, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            burst.project_id,
            member_group_id,
            "split_exclude",
            None::<String>,
            now
        ],
    )?;

    if remaining_member_ids.len() < 2 {
        connection.execute(
            "DELETE FROM burst_group_members WHERE burst_group_id = ?1",
            params![burst_group_id],
        )?;
        connection.execute(
            "DELETE FROM burst_groups WHERE burst_group_id = ?1",
            params![burst_group_id],
        )?;
        return Ok(None);
    }

    connection.execute(
        "DELETE FROM burst_group_members WHERE burst_group_id = ?1",
        params![burst_group_id],
    )?;
    for member_group_id in remaining_member_ids.iter() {
        connection.execute(
            "INSERT INTO burst_group_members (burst_group_id, member_group_id)
             VALUES (?1, ?2)",
            params![burst_group_id, member_group_id],
        )?;
    }
    connection.execute(
        "UPDATE burst_groups
         SET member_count = ?1,
             grouping_version = grouping_version + 1,
             recommendation_status = ?2,
             manual_grouping_state = ?3,
             updated_at_ms = ?4
         WHERE burst_group_id = ?5",
        params![
            remaining_member_ids.len() as i64,
            SelectionRecommendationStatus::Pending.as_str(),
            "split",
            now,
            burst_group_id,
        ],
    )?;

    burst_group_by_id(connection, burst_group_id)
}

fn create_manual_burst_group_for_connection(
    connection: &Connection,
    project_id: &str,
    member_group_ids: &[String],
) -> std::result::Result<Option<BurstGroup>, rusqlite::Error> {
    ensure_project_exists(connection, project_id)?;

    let mut expanded_member_ids = Vec::new();
    let mut affected_burst_ids = Vec::new();
    let mut requested_container_ids = Vec::new();
    for raw_member_group_id in member_group_ids {
        let member_group_id = raw_member_group_id.trim();
        if member_group_id.is_empty() {
            continue;
        }
        if stored_asset_group_by_id(connection, project_id, member_group_id)?.is_none() {
            return Err(sqlite_data_error(
                "member group not found in target project",
            ));
        }
        if let Some(source_burst) =
            burst_group_for_member_group(connection, project_id, member_group_id)?
        {
            push_unique_string(
                &mut requested_container_ids,
                source_burst.burst_group_id.clone(),
            );
            push_unique_string(&mut affected_burst_ids, source_burst.burst_group_id.clone());
            for source_member_group_id in source_burst.member_group_ids {
                push_unique_string(&mut expanded_member_ids, source_member_group_id);
            }
        } else {
            push_unique_string(&mut requested_container_ids, member_group_id.to_string());
            push_unique_string(&mut expanded_member_ids, member_group_id.to_string());
        }
    }

    if requested_container_ids.len() < 2 || expanded_member_ids.len() < 2 {
        return Ok(None);
    }

    let mut stable_member_ids = expanded_member_ids.clone();
    stable_member_ids.sort();
    let manual_burst_group_id = format!(
        "manual-burst-{}",
        stable_key(&format!("{project_id}\t{}", stable_member_ids.join(",")))
    );
    let now = current_time_ms();

    let mut cleanup_burst_ids = affected_burst_ids.clone();
    push_unique_string(&mut cleanup_burst_ids, manual_burst_group_id.clone());
    for burst_group_id in cleanup_burst_ids.iter() {
        connection.execute(
            "DELETE FROM selection_recommendations
             WHERE project_id = ?1 AND scope = ?2 AND subject_id = ?3",
            params![
                project_id,
                SelectionRecommendationScope::BurstGroup.as_str(),
                burst_group_id,
            ],
        )?;
        connection.execute(
            "DELETE FROM burst_group_members WHERE burst_group_id = ?1",
            params![burst_group_id],
        )?;
        connection.execute(
            "DELETE FROM burst_groups WHERE burst_group_id = ?1",
            params![burst_group_id],
        )?;
    }

    let mut member_groups = Vec::new();
    for member_group_id in expanded_member_ids.iter() {
        if let Some(group) = stored_asset_group_by_id(connection, project_id, member_group_id)? {
            member_groups.push(group);
        }
        connection.execute(
            "DELETE FROM burst_member_manual_edits
             WHERE project_id = ?1 AND member_group_id = ?2
               AND action IN ('split_exclude', 'merge_include')",
            params![project_id, member_group_id],
        )?;
        connection.execute(
            "INSERT INTO burst_member_manual_edits (
                project_id, member_group_id, action, manual_group_id, created_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                project_id,
                member_group_id,
                "merge_include",
                manual_burst_group_id,
                now,
            ],
        )?;
    }

    member_groups.sort_by(|left, right| {
        (
            left.first_capture_at_ms
                .or(left.first_received_at_ms)
                .or(Some(left.created_at_ms)),
            left.display_key.as_str(),
            left.group_id.as_str(),
        )
            .cmp(&(
                right
                    .first_capture_at_ms
                    .or(right.first_received_at_ms)
                    .or(Some(right.created_at_ms)),
                right.display_key.as_str(),
                right.group_id.as_str(),
            ))
    });
    let sorted_member_group_ids = member_groups
        .iter()
        .map(|group| group.group_id.clone())
        .collect::<Vec<_>>();
    let started_at_ms = member_groups
        .iter()
        .filter_map(|group| group.first_capture_at_ms.or(group.first_received_at_ms))
        .min();
    let ended_at_ms = member_groups
        .iter()
        .filter_map(|group| group.first_capture_at_ms.or(group.first_received_at_ms))
        .max();
    let source_identity = common_burst_source_identity(connection, project_id, &member_groups)?;
    let burst = BurstGroup {
        burst_group_id: manual_burst_group_id,
        project_id: project_id.to_string(),
        source_identity,
        started_at_ms,
        ended_at_ms,
        member_count: sorted_member_group_ids.len(),
        member_group_ids: sorted_member_group_ids,
        grouping_version: 1,
        recommendation_status: SelectionRecommendationStatus::Pending.as_str().to_string(),
        manual_grouping_state: Some("merge".to_string()),
        created_at_ms: now,
        updated_at_ms: now,
    };
    insert_burst_group(connection, &burst)?;
    Ok(Some(burst))
}

fn burst_group_for_member_group(
    connection: &Connection,
    project_id: &str,
    member_group_id: &str,
) -> std::result::Result<Option<BurstGroup>, rusqlite::Error> {
    let burst_group_id = connection
        .query_row(
            "SELECT bg.burst_group_id
             FROM burst_group_members bgm
             JOIN burst_groups bg ON bg.burst_group_id = bgm.burst_group_id
             WHERE bg.project_id = ?1 AND bgm.member_group_id = ?2
             ORDER BY bg.updated_at_ms DESC, bg.burst_group_id DESC
             LIMIT 1",
            params![project_id, member_group_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    burst_group_id
        .map(|burst_group_id| burst_group_by_id(connection, &burst_group_id))
        .transpose()
        .map(|value| value.flatten())
}

fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn apply_technical_summary(
    connection: &Connection,
    asset_group_id: &str,
    group: &mut ReceivedAssetGroup,
) -> std::result::Result<(), rusqlite::Error> {
    let Some(assessment) = latest_technical_assessment_for_asset_group(connection, asset_group_id)?
    else {
        return Ok(());
    };
    group.technical_status = Some(assessment.status.as_str().to_string());
    group.technical_gate_status = Some(assessment.gate_status.as_str().to_string());
    group.technical_defects = assessment
        .defect_flags
        .into_iter()
        .map(|flag| ReceivedAssetTechnicalDefectSummary {
            defect_type: flag.defect_type.as_str().to_string(),
            severity: flag.severity.as_str().to_string(),
            confidence: flag.confidence,
            reason: (!flag.reason.is_empty()).then_some(flag.reason),
        })
        .collect();
    Ok(())
}

fn latest_technical_assessment_for_asset_group(
    connection: &Connection,
    asset_group_id: &str,
) -> std::result::Result<Option<TechnicalAssessment>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT asset_group_id, assessor_version, status, gate_status, defect_flags_json,
                    preview_source, visual_signature, analyzed_at_ms
             FROM technical_assessments
             WHERE asset_group_id = ?1
             ORDER BY analyzed_at_ms DESC, assessor_version DESC
             LIMIT 1",
            params![asset_group_id],
            technical_assessment_from_row,
        )
        .optional()
}

fn apply_model_evaluation_summary(
    connection: &Connection,
    asset_group_id: &str,
    group: &mut ReceivedAssetGroup,
) -> std::result::Result<(), rusqlite::Error> {
    let Some(evaluation) = latest_any_model_evaluation_for_asset_group(connection, asset_group_id)?
    else {
        return Ok(());
    };
    group.model_status = Some(evaluation.status.as_str().to_string());
    group.model_score = Some(evaluation.score);
    group.model_tier = Some(evaluation.tier.as_str().to_string());
    group.model_evaluator_kind = Some(evaluation.evaluator_kind.as_str().to_string());
    group.model_summary = Some(evaluation.summary);
    Ok(())
}

fn latest_any_model_evaluation_for_asset_group(
    connection: &Connection,
    asset_group_id: &str,
) -> std::result::Result<Option<ModelEvaluation>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT evaluation_id, project_id, asset_group_id, evaluator_kind, evaluator_version,
                    run_id, status, score, tier, selectable, summary, strengths_json, weaknesses_json,
                    technical_warnings_json, prompt_pack_id, prompt_pack_version, prompt_hash,
                    created_at_ms, updated_at_ms
             FROM model_evaluations
             WHERE asset_group_id = ?1
             ORDER BY updated_at_ms DESC, evaluator_version DESC
             LIMIT 1",
            params![asset_group_id],
            model_evaluation_from_row,
        )
        .optional()
}

fn is_model_selected_asset_group(
    connection: &Connection,
    project_id: &str,
    asset_group_id: &str,
    burst_group_id: Option<&str>,
) -> std::result::Result<bool, rusqlite::Error> {
    if selection_recommendation_selects(
        latest_selection_recommendation_for_connection(
            connection,
            project_id,
            SelectionRecommendationScope::Project,
            project_id,
        )?,
        asset_group_id,
    ) {
        return Ok(true);
    }
    if let Some(burst_group_id) = burst_group_id {
        return Ok(selection_recommendation_selects(
            latest_selection_recommendation_for_connection(
                connection,
                project_id,
                SelectionRecommendationScope::BurstGroup,
                burst_group_id,
            )?,
            asset_group_id,
        ));
    }
    Ok(false)
}

fn selection_recommendation_selects(
    recommendation: Option<SelectionRecommendation>,
    asset_group_id: &str,
) -> bool {
    recommendation
        .filter(|value| value.status == SelectionRecommendationStatus::Ready)
        .map(|value| {
            value
                .selected_asset_group_ids
                .iter()
                .any(|selected| selected == asset_group_id)
        })
        .unwrap_or(false)
}

fn trailing_sequence_number(value: &str) -> Option<i64> {
    let reversed_digits = value
        .chars()
        .rev()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if reversed_digits.is_empty() {
        return None;
    }
    reversed_digits
        .chars()
        .rev()
        .collect::<String>()
        .parse()
        .ok()
}

fn received_assets_for_group(
    connection: &Connection,
    project_id: &str,
    group_id: &str,
) -> std::result::Result<Vec<ReceivedAsset>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT asset_id, project_id, group_id, transfer_id, group_role,
                media_kind, format, original_filename, final_filename, normalized_stem, original_path,
                original_parent_path, final_location_payload, size_bytes, capture_at_ms,
                received_at_ms, published_at_ms, source_identity, username, remote_addr,
                source_status, source_modified_at_ms, last_seen_scan_id, duplicate_index,
                duplicate_count
         FROM assets
         WHERE project_id = ?1 AND group_id = ?2
         ORDER BY CASE group_role
                    WHEN 'jpeg' THEN 0
                    WHEN 'raw' THEN 1
                    WHEN 'video' THEN 2
                    ELSE 3
                  END ASC,
                  published_at_ms ASC,
                  asset_id ASC",
    )?;
    let rows = statement.query_map(params![project_id, group_id], received_asset_from_row)?;
    collect_rows(rows)
}

fn user_marks_for_asset_group(
    connection: &Connection,
    project_id: &str,
    group_id: &str,
) -> std::result::Result<AssetUserMarks, rusqlite::Error> {
    connection
        .query_row(
            "SELECT favorite, marked
             FROM asset_group_user_marks
             WHERE project_id = ?1 AND group_id = ?2",
            params![project_id, group_id],
            |row| {
                Ok(AssetUserMarks {
                    favorite: row.get::<_, bool>(0)?,
                    marked: row.get::<_, bool>(1)?,
                })
            },
        )
        .optional()
        .map(|marks| marks.unwrap_or_default())
}

fn initialize_schema(
    connection: &Connection,
    db_path: &Path,
) -> std::result::Result<(), rusqlite::Error> {
    ensure_wal_mode(connection, db_path)?;
    connection.execute_batch(
        "
        PRAGMA synchronous = NORMAL;
        PRAGMA wal_autocheckpoint = 1000;
        ",
    )?;
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS projects (
            project_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL,
            archived_at_ms INTEGER,
            default_output_target_id TEXT
        );

        CREATE TABLE IF NOT EXISTS project_evaluation_settings (
            project_id TEXT PRIMARY KEY REFERENCES projects(project_id),
            auto_evaluate_on_upload INTEGER NOT NULL,
            auto_burst_recommendation_enabled INTEGER NOT NULL,
            project_recommendation_mode TEXT NOT NULL,
            prompt_pack_id TEXT,
            model_provider_settings_id TEXT,
            scene_profile TEXT NOT NULL,
            cv_policy TEXT NOT NULL,
            cv_policy_overrides_json TEXT,
            allow_risky_model_selects INTEGER NOT NULL,
            max_image_side INTEGER,
            batch_size INTEGER,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS evaluation_runs (
            run_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            run_type TEXT NOT NULL,
            trigger TEXT NOT NULL,
            status TEXT NOT NULL,
            provider_kind TEXT NOT NULL,
            provider_model TEXT NOT NULL,
            prompt_pack_id TEXT,
            prompt_pack_version TEXT,
            prompt_hash TEXT,
            settings_snapshot_json TEXT NOT NULL,
            error_message TEXT,
            started_at_ms INTEGER,
            completed_at_ms INTEGER,
            created_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS subject_assessments (
            assessment_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            asset_group_id TEXT NOT NULL,
            subject_type TEXT NOT NULL,
            detector_kind TEXT NOT NULL,
            detector_version TEXT NOT NULL,
            status TEXT NOT NULL,
            gate_status TEXT NOT NULL,
            regions_json TEXT NOT NULL,
            signals_json TEXT NOT NULL,
            summary TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS receiver_accounts (
            username TEXT PRIMARY KEY,
            password_hash TEXT,
            device_name TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS connected_devices (
            remote_addr TEXT PRIMARY KEY,
            source_name TEXT,
            username TEXT,
            first_seen_at_ms INTEGER NOT NULL,
            last_seen_at_ms INTEGER NOT NULL,
            last_disconnected_at_ms INTEGER,
            last_remote_port INTEGER,
            active_connections INTEGER NOT NULL,
            online INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS receiver_status (
            key TEXT PRIMARY KEY,
            phase TEXT NOT NULL,
            protocol TEXT,
            auth_mode TEXT NOT NULL,
            local_addr TEXT,
            output_dir TEXT,
            state_dir TEXT,
            account_count INTEGER NOT NULL,
            message TEXT,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS transfers (
            transfer_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            protocol TEXT NOT NULL,
            status TEXT NOT NULL,
            original_path TEXT NOT NULL,
            final_filename TEXT NOT NULL,
            final_location_kind TEXT,
            final_location_payload TEXT,
            size_bytes INTEGER NOT NULL,
            username TEXT,
            remote_addr TEXT,
            source_name TEXT,
            started_at_ms INTEGER NOT NULL,
            completed_at_ms INTEGER,
            error TEXT
        );

        CREATE TABLE IF NOT EXISTS desktop_scan_runs (
            scan_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            root_path TEXT NOT NULL,
            root_key TEXT NOT NULL,
            root_label TEXT NOT NULL,
            phase TEXT NOT NULL,
            files_seen INTEGER NOT NULL,
            assets_indexed INTEGER NOT NULL,
            groups_updated INTEGER NOT NULL,
            started_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL,
            completed_at_ms INTEGER,
            error TEXT
        );

        CREATE TABLE IF NOT EXISTS asset_groups (
            group_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            group_identity TEXT NOT NULL UNIQUE,
            display_key TEXT NOT NULL,
            source_identity TEXT,
            original_parent_path TEXT,
            primary_asset_id TEXT,
            preview_asset_id TEXT,
            member_count INTEGER NOT NULL DEFAULT 0,
            has_raw INTEGER NOT NULL DEFAULT 0,
            has_jpeg INTEGER NOT NULL DEFAULT 0,
            has_video INTEGER NOT NULL DEFAULT 0,
            first_capture_at_ms INTEGER,
            last_capture_at_ms INTEGER,
            first_received_at_ms INTEGER,
            last_received_at_ms INTEGER,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS assets (
            asset_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            group_id TEXT NOT NULL REFERENCES asset_groups(group_id),
            transfer_id TEXT NOT NULL UNIQUE REFERENCES transfers(transfer_id),
            group_role TEXT NOT NULL,
            media_kind TEXT NOT NULL,
            format TEXT NOT NULL,
            original_filename TEXT NOT NULL,
            final_filename TEXT NOT NULL,
            normalized_stem TEXT NOT NULL,
            original_path TEXT NOT NULL,
            original_parent_path TEXT,
            final_location_kind TEXT,
            final_location_payload TEXT,
            size_bytes INTEGER NOT NULL,
            capture_at_ms INTEGER,
            received_at_ms INTEGER,
            published_at_ms INTEGER,
            source_identity TEXT,
            username TEXT,
            remote_addr TEXT,
            source_status TEXT NOT NULL DEFAULT 'available',
            source_modified_at_ms INTEGER,
            last_seen_scan_id TEXT,
            duplicate_key TEXT,
            duplicate_index INTEGER,
            duplicate_count INTEGER
        );

        CREATE TABLE IF NOT EXISTS publish_queue (
            queue_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            transfer_id TEXT NOT NULL,
            staged_path TEXT NOT NULL,
            final_filename TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            protocol TEXT,
            original_path TEXT,
            username TEXT,
            remote_addr TEXT,
            source_name TEXT,
            started_at_ms INTEGER,
            state TEXT NOT NULL,
            attempt_count INTEGER NOT NULL,
            last_error TEXT,
            next_attempt_at_ms INTEGER,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS background_jobs (
            job_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            job_type TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            dedupe_key TEXT NOT NULL,
            status TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            attempts INTEGER NOT NULL DEFAULT 0,
            next_attempt_at_ms INTEGER,
            last_error TEXT,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS burst_groups (
            burst_group_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            source_identity TEXT,
            started_at_ms INTEGER,
            ended_at_ms INTEGER,
            member_count INTEGER NOT NULL,
            grouping_version INTEGER NOT NULL,
            recommendation_status TEXT NOT NULL,
            manual_grouping_state TEXT,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS burst_group_members (
            burst_group_id TEXT NOT NULL REFERENCES burst_groups(burst_group_id) ON DELETE CASCADE,
            member_group_id TEXT NOT NULL REFERENCES asset_groups(group_id),
            PRIMARY KEY(burst_group_id, member_group_id)
        );

        CREATE TABLE IF NOT EXISTS burst_member_manual_edits (
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            member_group_id TEXT NOT NULL REFERENCES asset_groups(group_id),
            action TEXT NOT NULL,
            manual_group_id TEXT,
            created_at_ms INTEGER NOT NULL,
            PRIMARY KEY(project_id, member_group_id, action)
        );

        CREATE TABLE IF NOT EXISTS asset_group_user_marks (
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            group_id TEXT NOT NULL REFERENCES asset_groups(group_id) ON DELETE CASCADE,
            favorite INTEGER NOT NULL DEFAULT 0,
            marked INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL,
            PRIMARY KEY(project_id, group_id)
        );

        CREATE TABLE IF NOT EXISTS technical_assessments (
            asset_group_id TEXT NOT NULL,
            assessor_version TEXT NOT NULL,
            status TEXT NOT NULL,
            gate_status TEXT NOT NULL,
            defect_flags_json TEXT NOT NULL,
            preview_source TEXT,
            visual_signature TEXT,
            analyzed_at_ms INTEGER NOT NULL,
            PRIMARY KEY(asset_group_id, assessor_version)
        );

        CREATE TABLE IF NOT EXISTS model_evaluations (
            evaluation_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            asset_group_id TEXT NOT NULL,
            evaluator_kind TEXT NOT NULL,
            evaluator_version TEXT NOT NULL,
            status TEXT NOT NULL,
            score INTEGER NOT NULL,
            tier TEXT NOT NULL,
            selectable INTEGER NOT NULL,
            summary TEXT NOT NULL,
            strengths_json TEXT NOT NULL,
            weaknesses_json TEXT NOT NULL,
            technical_warnings_json TEXT NOT NULL,
            prompt_pack_id TEXT,
            prompt_pack_version TEXT,
            prompt_hash TEXT,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS selection_recommendations (
            recommendation_id TEXT PRIMARY KEY,
            run_id TEXT,
            scope TEXT NOT NULL,
            project_id TEXT NOT NULL,
            subject_id TEXT NOT NULL,
            selected_asset_group_ids_json TEXT NOT NULL,
            candidate_asset_group_ids_json TEXT NOT NULL,
            rejected_asset_group_ids_json TEXT NOT NULL,
            source TEXT NOT NULL,
            status TEXT NOT NULL,
            confidence REAL NOT NULL,
            reason TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_assets_project_group ON assets(project_id, group_id);
        CREATE INDEX IF NOT EXISTS idx_asset_groups_project ON asset_groups(project_id, updated_at_ms);
        CREATE INDEX IF NOT EXISTS idx_desktop_scan_runs_project ON desktop_scan_runs(project_id, started_at_ms);
        CREATE INDEX IF NOT EXISTS idx_connected_devices_username ON connected_devices(username);
        CREATE INDEX IF NOT EXISTS idx_connected_devices_sort ON connected_devices(online, last_seen_at_ms);
        CREATE INDEX IF NOT EXISTS idx_receiver_accounts_enabled ON receiver_accounts(enabled, updated_at_ms);
        CREATE INDEX IF NOT EXISTS idx_publish_queue_state ON publish_queue(state, created_at_ms);
        CREATE INDEX IF NOT EXISTS idx_background_jobs_claim ON background_jobs(status, priority, next_attempt_at_ms, created_at_ms);
        CREATE INDEX IF NOT EXISTS idx_background_jobs_dedupe ON background_jobs(dedupe_key);
        CREATE INDEX IF NOT EXISTS idx_evaluation_runs_project ON evaluation_runs(project_id, run_type, status, created_at_ms);
        CREATE INDEX IF NOT EXISTS idx_subject_assessments_group ON subject_assessments(project_id, asset_group_id, subject_type);
        CREATE INDEX IF NOT EXISTS idx_burst_groups_project ON burst_groups(project_id, updated_at_ms);
        CREATE INDEX IF NOT EXISTS idx_burst_members_group ON burst_group_members(member_group_id, burst_group_id);
        CREATE INDEX IF NOT EXISTS idx_burst_member_manual_edits_project ON burst_member_manual_edits(project_id, action, member_group_id);
        CREATE INDEX IF NOT EXISTS idx_asset_group_user_marks_project ON asset_group_user_marks(project_id, favorite, marked);
        CREATE INDEX IF NOT EXISTS idx_technical_assessments_status ON technical_assessments(status, gate_status);
        CREATE INDEX IF NOT EXISTS idx_model_evaluations_project ON model_evaluations(project_id, status, tier);
        CREATE INDEX IF NOT EXISTS idx_model_evaluations_asset_group ON model_evaluations(asset_group_id, evaluator_version);
        CREATE INDEX IF NOT EXISTS idx_recommendations_scope ON selection_recommendations(project_id, scope, subject_id, status);
        ",
    )?;
    ensure_desktop_scan_asset_columns(connection)?;
    connection.execute(
        "CREATE INDEX IF NOT EXISTS idx_assets_desktop_scan
         ON assets(project_id, transfer_id, last_seen_scan_id)",
        [],
    )?;
    Ok(())
}

static SQLITE_WAL_CONFIGURED_PATHS: OnceLock<Mutex<BTreeSet<PathBuf>>> = OnceLock::new();

fn ensure_wal_mode(
    connection: &Connection,
    db_path: &Path,
) -> std::result::Result<(), rusqlite::Error> {
    let key = sqlite_lock_key(db_path);
    {
        let configured = SQLITE_WAL_CONFIGURED_PATHS
            .get_or_init(|| Mutex::new(BTreeSet::new()))
            .lock()
            .expect("sqlite WAL registry should not be poisoned");
        if configured.contains(&key) {
            return Ok(());
        }
    }

    connection.query_row("PRAGMA journal_mode = WAL", [], |row| {
        row.get::<_, String>(0)
    })?;

    SQLITE_WAL_CONFIGURED_PATHS
        .get_or_init(|| Mutex::new(BTreeSet::new()))
        .lock()
        .expect("sqlite WAL registry should not be poisoned")
        .insert(key);
    Ok(())
}

fn ensure_desktop_scan_asset_columns(
    connection: &Connection,
) -> std::result::Result<(), rusqlite::Error> {
    add_column_if_missing(
        connection,
        "assets",
        "source_status",
        "TEXT NOT NULL DEFAULT 'available'",
    )?;
    add_column_if_missing(connection, "assets", "source_modified_at_ms", "INTEGER")?;
    add_column_if_missing(connection, "assets", "last_seen_scan_id", "TEXT")?;
    Ok(())
}

fn add_column_if_missing(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> std::result::Result<(), rusqlite::Error> {
    let columns = table_columns(connection, table_name)?;
    if columns.is_empty() || columns.contains(column_name) {
        return Ok(());
    }
    connection.execute(
        &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_definition}"),
        [],
    )?;
    Ok(())
}

fn table_columns(
    connection: &Connection,
    table_name: &str,
) -> std::result::Result<BTreeSet<String>, rusqlite::Error> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    let mut columns = BTreeSet::new();
    for row in rows {
        columns.insert(row?);
    }
    Ok(columns)
}

fn project_evaluation_settings_for_project(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<Option<ProjectEvaluationSettings>, rusqlite::Error> {
    if project_by_id(connection, project_id)?.is_none() {
        return Ok(None);
    }
    if let Some(settings) = project_evaluation_settings_by_project_id(connection, project_id)? {
        return Ok(Some(settings));
    }
    let settings =
        ProjectEvaluationSettings::default_for_project(project_id.to_string(), current_time_ms());
    save_project_evaluation_settings_for_connection(connection, settings).map(Some)
}

fn project_evaluation_settings_by_project_id(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<Option<ProjectEvaluationSettings>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT project_id, auto_evaluate_on_upload,
                    auto_burst_recommendation_enabled, project_recommendation_mode,
                    prompt_pack_id, model_provider_settings_id, scene_profile, cv_policy,
                    cv_policy_overrides_json, allow_risky_model_selects, max_image_side,
                    batch_size, updated_at_ms
             FROM project_evaluation_settings
             WHERE project_id = ?1",
            params![project_id],
            project_evaluation_settings_from_row,
        )
        .optional()
}

fn save_project_evaluation_settings_for_connection(
    connection: &Connection,
    settings: ProjectEvaluationSettings,
) -> std::result::Result<ProjectEvaluationSettings, rusqlite::Error> {
    ensure_project_exists(connection, &settings.project_id)?;
    validate_project_evaluation_settings(connection, &settings)?;
    connection.execute(
        "INSERT INTO project_evaluation_settings (
            project_id, auto_evaluate_on_upload,
            auto_burst_recommendation_enabled, project_recommendation_mode, prompt_pack_id,
            model_provider_settings_id, scene_profile, cv_policy, cv_policy_overrides_json,
            allow_risky_model_selects, max_image_side, batch_size, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(project_id) DO UPDATE SET
            auto_evaluate_on_upload = excluded.auto_evaluate_on_upload,
            auto_burst_recommendation_enabled = excluded.auto_burst_recommendation_enabled,
            project_recommendation_mode = excluded.project_recommendation_mode,
            prompt_pack_id = excluded.prompt_pack_id,
            model_provider_settings_id = excluded.model_provider_settings_id,
            scene_profile = excluded.scene_profile,
            cv_policy = excluded.cv_policy,
            cv_policy_overrides_json = excluded.cv_policy_overrides_json,
            allow_risky_model_selects = excluded.allow_risky_model_selects,
            max_image_side = excluded.max_image_side,
            batch_size = excluded.batch_size,
            updated_at_ms = excluded.updated_at_ms",
        params![
            &settings.project_id,
            settings.auto_evaluate_on_upload,
            settings.auto_burst_recommendation_enabled,
            settings.project_recommendation_mode.as_str(),
            settings.prompt_pack_id.as_deref(),
            settings.model_provider_settings_id.as_deref(),
            settings.scene_profile.as_str(),
            settings.cv_policy.as_str(),
            technical_assessment_policy_json(settings.cv_policy_overrides.as_ref())?,
            settings.allow_risky_model_selects,
            settings.max_image_side,
            settings.batch_size,
            settings.updated_at_ms,
        ],
    )?;
    project_evaluation_settings_by_project_id(connection, &settings.project_id)?.ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("project evaluation settings not found".to_string())
    })
}

fn validate_project_evaluation_settings(
    _connection: &Connection,
    settings: &ProjectEvaluationSettings,
) -> std::result::Result<(), rusqlite::Error> {
    if settings
        .prompt_pack_id
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(sqlite_data_error("prompt pack id cannot be blank"));
    }
    Ok(())
}

fn project_evaluation_settings_from_row(
    row: &Row<'_>,
) -> std::result::Result<ProjectEvaluationSettings, rusqlite::Error> {
    let project_recommendation_mode: String = row.get(3)?;
    let scene_profile: String = row.get(6)?;
    let cv_policy: String = row.get(7)?;
    let cv_policy_overrides_json: Option<String> = row.get(8)?;
    Ok(ProjectEvaluationSettings {
        project_id: row.get(0)?,
        auto_evaluate_on_upload: row.get(1)?,
        auto_burst_recommendation_enabled: row.get(2)?,
        project_recommendation_mode: ProjectRecommendationMode::from_str(
            &project_recommendation_mode,
        ),
        prompt_pack_id: row.get(4)?,
        model_provider_settings_id: row.get(5)?,
        scene_profile: SceneProfile::from_str(&scene_profile),
        cv_policy: CvPolicy::from_str(&cv_policy),
        cv_policy_overrides: technical_assessment_policy_from_json(cv_policy_overrides_json)?,
        allow_risky_model_selects: row.get(9)?,
        max_image_side: row.get(10)?,
        batch_size: row.get(11)?,
        updated_at_ms: row.get(12)?,
    })
}

fn save_evaluation_run_for_connection(
    connection: &Connection,
    run: EvaluationRun,
) -> std::result::Result<EvaluationRun, rusqlite::Error> {
    ensure_project_exists(connection, &run.project_id)?;
    connection.execute(
        "INSERT INTO evaluation_runs (
            run_id, project_id, run_type, trigger, status, provider_kind, provider_model,
            prompt_pack_id, prompt_pack_version, prompt_hash, settings_snapshot_json,
            error_message, started_at_ms, completed_at_ms, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
         ON CONFLICT(run_id) DO UPDATE SET
            project_id = excluded.project_id,
            run_type = excluded.run_type,
            trigger = excluded.trigger,
            status = excluded.status,
            provider_kind = excluded.provider_kind,
            provider_model = excluded.provider_model,
            prompt_pack_id = excluded.prompt_pack_id,
            prompt_pack_version = excluded.prompt_pack_version,
            prompt_hash = excluded.prompt_hash,
            settings_snapshot_json = excluded.settings_snapshot_json,
            error_message = excluded.error_message,
            started_at_ms = excluded.started_at_ms,
            completed_at_ms = excluded.completed_at_ms,
            created_at_ms = excluded.created_at_ms",
        params![
            run.run_id,
            run.project_id,
            run.run_type.as_str(),
            run.trigger.as_str(),
            run.status.as_str(),
            run.provider_kind.as_str(),
            run.provider_model,
            run.prompt_pack_id,
            run.prompt_pack_version,
            run.prompt_hash,
            run.settings_snapshot_json,
            run.error_message,
            run.started_at_ms,
            run.completed_at_ms,
            run.created_at_ms,
        ],
    )?;
    evaluation_run_by_id(connection, &run.run_id)?.ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("evaluation run not found".to_string())
    })
}

fn evaluation_run_by_id(
    connection: &Connection,
    run_id: &str,
) -> std::result::Result<Option<EvaluationRun>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT run_id, project_id, run_type, trigger, status, provider_kind, provider_model,
                    prompt_pack_id, prompt_pack_version, prompt_hash, settings_snapshot_json,
                    error_message, started_at_ms, completed_at_ms, created_at_ms
             FROM evaluation_runs
             WHERE run_id = ?1",
            params![run_id],
            evaluation_run_from_row,
        )
        .optional()
}

fn latest_evaluation_run(
    connection: &Connection,
    project_id: &str,
    run_type: EvaluationRunType,
) -> std::result::Result<Option<EvaluationRun>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT run_id, project_id, run_type, trigger, status, provider_kind, provider_model,
                    prompt_pack_id, prompt_pack_version, prompt_hash, settings_snapshot_json,
                    error_message, started_at_ms, completed_at_ms, created_at_ms
             FROM evaluation_runs
             WHERE project_id = ?1 AND run_type = ?2
             ORDER BY created_at_ms DESC, run_id DESC
             LIMIT 1",
            params![project_id, run_type.as_str()],
            evaluation_run_from_row,
        )
        .optional()
}

fn evaluation_run_from_row(row: &Row<'_>) -> std::result::Result<EvaluationRun, rusqlite::Error> {
    let run_type: String = row.get(2)?;
    let trigger: String = row.get(3)?;
    let status: String = row.get(4)?;
    let provider_kind: String = row.get(5)?;
    Ok(EvaluationRun {
        run_id: row.get(0)?,
        project_id: row.get(1)?,
        run_type: EvaluationRunType::from_str(&run_type),
        trigger: EvaluationRunTrigger::from_str(&trigger),
        status: EvaluationRunStatus::from_str(&status),
        provider_kind: ModelProviderKind::from_str(&provider_kind),
        provider_model: row.get(6)?,
        prompt_pack_id: row.get(7)?,
        prompt_pack_version: row.get(8)?,
        prompt_hash: row.get(9)?,
        settings_snapshot_json: row.get(10)?,
        error_message: row.get(11)?,
        started_at_ms: row.get(12)?,
        completed_at_ms: row.get(13)?,
        created_at_ms: row.get(14)?,
    })
}

fn save_subject_assessment_for_connection(
    connection: &Connection,
    assessment: SubjectAssessment,
) -> std::result::Result<SubjectAssessment, rusqlite::Error> {
    ensure_project_exists(connection, &assessment.project_id)?;
    ensure_asset_group_exists_in_project(
        connection,
        &assessment.project_id,
        &assessment.asset_group_id,
    )?;
    ensure_subject_assessment_identity_is_stable(connection, &assessment)?;
    validate_subject_assessment_json(&assessment)?;
    connection.execute(
        "INSERT INTO subject_assessments (
            assessment_id, project_id, asset_group_id, subject_type, detector_kind,
            detector_version, status, gate_status, regions_json, signals_json, summary,
            created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(assessment_id) DO UPDATE SET
            project_id = excluded.project_id,
            asset_group_id = excluded.asset_group_id,
            subject_type = excluded.subject_type,
            detector_kind = excluded.detector_kind,
            detector_version = excluded.detector_version,
            status = excluded.status,
            gate_status = excluded.gate_status,
            regions_json = excluded.regions_json,
            signals_json = excluded.signals_json,
            summary = excluded.summary,
            updated_at_ms = excluded.updated_at_ms",
        params![
            assessment.assessment_id,
            assessment.project_id,
            assessment.asset_group_id,
            assessment.subject_type,
            assessment.detector_kind,
            assessment.detector_version,
            assessment.status.as_str(),
            assessment.gate_status,
            assessment.regions_json,
            assessment.signals_json,
            assessment.summary,
            assessment.created_at_ms,
            assessment.updated_at_ms,
        ],
    )?;
    subject_assessment_by_id(connection, &assessment.assessment_id)?.ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("subject assessment not found".to_string())
    })
}

fn ensure_asset_group_exists_in_project(
    connection: &Connection,
    project_id: &str,
    asset_group_id: &str,
) -> std::result::Result<(), rusqlite::Error> {
    let exists = connection.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM asset_groups WHERE project_id = ?1 AND group_id = ?2
         )",
        params![project_id, asset_group_id],
        |row| row.get::<_, bool>(0),
    )?;
    if exists {
        Ok(())
    } else {
        Err(sqlite_data_error(
            "subject assessment asset group not found in project",
        ))
    }
}

fn ensure_subject_assessment_identity_is_stable(
    connection: &Connection,
    assessment: &SubjectAssessment,
) -> std::result::Result<(), rusqlite::Error> {
    let existing = connection
        .query_row(
            "SELECT project_id, asset_group_id
             FROM subject_assessments
             WHERE assessment_id = ?1",
            params![assessment.assessment_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    if let Some((project_id, asset_group_id)) = existing {
        if project_id != assessment.project_id || asset_group_id != assessment.asset_group_id {
            return Err(sqlite_data_error(
                "subject assessment id cannot move between asset groups",
            ));
        }
    }
    Ok(())
}

fn validate_subject_assessment_json(
    assessment: &SubjectAssessment,
) -> std::result::Result<(), rusqlite::Error> {
    let regions =
        serde_json::from_str::<serde_json::Value>(&assessment.regions_json).map_err(|error| {
            sqlite_data_error(format!("invalid subject assessment regions_json: {error}"))
        })?;
    if !regions.is_array() {
        return Err(sqlite_data_error(
            "subject assessment regions_json must be a JSON array",
        ));
    }
    let signals =
        serde_json::from_str::<serde_json::Value>(&assessment.signals_json).map_err(|error| {
            sqlite_data_error(format!("invalid subject assessment signals_json: {error}"))
        })?;
    if !signals.is_object() {
        return Err(sqlite_data_error(
            "subject assessment signals_json must be a JSON object",
        ));
    }
    Ok(())
}

fn subject_assessment_by_id(
    connection: &Connection,
    assessment_id: &str,
) -> std::result::Result<Option<SubjectAssessment>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT assessment_id, project_id, asset_group_id, subject_type, detector_kind,
                    detector_version, status, gate_status, regions_json, signals_json, summary,
                    created_at_ms, updated_at_ms
             FROM subject_assessments
             WHERE assessment_id = ?1",
            params![assessment_id],
            subject_assessment_from_row,
        )
        .optional()
}

fn subject_assessments_for_asset_groups(
    connection: &Connection,
    project_id: &str,
    group_ids: &[String],
) -> std::result::Result<Vec<SubjectAssessment>, rusqlite::Error> {
    let mut assessments = Vec::new();
    for group_id in group_ids {
        let mut statement = connection.prepare(
            "SELECT assessment_id, project_id, asset_group_id, subject_type, detector_kind,
                    detector_version, status, gate_status, regions_json, signals_json, summary,
                    created_at_ms, updated_at_ms
             FROM subject_assessments
             WHERE project_id = ?1 AND asset_group_id = ?2
             ORDER BY created_at_ms DESC, assessment_id DESC",
        )?;
        let rows =
            statement.query_map(params![project_id, group_id], subject_assessment_from_row)?;
        assessments.extend(collect_rows(rows)?);
    }
    Ok(assessments)
}

fn subject_assessment_from_row(
    row: &Row<'_>,
) -> std::result::Result<SubjectAssessment, rusqlite::Error> {
    let status: String = row.get(6)?;
    Ok(SubjectAssessment {
        assessment_id: row.get(0)?,
        project_id: row.get(1)?,
        asset_group_id: row.get(2)?,
        subject_type: row.get(3)?,
        detector_kind: row.get(4)?,
        detector_version: row.get(5)?,
        status: EvaluationRunStatus::from_str(&status),
        gate_status: row.get(7)?,
        regions_json: row.get(8)?,
        signals_json: row.get(9)?,
        summary: row.get(10)?,
        created_at_ms: row.get(11)?,
        updated_at_ms: row.get(12)?,
    })
}

fn receiver_account_by_username(
    connection: &Connection,
    username: &str,
) -> std::result::Result<Option<StoredReceiverAccount>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT username, password_hash, device_name, enabled, created_at_ms, updated_at_ms
             FROM receiver_accounts
             WHERE username = ?1",
            params![username],
            receiver_account_from_row,
        )
        .optional()
}

fn receiver_account_from_row(
    row: &Row<'_>,
) -> std::result::Result<StoredReceiverAccount, rusqlite::Error> {
    Ok(StoredReceiverAccount {
        username: row.get(0)?,
        password_hash: row.get(1)?,
        device_name: row.get(2)?,
        enabled: row.get::<_, i64>(3)? != 0,
        created_at_ms: row.get(4)?,
        updated_at_ms: row.get(5)?,
    })
}

fn connected_devices_from_connection(
    connection: &Connection,
) -> std::result::Result<Vec<ConnectedDevice>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT remote_addr, source_name, username, first_seen_at_ms, last_seen_at_ms,
                last_disconnected_at_ms, last_remote_port, active_connections, online
         FROM connected_devices",
    )?;
    let rows = statement.query_map([], connected_device_from_row)?;
    collect_rows(rows)
}

fn connected_device_from_row(
    row: &Row<'_>,
) -> std::result::Result<ConnectedDevice, rusqlite::Error> {
    Ok(ConnectedDevice {
        remote_addr: row.get(0)?,
        source_name: row.get(1)?,
        username: row.get(2)?,
        first_seen_at_ms: row.get(3)?,
        last_seen_at_ms: row.get(4)?,
        last_disconnected_at_ms: row.get(5)?,
        last_remote_port: row.get::<_, Option<i64>>(6)?.map(|port| port as u16),
        active_connections: row.get::<_, i64>(7)? as u32,
        online: row.get::<_, i64>(8)? != 0,
    })
}

fn replace_connected_devices(
    connection: &Connection,
    devices: &[ConnectedDevice],
) -> std::result::Result<(), rusqlite::Error> {
    connection.execute("DELETE FROM connected_devices", [])?;
    for device in devices {
        connection.execute(
            "INSERT INTO connected_devices (
                remote_addr, source_name, username, first_seen_at_ms, last_seen_at_ms,
                last_disconnected_at_ms, last_remote_port, active_connections, online
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &device.remote_addr,
                device.source_name.as_deref(),
                device.username.as_deref(),
                device.first_seen_at_ms,
                device.last_seen_at_ms,
                device.last_disconnected_at_ms,
                device.last_remote_port.map(|port| port as i64),
                device.active_connections as i64,
                if device.online { 1_i64 } else { 0_i64 },
            ],
        )?;
    }
    Ok(())
}

fn sort_connected_devices(devices: &mut [ConnectedDevice]) {
    devices.sort_by(|left, right| {
        right
            .online
            .cmp(&left.online)
            .then_with(|| right.last_seen_at_ms.cmp(&left.last_seen_at_ms))
            .then_with(|| left.remote_addr.cmp(&right.remote_addr))
    });
}

fn receiver_runtime_status_from_row(
    row: &Row<'_>,
) -> std::result::Result<ReceiverRuntimeStatus, rusqlite::Error> {
    let phase = row.get::<_, String>(0)?;
    let protocol = row.get::<_, Option<String>>(1)?;
    let auth_mode = row.get::<_, String>(2)?;
    let local_addr = row.get::<_, Option<String>>(3)?;
    let output_dir = row.get::<_, Option<String>>(4)?;
    let state_dir = row.get::<_, Option<String>>(5)?;
    Ok(ReceiverRuntimeStatus {
        phase: receiver_runtime_phase_from_name(&phase)?,
        protocol: protocol
            .as_deref()
            .map(push_protocol_from_name)
            .transpose()?,
        auth_mode: receiver_auth_mode_from_name(&auth_mode)?,
        local_addr: local_addr
            .as_deref()
            .map(|value| value.parse().map_err(sqlite_data_error))
            .transpose()?,
        output_dir: output_dir.map(PathBuf::from),
        state_dir: state_dir.map(PathBuf::from),
        account_count: row.get::<_, i64>(6)? as usize,
        message: row.get(7)?,
    })
}

fn receiver_runtime_phase_name(phase: ReceiverRuntimePhase) -> &'static str {
    match phase {
        ReceiverRuntimePhase::Stopped => "stopped",
        ReceiverRuntimePhase::Starting => "starting",
        ReceiverRuntimePhase::Running => "running",
        ReceiverRuntimePhase::Stopping => "stopping",
        ReceiverRuntimePhase::Failed => "failed",
    }
}

fn receiver_runtime_phase_from_name(
    phase: &str,
) -> std::result::Result<ReceiverRuntimePhase, rusqlite::Error> {
    match phase {
        "stopped" => Ok(ReceiverRuntimePhase::Stopped),
        "starting" => Ok(ReceiverRuntimePhase::Starting),
        "running" => Ok(ReceiverRuntimePhase::Running),
        "stopping" => Ok(ReceiverRuntimePhase::Stopping),
        "failed" => Ok(ReceiverRuntimePhase::Failed),
        value => Err(sqlite_data_error(format!(
            "invalid receiver runtime phase: {value}"
        ))),
    }
}

fn receiver_auth_mode_name(auth_mode: ReceiverAuthMode) -> &'static str {
    match auth_mode {
        ReceiverAuthMode::Anonymous => "anonymous",
        ReceiverAuthMode::Accounts => "accounts",
    }
}

fn receiver_auth_mode_from_name(
    auth_mode: &str,
) -> std::result::Result<ReceiverAuthMode, rusqlite::Error> {
    match auth_mode {
        "anonymous" => Ok(ReceiverAuthMode::Anonymous),
        "accounts" => Ok(ReceiverAuthMode::Accounts),
        value => Err(sqlite_data_error(format!(
            "invalid receiver auth mode: {value}"
        ))),
    }
}

fn push_protocol_name(protocol: PushProtocol) -> &'static str {
    match protocol {
        PushProtocol::Ftp => "ftp",
        PushProtocol::Sftp => "sftp",
    }
}

fn push_protocol_from_name(protocol: &str) -> std::result::Result<PushProtocol, rusqlite::Error> {
    match protocol {
        "ftp" => Ok(PushProtocol::Ftp),
        "sftp" => Ok(PushProtocol::Sftp),
        value => Err(sqlite_data_error(format!("invalid push protocol: {value}"))),
    }
}

fn sqlite_data_error(error: impl ToString) -> rusqlite::Error {
    rusqlite::Error::InvalidParameterName(error.to_string())
}

fn ensure_project_exists(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<(), rusqlite::Error> {
    if project_by_id(connection, project_id)?.is_none() {
        return Err(rusqlite::Error::InvalidParameterName(
            "project not found".to_string(),
        ));
    }
    Ok(())
}

fn ensure_project_is_active(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<Project, rusqlite::Error> {
    let project = project_by_id(connection, project_id)?
        .ok_or_else(|| rusqlite::Error::InvalidParameterName("project not found".to_string()))?;
    if project.status == ProjectStatus::Archived {
        return Err(rusqlite::Error::InvalidParameterName(
            "project archived".to_string(),
        ));
    }
    Ok(project)
}

fn project_by_id(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<Option<Project>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT project_id, name, slug, status, created_at_ms, updated_at_ms,
                    archived_at_ms, default_output_target_id
             FROM projects
             WHERE project_id = ?1",
            params![project_id],
            project_from_row,
        )
        .optional()
}

fn desktop_scan_run_by_id(
    connection: &Connection,
    scan_id: &str,
) -> std::result::Result<Option<DesktopScanRun>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT scan_id, project_id, root_path, root_key, root_label, phase,
                    files_seen, assets_indexed, groups_updated, started_at_ms, updated_at_ms,
                    completed_at_ms, error
             FROM desktop_scan_runs
             WHERE scan_id = ?1",
            params![scan_id],
            desktop_scan_run_from_row,
        )
        .optional()
}

fn desktop_scan_asset_source_state(
    connection: &Connection,
    transfer_id: &str,
) -> std::result::Result<Option<(Option<i64>, u64)>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT source_modified_at_ms, size_bytes
             FROM assets
             WHERE transfer_id = ?1",
            params![transfer_id],
            |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, i64>(1)? as u64)),
        )
        .optional()
}

fn desktop_scan_missing_group_ids(
    connection: &Connection,
    project_id: &str,
    root_transfer_prefix: &str,
    scan_id: &str,
) -> std::result::Result<Vec<String>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT DISTINCT group_id
         FROM assets
         WHERE project_id = ?1
           AND transfer_id LIKE ?2
           AND (last_seen_scan_id IS NULL OR last_seen_scan_id <> ?3)",
    )?;
    let rows = statement.query_map(params![project_id, root_transfer_prefix, scan_id], |row| {
        row.get::<_, String>(0)
    })?;
    collect_rows(rows)
}

fn insert_transfer(
    connection: &Connection,
    project_id: &str,
    record: &TransferRecord,
) -> std::result::Result<(), rusqlite::Error> {
    let final_location = record.resolved_final_location();
    let final_location_payload = final_location_json(final_location.as_ref())?;
    connection.execute(
        "INSERT OR REPLACE INTO transfers (
            transfer_id, project_id, protocol, status, original_path, final_filename,
            final_location_kind, final_location_payload, size_bytes, username, remote_addr,
            source_name, started_at_ms, completed_at_ms, error
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            record.transfer_id,
            project_id,
            record.protocol,
            transfer_status_name(record.status),
            record.original_path,
            record.final_filename,
            final_location.as_ref().map(StoredObjectLocation::kind),
            final_location_payload,
            record.size_bytes as i64,
            record.username,
            record.remote_addr,
            record.source_name,
            record.started_at_ms,
            record.completed_at_ms,
            record.error,
        ],
    )?;
    Ok(())
}

fn insert_asset_for_transfer(
    connection: &Connection,
    project_id: &str,
    record: &TransferRecord,
) -> std::result::Result<Option<String>, rusqlite::Error> {
    let format = ObjectFormat::from_filename(&record.final_filename);
    if !format.is_supported_media() {
        return Ok(None);
    }

    let now = current_time_ms();
    let normalized_stem =
        normalized_stem(&record.final_filename).unwrap_or_else(|| record.final_filename.clone());
    let asset_original_parent_path = original_parent_path(&record.original_path);
    let group_original_parent_path = if record.protocol == DESKTOP_SCAN_PROTOCOL {
        None
    } else {
        asset_original_parent_path.clone()
    };
    let source_identity = source_identity(record);
    let group_identity = asset_group_identity(
        project_id,
        source_identity.as_deref(),
        group_original_parent_path.as_deref(),
        &normalized_stem,
    );
    let group_id = format!("group-{}", stable_key(&group_identity));
    let final_location = record.resolved_final_location();
    let final_location_payload = final_location_json(final_location.as_ref())?;
    let group_role = group_role(format).to_string();
    let published_at_ms = record.completed_at_ms.or(Some(record.started_at_ms));
    let capture_at_ms = final_location
        .as_ref()
        .and_then(|location| location.as_local_path())
        .and_then(extract_capture_time_ms);
    let duplicate_key = duplicate_key(record);
    let previous_duplicate_key = connection
        .query_row(
            "SELECT duplicate_key FROM assets WHERE transfer_id = ?1",
            params![record.transfer_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten();

    connection.execute(
        "INSERT INTO asset_groups (
            group_id, project_id, group_identity, display_key, source_identity, original_parent_path,
            created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(group_identity) DO UPDATE SET updated_at_ms = excluded.updated_at_ms",
        params![
            group_id,
            project_id,
            group_identity,
            normalized_stem,
            source_identity,
            group_original_parent_path,
            now,
            now,
        ],
    )?;

    connection.execute(
        "INSERT OR REPLACE INTO assets (
            asset_id, project_id, group_id, transfer_id, group_role, media_kind, format,
            original_filename, final_filename, normalized_stem, original_path, original_parent_path,
            final_location_kind, final_location_payload, size_bytes, capture_at_ms, received_at_ms,
            published_at_ms, source_identity, username, remote_addr, source_status,
            source_modified_at_ms, last_seen_scan_id, duplicate_key
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)",
        params![
            record.transfer_id,
            project_id,
            group_id,
            record.transfer_id,
            group_role,
            media_kind(format),
            format_name(format),
            original_filename(&record.original_path),
            record.final_filename,
            normalized_stem,
            record.original_path,
            asset_original_parent_path,
            final_location.as_ref().map(StoredObjectLocation::kind),
            final_location_payload,
            record.size_bytes as i64,
            capture_at_ms,
            published_at_ms,
            published_at_ms,
            source_identity.clone(),
            record.username,
            record.remote_addr,
            DesktopSourceStatus::Available.as_str(),
            None::<i64>,
            None::<String>,
            duplicate_key,
        ],
    )?;
    refresh_group_rollup(connection, &group_id)?;
    refresh_duplicate_info_for_key(connection, project_id, previous_duplicate_key.as_deref())?;
    if previous_duplicate_key.as_deref() != duplicate_key.as_deref() {
        refresh_duplicate_info_for_key(connection, project_id, duplicate_key.as_deref())?;
    }
    Ok(Some(group_id))
}

fn refresh_group_rollup(
    connection: &Connection,
    group_id: &str,
) -> std::result::Result<(), rusqlite::Error> {
    let primary_asset_id: Option<String> = connection
        .query_row(
            "SELECT asset_id
             FROM assets
             WHERE group_id = ?1
             ORDER BY CASE group_role
                        WHEN 'jpeg' THEN 0
                        WHEN 'raw' THEN 1
                        WHEN 'video' THEN 2
                        ELSE 3
                      END ASC,
                      published_at_ms ASC,
                      asset_id ASC
             LIMIT 1",
            params![group_id],
            |row| row.get(0),
        )
        .optional()?;
    let preview_asset_id: Option<String> = connection
        .query_row(
            "SELECT asset_id FROM assets WHERE group_id = ?1 AND group_role = 'jpeg'
             ORDER BY published_at_ms ASC, asset_id ASC LIMIT 1",
            params![group_id],
            |row| row.get(0),
        )
        .optional()?
        .or_else(|| primary_asset_id.clone());

    connection.execute(
        "UPDATE asset_groups
         SET primary_asset_id = ?2,
             preview_asset_id = ?3,
             member_count = (SELECT COUNT(*) FROM assets WHERE group_id = ?1),
             has_raw = EXISTS(SELECT 1 FROM assets WHERE group_id = ?1 AND group_role = 'raw'),
             has_jpeg = EXISTS(SELECT 1 FROM assets WHERE group_id = ?1 AND group_role = 'jpeg'),
             has_video = EXISTS(SELECT 1 FROM assets WHERE group_id = ?1 AND group_role = 'video'),
             first_capture_at_ms = (SELECT MIN(capture_at_ms) FROM assets WHERE group_id = ?1),
             last_capture_at_ms = (SELECT MAX(capture_at_ms) FROM assets WHERE group_id = ?1),
             first_received_at_ms = (SELECT MIN(received_at_ms) FROM assets WHERE group_id = ?1),
             last_received_at_ms = (SELECT MAX(received_at_ms) FROM assets WHERE group_id = ?1),
             updated_at_ms = ?4
         WHERE group_id = ?1",
        params![
            group_id,
            primary_asset_id,
            preview_asset_id,
            current_time_ms()
        ],
    )?;
    Ok(())
}

fn refresh_duplicate_info(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<(), rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT duplicate_key FROM assets
         WHERE project_id = ?1 AND duplicate_key IS NOT NULL
         GROUP BY duplicate_key
         HAVING COUNT(*) > 1",
    )?;
    let duplicate_keys = statement
        .query_map(params![project_id], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    drop(statement);

    connection.execute(
        "UPDATE assets SET duplicate_index = NULL, duplicate_count = NULL WHERE project_id = ?1",
        params![project_id],
    )?;

    for duplicate_key in duplicate_keys {
        let mut assets = connection
            .prepare(
                "SELECT asset_id FROM assets
                 WHERE project_id = ?1 AND duplicate_key = ?2
                 ORDER BY published_at_ms ASC, asset_id ASC",
            )?
            .query_map(params![project_id, duplicate_key], |row| {
                row.get::<_, String>(0)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let count = assets.len() as i64;
        for (index, asset_id) in assets.drain(..).enumerate() {
            connection.execute(
                "UPDATE assets SET duplicate_index = ?1, duplicate_count = ?2 WHERE asset_id = ?3",
                params![index as i64 + 1, count, asset_id],
            )?;
        }
    }
    Ok(())
}

fn refresh_duplicate_info_for_key(
    connection: &Connection,
    project_id: &str,
    duplicate_key: Option<&str>,
) -> std::result::Result<(), rusqlite::Error> {
    let Some(duplicate_key) = duplicate_key else {
        return Ok(());
    };
    let mut assets = connection
        .prepare(
            "SELECT asset_id FROM assets
             WHERE project_id = ?1 AND duplicate_key = ?2
             ORDER BY published_at_ms ASC, asset_id ASC",
        )?
        .query_map(params![project_id, duplicate_key], |row| {
            row.get::<_, String>(0)
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let count = assets.len() as i64;
    if count <= 1 {
        connection.execute(
            "UPDATE assets
             SET duplicate_index = NULL, duplicate_count = NULL
             WHERE project_id = ?1 AND duplicate_key = ?2",
            params![project_id, duplicate_key],
        )?;
        return Ok(());
    }

    for (index, asset_id) in assets.drain(..).enumerate() {
        connection.execute(
            "UPDATE assets SET duplicate_index = ?1, duplicate_count = ?2 WHERE asset_id = ?3",
            params![index as i64 + 1, count, asset_id],
        )?;
    }
    Ok(())
}

fn project_from_row(row: &Row<'_>) -> std::result::Result<Project, rusqlite::Error> {
    let status: String = row.get(3)?;
    Ok(Project {
        project_id: row.get(0)?,
        name: row.get(1)?,
        slug: row.get(2)?,
        status: ProjectStatus::from_str(&status),
        created_at_ms: row.get(4)?,
        updated_at_ms: row.get(5)?,
        archived_at_ms: row.get(6)?,
        default_output_target_id: row.get(7)?,
    })
}

fn desktop_scan_run_from_row(
    row: &Row<'_>,
) -> std::result::Result<DesktopScanRun, rusqlite::Error> {
    let phase: String = row.get(5)?;
    Ok(DesktopScanRun {
        scan_id: row.get(0)?,
        project_id: row.get(1)?,
        root_path: PathBuf::from(row.get::<_, String>(2)?),
        root_key: row.get(3)?,
        root_label: row.get(4)?,
        phase: DesktopScanPhase::from_str(&phase),
        files_seen: row.get::<_, i64>(6)? as usize,
        assets_indexed: row.get::<_, i64>(7)? as usize,
        groups_updated: row.get::<_, i64>(8)? as usize,
        started_at_ms: row.get(9)?,
        updated_at_ms: row.get(10)?,
        completed_at_ms: row.get(11)?,
        error: row.get(12)?,
    })
}

fn received_asset_from_row(row: &Row<'_>) -> std::result::Result<ReceivedAsset, rusqlite::Error> {
    let stored = stored_asset_from_row(row)?;
    let mut asset = ReceivedAsset::new(
        stored.asset_id,
        stored.final_filename,
        stored.size_bytes,
        import_source_from_transfer_id(&stored.transfer_id),
    );
    asset.format = stored.format;
    asset.group_key = Some(stored.normalized_stem);
    asset.storage_location = stored.final_location;
    asset.original_path = Some(stored.original_path);
    asset.username = stored.username;
    asset.display_source = stored.source_identity;
    asset.remote_addr = stored.remote_addr;
    asset.received_time_ms = stored.received_at_ms;
    asset.capture_time_ms = stored.capture_at_ms;
    asset.source_status = Some(stored.source_status);
    asset.source_modified_at_ms = stored.source_modified_at_ms;
    asset.last_seen_scan_id = stored.last_seen_scan_id;
    asset.duplicate_index = stored.duplicate_index;
    asset.duplicate_count = stored.duplicate_count;
    asset.virtual_display_path = asset.display_source.as_deref().map(|source| {
        format!(
            "{source}/{}",
            asset.original_path.as_deref().unwrap_or(&asset.filename)
        )
    });
    Ok(asset)
}

fn stored_asset_from_row(row: &Row<'_>) -> std::result::Result<StoredAsset, rusqlite::Error> {
    let media_kind: String = row.get(5)?;
    let format: String = row.get(6)?;
    let final_location_payload: Option<String> = row.get(12)?;
    Ok(StoredAsset {
        asset_id: row.get(0)?,
        project_id: row.get(1)?,
        group_id: row.get(2)?,
        transfer_id: row.get(3)?,
        group_role: row.get(4)?,
        media_kind,
        format: parse_format(&format),
        original_filename: row.get(7)?,
        final_filename: row.get(8)?,
        normalized_stem: row.get(9)?,
        original_path: row.get(10)?,
        original_parent_path: row.get(11)?,
        final_location: parse_location(final_location_payload)?,
        size_bytes: row.get::<_, i64>(13)? as u64,
        capture_at_ms: row.get(14)?,
        received_at_ms: row.get(15)?,
        published_at_ms: row.get(16)?,
        source_identity: row.get(17)?,
        username: row.get(18)?,
        remote_addr: row.get(19)?,
        source_status: row.get(20)?,
        source_modified_at_ms: row.get(21)?,
        last_seen_scan_id: row.get(22)?,
        duplicate_index: row.get::<_, Option<i64>>(23)?.map(|value| value as usize),
        duplicate_count: row.get::<_, Option<i64>>(24)?.map(|value| value as usize),
    })
}

fn stored_asset_group_from_row(
    row: &Row<'_>,
) -> std::result::Result<StoredAssetGroup, rusqlite::Error> {
    Ok(StoredAssetGroup {
        group_id: row.get(0)?,
        project_id: row.get(1)?,
        group_identity: row.get(2)?,
        display_key: row.get(3)?,
        source_identity: row.get(4)?,
        original_parent_path: row.get(5)?,
        primary_asset_id: row.get(6)?,
        preview_asset_id: row.get(7)?,
        member_count: row.get::<_, i64>(8)? as usize,
        has_raw: row.get::<_, i64>(9)? != 0,
        has_jpeg: row.get::<_, i64>(10)? != 0,
        has_video: row.get::<_, i64>(11)? != 0,
        first_capture_at_ms: row.get(12)?,
        last_capture_at_ms: row.get(13)?,
        first_received_at_ms: row.get(14)?,
        last_received_at_ms: row.get(15)?,
        created_at_ms: row.get(16)?,
        updated_at_ms: row.get(17)?,
    })
}

fn transfer_record_from_row(row: &Row<'_>) -> std::result::Result<TransferRecord, rusqlite::Error> {
    let status: String = row.get(2)?;
    let final_location_payload: Option<String> = row.get(5)?;
    Ok(TransferRecord {
        transfer_id: row.get(0)?,
        protocol: row.get(1)?,
        status: parse_transfer_status(&status),
        original_path: row.get(3)?,
        final_filename: row.get(4)?,
        final_location: parse_location(final_location_payload)?,
        size_bytes: row.get::<_, i64>(6)? as u64,
        username: row.get(7)?,
        remote_addr: row.get(8)?,
        source_name: row.get(9)?,
        started_at_ms: row.get(10)?,
        completed_at_ms: row.get(11)?,
        error: row.get(12)?,
    })
}

fn publish_item_from_row(row: &Row<'_>) -> std::result::Result<PublishQueueItem, rusqlite::Error> {
    let state: String = row.get(12)?;
    Ok(PublishQueueItem {
        queue_id: row.get(0)?,
        project_id: row.get(1)?,
        transfer_id: row.get(2)?,
        staged_path: row.get(3)?,
        final_filename: row.get(4)?,
        size_bytes: row.get::<_, i64>(5)? as u64,
        protocol: row.get(6)?,
        original_path: row.get(7)?,
        username: row.get(8)?,
        remote_addr: row.get(9)?,
        source_name: row.get(10)?,
        started_at_ms: row.get(11)?,
        state: PublishState::from_str(&state),
        attempt_count: row.get::<_, i64>(13)? as u32,
        last_error: row.get(14)?,
        next_attempt_at_ms: row.get(15)?,
        created_at_ms: row.get(16)?,
        updated_at_ms: row.get(17)?,
    })
}

fn save_technical_assessment_for_connection(
    connection: &Connection,
    assessment: TechnicalAssessment,
) -> std::result::Result<TechnicalAssessment, rusqlite::Error> {
    connection.execute(
        "INSERT INTO technical_assessments (
            asset_group_id, assessor_version, status, gate_status, defect_flags_json,
            preview_source, visual_signature, analyzed_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(asset_group_id, assessor_version) DO UPDATE SET
            status = excluded.status,
            gate_status = excluded.gate_status,
            defect_flags_json = excluded.defect_flags_json,
            preview_source = excluded.preview_source,
            visual_signature = excluded.visual_signature,
            analyzed_at_ms = excluded.analyzed_at_ms",
        params![
            assessment.asset_group_id,
            assessment.assessor_version,
            assessment.status.as_str(),
            assessment.gate_status.as_str(),
            technical_defect_flags_json(&assessment.defect_flags)?,
            assessment.preview_source,
            assessment.visual_signature,
            assessment.analyzed_at_ms,
        ],
    )?;
    technical_assessment_by_key(
        connection,
        &assessment.asset_group_id,
        &assessment.assessor_version,
    )?
    .ok_or_else(|| sqlite_data_error("technical assessment not found"))
}

fn technical_assessment_by_key(
    connection: &Connection,
    asset_group_id: &str,
    assessor_version: &str,
) -> std::result::Result<Option<TechnicalAssessment>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT asset_group_id, assessor_version, status, gate_status, defect_flags_json,
                    preview_source, visual_signature, analyzed_at_ms
             FROM technical_assessments
             WHERE asset_group_id = ?1 AND assessor_version = ?2",
            params![asset_group_id, assessor_version],
            technical_assessment_from_row,
        )
        .optional()
}

fn technical_assessments_for_asset_group_ids(
    connection: &Connection,
    asset_group_ids: &[String],
    assessor_version: &str,
) -> std::result::Result<Vec<TechnicalAssessment>, rusqlite::Error> {
    let mut assessments = Vec::new();
    for asset_group_id in asset_group_ids {
        if let Some(assessment) =
            technical_assessment_by_key(connection, asset_group_id, assessor_version)?
        {
            assessments.push(assessment);
        }
    }
    Ok(assessments)
}

fn technical_assessment_from_row(
    row: &Row<'_>,
) -> std::result::Result<TechnicalAssessment, rusqlite::Error> {
    let status: String = row.get(2)?;
    let gate_status: String = row.get(3)?;
    Ok(TechnicalAssessment {
        asset_group_id: row.get(0)?,
        assessor_version: row.get(1)?,
        status: TechnicalAssessmentStatus::from_str(&status),
        gate_status: TechnicalGateStatus::from_str(&gate_status),
        defect_flags: technical_defect_flags_from_json(row.get::<_, String>(4)?)?,
        preview_source: row.get(5)?,
        visual_signature: row.get(6)?,
        analyzed_at_ms: row.get(7)?,
    })
}

fn save_model_evaluation_for_connection(
    connection: &Connection,
    evaluation: ModelEvaluation,
) -> std::result::Result<ModelEvaluation, rusqlite::Error> {
    connection.execute(
        "INSERT INTO model_evaluations (
            evaluation_id, run_id, project_id, asset_group_id, evaluator_kind, evaluator_version,
            status, score, tier, selectable, summary, strengths_json, weaknesses_json,
            technical_warnings_json, prompt_pack_id, prompt_pack_version, prompt_hash,
            created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
         ON CONFLICT(evaluation_id) DO UPDATE SET
            run_id = excluded.run_id,
            project_id = excluded.project_id,
            asset_group_id = excluded.asset_group_id,
            evaluator_kind = excluded.evaluator_kind,
            evaluator_version = excluded.evaluator_version,
            status = excluded.status,
            score = excluded.score,
            tier = excluded.tier,
            selectable = excluded.selectable,
            summary = excluded.summary,
            strengths_json = excluded.strengths_json,
            weaknesses_json = excluded.weaknesses_json,
            technical_warnings_json = excluded.technical_warnings_json,
            prompt_pack_id = excluded.prompt_pack_id,
            prompt_pack_version = excluded.prompt_pack_version,
            prompt_hash = excluded.prompt_hash,
            updated_at_ms = excluded.updated_at_ms",
        params![
            evaluation.evaluation_id,
            evaluation.run_id,
            evaluation.project_id,
            evaluation.asset_group_id,
            evaluation.evaluator_kind.as_str(),
            evaluation.evaluator_version,
            evaluation.status.as_str(),
            evaluation.score,
            evaluation.tier.as_str(),
            evaluation.selectable,
            evaluation.summary,
            string_vec_json(&evaluation.strengths)?,
            string_vec_json(&evaluation.weaknesses)?,
            string_vec_json(&evaluation.technical_warnings)?,
            evaluation.prompt_pack_id,
            evaluation.prompt_pack_version,
            evaluation.prompt_hash,
            evaluation.created_at_ms,
            evaluation.updated_at_ms,
        ],
    )?;
    model_evaluation_by_id(connection, &evaluation.evaluation_id)?
        .ok_or_else(|| sqlite_data_error("model evaluation not found"))
}

fn model_evaluation_by_id(
    connection: &Connection,
    evaluation_id: &str,
) -> std::result::Result<Option<ModelEvaluation>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT evaluation_id, project_id, asset_group_id, evaluator_kind, evaluator_version,
                    run_id, status, score, tier, selectable, summary, strengths_json, weaknesses_json,
                    technical_warnings_json, prompt_pack_id, prompt_pack_version, prompt_hash,
                    created_at_ms, updated_at_ms
             FROM model_evaluations
             WHERE evaluation_id = ?1",
            params![evaluation_id],
            model_evaluation_from_row,
        )
        .optional()
}

fn latest_model_evaluation_for_asset_group(
    connection: &Connection,
    asset_group_id: &str,
    evaluator_version: &str,
) -> std::result::Result<Option<ModelEvaluation>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT evaluation_id, project_id, asset_group_id, evaluator_kind, evaluator_version,
                    run_id, status, score, tier, selectable, summary, strengths_json, weaknesses_json,
                    technical_warnings_json, prompt_pack_id, prompt_pack_version, prompt_hash,
                    created_at_ms, updated_at_ms
             FROM model_evaluations
             WHERE asset_group_id = ?1 AND evaluator_version = ?2
             ORDER BY updated_at_ms DESC, evaluation_id DESC
             LIMIT 1",
            params![asset_group_id, evaluator_version],
            model_evaluation_from_row,
        )
        .optional()
}

fn model_evaluations_for_asset_group_ids(
    connection: &Connection,
    asset_group_ids: &[String],
    evaluator_version: &str,
) -> std::result::Result<Vec<ModelEvaluation>, rusqlite::Error> {
    let mut evaluations = Vec::new();
    for asset_group_id in asset_group_ids {
        if let Some(evaluation) =
            latest_model_evaluation_for_asset_group(connection, asset_group_id, evaluator_version)?
        {
            evaluations.push(evaluation);
        }
    }
    Ok(evaluations)
}

fn model_evaluation_from_row(
    row: &Row<'_>,
) -> std::result::Result<ModelEvaluation, rusqlite::Error> {
    let evaluator_kind: String = row.get(3)?;
    let status: String = row.get(6)?;
    let tier: String = row.get(8)?;
    Ok(ModelEvaluation {
        evaluation_id: row.get(0)?,
        run_id: row.get(5)?,
        project_id: row.get(1)?,
        asset_group_id: row.get(2)?,
        evaluator_kind: ModelEvaluatorKind::from_str(&evaluator_kind),
        evaluator_version: row.get(4)?,
        status: ModelEvaluationStatus::from_str(&status),
        score: row.get(7)?,
        tier: ModelEvaluationTier::from_str(&tier),
        selectable: row.get::<_, bool>(9)?,
        summary: row.get(10)?,
        strengths: string_vec_from_json(row.get::<_, String>(11)?)?,
        weaknesses: string_vec_from_json(row.get::<_, String>(12)?)?,
        technical_warnings: string_vec_from_json(row.get::<_, String>(13)?)?,
        prompt_pack_id: row.get(14)?,
        prompt_pack_version: row.get(15)?,
        prompt_hash: row.get(16)?,
        created_at_ms: row.get(17)?,
        updated_at_ms: row.get(18)?,
    })
}

fn save_selection_recommendation_for_connection(
    connection: &Connection,
    recommendation: SelectionRecommendation,
) -> std::result::Result<SelectionRecommendation, rusqlite::Error> {
    connection.execute(
        "INSERT INTO selection_recommendations (
            recommendation_id, run_id, scope, project_id, subject_id, selected_asset_group_ids_json,
            candidate_asset_group_ids_json, rejected_asset_group_ids_json, source, status,
            confidence, reason, created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(recommendation_id) DO UPDATE SET
            run_id = excluded.run_id,
            scope = excluded.scope,
            project_id = excluded.project_id,
            subject_id = excluded.subject_id,
            selected_asset_group_ids_json = excluded.selected_asset_group_ids_json,
            candidate_asset_group_ids_json = excluded.candidate_asset_group_ids_json,
            rejected_asset_group_ids_json = excluded.rejected_asset_group_ids_json,
            source = excluded.source,
            status = excluded.status,
            confidence = excluded.confidence,
            reason = excluded.reason,
            updated_at_ms = excluded.updated_at_ms",
        params![
            recommendation.recommendation_id,
            recommendation.run_id,
            recommendation.scope.as_str(),
            recommendation.project_id,
            recommendation.subject_id,
            string_vec_json(&recommendation.selected_asset_group_ids)?,
            string_vec_json(&recommendation.candidate_asset_group_ids)?,
            string_vec_json(&recommendation.rejected_asset_group_ids)?,
            recommendation.source.as_str(),
            recommendation.status.as_str(),
            recommendation.confidence,
            recommendation.reason,
            recommendation.created_at_ms,
            recommendation.updated_at_ms,
        ],
    )?;
    selection_recommendation_by_id(connection, &recommendation.recommendation_id)?
        .ok_or_else(|| sqlite_data_error("selection recommendation not found"))
}

fn latest_selection_recommendation_for_connection(
    connection: &Connection,
    project_id: &str,
    scope: SelectionRecommendationScope,
    subject_id: &str,
) -> std::result::Result<Option<SelectionRecommendation>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT recommendation_id, scope, project_id, subject_id, selected_asset_group_ids_json,
                    run_id, candidate_asset_group_ids_json, rejected_asset_group_ids_json, source,
                    status, confidence, reason, created_at_ms, updated_at_ms
             FROM selection_recommendations
             WHERE project_id = ?1 AND scope = ?2 AND subject_id = ?3
             ORDER BY updated_at_ms DESC, recommendation_id DESC
             LIMIT 1",
            params![project_id, scope.as_str(), subject_id],
            selection_recommendation_from_row,
        )
        .optional()
}

fn selection_recommendation_by_id(
    connection: &Connection,
    recommendation_id: &str,
) -> std::result::Result<Option<SelectionRecommendation>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT recommendation_id, scope, project_id, subject_id, selected_asset_group_ids_json,
                    run_id, candidate_asset_group_ids_json, rejected_asset_group_ids_json, source,
                    status, confidence, reason, created_at_ms, updated_at_ms
             FROM selection_recommendations
             WHERE recommendation_id = ?1",
            params![recommendation_id],
            selection_recommendation_from_row,
        )
        .optional()
}

fn selection_recommendation_from_row(
    row: &Row<'_>,
) -> std::result::Result<SelectionRecommendation, rusqlite::Error> {
    let scope: String = row.get(1)?;
    let source: String = row.get(8)?;
    let status: String = row.get(9)?;
    Ok(SelectionRecommendation {
        recommendation_id: row.get(0)?,
        run_id: row.get(5)?,
        scope: SelectionRecommendationScope::from_str(&scope),
        project_id: row.get(2)?,
        subject_id: row.get(3)?,
        selected_asset_group_ids: string_vec_from_json(row.get::<_, String>(4)?)?,
        candidate_asset_group_ids: string_vec_from_json(row.get::<_, String>(6)?)?,
        rejected_asset_group_ids: string_vec_from_json(row.get::<_, String>(7)?)?,
        source: SelectionSource::from_str(&source)
            .ok_or_else(|| sqlite_data_error(format!("unknown selection source: {source}")))?,
        status: SelectionRecommendationStatus::from_str(&status),
        confidence: row.get(10)?,
        reason: row.get(11)?,
        created_at_ms: row.get(12)?,
        updated_at_ms: row.get(13)?,
    })
}

fn enqueue_detect_burst_job_for_connection(
    connection: &Connection,
    project_id: &str,
    asset_group_id: &str,
) -> std::result::Result<AnalysisJob, rusqlite::Error> {
    enqueue_analysis_job_for_connection(
        connection,
        NewAnalysisJob::new(
            project_id,
            AnalysisJobType::DetectBurstForAssetGroup,
            AnalysisEntityType::AssetGroup,
            asset_group_id,
            &format!("burst:{project_id}:{asset_group_id}"),
        ),
    )
}

fn enqueue_portrait_subject_assessment_job_for_connection(
    connection: &Connection,
    project_id: &str,
    asset_group_id: &str,
) -> std::result::Result<AnalysisJob, rusqlite::Error> {
    let mut job = NewAnalysisJob::new(
        project_id,
        AnalysisJobType::AssessPortraitSubject,
        AnalysisEntityType::AssetGroup,
        asset_group_id,
        &format!("subject:portrait:{project_id}:{asset_group_id}"),
    );
    job.priority = 15;
    enqueue_analysis_job_for_connection(connection, job)
}

fn should_schedule_subject_assessment_for_project(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<bool, rusqlite::Error> {
    let scene_profile = connection
        .query_row(
            "SELECT scene_profile
             FROM project_evaluation_settings
             WHERE project_id = ?1",
            params![project_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(scene_profile
        .as_deref()
        .map(SceneProfile::from_str)
        .unwrap_or(SceneProfile::General)
        == SceneProfile::Portrait)
}

fn enqueue_analysis_job_for_connection(
    connection: &Connection,
    job: NewAnalysisJob,
) -> std::result::Result<AnalysisJob, rusqlite::Error> {
    if let Some(existing) = connection
        .query_row(
            "SELECT job_id, project_id, job_type, entity_type, entity_id, dedupe_key, status,
                    priority, attempts, next_attempt_at_ms, last_error, created_at_ms, updated_at_ms
             FROM background_jobs
             WHERE dedupe_key = ?1 AND status != 'completed'
             ORDER BY created_at_ms ASC
             LIMIT 1",
            params![job.dedupe_key],
            analysis_job_from_row,
        )
        .optional()?
    {
        return Ok(existing);
    }
    let now = current_time_ms();
    let job_id = format!("analysis-job-{}-{}", now, stable_key(&job.dedupe_key));
    connection.execute(
        "INSERT INTO background_jobs (
            job_id, project_id, job_type, entity_type, entity_id, dedupe_key, status, priority,
            attempts, next_attempt_at_ms, last_error, created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, NULL, ?10, ?10)",
        params![
            job_id,
            job.project_id,
            job.job_type.as_str(),
            job.entity_type.as_str(),
            job.entity_id,
            job.dedupe_key,
            AnalysisJobStatus::Pending.as_str(),
            job.priority,
            job.next_attempt_at_ms,
            now,
        ],
    )?;
    analysis_job_by_id(connection, &job_id)?
        .ok_or_else(|| sqlite_data_error("analysis job not found"))
}

fn claim_analysis_jobs_for_connection(
    connection: &mut Connection,
    now_ms: i64,
    limit: usize,
) -> std::result::Result<Vec<AnalysisJob>, rusqlite::Error> {
    let transaction = connection.unchecked_transaction()?;
    let jobs = {
        let mut statement = transaction.prepare(
            "SELECT job_id, project_id, job_type, entity_type, entity_id, dedupe_key, status,
                    priority, attempts, next_attempt_at_ms, last_error, created_at_ms, updated_at_ms
             FROM background_jobs
             WHERE status IN ('pending', 'failed')
               AND (next_attempt_at_ms IS NULL OR next_attempt_at_ms <= ?1)
             ORDER BY priority DESC, created_at_ms ASC, job_id ASC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![now_ms, limit as i64], analysis_job_from_row)?;
        collect_rows(rows)?
    };
    for job in &jobs {
        transaction.execute(
            "UPDATE background_jobs
             SET status = ?1, updated_at_ms = ?2
             WHERE job_id = ?3",
            params![AnalysisJobStatus::Running.as_str(), now_ms, job.job_id],
        )?;
    }
    transaction.commit()?;
    Ok(jobs
        .into_iter()
        .map(|job| AnalysisJob {
            status: AnalysisJobStatus::Running,
            updated_at_ms: now_ms,
            ..job
        })
        .collect())
}

fn analysis_job_by_id(
    connection: &Connection,
    job_id: &str,
) -> std::result::Result<Option<AnalysisJob>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT job_id, project_id, job_type, entity_type, entity_id, dedupe_key, status,
                    priority, attempts, next_attempt_at_ms, last_error, created_at_ms, updated_at_ms
             FROM background_jobs
             WHERE job_id = ?1",
            params![job_id],
            analysis_job_from_row,
        )
        .optional()
}

fn analysis_job_from_row(row: &Row<'_>) -> std::result::Result<AnalysisJob, rusqlite::Error> {
    let job_type: String = row.get(2)?;
    let entity_type: String = row.get(3)?;
    let status: String = row.get(6)?;
    Ok(AnalysisJob {
        job_id: row.get(0)?,
        project_id: row.get(1)?,
        job_type: AnalysisJobType::from_str(&job_type),
        entity_type: AnalysisEntityType::from_str(&entity_type),
        entity_id: row.get(4)?,
        dedupe_key: row.get(5)?,
        status: AnalysisJobStatus::from_str(&status),
        priority: row.get(7)?,
        attempts: row.get(8)?,
        next_attempt_at_ms: row.get(9)?,
        last_error: row.get(10)?,
        created_at_ms: row.get(11)?,
        updated_at_ms: row.get(12)?,
    })
}

fn technical_defect_flags_json(
    value: &[TechnicalDefectFlag],
) -> std::result::Result<String, rusqlite::Error> {
    serde_json::to_string(value).map_err(|error| sqlite_data_error(error.to_string()))
}

fn technical_defect_flags_from_json(
    value: String,
) -> std::result::Result<Vec<TechnicalDefectFlag>, rusqlite::Error> {
    serde_json::from_str(&value).map_err(|error| sqlite_data_error(error.to_string()))
}

fn technical_assessment_policy_json(
    value: Option<&TechnicalAssessmentPolicy>,
) -> std::result::Result<Option<String>, rusqlite::Error> {
    value
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| sqlite_data_error(error.to_string()))
}

fn technical_assessment_policy_from_json(
    value: Option<String>,
) -> std::result::Result<Option<TechnicalAssessmentPolicy>, rusqlite::Error> {
    value
        .filter(|raw| !raw.trim().is_empty())
        .map(|raw| serde_json::from_str(&raw))
        .transpose()
        .map_err(|error| sqlite_data_error(error.to_string()))
}

fn string_vec_json(value: &[String]) -> std::result::Result<String, rusqlite::Error> {
    serde_json::to_string(value).map_err(|error| sqlite_data_error(error.to_string()))
}

fn string_vec_from_json(value: String) -> std::result::Result<Vec<String>, rusqlite::Error> {
    serde_json::from_str(&value).map_err(|error| sqlite_data_error(error.to_string()))
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&Row<'_>) -> std::result::Result<T, rusqlite::Error>>,
) -> std::result::Result<Vec<T>, rusqlite::Error> {
    rows.collect()
}

fn summarize_asset_groups(groups: &[ReceivedAssetGroup]) -> AssetGroupSummary {
    let mut source_counts = BTreeMap::<String, usize>::new();
    let mut remote_addr_counts = BTreeMap::<String, usize>::new();
    for group in groups {
        if let Some(source) = group.primary.display_source.as_ref() {
            *source_counts.entry(source.clone()).or_default() += 1;
        }
        if let Some(remote_addr) = group.primary.remote_addr.as_ref() {
            *remote_addr_counts.entry(remote_addr.clone()).or_default() += 1;
        }
    }
    AssetGroupSummary {
        group_count: groups.len(),
        asset_count: groups.iter().map(|group| group_assets(group).len()).sum(),
        groups_with_jpeg: groups.iter().filter(|group| group.jpeg.is_some()).count(),
        groups_with_raw: groups.iter().filter(|group| group.raw.is_some()).count(),
        groups_with_video: groups.iter().filter(|group| group.video.is_some()).count(),
        source_counts: facet_counts(source_counts),
        remote_addr_counts: facet_counts(remote_addr_counts),
    }
}

fn asset_group_matches(group: &ReceivedAssetGroup, query: &AssetGroupQuery) -> bool {
    group_assets(group).into_iter().any(|asset| {
        query
            .username
            .as_ref()
            .map(|expected| asset.username.as_ref() == Some(expected))
            .unwrap_or(true)
            && query
                .source_name
                .as_ref()
                .map(|expected| asset.display_source.as_ref() == Some(expected))
                .unwrap_or(true)
            && query
                .remote_addr
                .as_ref()
                .map(|expected| asset.remote_addr.as_ref() == Some(expected))
                .unwrap_or(true)
            && query
                .original_path
                .as_ref()
                .map(|expected| {
                    asset
                        .original_path
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&expected.to_ascii_lowercase())
                })
                .unwrap_or(true)
            && query
                .format
                .map(|expected| asset.format == expected)
                .unwrap_or(true)
            && query
                .role
                .map(|expected| asset.format.role() == expected)
                .unwrap_or(true)
    })
}

fn asset_group_matches_analysis(group: &ReceivedAssetGroup, query: &AssetGroupQuery) -> bool {
    let favorite_matches = query
        .favorite
        .map(|expected| group.user_marks.favorite == expected)
        .unwrap_or(true);
    let marked_matches = query
        .marked
        .map(|expected| group.user_marks.marked == expected)
        .unwrap_or(true);
    let collection_matches = query
        .collection
        .as_ref()
        .map(|collection| asset_group_matches_collection(group, collection))
        .unwrap_or(true);

    favorite_matches && marked_matches && collection_matches
}

fn asset_group_matches_collection(group: &ReceivedAssetGroup, collection: &str) -> bool {
    match normalized_asset_collection_key(collection).as_str() {
        "all" => true,
        "model_selects" => group.is_model_select,
        "favorites" => group.is_favorite,
        "marked" => group.is_flagged,
        "technical_risk" => {
            matches!(
                group.technical_gate_status.as_deref(),
                Some("warn" | "reject" | "inconclusive" | "unsupported")
            ) || matches!(group.model_tier.as_deref(), Some("weak" | "reject"))
        }
        "pending_analysis" => {
            group.model_status.is_none()
                || matches!(group.model_status.as_deref(), Some("pending" | "running"))
                || matches!(
                    group.technical_status.as_deref(),
                    Some("pending" | "analyzing")
                )
        }
        _ => true,
    }
}

fn normalized_asset_collection_key(collection: &str) -> String {
    match collection.trim().to_ascii_lowercase().as_str() {
        "" | "all" => "all".to_string(),
        "model_select" | "model_selects" | "algorithm_select" | "algorithm_selects" => {
            "model_selects".to_string()
        }
        "favorite" | "favorites" => "favorites".to_string(),
        "mark" | "marked" | "flag" | "flagged" => "marked".to_string(),
        "technical_risk" | "risk" => "technical_risk".to_string(),
        "pending_analysis" | "analysis_pending" => "pending_analysis".to_string(),
        value => value.to_string(),
    }
}

fn sort_asset_groups_for_query(groups: &mut [ReceivedAssetGroup], sort: AssetGroupSort) {
    match sort {
        AssetGroupSort::LatestReceived => {}
        AssetGroupSort::Filename => {
            groups.sort_by(|left, right| {
                left.group_key
                    .cmp(&right.group_key)
                    .then_with(|| left.group_id.cmp(&right.group_id))
            });
        }
        AssetGroupSort::ModelScore => {
            groups.sort_by(|left, right| {
                let left_score = group_best_score(left);
                let right_score = group_best_score(right);
                let left_own_score = left.model_score.map(|score| score as f64);
                let right_own_score = right.model_score.map(|score| score as f64);
                score_sort_key(right_score)
                    .cmp(&score_sort_key(left_score))
                    .then_with(|| {
                        score_sort_key(right_own_score).cmp(&score_sort_key(left_own_score))
                    })
                    .then_with(|| {
                        group_received_sort_time(right).cmp(&group_received_sort_time(left))
                    })
                    .then_with(|| left.group_key.cmp(&right.group_key))
            });
        }
    }
}

fn group_best_score(group: &ReceivedAssetGroup) -> Option<f64> {
    group
        .burst
        .as_ref()
        .and_then(|burst| burst.best_score)
        .or_else(|| group.model_score.map(|score| score as f64))
}

fn score_sort_key(score: Option<f64>) -> i64 {
    score
        .filter(|value| value.is_finite())
        .map(|value| (value * 1_000_000.0).round() as i64)
        .unwrap_or(i64::MIN)
}

fn group_received_sort_time(group: &ReceivedAssetGroup) -> Option<i64> {
    group_assets(group)
        .into_iter()
        .filter_map(|asset| asset.capture_time_ms.or(asset.received_time_ms))
        .max()
}

fn group_assets(group: &ReceivedAssetGroup) -> Vec<&ReceivedAsset> {
    let mut assets = Vec::new();
    push_unique_asset(&mut assets, &group.primary);
    if let Some(asset) = group.jpeg.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    if let Some(asset) = group.raw.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    if let Some(asset) = group.video.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    assets
}

fn push_unique_asset<'a>(assets: &mut Vec<&'a ReceivedAsset>, asset: &'a ReceivedAsset) {
    if !assets.iter().any(|existing| existing.id == asset.id) {
        assets.push(asset);
    }
}

fn facet_counts(counts: BTreeMap<String, usize>) -> Vec<AssetFacetCount> {
    counts
        .into_iter()
        .map(|(value, group_count)| AssetFacetCount { value, group_count })
        .collect()
}

fn final_location_json(
    location: Option<&StoredObjectLocation>,
) -> std::result::Result<Option<String>, rusqlite::Error> {
    location
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
}

fn parse_location(
    value: Option<String>,
) -> std::result::Result<Option<StoredObjectLocation>, rusqlite::Error> {
    value
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
}

fn normalized_required(field: &str, value: &str) -> Result<String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(ImporterError::internal(format!("{field} cannot be empty")));
    }
    Ok(normalized)
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn slugify(value: &str) -> String {
    let slug = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "project".to_string()
    } else {
        slug
    }
}

fn stable_key(value: &str) -> String {
    let mut hash = 1469598103934665603_u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}

fn asset_group_identity(
    project_id: &str,
    source_identity: Option<&str>,
    original_parent_path: Option<&str>,
    normalized_stem: &str,
) -> String {
    format!(
        "{}\t{}\t{}\t{}",
        project_id,
        source_identity.unwrap_or_default(),
        original_parent_path.unwrap_or_default(),
        normalized_stem
    )
}

fn normalized_stem(filename: &str) -> Option<String> {
    let name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    let (stem, _) = name.rsplit_once('.')?;
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_ascii_uppercase())
    }
}

fn original_parent_path(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let parent = normalized
        .rsplit_once('/')
        .map(|(parent, _)| parent.trim_matches('/'))?;
    if parent.is_empty() {
        None
    } else {
        Some(parent.to_string())
    }
}

fn original_filename(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn source_identity(record: &TransferRecord) -> Option<String> {
    record
        .source_name
        .clone()
        .or_else(|| record.username.clone())
        .or_else(|| record.remote_addr.clone())
}

fn duplicate_key(record: &TransferRecord) -> Option<String> {
    let original = normalized_duplicate_segment(&record.original_path)?;
    let identity = source_identity(record)
        .and_then(|value| normalized_duplicate_segment(&value))
        .unwrap_or_else(|| "-".to_string());
    Some(format!("{identity}\t{original}"))
}

fn normalized_duplicate_segment(value: &str) -> Option<String> {
    let normalized = value.trim().replace('\\', "/").to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn transfer_status_name(status: TransferStatus) -> &'static str {
    match status {
        TransferStatus::Completed => "completed",
        TransferStatus::Failed => "failed",
    }
}

fn parse_transfer_status(value: &str) -> TransferStatus {
    match value {
        "failed" => TransferStatus::Failed,
        _ => TransferStatus::Completed,
    }
}

fn format_name(format: ObjectFormat) -> &'static str {
    match format {
        ObjectFormat::Jpeg => "jpeg",
        ObjectFormat::Nef => "nef",
        ObjectFormat::Nrw => "nrw",
        ObjectFormat::Cr2 => "cr2",
        ObjectFormat::Cr3 => "cr3",
        ObjectFormat::Arw => "arw",
        ObjectFormat::Raf => "raf",
        ObjectFormat::Rw2 => "rw2",
        ObjectFormat::Orf => "orf",
        ObjectFormat::Pef => "pef",
        ObjectFormat::Dng => "dng",
        ObjectFormat::Mov => "mov",
        ObjectFormat::Mp4 => "mp4",
        ObjectFormat::Tiff => "tiff",
        ObjectFormat::Unknown => "unknown",
    }
}

fn parse_format(value: &str) -> ObjectFormat {
    match value {
        "jpeg" => ObjectFormat::Jpeg,
        "nef" => ObjectFormat::Nef,
        "nrw" => ObjectFormat::Nrw,
        "cr2" => ObjectFormat::Cr2,
        "cr3" => ObjectFormat::Cr3,
        "arw" => ObjectFormat::Arw,
        "raf" => ObjectFormat::Raf,
        "rw2" => ObjectFormat::Rw2,
        "orf" => ObjectFormat::Orf,
        "pef" => ObjectFormat::Pef,
        "dng" => ObjectFormat::Dng,
        "mov" => ObjectFormat::Mov,
        "mp4" => ObjectFormat::Mp4,
        "tiff" => ObjectFormat::Tiff,
        _ => ObjectFormat::Unknown,
    }
}

fn group_role(format: ObjectFormat) -> &'static str {
    if format == ObjectFormat::Jpeg {
        "jpeg"
    } else if format.is_raw() {
        "raw"
    } else if format.is_video() {
        "video"
    } else {
        "other"
    }
}

fn media_kind(format: ObjectFormat) -> &'static str {
    if format.is_video() {
        "video"
    } else if format.is_photo() {
        "photo"
    } else {
        "unknown"
    }
}

fn import_source_from_transfer_id(transfer_id: &str) -> ImportSource {
    if transfer_id.starts_with("desktop-scan-") {
        ImportSource::DesktopScan
    } else if transfer_id.starts_with("sftp:") {
        ImportSource::SftpPush
    } else if transfer_id.starts_with("ftp:") {
        ImportSource::FtpPush
    } else {
        ImportSource::ManualDrop
    }
}

#[cfg(test)]
mod read_write_concurrency_tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn read_connections_do_not_wait_for_write_transaction_gate() {
        let temp_dir = tempfile::tempdir().expect("temp dir should create");
        let store =
            SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
        store
            .create_project("Concurrent Reads")
            .expect("project should create");

        let writer_store = store.clone();
        let (locked_tx, locked_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let writer = thread::spawn(move || {
            writer_store
                .with_write_connection(|connection| {
                    let transaction = connection.unchecked_transaction()?;
                    transaction.execute(
                        "UPDATE projects
                         SET updated_at_ms = updated_at_ms
                         WHERE project_id = (SELECT project_id FROM projects LIMIT 1)",
                        [],
                    )?;
                    locked_tx
                        .send(())
                        .expect("lock signal should send from writer");
                    release_rx
                        .recv()
                        .expect("writer should wait for release signal");
                    transaction.commit()?;
                    Ok(())
                })
                .expect("writer should finish");
        });

        locked_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("writer should enter write transaction");
        let started_at = Instant::now();
        let projects = store
            .list_projects()
            .expect("read should not wait for writer");
        let elapsed = started_at.elapsed();

        release_tx
            .send(())
            .expect("release signal should send to writer");
        writer.join().expect("writer thread should join");

        assert_eq!(projects.len(), 1);
        assert!(
            elapsed < Duration::from_millis(200),
            "read waited for write transaction for {:?}",
            elapsed
        );
    }
}

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    AssetFormatRole, ConnectedDevice, GlobalAssetSummary, ObjectFormat, PreviewSample,
    PublishQueueSummary, PushProtocol, ReceivedAssetGroup, ReceiverRuntimeStatus,
    ReceiverSettingsConfig, SceneProfile, TransferRecord, TransferStatus,
};

#[derive(Debug, Clone)]
pub struct CameraConnectorService {
    pub(super) config_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ReceiverConfigRequest {
    pub protocol: Option<PushProtocol>,
    pub bind_host: Option<String>,
    pub port: Option<u16>,
    pub output_dir: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
    pub defer_publish: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct ReceiverSettingsUpdate {
    pub protocol: Option<PushProtocol>,
    pub bind_host: Option<String>,
    pub ftp_port: Option<u16>,
    pub sftp_port: Option<u16>,
    pub output_dir: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
    pub defer_publish: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct SavePromptPackRequest {
    pub project_id: String,
    pub prompt_pack_id: String,
    pub name: String,
    pub style_tags: Vec<String>,
    pub scene_profile: SceneProfile,
    pub prompt_text: String,
    pub now_ms: i64,
}

#[derive(Debug, Clone, Default)]
pub struct TransferQuery {
    pub status: Option<TransferStatus>,
    pub transfer_id: Option<String>,
    pub original_path: Option<String>,
    pub final_filename: Option<String>,
    pub username: Option<String>,
    pub source_name: Option<String>,
    pub remote_addr: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetGroupQuery {
    pub username: Option<String>,
    pub source_name: Option<String>,
    pub original_path: Option<String>,
    pub remote_addr: Option<String>,
    pub format: Option<ObjectFormat>,
    pub role: Option<AssetFormatRole>,
    pub sort: AssetGroupSort,
    pub collection: Option<String>,
    pub favorite: Option<bool>,
    pub marked: Option<bool>,
    #[serde(default)]
    pub user_mark_any: Vec<String>,
    pub guest_mark: Option<String>,
    pub min_model_score: Option<i64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetGroupSort {
    #[default]
    LatestReceived,
    Filename,
    ModelScore,
}

impl AssetGroupSort {
    pub fn from_wire(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "latest_received" | "latest" | "received" => Some(Self::LatestReceived),
            "filename" | "name" => Some(Self::Filename),
            "model_score" | "model_selects" => Some(Self::ModelScore),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateInfo {
    pub index: usize,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferRecordView {
    pub record: TransferRecord,
    pub display_source: Option<String>,
    pub virtual_display_path: String,
    pub final_location_kind: Option<String>,
    pub final_location_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferSummary {
    pub total_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetFacetCount {
    pub value: String,
    pub group_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetGroupSummary {
    pub group_count: usize,
    pub asset_count: usize,
    pub groups_with_jpeg: usize,
    pub groups_with_raw: usize,
    pub groups_with_video: usize,
    pub source_counts: Vec<AssetFacetCount>,
    pub remote_addr_counts: Vec<AssetFacetCount>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetGroupPage {
    pub groups: Vec<ReceivedAssetGroup>,
    pub summary: AssetGroupSummary,
    pub offset: usize,
    pub limit: usize,
    pub total_groups: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectedDeviceView {
    pub device: ConnectedDevice,
    pub display_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountView {
    pub username: String,
    pub device_name: String,
    pub password_configured: bool,
    pub online: bool,
    pub active_connections: u32,
    pub last_remote_addr: Option<String>,
    pub last_remote_port: Option<u16>,
    pub last_seen_at_ms: Option<i64>,
    pub last_disconnected_at_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishQueueFailureView {
    pub queue_id: String,
    pub transfer_id: String,
    pub final_filename: String,
    pub original_path: Option<String>,
    pub display_source: Option<String>,
    pub username: Option<String>,
    pub attempt_count: u32,
    pub last_error: Option<String>,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisDrainSummary {
    pub claimed_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetGroupModelEvaluationInput {
    pub asset_group_id: String,
    pub preview_sample: PreviewSample,
    pub preview_image_data_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemPathsView {
    pub config_path: PathBuf,
    pub state_dir: PathBuf,
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraConnectorDashboard {
    pub receiver_status: Option<ReceiverRuntimeStatus>,
    pub receiver_settings: ReceiverSettingsConfig,
    pub paths: SystemPathsView,
    pub accounts: Vec<AccountView>,
    pub devices: Vec<ConnectedDeviceView>,
    pub transfers: TransferSummary,
    pub publish_queue: PublishQueueSummary,
    pub global_assets: GlobalAssetSummary,
    pub recent_failures: Vec<TransferRecordView>,
    pub recent_publish_failures: Vec<PublishQueueFailureView>,
    pub assets: AssetGroupPage,
}

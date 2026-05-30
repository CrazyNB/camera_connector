use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    append_transfer_record, group_received_assets, read_connected_devices,
    read_receiver_runtime_status, read_transfer_log, recommend_from_scores, scan_inbox_groups,
    score_preview_sample, AnalysisEntityType, AnalysisJob, AnalysisJobType, AssetFormatRole,
    BurstGroup, CameraConnectorConfig, ConnectedDevice, ImportSource, NewAnalysisJob, ObjectFormat,
    PreviewSample, PublishQueueItem, PublishQueueSummary, PushProtocol, PushReceiverConfig,
    QualityScore, ReceivedAsset, ReceivedAssetGroup, ReceiverAccountConfig, ReceiverRuntimeStatus,
    ReceiverSettingsConfig, Result, ReviewQueueSummary, SelectionRecommendation,
    SelectionRecommendationStatus, SelectionSource, SqliteStore, StoredAsset, StoredObjectLocation,
    StrategyProfile, TransferRecord, TransferStatus,
};

#[derive(Debug, Clone)]
pub struct CameraConnectorService {
    config_path: Option<PathBuf>,
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

#[derive(Debug, Clone, Default)]
pub struct AssetGroupQuery {
    pub username: Option<String>,
    pub source_name: Option<String>,
    pub original_path: Option<String>,
    pub remote_addr: Option<String>,
    pub format: Option<ObjectFormat>,
    pub role: Option<AssetFormatRole>,
    pub sort: AssetGroupSort,
    pub recommendation_state: Option<String>,
    pub score_min: Option<f64>,
    pub score_max: Option<f64>,
    pub analysis_status: Option<String>,
    pub review_queue: Option<String>,
    pub strategy_profile_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetGroupSort {
    #[default]
    LatestReceived,
    Filename,
    GroupBestScore,
}

impl AssetGroupSort {
    pub fn from_wire(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "latest_received" | "latest" | "received" => Some(Self::LatestReceived),
            "filename" | "name" => Some(Self::Filename),
            "group_best_score" | "best_score" | "score" => Some(Self::GroupBestScore),
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
    pub recent_failures: Vec<TransferRecordView>,
    pub recent_publish_failures: Vec<PublishQueueFailureView>,
    pub assets: AssetGroupPage,
}

impl CameraConnectorService {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        Self { config_path }
    }

    pub fn config_path(&self) -> PathBuf {
        CameraConnectorConfig::resolved_path(self.config_path.as_deref())
    }

    pub fn state_dir(&self) -> PathBuf {
        CameraConnectorConfig::default_state_dir(self.config_path.as_deref())
    }

    pub fn load_config(&self) -> Result<CameraConnectorConfig> {
        CameraConnectorConfig::load(self.config_path.as_deref())
    }

    pub fn save_config(&self, config: &CameraConnectorConfig) -> Result<PathBuf> {
        config.save(self.config_path.as_deref())
    }

    pub fn receiver_config(&self, request: ReceiverConfigRequest) -> Result<PushReceiverConfig> {
        let mut app_config = self.load_config()?;
        let receiver_settings = app_config.receiver.clone();
        let protocol = request.protocol.unwrap_or(receiver_settings.protocol);
        let bind_host = request
            .bind_host
            .unwrap_or_else(|| receiver_settings.bind_host.clone());
        let port = request.port.unwrap_or(match protocol {
            PushProtocol::Ftp => receiver_settings.ftp_port,
            PushProtocol::Sftp => receiver_settings.sftp_port,
        });
        let output_dir = request
            .output_dir
            .or(receiver_settings.output_dir.clone())
            .unwrap_or_else(CameraConnectorConfig::default_output_dir);
        let state_dir = request
            .state_dir
            .or(receiver_settings.state_dir.clone())
            .unwrap_or_else(|| self.state_dir());
        let mut config = PushReceiverConfig::new(protocol, bind_host, port, output_dir)
            .with_state_dir(state_dir);
        config.advertised_host = request
            .advertised_host
            .or(receiver_settings.advertised_host);
        config.source_name = request.source_name.or(receiver_settings.source_name);
        config.defer_publish = request
            .defer_publish
            .unwrap_or(receiver_settings.defer_publish);
        let account_state_dir = receiver_settings
            .state_dir
            .clone()
            .unwrap_or_else(|| self.state_dir());
        app_config.accounts = receiver_account_configs_from_state_dir(account_state_dir)?;
        config.accounts = app_config.effective_accounts(
            request.username.as_deref(),
            request.password.as_deref(),
            config.source_name.as_deref(),
        )?;
        config.active_project_id = SqliteStore::open_state_dir(&config.state_dir)?
            .active_project()?
            .map(|project| project.project_id);
        Ok(config)
    }

    pub fn storage_state_dir(&self) -> Result<PathBuf> {
        Ok(self
            .load_config()?
            .receiver
            .state_dir
            .unwrap_or_else(|| self.state_dir()))
    }

    pub fn storage_store(&self) -> Result<SqliteStore> {
        SqliteStore::open_state_dir(self.storage_state_dir()?)
    }

    fn receiver_account_configs(&self) -> Result<BTreeMap<String, ReceiverAccountConfig>> {
        receiver_account_configs_from_state_dir(self.storage_state_dir()?)
    }

    pub fn create_project(&self, name: impl AsRef<str>) -> Result<crate::Project> {
        self.storage_store()?.create_project(name)
    }

    pub fn rename_project(
        &self,
        project_id: &str,
        name: impl AsRef<str>,
    ) -> Result<crate::Project> {
        self.storage_store()?.rename_project(project_id, name)
    }

    pub fn archive_project(&self, project_id: &str) -> Result<crate::Project> {
        self.storage_store()?.archive_project(project_id)
    }

    pub fn restore_project(&self, project_id: &str) -> Result<crate::Project> {
        self.storage_store()?.restore_project(project_id)
    }

    pub fn set_active_project(&self, project_id: &str) -> Result<()> {
        self.storage_store()?.set_active_project(project_id)
    }

    pub fn active_project(&self) -> Result<Option<crate::Project>> {
        self.storage_store()?.active_project()
    }

    pub fn list_projects(&self) -> Result<Vec<crate::Project>> {
        self.storage_store()?.list_projects()
    }

    pub fn record_project_transfer(&self, project_id: &str, record: TransferRecord) -> Result<()> {
        self.storage_store()?.record_transfer(project_id, record)
    }

    pub fn claim_next_publish_item(&self) -> Result<Option<PublishQueueItem>> {
        self.storage_store()?.claim_next_publish_item()
    }

    pub fn mark_publish_completed(&self, queue_id: &str) -> Result<()> {
        self.storage_store()?.mark_publish_completed(queue_id)
    }

    pub fn complete_publish(
        &self,
        queue_id: &str,
        final_filename: &str,
        final_location: StoredObjectLocation,
    ) -> Result<TransferRecord> {
        let state_dir = self.storage_state_dir()?;
        let record =
            self.storage_store()?
                .complete_publish(queue_id, final_filename, final_location)?;
        append_transfer_record(&state_dir, &record)?;
        Ok(record)
    }

    pub fn mark_publish_failed(&self, queue_id: &str, error: &str) -> Result<()> {
        self.storage_store()?.mark_publish_failed(queue_id, error)
    }

    pub fn release_failed_publish_retries(&self, project_id: &str) -> Result<usize> {
        self.storage_store()?
            .release_failed_publish_retries(project_id)
    }

    pub fn drain_analysis_jobs(&self, limit: usize) -> Result<AnalysisDrainSummary> {
        let store = self.storage_store()?;
        let now = current_time_ms();
        let jobs = store.claim_analysis_jobs(now, limit)?;
        let claimed_count = jobs.len();
        let mut completed_count = 0;
        let mut failed_count = 0;

        for job in jobs {
            match run_analysis_job(&store, &job) {
                Ok(()) => {
                    store.complete_analysis_job(&job.job_id)?;
                    completed_count += 1;
                }
                Err(error) => {
                    let retry_at = current_time_ms().saturating_add(30_000);
                    store.fail_analysis_job(&job.job_id, &error.to_string(), retry_at)?;
                    failed_count += 1;
                }
            }
        }

        Ok(AnalysisDrainSummary {
            claimed_count,
            completed_count,
            failed_count,
        })
    }

    pub fn score_asset_group_preview(
        &self,
        asset_group_id: &str,
        sample: PreviewSample,
        scorer_version: &str,
    ) -> Result<QualityScore> {
        let score = score_preview_sample(asset_group_id, sample, scorer_version, current_time_ms());
        let store = self.storage_store()?;
        let saved = store.save_quality_score(score)?;
        if let Some(burst) = store.burst_group_for_asset_group(&saved.asset_group_id)? {
            let profile = default_strategy_profile(&store)?;
            let dedupe_key = recommend_job_dedupe_key(&burst.burst_group_id, &profile);
            let mut job = NewAnalysisJob::new(
                &burst.project_id,
                AnalysisJobType::RecommendBurstGroup,
                AnalysisEntityType::BurstGroup,
                &burst.burst_group_id,
                &dedupe_key,
            );
            job.priority = 25;
            store.enqueue_analysis_job(job)?;
        }
        Ok(saved)
    }

    pub fn recommend_burst_group(
        &self,
        burst_group_id: &str,
        strategy_profile_id: Option<&str>,
    ) -> Result<SelectionRecommendation> {
        let store = self.storage_store()?;
        let burst = store
            .burst_group(burst_group_id)?
            .ok_or_else(|| crate::ImporterError::internal("burst group not found"))?;
        let profile = strategy_profile_id
            .and_then(|profile_id| {
                store.strategy_profiles().ok().and_then(|profiles| {
                    profiles
                        .into_iter()
                        .find(|profile| profile.profile_id == profile_id)
                })
            })
            .unwrap_or_else(StrategyProfile::general);
        let scores = store.quality_scores_for_asset_groups(&burst.member_group_ids, "local-v1")?;
        let recommendation = recommend_from_scores(
            burst_group_id,
            &profile,
            &scores,
            burst.grouping_version,
            current_time_ms(),
        );
        store.save_selection_recommendation(recommendation)
    }

    pub fn accept_recommended_best(
        &self,
        burst_group_id: &str,
        strategy_profile_id: Option<&str>,
    ) -> Result<SelectionRecommendation> {
        self.update_review_recommendation_status(
            burst_group_id,
            strategy_profile_id,
            SelectionRecommendationStatus::Accepted,
        )
    }

    pub fn mark_burst_needs_review(
        &self,
        burst_group_id: &str,
        strategy_profile_id: Option<&str>,
    ) -> Result<SelectionRecommendation> {
        self.update_review_recommendation_status(
            burst_group_id,
            strategy_profile_id,
            SelectionRecommendationStatus::NeedsReview,
        )
    }

    pub fn restore_automatic_recommendation(
        &self,
        burst_group_id: &str,
        strategy_profile_id: Option<&str>,
    ) -> Result<SelectionRecommendation> {
        self.update_review_recommendation_status(
            burst_group_id,
            strategy_profile_id,
            SelectionRecommendationStatus::Ready,
        )
    }

    pub fn clear_recommendation(
        &self,
        burst_group_id: &str,
        strategy_profile_id: Option<&str>,
    ) -> Result<SelectionRecommendation> {
        self.update_latest_recommendation_decision(
            burst_group_id,
            strategy_profile_id,
            SelectionRecommendationStatus::Cleared,
            |recommendation| {
                recommendation.best_asset_group_id = None;
                recommendation.alternate_asset_group_ids.clear();
                recommendation.low_score_asset_group_ids.clear();
                recommendation.near_duplicate_asset_group_ids.clear();
                recommendation.reasons = vec!["user cleared recommendation".to_string()];
            },
        )
    }

    pub fn keep_all_candidates(
        &self,
        burst_group_id: &str,
        strategy_profile_id: Option<&str>,
    ) -> Result<SelectionRecommendation> {
        self.update_latest_recommendation_decision(
            burst_group_id,
            strategy_profile_id,
            SelectionRecommendationStatus::KeptAll,
            |recommendation| {
                recommendation.low_score_asset_group_ids.clear();
                recommendation.near_duplicate_asset_group_ids.clear();
                recommendation.reasons = vec!["user kept all candidates".to_string()];
            },
        )
    }

    pub fn hide_low_score_candidates(
        &self,
        burst_group_id: &str,
        strategy_profile_id: Option<&str>,
    ) -> Result<SelectionRecommendation> {
        self.update_latest_recommendation_decision(
            burst_group_id,
            strategy_profile_id,
            SelectionRecommendationStatus::LowScoreHidden,
            |recommendation| {
                let low_score_ids = recommendation.low_score_asset_group_ids.clone();
                recommendation
                    .alternate_asset_group_ids
                    .retain(|group_id| !low_score_ids.iter().any(|low_id| low_id == group_id));
                recommendation.low_score_asset_group_ids.clear();
                recommendation.reasons = vec!["user hid low-score candidates".to_string()];
            },
        )
    }

    pub fn override_recommended_best(
        &self,
        burst_group_id: &str,
        best_asset_group_id: &str,
        strategy_profile_id: Option<&str>,
    ) -> Result<SelectionRecommendation> {
        let store = self.storage_store()?;
        let profile_id = normalized_strategy_profile_id(strategy_profile_id);
        let burst = store
            .burst_group(burst_group_id)?
            .ok_or_else(|| crate::ImporterError::internal("burst group not found"))?;
        let best_group_id = best_asset_group_id.trim();
        if best_group_id.is_empty() {
            return Err(crate::ImporterError::internal(
                "best asset group id cannot be empty",
            ));
        }
        if !burst
            .member_group_ids
            .iter()
            .any(|member_group_id| member_group_id == best_group_id)
        {
            return Err(crate::ImporterError::internal(
                "best asset group is not in burst group",
            ));
        }
        let mut recommendation = store
            .latest_selection_recommendation(burst_group_id, &profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("selection recommendation not found"))?;
        recommendation.best_asset_group_id = Some(best_group_id.to_string());
        recommendation.alternate_asset_group_ids = burst
            .member_group_ids
            .iter()
            .filter(|member_group_id| member_group_id.as_str() != best_group_id)
            .cloned()
            .collect();
        recommendation.source = SelectionSource::UserOverride;
        recommendation.status = SelectionRecommendationStatus::UserOverridden;
        recommendation.reasons = vec!["user selected best".to_string()];
        recommendation.updated_at_ms = current_time_ms();
        store.save_selection_recommendation(recommendation)
    }

    pub fn split_burst_member(
        &self,
        burst_group_id: &str,
        member_group_id: &str,
    ) -> Result<Option<BurstGroup>> {
        self.storage_store()?
            .split_burst_member(burst_group_id, member_group_id)
    }

    pub fn merge_burst_member(
        &self,
        target_burst_group_id: &str,
        member_group_id: &str,
    ) -> Result<Option<BurstGroup>> {
        self.storage_store()?
            .merge_burst_member(target_burst_group_id, member_group_id)
    }

    pub fn strategy_profiles(&self) -> Result<Vec<StrategyProfile>> {
        self.storage_store()?.strategy_profiles()
    }

    pub fn save_custom_strategy_profile(
        &self,
        mut profile: StrategyProfile,
    ) -> Result<StrategyProfile> {
        let profile_id = profile.profile_id.trim();
        if profile_id.is_empty() {
            return Err(crate::ImporterError::internal(
                "strategy profile id cannot be empty",
            ));
        }
        if StrategyProfile::built_in_profiles()
            .iter()
            .any(|built_in| built_in.profile_id == profile_id)
        {
            return Err(crate::ImporterError::internal(
                "built-in strategy profiles are read-only",
            ));
        }
        profile.profile_id = profile_id.to_string();
        profile.built_in = false;
        profile.weights.composition = profile.weights.composition.clamp(0.0, 0.12);
        profile.updated_at_ms = current_time_ms();
        self.storage_store()?.save_strategy_profile(profile)
    }

    pub fn project_review_queue_summary(
        &self,
        project_id: &str,
        strategy_profile_id: Option<&str>,
    ) -> Result<ReviewQueueSummary> {
        self.storage_store()?
            .review_queue_summary(project_id, strategy_profile_id)
    }

    pub fn project_review_queue_asset_group_page(
        &self,
        project_id: &str,
        strategy_profile_id: Option<&str>,
        queue: &str,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        self.storage_store()?.review_queue_asset_group_page(
            project_id,
            AssetGroupQuery {
                review_queue: Some(queue.to_string()),
                strategy_profile_id: strategy_profile_id.map(ToOwned::to_owned),
                ..AssetGroupQuery::default()
            },
            offset,
            limit,
        )
    }

    pub fn project_selects_asset_group_page(
        &self,
        project_id: &str,
        strategy_profile_id: Option<&str>,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        self.storage_store()?.selects_asset_group_page(
            project_id,
            strategy_profile_id,
            offset,
            limit,
        )
    }

    fn update_review_recommendation_status(
        &self,
        burst_group_id: &str,
        strategy_profile_id: Option<&str>,
        status: SelectionRecommendationStatus,
    ) -> Result<SelectionRecommendation> {
        let store = self.storage_store()?;
        let profile_id = normalized_strategy_profile_id(strategy_profile_id);
        store.update_latest_selection_recommendation_status(burst_group_id, &profile_id, status)
    }

    fn update_latest_recommendation_decision(
        &self,
        burst_group_id: &str,
        strategy_profile_id: Option<&str>,
        status: SelectionRecommendationStatus,
        update: impl FnOnce(&mut SelectionRecommendation),
    ) -> Result<SelectionRecommendation> {
        let store = self.storage_store()?;
        let profile_id = normalized_strategy_profile_id(strategy_profile_id);
        let mut recommendation = store
            .latest_selection_recommendation(burst_group_id, &profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("selection recommendation not found"))?;
        update(&mut recommendation);
        recommendation.source = SelectionSource::UserOverride;
        recommendation.status = status;
        recommendation.updated_at_ms = current_time_ms();
        store.save_selection_recommendation(recommendation)
    }

    pub fn set_account(
        &self,
        username: impl Into<String>,
        password: Option<&str>,
        device_name: impl Into<String>,
    ) -> Result<(ReceiverAccountConfig, PathBuf)> {
        let account = ReceiverAccountConfig::new(username, password, device_name)?;
        let stored = self.storage_store()?.upsert_receiver_account(account)?;
        Ok((stored.into_account_config()?, self.config_path()))
    }

    pub fn remove_account(&self, username: &str) -> Result<(bool, PathBuf)> {
        let removed = self.storage_store()?.remove_receiver_account(username)?;
        Ok((removed, self.config_path()))
    }

    pub fn set_receiver_settings(
        &self,
        update: ReceiverSettingsUpdate,
    ) -> Result<(ReceiverSettingsConfig, PathBuf)> {
        let mut config = self.load_config()?;
        if let Some(protocol) = update.protocol {
            config.receiver.protocol = protocol;
        }
        if let Some(bind_host) = update.bind_host {
            config.receiver.bind_host = bind_host;
        }
        if let Some(ftp_port) = update.ftp_port {
            config.receiver.ftp_port = ftp_port;
        }
        if let Some(sftp_port) = update.sftp_port {
            config.receiver.sftp_port = sftp_port;
        }
        if let Some(output_dir) = update.output_dir {
            config.receiver.output_dir = Some(output_dir);
        }
        if let Some(state_dir) = update.state_dir {
            config.receiver.state_dir = Some(state_dir);
        }
        if let Some(advertised_host) = update.advertised_host {
            config.receiver.advertised_host = Some(advertised_host);
        }
        if let Some(source_name) = update.source_name {
            config.receiver.source_name = Some(source_name);
        }
        if let Some(defer_publish) = update.defer_publish {
            config.receiver.defer_publish = defer_publish;
        }
        let settings = config.receiver.clone();
        let path = self.save_config(&config)?;
        Ok((settings, path))
    }

    pub fn accounts(&self) -> Result<Vec<AccountView>> {
        let state_dir = self.storage_state_dir()?;
        let accounts = self
            .receiver_account_configs()?
            .into_values()
            .map(account_view)
            .collect();
        let devices = self.connected_devices(&state_dir, None, false)?;
        Ok(accounts_with_devices(accounts, &devices))
    }

    pub fn diagnostic_inbox_groups(
        &self,
        output_dir: impl AsRef<Path>,
        source: ImportSource,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        scan_inbox_groups(output_dir, source)
    }

    pub fn diagnostic_transfer_asset_groups(
        &self,
        state_dir: impl AsRef<Path>,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        self.diagnostic_transfer_asset_groups_with_query(state_dir, AssetGroupQuery::default())
    }

    pub fn diagnostic_transfer_asset_groups_with_query(
        &self,
        state_dir: impl AsRef<Path>,
        query: AssetGroupQuery,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        let accounts = self.receiver_account_configs()?;
        let records = read_transfer_log(state_dir)?
            .into_iter()
            .filter(|record| record.status == TransferStatus::Completed)
            .collect::<Vec<_>>();
        let duplicates = duplicate_info_by_transfer_id(&records, &accounts);
        let assets = records
            .into_iter()
            .map(|record| asset_from_transfer_record(record, &accounts, &duplicates))
            .filter(|asset| asset.format.is_supported_media())
            .collect::<Vec<_>>();
        Ok(group_received_assets(assets)
            .into_iter()
            .filter(|group| asset_group_matches(group, &query))
            .collect())
    }

    pub fn diagnostic_transfer_asset_summary_with_query(
        &self,
        state_dir: impl AsRef<Path>,
        query: AssetGroupQuery,
    ) -> Result<AssetGroupSummary> {
        self.diagnostic_transfer_asset_groups_with_query(state_dir, query)
            .map(|groups| summarize_asset_groups(&groups))
    }

    pub fn diagnostic_transfer_asset_group_page_with_query(
        &self,
        state_dir: impl AsRef<Path>,
        query: AssetGroupQuery,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        let groups = self.diagnostic_transfer_asset_groups_with_query(state_dir, query)?;
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
    }

    pub fn project_asset_group_page_with_query(
        &self,
        project_id: &str,
        query: AssetGroupQuery,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        if query
            .review_queue
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_some()
        {
            return self
                .storage_store()?
                .review_queue_asset_group_page(project_id, query, offset, limit);
        }
        self.storage_store()?
            .asset_group_page(project_id, query, offset, limit)
    }

    pub fn project_group_assets(
        &self,
        project_id: &str,
        group_id: &str,
    ) -> Result<Vec<StoredAsset>> {
        self.storage_store()?.assets_for_group(project_id, group_id)
    }

    pub fn move_project_asset_group(
        &self,
        source_project_id: &str,
        group_id: &str,
        target_project_id: &str,
    ) -> Result<Option<crate::StoredAssetGroup>> {
        self.storage_store()?
            .move_asset_group(source_project_id, group_id, target_project_id)
    }

    pub fn receiver_status(
        &self,
        output_dir: impl AsRef<Path>,
    ) -> Result<Option<ReceiverRuntimeStatus>> {
        read_receiver_runtime_status(output_dir)
    }

    pub fn diagnostic_transfers(
        &self,
        output_dir: impl AsRef<Path>,
        query: TransferQuery,
    ) -> Result<Vec<TransferRecordView>> {
        let accounts = self.receiver_account_configs()?;
        let views = read_transfer_log(output_dir)?
            .into_iter()
            .filter(|record| transfer_matches(record, &query, &accounts))
            .map(|record| {
                let display_source = record_display_source(&record, &accounts);
                let virtual_display_path = record.virtual_display_path(display_source.as_deref());
                TransferRecordView {
                    final_location_kind: record.final_location_kind().map(ToOwned::to_owned),
                    final_location_label: record.final_location_label(),
                    record,
                    display_source,
                    virtual_display_path,
                }
            })
            .collect::<Vec<_>>();
        Ok(views)
    }

    pub fn project_transfers(
        &self,
        project_id: &str,
        query: TransferQuery,
    ) -> Result<Vec<TransferRecordView>> {
        let accounts = self.receiver_account_configs()?;
        let views = self
            .storage_store()?
            .transfer_records(project_id)?
            .into_iter()
            .filter(|record| transfer_matches(record, &query, &accounts))
            .map(|record| {
                let display_source = record_display_source(&record, &accounts);
                let virtual_display_path = record.virtual_display_path(display_source.as_deref());
                TransferRecordView {
                    final_location_kind: record.final_location_kind().map(ToOwned::to_owned),
                    final_location_label: record.final_location_label(),
                    record,
                    display_source,
                    virtual_display_path,
                }
            })
            .collect::<Vec<_>>();
        Ok(views)
    }

    pub fn diagnostic_transfer_summary_with_query(
        &self,
        output_dir: impl AsRef<Path>,
        query: TransferQuery,
    ) -> Result<TransferSummary> {
        let accounts = self.receiver_account_configs()?;
        let records = read_transfer_log(output_dir)?
            .into_iter()
            .filter(|record| transfer_matches(record, &query, &accounts))
            .collect::<Vec<_>>();
        Ok(summarize_transfers(&records))
    }

    pub fn project_transfer_summary_with_query(
        &self,
        project_id: &str,
        query: TransferQuery,
    ) -> Result<TransferSummary> {
        let records = self
            .project_transfers(project_id, query)?
            .into_iter()
            .map(|view| view.record)
            .collect::<Vec<_>>();
        Ok(summarize_transfers(&records))
    }

    pub fn diagnostic_recent_failed_transfers(
        &self,
        output_dir: impl AsRef<Path>,
        query: TransferQuery,
        limit: usize,
    ) -> Result<Vec<TransferRecordView>> {
        let mut views = self.diagnostic_transfers(
            output_dir,
            TransferQuery {
                status: Some(TransferStatus::Failed),
                ..query
            },
        )?;
        views.sort_by(|left, right| {
            let left_at = left
                .record
                .completed_at_ms
                .unwrap_or(left.record.started_at_ms);
            let right_at = right
                .record
                .completed_at_ms
                .unwrap_or(right.record.started_at_ms);
            right_at
                .cmp(&left_at)
                .then_with(|| right.record.started_at_ms.cmp(&left.record.started_at_ms))
                .then_with(|| right.record.transfer_id.cmp(&left.record.transfer_id))
        });
        views.truncate(limit);
        Ok(views)
    }

    pub fn project_recent_failed_transfers(
        &self,
        project_id: &str,
        query: TransferQuery,
        limit: usize,
    ) -> Result<Vec<TransferRecordView>> {
        let mut views = self.project_transfers(
            project_id,
            TransferQuery {
                status: Some(TransferStatus::Failed),
                ..query
            },
        )?;
        views.truncate(limit);
        Ok(views)
    }

    pub fn project_recent_publish_failures(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<Vec<PublishQueueFailureView>> {
        let accounts = self.receiver_account_configs()?;
        let failures = self
            .storage_store()?
            .failed_publish_items(project_id, limit)?
            .into_iter()
            .map(|item| {
                let display_source = publish_item_display_source(&item, &accounts);
                PublishQueueFailureView {
                    queue_id: item.queue_id,
                    transfer_id: item.transfer_id,
                    final_filename: item.final_filename,
                    original_path: item.original_path,
                    display_source,
                    username: item.username,
                    attempt_count: item.attempt_count,
                    last_error: item.last_error,
                    updated_at_ms: item.updated_at_ms,
                }
            })
            .collect::<Vec<_>>();
        Ok(failures)
    }

    pub fn connected_devices(
        &self,
        output_dir: impl AsRef<Path>,
        username: Option<&str>,
        online: bool,
    ) -> Result<Vec<ConnectedDeviceView>> {
        let accounts = self.receiver_account_configs()?;
        let views = read_connected_devices(output_dir)?
            .into_iter()
            .filter(|device| device_matches(device, username, online))
            .map(|device| {
                let display_source = device_display_source(&device, &accounts)
                    .unwrap_or_else(|| remote_addr_display_label(&device.remote_addr));
                ConnectedDeviceView {
                    device,
                    display_source,
                }
            })
            .collect::<Vec<_>>();
        Ok(views)
    }

    pub fn project_dashboard(
        &self,
        project_id: &str,
        asset_query: AssetGroupQuery,
        offset: usize,
        limit: usize,
        online_devices_only: bool,
    ) -> Result<CameraConnectorDashboard> {
        let state_dir = self.storage_state_dir()?;
        let config = self.load_config()?;
        let receiver_settings = config.receiver.clone();
        let receiver_status = self.receiver_status(&state_dir)?;
        let devices = self.connected_devices(
            &state_dir,
            asset_query.username.as_deref(),
            online_devices_only,
        )?;
        let accounts = accounts_with_devices(self.accounts()?, &devices);
        let store = self.storage_store()?;
        let transfer_query = transfer_query_from_asset_query(&asset_query);
        let transfers =
            self.project_transfer_summary_with_query(project_id, transfer_query.clone())?;
        let output_dir = receiver_status
            .as_ref()
            .and_then(|status| status.output_dir.clone())
            .or_else(|| receiver_settings.output_dir.clone())
            .or_else(|| Some(CameraConnectorConfig::default_output_dir()));
        Ok(CameraConnectorDashboard {
            receiver_settings,
            paths: SystemPathsView {
                config_path: self.config_path(),
                state_dir: state_dir.clone(),
                output_dir,
            },
            receiver_status,
            accounts,
            devices,
            transfers,
            publish_queue: store.publish_queue_summary(project_id)?,
            recent_failures: self.project_recent_failed_transfers(project_id, transfer_query, 5)?,
            recent_publish_failures: self.project_recent_publish_failures(project_id, 5)?,
            assets: store.asset_group_page(project_id, asset_query, offset, limit)?,
        })
    }
}

fn receiver_account_configs_from_state_dir(
    state_dir: impl AsRef<Path>,
) -> Result<BTreeMap<String, ReceiverAccountConfig>> {
    SqliteStore::open_state_dir(state_dir)?
        .receiver_accounts()?
        .into_iter()
        .map(|account| {
            let account = account.into_account_config()?;
            Ok((account.username.clone(), account))
        })
        .collect()
}

fn transfer_query_from_asset_query(query: &AssetGroupQuery) -> TransferQuery {
    TransferQuery {
        status: None,
        transfer_id: None,
        original_path: query.original_path.clone(),
        final_filename: None,
        username: query.username.clone(),
        source_name: query.source_name.clone(),
        remote_addr: query.remote_addr.clone(),
    }
}

fn summarize_transfers(records: &[TransferRecord]) -> TransferSummary {
    TransferSummary {
        total_count: records.len(),
        completed_count: records
            .iter()
            .filter(|record| record.status == TransferStatus::Completed)
            .count(),
        failed_count: records
            .iter()
            .filter(|record| record.status == TransferStatus::Failed)
            .count(),
    }
}

fn account_view(account: ReceiverAccountConfig) -> AccountView {
    let password_configured = account.password_configured();
    AccountView {
        username: account.username,
        device_name: account.device_name,
        password_configured,
        online: false,
        active_connections: 0,
        last_remote_addr: None,
        last_remote_port: None,
        last_seen_at_ms: None,
        last_disconnected_at_ms: None,
    }
}

fn accounts_with_devices(
    accounts: Vec<AccountView>,
    devices: &[ConnectedDeviceView],
) -> Vec<AccountView> {
    accounts
        .into_iter()
        .map(|mut account| {
            account.online = false;
            account.active_connections = 0;
            account.last_remote_addr = None;
            account.last_remote_port = None;
            account.last_seen_at_ms = None;
            account.last_disconnected_at_ms = None;
            let matching_devices = devices
                .iter()
                .filter(|view| view.device.username.as_deref() == Some(account.username.as_str()));
            let mut active_connections = 0u32;
            let mut latest: Option<&ConnectedDevice> = None;
            for device in matching_devices.map(|view| &view.device) {
                active_connections = active_connections.saturating_add(device.active_connections);
                latest = Some(match latest {
                    Some(current)
                        if current.online && !device.online
                            || current.online == device.online
                                && current.last_seen_at_ms >= device.last_seen_at_ms =>
                    {
                        current
                    }
                    _ => device,
                });
            }
            if let Some(device) = latest {
                account.online = device.online;
                account.active_connections = active_connections;
                account.last_remote_addr = Some(device.remote_addr.clone());
                account.last_remote_port = device.last_remote_port;
                account.last_seen_at_ms = Some(device.last_seen_at_ms);
                account.last_disconnected_at_ms = device.last_disconnected_at_ms;
            }
            account
        })
        .collect()
}

fn run_analysis_job(store: &SqliteStore, job: &AnalysisJob) -> Result<()> {
    match (job.job_type, job.entity_type) {
        (AnalysisJobType::DetectBurstForAssetGroup, AnalysisEntityType::AssetGroup) => {
            let profile = default_strategy_profile(store)?;
            let _ =
                store.detect_bursts_for_asset_group(&job.project_id, &job.entity_id, &profile)?;
            Ok(())
        }
        (AnalysisJobType::ScoreAssetGroup, AnalysisEntityType::AssetGroup) => Ok(()),
        (AnalysisJobType::RecommendBurstGroup, AnalysisEntityType::BurstGroup) => {
            let profile = default_strategy_profile(store)?;
            let burst = store
                .burst_group(&job.entity_id)?
                .ok_or_else(|| crate::ImporterError::internal("burst group not found"))?;
            let scores =
                store.quality_scores_for_asset_groups(&burst.member_group_ids, "local-v1")?;
            let recommendation = recommend_from_scores(
                &burst.burst_group_id,
                &profile,
                &scores,
                burst.grouping_version,
                current_time_ms(),
            );
            store.save_selection_recommendation(recommendation)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn default_strategy_profile(store: &SqliteStore) -> Result<StrategyProfile> {
    Ok(store
        .strategy_profiles()?
        .into_iter()
        .find(|profile| profile.profile_id == "general")
        .unwrap_or_else(StrategyProfile::general))
}

fn normalized_strategy_profile_id(strategy_profile_id: Option<&str>) -> String {
    strategy_profile_id
        .map(str::trim)
        .filter(|profile_id| !profile_id.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "general".to_string())
}

fn recommend_job_dedupe_key(burst_group_id: &str, profile: &StrategyProfile) -> String {
    format!(
        "recommend:{burst_group_id}:{}:{}",
        profile.profile_id, profile.strategy_version
    )
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
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

fn facet_counts(counts: BTreeMap<String, usize>) -> Vec<AssetFacetCount> {
    counts
        .into_iter()
        .map(|(value, group_count)| AssetFacetCount { value, group_count })
        .collect()
}

fn asset_group_matches(group: &ReceivedAssetGroup, query: &AssetGroupQuery) -> bool {
    group_assets(group)
        .into_iter()
        .any(|asset| asset_matches(asset, query))
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

fn asset_matches(asset: &ReceivedAsset, query: &AssetGroupQuery) -> bool {
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
}

fn asset_from_transfer_record(
    record: TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
    duplicates: &BTreeMap<String, DuplicateInfo>,
) -> ReceivedAsset {
    let storage_location = record.resolved_final_location();
    let display_source = record_display_source(&record, accounts);
    let virtual_display_path = record.virtual_display_path(display_source.as_deref());
    let mut asset = ReceivedAsset::new(
        record.transfer_id.clone(),
        record.final_filename.clone(),
        record.size_bytes,
        import_source_from_protocol(&record.protocol),
    );
    asset.received_time_ms = record.completed_at_ms.or(Some(record.started_at_ms));
    asset.storage_location = storage_location;
    asset.original_path = Some(record.original_path);
    asset.username = record.username;
    asset.display_source = display_source;
    asset.remote_addr = record.remote_addr;
    asset.virtual_display_path = Some(virtual_display_path);
    if let Some(duplicate) = duplicates.get(&record.transfer_id) {
        asset.duplicate_index = Some(duplicate.index);
        asset.duplicate_count = Some(duplicate.count);
    }
    asset
}

fn duplicate_info_by_transfer_id(
    records: &[TransferRecord],
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> BTreeMap<String, DuplicateInfo> {
    let mut duplicate_keys = BTreeMap::<String, Vec<&TransferRecord>>::new();
    for record in records {
        if let Some(key) = duplicate_key(record, accounts) {
            duplicate_keys.entry(key).or_default().push(record);
        }
    }

    let mut duplicates = BTreeMap::new();
    for duplicate_records in duplicate_keys.values_mut() {
        if duplicate_records.len() < 2 {
            continue;
        }
        duplicate_records.sort_by_key(|record| {
            (
                record.completed_at_ms.unwrap_or(record.started_at_ms),
                record.started_at_ms,
                record.transfer_id.clone(),
            )
        });
        let count = duplicate_records.len();
        for (index, record) in duplicate_records.iter().enumerate() {
            duplicates.insert(
                record.transfer_id.clone(),
                DuplicateInfo {
                    index: index + 1,
                    count,
                },
            );
        }
    }
    duplicates
}

fn duplicate_key(
    record: &TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    let original_path = normalized_duplicate_segment(&record.original_path)?;
    let identity = record
        .username
        .as_deref()
        .and_then(normalized_duplicate_segment)
        .or_else(|| {
            record_display_source(record, accounts)
                .and_then(|value| normalized_duplicate_segment(&value))
        })
        .or_else(|| {
            record
                .remote_addr
                .as_deref()
                .and_then(normalized_duplicate_segment)
        })
        .unwrap_or_else(|| "-".to_string());
    Some(format!("{identity}\t{original_path}"))
}

fn normalized_duplicate_segment(value: &str) -> Option<String> {
    let normalized = value.trim().replace('\\', "/").to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn import_source_from_protocol(protocol: &str) -> ImportSource {
    match protocol.to_ascii_lowercase().as_str() {
        "ftp" => ImportSource::FtpPush,
        "sftp" => ImportSource::SftpPush,
        _ => ImportSource::ManualDrop,
    }
}

fn transfer_matches(
    record: &TransferRecord,
    query: &TransferQuery,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> bool {
    query
        .status
        .map(|expected| record.status == expected)
        .unwrap_or(true)
        && query
            .username
            .as_ref()
            .map(|expected| record.username.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .source_name
            .as_ref()
            .map(|expected| record_display_source(record, accounts).as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .remote_addr
            .as_ref()
            .map(|expected| record.remote_addr.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .transfer_id
            .as_ref()
            .map(|expected| record.transfer_id.contains(expected))
            .unwrap_or(true)
        && query
            .original_path
            .as_ref()
            .map(|expected| {
                record
                    .original_path
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
            })
            .unwrap_or(true)
        && query
            .final_filename
            .as_ref()
            .map(|expected| {
                record
                    .final_filename
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
            })
            .unwrap_or(true)
}

fn record_display_source(
    record: &TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    record
        .username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| record.source_name.clone())
}

fn publish_item_display_source(
    item: &PublishQueueItem,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    item.username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| item.source_name.clone())
}

fn device_matches(device: &ConnectedDevice, username: Option<&str>, online: bool) -> bool {
    (!online || device.online)
        && username
            .map(|expected| device.username.as_deref() == Some(expected))
            .unwrap_or(true)
}

fn device_display_source(
    device: &ConnectedDevice,
    accounts: &BTreeMap<String, crate::ReceiverAccountConfig>,
) -> Option<String> {
    device
        .username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| device.source_name.clone())
}

fn remote_addr_display_label(remote_addr: &str) -> String {
    if let Some(last_octet) = remote_addr
        .rsplit('.')
        .next()
        .filter(|value| !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit()))
        .and_then(|value| value.parse::<u8>().ok())
    {
        return format!("IP-{last_octet:03}");
    }

    let digits = remote_addr
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>();
    if digits.is_empty() {
        "IP".to_string()
    } else {
        let start = digits.len().saturating_sub(3);
        format!("IP-{:0>3}", &digits[start..])
    }
}

use std::fs;
use std::path::Path;

mod analysis;
mod analysis_helpers;
mod analysis_recommendations;
mod asset_groups;
mod config;
mod desktop_scan;
mod model_providers;
mod projects;
mod prompt_packs;
mod publish;
mod receiver;
mod support;
mod transfers;
mod types;

use analysis_helpers::{
    evaluator_version_for_runtime_provider, model_evaluation_skipped,
    model_provider_ready_for_work, provider_configured_for_project_from_list,
    provider_has_required_secret, runtime_model_provider_for_project_from_list,
    technical_assessment_policy_for_settings,
};
use asset_groups::{
    asset_from_transfer_record, asset_group_matches, duplicate_info_by_transfer_id,
    summarize_asset_groups,
};
use model_providers::{
    model_provider_config_by_id, model_provider_settings_from_config,
    model_provider_settings_to_config, normalized_model_provider_settings_id,
    runtime_model_providers_from_config, upsert_model_provider_config, RuntimeModelProvider,
};
use support::{
    accounts_with_devices, active_lan_share_session, current_time_ms,
    default_burst_grouping_profile, ensure_service_project_is_active, evaluation_run_id,
    mapped_project_sync_group_ids, project_sync_model_evaluation,
    receiver_account_configs_from_state_dir, recommend_job_dedupe_key,
    should_schedule_subject_assessment_for_settings, stable_project_sync_key, stable_prompt_hash,
    transfer_query_from_asset_query,
};

use crate::{
    group_received_assets, match_project_sync_snapshot, read_transfer_log,
    scan_received_asset_groups, AssetUserMarks, CameraConnectorConfig, GuestMark, ImportSource,
    LanShareGuestMark, LanShareSession, ModelProviderSettings, ProjectEvaluationSettings,
    ProjectRecommendationMode, ProjectSyncApplySummary, ProjectSyncSnapshot, PromptPack,
    ReceivedAssetGroup, Result, SceneProfile, SelectionRecommendation,
    SelectionRecommendationScope, SelectionRecommendationStatus, SelectionSource, StoredAsset,
    StoredObjectLocation, SubjectAssessment, TransferStatus,
};
pub use desktop_scan::DesktopProjectScanResult;
use prompt_packs::{
    builtin_prompt_packs, default_prompt_pack_capabilities, load_user_prompt_packs,
    normalized_distribution_folder, normalized_prompt_pack_name, prompt_distribution_dir,
    prompt_pack_content_json_from_input, prompt_pack_dir, prompt_pack_markdown_from_json,
    prompt_pack_sort_key, prompt_snapshot_for_settings, save_user_prompt_pack, stable_id_fragment,
    unique_user_prompt_pack_id,
};
pub use types::*;

const MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION: &str = "model-evaluation-v1";

impl CameraConnectorService {
    pub fn model_provider_settings(&self) -> Result<Option<crate::ModelProviderSettings>> {
        Ok(self
            .runtime_model_provider()?
            .map(|provider| provider.settings))
    }

    pub fn model_provider_settings_list(&self) -> Result<Vec<ModelProviderSettings>> {
        Ok(self
            .runtime_model_providers()?
            .into_iter()
            .map(|provider| provider.settings)
            .collect())
    }

    pub fn save_model_provider_settings(
        &self,
        settings: crate::ModelProviderSettings,
    ) -> Result<crate::ModelProviderSettings> {
        self.save_model_provider_settings_with_api_key(settings, None)
    }

    pub fn save_model_provider_settings_with_api_key(
        &self,
        settings: ModelProviderSettings,
        api_key: Option<String>,
    ) -> Result<ModelProviderSettings> {
        let mut config = self.load_config()?;
        let settings_id = normalized_model_provider_settings_id(&settings.settings_id);
        let existing_api_key = model_provider_config_by_id(&config, &settings_id)
            .and_then(|existing| existing.api_key.clone());
        let saved_config =
            model_provider_settings_to_config(settings, api_key.or(existing_api_key));
        upsert_model_provider_config(&mut config, saved_config.clone());
        self.save_config(&config)?;
        Ok(model_provider_settings_from_config(saved_config))
    }

    pub fn delete_model_provider_settings(&self, settings_id: &str) -> Result<bool> {
        let mut config = self.load_config()?;
        let settings_id = normalized_model_provider_settings_id(settings_id);
        let original_len = config.model_providers.len();
        config.model_providers.retain(|provider| {
            normalized_model_provider_settings_id(&provider.settings_id) != settings_id
        });
        let removed = config.model_providers.len() != original_len;
        if removed {
            self.save_config(&config)?;
        }
        Ok(removed)
    }

    pub fn project_evaluation_settings(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEvaluationSettings>> {
        self.storage_store()?
            .project_evaluation_settings(project_id)
    }

    pub fn save_project_evaluation_settings(
        &self,
        mut settings: ProjectEvaluationSettings,
    ) -> Result<ProjectEvaluationSettings> {
        settings.project_recommendation_mode = ProjectRecommendationMode::Manual;
        if let Some(prompt_pack_id) = settings.prompt_pack_id.as_deref() {
            let Some(pack) = self.prompt_pack_by_id(prompt_pack_id)? else {
                return Err(crate::ImporterError::internal("prompt pack not found"));
            };
            if !pack.enabled {
                return Err(crate::ImporterError::internal("prompt pack is disabled"));
            }
        }
        self.storage_store()?
            .save_project_evaluation_settings(settings)
    }

    pub fn should_schedule_subject_assessment(&self, project_id: &str) -> Result<bool> {
        let settings = self
            .project_evaluation_settings(project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            });
        Ok(should_schedule_subject_assessment_for_settings(&settings))
    }

    pub fn save_subject_assessment(
        &self,
        assessment: SubjectAssessment,
    ) -> Result<SubjectAssessment> {
        self.storage_store()?.save_subject_assessment(assessment)
    }

    pub fn subject_assessments_for_asset_groups(
        &self,
        project_id: &str,
        group_ids: &[String],
    ) -> Result<Vec<SubjectAssessment>> {
        self.storage_store()?
            .subject_assessments_for_asset_groups(project_id, group_ids)
    }

    pub fn prompt_packs_for_project(&self, _project_id: &str) -> Result<Vec<PromptPack>> {
        self.global_prompt_packs()
    }

    pub fn global_prompt_packs(&self) -> Result<Vec<PromptPack>> {
        let mut packs = builtin_prompt_packs();
        packs.extend(load_user_prompt_packs(&self.storage_state_dir()?)?);
        packs.sort_by(|left, right| {
            prompt_pack_sort_key(left)
                .cmp(&prompt_pack_sort_key(right))
                .then_with(|| left.name.cmp(&right.name))
                .then_with(|| left.prompt_pack_id.cmp(&right.prompt_pack_id))
        });
        Ok(packs.into_iter().filter(|pack| pack.enabled).collect())
    }

    pub fn prompt_pack_by_id(&self, prompt_pack_id: &str) -> Result<Option<PromptPack>> {
        Ok(self
            .global_prompt_packs()?
            .into_iter()
            .find(|pack| pack.prompt_pack_id == prompt_pack_id))
    }

    pub fn prompt_text_for_pack(&self, prompt_pack_id: &str) -> Result<Option<String>> {
        Ok(self
            .global_prompt_packs()?
            .into_iter()
            .find(|pack| pack.prompt_pack_id == prompt_pack_id)
            .map(|pack| pack.prompt_text))
    }

    pub fn prompt_markdown_for_pack(&self, prompt_pack_id: &str) -> Result<Option<String>> {
        self.prompt_text_for_pack(prompt_pack_id)?
            .map(|prompt_text| prompt_pack_markdown_from_json(&prompt_text))
            .transpose()
    }

    pub fn create_global_prompt_pack(
        &self,
        name: impl AsRef<str>,
        style_tags: Vec<String>,
        scene_profile: SceneProfile,
        distribution_folder: impl AsRef<str>,
        shared_preference: impl AsRef<str>,
        now_ms: i64,
    ) -> Result<PromptPack> {
        let name = name.as_ref().trim();
        if name.is_empty() {
            return Err(crate::ImporterError::internal(
                "prompt pack name is required",
            ));
        }
        let state_dir = self.storage_state_dir()?;
        let distribution_folder = normalized_distribution_folder(distribution_folder.as_ref());
        let prompt_text = prompt_pack_content_json_from_input(shared_preference.as_ref())?;
        let prompt_pack_id = unique_user_prompt_pack_id(&state_dir, name)?;
        let pack = PromptPack {
            prompt_pack_id: prompt_pack_id.clone(),
            distribution_folder,
            name: name.to_string(),
            version: format!("user-{now_ms}"),
            author: "user".to_string(),
            style_tags: style_tags
                .into_iter()
                .map(|tag| tag.trim().to_string())
                .filter(|tag| !tag.is_empty())
                .collect(),
            scene_profile,
            schema: MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION.to_string(),
            capabilities: default_prompt_pack_capabilities(),
            built_in: false,
            enabled: true,
            prompt_hash: stable_prompt_hash(MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION, &prompt_text),
            prompt_text,
            updated_at_ms: now_ms,
        };
        save_user_prompt_pack(&state_dir, &pack)
    }

    pub fn fork_global_prompt_pack(
        &self,
        source_profile_id: &str,
        name: impl AsRef<str>,
        distribution_folder: impl AsRef<str>,
        now_ms: i64,
    ) -> Result<PromptPack> {
        let source = self
            .prompt_pack_by_id(source_profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt pack not found"))?;
        if !source.enabled {
            return Err(crate::ImporterError::internal("prompt pack is disabled"));
        }
        let state_dir = self.storage_state_dir()?;
        let name = normalized_prompt_pack_name(name.as_ref(), &source.name);
        let distribution_folder = normalized_distribution_folder(distribution_folder.as_ref());
        let prompt_pack_id = unique_user_prompt_pack_id(&state_dir, &name)?;
        let prompt_text = source.prompt_text.clone();
        let pack = PromptPack {
            prompt_pack_id: prompt_pack_id.clone(),
            distribution_folder,
            name,
            version: format!("user-{now_ms}"),
            author: "user".to_string(),
            style_tags: source.style_tags,
            scene_profile: source.scene_profile,
            schema: MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION.to_string(),
            capabilities: source.capabilities,
            built_in: false,
            enabled: true,
            prompt_hash: stable_prompt_hash(MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION, &prompt_text),
            prompt_text,
            updated_at_ms: now_ms,
        };
        save_user_prompt_pack(&state_dir, &pack)
    }

    pub fn save_global_prompt_pack(
        &self,
        prompt_pack_id: &str,
        name: impl AsRef<str>,
        style_tags: Vec<String>,
        scene_profile: SceneProfile,
        prompt_text: impl AsRef<str>,
        now_ms: i64,
    ) -> Result<PromptPack> {
        let mut pack = self
            .prompt_pack_by_id(prompt_pack_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt pack not found"))?;
        if pack.built_in || !pack.enabled {
            return Err(crate::ImporterError::internal(
                "built-in prompt packs must be forked before editing",
            ));
        }
        let name = name.as_ref().trim();
        if name.is_empty() {
            return Err(crate::ImporterError::internal(
                "prompt pack name is required",
            ));
        }
        let prompt_text = prompt_pack_content_json_from_input(prompt_text.as_ref())?;
        pack.name = name.to_string();
        pack.style_tags = style_tags
            .into_iter()
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty())
            .collect();
        pack.scene_profile = scene_profile;
        pack.version = format!("user-{now_ms}");
        pack.prompt_hash = stable_prompt_hash(MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION, &prompt_text);
        pack.prompt_text = prompt_text;
        pack.updated_at_ms = now_ms;
        save_user_prompt_pack(&self.storage_state_dir()?, &pack)
    }

    pub fn delete_global_prompt_pack(&self, prompt_pack_id: &str) -> Result<bool> {
        let pack = self
            .prompt_pack_by_id(prompt_pack_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt pack not found"))?;
        if pack.built_in {
            return Err(crate::ImporterError::internal(
                "built-in prompt packs cannot be deleted",
            ));
        }

        for project in self.list_projects()? {
            if let Some(mut settings) = self.project_evaluation_settings(&project.project_id)? {
                if settings.prompt_pack_id.as_deref() == Some(&pack.prompt_pack_id) {
                    settings.prompt_pack_id = None;
                    self.save_project_evaluation_settings(settings)?;
                }
            }
        }

        let dir = prompt_pack_dir(
            &self.storage_state_dir()?,
            &pack.distribution_folder,
            &pack.prompt_pack_id,
        );
        if dir.exists() {
            fs::remove_dir_all(dir)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn delete_global_prompt_package(&self, distribution_folder: &str) -> Result<bool> {
        let distribution_folder = normalized_distribution_folder(distribution_folder);
        if distribution_folder == "builtin" {
            return Err(crate::ImporterError::internal(
                "built-in prompt package cannot be deleted",
            ));
        }

        let pack_ids = self
            .global_prompt_packs()?
            .into_iter()
            .filter(|pack| !pack.built_in && pack.distribution_folder == distribution_folder)
            .map(|pack| pack.prompt_pack_id)
            .collect::<Vec<_>>();
        let mut deleted = false;
        for prompt_pack_id in pack_ids {
            deleted = self.delete_global_prompt_pack(&prompt_pack_id)? || deleted;
        }

        let dir = prompt_distribution_dir(&self.storage_state_dir()?, &distribution_folder);
        if dir.exists() {
            fs::remove_dir_all(dir)?;
            deleted = true;
        }
        Ok(deleted)
    }

    pub fn fork_prompt_pack_for_project(
        &self,
        project_id: &str,
        source_profile_id: &str,
        name: impl AsRef<str>,
        distribution_folder: impl AsRef<str>,
        now_ms: i64,
    ) -> Result<PromptPack> {
        ensure_service_project_is_active(&self.storage_store()?, project_id)?;
        self.fork_global_prompt_pack(source_profile_id, name, distribution_folder, now_ms)
    }

    pub fn save_prompt_pack(&self, request: SavePromptPackRequest) -> Result<PromptPack> {
        ensure_service_project_is_active(&self.storage_store()?, &request.project_id)?;
        self.save_global_prompt_pack(
            &request.prompt_pack_id,
            request.name,
            request.style_tags,
            request.scene_profile,
            request.prompt_text,
            request.now_ms,
        )
    }

    pub fn diagnostic_received_asset_groups(
        &self,
        output_dir: impl AsRef<Path>,
        source: ImportSource,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        scan_received_asset_groups(output_dir, source)
    }

    pub fn diagnostic_transfer_asset_groups_with_query(
        &self,
        state_dir: impl AsRef<Path>,
        query: AssetGroupQuery,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        let state_dir = state_dir.as_ref();
        let accounts = receiver_account_configs_from_state_dir(state_dir)?;
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
        self.storage_store()?
            .asset_group_page(project_id, query, offset, limit)
    }

    pub fn create_lan_share_session(
        &self,
        project_id: &str,
        query: AssetGroupQuery,
        title: Option<String>,
    ) -> Result<LanShareSession> {
        let store = self.storage_store()?;
        ensure_service_project_is_active(&store, project_id)?;
        store.create_lan_share_session(project_id, query, title, current_time_ms())
    }

    pub fn stop_lan_share_session(&self, share_id: &str) -> Result<Option<LanShareSession>> {
        self.storage_store()?
            .stop_lan_share_session(share_id, current_time_ms())
    }

    pub fn lan_share_asset_group_page(
        &self,
        token: &str,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        let store = self.storage_store()?;
        let session = active_lan_share_session(&store, token)?;
        store.asset_group_page(&session.project_id, session.query, offset, limit)
    }

    pub fn set_lan_share_guest_mark(
        &self,
        token: &str,
        asset_group_id: &str,
        guest_mark: Option<GuestMark>,
    ) -> Result<Option<LanShareGuestMark>> {
        let store = self.storage_store()?;
        let session = active_lan_share_session(&store, token)?;
        store.set_lan_share_guest_mark(
            &session.share_id,
            &session.project_id,
            asset_group_id,
            guest_mark,
            current_time_ms(),
        )
    }

    pub fn project_group_assets(
        &self,
        project_id: &str,
        group_id: &str,
    ) -> Result<Vec<StoredAsset>> {
        self.storage_store()?.assets_for_group(project_id, group_id)
    }

    pub fn sync_project_snapshot(
        &self,
        project_id: &str,
        snapshot: &ProjectSyncSnapshot,
    ) -> Result<ProjectSyncApplySummary> {
        let store = self.storage_store()?;
        let local_groups = store.stored_asset_groups(project_id)?;
        let mut local_assets = Vec::new();
        for group in &local_groups {
            local_assets.extend(store.assets_for_group(project_id, &group.group_id)?);
        }

        let match_summary = match_project_sync_snapshot(snapshot, &local_assets, &local_groups);
        let mut summary = ProjectSyncApplySummary {
            matched_assets: match_summary.matched_assets.len(),
            matched_groups: match_summary.matched_groups.len(),
            unresolved_records: match_summary.unmatched_assets.len()
                + match_summary.unmatched_groups.len(),
            ambiguous_records: match_summary.ambiguous_assets.len()
                + match_summary.ambiguous_groups.len(),
            ..ProjectSyncApplySummary::default()
        };

        for marks in &snapshot.user_marks {
            if marks.favorite.is_none() && marks.marked.is_none() {
                continue;
            }
            let Some(local_group_id) = match_summary.matched_groups.get(&marks.group_id) else {
                summary.unresolved_records += 1;
                continue;
            };
            store.set_asset_group_user_marks(
                project_id,
                local_group_id,
                marks.favorite,
                marks.marked,
            )?;
            summary.applied_user_marks += 1;
        }

        for evaluation in &snapshot.model_evaluations {
            let Some(local_group_id) = match_summary.matched_groups.get(&evaluation.group_id)
            else {
                summary.unresolved_records += 1;
                continue;
            };
            store.save_model_evaluation(project_sync_model_evaluation(
                project_id,
                local_group_id,
                evaluation,
            ))?;
            summary.applied_model_evaluations += 1;
        }

        for recommendation in &snapshot.selection_recommendations {
            let scope = SelectionRecommendationScope::from_str(&recommendation.scope);
            let Some(selected_group_ids) = mapped_project_sync_group_ids(
                &recommendation.selected_group_ids,
                &match_summary.matched_groups,
            ) else {
                summary.unresolved_records += 1;
                continue;
            };
            let Some(candidate_group_ids) = mapped_project_sync_group_ids(
                &recommendation.candidate_group_ids,
                &match_summary.matched_groups,
            ) else {
                summary.unresolved_records += 1;
                continue;
            };
            let Some(rejected_group_ids) = mapped_project_sync_group_ids(
                &recommendation.rejected_group_ids,
                &match_summary.matched_groups,
            ) else {
                summary.unresolved_records += 1;
                continue;
            };
            let subject_id = match scope {
                SelectionRecommendationScope::Project => project_id.to_string(),
                SelectionRecommendationScope::BurstGroup => {
                    let Some(subject_group_id) = recommendation.subject_group_id.as_ref() else {
                        summary.unresolved_records += 1;
                        continue;
                    };
                    let Some(local_group_id) = match_summary.matched_groups.get(subject_group_id)
                    else {
                        summary.unresolved_records += 1;
                        continue;
                    };
                    local_group_id.clone()
                }
            };
            store.save_selection_recommendation(SelectionRecommendation {
                recommendation_id: format!(
                    "project-sync-rec-{}",
                    stable_project_sync_key(&format!(
                        "{project_id}\t{}\t{subject_id}",
                        recommendation.recommendation_id
                    ))
                ),
                run_id: None,
                scope,
                project_id: project_id.to_string(),
                subject_id,
                selected_asset_group_ids: selected_group_ids,
                candidate_asset_group_ids: candidate_group_ids,
                rejected_asset_group_ids: rejected_group_ids,
                source: SelectionSource::Imported,
                status: SelectionRecommendationStatus::from_str(&recommendation.status),
                confidence: recommendation.confidence,
                reason: recommendation.reason.clone(),
                created_at_ms: recommendation.created_at_ms,
                updated_at_ms: recommendation.updated_at_ms,
            })?;
            summary.applied_selection_recommendations += 1;
        }

        Ok(summary)
    }

    pub fn set_asset_group_user_marks(
        &self,
        project_id: &str,
        group_id: &str,
        favorite: Option<bool>,
        marked: Option<bool>,
    ) -> Result<AssetUserMarks> {
        self.storage_store()?
            .set_asset_group_user_marks(project_id, group_id, favorite, marked)
    }

    pub fn delete_project_asset_group(&self, project_id: &str, group_id: &str) -> Result<bool> {
        let deleted_assets = self
            .storage_store()?
            .delete_asset_group(project_id, group_id)?;
        let Some(deleted_assets) = deleted_assets else {
            return Ok(false);
        };
        for asset in &deleted_assets {
            if let Some(path) = asset
                .final_location
                .as_ref()
                .and_then(StoredObjectLocation::as_local_path)
            {
                match fs::remove_file(path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => {
                        return Err(crate::ImporterError::internal(format!(
                            "delete asset file failed: {error}"
                        )));
                    }
                }
            }
        }
        Ok(true)
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
            global_assets: store.global_asset_summary()?,
            recent_failures: self.project_recent_failed_transfers(project_id, transfer_query, 5)?,
            recent_publish_failures: self.project_recent_publish_failures(project_id, 5)?,
            assets: store.asset_group_page(project_id, asset_query, offset, limit)?,
        })
    }
}

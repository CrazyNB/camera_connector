use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    current_time_ms, default_burst_grouping_profile, model_provider_ready_for_work,
    provider_configured_for_project_from_list, provider_has_required_secret,
    CameraConnectorService,
};
use crate::{
    discover_desktop_media_files, AnalysisEntityType, AnalysisJobType, DesktopScanIndexResult,
    DesktopScanPhase, DesktopScanRun, DesktopScanRunUpdate, NewAnalysisJob,
    ProjectEvaluationSettings, Result, SqliteStore,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopProjectScanResult {
    pub scan: DesktopScanRun,
    pub index: DesktopScanIndexResult,
}

impl CameraConnectorService {
    pub fn create_desktop_project_scan(
        &self,
        project_id: &str,
        root_path: impl AsRef<Path>,
    ) -> Result<DesktopScanRun> {
        self.storage_store()?
            .create_desktop_scan_run(project_id, root_path, current_time_ms())
    }

    pub fn latest_desktop_project_scan(&self, project_id: &str) -> Result<Option<DesktopScanRun>> {
        self.storage_store()?.latest_desktop_scan_run(project_id)
    }

    pub fn run_desktop_project_scan(&self, scan_id: &str) -> Result<DesktopProjectScanResult> {
        let store = self.storage_store()?;
        let scan = store.desktop_scan_run(scan_id)?.ok_or_else(|| {
            crate::ImporterError::internal(format!("desktop scan not found: {scan_id}"))
        })?;
        let result: Result<DesktopProjectScanResult> = (|| {
            store.update_desktop_scan_run(DesktopScanRunUpdate {
                scan_id,
                phase: DesktopScanPhase::Scanning,
                files_seen: 0,
                assets_indexed: 0,
                groups_updated: 0,
                error: None,
                now_ms: current_time_ms(),
            })?;
            let files = discover_desktop_media_files(&scan.root_path)?;
            store.update_desktop_scan_run(DesktopScanRunUpdate {
                scan_id,
                phase: DesktopScanPhase::Indexing,
                files_seen: files.len(),
                assets_indexed: 0,
                groups_updated: 0,
                error: None,
                now_ms: current_time_ms(),
            })?;
            let index = store.record_desktop_scan_files(scan_id, &files, current_time_ms())?;
            self.rebuild_desktop_scan_bursts(&store, &scan.project_id, &index.group_ids)?;
            self.enqueue_desktop_scan_analysis_jobs(&scan.project_id, &index.group_ids)?;
            let completed = store.update_desktop_scan_run(DesktopScanRunUpdate {
                scan_id,
                phase: DesktopScanPhase::Completed,
                files_seen: files.len(),
                assets_indexed: index.assets_indexed,
                groups_updated: index.group_ids.len(),
                error: None,
                now_ms: current_time_ms(),
            })?;
            Ok(DesktopProjectScanResult {
                scan: completed,
                index,
            })
        })();
        if let Err(error) = result.as_ref() {
            let error_message = error.to_string();
            let _ = store.update_desktop_scan_run(DesktopScanRunUpdate {
                scan_id,
                phase: DesktopScanPhase::Failed,
                files_seen: 0,
                assets_indexed: 0,
                groups_updated: 0,
                error: Some(&error_message),
                now_ms: current_time_ms(),
            });
        }
        result
    }

    fn enqueue_desktop_scan_analysis_jobs(
        &self,
        project_id: &str,
        group_ids: &[String],
    ) -> Result<usize> {
        let store = self.storage_store()?;
        let settings = store
            .project_evaluation_settings(project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            });
        let providers = self.runtime_model_providers()?;
        let provider_configured = providers.iter().any(|provider| {
            model_provider_ready_for_work(&provider.settings)
                && provider_has_required_secret(provider)
        });
        let model_jobs_enabled = settings.auto_evaluate_on_upload
            && provider_configured
            && provider_configured_for_project_from_list(&store, project_id, &providers)?;
        let mut enqueued = 0;
        let mut seen = BTreeSet::new();
        for group_id in group_ids {
            if !seen.insert(group_id.clone()) {
                continue;
            }
            let mut technical = NewAnalysisJob::new(
                project_id,
                AnalysisJobType::AssessAssetGroupTechnicalQuality,
                AnalysisEntityType::AssetGroup,
                group_id,
                &format!("desktop-scan-technical:{project_id}:{group_id}:technical-v1"),
            );
            technical.priority = 20;
            store.enqueue_analysis_job(technical)?;
            enqueued += 1;

            if model_jobs_enabled {
                let mut model = NewAnalysisJob::new(
                    project_id,
                    AnalysisJobType::EvaluateAssetGroupWithModel,
                    AnalysisEntityType::AssetGroup,
                    group_id,
                    &format!("desktop-scan-model:{project_id}:{group_id}"),
                );
                model.priority = 30;
                store.enqueue_analysis_job(model)?;
                enqueued += 1;
            }
        }
        Ok(enqueued)
    }

    fn rebuild_desktop_scan_bursts(
        &self,
        store: &SqliteStore,
        project_id: &str,
        group_ids: &[String],
    ) -> Result<()> {
        let Some(group_id) = group_ids.first() else {
            return Ok(());
        };
        let profile = default_burst_grouping_profile(store)?;
        let _ = store.detect_bursts_for_asset_group(project_id, group_id, &profile)?;
        Ok(())
    }
}

use std::collections::BTreeSet;

use crate::{
    assess_preview_sample_with_policy, AnalysisEntityType, AnalysisJob, AnalysisJobType,
    BurstGroup, EvaluationRun, EvaluationRunStatus, EvaluationRunTrigger, EvaluationRunType,
    NewAnalysisJob, PreviewSample, ProjectEvaluationSettings, ProjectStatus, Result,
    SelectionCandidateVisualInput, SelectionRecommendation, SelectionRecommendationStatus,
    SqliteStore, TechnicalAssessment,
};

use super::analysis_recommendations::{
    burst_recommendation_run, burst_selection_recommendation_from_provider_or_evaluations,
    candidate_visuals_for_asset_group_ids, evaluate_missing_model_candidates_for_burst,
    model_evaluation_for_upload, preselected_asset_group_ids,
    project_burst_recommendations_for_candidates, project_recommendation_candidate_group_ids,
    project_selection_recommendation_from_provider_or_evaluations,
    BurstSelectionRecommendationRequest, UploadModelEvaluationRequest,
};
use super::{
    current_time_ms, default_burst_grouping_profile, evaluation_run_id,
    evaluator_version_for_runtime_provider, model_provider_ready_for_work,
    prompt_snapshot_for_settings, provider_configured_for_project_from_list,
    provider_has_required_secret, recommend_job_dedupe_key,
    runtime_model_provider_for_project_from_list, runtime_model_providers_from_config,
    technical_assessment_policy_for_settings, AnalysisDrainSummary, AssetGroupModelEvaluationInput,
    CameraConnectorService, RuntimeModelProvider,
};

impl CameraConnectorService {
    pub fn drain_analysis_jobs(&self, limit: usize) -> Result<AnalysisDrainSummary> {
        let provider_configured = self.provider_configured_for_model_work()?;
        self.drain_analysis_jobs_with_provider_configured(limit, provider_configured)
    }

    pub fn drain_analysis_jobs_with_provider_configured(
        &self,
        limit: usize,
        provider_configured: bool,
    ) -> Result<AnalysisDrainSummary> {
        let store = self.storage_store()?;
        let providers = self.runtime_model_providers()?;
        let now = current_time_ms();
        let jobs = store.claim_analysis_jobs(now, limit)?;
        let claimed_count = jobs.len();
        let mut completed_count = 0;
        let mut failed_count = 0;

        for job in jobs {
            match run_analysis_job(&store, &job, provider_configured, &providers) {
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

    pub fn enqueue_model_evaluation_for_asset_groups(
        &self,
        project_id: &str,
        asset_group_ids: &[String],
    ) -> Result<usize> {
        let store = self.storage_store()?;
        let provider = self
            .runtime_model_provider_for_project(&store, project_id)?
            .ok_or_else(|| crate::ImporterError::internal("model provider is not configured"))?;
        if !model_provider_ready_for_work(&provider.settings)
            || !provider_has_required_secret(&provider)
        {
            return Err(crate::ImporterError::internal(
                "model provider is not configured",
            ));
        }
        let evaluator_version = evaluator_version_for_runtime_provider(Some(&provider));
        let mut enqueued_count = 0;
        let mut seen = BTreeSet::new();
        for asset_group_id in asset_group_ids {
            if !seen.insert(asset_group_id.clone()) {
                continue;
            }
            let owner_project_id = store
                .project_id_for_asset_group(asset_group_id)?
                .ok_or_else(|| crate::ImporterError::internal("asset group not found"))?;
            if owner_project_id != project_id {
                return Err(crate::ImporterError::internal(
                    "asset group does not belong to project",
                ));
            }
            let mut job = NewAnalysisJob::new(
                project_id,
                AnalysisJobType::EvaluateAssetGroupWithModel,
                AnalysisEntityType::AssetGroup,
                asset_group_id,
                &format!("manual-model-eval:{project_id}:{asset_group_id}:{evaluator_version}"),
            );
            job.priority = 40;
            store.enqueue_analysis_job(job)?;
            enqueued_count += 1;
        }
        Ok(enqueued_count)
    }

    pub fn evaluate_asset_groups_with_model_inputs(
        &self,
        project_id: &str,
        inputs: &[AssetGroupModelEvaluationInput],
    ) -> Result<usize> {
        let store = self.storage_store()?;
        let settings = store
            .project_evaluation_settings(project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            });
        let provider = self
            .runtime_model_provider_for_project(&store, project_id)?
            .ok_or_else(|| crate::ImporterError::internal("model provider is not configured"))?;
        if !model_provider_ready_for_work(&provider.settings)
            || !provider_has_required_secret(&provider)
        {
            return Err(crate::ImporterError::internal(
                "model provider is not configured",
            ));
        }

        let mut saved_count = 0;
        let mut seen = BTreeSet::new();
        for input in inputs {
            if !seen.insert(input.asset_group_id.clone()) {
                continue;
            }
            let owner_project_id = store
                .project_id_for_asset_group(&input.asset_group_id)?
                .ok_or_else(|| crate::ImporterError::internal("asset group not found"))?;
            if owner_project_id != project_id {
                return Err(crate::ImporterError::internal(
                    "asset group does not belong to project",
                ));
            }

            let now = current_time_ms();
            let assessment = assess_preview_sample_with_policy(
                &input.asset_group_id,
                input.preview_sample.clone(),
                "technical-v1",
                now,
                technical_assessment_policy_for_settings(&settings),
            );
            let saved_assessment = store.save_technical_assessment(assessment)?;
            let preview_image_data_url = input
                .preview_image_data_url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let evaluation = model_evaluation_for_upload(UploadModelEvaluationRequest {
                store: &store,
                project_id,
                asset_group_id: &input.asset_group_id,
                assessment: &saved_assessment,
                preview_image_data_url,
                preview_sample: Some(&input.preview_sample),
                provider: Some(provider.clone()),
                trigger: EvaluationRunTrigger::Manual,
                now_ms: now,
            })?;
            store.save_model_evaluation(evaluation)?;
            saved_count += 1;
        }
        Ok(saved_count)
    }

    pub fn assess_asset_group_preview(
        &self,
        asset_group_id: &str,
        sample: PreviewSample,
        assessor_version: &str,
    ) -> Result<TechnicalAssessment> {
        let provider_configured = self.provider_configured_for_model_work()?;
        self.assess_asset_group_preview_with_provider_configured(
            asset_group_id,
            sample,
            assessor_version,
            provider_configured,
        )
    }

    pub fn assess_asset_group_preview_with_provider_configured(
        &self,
        asset_group_id: &str,
        sample: PreviewSample,
        assessor_version: &str,
        provider_configured: bool,
    ) -> Result<TechnicalAssessment> {
        self.assess_asset_group_preview_with_image_data_url_and_provider_configured(
            asset_group_id,
            sample,
            None,
            assessor_version,
            provider_configured,
        )
    }

    pub fn assess_asset_group_preview_with_image_data_url_and_provider_configured(
        &self,
        asset_group_id: &str,
        sample: PreviewSample,
        preview_image_data_url: Option<&str>,
        assessor_version: &str,
        provider_configured: bool,
    ) -> Result<TechnicalAssessment> {
        let now = current_time_ms();
        let store = self.storage_store()?;
        let project_id = store
            .project_id_for_asset_group(asset_group_id)?
            .ok_or_else(|| crate::ImporterError::internal("asset group not found"))?;
        let settings = store
            .project_evaluation_settings(&project_id)?
            .unwrap_or_else(|| ProjectEvaluationSettings::default_for_project(&project_id, now));
        let sample_for_model = sample.clone();
        let assessment = assess_preview_sample_with_policy(
            asset_group_id,
            sample,
            assessor_version,
            now,
            technical_assessment_policy_for_settings(&settings),
        );
        let saved_assessment = store.save_technical_assessment(assessment)?;
        let providers = self.runtime_model_providers()?;
        if self.should_run_upload_model_evaluation(
            &store,
            &project_id,
            provider_configured,
            &providers,
        )? {
            let provider =
                runtime_model_provider_for_project_from_list(&store, &project_id, &providers)?;
            let evaluation = model_evaluation_for_upload(UploadModelEvaluationRequest {
                store: &store,
                project_id: &project_id,
                asset_group_id,
                assessment: &saved_assessment,
                preview_image_data_url,
                preview_sample: Some(&sample_for_model),
                provider,
                trigger: EvaluationRunTrigger::Upload,
                now_ms: now,
            })?;
            store.save_model_evaluation(evaluation)?;
        }
        if let Some(burst) = store.burst_group_for_asset_group(&saved_assessment.asset_group_id)? {
            let profile = default_burst_grouping_profile(&store)?;
            let refined_bursts = store.refine_burst_group_by_visual_similarity(
                &burst.burst_group_id,
                &profile,
                assessor_version,
            )?;
            let bursts = if refined_bursts.is_empty() {
                Vec::new()
            } else {
                refined_bursts
            };
            let settings = store
                .project_evaluation_settings(&project_id)?
                .unwrap_or_else(|| {
                    ProjectEvaluationSettings::default_for_project(&project_id, now)
                });
            if settings.auto_burst_recommendation_enabled {
                for burst in bursts {
                    let dedupe_key = recommend_job_dedupe_key(&burst.burst_group_id);
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
            }
        }
        Ok(saved_assessment)
    }

    pub fn recommend_burst_group_from_model(
        &self,
        burst_group_id: &str,
    ) -> Result<SelectionRecommendation> {
        self.recommend_burst_group_from_model_with_candidate_visuals(burst_group_id, &[])
    }

    pub fn recommend_burst_group_from_model_with_candidate_visuals(
        &self,
        burst_group_id: &str,
        candidate_visuals: &[SelectionCandidateVisualInput],
    ) -> Result<SelectionRecommendation> {
        let store = self.storage_store()?;
        let now_ms = current_time_ms();
        let burst = store
            .burst_group(burst_group_id)?
            .ok_or_else(|| crate::ImporterError::internal("burst group not found"))?;
        let provider = self.runtime_model_provider_for_project(&store, &burst.project_id)?;
        let evaluations = store.model_evaluations_for_asset_groups(
            &burst.member_group_ids,
            evaluator_version_for_runtime_provider(provider.as_ref()),
        )?;
        let assessments = store
            .technical_assessments_for_asset_groups(&burst.member_group_ids, "technical-v1")?;
        let run = burst_recommendation_run(
            &store,
            &burst.project_id,
            burst_group_id,
            EvaluationRunTrigger::Manual,
            provider.as_ref().map(|provider| provider.settings.clone()),
            now_ms,
        )?;
        let settings = store
            .project_evaluation_settings(&burst.project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(&burst.project_id, now_ms)
            });
        let prompt_snapshot = prompt_snapshot_for_settings(&store, &settings)?;
        let prompt_content = prompt_snapshot
            .as_ref()
            .map(|snapshot| snapshot.prompt_content.clone())
            .unwrap_or_default();
        let mut recommendation = burst_selection_recommendation_from_provider_or_evaluations(
            BurstSelectionRecommendationRequest {
                project_id: &burst.project_id,
                burst_group_id,
                evaluations: &evaluations,
                assessments: &assessments,
                provider: provider.as_ref(),
                candidate_visuals,
                prompt_content: &prompt_content,
                now_ms,
            },
        )?;
        if recommendation.status == SelectionRecommendationStatus::Pending
            && provider.is_some()
            && settings.auto_evaluate_on_upload
        {
            let preselected_ids = preselected_asset_group_ids(&recommendation);
            evaluate_missing_model_candidates_for_burst(
                &store,
                &burst.project_id,
                &preselected_ids,
                candidate_visuals,
                provider.as_ref(),
            )?;
            let final_evaluations = store.model_evaluations_for_asset_groups(
                &preselected_ids,
                evaluator_version_for_runtime_provider(provider.as_ref()),
            )?;
            if !final_evaluations.is_empty() {
                let final_candidate_ids = final_evaluations
                    .iter()
                    .map(|evaluation| evaluation.asset_group_id.clone())
                    .collect::<Vec<_>>();
                let final_candidate_visuals =
                    candidate_visuals_for_asset_group_ids(candidate_visuals, &final_candidate_ids);
                let final_assessments = store
                    .technical_assessments_for_asset_groups(&final_candidate_ids, "technical-v1")?;
                let final_now_ms = current_time_ms();
                let final_run = burst_recommendation_run(
                    &store,
                    &burst.project_id,
                    burst_group_id,
                    EvaluationRunTrigger::Manual,
                    provider.as_ref().map(|provider| provider.settings.clone()),
                    final_now_ms,
                )?;
                let mut final_recommendation =
                    burst_selection_recommendation_from_provider_or_evaluations(
                        BurstSelectionRecommendationRequest {
                            project_id: &burst.project_id,
                            burst_group_id,
                            evaluations: &final_evaluations,
                            assessments: &final_assessments,
                            provider: provider.as_ref(),
                            candidate_visuals: &final_candidate_visuals,
                            prompt_content: &prompt_content,
                            now_ms: final_now_ms,
                        },
                    )?;
                final_recommendation.run_id = Some(final_run.run_id.clone());
                store.save_evaluation_run(final_run)?;
                return store.save_selection_recommendation(final_recommendation);
            }
        }
        recommendation.run_id = Some(run.run_id.clone());
        store.save_evaluation_run(run)?;
        store.save_selection_recommendation(recommendation)
    }

    pub fn generate_project_recommendation(
        &self,
        project_id: &str,
        now_ms: i64,
    ) -> Result<SelectionRecommendation> {
        self.generate_project_recommendation_with_candidate_visuals(project_id, &[], now_ms)
    }

    pub fn generate_project_recommendation_with_candidate_visuals(
        &self,
        project_id: &str,
        candidate_visuals: &[SelectionCandidateVisualInput],
        now_ms: i64,
    ) -> Result<SelectionRecommendation> {
        let store = self.storage_store()?;
        let project = store
            .list_projects()?
            .into_iter()
            .find(|project| project.project_id == project_id)
            .ok_or_else(|| crate::ImporterError::internal("project not found"))?;
        if project.status == ProjectStatus::Archived {
            return Err(crate::ImporterError::internal("project is archived"));
        }
        let provider = self
            .runtime_model_provider_for_project(&store, project_id)?
            .ok_or_else(|| {
                crate::ImporterError::internal("model provider settings not configured")
            })?;
        if !model_provider_ready_for_work(&provider.settings)
            || !provider_has_required_secret(&provider)
        {
            return Err(crate::ImporterError::internal(
                "model provider is not configured",
            ));
        }
        let settings = store
            .project_evaluation_settings(project_id)?
            .unwrap_or_else(|| ProjectEvaluationSettings::default_for_project(project_id, now_ms));
        let prompt_snapshot = prompt_snapshot_for_settings(&store, &settings)?;
        let prompt_content = prompt_snapshot
            .as_ref()
            .map(|snapshot| snapshot.prompt_content.clone())
            .unwrap_or_default();
        let run_id = evaluation_run_id(
            project_id,
            EvaluationRunType::ProjectRecommendation,
            project_id,
            now_ms,
        );
        let run = EvaluationRun {
            run_id: run_id.clone(),
            project_id: project_id.to_string(),
            run_type: EvaluationRunType::ProjectRecommendation,
            trigger: EvaluationRunTrigger::Manual,
            status: EvaluationRunStatus::Ready,
            provider_kind: provider.settings.provider_kind,
            provider_model: provider.settings.default_model.clone(),
            prompt_pack_id: prompt_snapshot
                .as_ref()
                .map(|snapshot| snapshot.prompt_pack_id.clone()),
            prompt_pack_version: prompt_snapshot
                .as_ref()
                .map(|snapshot| snapshot.prompt_pack_version.clone()),
            prompt_hash: prompt_snapshot
                .as_ref()
                .map(|snapshot| snapshot.prompt_hash.clone()),
            settings_snapshot_json: serde_json::to_string(&settings)
                .map_err(|error| crate::ImporterError::internal(error.to_string()))?,
            error_message: None,
            started_at_ms: Some(now_ms),
            completed_at_ms: Some(now_ms),
            created_at_ms: now_ms,
        };
        store.save_evaluation_run(run)?;
        let group_ids = project_recommendation_candidate_group_ids(&store, project_id)?;
        let evaluations = store.model_evaluations_for_asset_groups(
            &group_ids,
            evaluator_version_for_runtime_provider(Some(&provider)),
        )?;
        let burst_recommendations =
            project_burst_recommendations_for_candidates(&store, project_id, &group_ids)?;
        let mut recommendation = project_selection_recommendation_from_provider_or_evaluations(
            project_id,
            &evaluations,
            &burst_recommendations,
            Some(&provider),
            candidate_visuals,
            &prompt_content,
            now_ms,
        )?;
        recommendation.run_id = Some(run_id);
        store.save_selection_recommendation(recommendation)
    }

    pub fn latest_project_recommendation_run_status(
        &self,
        project_id: &str,
    ) -> Result<Option<EvaluationRun>> {
        self.storage_store()?
            .latest_evaluation_run(project_id, EvaluationRunType::ProjectRecommendation)
    }

    pub fn split_burst_member(
        &self,
        burst_group_id: &str,
        member_group_id: &str,
    ) -> Result<Option<BurstGroup>> {
        self.storage_store()?
            .split_burst_member(burst_group_id, member_group_id)
    }

    pub fn create_manual_burst_group(
        &self,
        project_id: &str,
        member_group_ids: &[String],
    ) -> Result<Option<BurstGroup>> {
        self.storage_store()?
            .create_manual_burst_group(project_id, member_group_ids)
    }
    fn provider_configured_for_model_work(&self) -> Result<bool> {
        Ok(self.runtime_model_providers()?.iter().any(|provider| {
            model_provider_ready_for_work(&provider.settings)
                && provider_has_required_secret(provider)
        }))
    }

    pub(super) fn runtime_model_provider(&self) -> Result<Option<RuntimeModelProvider>> {
        Ok(self.runtime_model_providers()?.into_iter().next())
    }

    pub(super) fn runtime_model_providers(&self) -> Result<Vec<RuntimeModelProvider>> {
        Ok(runtime_model_providers_from_config(self.load_config()?))
    }

    fn runtime_model_provider_for_project(
        &self,
        store: &SqliteStore,
        project_id: &str,
    ) -> Result<Option<RuntimeModelProvider>> {
        let providers = self.runtime_model_providers()?;
        runtime_model_provider_for_project_from_list(store, project_id, &providers)
    }

    fn should_run_upload_model_evaluation(
        &self,
        store: &SqliteStore,
        project_id: &str,
        provider_configured: bool,
        providers: &[RuntimeModelProvider],
    ) -> Result<bool> {
        if !provider_configured {
            return Ok(false);
        }
        let settings = store
            .project_evaluation_settings(project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            });
        Ok(settings.auto_evaluate_on_upload
            && provider_configured_for_project_from_list(store, project_id, providers)?)
    }
}

fn run_analysis_job(
    store: &SqliteStore,
    job: &AnalysisJob,
    provider_configured: bool,
    providers: &[RuntimeModelProvider],
) -> Result<()> {
    match (job.job_type, job.entity_type) {
        (AnalysisJobType::DetectBurstForAssetGroup, AnalysisEntityType::AssetGroup) => {
            let profile = default_burst_grouping_profile(store)?;
            let _ =
                store.detect_bursts_for_asset_group(&job.project_id, &job.entity_id, &profile)?;
            Ok(())
        }
        (AnalysisJobType::AssessAssetGroupTechnicalQuality, AnalysisEntityType::AssetGroup) => {
            Ok(())
        }
        (AnalysisJobType::AssessPortraitSubject, AnalysisEntityType::AssetGroup) => {
            // Core owns the scheduling and storage contract. Android/imported clients provide
            // detector output through save_subject_assessment; no detector is bundled here.
            Ok(())
        }
        (AnalysisJobType::EvaluateAssetGroupWithModel, AnalysisEntityType::AssetGroup) => {
            let project_provider_configured = provider_configured
                && provider_configured_for_project_from_list(store, &job.project_id, providers)?;
            if !project_provider_configured {
                return Ok(());
            }
            let assessments = store.technical_assessments_for_asset_groups(
                std::slice::from_ref(&job.entity_id),
                "technical-v1",
            )?;
            let assessment = assessments
                .first()
                .ok_or_else(|| crate::ImporterError::internal("technical assessment not found"))?;
            let provider =
                runtime_model_provider_for_project_from_list(store, &job.project_id, providers)?;
            let evaluation = model_evaluation_for_upload(UploadModelEvaluationRequest {
                store,
                project_id: &job.project_id,
                asset_group_id: &job.entity_id,
                assessment,
                preview_image_data_url: None,
                preview_sample: None,
                provider: provider.clone(),
                trigger: EvaluationRunTrigger::Manual,
                now_ms: current_time_ms(),
            })?;
            store.save_model_evaluation(evaluation)?;
            Ok(())
        }
        (AnalysisJobType::RecommendBurstGroup, AnalysisEntityType::BurstGroup) => {
            let settings = store
                .project_evaluation_settings(&job.project_id)?
                .unwrap_or_else(|| {
                    ProjectEvaluationSettings::default_for_project(
                        &job.project_id,
                        current_time_ms(),
                    )
                });
            if !settings.auto_burst_recommendation_enabled {
                return Ok(());
            }
            let burst = store
                .burst_group(&job.entity_id)?
                .ok_or_else(|| crate::ImporterError::internal("burst group not found"))?;
            let provider =
                runtime_model_provider_for_project_from_list(store, &burst.project_id, providers)?;
            let evaluations = store.model_evaluations_for_asset_groups(
                &burst.member_group_ids,
                evaluator_version_for_runtime_provider(provider.as_ref()),
            )?;
            if !evaluations.is_empty() {
                let now_ms = current_time_ms();
                let run = burst_recommendation_run(
                    store,
                    &burst.project_id,
                    &burst.burst_group_id,
                    EvaluationRunTrigger::BurstStable,
                    provider.as_ref().map(|provider| provider.settings.clone()),
                    now_ms,
                )?;
                let assessments = store.technical_assessments_for_asset_groups(
                    &burst.member_group_ids,
                    "technical-v1",
                )?;
                let settings = store
                    .project_evaluation_settings(&burst.project_id)?
                    .unwrap_or_else(|| {
                        ProjectEvaluationSettings::default_for_project(&burst.project_id, now_ms)
                    });
                let prompt_snapshot = prompt_snapshot_for_settings(store, &settings)?;
                let prompt_content = prompt_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.prompt_content.clone())
                    .unwrap_or_default();
                let mut scoped_recommendation =
                    burst_selection_recommendation_from_provider_or_evaluations(
                        BurstSelectionRecommendationRequest {
                            project_id: &burst.project_id,
                            burst_group_id: &burst.burst_group_id,
                            evaluations: &evaluations,
                            assessments: &assessments,
                            provider: provider.as_ref(),
                            candidate_visuals: &[],
                            prompt_content: &prompt_content,
                            now_ms,
                        },
                    )?;
                scoped_recommendation.run_id = Some(run.run_id.clone());
                store.save_evaluation_run(run)?;
                store.save_selection_recommendation(scoped_recommendation)?;
            }
            Ok(())
        }
        (AnalysisJobType::GenerateProjectRecommendation, AnalysisEntityType::Project) => {
            // Manual-only: stale/background project recommendation jobs are completed as ignored work so
            // upload drains cannot create project recommendations or retry them forever.
            Ok(())
        }
        _ => Ok(()),
    }
}

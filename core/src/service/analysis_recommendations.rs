use std::collections::{BTreeMap, BTreeSet};

use crate::{
    assess_preview_sample, evaluate_asset_group_with_model_provider,
    evaluate_asset_group_with_stub, recommend_burst_group_from_model_evaluations,
    recommend_project_model_selections, recommend_selection_with_model_provider, EvaluationRun,
    EvaluationRunStatus, EvaluationRunTrigger, EvaluationRunType, ModelProviderEvaluationRequest,
    ModelProviderKind, ModelProviderSelectionRequest, ModelProviderSettings, PreviewSample,
    ProjectEvaluationSettings, PromptPackContent, Result, SelectionCandidateVisualInput,
    SelectionRecommendation, SelectionRecommendationScope, SelectionRecommendationStatus,
    SqliteStore, TechnicalAssessment,
};

use super::{
    current_time_ms, evaluation_run_id, evaluator_version_for_runtime_provider,
    model_evaluation_skipped, model_provider_ready_for_work, prompt_snapshot_for_settings,
    provider_has_required_secret, RuntimeModelProvider,
};

pub(super) fn project_recommendation_candidate_group_ids(
    store: &SqliteStore,
    project_id: &str,
) -> Result<Vec<String>> {
    let group_ids = store
        .stored_asset_groups(project_id)?
        .into_iter()
        .map(|group| group.group_id)
        .collect::<Vec<_>>();
    let mut candidate_ids = Vec::new();
    let mut burst_group_ids = BTreeSet::new();
    for group_id in &group_ids {
        if let Some(burst) = store.burst_group_for_asset_group(group_id)? {
            burst_group_ids.insert(burst.burst_group_id);
        } else {
            candidate_ids.push(group_id.clone());
        }
    }
    for burst_group_id in burst_group_ids {
        let Some(recommendation) = store.latest_selection_recommendation(
            project_id,
            SelectionRecommendationScope::BurstGroup,
            &burst_group_id,
        )?
        else {
            continue;
        };
        if recommendation.status == SelectionRecommendationStatus::Ready {
            candidate_ids.extend(recommendation.selected_asset_group_ids);
        }
    }
    candidate_ids.sort();
    candidate_ids.dedup();
    Ok(candidate_ids)
}

pub(super) fn project_burst_recommendations_for_candidates(
    store: &SqliteStore,
    project_id: &str,
    group_ids: &[String],
) -> Result<Vec<SelectionRecommendation>> {
    let mut burst_group_ids = BTreeSet::new();
    for group_id in group_ids {
        if let Some(burst) = store.burst_group_for_asset_group(group_id)? {
            burst_group_ids.insert(burst.burst_group_id);
        }
    }
    let mut burst_recommendations = Vec::new();
    for burst_group_id in burst_group_ids {
        if let Some(recommendation) = store.latest_selection_recommendation(
            project_id,
            SelectionRecommendationScope::BurstGroup,
            &burst_group_id,
        )? {
            burst_recommendations.push(recommendation);
        }
    }
    Ok(burst_recommendations)
}

pub(super) fn preselected_asset_group_ids(recommendation: &SelectionRecommendation) -> Vec<String> {
    let mut seen = BTreeSet::new();
    recommendation
        .selected_asset_group_ids
        .iter()
        .chain(recommendation.candidate_asset_group_ids.iter())
        .filter(|asset_group_id| seen.insert((*asset_group_id).clone()))
        .cloned()
        .collect()
}

pub(super) fn candidate_visuals_for_asset_group_ids(
    candidate_visuals: &[SelectionCandidateVisualInput],
    asset_group_ids: &[String],
) -> Vec<SelectionCandidateVisualInput> {
    let wanted_ids = asset_group_ids.iter().collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    candidate_visuals
        .iter()
        .filter(|visual| {
            wanted_ids.contains(&visual.asset_group_id)
                && !visual.image_data_url.trim().is_empty()
                && seen.insert(visual.asset_group_id.clone())
        })
        .cloned()
        .collect()
}

pub(super) fn evaluate_missing_model_candidates_for_burst(
    store: &SqliteStore,
    project_id: &str,
    candidate_ids: &[String],
    candidate_visuals: &[SelectionCandidateVisualInput],
    provider: Option<&RuntimeModelProvider>,
) -> Result<usize> {
    let Some(provider) = provider.filter(|provider| {
        matches!(
            provider.settings.provider_kind,
            ModelProviderKind::OpenAi | ModelProviderKind::Custom
        ) && model_provider_ready_for_work(&provider.settings)
            && provider_has_required_secret(provider)
    }) else {
        return Ok(0);
    };
    if candidate_ids.is_empty() || candidate_visuals.is_empty() {
        return Ok(0);
    }

    let wanted_ids = candidate_ids.iter().collect::<BTreeSet<_>>();
    let visual_by_group = candidate_visuals
        .iter()
        .filter(|visual| {
            wanted_ids.contains(&visual.asset_group_id) && !visual.image_data_url.trim().is_empty()
        })
        .map(|visual| {
            (
                visual.asset_group_id.as_str(),
                visual.image_data_url.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if visual_by_group.is_empty() {
        return Ok(0);
    }

    let evaluator_version = evaluator_version_for_runtime_provider(Some(provider));
    let existing_evaluations =
        store.model_evaluations_for_asset_groups(candidate_ids, evaluator_version)?;
    let mut evaluated_ids = existing_evaluations
        .into_iter()
        .map(|evaluation| evaluation.asset_group_id)
        .collect::<BTreeSet<_>>();
    let mut assessment_by_group = store
        .technical_assessments_for_asset_groups(candidate_ids, "technical-v1")?
        .into_iter()
        .map(|assessment| (assessment.asset_group_id.clone(), assessment))
        .collect::<BTreeMap<_, _>>();

    let mut saved_count = 0;
    for asset_group_id in candidate_ids {
        if evaluated_ids.contains(asset_group_id) {
            continue;
        }
        let Some(image_data_url) = visual_by_group.get(asset_group_id.as_str()).copied() else {
            continue;
        };
        let owner_project_id = store
            .project_id_for_asset_group(asset_group_id)?
            .ok_or_else(|| crate::ImporterError::internal("asset group not found"))?;
        if owner_project_id != project_id {
            return Err(crate::ImporterError::internal(
                "asset group does not belong to project",
            ));
        }

        let now_ms = current_time_ms();
        let assessment = match assessment_by_group.get(asset_group_id) {
            Some(assessment) => assessment.clone(),
            None => {
                let fallback_assessment = assess_preview_sample(
                    asset_group_id,
                    PreviewSample {
                        width: 0,
                        height: 0,
                        luma: Vec::new(),
                        red: None,
                        green: None,
                        blue: None,
                        preview_source: Some("selection-candidate-visual".to_string()),
                    },
                    "technical-v1",
                    now_ms,
                );
                let saved_assessment = store.save_technical_assessment(fallback_assessment)?;
                assessment_by_group.insert(asset_group_id.clone(), saved_assessment.clone());
                saved_assessment
            }
        };
        let evaluation = model_evaluation_for_upload(UploadModelEvaluationRequest {
            store,
            project_id,
            asset_group_id,
            assessment: &assessment,
            preview_image_data_url: Some(image_data_url),
            preview_sample: None,
            provider: Some(provider.clone()),
            trigger: EvaluationRunTrigger::Manual,
            now_ms,
        })?;
        store.save_model_evaluation(evaluation)?;
        evaluated_ids.insert(asset_group_id.clone());
        saved_count += 1;
    }
    Ok(saved_count)
}

pub(super) struct BurstSelectionRecommendationRequest<'a> {
    pub(super) project_id: &'a str,
    pub(super) burst_group_id: &'a str,
    pub(super) evaluations: &'a [crate::ModelEvaluation],
    pub(super) assessments: &'a [TechnicalAssessment],
    pub(super) provider: Option<&'a RuntimeModelProvider>,
    pub(super) candidate_visuals: &'a [SelectionCandidateVisualInput],
    pub(super) prompt_content: &'a PromptPackContent,
    pub(super) now_ms: i64,
}

pub(super) fn burst_selection_recommendation_from_provider_or_evaluations(
    request: BurstSelectionRecommendationRequest<'_>,
) -> Result<SelectionRecommendation> {
    let BurstSelectionRecommendationRequest {
        project_id,
        burst_group_id,
        evaluations,
        assessments,
        provider,
        candidate_visuals,
        prompt_content,
        now_ms,
    } = request;
    if let Some(provider) = provider.filter(|provider| {
        matches!(
            provider.settings.provider_kind,
            ModelProviderKind::OpenAi | ModelProviderKind::Custom
        ) && provider_has_required_secret(provider)
    }) {
        return recommend_selection_with_model_provider(ModelProviderSelectionRequest {
            project_id,
            scope: SelectionRecommendationScope::BurstGroup,
            subject_id: burst_group_id,
            evaluations,
            assessments,
            candidate_visuals,
            now_ms,
            provider: &provider.settings,
            api_key: provider.api_key.as_deref().unwrap_or_default(),
            prompt_content,
        });
    }
    Ok(recommend_burst_group_from_model_evaluations(
        project_id,
        burst_group_id,
        evaluations,
        assessments,
        now_ms,
    ))
}

pub(super) fn project_selection_recommendation_from_provider_or_evaluations(
    project_id: &str,
    evaluations: &[crate::ModelEvaluation],
    burst_recommendations: &[SelectionRecommendation],
    provider: Option<&RuntimeModelProvider>,
    candidate_visuals: &[SelectionCandidateVisualInput],
    prompt_content: &PromptPackContent,
    now_ms: i64,
) -> Result<SelectionRecommendation> {
    if let Some(provider) = provider.filter(|provider| {
        matches!(
            provider.settings.provider_kind,
            ModelProviderKind::OpenAi | ModelProviderKind::Custom
        ) && provider_has_required_secret(provider)
    }) {
        return recommend_selection_with_model_provider(ModelProviderSelectionRequest {
            project_id,
            scope: SelectionRecommendationScope::Project,
            subject_id: project_id,
            evaluations,
            assessments: &[],
            candidate_visuals,
            now_ms,
            provider: &provider.settings,
            api_key: provider.api_key.as_deref().unwrap_or_default(),
            prompt_content,
        });
    }
    Ok(recommend_project_model_selections(
        project_id,
        evaluations,
        burst_recommendations,
        now_ms,
    ))
}

pub(super) struct UploadModelEvaluationRequest<'a> {
    pub(super) store: &'a SqliteStore,
    pub(super) project_id: &'a str,
    pub(super) asset_group_id: &'a str,
    pub(super) assessment: &'a TechnicalAssessment,
    pub(super) preview_image_data_url: Option<&'a str>,
    pub(super) preview_sample: Option<&'a PreviewSample>,
    pub(super) provider: Option<RuntimeModelProvider>,
    pub(super) trigger: EvaluationRunTrigger,
    pub(super) now_ms: i64,
}

pub(super) fn model_evaluation_for_upload(
    request: UploadModelEvaluationRequest<'_>,
) -> Result<crate::ModelEvaluation> {
    let UploadModelEvaluationRequest {
        store,
        project_id,
        asset_group_id,
        assessment,
        preview_image_data_url,
        preview_sample,
        provider,
        trigger,
        now_ms,
    } = request;
    let settings = store
        .project_evaluation_settings(project_id)?
        .unwrap_or_else(|| ProjectEvaluationSettings::default_for_project(project_id, now_ms));
    let prompt_snapshot = prompt_snapshot_for_settings(store, &settings)?;
    let prompt_content = prompt_snapshot
        .as_ref()
        .map(|snapshot| snapshot.prompt_content.clone())
        .unwrap_or_default();
    let mut evaluation = match provider.as_ref() {
        Some(provider)
            if matches!(
                provider.settings.provider_kind,
                ModelProviderKind::OpenAi | ModelProviderKind::Custom
            ) && provider
                .api_key
                .as_deref()
                .map(str::trim)
                .is_some_and(|key| !key.is_empty()) =>
        {
            evaluate_asset_group_with_model_provider(ModelProviderEvaluationRequest {
                project_id,
                asset_group_id,
                assessment,
                preview_image_data_url,
                preview_sample,
                now_ms,
                provider: &provider.settings,
                api_key: provider.api_key.as_deref().unwrap_or_default(),
                prompt_content: &prompt_content,
            })?
        }
        Some(provider) if provider.settings.provider_kind == ModelProviderKind::Imported => {
            evaluate_asset_group_with_stub(project_id, asset_group_id, assessment, now_ms)
        }
        _ => model_evaluation_skipped(
            project_id,
            asset_group_id,
            provider
                .as_ref()
                .map(|provider| provider.settings.default_model.as_str())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("model-unconfigured"),
            "model provider API key is not configured",
            now_ms,
        ),
    };
    let run_id = evaluation_run_id(
        project_id,
        EvaluationRunType::AssetEvaluation,
        asset_group_id,
        now_ms,
    );
    let run = EvaluationRun {
        run_id: run_id.clone(),
        project_id: project_id.to_string(),
        run_type: EvaluationRunType::AssetEvaluation,
        trigger,
        status: EvaluationRunStatus::Ready,
        provider_kind: provider
            .as_ref()
            .map(|provider| provider.settings.provider_kind)
            .unwrap_or(ModelProviderKind::None),
        provider_model: provider
            .map(|provider| provider.settings.default_model)
            .unwrap_or_else(|| "model-stub-v1".to_string()),
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
    evaluation.run_id = run_id;
    if let Some(snapshot) = prompt_snapshot {
        evaluation.prompt_pack_id = Some(snapshot.prompt_pack_id);
        evaluation.prompt_pack_version = Some(snapshot.prompt_pack_version);
        evaluation.prompt_hash = Some(snapshot.prompt_hash);
    }
    Ok(evaluation)
}

pub(super) fn burst_recommendation_run(
    store: &SqliteStore,
    project_id: &str,
    burst_group_id: &str,
    trigger: EvaluationRunTrigger,
    provider: Option<ModelProviderSettings>,
    now_ms: i64,
) -> Result<EvaluationRun> {
    let settings = store
        .project_evaluation_settings(project_id)?
        .unwrap_or_else(|| ProjectEvaluationSettings::default_for_project(project_id, now_ms));
    let prompt_snapshot = prompt_snapshot_for_settings(store, &settings)?;
    Ok(EvaluationRun {
        run_id: evaluation_run_id(
            project_id,
            EvaluationRunType::BurstRecommendation,
            burst_group_id,
            now_ms,
        ),
        project_id: project_id.to_string(),
        run_type: EvaluationRunType::BurstRecommendation,
        trigger,
        status: EvaluationRunStatus::Ready,
        provider_kind: provider
            .as_ref()
            .map(|settings| settings.provider_kind)
            .unwrap_or(ModelProviderKind::None),
        provider_model: provider
            .map(|settings| settings.default_model)
            .unwrap_or_else(|| "model-stub-v1".to_string()),
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
    })
}

use crate::{
    CvPolicy, EvaluationRunType, ModelEvaluation, ModelEvaluationStatus, ModelEvaluationTier,
    ModelEvaluatorKind, ModelProviderKind, ModelProviderSettings, ProjectEvaluationSettings,
    Result, SqliteStore, TechnicalAssessmentPolicy,
};

use super::{
    evaluation_run_id, normalized_model_provider_settings_id, stable_id_fragment,
    RuntimeModelProvider,
};

pub(super) fn model_provider_ready_for_work(settings: &ModelProviderSettings) -> bool {
    if !settings.configured || matches!(settings.provider_kind, ModelProviderKind::None) {
        return false;
    }
    match settings.provider_kind {
        ModelProviderKind::Imported => true,
        ModelProviderKind::OpenAi | ModelProviderKind::Custom => {
            !settings.base_url.trim().is_empty() && !settings.default_model.trim().is_empty()
        }
        ModelProviderKind::None => false,
    }
}

pub(super) fn provider_has_required_secret(provider: &RuntimeModelProvider) -> bool {
    match provider.settings.provider_kind {
        ModelProviderKind::Imported => true,
        ModelProviderKind::OpenAi | ModelProviderKind::Custom => provider
            .api_key
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty()),
        ModelProviderKind::None => false,
    }
}

pub(super) fn evaluator_version_for_runtime_provider(
    provider: Option<&RuntimeModelProvider>,
) -> &str {
    let Some(provider) = provider else {
        return "model-stub-v1";
    };
    if provider.settings.provider_kind == ModelProviderKind::Imported {
        return "model-stub-v1";
    }
    let model = provider.settings.default_model.trim();
    if model.is_empty() {
        "model-stub-v1"
    } else {
        model
    }
}

pub(super) fn technical_assessment_policy_for_settings(
    settings: &ProjectEvaluationSettings,
) -> TechnicalAssessmentPolicy {
    settings
        .cv_policy_overrides
        .unwrap_or_else(|| technical_assessment_policy_for_cv_policy(settings.cv_policy))
}

fn technical_assessment_policy_for_cv_policy(cv_policy: CvPolicy) -> TechnicalAssessmentPolicy {
    match cv_policy {
        CvPolicy::Loose => TechnicalAssessmentPolicy::loose(),
        CvPolicy::Strict => TechnicalAssessmentPolicy::strict(),
        CvPolicy::Standard => TechnicalAssessmentPolicy::standard(),
    }
}

pub(super) fn runtime_model_provider_for_project_from_list(
    store: &SqliteStore,
    project_id: &str,
    providers: &[RuntimeModelProvider],
) -> Result<Option<RuntimeModelProvider>> {
    let Some(settings_id) = store
        .project_evaluation_settings(project_id)?
        .and_then(|settings| settings.model_provider_settings_id)
        .map(|value| normalized_model_provider_settings_id(&value))
    else {
        return Ok(None);
    };
    Ok(providers
        .iter()
        .find(|provider| provider.settings.settings_id == settings_id)
        .cloned())
}

pub(super) fn provider_configured_for_project_from_list(
    store: &SqliteStore,
    project_id: &str,
    providers: &[RuntimeModelProvider],
) -> Result<bool> {
    Ok(
        runtime_model_provider_for_project_from_list(store, project_id, providers)?
            .as_ref()
            .is_some_and(|provider| {
                model_provider_ready_for_work(&provider.settings)
                    && provider_has_required_secret(provider)
            }),
    )
}

pub(super) fn model_evaluation_skipped(
    project_id: &str,
    asset_group_id: &str,
    evaluator_version: &str,
    summary: &str,
    now_ms: i64,
) -> ModelEvaluation {
    ModelEvaluation {
        evaluation_id: format!(
            "model-evaluation-skipped-{}",
            stable_id_fragment(asset_group_id)
        ),
        run_id: evaluation_run_id(
            project_id,
            EvaluationRunType::AssetEvaluation,
            asset_group_id,
            now_ms,
        ),
        project_id: project_id.to_string(),
        asset_group_id: asset_group_id.to_string(),
        evaluator_kind: ModelEvaluatorKind::LlmVlm,
        evaluator_version: evaluator_version.to_string(),
        status: ModelEvaluationStatus::Skipped,
        score: 0,
        tier: ModelEvaluationTier::Reject,
        selectable: false,
        summary: summary.to_string(),
        strengths: Vec::new(),
        weaknesses: vec![summary.to_string()],
        technical_warnings: Vec::new(),
        prompt_pack_id: None,
        prompt_pack_version: None,
        prompt_hash: None,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    }
}

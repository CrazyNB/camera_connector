use std::str::FromStr;

use camera_connector_core::{
    AssetFormatRole, AssetGroupQuery, AssetGroupSort, AssetUserMarks, EvaluationRun, GuestMark,
    ModelProviderKind, ModelProviderSettings, ModelSendMode, ObjectFormat, Project,
    ProjectEvaluationSettings, PromptPack, SelectionRecommendation, SubjectAssessment,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{current_time_ms, MobileCoreError, MobileCoreResult};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct MobileAssetGroupQuery {
    pub username: Option<String>,
    pub source_name: Option<String>,
    pub original_path: Option<String>,
    pub remote_addr: Option<String>,
    pub format: Option<String>,
    pub role: Option<String>,
    pub sort: Option<String>,
    pub collection: Option<String>,
    pub favorite: Option<bool>,
    pub marked: Option<bool>,
    #[serde(default)]
    pub user_mark_any: Vec<String>,
    pub guest_mark: Option<String>,
    pub min_model_score: Option<i64>,
}

pub(crate) fn parse_json_value(json: &str) -> MobileCoreResult<Value> {
    Ok(serde_json::from_str(json)?)
}

pub(crate) fn project_json(project: Project) -> MobileCoreResult<String> {
    Ok(serde_json::to_string(&project.into_view())?)
}

pub(crate) fn project_option_json(project: Option<Project>) -> MobileCoreResult<String> {
    Ok(serde_json::to_string(&project.map(Project::into_view))?)
}

pub(crate) fn project_list_json(projects: Vec<Project>) -> MobileCoreResult<String> {
    let views = projects
        .into_iter()
        .map(Project::into_view)
        .collect::<Vec<_>>();
    Ok(serde_json::to_string(&views)?)
}

pub(crate) fn user_marks_json(marks: AssetUserMarks) -> MobileCoreResult<String> {
    Ok(serde_json::to_string(&marks)?)
}

pub(crate) fn default_unconfigured_provider_settings() -> ModelProviderSettings {
    ModelProviderSettings {
        settings_id: "global".to_string(),
        provider_kind: ModelProviderKind::None,
        provider_label: String::new(),
        base_url: String::new(),
        default_model: String::new(),
        default_max_image_side: 1024,
        default_send_mode: ModelSendMode::PreviewOnly,
        default_batch_size: 1,
        configured: false,
        api_key_configured: false,
        key_alias: None,
        updated_at_ms: current_time_ms(),
    }
}

pub(crate) fn model_provider_settings_json_value(settings: &ModelProviderSettings) -> Value {
    json!({
        "settings_id": settings.settings_id,
        "provider_kind": settings.provider_kind.as_str(),
        "provider_label": settings.provider_label,
        "base_url": settings.base_url,
        "default_model": settings.default_model,
        "default_max_image_side": settings.default_max_image_side,
        "default_send_mode": settings.default_send_mode.as_str(),
        "default_batch_size": settings.default_batch_size,
        "configured": settings.configured,
        "api_key_configured": settings.api_key_configured,
        "key_alias": settings.key_alias,
        "updated_at_ms": settings.updated_at_ms,
    })
}

pub(crate) fn project_evaluation_settings_json_value(
    settings: &ProjectEvaluationSettings,
) -> Value {
    json!({
        "project_id": settings.project_id,
        "auto_evaluate_on_upload": settings.auto_evaluate_on_upload,
        "auto_burst_recommendation_enabled": settings.auto_burst_recommendation_enabled,
        "project_recommendation_mode": settings.project_recommendation_mode.as_str(),
        "prompt_pack_id": settings.prompt_pack_id,
        "model_provider_settings_id": settings.model_provider_settings_id,
        "scene_profile": settings.scene_profile.as_str(),
        "cv_policy": settings.cv_policy.as_str(),
        "cv_policy_overrides": settings.cv_policy_overrides,
        "allow_risky_model_selects": settings.allow_risky_model_selects,
        "max_image_side": settings.max_image_side,
        "batch_size": settings.batch_size,
        "updated_at_ms": settings.updated_at_ms,
    })
}

pub(crate) fn prompt_pack_json_value(profile: &PromptPack) -> Value {
    json!({
        "prompt_pack_id": profile.prompt_pack_id,
        "distribution_folder": profile.distribution_folder,
        "name": profile.name,
        "version": profile.version,
        "author": profile.author,
        "style_tags": profile.style_tags,
        "scene_profile": profile.scene_profile.as_str(),
        "schema": profile.schema,
        "capabilities": profile.capabilities,
        "built_in": profile.built_in,
        "enabled": profile.enabled,
        "prompt_hash": profile.prompt_hash,
        "updated_at_ms": profile.updated_at_ms,
    })
}

pub(crate) fn prompt_pack_json_value_with_text(
    profile: &PromptPack,
    prompt_text: Option<String>,
) -> Value {
    let mut value = prompt_pack_json_value(profile);
    value["prompt_text"] = prompt_text.map(Value::String).unwrap_or(Value::Null);
    value
}

pub(crate) fn evaluation_run_json_value(run: &EvaluationRun) -> Value {
    json!({
        "run_id": run.run_id,
        "project_id": run.project_id,
        "run_type": run.run_type.as_str(),
        "trigger": run.trigger.as_str(),
        "status": run.status.as_str(),
        "provider_kind": run.provider_kind.as_str(),
        "provider_model": run.provider_model,
        "prompt_pack_id": run.prompt_pack_id,
        "prompt_pack_version": run.prompt_pack_version,
        "prompt_hash": run.prompt_hash,
        "error_message": run.error_message,
        "started_at_ms": run.started_at_ms,
        "completed_at_ms": run.completed_at_ms,
        "created_at_ms": run.created_at_ms,
    })
}

pub(crate) fn subject_assessment_json_value(assessment: &SubjectAssessment) -> Value {
    json!({
        "assessment_id": assessment.assessment_id,
        "project_id": assessment.project_id,
        "asset_group_id": assessment.asset_group_id,
        "subject_type": assessment.subject_type,
        "detector_kind": assessment.detector_kind,
        "detector_version": assessment.detector_version,
        "status": assessment.status.as_str(),
        "gate_status": assessment.gate_status,
        "regions": serde_json::from_str::<Value>(&assessment.regions_json).unwrap_or(Value::Null),
        "signals": serde_json::from_str::<Value>(&assessment.signals_json).unwrap_or(Value::Null),
        "summary": assessment.summary,
        "created_at_ms": assessment.created_at_ms,
        "updated_at_ms": assessment.updated_at_ms,
    })
}

pub(crate) fn selection_recommendation_json(
    recommendation: SelectionRecommendation,
) -> MobileCoreResult<String> {
    Ok(serde_json::to_string(&recommendation)?)
}

pub(crate) fn asset_group_query_from_json(query_json: &str) -> MobileCoreResult<AssetGroupQuery> {
    let query: MobileAssetGroupQuery = if query_json.trim().is_empty() {
        MobileAssetGroupQuery::default()
    } else {
        serde_json::from_str(query_json)?
    };
    Ok(AssetGroupQuery {
        username: query.username.and_then(non_blank),
        source_name: query.source_name.and_then(non_blank),
        original_path: query.original_path.and_then(non_blank),
        remote_addr: query.remote_addr.and_then(non_blank),
        format: query
            .format
            .and_then(non_blank)
            .map(|value| {
                ObjectFormat::from_str(&value)
                    .map_err(|_| MobileCoreError::InvalidAssetFormat(value))
            })
            .transpose()?,
        role: query
            .role
            .and_then(non_blank)
            .map(|value| {
                AssetFormatRole::from_str(&value)
                    .map_err(|_| MobileCoreError::InvalidAssetRole(value))
            })
            .transpose()?,
        sort: query
            .sort
            .and_then(non_blank)
            .and_then(|value| AssetGroupSort::from_wire(&value))
            .unwrap_or_default(),
        collection: query.collection.and_then(non_blank),
        favorite: query.favorite,
        marked: query.marked,
        user_mark_any: query.user_mark_any,
        guest_mark: query.guest_mark,
        min_model_score: query.min_model_score,
    })
}

pub(crate) fn guest_mark_from_patch(value: Option<String>) -> MobileCoreResult<Option<GuestMark>> {
    value
        .and_then(non_blank)
        .map(|value| GuestMark::from_wire(&value).ok_or(MobileCoreError::InvalidGuestMark(value)))
        .transpose()
}

pub(crate) fn non_blank(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

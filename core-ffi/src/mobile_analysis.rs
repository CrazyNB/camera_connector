use camera_connector_core::{
    AssetGroupModelEvaluationInput, ModelProviderKind, ModelProviderSettings, ModelSendMode,
    PreviewSample, ProjectEvaluationSettings, ProjectRecommendationMode,
    SelectionCandidateVisualInput, SubjectAssessment, TechnicalAssessmentPolicy,
};
use serde::Deserialize;
use serde_json::{json, Value};

use super::json_support::{
    default_unconfigured_provider_settings, evaluation_run_json_value,
    model_provider_settings_json_value, project_evaluation_settings_json_value,
    selection_recommendation_json, subject_assessment_json_value,
};
use super::parsing::{
    parse_cv_policy, parse_evaluation_run_status, parse_model_provider_kind, parse_model_send_mode,
    parse_project_recommendation_mode, parse_scene_profile,
};
use super::patch::{deserialize_patch_field, JsonPatchField};
use super::{current_time_ms, MobileCore, MobileCoreResult};

#[derive(Debug, Clone, Default, Deserialize)]
struct MobileModelProviderSettingsPatch {
    settings_id: Option<String>,
    provider_kind: Option<String>,
    provider_label: Option<String>,
    base_url: Option<String>,
    default_model: Option<String>,
    default_max_image_side: Option<i64>,
    default_send_mode: Option<String>,
    default_batch_size: Option<i64>,
    configured: Option<bool>,
    api_key: Option<String>,
    key_alias: Option<String>,
    updated_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct MobileProjectEvaluationSettingsPatch {
    auto_evaluate_on_upload: Option<bool>,
    auto_burst_recommendation_enabled: Option<bool>,
    project_recommendation_mode: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_field")]
    prompt_pack_id: JsonPatchField<String>,
    #[serde(default, deserialize_with = "deserialize_patch_field")]
    model_provider_settings_id: JsonPatchField<String>,
    scene_profile: Option<String>,
    cv_policy: Option<String>,
    #[serde(default, deserialize_with = "deserialize_patch_field")]
    cv_policy_overrides: JsonPatchField<TechnicalAssessmentPolicy>,
    allow_risky_model_selects: Option<bool>,
    max_image_side: Option<i64>,
    batch_size: Option<i64>,
    updated_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
struct MobileManualModelEvaluationRequest {
    project_id: String,
    asset_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MobileManualModelEvaluationInput {
    asset_group_id: String,
    sample: Value,
    preview_image_data_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MobileManualModelEvaluationInputsRequest {
    project_id: String,
    inputs: Vec<MobileManualModelEvaluationInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct MobileBurstRecommendationWithVisualsRequest {
    burst_group_id: String,
    candidate_visuals: Vec<SelectionCandidateVisualInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct MobileManualBurstGroupRequest {
    project_id: String,
    member_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MobileProjectRecommendationWithVisualsRequest {
    project_id: String,
    candidate_visuals: Vec<SelectionCandidateVisualInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct MobileSubjectAssessmentPayload {
    assessment_id: String,
    project_id: String,
    asset_group_id: String,
    subject_type: String,
    detector_kind: String,
    detector_version: String,
    status: String,
    gate_status: String,
    regions: Value,
    signals: Value,
    summary: String,
    created_at_ms: Option<i64>,
    updated_at_ms: Option<i64>,
}

impl MobileCore {
    pub fn drain_analysis_jobs_json(&self, limit: u32) -> MobileCoreResult<String> {
        let summary = self.service.drain_analysis_jobs(limit as usize)?;
        Ok(serde_json::to_string(&summary)?)
    }

    pub fn drain_analysis_jobs_with_provider_configured_json(
        &self,
        limit: u32,
        provider_configured: bool,
    ) -> MobileCoreResult<String> {
        let summary = self
            .service
            .drain_analysis_jobs_with_provider_configured(limit as usize, provider_configured)?;
        Ok(serde_json::to_string(&summary)?)
    }

    pub fn enqueue_model_evaluation_for_asset_groups_json(
        &self,
        request_json: String,
    ) -> MobileCoreResult<String> {
        let request = serde_json::from_str::<MobileManualModelEvaluationRequest>(&request_json)?;
        let enqueued_count = self.service.enqueue_model_evaluation_for_asset_groups(
            &request.project_id,
            &request.asset_group_ids,
        )?;
        Ok(serde_json::to_string(&json!({
            "project_id": request.project_id,
            "enqueued_count": enqueued_count,
        }))?)
    }

    pub fn evaluate_asset_groups_with_model_inputs_json(
        &self,
        request_json: String,
    ) -> MobileCoreResult<String> {
        let request =
            serde_json::from_str::<MobileManualModelEvaluationInputsRequest>(&request_json)?;
        let mut inputs = Vec::with_capacity(request.inputs.len());
        for input in request.inputs {
            let image_data_url = input
                .preview_image_data_url
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .or_else(|| {
                    input
                        .sample
                        .get("image_data_url")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                });
            let sample = serde_json::from_value::<PreviewSample>(input.sample)?;
            inputs.push(AssetGroupModelEvaluationInput {
                asset_group_id: input.asset_group_id,
                preview_sample: sample,
                preview_image_data_url: image_data_url,
            });
        }
        let saved_count = self
            .service
            .evaluate_asset_groups_with_model_inputs(&request.project_id, &inputs)?;
        Ok(serde_json::to_string(&json!({
            "project_id": request.project_id,
            "saved_count": saved_count,
        }))?)
    }

    pub fn recommend_burst_group_with_candidate_visuals_json(
        &self,
        request_json: String,
    ) -> MobileCoreResult<String> {
        let request =
            serde_json::from_str::<MobileBurstRecommendationWithVisualsRequest>(&request_json)?;
        let recommendation = self
            .service
            .recommend_burst_group_from_model_with_candidate_visuals(
                &request.burst_group_id,
                &request.candidate_visuals,
            )?;
        Ok(serde_json::to_string(&recommendation)?)
    }

    pub fn assess_asset_group_preview_json(
        &self,
        asset_group_id: String,
        sample_json: String,
        assessor_version: String,
    ) -> MobileCoreResult<String> {
        let sample = serde_json::from_str::<PreviewSample>(&sample_json)?;
        let assessment =
            self.service
                .assess_asset_group_preview(&asset_group_id, sample, &assessor_version)?;
        Ok(serde_json::to_string(&assessment)?)
    }

    pub fn assess_asset_group_preview_with_provider_configured_json(
        &self,
        asset_group_id: String,
        sample_json: String,
        assessor_version: String,
        provider_configured: bool,
    ) -> MobileCoreResult<String> {
        let sample_value = serde_json::from_str::<Value>(&sample_json)?;
        let preview_image_data_url = sample_value
            .get("image_data_url")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let sample = serde_json::from_value::<PreviewSample>(sample_value)?;
        let assessment = self
            .service
            .assess_asset_group_preview_with_image_data_url_and_provider_configured(
                &asset_group_id,
                sample,
                preview_image_data_url.as_deref(),
                &assessor_version,
                provider_configured,
            )?;
        Ok(serde_json::to_string(&assessment)?)
    }

    pub fn split_burst_member_json(
        &self,
        burst_group_id: String,
        member_group_id: String,
    ) -> MobileCoreResult<String> {
        let burst = self
            .service
            .split_burst_member(&burst_group_id, &member_group_id)?;
        Ok(serde_json::to_string(&burst)?)
    }

    pub fn create_manual_burst_group_json(&self, request_json: &str) -> MobileCoreResult<String> {
        let request: MobileManualBurstGroupRequest = serde_json::from_str(request_json)?;
        let burst = self
            .service
            .create_manual_burst_group(&request.project_id, &request.member_group_ids)?;
        Ok(serde_json::to_string(&burst)?)
    }

    pub fn model_provider_settings_json(&self) -> MobileCoreResult<String> {
        let settings = self
            .service
            .model_provider_settings()?
            .unwrap_or_else(default_unconfigured_provider_settings);
        Ok(serde_json::to_string(&model_provider_settings_json_value(
            &settings,
        ))?)
    }

    pub fn model_provider_settings_list_json(&self) -> MobileCoreResult<String> {
        let settings = self.service.model_provider_settings_list()?;
        Ok(serde_json::to_string(
            &settings
                .iter()
                .map(model_provider_settings_json_value)
                .collect::<Vec<_>>(),
        )?)
    }

    pub fn save_model_provider_settings_json(
        &self,
        settings_json: String,
    ) -> MobileCoreResult<String> {
        let patch: MobileModelProviderSettingsPatch = serde_json::from_str(&settings_json)?;
        let settings = ModelProviderSettings {
            settings_id: patch.settings_id.unwrap_or_else(|| "global".to_string()),
            provider_kind: patch
                .provider_kind
                .as_deref()
                .map(parse_model_provider_kind)
                .transpose()?
                .unwrap_or(ModelProviderKind::None),
            provider_label: patch.provider_label.unwrap_or_default(),
            base_url: patch.base_url.unwrap_or_default(),
            default_model: patch.default_model.unwrap_or_default(),
            default_max_image_side: patch.default_max_image_side.unwrap_or(1024),
            default_send_mode: patch
                .default_send_mode
                .as_deref()
                .map(parse_model_send_mode)
                .transpose()?
                .unwrap_or(ModelSendMode::PreviewOnly),
            default_batch_size: patch.default_batch_size.unwrap_or(1),
            configured: patch.configured.unwrap_or(false),
            api_key_configured: patch
                .api_key
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_some()
                || patch
                    .key_alias
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .is_some(),
            key_alias: patch.key_alias,
            updated_at_ms: patch.updated_at_ms.unwrap_or_else(current_time_ms),
        };
        let saved = self
            .service
            .save_model_provider_settings_with_api_key(settings, patch.api_key)?;
        Ok(serde_json::to_string(&model_provider_settings_json_value(
            &saved,
        ))?)
    }

    pub fn delete_model_provider_settings_json(
        &self,
        settings_id: String,
    ) -> MobileCoreResult<String> {
        let deleted = self.service.delete_model_provider_settings(&settings_id)?;
        Ok(serde_json::to_string(&json!({
            "settings_id": settings_id,
            "deleted": deleted,
        }))?)
    }

    pub fn project_evaluation_settings_json(&self, project_id: String) -> MobileCoreResult<String> {
        let settings = self
            .service
            .project_evaluation_settings(&project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            });
        Ok(serde_json::to_string(
            &project_evaluation_settings_json_value(&settings),
        )?)
    }

    pub fn save_project_evaluation_settings_json(
        &self,
        project_id: String,
        settings_json: String,
    ) -> MobileCoreResult<String> {
        let patch: MobileProjectEvaluationSettingsPatch = serde_json::from_str(&settings_json)?;
        let existing = self
            .service
            .project_evaluation_settings(&project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(
                    project_id.clone(),
                    current_time_ms(),
                )
            });
        let settings = ProjectEvaluationSettings {
            project_id,
            auto_evaluate_on_upload: patch
                .auto_evaluate_on_upload
                .unwrap_or(existing.auto_evaluate_on_upload),
            auto_burst_recommendation_enabled: patch
                .auto_burst_recommendation_enabled
                .unwrap_or(existing.auto_burst_recommendation_enabled),
            project_recommendation_mode: patch
                .project_recommendation_mode
                .as_deref()
                .map(parse_project_recommendation_mode)
                .transpose()?
                .unwrap_or(ProjectRecommendationMode::Manual),
            prompt_pack_id: match patch.prompt_pack_id {
                JsonPatchField::Missing => existing.prompt_pack_id,
                JsonPatchField::Null => None,
                JsonPatchField::Value(prompt_pack_id) => Some(prompt_pack_id),
            },
            model_provider_settings_id: match patch.model_provider_settings_id {
                JsonPatchField::Missing => existing.model_provider_settings_id,
                JsonPatchField::Null => None,
                JsonPatchField::Value(settings_id) => Some(settings_id),
            },
            scene_profile: patch
                .scene_profile
                .as_deref()
                .map(parse_scene_profile)
                .transpose()?
                .unwrap_or(existing.scene_profile),
            cv_policy: patch
                .cv_policy
                .as_deref()
                .map(parse_cv_policy)
                .transpose()?
                .unwrap_or(existing.cv_policy),
            cv_policy_overrides: match patch.cv_policy_overrides {
                JsonPatchField::Missing => existing.cv_policy_overrides,
                JsonPatchField::Null => None,
                JsonPatchField::Value(policy) => Some(policy),
            },
            allow_risky_model_selects: patch
                .allow_risky_model_selects
                .unwrap_or(existing.allow_risky_model_selects),
            max_image_side: patch.max_image_side.or(existing.max_image_side),
            batch_size: patch.batch_size.or(existing.batch_size),
            updated_at_ms: patch.updated_at_ms.unwrap_or_else(current_time_ms),
        };
        let saved = self.service.save_project_evaluation_settings(settings)?;
        Ok(serde_json::to_string(
            &project_evaluation_settings_json_value(&saved),
        )?)
    }

    pub fn generate_project_recommendation_json(
        &self,
        project_id: String,
    ) -> MobileCoreResult<String> {
        let recommendation = self
            .service
            .generate_project_recommendation(&project_id, self.next_action_time_ms())?;
        selection_recommendation_json(recommendation)
    }

    pub fn generate_project_recommendation_with_candidate_visuals_json(
        &self,
        request_json: String,
    ) -> MobileCoreResult<String> {
        let request =
            serde_json::from_str::<MobileProjectRecommendationWithVisualsRequest>(&request_json)?;
        let recommendation = self
            .service
            .generate_project_recommendation_with_candidate_visuals(
                &request.project_id,
                &request.candidate_visuals,
                self.next_action_time_ms(),
            )?;
        selection_recommendation_json(recommendation)
    }

    pub fn latest_project_recommendation_run_status_json(
        &self,
        project_id: String,
    ) -> MobileCoreResult<String> {
        let run = self
            .service
            .latest_project_recommendation_run_status(&project_id)?;
        Ok(serde_json::to_string(
            &run.map(|run| evaluation_run_json_value(&run)),
        )?)
    }

    pub fn should_schedule_subject_assessment_json(
        &self,
        project_id: String,
    ) -> MobileCoreResult<String> {
        let should_schedule = self
            .service
            .should_schedule_subject_assessment(&project_id)?;
        Ok(serde_json::to_string(&should_schedule)?)
    }

    pub fn save_subject_assessment_json(
        &self,
        assessment_json: String,
    ) -> MobileCoreResult<String> {
        let payload: MobileSubjectAssessmentPayload = serde_json::from_str(&assessment_json)?;
        let now_ms = self.next_action_time_ms();
        let assessment = SubjectAssessment {
            assessment_id: payload.assessment_id,
            project_id: payload.project_id,
            asset_group_id: payload.asset_group_id,
            subject_type: payload.subject_type,
            detector_kind: payload.detector_kind,
            detector_version: payload.detector_version,
            status: parse_evaluation_run_status(&payload.status)?,
            gate_status: payload.gate_status,
            regions_json: serde_json::to_string(&payload.regions)?,
            signals_json: serde_json::to_string(&payload.signals)?,
            summary: payload.summary,
            created_at_ms: payload.created_at_ms.unwrap_or(now_ms),
            updated_at_ms: payload.updated_at_ms.unwrap_or(now_ms),
        };
        let saved = self.service.save_subject_assessment(assessment)?;
        Ok(serde_json::to_string(&subject_assessment_json_value(
            &saved,
        ))?)
    }

    pub fn subject_assessments_for_asset_groups_json(
        &self,
        project_id: String,
        group_ids_json: String,
    ) -> MobileCoreResult<String> {
        let group_ids: Vec<String> = serde_json::from_str(&group_ids_json)?;
        let assessments = self
            .service
            .subject_assessments_for_asset_groups(&project_id, &group_ids)?;
        let values = assessments
            .iter()
            .map(subject_assessment_json_value)
            .collect::<Vec<_>>();
        Ok(serde_json::to_string(&values)?)
    }
}

use camera_connector_core::{
    CameraConnectorService, CvPolicy, ModelProviderKind, ModelProviderSettings, ModelSendMode,
    ProjectEvaluationSettings, ProjectRecommendationMode, PromptPack, SceneProfile,
};

use super::{
    current_time_ms, desktop_error, DesktopError, DesktopModelProviderSettings,
    DesktopProjectEvaluationSettings, DesktopPromptPack, SaveModelProviderSettingsRequest,
};

pub(super) fn desktop_model_provider_settings(
    settings: ModelProviderSettings,
) -> DesktopModelProviderSettings {
    DesktopModelProviderSettings {
        settings_id: settings.settings_id,
        provider_kind: settings.provider_kind.as_str().to_string(),
        provider_label: settings.provider_label,
        base_url: settings.base_url,
        default_model: settings.default_model,
        default_max_image_side: settings.default_max_image_side,
        default_send_mode: settings.default_send_mode.as_str().to_string(),
        default_batch_size: settings.default_batch_size,
        configured: settings.configured,
        api_key_configured: settings.api_key_configured,
        key_alias: settings.key_alias,
        updated_at_ms: settings.updated_at_ms,
    }
}

pub(super) fn model_provider_settings_from_request(
    request: SaveModelProviderSettingsRequest,
) -> ModelProviderSettings {
    ModelProviderSettings {
        settings_id: request.settings_id,
        provider_kind: ModelProviderKind::from_str(request.provider_kind.trim()),
        provider_label: request.provider_label,
        base_url: request.base_url,
        default_model: request.default_model,
        default_max_image_side: request.default_max_image_side.max(1),
        default_send_mode: ModelSendMode::from_str(request.default_send_mode.trim()),
        default_batch_size: request.default_batch_size.max(1),
        configured: request.configured,
        api_key_configured: request
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_some(),
        key_alias: request.key_alias,
        updated_at_ms: current_time_ms(),
    }
}

pub(super) fn desktop_prompt_pack(
    service: &CameraConnectorService,
    pack: PromptPack,
) -> Result<DesktopPromptPack, DesktopError> {
    let shared_preference = service
        .prompt_markdown_for_pack(&pack.prompt_pack_id)
        .map_err(desktop_error)?;
    Ok(DesktopPromptPack {
        prompt_pack_id: pack.prompt_pack_id,
        distribution_folder: pack.distribution_folder,
        name: pack.name,
        version: pack.version,
        author: pack.author,
        style_tags: pack.style_tags,
        scene_profile: pack.scene_profile.as_str().to_string(),
        schema: pack.schema,
        capabilities: pack.capabilities,
        built_in: pack.built_in,
        enabled: pack.enabled,
        shared_preference,
        prompt_hash: pack.prompt_hash,
        updated_at_ms: pack.updated_at_ms,
    })
}

pub(super) fn desktop_prompt_packs(
    service: &CameraConnectorService,
    packs: Vec<PromptPack>,
) -> Result<Vec<DesktopPromptPack>, DesktopError> {
    packs
        .into_iter()
        .map(|pack| desktop_prompt_pack(service, pack))
        .collect()
}

pub(super) fn desktop_project_evaluation_settings(
    settings: ProjectEvaluationSettings,
) -> DesktopProjectEvaluationSettings {
    DesktopProjectEvaluationSettings {
        project_id: settings.project_id,
        auto_evaluate_on_upload: settings.auto_evaluate_on_upload,
        auto_burst_recommendation_enabled: settings.auto_burst_recommendation_enabled,
        project_recommendation_mode: settings.project_recommendation_mode.as_str().to_string(),
        prompt_pack_id: settings.prompt_pack_id,
        model_provider_settings_id: settings.model_provider_settings_id,
        scene_profile: settings.scene_profile.as_str().to_string(),
        cv_policy: settings.cv_policy.as_str().to_string(),
        cv_policy_overrides: settings.cv_policy_overrides,
        allow_risky_model_selects: settings.allow_risky_model_selects,
        max_image_side: settings.max_image_side,
        batch_size: settings.batch_size,
        updated_at_ms: settings.updated_at_ms,
    }
}

pub(super) fn project_evaluation_settings_from_desktop(
    settings: DesktopProjectEvaluationSettings,
) -> ProjectEvaluationSettings {
    ProjectEvaluationSettings {
        project_id: settings.project_id,
        auto_evaluate_on_upload: settings.auto_evaluate_on_upload,
        auto_burst_recommendation_enabled: settings.auto_burst_recommendation_enabled,
        project_recommendation_mode: ProjectRecommendationMode::from_str(
            settings.project_recommendation_mode.trim(),
        ),
        prompt_pack_id: settings.prompt_pack_id,
        model_provider_settings_id: settings.model_provider_settings_id,
        scene_profile: SceneProfile::from_str(settings.scene_profile.trim()),
        cv_policy: CvPolicy::from_str(settings.cv_policy.trim()),
        cv_policy_overrides: settings.cv_policy_overrides,
        allow_risky_model_selects: settings.allow_risky_model_selects,
        max_image_side: settings.max_image_side,
        batch_size: settings.batch_size,
        updated_at_ms: current_time_ms(),
    }
}

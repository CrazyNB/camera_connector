use std::path::PathBuf;

use camera_connector_core::{
    CvPolicy, ModelProviderKind, ModelSendMode, ProjectRecommendationMode, PushProtocol,
    ReceiverSettingsUpdate, SceneProfile, StoredObjectLocation,
};

use super::{MobileCoreError, MobileCoreResult, MobileReceiverSettingsPatch};

impl TryFrom<MobileReceiverSettingsPatch> for ReceiverSettingsUpdate {
    type Error = MobileCoreError;

    fn try_from(patch: MobileReceiverSettingsPatch) -> MobileCoreResult<Self> {
        Ok(Self {
            protocol: patch.protocol.map(parse_protocol).transpose()?,
            bind_host: patch.bind_host,
            ftp_port: patch.ftp_port,
            sftp_port: patch.sftp_port,
            output_dir: patch.output_dir.map(PathBuf::from),
            state_dir: patch.state_dir.map(PathBuf::from),
            advertised_host: patch.advertised_host,
            source_name: patch.source_name,
            defer_publish: patch.defer_publish,
        })
    }
}

pub(crate) fn parse_protocol(protocol: String) -> MobileCoreResult<PushProtocol> {
    match protocol.trim().to_ascii_lowercase().as_str() {
        "ftp" => Ok(PushProtocol::Ftp),
        "sftp" => Ok(PushProtocol::Sftp),
        _ => Err(MobileCoreError::InvalidProtocol(protocol)),
    }
}

pub(crate) fn parse_model_provider_kind(value: &str) -> MobileCoreResult<ModelProviderKind> {
    match value {
        "none" => Ok(ModelProviderKind::None),
        "openai" => Ok(ModelProviderKind::OpenAi),
        "custom" => Ok(ModelProviderKind::Custom),
        "imported" => Ok(ModelProviderKind::Imported),
        _ => invalid_config_value("provider_kind", value),
    }
}

pub(crate) fn parse_model_send_mode(value: &str) -> MobileCoreResult<ModelSendMode> {
    match value {
        "preview_only" => Ok(ModelSendMode::PreviewOnly),
        "detail_image" => Ok(ModelSendMode::DetailImage),
        _ => invalid_config_value("default_send_mode", value),
    }
}

pub(crate) fn parse_scene_profile(value: &str) -> MobileCoreResult<SceneProfile> {
    match value {
        "general" => Ok(SceneProfile::General),
        "portrait" => Ok(SceneProfile::Portrait),
        "action" => Ok(SceneProfile::Action),
        "landscape" => Ok(SceneProfile::Landscape),
        "custom" => Ok(SceneProfile::Custom),
        _ => invalid_config_value("scene_profile", value),
    }
}

pub(crate) fn parse_cv_policy(value: &str) -> MobileCoreResult<CvPolicy> {
    match value {
        "loose" => Ok(CvPolicy::Loose),
        "standard" => Ok(CvPolicy::Standard),
        "strict" => Ok(CvPolicy::Strict),
        _ => invalid_config_value("cv_policy", value),
    }
}

pub(crate) fn parse_project_recommendation_mode(
    value: &str,
) -> MobileCoreResult<ProjectRecommendationMode> {
    match value {
        "manual" => Ok(ProjectRecommendationMode::Manual),
        _ => invalid_config_value("project_recommendation_mode", value),
    }
}

pub(crate) fn parse_evaluation_run_status(
    value: &str,
) -> MobileCoreResult<camera_connector_core::EvaluationRunStatus> {
    match value {
        "pending" => Ok(camera_connector_core::EvaluationRunStatus::Pending),
        "running" => Ok(camera_connector_core::EvaluationRunStatus::Running),
        "ready" => Ok(camera_connector_core::EvaluationRunStatus::Ready),
        "failed" => Ok(camera_connector_core::EvaluationRunStatus::Failed),
        "skipped" => Ok(camera_connector_core::EvaluationRunStatus::Skipped),
        _ => invalid_config_value("status", value),
    }
}

pub(crate) fn parse_storage_location(
    kind: String,
    location: String,
) -> MobileCoreResult<StoredObjectLocation> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "local_path" => Ok(StoredObjectLocation::local_path(location)),
        "document_uri" => Ok(StoredObjectLocation::document_uri(location)),
        "media_uri" => Ok(StoredObjectLocation::media_uri(location)),
        "photo_asset" => Ok(StoredObjectLocation::photo_asset(location)),
        _ => Err(MobileCoreError::InvalidLocationKind(kind)),
    }
}

fn invalid_config_value<T>(field: &'static str, value: &str) -> MobileCoreResult<T> {
    Err(MobileCoreError::InvalidConfigValue {
        field,
        value: value.to_string(),
    })
}

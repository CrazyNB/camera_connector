use std::collections::BTreeSet;

use camera_connector_core::{
    AnalysisJobType, CameraConnectorConfig, CameraConnectorService, CvPolicy, EvaluationRun,
    EvaluationRunStatus, EvaluationRunTrigger, EvaluationRunType, ModelProviderKind,
    ModelProviderSettings, ModelSendMode, ProjectRecommendationMode, SceneProfile, SqliteStore,
    StoredObjectLocation, SubjectAssessment, TechnicalAssessmentPolicy, TransferRecord,
    TransferStatus,
};

#[path = "evaluation_config_tests/prompt_packs.rs"]
mod prompt_packs;
#[path = "evaluation_config_tests/subjects.rs"]
mod subjects;

#[test]
fn evaluation_config_tests_builtin_prompt_packs_cover_multiple_photographic_scenarios() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));

    let packs = service
        .global_prompt_packs()
        .expect("built-in prompt packs should list");
    let built_in = packs
        .iter()
        .filter(|pack| pack.built_in)
        .collect::<Vec<_>>();
    let built_in_ids = built_in
        .iter()
        .map(|pack| pack.prompt_pack_id.as_str())
        .collect::<BTreeSet<_>>();

    assert!(
        built_in.len() >= 5,
        "shipping prompt pack library should cover at least five styles/scenes"
    );
    for expected_id in [
        "general-default",
        "documentary-integrity",
        "portrait-editorial",
        "portrait-lifestyle",
        "landscape-fine-art",
        "wildlife-ethics",
        "action-sports-moment",
    ] {
        assert!(
            built_in_ids.contains(expected_id),
            "missing built-in prompt pack {expected_id}"
        );
    }

    let scene_profiles = built_in
        .iter()
        .map(|pack| pack.scene_profile.as_str())
        .collect::<BTreeSet<_>>();
    assert!(
        scene_profiles.len() >= 4,
        "prompt packs should cover multiple scene profiles"
    );

    for pack in built_in {
        assert_eq!(pack.distribution_folder, "builtin");
        assert_eq!(pack.schema, "model-evaluation-v1");
        assert!(pack.capabilities.contains(&"single_evaluation".to_string()));
        assert!(pack.capabilities.contains(&"burst_selection".to_string()));
        assert!(pack.capabilities.contains(&"project_selection".to_string()));
        assert!(
            pack.style_tags.len() >= 2,
            "{} should expose product-facing style tags",
            pack.prompt_pack_id
        );
        let markdown = service
            .prompt_markdown_for_pack(&pack.prompt_pack_id)
            .expect("prompt markdown should load")
            .expect("built-in prompt markdown should exist");
        assert!(
            markdown.chars().count() >= 600,
            "{} prompt is too thin to guide aesthetic judging",
            pack.prompt_pack_id
        );
        assert!(
            markdown.contains("评分维度") && markdown.contains("淘汰") && markdown.contains("连拍"),
            "{} prompt should include scoring dimensions, rejection criteria, and burst guidance",
            pack.prompt_pack_id
        );
    }

    let portrait_lifestyle = service
        .prompt_markdown_for_pack("portrait-lifestyle")
        .expect("portrait lifestyle prompt should load")
        .expect("portrait lifestyle prompt should exist");
    for keyword in ["故事感", "情绪", "日系", "运动", "广角", "剪影"] {
        assert!(
            portrait_lifestyle.contains(keyword),
            "portrait lifestyle prompt should cover {keyword}"
        );
    }
}

#[test]
fn evaluation_config_tests_enums_round_trip_and_fallback() {
    for variant in [
        ModelProviderKind::None,
        ModelProviderKind::OpenAi,
        ModelProviderKind::Custom,
        ModelProviderKind::Imported,
    ] {
        assert_eq!(ModelProviderKind::from_str(variant.as_str()), variant);
    }
    assert_eq!(
        ModelProviderKind::from_str("unexpected"),
        ModelProviderKind::None
    );

    for variant in [ModelSendMode::PreviewOnly, ModelSendMode::DetailImage] {
        assert_eq!(ModelSendMode::from_str(variant.as_str()), variant);
    }
    assert_eq!(
        ModelSendMode::from_str("unexpected"),
        ModelSendMode::PreviewOnly
    );

    for variant in [
        SceneProfile::General,
        SceneProfile::Portrait,
        SceneProfile::Action,
        SceneProfile::Landscape,
        SceneProfile::Custom,
    ] {
        assert_eq!(SceneProfile::from_str(variant.as_str()), variant);
    }
    assert_eq!(SceneProfile::from_str("macro"), SceneProfile::General);

    for variant in [CvPolicy::Loose, CvPolicy::Standard, CvPolicy::Strict] {
        assert_eq!(CvPolicy::from_str(variant.as_str()), variant);
    }
    assert_eq!(CvPolicy::from_str("surprising"), CvPolicy::Standard);

    {
        let variant = ProjectRecommendationMode::Manual;
        assert_eq!(
            ProjectRecommendationMode::from_str(variant.as_str()),
            variant
        );
    }
    assert_eq!(
        ProjectRecommendationMode::from_str("automatic"),
        ProjectRecommendationMode::Manual
    );

    for variant in [
        EvaluationRunType::AssetEvaluation,
        EvaluationRunType::BurstRecommendation,
        EvaluationRunType::ProjectRecommendation,
    ] {
        assert_eq!(EvaluationRunType::from_str(variant.as_str()), variant);
    }
    assert_eq!(
        EvaluationRunType::from_str("unexpected"),
        EvaluationRunType::AssetEvaluation
    );

    for variant in [
        EvaluationRunTrigger::Upload,
        EvaluationRunTrigger::BurstStable,
        EvaluationRunTrigger::Manual,
        EvaluationRunTrigger::Retry,
    ] {
        assert_eq!(EvaluationRunTrigger::from_str(variant.as_str()), variant);
    }
    assert_eq!(
        EvaluationRunTrigger::from_str("unexpected"),
        EvaluationRunTrigger::Upload
    );

    for variant in [
        EvaluationRunStatus::Pending,
        EvaluationRunStatus::Running,
        EvaluationRunStatus::Ready,
        EvaluationRunStatus::Failed,
        EvaluationRunStatus::Skipped,
    ] {
        assert_eq!(EvaluationRunStatus::from_str(variant.as_str()), variant);
    }
    assert_eq!(
        EvaluationRunStatus::from_str("unexpected"),
        EvaluationRunStatus::Pending
    );
}

#[test]
fn evaluation_config_tests_service_persists_model_provider_settings_in_app_config() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("camera-connector.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));

    assert!(service
        .model_provider_settings()
        .expect("settings should query")
        .is_none());

    let saved = service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "global".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "OpenAI".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                default_model: "gpt-5.1-mini".to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::DetailImage,
                default_batch_size: 4,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 1000,
            },
            Some("sk-test".to_string()),
        )
        .expect("settings should save");

    let config = CameraConnectorConfig::load(Some(&config_path)).expect("config should load");
    let raw_config = std::fs::read_to_string(&config_path).expect("config should read");
    assert!(
        !raw_config.contains("\"model_provider\":"),
        "model provider configuration should be stored only as resource list"
    );
    assert_eq!(saved.base_url, "https://api.openai.com/v1");
    assert!(saved.api_key_configured);
    assert_eq!(config.model_providers.len(), 1);
    assert_eq!(
        config.model_providers[0].base_url,
        "https://api.openai.com/v1"
    );
    assert_eq!(
        config.model_providers[0].api_key.as_deref(),
        Some("sk-test")
    );
}

#[test]
fn evaluation_config_tests_model_provider_profiles_are_independent_resources() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("camera-connector.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));

    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "preview-fast".to_string(),
                provider_kind: ModelProviderKind::Custom,
                provider_label: "OpenAI compatible".to_string(),
                base_url: "https://models.example/v1".to_string(),
                default_model: "gpt-fast".to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 4,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 1000,
            },
            Some("sk-shared".to_string()),
        )
        .expect("first settings should save");
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "detail-eval".to_string(),
                provider_kind: ModelProviderKind::Custom,
                provider_label: "OpenAI compatible".to_string(),
                base_url: "https://models.example/v1".to_string(),
                default_model: "gpt-photo-eval".to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::DetailImage,
                default_batch_size: 1,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 2000,
            },
            Some("sk-shared".to_string()),
        )
        .expect("second settings should save");

    let config = CameraConnectorConfig::load(Some(&config_path)).expect("config should load");
    let ids = config
        .model_providers
        .iter()
        .map(|provider| {
            (
                provider.settings_id.as_str(),
                provider.base_url.as_str(),
                provider.default_model.as_str(),
                provider.api_key.as_deref(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            (
                "preview-fast",
                "https://models.example/v1",
                "gpt-fast",
                Some("sk-shared")
            ),
            (
                "detail-eval",
                "https://models.example/v1",
                "gpt-photo-eval",
                Some("sk-shared")
            ),
        ]
    );
}

#[test]
fn evaluation_config_tests_delete_model_provider_profile_removes_only_that_resource() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("camera-connector.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));

    for (settings_id, default_model) in [
        ("preview-fast", "gpt-fast"),
        ("detail-eval", "gpt-photo-eval"),
    ] {
        service
            .save_model_provider_settings_with_api_key(
                ModelProviderSettings {
                    settings_id: settings_id.to_string(),
                    provider_kind: ModelProviderKind::Custom,
                    provider_label: "OpenAI compatible".to_string(),
                    base_url: "https://models.example/v1".to_string(),
                    default_model: default_model.to_string(),
                    default_max_image_side: 1600,
                    default_send_mode: ModelSendMode::PreviewOnly,
                    default_batch_size: 1,
                    configured: true,
                    api_key_configured: false,
                    key_alias: None,
                    updated_at_ms: 1000,
                },
                Some("sk-shared".to_string()),
            )
            .expect("settings should save");
    }

    service
        .delete_model_provider_settings("preview-fast")
        .expect("settings should delete");

    let profiles = service
        .model_provider_settings_list()
        .expect("settings should list");
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].settings_id, "detail-eval");
    assert_eq!(profiles[0].default_model, "gpt-photo-eval");
}

fn base_project_settings(project_id: &str) -> camera_connector_core::ProjectEvaluationSettings {
    camera_connector_core::ProjectEvaluationSettings {
        project_id: project_id.to_string(),
        auto_evaluate_on_upload: false,
        auto_burst_recommendation_enabled: true,
        project_recommendation_mode: ProjectRecommendationMode::Manual,
        prompt_pack_id: None,
        model_provider_settings_id: None,
        scene_profile: SceneProfile::General,
        cv_policy: CvPolicy::Standard,
        cv_policy_overrides: None,
        allow_risky_model_selects: false,
        max_image_side: None,
        batch_size: None,
        updated_at_ms: 500,
    }
}

#[test]
fn evaluation_config_tests_project_settings_round_trip_cv_policy_overrides() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Custom CV Policy")
        .expect("project should create");
    let mut settings = base_project_settings(&project.project_id);
    let mut policy = TechnicalAssessmentPolicy::standard();
    policy.clipping_high_ratio = 0.07;
    policy.clipping_high_connected_ratio = 0.11;
    settings.cv_policy_overrides = Some(policy);

    let saved = store
        .save_project_evaluation_settings(settings.clone())
        .expect("settings should save");
    let reloaded = store
        .project_evaluation_settings(&project.project_id)
        .expect("settings should query")
        .expect("settings should exist");

    assert_eq!(saved.cv_policy_overrides, Some(policy));
    assert_eq!(reloaded.cv_policy_overrides, Some(policy));

    settings.cv_policy_overrides = None;
    store
        .save_project_evaluation_settings(settings)
        .expect("settings should clear override");
    let cleared = store
        .project_evaluation_settings(&project.project_id)
        .expect("settings should reload")
        .expect("settings should exist");

    assert_eq!(cleared.cv_policy_overrides, None);
}

#[test]
fn evaluation_config_tests_evaluation_run_save_query_preserves_manual_trigger_and_status() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Run Project")
        .expect("project should create");
    let run = EvaluationRun {
        run_id: "run-manual-project-1".to_string(),
        project_id: project.project_id.clone(),
        run_type: EvaluationRunType::ProjectRecommendation,
        trigger: EvaluationRunTrigger::Manual,
        status: EvaluationRunStatus::Ready,
        provider_kind: ModelProviderKind::Imported,
        provider_model: "imported-evaluator-v1".to_string(),
        prompt_pack_id: Some("general-default".to_string()),
        prompt_pack_version: Some("general-default-v1".to_string()),
        prompt_hash: Some("builtin-general-default-v1".to_string()),
        settings_snapshot_json: "{\"manual\":true}".to_string(),
        error_message: None,
        started_at_ms: Some(3000),
        completed_at_ms: Some(3200),
        created_at_ms: 3000,
    };

    store
        .save_evaluation_run(run.clone())
        .expect("run should save");
    let loaded = store
        .latest_evaluation_run(
            &project.project_id,
            EvaluationRunType::ProjectRecommendation,
        )
        .expect("run should query")
        .expect("run should exist");

    assert_eq!(loaded, run);
    assert_eq!(loaded.trigger, EvaluationRunTrigger::Manual);
    assert_eq!(loaded.status, EvaluationRunStatus::Ready);
}

fn completed_transfer(
    transfer_id: &str,
    original_path: &str,
    completed_at_ms: i64,
) -> TransferRecord {
    let final_filename = original_path
        .rsplit('/')
        .next()
        .expect("filename should exist")
        .to_string();
    TransferRecord {
        transfer_id: transfer_id.to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: original_path.to_string(),
        final_filename: final_filename.clone(),
        final_location: Some(StoredObjectLocation::local_path(final_filename)),
        size_bytes: 100,
        username: Some("z5".to_string()),
        remote_addr: Some("192.168.137.56".to_string()),
        source_name: Some("Studio Z5".to_string()),
        started_at_ms: completed_at_ms - 1,
        completed_at_ms: Some(completed_at_ms),
        error: None,
    }
}

fn create_subject_asset_group(
    store: &SqliteStore,
    project_id: &str,
    transfer_id: &str,
    original_path: &str,
) -> String {
    store
        .record_transfer(
            project_id,
            completed_transfer(transfer_id, original_path, 10_000),
        )
        .expect("transfer should record");
    store
        .stored_asset_groups(project_id)
        .expect("asset groups should query")
        .into_iter()
        .find(|group| group.display_key == file_stem(original_path))
        .map(|group| group.group_id)
        .expect("created asset group should exist")
}

fn file_stem(path: &str) -> String {
    let filename = path.rsplit('/').next().unwrap_or(path);
    filename
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(filename)
        .to_string()
}

fn subject_assessment_for(
    project_id: &str,
    asset_group_id: &str,
    assessment_id: &str,
) -> SubjectAssessment {
    SubjectAssessment {
        assessment_id: assessment_id.to_string(),
        project_id: project_id.to_string(),
        asset_group_id: asset_group_id.to_string(),
        subject_type: "face".to_string(),
        detector_kind: "imported".to_string(),
        detector_version: "imported-face-v1".to_string(),
        status: EvaluationRunStatus::Ready,
        gate_status: "pass".to_string(),
        regions_json: "[]".to_string(),
        signals_json: "{}".to_string(),
        summary: "Imported subject assessment.".to_string(),
        created_at_ms: 4000,
        updated_at_ms: 4100,
    }
}

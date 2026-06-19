use std::{collections::BTreeSet, fs};

use camera_connector_core::{
    AnalysisJobType, CameraConnectorConfig, CameraConnectorService, CvPolicy, EvaluationRun,
    EvaluationRunStatus, EvaluationRunTrigger, EvaluationRunType, ModelProviderKind,
    ModelProviderSettings, ModelSendMode, ProjectRecommendationMode, SceneProfile, SqliteStore,
    StoredObjectLocation, SubjectAssessment, TechnicalAssessmentPolicy, TransferRecord,
    TransferStatus,
};

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
fn evaluation_config_tests_service_creates_user_prompt_pack_from_shared_preference() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));

    let pack = service
        .create_global_prompt_pack(
            "Documentary Preference",
            vec!["documentary".to_string(), "portrait".to_string()],
            SceneProfile::General,
            "user",
            "Prefer quiet documentary emotion.",
            10_000,
        )
        .expect("prompt pack should create");

    assert_eq!(pack.name, "Documentary Preference");
    assert_eq!(pack.prompt_pack_id, "documentary-preference");
    assert!(!pack.built_in);
    assert_eq!(pack.version, "user-10000");
    assert!(pack.prompt_hash.starts_with("fnv1a64-"));
    assert!(pack.prompt_text.contains("shared_preference"));
    assert!(pack
        .prompt_text
        .contains("Prefer quiet documentary emotion."));
    assert!(service
        .prompt_text_for_pack(&pack.prompt_pack_id)
        .expect("prompt text should load")
        .expect("prompt text should exist")
        .contains("Prefer quiet documentary emotion."));
    assert_eq!(
        service
            .prompt_markdown_for_pack(&pack.prompt_pack_id)
            .expect("prompt markdown should load")
            .expect("prompt markdown should exist"),
        "Prefer quiet documentary emotion."
    );

    let prompt_pack_root = service
        .storage_state_dir()
        .expect("storage state dir should resolve")
        .join("prompt-packs");
    assert!(prompt_pack_root
        .join("user")
        .join("documentary-preference")
        .exists());
    let prompt_file = fs::read_dir(prompt_pack_root.join("user"))
        .expect("prompt pack root should exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().join("PROMPT.md"))
        .find(|path| path.exists())
        .expect("prompt markdown file should exist");
    let prompt_markdown = fs::read_to_string(prompt_file).expect("prompt markdown should read");
    assert_eq!(prompt_markdown, "Prefer quiet documentary emotion.");
    assert!(!prompt_markdown.contains("shared_preference"));
}

#[test]
fn evaluation_config_tests_user_prompt_pack_uses_shareable_folder_names() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));

    let first = service
        .create_global_prompt_pack(
            "xx",
            vec!["test".to_string()],
            SceneProfile::General,
            "portrait-pack",
            "First prompt.",
            10_000,
        )
        .expect("first prompt pack should create");
    let second = service
        .create_global_prompt_pack(
            "xx",
            vec!["test".to_string()],
            SceneProfile::General,
            "portrait-pack",
            "Second prompt.",
            10_001,
        )
        .expect("duplicate prompt pack should create with short suffix");

    assert_eq!(first.prompt_pack_id, "xx");
    assert_eq!(first.distribution_folder, "portrait-pack");
    assert_eq!(second.prompt_pack_id, "xx-2");
    assert_eq!(second.distribution_folder, "portrait-pack");

    let prompt_pack_root = service
        .storage_state_dir()
        .expect("storage state dir should resolve")
        .join("prompt-packs");
    assert_eq!(
        fs::read_to_string(
            prompt_pack_root
                .join("portrait-pack")
                .join("xx")
                .join("PROMPT.md"),
        )
        .expect("first prompt should read"),
        "First prompt."
    );
    assert_eq!(
        fs::read_to_string(
            prompt_pack_root
                .join("portrait-pack")
                .join("xx-2")
                .join("PROMPT.md"),
        )
        .expect("second prompt should read"),
        "Second prompt."
    );
}

#[test]
fn evaluation_config_tests_user_prompt_pack_keeps_json_like_markdown_literal() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let json_like_markdown = r#"{"shared_preference":"Keep this literal markdown."}"#;

    let pack = service
        .create_global_prompt_pack(
            "Json Looking Prompt",
            vec!["literal".to_string()],
            SceneProfile::General,
            "user",
            json_like_markdown,
            12_000,
        )
        .expect("prompt pack should create");

    assert_eq!(
        service
            .prompt_markdown_for_pack(&pack.prompt_pack_id)
            .expect("prompt markdown should load")
            .expect("prompt markdown should exist"),
        json_like_markdown
    );

    let prompt_file = service
        .storage_state_dir()
        .expect("storage state dir should resolve")
        .join("prompt-packs")
        .join("user")
        .join(&pack.prompt_pack_id)
        .join("PROMPT.md");
    assert_eq!(
        fs::read_to_string(prompt_file).expect("prompt markdown should read"),
        json_like_markdown
    );
}

#[test]
fn evaluation_config_tests_delete_user_prompt_pack_removes_files_and_project_references() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Prompt Delete Project")
        .expect("project should create");
    let pack = service
        .create_global_prompt_pack(
            "Delete Me",
            vec!["custom".to_string()],
            SceneProfile::General,
            "my-pack",
            "Temporary preference.",
            13_000,
        )
        .expect("prompt pack should create");
    let prompt_pack_dir = service
        .storage_state_dir()
        .expect("storage state dir should resolve")
        .join("prompt-packs")
        .join("my-pack")
        .join(&pack.prompt_pack_id);

    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.prompt_pack_id = Some(pack.prompt_pack_id.clone());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save selected prompt pack");

    let deleted = service
        .delete_global_prompt_pack(&pack.prompt_pack_id)
        .expect("prompt pack should delete");

    assert!(deleted);
    assert!(!prompt_pack_dir.exists());
    assert!(service
        .prompt_pack_by_id(&pack.prompt_pack_id)
        .expect("prompt pack lookup should succeed")
        .is_none());
    assert_eq!(
        service
            .project_evaluation_settings(&project.project_id)
            .expect("settings should reload")
            .expect("settings should exist")
            .prompt_pack_id,
        None
    );
}

#[test]
fn evaluation_config_tests_delete_built_in_prompt_pack_is_rejected() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));

    let error = service
        .delete_global_prompt_pack("general-default")
        .expect_err("built-in prompt pack should not delete")
        .to_string();

    assert!(error.contains("built-in prompt packs cannot be deleted"));
    assert!(service
        .prompt_pack_by_id("general-default")
        .expect("built-in prompt pack should still load")
        .is_some());
}

#[test]
fn evaluation_config_tests_delete_user_prompt_package_removes_all_packs_in_package() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Prompt Package Delete Project")
        .expect("project should create");
    let first = service
        .create_global_prompt_pack(
            "Package One",
            vec!["custom".to_string()],
            SceneProfile::General,
            "shareable-pack",
            "First preference.",
            14_000,
        )
        .expect("first prompt pack should create");
    let second = service
        .create_global_prompt_pack(
            "Package Two",
            vec!["custom".to_string()],
            SceneProfile::Portrait,
            "shareable-pack",
            "Second preference.",
            14_001,
        )
        .expect("second prompt pack should create");
    let other = service
        .create_global_prompt_pack(
            "Other Package",
            vec!["custom".to_string()],
            SceneProfile::General,
            "other-pack",
            "Other preference.",
            14_002,
        )
        .expect("other prompt pack should create");

    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.prompt_pack_id = Some(second.prompt_pack_id.clone());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save selected prompt pack");

    let deleted = service
        .delete_global_prompt_package("shareable-pack")
        .expect("prompt package should delete");

    assert!(deleted);
    assert!(service
        .prompt_pack_by_id(&first.prompt_pack_id)
        .expect("first lookup should succeed")
        .is_none());
    assert!(service
        .prompt_pack_by_id(&second.prompt_pack_id)
        .expect("second lookup should succeed")
        .is_none());
    assert!(service
        .prompt_pack_by_id(&other.prompt_pack_id)
        .expect("other lookup should succeed")
        .is_some());
    assert_eq!(
        service
            .project_evaluation_settings(&project.project_id)
            .expect("settings should reload")
            .expect("settings should exist")
            .prompt_pack_id,
        None
    );
    assert!(!service
        .storage_state_dir()
        .expect("storage state dir should resolve")
        .join("prompt-packs")
        .join("shareable-pack")
        .exists());
}

#[test]
fn evaluation_config_tests_delete_built_in_prompt_package_is_rejected() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));

    let error = service
        .delete_global_prompt_package("builtin")
        .expect_err("built-in prompt package should not delete")
        .to_string();

    assert!(error.contains("built-in prompt package cannot be deleted"));
}

#[test]
fn evaluation_config_tests_corrupt_user_prompt_pack_reports_load_error() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    service
        .create_global_prompt_pack(
            "Broken Pack",
            vec!["broken".to_string()],
            SceneProfile::General,
            "user",
            "This prompt will be corrupted.",
            11_000,
        )
        .expect("prompt pack should create");

    let manifest_file = fs::read_dir(
        service
            .storage_state_dir()
            .expect("storage state dir should resolve")
            .join("prompt-packs")
            .join("user"),
    )
    .expect("prompt pack root should exist")
    .filter_map(|entry| entry.ok())
    .map(|entry| entry.path().join("manifest.json"))
    .find(|path| path.exists())
    .expect("manifest should exist");
    fs::write(manifest_file, "{not valid json").expect("manifest should corrupt");

    let error = service
        .global_prompt_packs()
        .expect_err("corrupt user prompt pack should fail loudly")
        .to_string();
    assert!(!error.contains("prompt pack not found"));
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

    for variant in [ProjectRecommendationMode::Manual] {
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

#[test]
fn evaluation_config_tests_subject_assessment_save_query_round_trips_portrait_face_assessment() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Subject Project")
        .expect("project should create");
    let group_id = create_subject_asset_group(
        &store,
        &project.project_id,
        "ftp:subject-roundtrip",
        "DCIM/100/SUBJECT_0001.JPG",
    );
    let assessment = SubjectAssessment {
        assessment_id: "assessment-face-1".to_string(),
        project_id: project.project_id.clone(),
        asset_group_id: group_id.clone(),
        subject_type: "face".to_string(),
        detector_kind: "android_mlkit".to_string(),
        detector_version: "mlkit-face-v1".to_string(),
        status: EvaluationRunStatus::Ready,
        gate_status: "warn".to_string(),
        regions_json: "[{\"x\":10,\"y\":20,\"w\":80,\"h\":90}]".to_string(),
        signals_json: "{\"eyes\":\"open\",\"sharpness\":0.72}".to_string(),
        summary: "Face is usable with mild softness.".to_string(),
        created_at_ms: 4000,
        updated_at_ms: 4100,
    };

    store
        .save_subject_assessment(assessment.clone())
        .expect("assessment should save");
    let loaded = store
        .subject_assessments_for_asset_groups(&project.project_id, &[group_id])
        .expect("assessments should query");

    assert_eq!(loaded, vec![assessment]);
}

#[test]
fn evaluation_config_tests_subject_assessment_rejects_missing_or_wrong_project_asset_group() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let first_project = store
        .create_project("Subject First Project")
        .expect("first project should create");
    let second_project = store
        .create_project("Subject Second Project")
        .expect("second project should create");
    let first_group_id = create_subject_asset_group(
        &store,
        &first_project.project_id,
        "ftp:subject-wrong-project",
        "DCIM/100/SUBJECT_1001.JPG",
    );

    let missing_group = subject_assessment_for(
        &first_project.project_id,
        "missing-group",
        "assessment-missing-group",
    );
    let wrong_project = subject_assessment_for(
        &second_project.project_id,
        &first_group_id,
        "assessment-wrong-project",
    );

    assert!(store.save_subject_assessment(missing_group).is_err());
    assert!(store.save_subject_assessment(wrong_project).is_err());
}

#[test]
fn evaluation_config_tests_subject_assessment_id_cannot_move_between_asset_groups() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Subject Conflict Project")
        .expect("project should create");
    let first_group_id = create_subject_asset_group(
        &store,
        &project.project_id,
        "ftp:subject-conflict-a",
        "DCIM/100/SUBJECT_2001.JPG",
    );
    let second_group_id = create_subject_asset_group(
        &store,
        &project.project_id,
        "ftp:subject-conflict-b",
        "DCIM/101/SUBJECT_2002.JPG",
    );
    let first =
        subject_assessment_for(&project.project_id, &first_group_id, "assessment-stable-id");
    let moved = subject_assessment_for(
        &project.project_id,
        &second_group_id,
        "assessment-stable-id",
    );

    store
        .save_subject_assessment(first)
        .expect("first assessment should save");

    assert!(store.save_subject_assessment(moved).is_err());
}

#[test]
fn evaluation_config_tests_subject_assessment_requires_regions_array_and_signals_object() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Subject Json Project")
        .expect("project should create");
    let group_id = create_subject_asset_group(
        &store,
        &project.project_id,
        "ftp:subject-json",
        "DCIM/100/SUBJECT_3001.JPG",
    );

    let mut invalid_regions =
        subject_assessment_for(&project.project_id, &group_id, "assessment-invalid-regions");
    invalid_regions.regions_json = "{\"x\":10}".to_string();
    let mut invalid_signals =
        subject_assessment_for(&project.project_id, &group_id, "assessment-invalid-signals");
    invalid_signals.signals_json = "[\"closed_eyes\"]".to_string();
    let mut malformed_regions = subject_assessment_for(
        &project.project_id,
        &group_id,
        "assessment-malformed-regions",
    );
    malformed_regions.regions_json = "not-json".to_string();

    assert!(store.save_subject_assessment(invalid_regions).is_err());
    assert!(store.save_subject_assessment(invalid_signals).is_err());
    assert!(store.save_subject_assessment(malformed_regions).is_err());
}

#[test]
fn evaluation_config_tests_general_projects_do_not_schedule_portrait_subject_assessment() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("General Subject Scheduling")
        .expect("project should create");

    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:subject-general", "DCIM/100/GENERAL_0001.JPG", 1000),
        )
        .expect("transfer should record");
    let jobs = store
        .claim_analysis_jobs(i64::MAX, 10)
        .expect("jobs should claim");

    assert!(jobs
        .iter()
        .all(|job| job.job_type != AnalysisJobType::AssessPortraitSubject));
}

#[test]
fn evaluation_config_tests_portrait_projects_schedule_portrait_subject_assessment() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Portrait Subject Scheduling")
        .expect("project should create");
    let mut settings = base_project_settings(&project.project_id);
    settings.scene_profile = SceneProfile::Portrait;
    store
        .save_project_evaluation_settings(settings)
        .expect("settings should save");

    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:subject-portrait", "DCIM/100/PORTRAIT_0001.JPG", 1000),
        )
        .expect("transfer should record");
    let jobs = store
        .claim_analysis_jobs(i64::MAX, 10)
        .expect("jobs should claim");

    assert!(jobs
        .iter()
        .any(|job| job.job_type == AnalysisJobType::AssessPortraitSubject));
}

#[test]
fn evaluation_config_tests_service_reports_portrait_subject_assessment_schedule_condition() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Service Subject Scheduling")
        .expect("project should create");

    assert!(!service
        .should_schedule_subject_assessment(&project.project_id)
        .expect("schedule condition should load"));

    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should query")
        .expect("settings should exist");
    settings.scene_profile = SceneProfile::Portrait;
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");

    assert!(service
        .should_schedule_subject_assessment(&project.project_id)
        .expect("schedule condition should load"));
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

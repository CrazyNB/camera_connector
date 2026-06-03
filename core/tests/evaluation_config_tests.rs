use camera_connector_core::{
    AnalysisJobType, CameraConnectorConfig, CameraConnectorService, CvPolicy, EvaluationRun,
    EvaluationRunStatus, EvaluationRunTrigger, EvaluationRunType, ModelProviderKind,
    ModelProviderSettings, ModelSendMode, ProjectRecommendationMode, PromptProfile,
    PromptProfileVersion, PromptScope, SceneProfile, SqliteStore, StoredObjectLocation,
    SubjectAssessment, TransferRecord, TransferStatus,
};
use rusqlite::{params, Connection};

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

    for variant in [ModelSendMode::PreviewOnly, ModelSendMode::ReviewImage] {
        assert_eq!(ModelSendMode::from_str(variant.as_str()), variant);
    }
    assert_eq!(
        ModelSendMode::from_str("unexpected"),
        ModelSendMode::PreviewOnly
    );

    for variant in [PromptScope::Global, PromptScope::Project] {
        assert_eq!(PromptScope::from_str(variant.as_str()), variant);
    }
    assert_eq!(PromptScope::from_str("unexpected"), PromptScope::Global);

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
                default_send_mode: ModelSendMode::ReviewImage,
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
    assert_eq!(saved.base_url, "https://api.openai.com/v1");
    assert!(saved.api_key_configured);
    assert_eq!(config.model_provider.base_url, "https://api.openai.com/v1");
    assert_eq!(config.model_provider.api_key.as_deref(), Some("sk-test"));
}

#[test]
fn evaluation_config_tests_store_seeds_builtin_prompt_profiles_with_active_versions() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let profiles = store
        .prompt_profiles_for_project("project-missing")
        .expect("profiles should query");

    let ids = profiles
        .iter()
        .map(|profile| profile.prompt_profile_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        vec![
            "general-default",
            "portrait-conservative",
            "landscape-technical"
        ]
    );
    assert!(profiles
        .iter()
        .all(|profile| profile.scope == PromptScope::Global));
    assert!(profiles.iter().all(|profile| profile.built_in));
    assert!(profiles.iter().all(|profile| profile.enabled));
    assert_eq!(
        profile_style_tags(&profiles, "general-default"),
        vec!["general".to_string(), "balanced".to_string()]
    );
    assert_eq!(
        profile_style_tags(&profiles, "portrait-conservative"),
        vec!["portrait".to_string(), "conservative".to_string()]
    );
    assert_eq!(
        profile_style_tags(&profiles, "landscape-technical"),
        vec!["landscape".to_string(), "technical".to_string()]
    );

    for profile in profiles {
        let active_version_id = profile
            .active_version_id
            .as_deref()
            .expect("built-in profile should have active version");
        let version = store
            .prompt_profile_version(active_version_id)
            .expect("version should query")
            .expect("active version should exist");
        assert_eq!(version.prompt_profile_id, profile.prompt_profile_id);
        assert!(!version.prompt_text.is_empty());
        assert!(!version.prompt_hash.is_empty());
    }
}

#[test]
fn evaluation_config_tests_builtin_prompt_seed_repairs_missing_builtins_when_custom_rows_exist() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let db_path = temp_dir.path().join("state.sqlite");
    {
        let store = SqliteStore::open(&db_path).expect("store should open");
        let project = store
            .create_project("Custom Prompt Project")
            .expect("project should create");
        store
            .save_prompt_profile(PromptProfile {
                prompt_profile_id: "custom-only".to_string(),
                scope: PromptScope::Project,
                project_id: Some(project.project_id),
                name: "Custom Only".to_string(),
                style_tags: vec!["custom".to_string()],
                scene_profile: SceneProfile::Custom,
                active_version_id: None,
                built_in: false,
                enabled: true,
                created_at_ms: 10,
                updated_at_ms: 10,
            })
            .expect("custom profile should save");
    }
    {
        let connection = Connection::open(&db_path).expect("db should open");
        connection
            .execute("DELETE FROM prompt_profile_versions", [])
            .expect("versions should delete");
        connection
            .execute(
                "DELETE FROM prompt_profiles WHERE prompt_profile_id <> ?1",
                params!["custom-only"],
            )
            .expect("built-ins should delete");
    }

    let reopened = SqliteStore::open(&db_path).expect("store should reopen");
    let profiles = reopened
        .prompt_profiles_for_project("project-missing")
        .expect("profiles should query");
    let ids = profiles
        .iter()
        .map(|profile| profile.prompt_profile_id.as_str())
        .collect::<Vec<_>>();

    assert!(ids.contains(&"general-default"));
    assert!(ids.contains(&"portrait-conservative"));
    assert!(ids.contains(&"landscape-technical"));
}

fn profile_style_tags(profiles: &[PromptProfile], profile_id: &str) -> Vec<String> {
    profiles
        .iter()
        .find(|profile| profile.prompt_profile_id == profile_id)
        .expect("profile should be seeded")
        .style_tags
        .clone()
}

fn base_project_settings(project_id: &str) -> camera_connector_core::ProjectEvaluationSettings {
    camera_connector_core::ProjectEvaluationSettings {
        project_id: project_id.to_string(),
        model_evaluation_enabled: false,
        auto_evaluate_on_upload: false,
        auto_burst_recommendation_enabled: true,
        project_recommendation_mode: ProjectRecommendationMode::Manual,
        prompt_profile_id: None,
        scene_profile: SceneProfile::General,
        cv_policy: CvPolicy::Standard,
        allow_risky_model_selects: false,
        max_image_side: None,
        batch_size: None,
        updated_at_ms: 500,
    }
}

#[test]
fn evaluation_config_tests_new_project_gets_default_evaluation_settings_with_model_evaluation_disabled(
) {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Evaluation Defaults")
        .expect("project should create");

    let settings = store
        .project_evaluation_settings(&project.project_id)
        .expect("settings should query")
        .expect("settings should exist");

    assert_eq!(settings.project_id, project.project_id);
    assert!(!settings.model_evaluation_enabled);
    assert!(!settings.auto_evaluate_on_upload);
    assert!(settings.auto_burst_recommendation_enabled);
    assert_eq!(
        settings.project_recommendation_mode,
        ProjectRecommendationMode::Manual
    );
    assert_eq!(settings.prompt_profile_id, None);
    assert_eq!(settings.scene_profile, SceneProfile::General);
    assert_eq!(settings.cv_policy, CvPolicy::Standard);
    assert!(!settings.allow_risky_model_selects);
    assert_eq!(settings.max_image_side, None);
    assert_eq!(settings.batch_size, None);
}

#[test]
fn evaluation_config_tests_prompt_profile_and_version_save_query_preserves_prompt_text_and_hash() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Prompt Project")
        .expect("project should create");
    let profile = PromptProfile {
        prompt_profile_id: "project-prompt".to_string(),
        scope: PromptScope::Project,
        project_id: Some(project.project_id.clone()),
        name: "Project Prompt".to_string(),
        style_tags: vec!["portrait".to_string(), "soft-light".to_string()],
        scene_profile: SceneProfile::Portrait,
        active_version_id: None,
        built_in: false,
        enabled: true,
        created_at_ms: 2000,
        updated_at_ms: 2000,
    };
    let version = PromptProfileVersion {
        prompt_version_id: "project-prompt-v1".to_string(),
        prompt_profile_id: "project-prompt".to_string(),
        prompt_text: "Judge focus, expression, and skin tone conservatively.".to_string(),
        output_schema_version: "model-evaluation-v1".to_string(),
        prompt_hash: "hash-project-prompt-v1".to_string(),
        created_at_ms: 2100,
    };

    store
        .save_prompt_profile(profile)
        .expect("profile should save");
    store
        .save_prompt_profile_version(version.clone())
        .expect("version should save");

    let loaded = store
        .prompt_profile_version("project-prompt-v1")
        .expect("version should query")
        .expect("version should exist");
    assert_eq!(loaded, version);

    let mut loaded_profiles = store
        .prompt_profiles_for_project(&project.project_id)
        .expect("profiles should query");
    loaded_profiles.retain(|loaded| loaded.prompt_profile_id == "project-prompt");
    assert_eq!(loaded_profiles.len(), 1);
    assert_eq!(
        loaded_profiles[0].style_tags,
        vec!["portrait".to_string(), "soft-light".to_string()]
    );
}

#[test]
fn evaluation_config_tests_rejects_orphan_prompt_version_with_foreign_keys_enabled() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let version = PromptProfileVersion {
        prompt_version_id: "orphan-version".to_string(),
        prompt_profile_id: "missing-profile".to_string(),
        prompt_text: "This should not persist.".to_string(),
        output_schema_version: "model-evaluation-v1".to_string(),
        prompt_hash: "hash-orphan".to_string(),
        created_at_ms: 42,
    };

    assert!(store.save_prompt_profile_version(version).is_err());
    assert!(store
        .prompt_profile_version("orphan-version")
        .expect("version lookup should query")
        .is_none());
}

#[test]
fn evaluation_config_tests_rejects_inconsistent_prompt_profiles() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Prompt Validation Project")
        .expect("project should create");

    assert!(store
        .save_prompt_profile(PromptProfile {
            prompt_profile_id: "bad-global".to_string(),
            scope: PromptScope::Global,
            project_id: Some(project.project_id.clone()),
            name: "Bad Global".to_string(),
            style_tags: vec![],
            scene_profile: SceneProfile::General,
            active_version_id: None,
            built_in: false,
            enabled: true,
            created_at_ms: 100,
            updated_at_ms: 100,
        })
        .is_err());

    assert!(store
        .save_prompt_profile(PromptProfile {
            prompt_profile_id: "bad-project".to_string(),
            scope: PromptScope::Project,
            project_id: Some("missing-project".to_string()),
            name: "Bad Project".to_string(),
            style_tags: vec![],
            scene_profile: SceneProfile::General,
            active_version_id: None,
            built_in: false,
            enabled: true,
            created_at_ms: 100,
            updated_at_ms: 100,
        })
        .is_err());

    store
        .save_prompt_profile(PromptProfile {
            prompt_profile_id: "profile-a".to_string(),
            scope: PromptScope::Project,
            project_id: Some(project.project_id.clone()),
            name: "Profile A".to_string(),
            style_tags: vec![],
            scene_profile: SceneProfile::General,
            active_version_id: None,
            built_in: false,
            enabled: true,
            created_at_ms: 100,
            updated_at_ms: 100,
        })
        .expect("profile a should save");
    store
        .save_prompt_profile(PromptProfile {
            prompt_profile_id: "profile-b".to_string(),
            scope: PromptScope::Project,
            project_id: Some(project.project_id.clone()),
            name: "Profile B".to_string(),
            style_tags: vec![],
            scene_profile: SceneProfile::General,
            active_version_id: None,
            built_in: false,
            enabled: true,
            created_at_ms: 100,
            updated_at_ms: 100,
        })
        .expect("profile b should save");
    store
        .save_prompt_profile_version(PromptProfileVersion {
            prompt_version_id: "profile-b-v1".to_string(),
            prompt_profile_id: "profile-b".to_string(),
            prompt_text: "Profile B text".to_string(),
            output_schema_version: "model-evaluation-v1".to_string(),
            prompt_hash: "hash-profile-b".to_string(),
            created_at_ms: 110,
        })
        .expect("profile b version should save");

    assert!(store
        .save_prompt_profile(PromptProfile {
            prompt_profile_id: "profile-a".to_string(),
            scope: PromptScope::Project,
            project_id: Some(project.project_id),
            name: "Profile A".to_string(),
            style_tags: vec![],
            scene_profile: SceneProfile::General,
            active_version_id: Some("profile-b-v1".to_string()),
            built_in: false,
            enabled: true,
            created_at_ms: 100,
            updated_at_ms: 120,
        })
        .is_err());
}

#[test]
fn evaluation_config_tests_rejects_inconsistent_project_evaluation_settings() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let first_project = store
        .create_project("First Project")
        .expect("first project should create");
    let second_project = store
        .create_project("Second Project")
        .expect("second project should create");
    let second_profile = PromptProfile {
        prompt_profile_id: "second-project-profile".to_string(),
        scope: PromptScope::Project,
        project_id: Some(second_project.project_id.clone()),
        name: "Second Project Profile".to_string(),
        style_tags: vec![],
        scene_profile: SceneProfile::General,
        active_version_id: None,
        built_in: false,
        enabled: true,
        created_at_ms: 100,
        updated_at_ms: 100,
    };
    store
        .save_prompt_profile(second_profile)
        .expect("second project profile should save");

    let mut settings = base_project_settings(&first_project.project_id);
    settings.model_evaluation_enabled = true;
    settings.prompt_profile_id = None;
    assert!(store
        .save_project_evaluation_settings(settings.clone())
        .is_err());

    settings.model_evaluation_enabled = false;
    settings.prompt_profile_id = Some("missing-profile".to_string());
    assert!(store
        .save_project_evaluation_settings(settings.clone())
        .is_err());

    settings.prompt_profile_id = Some("second-project-profile".to_string());
    assert!(store
        .save_project_evaluation_settings(settings.clone())
        .is_err());

    settings.model_evaluation_enabled = true;
    settings.prompt_profile_id = Some("general-default".to_string());
    store
        .save_project_evaluation_settings(settings.clone())
        .expect("global prompt profile should be accepted");

    let first_profile = PromptProfile {
        prompt_profile_id: "first-project-profile".to_string(),
        scope: PromptScope::Project,
        project_id: Some(first_project.project_id.clone()),
        name: "First Project Profile".to_string(),
        style_tags: vec![],
        scene_profile: SceneProfile::General,
        active_version_id: None,
        built_in: false,
        enabled: true,
        created_at_ms: 100,
        updated_at_ms: 100,
    };
    store
        .save_prompt_profile(first_profile)
        .expect("first project profile should save");
    store
        .save_prompt_profile_version(PromptProfileVersion {
            prompt_version_id: "first-project-profile-v1".to_string(),
            prompt_profile_id: "first-project-profile".to_string(),
            prompt_text: "First project prompt text".to_string(),
            output_schema_version: "model-evaluation-v1".to_string(),
            prompt_hash: "hash-first-project".to_string(),
            created_at_ms: 110,
        })
        .expect("first project profile version should save");
    settings.prompt_profile_id = Some("first-project-profile".to_string());
    store
        .save_project_evaluation_settings(settings)
        .expect("same project prompt profile should be accepted");
}

#[test]
fn evaluation_config_tests_project_settings_require_usable_prompt_profiles() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Prompt Usability Project")
        .expect("project should create");

    store
        .save_prompt_profile(PromptProfile {
            prompt_profile_id: "disabled-project-profile".to_string(),
            scope: PromptScope::Project,
            project_id: Some(project.project_id.clone()),
            name: "Disabled Project Profile".to_string(),
            style_tags: vec![],
            scene_profile: SceneProfile::General,
            active_version_id: None,
            built_in: false,
            enabled: true,
            created_at_ms: 100,
            updated_at_ms: 100,
        })
        .expect("disabled profile should initially save");
    store
        .save_prompt_profile_version(PromptProfileVersion {
            prompt_version_id: "disabled-project-profile-v1".to_string(),
            prompt_profile_id: "disabled-project-profile".to_string(),
            prompt_text: "Disabled prompt text".to_string(),
            output_schema_version: "model-evaluation-v1".to_string(),
            prompt_hash: "hash-disabled".to_string(),
            created_at_ms: 110,
        })
        .expect("disabled profile version should save");
    store
        .save_prompt_profile(PromptProfile {
            prompt_profile_id: "disabled-project-profile".to_string(),
            scope: PromptScope::Project,
            project_id: Some(project.project_id.clone()),
            name: "Disabled Project Profile".to_string(),
            style_tags: vec![],
            scene_profile: SceneProfile::General,
            active_version_id: Some("disabled-project-profile-v1".to_string()),
            built_in: false,
            enabled: false,
            created_at_ms: 100,
            updated_at_ms: 120,
        })
        .expect("disabled profile should update");

    store
        .save_prompt_profile(PromptProfile {
            prompt_profile_id: "no-active-project-profile".to_string(),
            scope: PromptScope::Project,
            project_id: Some(project.project_id.clone()),
            name: "No Active Project Profile".to_string(),
            style_tags: vec![],
            scene_profile: SceneProfile::General,
            active_version_id: None,
            built_in: false,
            enabled: true,
            created_at_ms: 200,
            updated_at_ms: 200,
        })
        .expect("no-active profile should save");

    let mut settings = base_project_settings(&project.project_id);
    settings.model_evaluation_enabled = true;
    settings.prompt_profile_id = Some("disabled-project-profile".to_string());
    assert!(store
        .save_project_evaluation_settings(settings.clone())
        .is_err());

    settings.model_evaluation_enabled = false;
    assert!(store
        .save_project_evaluation_settings(settings.clone())
        .is_err());

    settings.model_evaluation_enabled = true;
    settings.prompt_profile_id = Some("no-active-project-profile".to_string());
    assert!(store
        .save_project_evaluation_settings(settings.clone())
        .is_err());

    settings.model_evaluation_enabled = false;
    settings.prompt_profile_id = None;
    store
        .save_project_evaluation_settings(settings.clone())
        .expect("disabled model evaluation may omit prompt");

    settings.model_evaluation_enabled = true;
    settings.prompt_profile_id = Some("general-default".to_string());
    store
        .save_project_evaluation_settings(settings.clone())
        .expect("enabled model evaluation accepts usable global prompt");

    let project_profile = PromptProfile {
        prompt_profile_id: "usable-project-profile".to_string(),
        scope: PromptScope::Project,
        project_id: Some(project.project_id.clone()),
        name: "Usable Project Profile".to_string(),
        style_tags: vec![],
        scene_profile: SceneProfile::General,
        active_version_id: None,
        built_in: false,
        enabled: true,
        created_at_ms: 300,
        updated_at_ms: 300,
    };
    store
        .save_prompt_profile(project_profile)
        .expect("usable profile should save");
    store
        .save_prompt_profile_version(PromptProfileVersion {
            prompt_version_id: "usable-project-profile-v1".to_string(),
            prompt_profile_id: "usable-project-profile".to_string(),
            prompt_text: "Usable prompt text".to_string(),
            output_schema_version: "model-evaluation-v1".to_string(),
            prompt_hash: "hash-usable".to_string(),
            created_at_ms: 310,
        })
        .expect("usable profile version should save");
    settings.prompt_profile_id = Some("usable-project-profile".to_string());
    store
        .save_project_evaluation_settings(settings)
        .expect("enabled model evaluation accepts usable project prompt");
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
        provider_model: "imported-reviewer-v1".to_string(),
        prompt_profile_id: Some("general-default".to_string()),
        prompt_version_id: Some("general-default-v1".to_string()),
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
        final_path: None,
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

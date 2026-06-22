use super::*;

#[test]
fn service_rejects_invalid_prompt_id_when_prompt_is_selected() {
    let (service, config_path, state_dir) = service_with_state_dir("service-settings-invalid");
    let project = service
        .create_project("Invalid Prompt Settings")
        .expect("project should create");
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.prompt_pack_id = Some("missing-prompt".to_string());
    settings.updated_at_ms = 5_000;

    assert!(service.save_project_evaluation_settings(settings).is_err());

    let mut without_prompt = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should reload")
        .expect("settings should exist");
    without_prompt.prompt_pack_id = None;
    without_prompt.updated_at_ms = 5_100;
    let saved = service
        .save_project_evaluation_settings(without_prompt)
        .expect("prompt pack may be omitted");
    assert_eq!(saved.prompt_pack_id, None);

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_saves_provider_and_project_settings_with_manual_recommendation_mode() {
    let (service, config_path, state_dir) = service_with_state_dir("service-settings-manual");
    let project = service
        .create_project("Manual Settings")
        .expect("project should create");

    let provider = service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "contains-no-secret-fields".to_string(),
            provider_kind: ModelProviderKind::OpenAi,
            provider_label: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-5.1-mini".to_string(),
            default_max_image_side: 1600,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 3,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 6_000,
        })
        .expect("provider settings should save");
    assert_eq!(provider.settings_id, "contains-no-secret-fields");
    assert_eq!(
        service
            .model_provider_settings()
            .expect("provider should load")
            .expect("provider should exist"),
        provider
    );

    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.project_recommendation_mode = ProjectRecommendationMode::Manual;
    settings.scene_profile = SceneProfile::Portrait;
    settings.cv_policy = CvPolicy::Strict;
    settings.updated_at_ms = 6_100;
    let saved = service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");

    assert_eq!(
        saved.project_recommendation_mode,
        ProjectRecommendationMode::Manual
    );
    assert_eq!(saved.scene_profile, SceneProfile::Portrait);
    assert_eq!(saved.cv_policy, CvPolicy::Strict);

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_project_evaluation_settings_can_override_technical_thresholds() {
    let (service, config_path, state_dir) =
        service_with_state_dir("service-cv-threshold-overrides");
    let project = service
        .create_project("CV Threshold Overrides")
        .expect("project should create");
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.cv_policy = CvPolicy::Standard;
    let mut policy = TechnicalAssessmentPolicy::standard();
    policy.clipping_high_ratio = 0.08;
    policy.clipping_high_connected_ratio = 0.08;
    settings.cv_policy_overrides = Some(policy);
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:threshold-overrides",
                "DCIM/100/IMG_THRESHOLD.JPG",
                1_000,
            ),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    let assessment = service
        .assess_asset_group_preview(
            &group_id,
            scattered_highlight_sample(100, 100, 900),
            "technical-v1",
        )
        .expect("assessment should save");

    assert_eq!(assessment.gate_status, TechnicalGateStatus::Warn);
    assert!(assessment
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::HighlightClip));

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

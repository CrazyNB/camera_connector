use camera_connector_core::{
    AssetGroupQuery, CameraConnectorService, ModelProviderKind, ModelProviderSettings,
    ModelSendMode, PublishTransferMetadata, StoredObjectLocation, TransferRecord, TransferStatus,
};
use camera_connector_ffi::{MobileCore, MobileReceiverSettingsPatch};
use serde_json::Value;

#[test]
fn mobile_core_saves_receiver_settings_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let output_dir = temp.path().join("output");
    let state_dir = temp.path().join("state");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let json = core
        .save_receiver_settings_json(MobileReceiverSettingsPatch {
            protocol: Some("sftp".to_string()),
            bind_host: Some("0.0.0.0".to_string()),
            ftp_port: Some(2121),
            sftp_port: Some(2222),
            output_dir: Some(output_dir.to_string_lossy().into_owned()),
            state_dir: Some(state_dir.to_string_lossy().into_owned()),
            advertised_host: Some("192.168.137.1".to_string()),
            source_name: Some("Studio Camera".to_string()),
            defer_publish: Some(true),
        })
        .unwrap();

    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["protocol"], "Sftp");
    assert_eq!(value["bind_host"], "0.0.0.0");
    assert_eq!(value["ftp_port"], 2121);
    assert_eq!(value["sftp_port"], 2222);
    assert_eq!(value["source_name"], "Studio Camera");
    assert_eq!(value["defer_publish"], true);
}

#[test]
fn mobile_core_persists_user_marks_and_filters_asset_groups_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Client Selects").unwrap();
    let project_id = project.project_id.clone();

    service
        .record_project_transfer(
            &project_id,
            completed_transfer("ftp:favorite", "DCIM/100/KEEP_0001.JPG", 10),
        )
        .expect("transfer should record");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project_id.clone(), "{}".to_string(), 0, 25)
            .expect("asset page should query"),
    )
    .unwrap();
    let group_id = page["groups"][0]["group_id"].as_str().unwrap().to_string();

    let marks: Value = serde_json::from_str(
        &core
            .set_asset_group_user_marks_json(
                project_id.clone(),
                group_id.clone(),
                r#"{"favorite":true}"#.to_string(),
            )
            .expect("user mark should save"),
    )
    .unwrap();
    assert_eq!(marks["favorite"], true);
    assert_eq!(marks["marked"], false);

    let favorites: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project_id, r#"{"favorite":true}"#.to_string(), 0, 25)
            .expect("favorite page should query"),
    )
    .unwrap();

    assert_eq!(favorites["total_groups"], 1);
    assert_eq!(favorites["groups"][0]["group_id"], group_id);
    assert_eq!(favorites["groups"][0]["user_marks"]["favorite"], true);
}

#[test]
fn mobile_core_asset_group_json_exposes_model_evaluator_kind() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Model Source Json").unwrap();
    service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "global".to_string(),
            provider_kind: ModelProviderKind::Imported,
            provider_label: "Imported".to_string(),
            base_url: "local://imported".to_string(),
            default_model: "model-stub-v1".to_string(),
            default_max_image_side: 1024,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 2,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 1000,
        })
        .unwrap();
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .unwrap()
        .unwrap();
    settings.auto_evaluate_on_upload = true;
    settings.prompt_profile_id = Some("general-default".to_string());
    settings.model_provider_settings_id = Some("global".to_string());
    service.save_project_evaluation_settings(settings).unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:model-source", "DCIM/100/IMG_3001.JPG", 10),
        )
        .unwrap();

    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let initial_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let group_id = initial_page["groups"][0]["group_id"]
        .as_str()
        .unwrap()
        .to_string();
    core.assess_asset_group_preview_json(
        group_id,
        balanced_detail_sample_json(16, 16),
        "technical-v1".to_string(),
    )
    .unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id, "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(page["groups"][0]["model_status"], "ready");
    assert_eq!(page["groups"][0]["model_evaluator_kind"], "local_stub");
}

#[test]
fn mobile_core_round_trips_model_provider_settings_without_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let missing: Value =
        serde_json::from_str(&core.model_provider_settings_json().unwrap()).unwrap();
    assert_eq!(missing["configured"], false);
    assert!(!missing.to_string().contains("\"api_key\":"));
    assert!(!missing.to_string().contains("secret"));

    let saved: Value = serde_json::from_str(
        &core
            .save_model_provider_settings_json(
                r#"{
                    "settings_id":"photo-eval-model",
                    "provider_kind":"openai",
                    "provider_label":"OpenAI",
                    "base_url":"https://api.openai.com/v1",
                    "default_model":"gpt-5.1-mini",
                    "default_max_image_side":1536,
                    "default_send_mode":"detail_image",
                    "default_batch_size":4,
                    "configured":true,
                    "api_key":"must-not-round-trip"
                }"#
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let loaded: Value =
        serde_json::from_str(&core.model_provider_settings_json().unwrap()).unwrap();

    assert_eq!(saved["provider_kind"], "openai");
    assert_eq!(saved["settings_id"], "photo-eval-model");
    assert_eq!(saved["base_url"], "https://api.openai.com/v1");
    assert_eq!(saved["api_key_configured"], true);
    assert_eq!(saved["default_send_mode"], "detail_image");
    assert_eq!(loaded["configured"], true);
    assert_eq!(loaded["settings_id"], "photo-eval-model");
    assert_eq!(loaded["base_url"], "https://api.openai.com/v1");
    assert_eq!(loaded["api_key_configured"], true);
    assert_eq!(loaded["default_batch_size"], 4);
    assert!(!saved.to_string().contains("must-not-round-trip"));
    assert!(!loaded.to_string().contains("\"api_key\":"));

    let list: Value =
        serde_json::from_str(&core.model_provider_settings_list_json().unwrap()).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["settings_id"], "photo-eval-model");

    core.delete_model_provider_settings_json("photo-eval-model".to_string())
        .expect("settings should delete");
    let list_after_delete: Value =
        serde_json::from_str(&core.model_provider_settings_list_json().unwrap()).unwrap();
    assert!(list_after_delete.as_array().unwrap().is_empty());
}

#[test]
fn mobile_core_round_trips_project_evaluation_settings_and_manual_mode() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Evaluation Settings").unwrap();
    let project_id = project.project_id.clone();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let default_settings: Value = serde_json::from_str(
        &core
            .project_evaluation_settings_json(project_id.clone())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(default_settings["project_recommendation_mode"], "manual");
    assert!(default_settings["cv_policy_overrides"].is_null());

    let saved: Value = serde_json::from_str(
        &core
            .save_project_evaluation_settings_json(
                project_id.clone(),
                r#"{
                    "auto_evaluate_on_upload":true,
                    "auto_burst_recommendation_enabled":false,
                    "project_recommendation_mode":"manual",
                    "prompt_profile_id":null,
                    "model_provider_settings_id":"photo-eval-model",
                    "scene_profile":"portrait",
                    "cv_policy":"strict",
                    "allow_risky_model_selects":true,
                    "max_image_side":2048,
                    "batch_size":8,
                    "cv_policy_overrides":{
                        "blur_severe_edge_threshold":0.06,
                        "blur_severe_frequency_threshold":0.06,
                        "blur_high_edge_threshold":0.16,
                        "blur_high_frequency_threshold":0.16,
                        "highlight_clip_threshold":242,
                        "shadow_clip_threshold":13,
                        "clipping_high_ratio":0.09,
                        "clipping_high_connected_ratio":0.14,
                        "clipping_severe_ratio":0.40,
                        "clipping_severe_connected_ratio":0.40,
                        "color_cast_high_threshold":0.32,
                        "color_cast_severe_threshold":0.55,
                        "face_eye_open_warn_threshold":0.45,
                        "face_exposure_warn_ratio":0.16,
                        "face_color_cast_warn_threshold":0.32
                    }
                }"#
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let loaded: Value = serde_json::from_str(
        &core
            .project_evaluation_settings_json(project_id.clone())
            .unwrap(),
    )
    .unwrap();

    assert_eq!(saved["project_recommendation_mode"], "manual");
    assert_eq!(saved["model_provider_settings_id"], "photo-eval-model");
    assert_eq!(loaded["scene_profile"], "portrait");
    assert_eq!(loaded["model_provider_settings_id"], "photo-eval-model");
    assert_eq!(loaded["cv_policy"], "strict");
    assert_eq!(loaded["max_image_side"], 2048);
    assert_eq!(loaded["batch_size"], 8);
    assert_eq!(
        loaded["cv_policy_overrides"]["clipping_high_ratio"],
        serde_json::json!(0.09)
    );
    assert_eq!(
        loaded["cv_policy_overrides"]["clipping_high_connected_ratio"],
        serde_json::json!(0.14)
    );
    assert_eq!(
        loaded["cv_policy_overrides"]["color_cast_high_threshold"],
        serde_json::json!(0.32)
    );
    assert_eq!(
        loaded["cv_policy_overrides"]["face_eye_open_warn_threshold"],
        serde_json::json!(0.45)
    );
    assert_eq!(
        loaded["cv_policy_overrides"]["face_exposure_warn_ratio"],
        serde_json::json!(0.16)
    );
    assert_eq!(
        loaded["cv_policy_overrides"]["face_color_cast_warn_threshold"],
        serde_json::json!(0.32)
    );

    let preserved: Value = serde_json::from_str(
        &core
            .save_project_evaluation_settings_json(
                project_id.clone(),
                r#"{
                    "cv_policy":"loose"
                }"#
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(preserved["cv_policy"], "loose");
    assert_eq!(
        preserved["cv_policy_overrides"]["clipping_high_ratio"],
        serde_json::json!(0.09)
    );
    assert_eq!(
        preserved["cv_policy_overrides"]["color_cast_high_threshold"],
        serde_json::json!(0.32)
    );

    let cleared: Value = serde_json::from_str(
        &core
            .save_project_evaluation_settings_json(
                project_id,
                r#"{
                    "cv_policy_overrides":null
                }"#
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert!(cleared["cv_policy_overrides"].is_null());
}

#[test]
fn mobile_core_lists_forks_and_versions_prompt_profiles_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Prompt Profiles").unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let profiles: Value = serde_json::from_str(
        &core
            .prompt_profiles_for_project_json(project.project_id.clone())
            .unwrap(),
    )
    .unwrap();
    let built_in = profiles
        .as_array()
        .unwrap()
        .iter()
        .find(|profile| profile["prompt_profile_id"] == "general-default")
        .unwrap();
    assert_eq!(built_in["scope"], "global");
    assert_eq!(built_in["built_in"], true);
    assert!(built_in["style_tags"]
        .as_array()
        .unwrap()
        .contains(&Value::String("general".to_string())));

    let forked: Value = serde_json::from_str(
        &core
            .fork_prompt_profile_json(
                project.project_id.clone(),
                "general-default".to_string(),
                "Client Editorial".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(forked["scope"], "project");
    assert_eq!(forked["project_id"], project.project_id);
    assert_eq!(forked["built_in"], false);

    let edited: Value = serde_json::from_str(
        &core
            .save_prompt_version_json(
                project.project_id,
                forked["prompt_profile_id"].as_str().unwrap().to_string(),
                "Return concise project recommendations.".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        edited["prompt_profile_id"],
        forked["prompt_profile_id"].as_str().unwrap()
    );
    assert!(edited["prompt_version_id"].as_str().unwrap().contains("-v"));
}

#[test]
fn mobile_core_rejects_invalid_settings_enum_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Invalid Enums").unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let provider_error = core
        .save_model_provider_settings_json(
            r#"{
                "provider_kind":"OpenAI",
                "default_send_mode":"preview_only",
                "configured":true
            }"#
            .to_string(),
        )
        .unwrap_err()
        .to_string();
    assert!(provider_error.contains("invalid provider_kind: OpenAI"));

    let send_mode_error = core
        .save_model_provider_settings_json(
            r#"{
                "provider_kind":"openai",
                "default_send_mode":"detailImage",
                "configured":true
            }"#
            .to_string(),
        )
        .unwrap_err()
        .to_string();
    assert!(send_mode_error.contains("invalid default_send_mode: detailImage"));

    for (field, value) in [
        ("scene_profile", "Portrait"),
        ("cv_policy", "Standard"),
        ("project_recommendation_mode", "automatic"),
    ] {
        let patch = serde_json::json!({
            "auto_evaluate_on_upload": false,
            "auto_burst_recommendation_enabled": true,
            "project_recommendation_mode": "manual",
            "prompt_profile_id": null,
            "scene_profile": "general",
            "cv_policy": "standard",
            "allow_risky_model_selects": false,
            field: value
        });
        let error = core
            .save_project_evaluation_settings_json(project.project_id.clone(), patch.to_string())
            .unwrap_err()
            .to_string();
        assert!(error.contains(&format!("invalid {field}: {value}")));
    }
}

#[test]
fn mobile_core_uses_monotonic_action_timestamps_for_prompt_edits() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Monotonic Prompts").unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let first: Value = serde_json::from_str(
        &core
            .fork_prompt_profile_json(
                project.project_id.clone(),
                "general-default".to_string(),
                "Editable A".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let second: Value = serde_json::from_str(
        &core
            .fork_prompt_profile_json(
                project.project_id.clone(),
                "general-default".to_string(),
                "Editable B".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_ne!(first["prompt_profile_id"], second["prompt_profile_id"]);

    let first_version: Value = serde_json::from_str(
        &core
            .save_prompt_version_json(
                project.project_id.clone(),
                first["prompt_profile_id"].as_str().unwrap().to_string(),
                "Prompt version one".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let second_version: Value = serde_json::from_str(
        &core
            .save_prompt_version_json(
                project.project_id,
                first["prompt_profile_id"].as_str().unwrap().to_string(),
                "Prompt version one".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_ne!(
        first_version["prompt_version_id"],
        second_version["prompt_version_id"]
    );
}

#[test]
fn mobile_core_generates_manual_project_recommendation_and_latest_run_status() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Manual Recommendation").unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let missing = core
        .generate_project_recommendation_json(project.project_id.clone())
        .unwrap_err()
        .to_string();
    assert!(missing.contains("model provider settings not configured"));

    service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "global".to_string(),
            provider_kind: ModelProviderKind::Imported,
            provider_label: "Imported".to_string(),
            base_url: "local://imported".to_string(),
            default_model: "model-stub-v1".to_string(),
            default_max_image_side: 1024,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 2,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 1000,
        })
        .unwrap();
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .unwrap()
        .unwrap();
    settings.model_provider_settings_id = Some("global".to_string());
    service.save_project_evaluation_settings(settings).unwrap();

    let recommendation: Value = serde_json::from_str(
        &core
            .generate_project_recommendation_json(project.project_id.clone())
            .unwrap(),
    )
    .unwrap();
    let run_status: Value = serde_json::from_str(
        &core
            .latest_project_recommendation_run_status_json(project.project_id)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(recommendation["scope"], "project");
    assert_eq!(recommendation["source"], "imported");
    assert!(recommendation["run_id"].as_str().is_some());
    assert_eq!(run_status["run_type"], "project_recommendation");
    assert_eq!(run_status["trigger"], "manual");
    assert_eq!(run_status["status"], "ready");
}

#[test]
fn mobile_core_generates_project_recommendation_with_candidate_visuals_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Manual Visual Project Recommendation")
        .unwrap();
    service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "global".to_string(),
            provider_kind: ModelProviderKind::Imported,
            provider_label: "Imported".to_string(),
            base_url: "local://imported".to_string(),
            default_model: "model-stub-v1".to_string(),
            default_max_image_side: 1024,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 2,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 1000,
        })
        .unwrap();
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .unwrap()
        .unwrap();
    settings.prompt_profile_id = Some("general-default".to_string());
    settings.model_provider_settings_id = Some("global".to_string());
    service.save_project_evaluation_settings(settings).unwrap();
    for (transfer_id, path, completed_at_ms) in [
        (
            "ftp:project-visual-mobile-1",
            "DCIM/100/PICK_0001.JPG",
            1000,
        ),
        (
            "ftp:project-visual-mobile-2",
            "DCIM/101/PICK_0002.JPG",
            2000,
        ),
    ] {
        service
            .record_project_transfer(
                &project.project_id,
                completed_transfer(transfer_id, path, completed_at_ms),
            )
            .unwrap();
    }
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let group_ids = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|group| group["group_id"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    let inputs = group_ids
        .iter()
        .map(|group_id| {
            serde_json::json!({
                "asset_group_id": group_id,
                "sample": serde_json::from_str::<Value>(&balanced_detail_sample_json(16, 16)).unwrap(),
            })
        })
        .collect::<Vec<_>>();
    core.evaluate_asset_groups_with_model_inputs_json(
        serde_json::json!({
            "project_id": project.project_id,
            "inputs": inputs,
        })
        .to_string(),
    )
    .unwrap();

    let recommendation: Value = serde_json::from_str(
        &core
            .generate_project_recommendation_with_candidate_visuals_json(
                serde_json::json!({
                    "project_id": project.project_id,
                    "candidate_visuals": group_ids.iter().enumerate().map(|(index, group_id)| {
                        serde_json::json!({
                            "asset_group_id": group_id,
                            "image_data_url": format!("data:image/jpeg;base64,cHJvamVjdC1tb2JpbGUt{index}"),
                        })
                    }).collect::<Vec<_>>(),
                })
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();

    assert_eq!(recommendation["scope"], "project");
    assert_eq!(recommendation["project_id"], project.project_id);
    assert_eq!(recommendation["status"], "ready");
}

#[test]
fn mobile_core_round_trips_subject_assessment_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Subject Assessment Json").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:subject-json", "DCIM/100/SUBJECT_0001.JPG", 1000),
        )
        .unwrap();
    let group_id = service
        .storage_store()
        .unwrap()
        .stored_asset_groups(&project.project_id)
        .unwrap()
        .into_iter()
        .find(|group| group.display_key == "SUBJECT_0001")
        .unwrap()
        .group_id;
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let default_should_schedule: Value = serde_json::from_str(
        &core
            .should_schedule_subject_assessment_json(project.project_id.clone())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(default_should_schedule, Value::Bool(false));

    core.save_project_evaluation_settings_json(
        project.project_id.clone(),
        r#"{
            "auto_evaluate_on_upload":false,
            "auto_burst_recommendation_enabled":true,
            "project_recommendation_mode":"manual",
            "prompt_profile_id":null,
            "scene_profile":"portrait",
            "cv_policy":"standard",
            "allow_risky_model_selects":false
        }"#
        .to_string(),
    )
    .unwrap();
    let portrait_should_schedule: Value = serde_json::from_str(
        &core
            .should_schedule_subject_assessment_json(project.project_id.clone())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(portrait_should_schedule, Value::Bool(true));

    let saved: Value = serde_json::from_str(
        &core
            .save_subject_assessment_json(
                serde_json::json!({
                    "assessment_id": "subject-assessment-1",
                    "project_id": project.project_id,
                    "asset_group_id": group_id.clone(),
                    "subject_type": "face",
                    "detector_kind": "imported",
                    "detector_version": "imported-face-v1",
                    "status": "ready",
                    "gate_status": "inconclusive",
                    "regions": [{"x": 10, "y": 20, "w": 80, "h": 90}],
                    "signals": {"closed_eyes": false, "face_sharpness": 0.72},
                    "summary": "Imported face assessment is usable.",
                    "created_at_ms": 10,
                    "updated_at_ms": 11
                })
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let loaded: Value = serde_json::from_str(
        &core
            .subject_assessments_for_asset_groups_json(
                project.project_id,
                serde_json::json!([group_id]).to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let loaded_item = &loaded.as_array().unwrap()[0];

    assert_eq!(saved["subject_type"], "face");
    assert_eq!(saved["detector_kind"], "imported");
    assert_eq!(saved["status"], "ready");
    assert_eq!(saved["gate_status"], "inconclusive");
    assert_eq!(saved["regions"][0]["w"], 80);
    assert_eq!(saved["signals"]["face_sharpness"], 0.72);
    assert_eq!(saved["summary"], "Imported face assessment is usable.");
    assert_eq!(loaded_item, &saved);
}

#[test]
fn mobile_core_rejects_ftps_protocol() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let error = core
        .save_receiver_settings_json(MobileReceiverSettingsPatch {
            protocol: Some("ftps".to_string()),
            ..MobileReceiverSettingsPatch::default()
        })
        .unwrap_err()
        .to_string();

    assert!(error.contains("invalid protocol: ftps"));
}

#[test]
fn mobile_core_saves_account_without_plaintext_password() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let json = core
        .save_device_account_json(
            "camera01".to_string(),
            Some("secret".to_string()),
            "Camera 01".to_string(),
        )
        .unwrap();

    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["username"], "camera01");
    assert_eq!(value["device_name"], "Camera 01");
    assert_eq!(value["password_configured"], true);
    assert!(!json.contains("secret"));
}

#[test]
fn mobile_core_removes_account_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.save_device_account_json(
        "camera01".to_string(),
        Some("secret".to_string()),
        "Camera 01".to_string(),
    )
    .unwrap();

    let json = core
        .remove_device_account_json("camera01".to_string())
        .unwrap();

    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["username"], "camera01");
    assert_eq!(value["removed"], true);
    let project: Value = serde_json::from_str(
        &core
            .create_project_json("Account Dashboard".to_string())
            .unwrap(),
    )
    .unwrap();
    let project_id = project["project_id"].as_str().unwrap().to_string();
    core.set_active_project_json(project_id.clone()).unwrap();
    let dashboard: Value =
        serde_json::from_str(&core.project_dashboard_json(project_id, 0, 25).unwrap()).unwrap();
    assert_eq!(dashboard["accounts"].as_array().unwrap().len(), 0);
}

#[test]
fn mobile_core_manages_projects_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let initial_active: Value = serde_json::from_str(&core.active_project_json().unwrap()).unwrap();
    assert!(initial_active.is_null());

    let created_json = core
        .create_project_json("Wedding Shoot".to_string())
        .unwrap();
    let created: Value = serde_json::from_str(&created_json).unwrap();
    assert_eq!(created["name"], "Wedding Shoot");
    assert_eq!(created["slug"], "wedding-shoot");
    assert_eq!(created["status"], "Active");
    let project_id = created["project_id"].as_str().unwrap().to_string();

    let listed_json = core.list_projects_json().unwrap();
    let listed: Value = serde_json::from_str(&listed_json).unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["project_id"], project_id);

    let active_json = core.set_active_project_json(project_id.clone()).unwrap();
    let active: Value = serde_json::from_str(&active_json).unwrap();
    assert_eq!(active["project_id"], project_id);

    let active_again: Value = serde_json::from_str(&core.active_project_json().unwrap()).unwrap();
    assert_eq!(active_again["project_id"], project_id);

    let archived_json = core.archive_project_json(project_id.clone()).unwrap();
    let archived: Value = serde_json::from_str(&archived_json).unwrap();
    assert_eq!(archived["project_id"], project_id);
    assert_eq!(archived["status"], "Archived");
    let active_after_archive: Value =
        serde_json::from_str(&core.active_project_json().unwrap()).unwrap();
    assert!(active_after_archive.is_null());
    assert!(core.set_active_project_json(project_id.clone()).is_err());

    let restored_json = core.restore_project_json(project_id.clone()).unwrap();
    let restored: Value = serde_json::from_str(&restored_json).unwrap();
    assert_eq!(restored["project_id"], project_id);
    assert_eq!(restored["status"], "Active");
    let active_after_restore: Value =
        serde_json::from_str(&core.set_active_project_json(project_id.clone()).unwrap()).unwrap();
    assert_eq!(active_after_restore["project_id"], project_id);
}

#[test]
fn mobile_core_exposes_project_capabilities_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let created: Value = serde_json::from_str(
        &core
            .create_project_json("Client Capability Shoot".to_string())
            .unwrap(),
    )
    .unwrap();
    let archived: Value = serde_json::from_str(
        &core
            .archive_project_json(created["project_id"].as_str().unwrap().to_string())
            .unwrap(),
    )
    .unwrap();

    assert_eq!(created["kind"], "User");
    assert_eq!(created["capabilities"]["can_archive"], true);
    assert_eq!(created["capabilities"]["can_rename"], true);
    assert_eq!(created["capabilities"]["can_accept_moved_groups"], true);
    assert_eq!(archived["capabilities"]["can_be_active_project"], false);
    assert_eq!(archived["capabilities"]["can_archive"], false);
    assert_eq!(archived["capabilities"]["can_restore"], true);
}

#[test]
fn mobile_core_renames_project_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let created: Value = serde_json::from_str(
        &core
            .create_project_json("Untitled Mobile Shoot".into())
            .unwrap(),
    )
    .unwrap();
    let project_id = created["project_id"].as_str().unwrap().to_string();
    core.set_active_project_json(project_id.clone()).unwrap();

    let renamed_json = core
        .rename_project_json(project_id.clone(), "Client Mobile Shoot".into())
        .unwrap();
    let active_json = core.active_project_json().unwrap();

    let renamed: Value = serde_json::from_str(&renamed_json).unwrap();
    let active: Value = serde_json::from_str(&active_json).unwrap();
    assert_eq!(renamed["project_id"], project_id);
    assert_eq!(renamed["name"], "Client Mobile Shoot");
    assert_eq!(renamed["slug"], "client-mobile-shoot");
    assert_eq!(active["project_id"], project_id);
    assert_eq!(active["name"], "Client Mobile Shoot");
}

#[test]
fn mobile_core_returns_project_dashboard_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let created: Value = serde_json::from_str(
        &core
            .create_project_json("Catalog Session".to_string())
            .unwrap(),
    )
    .unwrap();
    let project_id = created["project_id"].as_str().unwrap();

    let dashboard_json = core
        .project_dashboard_json(project_id.to_string(), 0, 50)
        .unwrap();

    let dashboard: Value = serde_json::from_str(&dashboard_json).unwrap();
    assert_eq!(dashboard["assets"]["total_groups"], 0);
    assert_eq!(dashboard["assets"]["limit"], 50);
    assert_eq!(dashboard["transfers"]["total_count"], 0);
    assert!(dashboard["paths"]["state_dir"]
        .as_str()
        .unwrap()
        .contains("state"));
}

#[test]
fn mobile_core_claims_and_completes_publish_queue_items_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Publisher").unwrap();
    let store = service.storage_store().unwrap();
    let item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:mobile-publish",
            "staging/mobile-publish.tmp",
            "IMG_6001.JPG",
            42,
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let claimed_json = core.claim_next_publish_item_json().unwrap();
    let claimed: Value = serde_json::from_str(&claimed_json).unwrap();
    assert_eq!(claimed["queue_id"], item.queue_id);
    assert_eq!(claimed["state"], "Publishing");
    assert_eq!(claimed["last_error"], Value::Null);
    let empty_after_claim: Value =
        serde_json::from_str(&core.claim_next_publish_item_json().unwrap()).unwrap();
    assert!(empty_after_claim.is_null());

    let completed_json = core
        .mark_publish_completed_json(item.queue_id.clone())
        .unwrap();
    let completed: Value = serde_json::from_str(&completed_json).unwrap();
    assert_eq!(completed["queue_id"], item.queue_id);
    assert_eq!(completed["completed"], true);
    let failed_item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:mobile-failed",
            "staging/mobile-failed.tmp",
            "IMG_6002.JPG",
            43,
        )
        .unwrap();
    let claimed_failed: Value =
        serde_json::from_str(&core.claim_next_publish_item_json().unwrap()).unwrap();
    assert_eq!(claimed_failed["queue_id"], failed_item.queue_id);
    let failed_json = core
        .mark_publish_failed_json(
            failed_item.queue_id.clone(),
            "permission revoked".to_string(),
        )
        .unwrap();
    let failed: Value = serde_json::from_str(&failed_json).unwrap();
    assert_eq!(failed["queue_id"], failed_item.queue_id);
    assert_eq!(failed["failed"], true);
    let dashboard: Value = serde_json::from_str(
        &core
            .project_dashboard_json(project.project_id.clone(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(dashboard["publish_queue"]["completed_count"], 1);
    assert_eq!(dashboard["publish_queue"]["failed_count"], 1);
    assert_eq!(dashboard["publish_queue"]["pending_count"], 1);
}

#[test]
fn mobile_core_releases_failed_publish_retries_for_project() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Retry").unwrap();
    let store = service.storage_store().unwrap();
    let item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:mobile-retry",
            "staging/mobile-retry.tmp",
            "IMG_6003.JPG",
            44,
        )
        .unwrap();
    store
        .mark_publish_failed(&item.queue_id, "permission revoked")
        .unwrap();

    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let deferred: Value =
        serde_json::from_str(&core.claim_next_publish_item_json().unwrap()).unwrap();
    assert!(deferred.is_null());

    let released: Value = serde_json::from_str(
        &core
            .release_failed_publish_retries_json(project.project_id.clone())
            .unwrap(),
    )
    .unwrap();
    let claimed: Value =
        serde_json::from_str(&core.claim_next_publish_item_json().unwrap()).unwrap();

    assert_eq!(released["project_id"], project.project_id);
    assert_eq!(released["released_count"], 1);
    assert_eq!(claimed["queue_id"], item.queue_id);
    assert_eq!(claimed["state"], "Publishing");
    assert_eq!(claimed["last_error"], Value::Null);
    assert_eq!(claimed["next_attempt_at_ms"], Value::Null);
}

#[test]
fn mobile_core_completes_publish_with_platform_location_into_project_assets() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Platform Publisher").unwrap();
    let store = service.storage_store().unwrap();
    let item = store
        .enqueue_publish_with_metadata(
            &project.project_id,
            "ftp:mobile-platform-publish",
            "staging/mobile-platform-publish.tmp",
            "IMG_6010.JPG",
            42,
            PublishTransferMetadata {
                protocol: "ftp".to_string(),
                original_path: "DCIM/100/IMG_6010.JPG".to_string(),
                username: Some("z5".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Studio Z5".to_string()),
                started_at_ms: 42,
            },
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let claimed: Value =
        serde_json::from_str(&core.claim_next_publish_item_json().unwrap()).unwrap();
    assert_eq!(claimed["queue_id"], item.queue_id);

    let completed_json = core
        .complete_publish_json(
            item.queue_id.clone(),
            "IMG_6010.JPG".to_string(),
            "media_uri".to_string(),
            "content://media/external/images/media/6010".to_string(),
        )
        .unwrap();
    let completed: Value = serde_json::from_str(&completed_json).unwrap();
    assert_eq!(completed["transfer_id"], "ftp:mobile-platform-publish");
    assert_eq!(completed["final_location"]["kind"], "media_uri");

    let dashboard: Value = serde_json::from_str(
        &core
            .project_dashboard_json(project.project_id.clone(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(dashboard["assets"]["total_groups"], 1);
    assert_eq!(
        dashboard["assets"]["groups"][0]["primary"]["storage_location"]["kind"],
        "media_uri"
    );
    assert_eq!(dashboard["publish_queue"]["completed_count"], 1);
}

#[test]
fn mobile_core_returns_project_group_assets_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Members").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-jpg", "DCIM/100/IMG_5001.JPG", 20),
        )
        .unwrap();
    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .unwrap();
    let group_id = page.groups[0].group_id.clone().unwrap();

    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let json = core
        .project_group_assets_json(project.project_id.clone(), group_id.clone())
        .unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let assets = value.as_array().unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0]["project_id"], project.project_id);
    assert_eq!(assets[0]["group_id"], group_id);
    assert_eq!(assets[0]["final_filename"], "IMG_5001.JPG");

    let ambiguous_json = core
        .project_group_assets_json(project.project_id, "IMG_5001".to_string())
        .unwrap();
    let ambiguous: Value = serde_json::from_str(&ambiguous_json).unwrap();
    assert!(ambiguous.as_array().unwrap().is_empty());
}

#[test]
fn mobile_core_moves_project_group_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let source_project = service.create_project("Wrong Mobile Project").unwrap();
    let target_project = service.create_project("Correct Mobile Project").unwrap();
    service
        .record_project_transfer(
            &source_project.project_id,
            completed_transfer("ftp:mobile-move-jpg", "DCIM/100/IMG_6101.JPG", 20),
        )
        .unwrap();
    service
        .record_project_transfer(
            &source_project.project_id,
            completed_transfer("ftp:mobile-move-raw", "DCIM/100/IMG_6101.NEF", 21),
        )
        .unwrap();
    let source_page = service
        .project_asset_group_page_with_query(
            &source_project.project_id,
            AssetGroupQuery::default(),
            0,
            25,
        )
        .unwrap();
    let group_id = source_page.groups[0].group_id.clone().unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let moved_json = core
        .move_project_group_json(
            source_project.project_id.clone(),
            group_id.clone(),
            target_project.project_id.clone(),
        )
        .unwrap();
    let source_dashboard: Value = serde_json::from_str(
        &core
            .project_dashboard_json(source_project.project_id.clone(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let target_dashboard: Value = serde_json::from_str(
        &core
            .project_dashboard_json(target_project.project_id.clone(), 0, 25)
            .unwrap(),
    )
    .unwrap();

    let moved: Value = serde_json::from_str(&moved_json).unwrap();
    assert_eq!(moved["project_id"], target_project.project_id);
    assert_eq!(moved["display_key"], "IMG_6101");
    assert_eq!(moved["member_count"], 2);
    assert_eq!(source_dashboard["assets"]["total_groups"], 0);
    assert_eq!(target_dashboard["assets"]["total_groups"], 1);
    assert_eq!(target_dashboard["assets"]["summary"]["asset_count"], 2);
}

#[test]
fn mobile_core_returns_project_asset_group_page_json_with_query() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Query").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-query-jpg", "DCIM/100/IMG_7001.JPG", 20),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-query-raw", "DCIM/100/IMG_7001.NEF", 21),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-query-video", "DCIM/100/VID_7002.MP4", 22),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let raw_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(
                project.project_id.clone(),
                r#"{"role":"raw"}"#.to_string(),
                0,
                25,
            )
            .unwrap(),
    )
    .unwrap();
    let video_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(
                project.project_id,
                r#"{"role":"video"}"#.to_string(),
                0,
                25,
            )
            .unwrap(),
    )
    .unwrap();

    assert_eq!(raw_page["total_groups"], 1);
    assert_eq!(raw_page["groups"][0]["group_key"], "IMG_7001");
    assert_eq!(video_page["total_groups"], 1);
    assert_eq!(video_page["groups"][0]["group_key"], "VID_7002");
}

#[test]
fn mobile_core_drain_analysis_jobs_exposes_burst_summary() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Burst").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:burst-1", "DCIM/100/IMG_7101.JPG", 1000),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:burst-2", "DCIM/100/IMG_7102.JPG", 1100),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let summary: Value = serde_json::from_str(&core.drain_analysis_jobs_json(10).unwrap()).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(summary["claimed_count"], 2);
    assert_eq!(summary["completed_count"], 2);
    assert_eq!(page["groups"][0]["burst"]["member_count"], 2);
}

#[test]
fn mobile_core_exposes_model_evaluation_and_technical_gate_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Mobile Model Evaluation Fields")
        .unwrap();
    service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "global".to_string(),
            provider_kind: ModelProviderKind::Imported,
            provider_label: "Imported".to_string(),
            base_url: "local://imported".to_string(),
            default_model: "model-stub-v1".to_string(),
            default_max_image_side: 1024,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 2,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 1000,
        })
        .unwrap();
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .unwrap()
        .unwrap();
    settings.auto_evaluate_on_upload = true;
    settings.prompt_profile_id = Some("general-default".to_string());
    settings.model_provider_settings_id = Some("global".to_string());
    service.save_project_evaluation_settings(settings).unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:model-fields-1", "DCIM/100/IMG_7251.JPG", 1000),
        )
        .unwrap();

    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let initial_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let group_id = initial_page["groups"][0]["group_id"]
        .as_str()
        .unwrap()
        .to_string();
    core.assess_asset_group_preview_json(
        group_id.clone(),
        balanced_detail_sample_json(16, 16),
        "technical-v1".to_string(),
    )
    .unwrap();

    let assessed_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id, "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let group = &assessed_page["groups"][0];
    assert_eq!(group["group_id"].as_str(), Some(group_id.as_str()));
    assert_eq!(group["technical_gate_status"].as_str(), Some("pass"));
    assert!(group["technical_defects"].as_array().unwrap().is_empty());
    assert_eq!(group["model_status"].as_str(), Some("ready"));
    assert_eq!(group["model_score"].as_i64(), Some(72));
    assert_eq!(group["model_tier"].as_str(), Some("good"));
    assert_eq!(
        group["model_summary"].as_str(),
        Some("passes local technical gate")
    );
    assert_eq!(group["is_model_select"].as_bool(), Some(false));
    assert_eq!(group["is_favorite"].as_bool(), Some(false));
    assert_eq!(group["is_flagged"].as_bool(), Some(false));
}

#[test]
fn mobile_core_enqueues_manual_model_evaluation_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Mobile Manual Model Evaluation")
        .unwrap();
    service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "global".to_string(),
            provider_kind: ModelProviderKind::Imported,
            provider_label: "Imported".to_string(),
            base_url: "local://imported".to_string(),
            default_model: "model-stub-v1".to_string(),
            default_max_image_side: 1024,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 2,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 1000,
        })
        .unwrap();
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .unwrap()
        .unwrap();
    settings.auto_evaluate_on_upload = false;
    settings.prompt_profile_id = Some("general-default".to_string());
    settings.model_provider_settings_id = Some("global".to_string());
    service.save_project_evaluation_settings(settings).unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:manual-mobile-1", "DCIM/100/IMG_8251.JPG", 1000),
        )
        .unwrap();

    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let initial_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let group_id = initial_page["groups"][0]["group_id"]
        .as_str()
        .unwrap()
        .to_string();
    core.assess_asset_group_preview_json(
        group_id.clone(),
        balanced_detail_sample_json(16, 16),
        "technical-v1".to_string(),
    )
    .unwrap();

    let response: Value = serde_json::from_str(
        &core
            .enqueue_model_evaluation_for_asset_groups_json(
                serde_json::json!({
                    "project_id": project.project_id,
                    "asset_group_ids": [group_id],
                })
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();

    assert_eq!(response["enqueued_count"].as_i64(), Some(1));
}

#[test]
fn mobile_core_evaluates_manual_model_inputs_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Mobile Manual Preview Evaluation")
        .unwrap();
    service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "global".to_string(),
            provider_kind: ModelProviderKind::Imported,
            provider_label: "Imported".to_string(),
            base_url: "local://imported".to_string(),
            default_model: "model-stub-v1".to_string(),
            default_max_image_side: 1024,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 2,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 1000,
        })
        .unwrap();
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .unwrap()
        .unwrap();
    settings.auto_evaluate_on_upload = false;
    settings.prompt_profile_id = Some("general-default".to_string());
    settings.model_provider_settings_id = Some("global".to_string());
    service.save_project_evaluation_settings(settings).unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:manual-input-1", "DCIM/100/IMG_8351.JPG", 1000),
        )
        .unwrap();

    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let initial_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let group_id = initial_page["groups"][0]["group_id"]
        .as_str()
        .unwrap()
        .to_string();
    let sample: Value = serde_json::from_str(&balanced_detail_sample_json(16, 16)).unwrap();

    let response: Value = serde_json::from_str(
        &core
            .evaluate_asset_groups_with_model_inputs_json(
                serde_json::json!({
                    "project_id": project.project_id,
                    "inputs": [{
                        "asset_group_id": group_id,
                        "sample": sample,
                    }],
                })
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();

    let assessed_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id, "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let group = &assessed_page["groups"][0];

    assert_eq!(response["saved_count"].as_i64(), Some(1));
    assert_eq!(group["technical_gate_status"].as_str(), Some("pass"));
    assert_eq!(group["model_status"].as_str(), Some("ready"));
    assert_eq!(group["model_score"].as_i64(), Some(72));
}

#[test]
fn mobile_core_recommends_burst_group_with_candidate_visuals_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Mobile Visual Burst Recommendation")
        .unwrap();
    service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "global".to_string(),
            provider_kind: ModelProviderKind::Imported,
            provider_label: "Imported".to_string(),
            base_url: "local://imported".to_string(),
            default_model: "model-stub-v1".to_string(),
            default_max_image_side: 1024,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 2,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 1000,
        })
        .unwrap();
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .unwrap()
        .unwrap();
    settings.auto_evaluate_on_upload = false;
    settings.auto_burst_recommendation_enabled = true;
    settings.prompt_profile_id = Some("general-default".to_string());
    settings.model_provider_settings_id = Some("global".to_string());
    service.save_project_evaluation_settings(settings).unwrap();
    for (transfer_id, path, completed_at_ms) in [
        ("ftp:visual-burst-1", "DCIM/100/VISUAL_1001.JPG", 1000),
        ("ftp:visual-burst-2", "DCIM/100/VISUAL_1002.JPG", 1100),
    ] {
        service
            .record_project_transfer(
                &project.project_id,
                completed_transfer(transfer_id, path, completed_at_ms),
            )
            .unwrap();
    }
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let groups = page["groups"].as_array().unwrap();
    let burst_id = groups[0]["burst"]["burst_group_id"]
        .as_str()
        .unwrap()
        .to_string();
    let group_ids = groups
        .iter()
        .map(|group| group["group_id"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    let inputs = group_ids
        .iter()
        .map(|group_id| {
            serde_json::json!({
                "asset_group_id": group_id,
                "sample": serde_json::from_str::<Value>(&balanced_detail_sample_json(16, 16)).unwrap(),
            })
        })
        .collect::<Vec<_>>();
    core.evaluate_asset_groups_with_model_inputs_json(
        serde_json::json!({
            "project_id": project.project_id,
            "inputs": inputs,
        })
        .to_string(),
    )
    .unwrap();

    let response: Value = serde_json::from_str(
        &core
            .recommend_burst_group_with_candidate_visuals_json(
                serde_json::json!({
                    "burst_group_id": burst_id,
                    "candidate_visuals": group_ids.iter().enumerate().map(|(index, group_id)| {
                        serde_json::json!({
                            "asset_group_id": group_id,
                            "image_data_url": format!("data:image/jpeg;base64,YnVyc3Qt{index}"),
                        })
                    }).collect::<Vec<_>>(),
                })
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();

    assert_eq!(response["scope"].as_str(), Some("burst_group"));
    assert_eq!(response["subject_id"].as_str(), Some(burst_id.as_str()));
    assert_eq!(response["status"].as_str(), Some("ready"));
    assert_eq!(
        response["selected_asset_group_ids"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn mobile_core_splits_burst_member_json() {
    let fixture = three_member_burst_fixture("Mobile Split Decision");

    let updated: Value = serde_json::from_str(
        &fixture
            .core
            .split_burst_member_json(
                fixture.burst_id.to_string(),
                fixture.member_group_id.to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let page: Value = serde_json::from_str(
        &fixture
            .core
            .project_asset_group_page_json(fixture.project_id, "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let split_group = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["group_id"].as_str() == Some(fixture.member_group_id.as_str()))
        .expect("split group should remain visible");

    assert_eq!(updated["member_count"], 2);
    assert_eq!(updated["recommendation_status"], "pending");
    assert!(split_group.get("burst").is_none());
}

#[test]
fn mobile_core_merges_burst_member_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Merge Decision").unwrap();
    for (transfer_id, path, completed_at_ms) in [
        ("ftp:merge-a-1", "DCIM/240/IMG_8241.JPG", 1000),
        ("ftp:merge-a-2", "DCIM/240/IMG_8242.JPG", 1100),
        ("ftp:merge-b-1", "DCIM/241/IMG_9241.JPG", 5000),
        ("ftp:merge-b-2", "DCIM/241/IMG_9242.JPG", 5100),
    ] {
        service
            .record_project_transfer(
                &project.project_id,
                completed_transfer(transfer_id, path, completed_at_ms),
            )
            .unwrap();
    }
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let target = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| {
            group["primary"]["original_path"]
                .as_str()
                .map(|path| path.ends_with("IMG_8241.JPG"))
                .unwrap_or(false)
        })
        .expect("target member should exist");
    let source = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| {
            group["primary"]["original_path"]
                .as_str()
                .map(|path| path.ends_with("IMG_9241.JPG"))
                .unwrap_or(false)
        })
        .expect("source member should exist");
    let target_burst_id = target["burst"]["burst_group_id"].as_str().unwrap();
    let source_group_id = source["group_id"].as_str().unwrap();

    let merged: Value = serde_json::from_str(
        &core
            .merge_burst_member_json(target_burst_id.to_string(), source_group_id.to_string())
            .unwrap(),
    )
    .unwrap();

    assert_eq!(merged["member_count"], 4);
    assert_eq!(merged["recommendation_status"], "pending");
    assert_eq!(merged["manual_grouping_state"], "merge");
}

#[test]
fn mobile_core_starts_and_stops_receiver_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let output_dir = temp.path().join("output");
    let state_dir = temp.path().join("state");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.save_receiver_settings_json(MobileReceiverSettingsPatch {
        protocol: Some("ftp".to_string()),
        bind_host: Some("127.0.0.1".to_string()),
        ftp_port: Some(0),
        output_dir: Some(output_dir.to_string_lossy().into_owned()),
        state_dir: Some(state_dir.to_string_lossy().into_owned()),
        ..MobileReceiverSettingsPatch::default()
    })
    .unwrap();

    let started_json = core.start_receiver_json().unwrap();
    let started: Value = serde_json::from_str(&started_json).unwrap();
    assert_eq!(started["phase"], "Running");
    assert_eq!(started["protocol"], "Ftp");
    assert!(started["local_addr"]
        .as_str()
        .unwrap()
        .starts_with("127.0.0.1:"));

    let stopped_json = core.stop_receiver_json().unwrap();
    let stopped: Value = serde_json::from_str(&stopped_json).unwrap();
    assert_eq!(stopped["phase"], "Stopped");
}

fn balanced_detail_sample_json(width: usize, height: usize) -> String {
    let mut luma = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let value = 80 + ((x * 17 + y * 23) % 96) as u8;
            luma.push(value);
        }
    }
    serde_json::json!({
        "width": width,
        "height": height,
        "luma": luma,
        "preview_source": "test"
    })
    .to_string()
}

struct MobileBurstFixture {
    _temp: tempfile::TempDir,
    core: MobileCore,
    project_id: String,
    burst_id: String,
    member_group_id: String,
}

fn three_member_burst_fixture(project_name: &str) -> MobileBurstFixture {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project(project_name).unwrap();
    for (transfer_id, path, completed_at_ms) in [
        ("ftp:three-decision-1", "DCIM/100/IMG_7601.JPG", 1000),
        ("ftp:three-decision-2", "DCIM/100/IMG_7602.JPG", 1100),
        ("ftp:three-decision-3", "DCIM/100/IMG_7603.JPG", 1200),
    ] {
        service
            .record_project_transfer(
                &project.project_id,
                completed_transfer(transfer_id, path, completed_at_ms),
            )
            .unwrap();
    }
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let burst_members = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|group| group.get("burst").is_some())
        .collect::<Vec<_>>();
    let burst_id = burst_members[0]["burst"]["burst_group_id"]
        .as_str()
        .unwrap()
        .to_string();
    let member_group_id = burst_members[1]["group_id"].as_str().unwrap().to_string();

    MobileBurstFixture {
        _temp: temp,
        core,
        project_id: project.project_id,
        burst_id,
        member_group_id,
    }
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

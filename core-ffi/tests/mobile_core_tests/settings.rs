use super::*;

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
                    "prompt_pack_id":null,
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

    let selected_prompt: Value = serde_json::from_str(
        &core
            .save_project_evaluation_settings_json(
                project_id.clone(),
                r#"{
                    "prompt_pack_id":"general-default"
                }"#
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(selected_prompt["prompt_pack_id"], "general-default");
    assert_eq!(
        selected_prompt["model_provider_settings_id"],
        "photo-eval-model"
    );

    let cleared_prompt_and_provider: Value = serde_json::from_str(
        &core
            .save_project_evaluation_settings_json(
                project_id.clone(),
                r#"{
                    "prompt_pack_id":null,
                    "model_provider_settings_id":null
                }"#
                .to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert!(cleared_prompt_and_provider["prompt_pack_id"].is_null());
    assert!(cleared_prompt_and_provider["model_provider_settings_id"].is_null());

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
fn mobile_core_lists_forks_and_edits_prompt_packs_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Prompt Packs").unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let profiles: Value = serde_json::from_str(
        &core
            .prompt_packs_for_project_json(project.project_id.clone())
            .unwrap(),
    )
    .unwrap();
    let built_in = profiles
        .as_array()
        .unwrap()
        .iter()
        .find(|profile| profile["prompt_pack_id"] == "general-default")
        .unwrap();
    assert_eq!(built_in["built_in"], true);
    assert!(built_in["style_tags"]
        .as_array()
        .unwrap()
        .contains(&Value::String("通用".to_string())));

    let forked: Value = serde_json::from_str(
        &core
            .fork_prompt_pack_json(
                project.project_id.clone(),
                "general-default".to_string(),
                "Client Editorial".to_string(),
                "client-pack".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(forked["built_in"], false);
    assert_eq!(forked["author"], "user");

    let edited: Value = serde_json::from_str(
        &core
            .save_prompt_pack_json(
                project.project_id.clone(),
                forked["prompt_pack_id"].as_str().unwrap().to_string(),
                "Client Editorial Edited".to_string(),
                r#"["client","edited"]"#.to_string(),
                "general".to_string(),
                "Return concise project recommendations.".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        edited["prompt_pack_id"],
        forked["prompt_pack_id"].as_str().unwrap()
    );
    assert!(edited["version"].as_str().unwrap().starts_with("user-"));
    assert_eq!(
        edited["prompt_text"],
        "Return concise project recommendations."
    );

    let deleted: Value = serde_json::from_str(
        &core
            .delete_global_prompt_pack_json(forked["prompt_pack_id"].as_str().unwrap().to_string())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(deleted["deleted"], true);
    let remaining: Value = serde_json::from_str(
        &core
            .prompt_packs_for_project_json(project.project_id.clone())
            .unwrap(),
    )
    .unwrap();
    assert!(!remaining.as_array().unwrap().iter().any(|profile| {
        profile["prompt_pack_id"] == forked["prompt_pack_id"].as_str().unwrap()
    }));

    let package_pack: Value = serde_json::from_str(
        &core
            .create_global_prompt_pack_json(
                "Package Delete Candidate".to_string(),
                r#"["client"]"#.to_string(),
                "general".to_string(),
                "delete-package".to_string(),
                "Temporary package prompt.".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(package_pack["distribution_folder"], "delete-package");
    let package_deleted: Value = serde_json::from_str(
        &core
            .delete_global_prompt_package_json("delete-package".to_string())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(package_deleted["deleted"], true);
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
            "prompt_pack_id": null,
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
            .fork_prompt_pack_json(
                project.project_id.clone(),
                "general-default".to_string(),
                "Editable A".to_string(),
                "user".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let second: Value = serde_json::from_str(
        &core
            .fork_prompt_pack_json(
                project.project_id.clone(),
                "general-default".to_string(),
                "Editable B".to_string(),
                "user".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_ne!(first["prompt_pack_id"], second["prompt_pack_id"]);

    let first_version: Value = serde_json::from_str(
        &core
            .save_prompt_pack_json(
                project.project_id.clone(),
                first["prompt_pack_id"].as_str().unwrap().to_string(),
                "Editable B Revised".to_string(),
                r#"["editable","first"]"#.to_string(),
                "general".to_string(),
                "Prompt version one".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let second_version: Value = serde_json::from_str(
        &core
            .save_prompt_pack_json(
                project.project_id,
                first["prompt_pack_id"].as_str().unwrap().to_string(),
                "Editable B Revised Again".to_string(),
                r#"["editable","second"]"#.to_string(),
                "general".to_string(),
                "Prompt version one".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_ne!(first_version["version"], second_version["version"]);
}

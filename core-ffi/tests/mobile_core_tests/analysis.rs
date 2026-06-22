use super::*;

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
    settings.prompt_pack_id = Some("general-default".to_string());
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
    settings.prompt_pack_id = Some("general-default".to_string());
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
            "prompt_pack_id":null,
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
    settings.prompt_pack_id = Some("general-default".to_string());
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
    settings.prompt_pack_id = Some("general-default".to_string());
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
    settings.prompt_pack_id = Some("general-default".to_string());
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
    settings.prompt_pack_id = Some("general-default".to_string());
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

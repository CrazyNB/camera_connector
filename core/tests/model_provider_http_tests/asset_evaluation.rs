use camera_connector_core::{
    AssetGroupModelEvaluationInput, CameraConnectorService, ModelProviderKind,
    ModelProviderSettings, ModelSendMode,
};

use super::support::{
    balanced_preview_sample, completed_transfer, enable_upload_model_evaluation,
    select_model_provider, select_prompt_pack, TestModelServer,
};

#[test]
fn upload_model_evaluation_uses_configured_openai_compatible_provider() {
    let server = TestModelServer::start(
        r#"{
            "choices": [{
                "message": {
                    "content": "{\"score\":91,\"tier\":\"excellent\",\"selectable\":true,\"summary\":\"Strong composition and subject presence\",\"strengths\":[\"clear subject\",\"balanced color\"],\"weaknesses\":[],\"technical_warnings\":[]}"
                }
            }]
        }"#,
    );
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("HTTP Model Evaluation")
        .expect("project should create");
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "global".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "OpenAI compatible".to_string(),
                base_url: server.base_url(),
                default_model: "gpt-test".to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 1,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 1_000,
            },
            Some("sk-test-provider".to_string()),
        )
        .expect("provider should save");
    enable_upload_model_evaluation(&service, &project.project_id);
    select_model_provider(&service, &project.project_id, "global");
    let prompt = service
        .create_global_prompt_pack(
            "通用评价偏好",
            vec!["通用".to_string()],
            camera_connector_core::SceneProfile::General,
            "user",
            "偏好主体清晰、情绪自然、技术稳定的照片。",
            2_000,
        )
        .expect("prompt pack should create");
    select_prompt_pack(&service, &project.project_id, &prompt.prompt_pack_id);
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:http-model", "DCIM/100/HTTP_0001.JPG", 1_000),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 10)
        .expect("asset page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    service
        .assess_asset_group_preview_with_provider_configured(
            &group_id,
            balanced_preview_sample(),
            "technical-v1",
            true,
        )
        .expect("model evaluation should save");

    let request = server
        .received_request()
        .expect("provider should receive a chat completions request");
    let request_lower = request.to_ascii_lowercase();
    assert!(request.contains("POST /chat/completions HTTP/1.1"));
    assert!(request_lower.contains("authorization: bearer sk-test-provider"));
    assert!(request.contains("\"model\":\"gpt-test\""));
    assert!(request.contains("Camera Connector"));
    assert!(request.contains("technical_gate"));
    assert!(request.contains("偏好主体清晰、情绪自然、技术稳定的照片。"));
    assert!(request.contains("Evaluation task instruction"));
    assert!(
        request.contains("\"type\":\"image_url\""),
        "model provider request should include a visual image part"
    );
    assert!(
        request.contains("data:image/png;base64,"),
        "preview samples should be sent as an inline PNG data URL"
    );

    let evaluations = service
        .storage_store()
        .expect("store should open")
        .model_evaluations_for_asset_groups(&[group_id], "gpt-test")
        .expect("model evaluations should query");
    assert_eq!(evaluations.len(), 1);
    assert_eq!(evaluations[0].score, 91);
    assert_eq!(evaluations[0].tier.as_str(), "excellent");
    assert_eq!(
        evaluations[0].summary,
        "Strong composition and subject presence"
    );
}

#[test]
fn upload_model_evaluation_without_api_key_does_not_fallback_to_fake_score() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Missing API Key")
        .expect("project should create");
    service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "global".to_string(),
            provider_kind: ModelProviderKind::OpenAi,
            provider_label: "OpenAI compatible".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-test".to_string(),
            default_max_image_side: 1600,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 1,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 1_000,
        })
        .expect("provider should save");
    enable_upload_model_evaluation(&service, &project.project_id);
    select_model_provider(&service, &project.project_id, "global");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:no-api-key", "DCIM/100/NO_KEY_0001.JPG", 1_000),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, false)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 10)
        .expect("asset page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    service
        .assess_asset_group_preview(&group_id, balanced_preview_sample(), "technical-v1")
        .expect("technical evaluation should save");

    let evaluations = service
        .storage_store()
        .expect("store should open")
        .model_evaluations_for_asset_groups(&[group_id], "gpt-test")
        .expect("model evaluations should query");
    assert!(evaluations.is_empty());
}

#[test]
fn project_model_evaluation_uses_selected_model_provider_profile() {
    let server = TestModelServer::start(
        r#"{
            "choices": [{
                "message": {
                    "content": "{\"score\":88,\"tier\":\"excellent\",\"selectable\":true,\"summary\":\"Selected model profile was used\",\"strengths\":[\"profile selection\"],\"weaknesses\":[],\"technical_warnings\":[]}"
                }
            }]
        }"#,
    );
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Selected Model Profile")
        .expect("project should create");
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "fast-model".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "Fast model".to_string(),
                base_url: server.base_url(),
                default_model: "gpt-fast".to_string(),
                default_max_image_side: 1200,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 1,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 1_000,
            },
            Some("sk-shared-provider".to_string()),
        )
        .expect("fast provider should save");
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "photo-eval-model".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "Photo evaluation model".to_string(),
                base_url: server.base_url(),
                default_model: "gpt-photo-eval".to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 1,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 1_001,
            },
            Some("sk-shared-provider".to_string()),
        )
        .expect("evaluation provider should save");
    enable_upload_model_evaluation(&service, &project.project_id);
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.model_provider_settings_id = Some("photo-eval-model".to_string());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:selected-model", "DCIM/100/SELECTED_0001.JPG", 1_000),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 10)
        .expect("asset page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    service
        .assess_asset_group_preview(&group_id, balanced_preview_sample(), "technical-v1")
        .expect("model evaluation should save");

    let request = server
        .received_request()
        .expect("provider should receive request");
    assert!(request.contains("\"model\":\"gpt-photo-eval\""));
    assert!(!request.contains("\"model\":\"gpt-fast\""));
    let evaluations = service
        .storage_store()
        .expect("store should open")
        .model_evaluations_for_asset_groups(&[group_id], "gpt-photo-eval")
        .expect("model evaluations should query");
    assert_eq!(evaluations.len(), 1);
    assert_eq!(evaluations[0].score, 88);
}

#[test]
fn project_model_evaluation_does_not_fallback_when_selected_provider_is_missing() {
    let server = TestModelServer::start(
        r#"{
            "choices": [{
                "message": {
                    "content": "{\"score\":88,\"tier\":\"excellent\",\"selectable\":true,\"summary\":\"Unexpected fallback\",\"strengths\":[],\"weaknesses\":[],\"technical_warnings\":[]}"
                }
            }]
        }"#,
    );
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Missing Selected Model Profile")
        .expect("project should create");
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "fast-model".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "Fast model".to_string(),
                base_url: server.base_url(),
                default_model: "gpt-fast".to_string(),
                default_max_image_side: 1200,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 1,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 1_000,
            },
            Some("sk-shared-provider".to_string()),
        )
        .expect("provider should save");
    enable_upload_model_evaluation(&service, &project.project_id);
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.model_provider_settings_id = Some("missing-model".to_string());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:missing-selected",
                "DCIM/100/MISSING_SELECTED_0001.JPG",
                1_000,
            ),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 10)
        .expect("asset page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    service
        .assess_asset_group_preview(&group_id, balanced_preview_sample(), "technical-v1")
        .expect("technical evaluation should save");

    assert!(
        server.received_request().is_none(),
        "missing selected provider must not silently use another configured model"
    );
    let evaluations = service
        .storage_store()
        .expect("store should open")
        .model_evaluations_for_asset_groups(&[group_id], "gpt-fast")
        .expect("model evaluations should query");
    assert!(evaluations.is_empty());
}

#[test]
fn project_model_evaluation_requires_project_selected_provider_profile() {
    let server = TestModelServer::start(
        r#"{
            "choices": [{
                "message": {
                    "content": "{\"score\":88,\"tier\":\"excellent\",\"selectable\":true,\"summary\":\"Unexpected fallback\",\"strengths\":[],\"weaknesses\":[],\"technical_warnings\":[]}"
                }
            }]
        }"#,
    );
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("No Selected Model Profile")
        .expect("project should create");
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "global".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "Global model".to_string(),
                base_url: server.base_url(),
                default_model: "gpt-global".to_string(),
                default_max_image_side: 1200,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 1,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 1_000,
            },
            Some("sk-shared-provider".to_string()),
        )
        .expect("provider should save");
    enable_upload_model_evaluation(&service, &project.project_id);
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:no-selected", "DCIM/100/NO_SELECTED_0001.JPG", 1_000),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 10)
        .expect("asset page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    service
        .assess_asset_group_preview(&group_id, balanced_preview_sample(), "technical-v1")
        .expect("technical evaluation should save");

    assert!(
        server.received_request().is_none(),
        "project model evaluation must wait for an explicit model provider selection"
    );
    let evaluations = service
        .storage_store()
        .expect("store should open")
        .model_evaluations_for_asset_groups(&[group_id], "gpt-global")
        .expect("model evaluations should query");
    assert!(evaluations.is_empty());
}

#[test]
fn upload_model_evaluation_prefers_supplied_preview_image_data_url() {
    let server = TestModelServer::start(
        r#"{
            "choices": [{
                "message": {
                    "content": "{\"score\":89,\"tier\":\"excellent\",\"selectable\":true,\"summary\":\"Color preview was used\",\"strengths\":[\"actual preview\"],\"weaknesses\":[],\"technical_warnings\":[]}"
                }
            }]
        }"#,
    );
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Supplied Preview Image")
        .expect("project should create");
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "global".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "OpenAI compatible".to_string(),
                base_url: server.base_url(),
                default_model: "gpt-preview".to_string(),
                default_max_image_side: 1200,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 1,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 1_000,
            },
            Some("sk-preview-provider".to_string()),
        )
        .expect("provider should save");
    enable_upload_model_evaluation(&service, &project.project_id);
    select_model_provider(&service, &project.project_id, "global");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:preview-image", "DCIM/100/PREVIEW_0001.JPG", 1_000),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 10)
        .expect("asset page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    service
        .assess_asset_group_preview_with_image_data_url_and_provider_configured(
            &group_id,
            balanced_preview_sample(),
            Some("data:image/jpeg;base64,ZmFrZQ=="),
            "technical-v1",
            true,
        )
        .expect("model evaluation should save");

    let request = server
        .received_request()
        .expect("provider should receive request");
    assert!(request.contains("data:image/jpeg;base64,ZmFrZQ=="));
    assert!(
        !request.contains("data:image/png;base64,"),
        "supplied preview image should be preferred over generated luma PNG"
    );
}

#[test]
fn manual_model_evaluation_with_preview_input_sends_supplied_image() {
    let server = TestModelServer::start(
        r#"{
            "choices": [{
                "message": {
                    "content": "{\"score\":87,\"tier\":\"excellent\",\"selectable\":true,\"summary\":\"Manual preview was evaluated\",\"strengths\":[\"expressive moment\"],\"weaknesses\":[],\"technical_warnings\":[]}"
                }
            }]
        }"#,
    );
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Manual Preview Model Evaluation")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:manual-preview", "DCIM/100/MANUAL_0001.JPG", 1_000),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, false)
        .expect("burst detection should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "manual-preview-provider".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "Manual preview".to_string(),
                base_url: server.base_url(),
                default_model: "gpt-manual-preview".to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 4,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 2_000,
            },
            Some("sk-manual-preview".to_string()),
        )
        .expect("provider should save");
    select_model_provider(&service, &project.project_id, "manual-preview-provider");
    enable_upload_model_evaluation(&service, &project.project_id);

    let saved_count = service
        .evaluate_asset_groups_with_model_inputs(
            &project.project_id,
            &[AssetGroupModelEvaluationInput {
                asset_group_id: group_id.clone(),
                preview_sample: balanced_preview_sample(),
                preview_image_data_url: Some("data:image/jpeg;base64,bWFudWFsLWpwZWc=".to_string()),
            }],
        )
        .expect("manual model evaluation should run");

    let store = service.storage_store().expect("store should open");
    let evaluations = store
        .model_evaluations_for_asset_groups(std::slice::from_ref(&group_id), "gpt-manual-preview")
        .expect("evaluations should query");
    let run = store
        .latest_evaluation_run(
            &project.project_id,
            camera_connector_core::EvaluationRunType::AssetEvaluation,
        )
        .expect("run should query")
        .expect("run should exist");
    let request = server
        .received_request()
        .expect("provider should receive manual preview request");

    assert_eq!(saved_count, 1);
    assert_eq!(evaluations.len(), 1);
    assert_eq!(evaluations[0].summary, "Manual preview was evaluated");
    assert_eq!(
        run.trigger,
        camera_connector_core::EvaluationRunTrigger::Manual
    );
    assert!(request.contains("\"model\":\"gpt-manual-preview\""));
    assert!(request.contains("data:image/jpeg;base64,bWFudWFsLWpwZWc="));
}

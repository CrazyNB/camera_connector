use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use camera_connector_core::{
    recommend_selection_with_model_provider, AssetGroupModelEvaluationInput,
    CameraConnectorService, ModelEvaluation, ModelEvaluationStatus, ModelEvaluationTier,
    ModelEvaluatorKind, ModelProviderKind, ModelProviderSettings, ModelSendMode, PreviewSample,
    ProjectEvaluationSettings, PromptPackContent, SelectionCandidateVisualInput,
    SelectionRecommendationScope, SelectionRecommendationStatus, SelectionSource,
    StoredObjectLocation, TransferRecord, TransferStatus,
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
        .expect("prompt profile should create");
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

#[test]
fn burst_selection_recommendation_uses_configured_model_provider_decision() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Provider Burst Recommendation")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:burst-model-1", "DCIM/100/BURST_MODEL_0001.JPG", 1_000),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:burst-model-2", "DCIM/100/BURST_MODEL_0002.JPG", 1_100),
        )
        .expect("second transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, false)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let first = &page.groups[0];
    let second = &page.groups[1];
    let first_id = first.group_id.as_ref().expect("first group id").clone();
    let second_id = second.group_id.as_ref().expect("second group id").clone();
    let burst_id = first
        .burst
        .as_ref()
        .expect("burst should exist")
        .burst_group_id
        .clone();
    let model = "gpt-selector";
    let store = service.storage_store().expect("store should open");
    store
        .save_model_evaluation(model_evaluation(
            &project.project_id,
            &first_id,
            model,
            92,
            "technically stronger but less expressive",
        ))
        .expect("first evaluation should save");
    store
        .save_model_evaluation(model_evaluation(
            &project.project_id,
            &second_id,
            model,
            74,
            "better decisive moment",
        ))
        .expect("second evaluation should save");
    drop(store);

    let response = format!(
        r#"{{
            "choices": [{{
                "message": {{
                    "content": "{{\"selected_asset_group_ids\":[\"{second_id}\"],\"candidate_asset_group_ids\":[\"{first_id}\"],\"rejected_asset_group_ids\":[],\"confidence\":0.84,\"reason\":\"model prefers the stronger photographic moment despite lower technical score\"}}"
                }}
            }}]
        }}"#
    );
    let server = TestModelServer::start_owned(response);
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "selector".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "Selector model".to_string(),
                base_url: server.base_url(),
                default_model: model.to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 4,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 2_000,
            },
            Some("sk-selector".to_string()),
        )
        .expect("provider should save");
    select_model_provider(&service, &project.project_id, "selector");
    let prompt = service
        .create_global_prompt_pack(
            "连拍优选偏好",
            vec!["连拍".to_string()],
            camera_connector_core::SceneProfile::General,
            "user",
            "连拍优选时优先选择决定性瞬间，其次考虑技术质量。",
            2_100,
        )
        .expect("prompt profile should create");
    select_prompt_pack(&service, &project.project_id, &prompt.prompt_pack_id);

    let recommendation = service
        .recommend_burst_group_from_model_with_candidate_visuals(
            &burst_id,
            &[
                SelectionCandidateVisualInput {
                    asset_group_id: first_id.clone(),
                    image_data_url: "data:image/jpeg;base64,c2VydmljZS1h".to_string(),
                },
                SelectionCandidateVisualInput {
                    asset_group_id: second_id.clone(),
                    image_data_url: "data:image/jpeg;base64,c2VydmljZS1i".to_string(),
                },
            ],
        )
        .expect("burst recommendation should save");

    assert_eq!(
        recommendation.scope,
        SelectionRecommendationScope::BurstGroup
    );
    assert_eq!(recommendation.source, SelectionSource::Llm);
    assert_eq!(
        recommendation.selected_asset_group_ids,
        vec![second_id.clone()]
    );
    assert_eq!(
        recommendation.candidate_asset_group_ids,
        vec![first_id.clone()]
    );
    assert_eq!(
        recommendation.reason,
        "model prefers the stronger photographic moment despite lower technical score"
    );
    let request = server
        .received_request()
        .expect("provider should receive a recommendation request");
    assert!(request.contains("\"model\":\"gpt-selector\""));
    assert!(request.contains("burst_group"));
    assert!(request.contains("data:image/jpeg;base64,c2VydmljZS1h"));
    assert!(request.contains("data:image/jpeg;base64,c2VydmljZS1i"));
    assert!(request.contains("technically stronger but less expressive"));
    assert!(request.contains("better decisive moment"));
    assert!(request.contains("连拍优选时优先选择决定性瞬间，其次考虑技术质量。"));
    assert!(request.contains("Burst selection instruction"));
    assert!(!request.contains("Project selection instruction"));
}

#[test]
fn burst_selection_recommendation_can_use_visual_candidates_without_prior_model_evaluations() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Provider Burst Visual Recommendation")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:burst-visual-1",
                "DCIM/100/BURST_VISUAL_0001.JPG",
                1_000,
            ),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:burst-visual-2",
                "DCIM/100/BURST_VISUAL_0002.JPG",
                1_100,
            ),
        )
        .expect("second transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, false)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let first = &page.groups[0];
    let second = &page.groups[1];
    let first_id = first.group_id.as_ref().expect("first group id").clone();
    let second_id = second.group_id.as_ref().expect("second group id").clone();
    let burst_id = first
        .burst
        .as_ref()
        .expect("burst should exist")
        .burst_group_id
        .clone();
    let response = format!(
        r#"{{
            "choices": [{{
                "message": {{
                    "content": "{{\"selected_asset_group_ids\":[\"{second_id}\"],\"candidate_asset_group_ids\":[\"{first_id}\"],\"rejected_asset_group_ids\":[],\"confidence\":0.79,\"reason\":\"visual-only burst comparison prefers the stronger frame\"}}"
                }}
            }}]
        }}"#
    );
    let server = TestModelServer::start_owned(response);
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "visual-selector".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "Visual selector".to_string(),
                base_url: server.base_url(),
                default_model: "gpt-visual-selector".to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 4,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 2_000,
            },
            Some("sk-visual-selector".to_string()),
        )
        .expect("provider should save");
    select_model_provider(&service, &project.project_id, "visual-selector");

    let recommendation = service
        .recommend_burst_group_from_model_with_candidate_visuals(
            &burst_id,
            &[
                SelectionCandidateVisualInput {
                    asset_group_id: first_id.clone(),
                    image_data_url: "data:image/jpeg;base64,dmlzdWFsLWJ1cnN0LWE=".to_string(),
                },
                SelectionCandidateVisualInput {
                    asset_group_id: second_id.clone(),
                    image_data_url: "data:image/jpeg;base64,dmlzdWFsLWJ1cnN0LWI=".to_string(),
                },
            ],
        )
        .expect("burst visual-only recommendation should save");

    assert_eq!(recommendation.source, SelectionSource::Llm);
    assert_eq!(
        recommendation.status,
        SelectionRecommendationStatus::Pending
    );
    assert_eq!(
        recommendation.selected_asset_group_ids,
        Vec::<String>::new()
    );
    assert_eq!(
        recommendation.candidate_asset_group_ids,
        vec![second_id.clone(), first_id.clone()]
    );
    let request = server
        .received_request()
        .expect("provider should receive a visual-only burst recommendation request");
    assert!(request.contains("\"model\":\"gpt-visual-selector\""));
    assert!(request.contains("\"type\":\"image_url\""));
    assert!(request.contains("data:image/jpeg;base64,dmlzdWFsLWJ1cnN0LWE="));
    assert!(request.contains("data:image/jpeg;base64,dmlzdWFsLWJ1cnN0LWI="));
    assert!(request.contains(first_id.as_str()));
    assert!(request.contains(second_id.as_str()));
}

#[test]
fn burst_selection_recommendation_evaluates_preselected_candidates_before_final_selection() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Provider Burst Candidate Evaluation")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:burst-candidate-1",
                "DCIM/100/BURST_CANDIDATE_0001.JPG",
                1_000,
            ),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:burst-candidate-2",
                "DCIM/100/BURST_CANDIDATE_0002.JPG",
                1_100,
            ),
        )
        .expect("second transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, false)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let first = &page.groups[0];
    let second = &page.groups[1];
    let first_id = first.group_id.as_ref().expect("first group id").clone();
    let second_id = second.group_id.as_ref().expect("second group id").clone();
    let burst_id = first
        .burst
        .as_ref()
        .expect("burst should exist")
        .burst_group_id
        .clone();
    let server = TestModelServer::start_sequence_owned(vec![
        format!(
            r#"{{
                "choices": [{{
                    "message": {{
                        "content": "{{\"selected_asset_group_ids\":[\"{second_id}\"],\"candidate_asset_group_ids\":[\"{first_id}\"],\"rejected_asset_group_ids\":[],\"confidence\":0.72,\"reason\":\"preselect strongest visual candidates\"}}"
                    }}
                }}]
            }}"#
        ),
        r#"{
            "choices": [{
                "message": {
                    "content": "{\"score\":88,\"tier\":\"good\",\"selectable\":true,\"summary\":\"Candidate two is strong enough for final burst comparison\",\"strengths\":[\"clear subject\"],\"weaknesses\":[],\"technical_warnings\":[]}"
                }
            }]
        }"#
        .to_string(),
        r#"{
            "choices": [{
                "message": {
                    "content": "{\"score\":64,\"tier\":\"usable\",\"selectable\":true,\"summary\":\"Candidate one is usable but weaker\",\"strengths\":[\"usable frame\"],\"weaknesses\":[\"less decisive\"],\"technical_warnings\":[]}"
                }
            }]
        }"#
        .to_string(),
        format!(
            r#"{{
                "choices": [{{
                    "message": {{
                        "content": "{{\"selected_asset_group_ids\":[\"{second_id}\"],\"candidate_asset_group_ids\":[\"{first_id}\"],\"rejected_asset_group_ids\":[],\"confidence\":0.91,\"reason\":\"final choice uses model-evaluated burst candidates\"}}"
                    }}
                }}]
            }}"#
        ),
    ]);
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "burst-pipeline".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "Burst pipeline".to_string(),
                base_url: server.base_url(),
                default_model: "gpt-burst-pipeline".to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 4,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 2_000,
            },
            Some("sk-burst-pipeline".to_string()),
        )
        .expect("provider should save");
    enable_upload_model_evaluation(&service, &project.project_id);
    select_model_provider(&service, &project.project_id, "burst-pipeline");

    let recommendation = service
        .recommend_burst_group_from_model_with_candidate_visuals(
            &burst_id,
            &[
                SelectionCandidateVisualInput {
                    asset_group_id: first_id.clone(),
                    image_data_url: "data:image/jpeg;base64,Y2FuZGlkYXRlLWE=".to_string(),
                },
                SelectionCandidateVisualInput {
                    asset_group_id: second_id.clone(),
                    image_data_url: "data:image/jpeg;base64,Y2FuZGlkYXRlLWI=".to_string(),
                },
            ],
        )
        .expect("burst recommendation should evaluate candidates and save final choice");

    assert_eq!(recommendation.status, SelectionRecommendationStatus::Ready);
    assert_eq!(
        recommendation.selected_asset_group_ids,
        vec![second_id.clone()]
    );
    assert_eq!(
        recommendation.reason,
        "final choice uses model-evaluated burst candidates"
    );
    let evaluations = service
        .storage_store()
        .expect("store should open")
        .model_evaluations_for_asset_groups(
            &[first_id.clone(), second_id.clone()],
            "gpt-burst-pipeline",
        )
        .expect("evaluations should query");
    assert_eq!(evaluations.len(), 2);
    let requests = server.received_requests(4);
    assert_eq!(requests.len(), 4);
    assert!(requests[0].contains("Selection context"));
    assert!(requests[1].contains("Evaluation task instruction"));
    assert!(requests[2].contains("Evaluation task instruction"));
    assert!(requests[3].contains("Candidate two is strong enough for final burst comparison"));
    assert!(requests[3].contains("Candidate one is usable but weaker"));
}

#[test]
fn project_selection_recommendation_uses_configured_model_provider_decision() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Provider Project Recommendation")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:project-model-1",
                "DCIM/100/PROJECT_MODEL_0001.JPG",
                1_000,
            ),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:project-model-2",
                "DCIM/101/PROJECT_MODEL_0042.JPG",
                20_000,
            ),
        )
        .expect("second transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, false)
        .expect("analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    assert_eq!(page.groups.len(), 2);
    let first_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("first group id")
        .clone();
    let second_id = page.groups[1]
        .group_id
        .as_ref()
        .expect("second group id")
        .clone();
    let model = "gpt-project-selector";
    let store = service.storage_store().expect("store should open");
    store
        .save_model_evaluation(model_evaluation(
            &project.project_id,
            &first_id,
            model,
            96,
            "clean but ordinary",
        ))
        .expect("first evaluation should save");
    store
        .save_model_evaluation(model_evaluation(
            &project.project_id,
            &second_id,
            model,
            66,
            "portfolio-worthy storytelling frame",
        ))
        .expect("second evaluation should save");
    drop(store);

    let response = format!(
        r#"{{
            "choices": [{{
                "message": {{
                    "content": "{{\"selected_asset_group_ids\":[\"{second_id}\"],\"candidate_asset_group_ids\":[],\"rejected_asset_group_ids\":[\"{first_id}\"],\"confidence\":0.79,\"reason\":\"model selects the stronger project-level keeper\"}}"
                }}
            }}]
        }}"#
    );
    let server = TestModelServer::start_owned(response);
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "project-selector".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "Project selector".to_string(),
                base_url: server.base_url(),
                default_model: model.to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 4,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 2_000,
            },
            Some("sk-project-selector".to_string()),
        )
        .expect("provider should save");
    select_model_provider(&service, &project.project_id, "project-selector");
    let prompt = service
        .create_global_prompt_pack(
            "项目优选偏好",
            vec!["项目".to_string()],
            camera_connector_core::SceneProfile::General,
            "user",
            "项目优选时偏好有叙事感、可交付、可入选作品集的照片。",
            2_200,
        )
        .expect("prompt profile should create");
    select_prompt_pack(&service, &project.project_id, &prompt.prompt_pack_id);

    let recommendation = service
        .generate_project_recommendation(&project.project_id, 30_000)
        .expect("project recommendation should save");

    assert_eq!(recommendation.scope, SelectionRecommendationScope::Project);
    assert_eq!(recommendation.source, SelectionSource::Llm);
    assert_eq!(recommendation.selected_asset_group_ids, vec![second_id]);
    assert_eq!(recommendation.rejected_asset_group_ids, vec![first_id]);
    assert_eq!(
        recommendation.reason,
        "model selects the stronger project-level keeper"
    );
    let request = server
        .received_request()
        .expect("provider should receive a project recommendation request");
    assert!(request.contains("\"model\":\"gpt-project-selector\""));
    assert!(request.contains("project"));
    assert!(request.contains("clean but ordinary"));
    assert!(request.contains("portfolio-worthy storytelling frame"));
    assert!(request.contains("项目优选时偏好有叙事感、可交付、可入选作品集的照片。"));
    assert!(request.contains("Project selection instruction"));
    assert!(!request.contains("Burst selection instruction"));
}

#[test]
fn project_selection_recommendation_sends_candidate_preview_images() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Provider Project Visual Recommendation")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:project-visual-1",
                "DCIM/100/PROJECT_VISUAL_0001.JPG",
                1_000,
            ),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:project-visual-2",
                "DCIM/101/PROJECT_VISUAL_0042.JPG",
                20_000,
            ),
        )
        .expect("second transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, false)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let first_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("first group id")
        .clone();
    let second_id = page.groups[1]
        .group_id
        .as_ref()
        .expect("second group id")
        .clone();
    let model = "gpt-project-visual-selector";
    let store = service.storage_store().expect("store should open");
    store
        .save_model_evaluation(model_evaluation(
            &project.project_id,
            &first_id,
            model,
            85,
            "technically clean candidate",
        ))
        .expect("first evaluation should save");
    store
        .save_model_evaluation(model_evaluation(
            &project.project_id,
            &second_id,
            model,
            82,
            "stronger story candidate",
        ))
        .expect("second evaluation should save");
    drop(store);

    let response = format!(
        r#"{{
            "choices": [{{
                "message": {{
                    "content": "{{\"selected_asset_group_ids\":[\"{second_id}\"],\"candidate_asset_group_ids\":[\"{first_id}\"],\"rejected_asset_group_ids\":[],\"confidence\":0.82,\"reason\":\"visual project comparison prefers the second candidate\"}}"
                }}
            }}]
        }}"#
    );
    let server = TestModelServer::start_owned(response);
    service
        .save_model_provider_settings_with_api_key(
            ModelProviderSettings {
                settings_id: "project-visual-selector".to_string(),
                provider_kind: ModelProviderKind::OpenAi,
                provider_label: "Project visual selector".to_string(),
                base_url: server.base_url(),
                default_model: model.to_string(),
                default_max_image_side: 1600,
                default_send_mode: ModelSendMode::PreviewOnly,
                default_batch_size: 4,
                configured: true,
                api_key_configured: false,
                key_alias: None,
                updated_at_ms: 2_000,
            },
            Some("sk-project-visual-selector".to_string()),
        )
        .expect("provider should save");
    select_model_provider(&service, &project.project_id, "project-visual-selector");

    let recommendation = service
        .generate_project_recommendation_with_candidate_visuals(
            &project.project_id,
            &[
                SelectionCandidateVisualInput {
                    asset_group_id: first_id.clone(),
                    image_data_url: "data:image/jpeg;base64,cHJvamVjdC1h".to_string(),
                },
                SelectionCandidateVisualInput {
                    asset_group_id: second_id.clone(),
                    image_data_url: "data:image/jpeg;base64,cHJvamVjdC1i".to_string(),
                },
            ],
            31_000,
        )
        .expect("project visual recommendation should save");

    assert_eq!(recommendation.scope, SelectionRecommendationScope::Project);
    assert_eq!(recommendation.source, SelectionSource::Llm);
    assert_eq!(recommendation.selected_asset_group_ids, vec![second_id]);
    let request = server
        .received_request()
        .expect("provider should receive a project visual recommendation request");
    assert!(request.contains("\"model\":\"gpt-project-visual-selector\""));
    assert!(request.contains("project"));
    assert!(request.contains("data:image/jpeg;base64,cHJvamVjdC1h"));
    assert!(request.contains("data:image/jpeg;base64,cHJvamVjdC1i"));
    assert!(request.contains("visual_candidate_1"));
    assert!(request.contains("visual_candidate_2"));
}

#[test]
fn burst_selection_recommendation_sends_candidate_preview_images() {
    let server = TestModelServer::start(
        r#"{
            "choices": [{
                "message": {
                    "content": "{\"selected_asset_group_ids\":[\"group-b\"],\"candidate_asset_group_ids\":[\"group-a\"],\"rejected_asset_group_ids\":[],\"confidence\":0.81,\"reason\":\"visual comparison prefers group-b\"}"
                }
            }]
        }"#,
    );
    let provider = ModelProviderSettings {
        settings_id: "selector".to_string(),
        provider_kind: ModelProviderKind::OpenAi,
        provider_label: "Selector".to_string(),
        base_url: server.base_url(),
        default_model: "gpt-visual-selector".to_string(),
        default_max_image_side: 1600,
        default_send_mode: ModelSendMode::PreviewOnly,
        default_batch_size: 4,
        configured: true,
        api_key_configured: true,
        key_alias: Some("test".to_string()),
        updated_at_ms: 1_000,
    };

    let recommendation = recommend_selection_with_model_provider(
        "project-visual",
        SelectionRecommendationScope::BurstGroup,
        "burst-visual",
        &[
            model_evaluation(
                "project-visual",
                "group-a",
                "gpt-visual-selector",
                82,
                "sharp frame",
            ),
            model_evaluation(
                "project-visual",
                "group-b",
                "gpt-visual-selector",
                78,
                "better moment",
            ),
        ],
        &[],
        &[
            SelectionCandidateVisualInput {
                asset_group_id: "group-a".to_string(),
                image_data_url: "data:image/jpeg;base64,ZmFrZS1h".to_string(),
            },
            SelectionCandidateVisualInput {
                asset_group_id: "group-b".to_string(),
                image_data_url: "data:image/jpeg;base64,ZmFrZS1i".to_string(),
            },
        ],
        42_000,
        &provider,
        "sk-visual-selector",
        &PromptPackContent::new("Prefer the frame with stronger photographic expression."),
    )
    .expect("visual recommendation should succeed");

    assert_eq!(recommendation.selected_asset_group_ids, vec!["group-b"]);
    let request = server
        .received_request()
        .expect("provider should receive visual recommendation request");
    assert!(request.contains("\"model\":\"gpt-visual-selector\""));
    assert!(request.contains("\"type\":\"image_url\""));
    assert!(request.contains("data:image/jpeg;base64,ZmFrZS1h"));
    assert!(request.contains("data:image/jpeg;base64,ZmFrZS1i"));
    assert!(request.contains("visual_candidate_1"));
    assert!(request.contains("group-a"));
    assert!(request.contains("visual_candidate_2"));
    assert!(request.contains("group-b"));
}

struct TestModelServer {
    address: String,
    request_rx: mpsc::Receiver<String>,
}

impl TestModelServer {
    fn start(response_body: &'static str) -> Self {
        Self::start_owned(response_body.to_string())
    }

    fn start_owned(response_body: String) -> Self {
        Self::start_sequence_owned(vec![response_body])
    }

    fn start_sequence_owned(response_bodies: Vec<String>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("server should bind");
        let address = listener.local_addr().expect("server addr").to_string();
        let (request_tx, request_rx) = mpsc::channel();
        thread::spawn(move || {
            for response_body in response_bodies {
                let (mut stream, _) = listener.accept().expect("server should accept request");
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("read timeout should set");
                let mut request_bytes = Vec::new();
                let mut buffer = [0_u8; 4096];
                loop {
                    let size = stream.read(&mut buffer).expect("request should read");
                    if size == 0 {
                        break;
                    }
                    request_bytes.extend_from_slice(&buffer[..size]);
                    if complete_http_request_len(&request_bytes)
                        .is_some_and(|expected| request_bytes.len() >= expected)
                    {
                        break;
                    }
                }
                let request = String::from_utf8_lossy(&request_bytes).to_string();
                request_tx.send(request).expect("request should send");
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body,
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("response should write");
            }
        });
        Self {
            address,
            request_rx,
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}", self.address)
    }

    fn received_request(&self) -> Option<String> {
        self.request_rx.recv_timeout(Duration::from_secs(2)).ok()
    }

    fn received_requests(&self, count: usize) -> Vec<String> {
        (0..count)
            .filter_map(|_| self.request_rx.recv_timeout(Duration::from_secs(2)).ok())
            .collect()
    }
}

fn complete_http_request_len(buffer: &[u8]) -> Option<usize> {
    let header_end = buffer.windows(4).position(|window| window == b"\r\n\r\n")?;
    let headers = String::from_utf8_lossy(&buffer[..header_end]).to_ascii_lowercase();
    let content_length = headers.lines().find_map(|line| {
        line.strip_prefix("content-length:")
            .and_then(|value| value.trim().parse::<usize>().ok())
    })?;
    Some(header_end + 4 + content_length)
}

fn enable_upload_model_evaluation(service: &CameraConnectorService, project_id: &str) {
    let mut settings = service
        .project_evaluation_settings(project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.auto_evaluate_on_upload = true;
    settings.prompt_pack_id = Some("general-default".to_string());
    service
        .save_project_evaluation_settings(ProjectEvaluationSettings { ..settings })
        .expect("settings should save");
}

fn select_model_provider(service: &CameraConnectorService, project_id: &str, settings_id: &str) {
    let mut settings = service
        .project_evaluation_settings(project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.model_provider_settings_id = Some(settings_id.to_string());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
}

fn select_prompt_pack(service: &CameraConnectorService, project_id: &str, prompt_pack_id: &str) {
    let mut settings = service
        .project_evaluation_settings(project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.prompt_pack_id = Some(prompt_pack_id.to_string());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
}

fn completed_transfer(transfer_id: &str, original_path: &str, at_ms: i64) -> TransferRecord {
    let final_filename = original_path
        .rsplit('/')
        .next()
        .unwrap_or(original_path)
        .to_string();
    TransferRecord {
        transfer_id: transfer_id.to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: original_path.to_string(),
        final_filename: final_filename.clone(),
        final_location: Some(StoredObjectLocation::local_path(final_filename)),
        size_bytes: 1024,
        username: Some("camera".to_string()),
        remote_addr: Some("127.0.0.1".to_string()),
        source_name: Some("HTTP Camera".to_string()),
        started_at_ms: at_ms,
        completed_at_ms: Some(at_ms),
        error: None,
    }
}

fn balanced_preview_sample() -> PreviewSample {
    PreviewSample {
        width: 4,
        height: 4,
        luma: vec![
            20, 40, 70, 90, 45, 80, 120, 145, 60, 110, 160, 205, 80, 130, 190, 235,
        ],
        red: None,
        green: None,
        blue: None,
        preview_source: Some("jpeg".to_string()),
    }
}

fn model_evaluation(
    project_id: &str,
    asset_group_id: &str,
    evaluator_version: &str,
    score: i64,
    summary: &str,
) -> ModelEvaluation {
    ModelEvaluation {
        evaluation_id: format!("evaluation-{asset_group_id}-{evaluator_version}"),
        run_id: format!("run-{asset_group_id}-{evaluator_version}"),
        project_id: project_id.to_string(),
        asset_group_id: asset_group_id.to_string(),
        evaluator_kind: ModelEvaluatorKind::LlmVlm,
        evaluator_version: evaluator_version.to_string(),
        status: ModelEvaluationStatus::Ready,
        score,
        tier: ModelEvaluationTier::from_score(score),
        selectable: score >= 50,
        summary: summary.to_string(),
        strengths: Vec::new(),
        weaknesses: Vec::new(),
        technical_warnings: Vec::new(),
        prompt_pack_id: None,
        prompt_pack_version: None,
        prompt_hash: None,
        created_at_ms: 1_500,
        updated_at_ms: 1_500,
    }
}

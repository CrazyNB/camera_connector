use camera_connector_core::{
    recommend_selection_with_model_provider, CameraConnectorService, ModelProviderKind,
    ModelProviderSelectionRequest, ModelProviderSettings, ModelSendMode, PromptPackContent,
    SelectionCandidateVisualInput, SelectionRecommendationScope, SelectionRecommendationStatus,
    SelectionSource,
};

use super::support::{
    completed_transfer, enable_upload_model_evaluation, model_evaluation, select_model_provider,
    select_prompt_pack, TestModelServer,
};

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
        .expect("prompt pack should create");
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
        .expect("prompt pack should create");
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

    let recommendation = recommend_selection_with_model_provider(ModelProviderSelectionRequest {
        project_id: "project-visual",
        scope: SelectionRecommendationScope::BurstGroup,
        subject_id: "burst-visual",
        evaluations: &[
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
        assessments: &[],
        candidate_visuals: &[
            SelectionCandidateVisualInput {
                asset_group_id: "group-a".to_string(),
                image_data_url: "data:image/jpeg;base64,ZmFrZS1h".to_string(),
            },
            SelectionCandidateVisualInput {
                asset_group_id: "group-b".to_string(),
                image_data_url: "data:image/jpeg;base64,ZmFrZS1i".to_string(),
            },
        ],
        now_ms: 42_000,
        provider: &provider,
        api_key: "sk-visual-selector",
        prompt_content: &PromptPackContent::new(
            "Prefer the frame with stronger photographic expression.",
        ),
    })
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

use camera_connector_core::{
    recommend_burst_group_from_model_evaluations, recommend_project_model_selections,
    CameraConnectorService, CvPolicy, EvaluationRunStatus, EvaluationRunTrigger, EvaluationRunType,
    ModelEvaluation, ModelEvaluationStatus, ModelEvaluationTier, ModelEvaluatorKind,
    ModelProviderKind, ModelProviderSettings, ModelSendMode, PreviewSample,
    ProjectEvaluationSettings, SelectionRecommendationScope, SelectionRecommendationStatus,
    SelectionSource, StoredObjectLocation, TechnicalAssessment, TechnicalAssessmentStatus,
    TechnicalGateStatus, TransferRecord, TransferStatus,
};

#[test]
fn burst_model_recommendation_selects_highest_selectable_evaluation() {
    let recommendation = recommend_burst_group_from_model_evaluations(
        "project-1",
        "burst-1",
        &[
            model_eval("project-1", "group-a", 71, true),
            model_eval("project-1", "group-best", 88, true),
            model_eval("project-1", "group-reject", 95, false),
        ],
        &[],
        9_000,
    );

    assert_eq!(
        recommendation.scope,
        SelectionRecommendationScope::BurstGroup
    );
    assert_eq!(recommendation.status, SelectionRecommendationStatus::Ready);
    assert_eq!(recommendation.source, SelectionSource::LocalStub);
    assert_eq!(recommendation.selected_asset_group_ids, vec!["group-best"]);
    assert!(recommendation
        .rejected_asset_group_ids
        .contains(&"group-reject".to_string()));
}

#[test]
fn burst_model_recommendation_can_have_no_selection() {
    let recommendation = recommend_burst_group_from_model_evaluations(
        "project-1",
        "burst-bad",
        &[
            model_eval("project-1", "group-a", 22, false),
            model_eval("project-1", "group-b", 30, false),
        ],
        &[],
        9_000,
    );

    assert_eq!(
        recommendation.status,
        SelectionRecommendationStatus::NoSelection
    );
    assert_eq!(recommendation.source, SelectionSource::LocalStub);
    assert!(recommendation.selected_asset_group_ids.is_empty());
}

#[test]
fn project_model_recommendation_selects_good_singletons_and_strong_burst_winners() {
    let burst_recommendation = recommend_burst_group_from_model_evaluations(
        "project-1",
        "burst-1",
        &[
            model_eval("project-1", "burst-weak", 58, true),
            model_eval("project-1", "burst-good", 76, true),
        ],
        &[],
        9_000,
    );
    let project_recommendation = recommend_project_model_selections(
        "project-1",
        &[
            model_eval("project-1", "single-good", 82, true),
            model_eval("project-1", "single-normal", 55, true),
            model_eval("project-1", "single-reject", 91, false),
            model_eval("project-1", "burst-good", 76, true),
        ],
        &[burst_recommendation],
        9_500,
    );

    assert_eq!(
        project_recommendation.scope,
        SelectionRecommendationScope::Project
    );
    assert_eq!(
        project_recommendation.status,
        SelectionRecommendationStatus::Ready
    );
    assert_eq!(project_recommendation.source, SelectionSource::LocalStub);
    assert_eq!(
        project_recommendation.selected_asset_group_ids,
        vec!["single-good", "burst-good"]
    );
    assert!(!project_recommendation
        .selected_asset_group_ids
        .contains(&"single-normal".to_string()));
}

#[test]
fn project_model_recommendation_respects_burst_winners() {
    let burst_recommendation = recommend_burst_group_from_model_evaluations(
        "project-1",
        "burst-1",
        &[
            model_eval("project-1", "burst-winner", 76, true),
            model_eval("project-1", "burst-runner-up", 74, true),
        ],
        &[],
        9_000,
    );
    let project_recommendation = recommend_project_model_selections(
        "project-1",
        &[
            model_eval("project-1", "single-good", 82, true),
            model_eval("project-1", "burst-winner", 76, true),
            model_eval("project-1", "burst-runner-up", 74, true),
        ],
        &[burst_recommendation],
        9_500,
    );

    assert_eq!(
        project_recommendation.selected_asset_group_ids,
        vec!["single-good", "burst-winner"]
    );
    assert!(!project_recommendation
        .selected_asset_group_ids
        .contains(&"burst-runner-up".to_string()));
}

#[test]
fn service_recommends_burst_group_from_model_evaluations() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Scoped Recommend Service")
        .expect("project should create");
    save_configured_provider(&service);
    enable_upload_model_evaluation(&service, &project.project_id);
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:model-1", "DCIM/100/IMG_8101.JPG", 1000),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:model-2", "DCIM/100/IMG_8102.JPG", 1100),
        )
        .expect("second transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    let first = &page.groups[0];
    let second = &page.groups[1];
    let burst_id = first
        .burst
        .as_ref()
        .expect("burst should exist")
        .burst_group_id
        .clone();
    service
        .assess_asset_group_preview_with_provider_configured(
            first.group_id.as_ref().expect("group id should exist"),
            flat_sample(16, 16, 128),
            "technical-v1",
            true,
        )
        .expect("first score should save");
    service
        .assess_asset_group_preview_with_provider_configured(
            second.group_id.as_ref().expect("group id should exist"),
            balanced_detail_sample(16, 16),
            "technical-v1",
            true,
        )
        .expect("second score should save");

    let recommendation = service
        .recommend_burst_group_from_model(&burst_id)
        .expect("selection recommendation should save");

    assert_eq!(
        recommendation.scope,
        SelectionRecommendationScope::BurstGroup
    );
    assert_eq!(
        recommendation.selected_asset_group_ids,
        vec![second
            .group_id
            .as_ref()
            .expect("group id should exist")
            .clone()]
    );
}

#[test]
fn service_preview_assessment_persists_technical_and_model_evaluation_records() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Evaluation Service")
        .expect("project should create");
    save_configured_provider(&service);
    enable_upload_model_evaluation(&service, &project.project_id);
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:eval-1", "DCIM/100/IMG_7001.JPG", 1000),
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

    service
        .assess_asset_group_preview_with_provider_configured(
            &group_id,
            balanced_detail_sample(16, 16),
            "technical-v1",
            true,
        )
        .expect("score should save");
    let store = service.storage_store().expect("store should open");
    let assessments = store
        .technical_assessments_for_asset_groups(&[group_id.clone()], "technical-v1")
        .expect("technical assessments should query");
    let evaluations = store
        .model_evaluations_for_asset_groups(&[group_id], "model-stub-v1")
        .expect("model evaluations should query");

    assert_eq!(assessments.len(), 1);
    assert_eq!(assessments[0].gate_status, TechnicalGateStatus::Pass);
    assert_eq!(evaluations.len(), 1);
    assert!(evaluations[0].selectable);
    assert!(!evaluations[0].run_id.is_empty());
}

#[test]
fn service_preview_assessment_uses_project_cv_policy() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Strict CV Policy")
        .expect("project should create");
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .unwrap_or_else(|| {
            ProjectEvaluationSettings::default_for_project(&project.project_id, 2_000)
        });
    settings.cv_policy = CvPolicy::Strict;
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:strict-policy", "DCIM/100/IMG_7101.JPG", 1000),
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

    service
        .assess_asset_group_preview(
            &group_id,
            scattered_highlight_sample(100, 100, 900),
            "technical-v1",
        )
        .expect("assessment should save");
    let assessments = service
        .storage_store()
        .expect("store should open")
        .technical_assessments_for_asset_groups(&[group_id], "technical-v1")
        .expect("technical assessments should query");

    assert_eq!(assessments.len(), 1);
    assert_eq!(assessments[0].gate_status, TechnicalGateStatus::Warn);
}

#[test]
fn manual_project_recommendation_creates_run_snapshot_and_project_scope_recommendation() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Manual Project Recommendation")
        .expect("project should create");
    save_configured_provider(&service);
    enable_upload_model_evaluation(&service, &project.project_id);
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:manual-1", "DCIM/100/IMG_9401.JPG", 1000),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:manual-2", "DCIM/100/IMG_9402.JPG", 1100),
        )
        .expect("second transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    for group in &page.groups {
        service
            .assess_asset_group_preview_with_provider_configured(
                group.group_id.as_ref().expect("group id should exist"),
                balanced_detail_sample(16, 16),
                "technical-v1",
                true,
            )
            .expect("score should save");
    }
    service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("burst recommendation should drain");

    let recommendation = service
        .generate_project_recommendation(&project.project_id, 20_000)
        .expect("manual project recommendation should run");
    let run_id = recommendation
        .run_id
        .clone()
        .expect("project recommendation should reference run id");
    let run = service
        .storage_store()
        .expect("store should open")
        .latest_evaluation_run(
            &project.project_id,
            EvaluationRunType::ProjectRecommendation,
        )
        .expect("run should query")
        .expect("run should exist");

    assert_eq!(recommendation.scope, SelectionRecommendationScope::Project);
    assert_eq!(recommendation.subject_id, project.project_id);
    assert_eq!(recommendation.run_id.as_deref(), Some(run_id.as_str()));
    assert_eq!(run.run_id, run_id);
    assert_eq!(run.trigger, EvaluationRunTrigger::Manual);
    assert_eq!(run.status, EvaluationRunStatus::Ready);
}

#[test]
fn manual_project_recommendation_excludes_burst_members_without_scoped_winner() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Manual Project Recommendation Without Burst Winner")
        .expect("project should create");
    save_configured_provider(&service);
    enable_upload_model_evaluation(&service, &project.project_id);
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:manual-no-winner-1", "DCIM/100/IMG_9501.JPG", 1000),
        )
        .expect("first burst transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:manual-no-winner-2", "DCIM/100/IMG_9502.JPG", 1100),
        )
        .expect("second burst transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:manual-single", "DCIM/100/IMG_9600.JPG", 10_000),
        )
        .expect("singleton transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    let burst_group_ids = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .map(|group| group.group_id.as_ref().expect("burst group id").clone())
        .collect::<Vec<_>>();
    let singleton_group_id = page
        .groups
        .iter()
        .find(|group| group.burst.is_none())
        .and_then(|group| group.group_id.clone())
        .expect("singleton group id should exist");
    assert_eq!(burst_group_ids.len(), 2);

    for group in &page.groups {
        service
            .assess_asset_group_preview_with_provider_configured(
                group.group_id.as_ref().expect("group id should exist"),
                balanced_detail_sample(16, 16),
                "technical-v1",
                true,
            )
            .expect("score should save");
    }

    let recommendation = service
        .generate_project_recommendation(&project.project_id, 21_000)
        .expect("manual project recommendation should run");

    assert_eq!(
        recommendation.selected_asset_group_ids,
        vec![singleton_group_id]
    );
    assert!(burst_group_ids
        .iter()
        .all(|group_id| !recommendation.selected_asset_group_ids.contains(group_id)));
}

#[test]
fn manual_project_recommendation_requires_configured_provider() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Manual Project Recommendation No Provider")
        .expect("project should create");

    assert!(service
        .generate_project_recommendation(&project.project_id, 20_000)
        .is_err());
}

fn model_eval(
    project_id: &str,
    asset_group_id: &str,
    score: i64,
    selectable: bool,
) -> ModelEvaluation {
    ModelEvaluation {
        evaluation_id: format!("evaluation-{asset_group_id}"),
        run_id: format!("run-{asset_group_id}"),
        project_id: project_id.to_string(),
        asset_group_id: asset_group_id.to_string(),
        evaluator_kind: ModelEvaluatorKind::LocalStub,
        evaluator_version: "model-stub-v1".to_string(),
        status: ModelEvaluationStatus::Ready,
        score,
        tier: ModelEvaluationTier::from_score(score),
        selectable,
        summary: "test model evaluation".to_string(),
        strengths: Vec::new(),
        weaknesses: Vec::new(),
        technical_warnings: Vec::new(),
        prompt_profile_id: None,
        prompt_version_id: None,
        prompt_hash: None,
        created_at_ms: 1_000,
        updated_at_ms: 1_000,
    }
}

fn save_configured_provider(service: &CameraConnectorService) {
    service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "global".to_string(),
            provider_kind: ModelProviderKind::Imported,
            provider_label: "Imported evaluator".to_string(),
            base_url: "local://imported".to_string(),
            default_model: "imported-local".to_string(),
            default_max_image_side: 1600,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 4,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 1_000,
        })
        .expect("provider settings should save");
}

fn enable_upload_model_evaluation(service: &CameraConnectorService, project_id: &str) {
    let mut settings = service
        .project_evaluation_settings(project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.auto_evaluate_on_upload = true;
    settings.prompt_profile_id = Some("general-default".to_string());
    settings.model_provider_settings_id = Some("global".to_string());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
}

#[allow(dead_code)]
fn technical_assessment(
    asset_group_id: &str,
    gate_status: TechnicalGateStatus,
) -> TechnicalAssessment {
    TechnicalAssessment {
        asset_group_id: asset_group_id.to_string(),
        assessor_version: "technical-v1".to_string(),
        status: TechnicalAssessmentStatus::Ready,
        gate_status,
        defect_flags: Vec::new(),
        preview_source: Some("test".to_string()),
        visual_signature: None,
        analyzed_at_ms: 1_000,
    }
}

fn flat_sample(width: usize, height: usize, value: u8) -> PreviewSample {
    PreviewSample {
        width,
        height,
        luma: vec![value; width * height],
        red: None,
        green: None,
        blue: None,
        preview_source: Some("test".to_string()),
    }
}

fn balanced_detail_sample(width: usize, height: usize) -> PreviewSample {
    let mut luma = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            luma.push(if (x + y) % 2 == 0 { 96 } else { 160 });
        }
    }
    PreviewSample {
        width,
        height,
        luma,
        red: None,
        green: None,
        blue: None,
        preview_source: Some("test".to_string()),
    }
}

fn scattered_highlight_sample(width: usize, height: usize, clipped_count: usize) -> PreviewSample {
    let mut luma = vec![128; width * height];
    for i in 0..clipped_count.min(luma.len()) {
        let index = (i * 37) % luma.len();
        luma[index] = 248;
    }
    PreviewSample {
        width,
        height,
        luma,
        red: None,
        green: None,
        blue: None,
        preview_source: Some("test".to_string()),
    }
}

fn completed_transfer(
    transfer_id: &str,
    original_path: &str,
    started_at_ms: i64,
) -> TransferRecord {
    let final_filename = original_path.rsplit('/').next().unwrap().to_string();
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
        source_name: Some("Nikon Z".to_string()),
        started_at_ms,
        completed_at_ms: Some(started_at_ms),
        error: None,
    }
}

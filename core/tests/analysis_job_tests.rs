use camera_connector_core::{
    AnalysisEntityType, AnalysisJobStatus, AnalysisJobType, AssetGroupQuery,
    CameraConnectorService, EvaluationRunStatus, EvaluationRunTrigger, EvaluationRunType,
    ModelProviderKind, ModelProviderSettings, ModelSendMode, NewAnalysisJob, PreviewSample,
    PublishTransferMetadata, SelectionRecommendationScope, SelectionSource, SqliteStore,
    StoredObjectLocation, TechnicalAssessment, TechnicalAssessmentStatus, TechnicalGateStatus,
    TransferRecord, TransferStatus,
};

#[test]
fn analysis_jobs_dedupe_by_key() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Job Project")
        .expect("project should create");
    let job = NewAnalysisJob::new(
        &project.project_id,
        AnalysisJobType::DetectBurstForAssetGroup,
        AnalysisEntityType::AssetGroup,
        "group-1",
        "burst:project:camera:1000",
    );

    let first = store
        .enqueue_analysis_job(job.clone())
        .expect("first job should enqueue");
    let second = store
        .enqueue_analysis_job(job)
        .expect("duplicate job should reuse active row");

    assert_eq!(first.job_id, second.job_id);
    assert_eq!(first.status, AnalysisJobStatus::Pending);
}

#[test]
fn analysis_job_type_round_trips_evaluation_pipeline_values() {
    assert_eq!(
        AnalysisJobType::from_str("assess_asset_group_technical_quality"),
        AnalysisJobType::AssessAssetGroupTechnicalQuality
    );
    assert_eq!(
        AnalysisJobType::from_str("assess_portrait_subject"),
        AnalysisJobType::AssessPortraitSubject
    );
    assert_eq!(
        AnalysisJobType::EvaluateAssetGroupWithModel.as_str(),
        "evaluate_asset_group_with_model"
    );
    assert_eq!(
        AnalysisJobType::from_str("recommend_project_selects"),
        AnalysisJobType::RecommendProjectSelects
    );
    assert_eq!(AnalysisEntityType::Project.as_str(), "project");
    assert_eq!(
        AnalysisEntityType::from_str("project"),
        AnalysisEntityType::Project
    );
}

#[test]
fn analysis_jobs_claim_in_priority_order() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Claim Project")
        .expect("project should create");
    let mut low = NewAnalysisJob::new(
        &project.project_id,
        AnalysisJobType::ScoreAssetGroup,
        AnalysisEntityType::AssetGroup,
        "group-low",
        "score:group-low:local-v1",
    );
    low.priority = 5;
    let mut high = NewAnalysisJob::new(
        &project.project_id,
        AnalysisJobType::RecommendBurstGroup,
        AnalysisEntityType::BurstGroup,
        "burst-high",
        "recommend:burst-high:general:v1",
    );
    high.priority = 50;
    store.enqueue_analysis_job(low).expect("low should enqueue");
    store
        .enqueue_analysis_job(high)
        .expect("high should enqueue");

    let claimed = store
        .claim_analysis_jobs(10_000, 2)
        .expect("jobs should claim");

    assert_eq!(claimed.len(), 2);
    assert_eq!(claimed[0].entity_id, "burst-high");
    assert_eq!(claimed[0].status, AnalysisJobStatus::Running);
    assert_eq!(claimed[1].entity_id, "group-low");
}

#[test]
fn analysis_job_failure_sets_next_attempt() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Retry Project")
        .expect("project should create");
    let job = store
        .enqueue_analysis_job(NewAnalysisJob::new(
            &project.project_id,
            AnalysisJobType::ScoreAssetGroup,
            AnalysisEntityType::AssetGroup,
            "group-1",
            "score:group-1:local-v1",
        ))
        .expect("job should enqueue");

    store
        .fail_analysis_job(&job.job_id, "decode failed", 12_345)
        .expect("job should fail");
    let claimed = store
        .claim_analysis_jobs(12_344, 5)
        .expect("jobs should query before retry");

    assert!(claimed.is_empty());

    let claimed = store
        .claim_analysis_jobs(12_345, 5)
        .expect("jobs should query at retry");

    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].attempts, 1);
    assert_eq!(claimed[0].last_error.as_deref(), Some("decode failed"));
}

#[test]
fn complete_publish_enqueues_burst_detection_job() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Publish Project")
        .expect("project should create");
    let item = store
        .enqueue_publish_with_metadata(
            &project.project_id,
            "ftp:publish",
            "staged/publish.tmp",
            "IMG_1001.JPG",
            123,
            PublishTransferMetadata {
                protocol: "ftp".to_string(),
                original_path: "DCIM/100/IMG_1001.JPG".to_string(),
                username: Some("z5".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Z5".to_string()),
                started_at_ms: 1000,
            },
        )
        .expect("publish item should enqueue");

    store
        .complete_publish(
            &item.queue_id,
            "IMG_1001.JPG",
            StoredObjectLocation::local_path("IMG_1001.JPG"),
        )
        .expect("publish should complete");
    let jobs = store
        .claim_analysis_jobs(i64::MAX, 10)
        .expect("analysis jobs should claim");

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].job_type, AnalysisJobType::DetectBurstForAssetGroup);
    assert_eq!(jobs[0].entity_type, AnalysisEntityType::AssetGroup);
    assert!(jobs[0].entity_id.starts_with("group-"));
}

#[test]
fn service_analysis_drain_persists_burst_summary_for_asset_page() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Drain Project")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:2001", "DCIM/100/IMG_2001.JPG", 1000),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:2002", "DCIM/100/IMG_2002.JPG", 1100),
        )
        .expect("second transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:2003", "DCIM/100/IMG_2003.JPG", 1200),
        )
        .expect("third transfer should record");

    let summary = service
        .drain_analysis_jobs(10)
        .expect("analysis jobs should drain");

    assert_eq!(summary.claimed_count, 3);
    assert_eq!(summary.completed_count, 3);
    assert_eq!(summary.failed_count, 0);

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should load");
    let burst_members = page
        .groups
        .iter()
        .filter_map(|group| group.burst.as_ref())
        .collect::<Vec<_>>();

    assert_eq!(burst_members.len(), 3);
    assert!(burst_members
        .iter()
        .all(|burst| burst.member_count == 3 && burst.recommendation_status == "pending"));
}

#[test]
fn scoring_preview_enqueues_and_drains_recommendation_job() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Score Recommend Job")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:score-job-1", "DCIM/100/IMG_6101.JPG", 1000),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:score-job-2", "DCIM/100/IMG_6102.JPG", 1100),
        )
        .expect("second transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should load");
    let soft_group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("soft group id should exist")
        .clone();
    let sharp_group_id = page.groups[1]
        .group_id
        .as_ref()
        .expect("sharp group id should exist")
        .clone();
    service
        .score_asset_group_preview(&soft_group_id, flat_sample(16, 16, 128), "local-v1")
        .expect("soft score should save");
    service
        .score_asset_group_preview(&sharp_group_id, balanced_detail_sample(16, 16), "local-v1")
        .expect("sharp score should save");

    let summary = service
        .drain_analysis_jobs(10)
        .expect("recommendation job should drain");

    assert_eq!(summary.claimed_count, 1);
    assert_eq!(summary.completed_count, 1);
    assert_eq!(summary.failed_count, 0);

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should reload");
    let burst = page.groups[0]
        .burst
        .as_ref()
        .expect("burst summary should exist");

    assert_eq!(burst.recommendation_status, "ready");
    assert_eq!(
        burst.best_asset_group_id.as_deref(),
        Some(sharp_group_id.as_str())
    );
    let scoped = service
        .storage_store()
        .expect("store should open")
        .latest_scoped_selection_recommendation(
            &project.project_id,
            SelectionRecommendationScope::BurstGroup,
            &burst.burst_group_id,
        )
        .expect("scoped recommendation should query");
    assert!(
        scoped.is_none(),
        "model-scoped burst recommendation should wait for model evaluations"
    );
    let project_scoped = service
        .storage_store()
        .expect("store should open")
        .latest_scoped_selection_recommendation(
            &project.project_id,
            SelectionRecommendationScope::Project,
            &project.project_id,
        )
        .expect("project scoped recommendation should query");
    assert!(
        project_scoped.is_none(),
        "automatic drains must not produce project-scope recommendations"
    );
}

#[test]
fn model_evaluation_job_uses_saved_technical_assessment() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Model Evaluation Job")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:job-eval", "DCIM/100/IMG_3401.JPG", 1000),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();
    let store = service.storage_store().expect("store should open");
    store
        .save_technical_assessment(TechnicalAssessment {
            asset_group_id: group_id.clone(),
            assessor_version: "technical-v1".to_string(),
            status: TechnicalAssessmentStatus::Ready,
            gate_status: TechnicalGateStatus::Pass,
            defect_flags: Vec::new(),
            preview_source: Some("test".to_string()),
            analyzed_at_ms: 10_000,
        })
        .expect("assessment should save");
    store
        .enqueue_analysis_job(NewAnalysisJob::new(
            &project.project_id,
            AnalysisJobType::EvaluateAssetGroupWithModel,
            AnalysisEntityType::AssetGroup,
            &group_id,
            &format!("model-eval:{group_id}:model-stub-v1"),
        ))
        .expect("model evaluation job should enqueue");

    save_configured_provider(&service);
    enable_upload_model_evaluation(&service, &project.project_id, true);

    let summary = service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("model evaluation job should drain");
    let evaluations = store
        .model_evaluations_for_asset_groups(std::slice::from_ref(&group_id), "model-stub-v1")
        .expect("model evaluation should query");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should load after model evaluation exists");

    assert_eq!(summary.completed_count, 1);
    assert_eq!(evaluations.len(), 1);
    assert!(evaluations[0].selectable);
    assert!(!evaluations[0].run_id.is_empty());
    assert!(page
        .groups
        .iter()
        .any(|group| group.model_status.as_deref() == Some("ready")
            && group.model_evaluator_kind.as_deref() == Some("local_stub")));
}

#[test]
fn model_evaluation_job_skips_when_project_model_evaluation_is_disabled() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Disabled Model Evaluation Job")
        .expect("project should create");
    let store = service.storage_store().expect("store should open");
    store
        .save_technical_assessment(TechnicalAssessment {
            asset_group_id: "group-job-disabled".to_string(),
            assessor_version: "technical-v1".to_string(),
            status: TechnicalAssessmentStatus::Ready,
            gate_status: TechnicalGateStatus::Pass,
            defect_flags: Vec::new(),
            preview_source: Some("test".to_string()),
            analyzed_at_ms: 10_000,
        })
        .expect("assessment should save");
    store
        .enqueue_analysis_job(NewAnalysisJob::new(
            &project.project_id,
            AnalysisJobType::EvaluateAssetGroupWithModel,
            AnalysisEntityType::AssetGroup,
            "group-job-disabled",
            "model-eval:group-job-disabled:model-stub-v1",
        ))
        .expect("model evaluation job should enqueue");

    let summary = service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("model evaluation job should drain");
    let evaluations = store
        .model_evaluations_for_asset_groups(&["group-job-disabled".to_string()], "model-stub-v1")
        .expect("model evaluation should query");

    assert_eq!(summary.completed_count, 1);
    assert!(evaluations.is_empty());
}

#[test]
fn score_preview_without_provider_keeps_technical_assessment_but_skips_model_evaluation() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("No Provider Evaluation")
        .expect("project should create");
    enable_upload_model_evaluation(&service, &project.project_id, true);
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:no-provider-1", "DCIM/100/IMG_9201.JPG", 1000),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, false)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    service
        .score_asset_group_preview_with_provider_configured(
            &group_id,
            balanced_detail_sample(16, 16),
            "local-v1",
            false,
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
    assert!(evaluations.is_empty());
}

#[test]
fn automatic_burst_model_recommendation_creates_run_snapshot() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Automatic Burst Model Run")
        .expect("project should create");
    save_configured_provider(&service);
    enable_upload_model_evaluation(&service, &project.project_id, true);
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:auto-model-run-1", "DCIM/100/IMG_9701.JPG", 1000),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:auto-model-run-2", "DCIM/100/IMG_9702.JPG", 1100),
        )
        .expect("second transfer should record");
    service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("model evaluation jobs should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should load");
    for group in &page.groups {
        service
            .score_asset_group_preview_with_provider_configured(
                group.group_id.as_ref().expect("group id should exist"),
                balanced_detail_sample(16, 16),
                "local-v1",
                true,
            )
            .expect("score should save");
    }
    service
        .drain_analysis_jobs_with_provider_configured(10, true)
        .expect("burst recommendation should drain");
    let store = service.storage_store().expect("store should open");
    let burst_id = page.groups[0]
        .burst
        .as_ref()
        .expect("burst summary should exist")
        .burst_group_id
        .clone();
    let recommendation = store
        .latest_scoped_selection_recommendation(
            &project.project_id,
            SelectionRecommendationScope::BurstGroup,
            &burst_id,
        )
        .expect("recommendation should query")
        .expect("scoped recommendation should exist");
    let run_id = recommendation
        .run_id
        .as_deref()
        .expect("burst recommendation should reference run id");
    let run = store
        .latest_evaluation_run(&project.project_id, EvaluationRunType::BurstRecommendation)
        .expect("run should query")
        .expect("run should exist");

    assert_eq!(recommendation.source, SelectionSource::LocalStub);
    assert_eq!(run.run_id, run_id);
    assert_eq!(run.trigger, EvaluationRunTrigger::BurstStable);
    assert_eq!(run.status, EvaluationRunStatus::Ready);
    assert_eq!(
        run.prompt_hash.as_deref(),
        Some("builtin-general-default-v1")
    );
    assert!(run.settings_snapshot_json.contains("\"prompt_profile_id\""));
}

#[test]
fn scoring_preview_obeys_auto_burst_recommendation_setting() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("No Auto Burst Recommend")
        .expect("project should create");
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.auto_burst_recommendation_enabled = false;
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:auto-burst-1", "DCIM/100/IMG_9301.JPG", 1000),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:auto-burst-2", "DCIM/100/IMG_9302.JPG", 1100),
        )
        .expect("second transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should load");
    let first_group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();
    let second_group_id = page.groups[1]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    service
        .score_asset_group_preview(&first_group_id, flat_sample(16, 16, 128), "local-v1")
        .expect("first score should save");
    service
        .score_asset_group_preview(&second_group_id, balanced_detail_sample(16, 16), "local-v1")
        .expect("second score should save");
    let summary = service
        .drain_analysis_jobs(10)
        .expect("recommendation drain should run");

    assert_eq!(summary.claimed_count, 0);
    assert!(service
        .storage_store()
        .expect("store should open")
        .latest_scoped_selection_recommendation(
            &project.project_id,
            SelectionRecommendationScope::BurstGroup,
            page.groups[0]
                .burst
                .as_ref()
                .expect("burst should exist")
                .burst_group_id
                .as_str(),
        )
        .expect("burst recommendation should query")
        .is_none());
}

#[test]
fn scoring_preview_does_not_enqueue_or_drain_project_model_selects_job() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Project Model Select Job")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:project-select-1", "DCIM/100/IMG_9101.JPG", 1000),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    service
        .score_asset_group_preview(&group_id, balanced_detail_sample(16, 16), "local-v1")
        .expect("score should save");
    let summary = service
        .drain_analysis_jobs(10)
        .expect("automatic drain should not run project selects");
    let recommendation = service
        .storage_store()
        .expect("store should open")
        .latest_scoped_selection_recommendation(
            &project.project_id,
            SelectionRecommendationScope::Project,
            &project.project_id,
        )
        .expect("project recommendation should query");

    assert_eq!(summary.claimed_count, 0);
    assert!(recommendation.is_none());
}

#[test]
fn project_selects_analysis_job_is_ignored_as_manual_only_without_retry() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Manual Only Project Select Job")
        .expect("project should create");
    let store = service.storage_store().expect("store should open");
    store
        .enqueue_analysis_job(NewAnalysisJob::new(
            &project.project_id,
            AnalysisJobType::RecommendProjectSelects,
            AnalysisEntityType::Project,
            &project.project_id,
            "recommend-project-selects:manual-only-test",
        ))
        .expect("project selects job should enqueue");

    let summary = service
        .drain_analysis_jobs(10)
        .expect("drain should ignore stale project selects job");
    let recommendation = store
        .latest_scoped_selection_recommendation(
            &project.project_id,
            SelectionRecommendationScope::Project,
            &project.project_id,
        )
        .expect("project recommendation should query");
    let claimed_after_ignore = store
        .claim_analysis_jobs(i64::MAX, 10)
        .expect("ignored job should not be claimable again");

    assert_eq!(summary.claimed_count, 1);
    assert_eq!(summary.completed_count, 1);
    assert_eq!(summary.failed_count, 0);
    assert!(recommendation.is_none());
    assert!(claimed_after_ignore.is_empty());
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

fn enable_upload_model_evaluation(
    service: &CameraConnectorService,
    project_id: &str,
    auto_evaluate_on_upload: bool,
) {
    let mut settings = service
        .project_evaluation_settings(project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.model_evaluation_enabled = true;
    settings.auto_evaluate_on_upload = auto_evaluate_on_upload;
    settings.prompt_profile_id = Some("general-default".to_string());
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
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
        final_filename,
        final_path: None,
        final_location: Some(StoredObjectLocation::local_path(original_path)),
        size_bytes: 100,
        username: Some("z5".to_string()),
        remote_addr: Some("192.168.137.56".to_string()),
        source_name: Some("Nikon Z".to_string()),
        started_at_ms,
        completed_at_ms: Some(started_at_ms),
        error: None,
    }
}

fn flat_sample(width: usize, height: usize, value: u8) -> PreviewSample {
    PreviewSample {
        width,
        height,
        luma: vec![value; width * height],
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
        preview_source: Some("test".to_string()),
    }
}

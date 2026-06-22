use super::*;

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
    let store = service.storage_store().expect("store should open");
    let burst_id = page.groups[0]
        .burst
        .as_ref()
        .expect("burst summary should exist")
        .burst_group_id
        .clone();
    let recommendation = store
        .latest_selection_recommendation(
            &project.project_id,
            SelectionRecommendationScope::BurstGroup,
            &burst_id,
        )
        .expect("recommendation should query")
        .expect("selection recommendation should exist");
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
    assert!(run
        .prompt_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("fnv1a64-")));
    assert!(run.settings_snapshot_json.contains("\"prompt_pack_id\""));
}

#[test]
fn preview_assessment_obeys_auto_burst_recommendation_setting() {
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
        .assess_asset_group_preview(&first_group_id, flat_sample(16, 16, 128), "technical-v1")
        .expect("first score should save");
    service
        .assess_asset_group_preview(
            &second_group_id,
            balanced_detail_sample(16, 16),
            "technical-v1",
        )
        .expect("second score should save");
    let summary = service
        .drain_analysis_jobs(10)
        .expect("recommendation drain should run");

    assert_eq!(summary.claimed_count, 0);
    assert!(service
        .storage_store()
        .expect("store should open")
        .latest_selection_recommendation(
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
fn preview_assessment_does_not_enqueue_or_drain_project_recommendation_job() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Project Recommendation Job")
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
        .assess_asset_group_preview(&group_id, balanced_detail_sample(16, 16), "technical-v1")
        .expect("technical assessment should save");
    let summary = service
        .drain_analysis_jobs(10)
        .expect("automatic drain should not run project recommendations");
    let recommendation = service
        .storage_store()
        .expect("store should open")
        .latest_selection_recommendation(
            &project.project_id,
            SelectionRecommendationScope::Project,
            &project.project_id,
        )
        .expect("project recommendation should query");

    assert_eq!(summary.claimed_count, 0);
    assert!(recommendation.is_none());
}

#[test]
fn project_recommendation_analysis_job_is_ignored_as_manual_only_without_retry() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Manual Only Project Recommendation Job")
        .expect("project should create");
    let store = service.storage_store().expect("store should open");
    store
        .enqueue_analysis_job(NewAnalysisJob::new(
            &project.project_id,
            AnalysisJobType::GenerateProjectRecommendation,
            AnalysisEntityType::Project,
            &project.project_id,
            "generate-project-recommendation:manual-only-test",
        ))
        .expect("project recommendation job should enqueue");

    let summary = service
        .drain_analysis_jobs(10)
        .expect("drain should ignore project recommendation job");
    let recommendation = store
        .latest_selection_recommendation(
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

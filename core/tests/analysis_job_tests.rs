use camera_connector_core::{
    AnalysisEntityType, AnalysisJobStatus, AnalysisJobType, AssetGroupQuery,
    CameraConnectorService, NewAnalysisJob, PreviewSample, PublishTransferMetadata, SqliteStore,
    StoredObjectLocation, TransferRecord, TransferStatus,
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
        .score_asset_group_preview(&sharp_group_id, checkerboard_sample(16, 16), "local-v1")
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

fn checkerboard_sample(width: usize, height: usize) -> PreviewSample {
    let mut luma = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            luma.push(if (x + y) % 2 == 0 { 0 } else { 255 });
        }
    }
    PreviewSample {
        width,
        height,
        luma,
        preview_source: Some("test".to_string()),
    }
}

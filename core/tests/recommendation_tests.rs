use camera_connector_core::{
    recommend_from_scores, CameraConnectorService, PreviewSample, QualityAnalysisStatus,
    QualityScore, SelectionRecommendationStatus, SelectionSource, SignalScore,
    StoredObjectLocation, StrategyProfile, TransferRecord, TransferStatus,
};

#[test]
fn general_profile_selects_sharpest_balanced_frame() {
    let profile = StrategyProfile::general();
    let recommendation = recommend_from_scores(
        "burst-1",
        &profile,
        &[
            score("group-soft", 0.45, 0.80, 0.65),
            score("group-best", 0.88, 0.74, 0.62),
        ],
        2,
        10_000,
    );

    assert_eq!(recommendation.status, SelectionRecommendationStatus::Ready);
    assert_eq!(recommendation.source, SelectionSource::LocalCv);
    assert_eq!(
        recommendation.best_asset_group_id.as_deref(),
        Some("group-best")
    );
    assert_eq!(recommendation.alternate_asset_group_ids, vec!["group-soft"]);
}

#[test]
fn composition_cannot_promote_low_sharpness_frame_to_best() {
    let profile = StrategyProfile::general();
    let recommendation = recommend_from_scores(
        "burst-2",
        &profile,
        &[
            score("group-pretty-soft", 0.12, 0.90, 1.00),
            score("group-technical", 0.52, 0.66, 0.42),
        ],
        1,
        10_000,
    );

    assert_eq!(
        recommendation.best_asset_group_id.as_deref(),
        Some("group-technical")
    );
    assert!(recommendation
        .low_score_asset_group_ids
        .contains(&"group-pretty-soft".to_string()));
}

#[test]
fn unsupported_scores_produce_needs_review() {
    let profile = StrategyProfile::general();
    let unsupported = QualityScore {
        analysis_status: QualityAnalysisStatus::Unsupported,
        sharpness: SignalScore::unavailable(),
        exposure: SignalScore::unavailable(),
        composition: SignalScore::unavailable(),
        overall: 0.0,
        reasons: vec!["unsupported preview sample".to_string()],
        ..score("group-unsupported", 0.0, 0.0, 0.0)
    };

    let recommendation =
        recommend_from_scores("burst-unsupported", &profile, &[unsupported], 1, 10_000);

    assert_eq!(
        recommendation.status,
        SelectionRecommendationStatus::NeedsReview
    );
    assert_eq!(recommendation.best_asset_group_id, None);
}

#[test]
fn service_scores_preview_samples_and_recommends_burst_group() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(Some(temp_dir.path().join("config.json")));
    let project = service
        .create_project("Recommend Service")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:5001", "DCIM/100/IMG_5001.JPG", 1000),
        )
        .expect("first transfer should record");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:5002", "DCIM/100/IMG_5002.JPG", 1100),
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
        .score_asset_group_preview(
            first.group_id.as_ref().expect("group id should exist"),
            flat_sample(16, 16, 128),
            "local-v1",
        )
        .expect("first score should save");
    service
        .score_asset_group_preview(
            second.group_id.as_ref().expect("group id should exist"),
            checkerboard_sample(16, 16),
            "local-v1",
        )
        .expect("second score should save");

    let recommendation = service
        .recommend_burst_group(&burst_id, None)
        .expect("recommendation should save");

    assert_eq!(recommendation.status, SelectionRecommendationStatus::Ready);
    assert_eq!(
        recommendation.best_asset_group_id.as_deref(),
        second.group_id.as_deref()
    );
}

fn score(group_id: &str, sharpness: f64, exposure: f64, composition: f64) -> QualityScore {
    let overall = sharpness * 0.55 + exposure * 0.30 + composition * 0.15;
    QualityScore {
        asset_group_id: group_id.to_string(),
        preview_source: Some("test".to_string()),
        scorer_version: "local-v1".to_string(),
        analysis_status: QualityAnalysisStatus::Ready,
        exif_status: None,
        capture_time_ms: None,
        sharpness: SignalScore::ready(sharpness),
        exposure: SignalScore::ready(exposure),
        highlight_clipping_penalty: SignalScore::ready(0.0),
        shadow_clipping_penalty: SignalScore::ready(0.0),
        composition: SignalScore::ready(composition),
        composition_confidence: 0.8,
        similarity_cluster_id: None,
        overall,
        reasons: vec!["test score".to_string()],
        analyzed_at_ms: 1_000,
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
        final_path: None,
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

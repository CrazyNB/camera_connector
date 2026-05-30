use camera_connector_core::{
    AssetGroupQuery, AssetGroupSort, QualityAnalysisStatus, QualityScore, SignalScore, SqliteStore,
    StoredObjectLocation, StrategyProfile, TransferRecord, TransferStatus,
};

#[test]
fn asset_groups_sort_by_group_best_score() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Score Sort")
        .expect("project should create");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:low", "DCIM/100/IMG_3001.JPG", 1000),
        )
        .expect("low transfer should record");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:high", "DCIM/100/IMG_3002.JPG", 1100),
        )
        .expect("high transfer should record");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:unscored", "DCIM/100/IMG_3003.JPG", 1200),
        )
        .expect("unscored transfer should record");
    save_score_for_display_key(&store, &project.project_id, "IMG_3001", 0.42);
    save_score_for_display_key(&store, &project.project_id, "IMG_3002", 0.91);

    let page = store
        .asset_group_page(
            &project.project_id,
            AssetGroupQuery {
                sort: AssetGroupSort::GroupBestScore,
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("asset groups should query");

    assert_eq!(page.groups[0].group_key, "IMG_3002");
    assert_eq!(page.groups[1].group_key, "IMG_3001");
    assert_eq!(page.groups[2].group_key, "IMG_3003");
}

#[test]
fn asset_groups_filter_by_score_range_and_analysis_status() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Score Filter")
        .expect("project should create");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:keep", "DCIM/100/IMG_4001.JPG", 1000),
        )
        .expect("keep transfer should record");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:drop", "DCIM/100/IMG_4002.JPG", 1100),
        )
        .expect("drop transfer should record");
    save_score_for_display_key(&store, &project.project_id, "IMG_4001", 0.86);
    save_score_for_display_key(&store, &project.project_id, "IMG_4002", 0.51);

    let page = store
        .asset_group_page(
            &project.project_id,
            AssetGroupQuery {
                score_min: Some(80.0),
                analysis_status: Some("ready".to_string()),
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("asset groups should query");

    assert_eq!(page.groups.len(), 1);
    assert_eq!(page.groups[0].group_key, "IMG_4001");
    assert_eq!(page.groups[0].quality.as_ref().unwrap().overall, 0.86);
}

#[test]
fn asset_group_quality_summary_exposes_signal_scores() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Signal Summary")
        .expect("project should create");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:signal", "DCIM/100/IMG_4100.JPG", 1000),
        )
        .expect("signal transfer should record");
    save_detailed_score_for_display_key(
        &store,
        &project.project_id,
        "IMG_4100",
        DetailedScore {
            overall: 0.82,
            sharpness: 0.73,
            exposure: 0.66,
            highlight_clipping_penalty: 0.08,
            shadow_clipping_penalty: 0.12,
            composition: 0.58,
            composition_confidence: 0.71,
        },
    );

    let page = store
        .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset groups should query");
    let quality = page.groups[0]
        .quality
        .as_ref()
        .expect("quality summary should exist");

    assert!((quality.overall - 0.82).abs() < 0.0001);
    assert_eq!(Some(0.73), quality.sharpness);
    assert_eq!(Some(0.66), quality.exposure);
    assert_eq!(Some(0.08), quality.highlight_clipping_penalty);
    assert_eq!(Some(0.12), quality.shadow_clipping_penalty);
    assert_eq!(Some(0.58), quality.composition);
    assert_eq!(Some(0.71), quality.composition_confidence);
}

#[test]
fn asset_groups_filter_by_score_range_uses_burst_best_score() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Burst Score Filter")
        .expect("project should create");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:best", "DCIM/100/IMG_5001.JPG", 1000),
        )
        .expect("best transfer should record");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:soft", "DCIM/100/IMG_5002.JPG", 1100),
        )
        .expect("soft transfer should record");
    let mut low_transfer = completed_transfer("ftp:low", "DCIM/100/IMG_5003.JPG", 3000);
    low_transfer.username = Some("z6".to_string());
    store
        .record_transfer(&project.project_id, low_transfer)
        .expect("low transfer should record");
    let first_group = store
        .stored_asset_groups(&project.project_id)
        .expect("groups should query")
        .into_iter()
        .find(|group| group.display_key == "IMG_5001")
        .expect("first group should exist")
        .group_id;
    store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group,
            &StrategyProfile::general(),
        )
        .expect("burst should detect");
    save_score_for_display_key(&store, &project.project_id, "IMG_5001", 0.92);
    save_score_for_display_key(&store, &project.project_id, "IMG_5002", 0.55);
    save_score_for_display_key(&store, &project.project_id, "IMG_5003", 0.45);

    let page = store
        .asset_group_page(
            &project.project_id,
            AssetGroupQuery {
                score_min: Some(80.0),
                sort: AssetGroupSort::GroupBestScore,
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("asset groups should query");

    assert_eq!(
        vec!["IMG_5001", "IMG_5002"],
        page.groups
            .iter()
            .map(|group| group.group_key.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        Some(0.92),
        page.groups[0]
            .burst
            .as_ref()
            .and_then(|burst| burst.best_score)
    );
    assert_eq!(
        Some(0.92),
        page.groups[1]
            .burst
            .as_ref()
            .and_then(|burst| burst.best_score)
    );
}

fn save_score_for_display_key(
    store: &SqliteStore,
    project_id: &str,
    display_key: &str,
    overall: f64,
) {
    save_detailed_score_for_display_key(
        store,
        project_id,
        display_key,
        DetailedScore {
            overall,
            sharpness: overall,
            exposure: overall,
            highlight_clipping_penalty: 0.0,
            shadow_clipping_penalty: 0.0,
            composition: overall,
            composition_confidence: 0.5,
        },
    )
}

struct DetailedScore {
    overall: f64,
    sharpness: f64,
    exposure: f64,
    highlight_clipping_penalty: f64,
    shadow_clipping_penalty: f64,
    composition: f64,
    composition_confidence: f64,
}

fn save_detailed_score_for_display_key(
    store: &SqliteStore,
    project_id: &str,
    display_key: &str,
    score: DetailedScore,
) {
    let group = store
        .stored_asset_groups(project_id)
        .expect("groups should query")
        .into_iter()
        .find(|group| group.display_key == display_key)
        .expect("group should exist");
    store
        .save_quality_score(QualityScore {
            asset_group_id: group.group_id,
            preview_source: Some("jpeg".to_string()),
            scorer_version: "local-v1".to_string(),
            analysis_status: QualityAnalysisStatus::Ready,
            exif_status: None,
            capture_time_ms: None,
            sharpness: SignalScore::ready(score.sharpness),
            exposure: SignalScore::ready(score.exposure),
            highlight_clipping_penalty: SignalScore::ready(score.highlight_clipping_penalty),
            shadow_clipping_penalty: SignalScore::ready(score.shadow_clipping_penalty),
            composition: SignalScore::ready(score.composition),
            composition_confidence: score.composition_confidence,
            similarity_cluster_id: None,
            overall: score.overall,
            reasons: vec!["评分完成".to_string()],
            analyzed_at_ms: 10_000,
        })
        .expect("score should save");
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

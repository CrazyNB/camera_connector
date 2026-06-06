use camera_connector_core::{
    AnalysisJobType, AssetGroupQuery, CameraConnectorService, DesktopScanPhase, DesktopScannedFile,
    DesktopSourceStatus, SqliteStore, StoredObjectLocation, TransferStatus,
};

#[test]
fn desktop_scan_indexes_local_file_as_desktop_scan_transfer() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Desktop Scan")
        .expect("project should create");
    let root = temp_dir.path().join("photos");
    std::fs::create_dir_all(&root).expect("root should create");
    let photo = root.join("IMG_1001.JPG");
    std::fs::write(&photo, [1_u8, 2, 3, 4]).expect("sample should write");

    let scan = store
        .create_desktop_scan_run(&project.project_id, &root, 10_000)
        .expect("scan run should create");
    let result = store
        .record_desktop_scan_files(
            &scan.scan_id,
            &[DesktopScannedFile {
                local_path: photo.clone(),
                relative_path: "IMG_1001.JPG".to_string(),
                original_filename: "IMG_1001.JPG".to_string(),
                normalized_stem: "IMG_1001".to_string(),
                size_bytes: 4,
                modified_at_ms: 10_001,
                capture_time_ms: None,
            }],
            10_002,
        )
        .expect("desktop scan file should index");

    assert_eq!(result.assets_indexed, 1);
    assert_eq!(result.group_ids.len(), 1);

    let transfers = store
        .transfer_records(&project.project_id)
        .expect("transfer records should query");
    assert_eq!(transfers.len(), 1);
    assert!(transfers[0].transfer_id.starts_with("desktop-scan-"));
    assert_eq!(transfers[0].protocol, "desktop_scan");
    assert_eq!(transfers[0].status, TransferStatus::Completed);
    assert_eq!(transfers[0].original_path, "IMG_1001.JPG");
    assert_eq!(
        transfers[0].final_location,
        Some(StoredObjectLocation::local_path(photo.clone()))
    );

    let page = store
        .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should query");
    assert_eq!(page.summary.asset_count, 1);
    assert_eq!(page.groups[0].group_key, "IMG_1001");
    assert_eq!(
        page.groups[0].primary.source_status.as_deref(),
        Some(DesktopSourceStatus::Available.as_str())
    );
    assert_eq!(
        page.groups[0].primary.last_seen_scan_id.as_deref(),
        Some(scan.scan_id.as_str())
    );
}

#[test]
fn service_scans_folder_and_groups_raw_jpeg_video_by_stem() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("config.json");
    let root = temp_dir.path().join("photos");
    std::fs::create_dir_all(&root).expect("photo root should create");
    std::fs::write(root.join("IMG_2001.JPG"), [1_u8]).expect("jpeg should write");
    std::fs::write(root.join("IMG_2001.NEF"), [2_u8]).expect("raw should write");
    std::fs::write(root.join("IMG_2001.MOV"), [3_u8]).expect("video should write");

    let service = CameraConnectorService::new(Some(config_path));
    let project = service
        .create_project("Desktop Folder")
        .expect("project should create");
    let scan = service
        .create_desktop_project_scan(&project.project_id, &root)
        .expect("scan should queue");
    let result = service
        .run_desktop_project_scan(&scan.scan_id)
        .expect("scan should complete");

    assert_eq!(result.scan.phase, DesktopScanPhase::Completed);
    assert_eq!(result.scan.files_seen, 3);
    assert_eq!(result.index.assets_indexed, 3);

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("assets should query");
    assert_eq!(page.total_groups, 1);
    assert!(page.groups[0].jpeg.is_some());
    assert!(page.groups[0].raw.is_some());
    assert!(page.groups[0].video.is_some());
}

#[test]
fn rescan_marks_missing_and_changed_without_deleting_group_marks() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("config.json");
    let root = temp_dir.path().join("photos");
    std::fs::create_dir_all(&root).expect("photo root should create");
    let first = root.join("IMG_3001.JPG");
    let second = root.join("IMG_3002.JPG");
    std::fs::write(&first, [1_u8]).expect("first should write");
    std::fs::write(&second, [2_u8]).expect("second should write");

    let service = CameraConnectorService::new(Some(config_path));
    let project = service
        .create_project("Rescan")
        .expect("project should create");
    let scan = service
        .create_desktop_project_scan(&project.project_id, &root)
        .expect("scan should queue");
    service
        .run_desktop_project_scan(&scan.scan_id)
        .expect("scan should run");
    let first_group = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page")
        .groups
        .into_iter()
        .find(|group| group.group_key == "IMG_3001")
        .expect("first group should exist")
        .group_id
        .expect("group id should exist");
    service
        .set_asset_group_user_marks(&project.project_id, &first_group, Some(true), Some(true))
        .expect("marks should save");

    std::fs::remove_file(&first).expect("first should remove");
    std::fs::write(&second, [2_u8, 3_u8]).expect("second should change");
    let rescan = service
        .create_desktop_project_scan(&project.project_id, &root)
        .expect("rescan should queue");
    service
        .run_desktop_project_scan(&rescan.scan_id)
        .expect("rescan should run");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should query");
    assert_eq!(page.total_groups, 2);
    let missing = page
        .groups
        .iter()
        .find(|group| group.group_key == "IMG_3001")
        .expect("missing group");
    assert_eq!(missing.primary.source_status.as_deref(), Some("missing"));
    assert!(missing.user_marks.favorite);
    assert!(missing.user_marks.marked);
    let changed = page
        .groups
        .iter()
        .find(|group| group.group_key == "IMG_3002")
        .expect("changed group");
    assert_eq!(changed.primary.source_status.as_deref(), Some("changed"));
}

#[test]
fn desktop_scan_completion_enqueues_asset_analysis_without_project_recommendation() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("config.json");
    let root = temp_dir.path().join("photos");
    std::fs::create_dir_all(&root).expect("photo root should create");
    std::fs::write(root.join("IMG_4001.JPG"), [1_u8]).expect("jpeg should write");

    let service = CameraConnectorService::new(Some(config_path));
    let project = service
        .create_project("Analysis Jobs")
        .expect("project should create");
    let scan = service
        .create_desktop_project_scan(&project.project_id, &root)
        .expect("scan should queue");
    service
        .run_desktop_project_scan(&scan.scan_id)
        .expect("scan should run");

    let jobs = service
        .storage_store()
        .expect("store should open")
        .claim_analysis_jobs(i64::MAX, 10)
        .expect("jobs should claim");
    assert_eq!(jobs.len(), 1);
    assert_eq!(
        jobs[0].job_type,
        AnalysisJobType::AssessAssetGroupTechnicalQuality
    );
    assert!(!jobs
        .iter()
        .any(|job| job.job_type == AnalysisJobType::GenerateProjectRecommendation));
}

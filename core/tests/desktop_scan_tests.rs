use camera_connector_core::{
    AssetGroupQuery, DesktopScannedFile, DesktopSourceStatus, SqliteStore, StoredObjectLocation,
    TransferStatus,
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

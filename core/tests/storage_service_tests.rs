use camera_connector_core::{
    AssetGroupQuery, CameraConnectorService, StoredObjectLocation, TransferRecord, TransferStatus,
};

#[test]
fn service_creates_and_selects_active_storage_project() {
    let config_path = unique_temp_path("storage-service-config");
    let service = CameraConnectorService::new(Some(config_path.clone()));

    let project = service
        .create_project("Wedding")
        .expect("project should create");
    service
        .set_active_project(&project.project_id)
        .expect("active project should save");

    let active = service
        .active_project()
        .expect("active project should load")
        .expect("active project should exist");

    assert_eq!(active.project_id, project.project_id);
    assert_eq!(active.name, "Wedding");

    let _ = std::fs::remove_file(config_path);
}

#[test]
fn service_queries_storage_asset_groups_inside_project_scope() {
    let config_path = unique_temp_path("storage-service-project-scope");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project_a = service.create_project("A").expect("project should create");
    let project_b = service.create_project("B").expect("project should create");

    service
        .record_project_transfer(
            &project_a.project_id,
            completed_transfer("ftp:a-jpg", "DCIM/100/IMG_1000.JPG", 20),
        )
        .expect("jpg should record");
    service
        .record_project_transfer(
            &project_a.project_id,
            completed_transfer("ftp:a-raw", "DCIM/100/IMG_1000.NEF", 21),
        )
        .expect("raw should record");
    service
        .record_project_transfer(
            &project_b.project_id,
            completed_transfer("ftp:b-jpg", "DCIM/100/IMG_1000.JPG", 22),
        )
        .expect("other project should record");

    let page = service
        .project_asset_group_page_with_query(
            &project_a.project_id,
            AssetGroupQuery::default(),
            0,
            25,
        )
        .expect("page should query");

    assert_eq!(page.total_groups, 1);
    assert_eq!(page.summary.asset_count, 2);
    assert_eq!(page.groups[0].group_key, "IMG_1000");
}

#[test]
fn service_builds_project_dashboard_from_sqlite_assets() {
    let config_path = unique_temp_path("storage-service-dashboard");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Dashboard")
        .expect("project should create");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:dashboard", "DCIM/100/DSC_0001.NEF", 30),
        )
        .expect("transfer should record");

    let dashboard = service
        .project_dashboard(
            &project.project_id,
            AssetGroupQuery::default(),
            0,
            25,
            false,
        )
        .expect("project dashboard should build");

    assert_eq!(dashboard.assets.total_groups, 1);
    assert_eq!(dashboard.assets.summary.asset_count, 1);
    assert_eq!(dashboard.transfers.total_count, 1);
    assert_eq!(dashboard.transfers.completed_count, 1);
    assert_eq!(dashboard.transfers.failed_count, 0);

    let _ = std::fs::remove_file(config_path);
}

fn completed_transfer(
    transfer_id: &str,
    original_path: &str,
    completed_at_ms: i64,
) -> TransferRecord {
    let final_filename = original_path
        .rsplit('/')
        .next()
        .expect("filename should exist")
        .to_string();
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
        source_name: Some("Studio Z5".to_string()),
        started_at_ms: completed_at_ms - 1,
        completed_at_ms: Some(completed_at_ms),
        error: None,
    }
}

fn unique_temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camera-connector-{name}-{}.json", unique_suffix()))
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

use camera_connector_core::{
    AssetGroupQuery, CameraConnectorService, ReceiverSettingsUpdate, StoredObjectLocation,
    TransferRecord, TransferStatus,
};

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

#[test]
fn service_project_dashboard_includes_project_scoped_publish_queue_summary() {
    let config_path = unique_temp_path("storage-service-dashboard-publish-queue");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project_a = service
        .create_project("Publish Queue A")
        .expect("project should create");
    let project_b = service
        .create_project("Publish Queue B")
        .expect("other project should create");
    service
        .set_active_project(&project_a.project_id)
        .expect("project should become active");
    let store = service.storage_store().expect("store should open");

    store
        .enqueue_publish(
            &project_a.project_id,
            "ftp:staged",
            "staging/staged.tmp",
            "IMG_1000.JPG",
            10,
        )
        .expect("staged publish should enqueue");
    let failed = store
        .enqueue_publish(
            &project_a.project_id,
            "ftp:failed",
            "staging/failed.tmp",
            "IMG_1001.JPG",
            20,
        )
        .expect("failed publish should enqueue");
    store
        .mark_publish_failed(&failed.queue_id, "permission revoked")
        .expect("publish should fail");
    let completed = store
        .enqueue_publish(
            &project_a.project_id,
            "ftp:completed",
            "staging/completed.tmp",
            "IMG_1002.JPG",
            30,
        )
        .expect("completed publish should enqueue");
    store
        .mark_publish_completed(&completed.queue_id)
        .expect("publish should complete");
    service
        .set_active_project(&project_b.project_id)
        .expect("other project should become active");
    store
        .enqueue_publish(
            &project_b.project_id,
            "ftp:other",
            "staging/other.tmp",
            "IMG_2000.JPG",
            40,
        )
        .expect("other project publish should enqueue");

    let dashboard = service
        .project_dashboard(
            &project_a.project_id,
            AssetGroupQuery::default(),
            0,
            25,
            false,
        )
        .expect("project dashboard should build");

    assert_eq!(dashboard.publish_queue.total_count, 3);
    assert_eq!(dashboard.publish_queue.pending_count, 2);
    assert_eq!(dashboard.publish_queue.staged_count, 1);
    assert_eq!(dashboard.publish_queue.completed_count, 1);
    assert_eq!(dashboard.publish_queue.failed_count, 1);

    let _ = std::fs::remove_file(config_path);
}

#[test]
fn service_project_dashboard_reports_configured_output_dir_when_receiver_stopped() {
    let config_path = unique_temp_path("storage-service-dashboard-output-dir");
    let output_dir = unique_temp_path("storage-service-dashboard-output-dir-files");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Output Dir")
        .expect("project should create");
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            output_dir: Some(output_dir.clone()),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");

    let dashboard = service
        .project_dashboard(
            &project.project_id,
            AssetGroupQuery::default(),
            0,
            20,
            false,
        )
        .expect("project dashboard should build");

    assert_eq!(
        dashboard.paths.output_dir.as_deref(),
        Some(output_dir.as_path())
    );

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn service_project_dashboard_includes_project_scoped_recent_publish_failures() {
    let config_path = unique_temp_path("storage-service-dashboard-publish-failures");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project_a = service
        .create_project("Publish Failure A")
        .expect("project should create");
    let project_b = service
        .create_project("Publish Failure B")
        .expect("other project should create");
    let store = service.storage_store().expect("store should open");

    let failed_a = store
        .enqueue_publish(
            &project_a.project_id,
            "ftp:publish-a",
            "staging/a.tmp",
            "IMG_1100.JPG",
            10,
        )
        .expect("publish should enqueue");
    store
        .mark_publish_failed(&failed_a.queue_id, "SAF permission revoked")
        .expect("publish failure should save");

    let failed_b = store
        .enqueue_publish(
            &project_b.project_id,
            "ftp:publish-b",
            "staging/b.tmp",
            "IMG_2200.JPG",
            20,
        )
        .expect("other publish should enqueue");
    store
        .mark_publish_failed(&failed_b.queue_id, "other project failed")
        .expect("other publish failure should save");

    let dashboard = service
        .project_dashboard(
            &project_a.project_id,
            AssetGroupQuery::default(),
            0,
            25,
            false,
        )
        .expect("project dashboard should build");

    assert_eq!(dashboard.recent_publish_failures.len(), 1);
    assert_eq!(
        dashboard.recent_publish_failures[0].queue_id,
        failed_a.queue_id
    );
    assert_eq!(
        dashboard.recent_publish_failures[0].last_error.as_deref(),
        Some("SAF permission revoked")
    );
    assert_eq!(
        dashboard.recent_publish_failures[0].final_filename,
        "IMG_1100.JPG"
    );

    let _ = std::fs::remove_file(config_path);
}

#[test]
fn service_project_dashboard_filters_transfer_summary_by_query() {
    let config_path = unique_temp_path("storage-service-dashboard-summary-filter");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Filtered Dashboard")
        .expect("project should create");

    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:z5-complete", "DCIM/100/IMG_3001.JPG", 40),
        )
        .expect("z5 completed transfer should record");

    let mut z5_failed = completed_transfer("ftp:z5-failed", "DCIM/100/IMG_3002.JPG", 50);
    z5_failed.status = TransferStatus::Failed;
    z5_failed.error = Some("simulated failure".to_string());
    service
        .record_project_transfer(&project.project_id, z5_failed)
        .expect("z5 failed transfer should record");

    let mut z6_complete = completed_transfer("ftp:z6-complete", "DCIM/100/IMG_3003.JPG", 60);
    z6_complete.username = Some("z6".to_string());
    z6_complete.source_name = Some("Studio Z6".to_string());
    service
        .record_project_transfer(&project.project_id, z6_complete)
        .expect("z6 completed transfer should record");

    let mut z6_failed = completed_transfer("ftp:z6-failed", "DCIM/100/IMG_3004.JPG", 70);
    z6_failed.status = TransferStatus::Failed;
    z6_failed.username = Some("z6".to_string());
    z6_failed.source_name = Some("Studio Z6".to_string());
    z6_failed.error = Some("other failure".to_string());
    service
        .record_project_transfer(&project.project_id, z6_failed)
        .expect("z6 failed transfer should record");

    let dashboard = service
        .project_dashboard(
            &project.project_id,
            AssetGroupQuery {
                username: Some("z5".to_string()),
                ..AssetGroupQuery::default()
            },
            0,
            25,
            false,
        )
        .expect("project dashboard should build");

    assert_eq!(dashboard.transfers.total_count, 2);
    assert_eq!(dashboard.transfers.completed_count, 1);
    assert_eq!(dashboard.transfers.failed_count, 1);

    let _ = std::fs::remove_file(config_path);
}

#[test]
fn service_project_dashboard_includes_project_scoped_recent_failures() {
    let config_path = unique_temp_path("storage-service-dashboard-failures");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project_a = service.create_project("A").expect("project should create");
    let project_b = service.create_project("B").expect("project should create");

    let mut failed_a = completed_transfer("ftp:a-failed", "DCIM/100/IMG_2001.JPG", 40);
    failed_a.status = TransferStatus::Failed;
    failed_a.error = Some("publish failed".to_string());
    service
        .record_project_transfer(&project_a.project_id, failed_a)
        .expect("project failure should record");

    let mut failed_b = completed_transfer("ftp:b-failed", "DCIM/100/IMG_2002.JPG", 50);
    failed_b.status = TransferStatus::Failed;
    failed_b.error = Some("other project failed".to_string());
    service
        .record_project_transfer(&project_b.project_id, failed_b)
        .expect("other project failure should record");

    let dashboard = service
        .project_dashboard(
            &project_a.project_id,
            AssetGroupQuery::default(),
            0,
            25,
            false,
        )
        .expect("project dashboard should build");

    assert_eq!(dashboard.recent_failures.len(), 1);
    assert_eq!(
        dashboard.recent_failures[0].record.transfer_id,
        "ftp:a-failed"
    );
    assert_eq!(
        dashboard.recent_failures[0].record.error.as_deref(),
        Some("publish failed")
    );

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
    let dir = std::env::temp_dir().join(format!("camera-connector-{name}-{}", unique_suffix()));
    std::fs::create_dir_all(&dir).expect("unique temp dir should create");
    dir.join("config.json")
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

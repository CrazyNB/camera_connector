use camera_connector_core::{
    AssetGroupQuery, CameraConnectorConfig, CameraConnectorService, ProjectStatus,
    ReceiverConfigRequest, ReceiverSettingsUpdate, SqliteStore, StoredObjectLocation,
    TransferQuery, TransferRecord, TransferStatus,
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
fn service_ensures_system_inbox_project_when_none_is_active() {
    let config_path = unique_temp_path("storage-service-ensure-inbox");
    let service = CameraConnectorService::new(Some(config_path.clone()));

    assert!(service
        .active_project()
        .expect("active project should load")
        .is_none());

    let project = service
        .ensure_active_project()
        .expect("active project should be ensured");

    assert_eq!(project.project_id, "project-inbox");
    assert_eq!(project.name, "Inbox");
    assert_eq!(
        service
            .active_project()
            .expect("active project should load")
            .expect("active project should exist")
            .project_id,
        "project-inbox"
    );

    let _ = std::fs::remove_file(config_path);
}

#[test]
fn service_ensure_active_project_keeps_existing_selection() {
    let config_path = unique_temp_path("storage-service-preserve-active");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Selected")
        .expect("project should create");
    service
        .set_active_project(&project.project_id)
        .expect("active project should save");

    let ensured = service
        .ensure_active_project()
        .expect("active project should be ensured");

    assert_eq!(ensured.project_id, project.project_id);

    let _ = std::fs::remove_file(config_path);
}

#[test]
fn service_archives_active_project_and_restores_selection() {
    let config_path = unique_temp_path("storage-service-archive-project");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Archive")
        .expect("project should create");
    service
        .set_active_project(&project.project_id)
        .expect("active project should save");

    let archived = service
        .archive_project(&project.project_id)
        .expect("project should archive");

    assert_eq!(archived.status, ProjectStatus::Archived);
    assert!(service
        .active_project()
        .expect("active project should load")
        .is_none());
    assert!(service.set_active_project(&project.project_id).is_err());

    let restored = service
        .restore_project(&project.project_id)
        .expect("project should restore");
    service
        .set_active_project(&project.project_id)
        .expect("restored project should be selectable");

    assert_eq!(restored.status, ProjectStatus::Active);
    assert_eq!(
        service
            .active_project()
            .expect("active project should load")
            .expect("active project should exist")
            .project_id,
        project.project_id
    );

    let _ = std::fs::remove_file(config_path);
}

#[test]
fn service_uses_configured_state_dir_for_project_storage() {
    let config_path = unique_temp_path("storage-service-configured-state");
    let configured_state_dir = config_path
        .parent()
        .expect("config path should have parent")
        .join("configured-state");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            state_dir: Some(configured_state_dir.clone()),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");

    let project = service
        .ensure_active_project()
        .expect("active project should be ensured");

    let configured_store =
        SqliteStore::open_state_dir(&configured_state_dir).expect("configured store should open");
    let active = configured_store
        .active_project()
        .expect("active project should load")
        .expect("active project should exist");
    assert_eq!(active.project_id, project.project_id);

    let dashboard = service
        .project_dashboard(
            &project.project_id,
            AssetGroupQuery::default(),
            0,
            25,
            false,
        )
        .expect("project dashboard should build");
    assert_eq!(dashboard.paths.state_dir, configured_state_dir);

    let _ = std::fs::remove_file(config_path);
}

#[test]
fn service_persists_receiver_accounts_in_sqlite_store() {
    let config_path = unique_temp_path("storage-service-account-model");
    let state_dir = config_path
        .parent()
        .expect("config path should have parent")
        .join("account-state");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            state_dir: Some(state_dir.clone()),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");

    let (account, saved_path) = service
        .set_account(" z5 ", Some("secret"), " Z5 II ")
        .expect("account should save");

    assert_eq!(saved_path, config_path);
    assert_eq!(account.username, "z5");
    assert_eq!(account.device_name, "Z5 II");
    assert!(account.password_configured());
    assert!(CameraConnectorConfig::load(Some(&config_path))
        .expect("json config should load")
        .accounts
        .is_empty());

    let stored_accounts = SqliteStore::open_state_dir(&state_dir)
        .expect("store should open")
        .receiver_accounts()
        .expect("accounts should load from sqlite");
    let account_views = service.accounts().expect("account views should load");
    let receiver_config = service
        .receiver_config(ReceiverConfigRequest {
            protocol: None,
            bind_host: None,
            port: None,
            output_dir: None,
            state_dir: None,
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
            defer_publish: None,
        })
        .expect("receiver config should build");

    assert_eq!(stored_accounts.len(), 1);
    assert_eq!(stored_accounts[0].username, "z5");
    assert_eq!(stored_accounts[0].device_name, "Z5 II");
    assert!(stored_accounts[0].enabled);
    assert_eq!(account_views.len(), 1);
    assert_eq!(account_views[0].username, "z5");
    assert_eq!(account_views[0].device_name, "Z5 II");
    assert_eq!(receiver_config.accounts.len(), 1);
    assert_eq!(receiver_config.accounts[0].username, "z5");
    assert_eq!(receiver_config.accounts[0].device_name, "Z5 II");

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn receiver_config_keeps_accounts_in_configured_state_when_runtime_state_is_overridden() {
    let config_path = unique_temp_path("storage-service-account-runtime-state");
    let account_state_dir = config_path
        .parent()
        .expect("config path should have parent")
        .join("account-state");
    let runtime_state_dir = config_path
        .parent()
        .expect("config path should have parent")
        .join("runtime-state");
    let output_dir = config_path
        .parent()
        .expect("config path should have parent")
        .join("runtime-output");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            state_dir: Some(account_state_dir.clone()),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");
    service
        .set_account("z5", Some("secret"), "Configured Account")
        .expect("account should save");

    let receiver_config = service
        .receiver_config(ReceiverConfigRequest {
            protocol: None,
            bind_host: None,
            port: None,
            output_dir: Some(output_dir.clone()),
            state_dir: Some(runtime_state_dir.clone()),
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
            defer_publish: None,
        })
        .expect("receiver config should build");

    assert_eq!(receiver_config.state_dir, runtime_state_dir);
    assert_eq!(receiver_config.accounts.len(), 1);
    assert_eq!(receiver_config.accounts[0].username, "z5");
    assert_eq!(
        receiver_config.accounts[0].device_name,
        "Configured Account"
    );

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(account_state_dir);
    let _ = std::fs::remove_dir_all(runtime_state_dir);
    let _ = std::fs::remove_dir_all(output_dir);
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
fn service_queries_project_group_members_by_stable_group_id() {
    let config_path = unique_temp_path("storage-service-project-group-members");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Group Members")
        .expect("project should create");

    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:z5", "DCIM/100/IMG_4001.JPG", 20),
        )
        .expect("first transfer should record");
    let mut second = completed_transfer("ftp:z6", "DCIM/200/IMG_4001.JPG", 21);
    second.username = Some("z6".to_string());
    second.source_name = Some("Studio Z6".to_string());
    service
        .record_project_transfer(&project.project_id, second)
        .expect("second transfer should record");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("project groups should query");
    let first_group_id = page.groups[0]
        .group_id
        .as_deref()
        .expect("first group should expose stable id");
    let second_group_id = page.groups[1]
        .group_id
        .as_deref()
        .expect("second group should expose stable id");

    let first_assets = service
        .project_group_assets(&project.project_id, first_group_id)
        .expect("first group members should query");
    let second_assets = service
        .project_group_assets(&project.project_id, second_group_id)
        .expect("second group members should query");
    let ambiguous_assets = service
        .project_group_assets(&project.project_id, "IMG_4001")
        .expect("display key should not query members");

    assert_eq!(first_assets.len(), 1);
    assert_eq!(second_assets.len(), 1);
    assert_ne!(first_assets[0].group_id, second_assets[0].group_id);
    assert!(ambiguous_assets.is_empty());

    let _ = std::fs::remove_file(config_path);
}

#[test]
fn service_filters_project_transfers_from_sqlite() {
    let config_path = unique_temp_path("storage-service-project-transfers");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service
        .create_project("Transfers")
        .expect("project should create");

    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:complete", "DCIM/100/IMG_1000.JPG", 20),
        )
        .expect("completed transfer should record");
    let mut failed = completed_transfer("ftp:failed", "DCIM/100/IMG_1001.JPG", 30);
    failed.status = TransferStatus::Failed;
    failed.error = Some("simulated failure".to_string());
    service
        .record_project_transfer(&project.project_id, failed)
        .expect("failed transfer should record");

    let transfers = service
        .project_transfers(
            &project.project_id,
            TransferQuery {
                status: Some(TransferStatus::Failed),
                source_name: Some("Studio Z5".to_string()),
                ..TransferQuery::default()
            },
        )
        .expect("project transfers should query");

    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].record.transfer_id, "ftp:failed");
    assert_eq!(transfers[0].display_source.as_deref(), Some("Studio Z5"));
    assert_eq!(
        transfers[0].record.error.as_deref(),
        Some("simulated failure")
    );
    assert!(transfers[0].virtual_display_path.contains("IMG_1001.JPG"));

    let _ = std::fs::remove_file(config_path);
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

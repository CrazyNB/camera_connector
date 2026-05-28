use camera_connector_core::{
    AssetGroupQuery, ObjectFormat, ProjectStatus, PushProtocol, ReceiverAccountConfig,
    ReceiverAuthMode, ReceiverRuntimePhase, ReceiverRuntimeStatus, SqliteStore,
    StoredObjectLocation, TransferRecord, TransferStatus,
};
use rusqlite::Connection;

#[test]
fn sqlite_store_creates_projects_and_tracks_active_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");

    let project = store
        .create_project("Studio Product Shoot")
        .expect("project should create");
    store
        .set_active_project(&project.project_id)
        .expect("active project should save");

    let active = store
        .active_project()
        .expect("active project should load")
        .expect("active project should exist");
    let projects = store.list_projects().expect("projects should list");

    assert_eq!(active.project_id, project.project_id);
    assert_eq!(active.name, "Studio Product Shoot");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].status.as_str(), "active");
}

#[test]
fn sqlite_store_archives_projects_and_clears_active_selection() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Archive Me")
        .expect("project should create");
    store
        .set_active_project(&project.project_id)
        .expect("active project should save");

    let archived = store
        .archive_project(&project.project_id)
        .expect("project should archive");

    assert_eq!(archived.status, ProjectStatus::Archived);
    assert!(archived.archived_at_ms.is_some());
    assert!(store
        .active_project()
        .expect("active project should load")
        .is_none());
    assert!(store.set_active_project(&project.project_id).is_err());

    let restored = store
        .restore_project(&project.project_id)
        .expect("project should restore");
    store
        .set_active_project(&project.project_id)
        .expect("restored project should become active");

    assert_eq!(restored.status, ProjectStatus::Active);
    assert!(restored.archived_at_ms.is_none());
    assert_eq!(
        store
            .active_project()
            .expect("active project should load")
            .expect("active project should exist")
            .project_id,
        project.project_id
    );
}

#[test]
fn sqlite_store_rejects_archiving_system_inbox_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let inbox = store
        .ensure_inbox_project()
        .expect("inbox project should exist");
    store
        .set_active_project(&inbox.project_id)
        .expect("inbox should become active");

    let result = store.archive_project(&inbox.project_id);

    assert!(result.is_err());
    assert!(result
        .err()
        .expect("error should exist")
        .to_string()
        .contains("system inbox project cannot be archived"));
    assert_eq!(
        store
            .active_project()
            .expect("active project should load")
            .expect("active project should remain")
            .project_id,
        "project-inbox"
    );
    assert_eq!(
        store
            .ensure_inbox_project()
            .expect("inbox project should load")
            .status,
        ProjectStatus::Active
    );
}

#[test]
fn sqlite_store_upserts_receiver_accounts() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");

    let created = store
        .upsert_receiver_account(
            ReceiverAccountConfig::new(" z5 ", Some("secret"), " Z5 II ")
                .expect("account should build"),
        )
        .expect("account should upsert");

    assert_eq!(created.username, "z5");
    assert_eq!(created.device_name, "Z5 II");
    assert!(created.enabled);
    assert!(created
        .password_hash
        .as_deref()
        .expect("password hash should exist")
        .starts_with("$argon2id$"));
    assert!(created.created_at_ms > 0);
    assert!(created.updated_at_ms >= created.created_at_ms);

    let updated = store
        .upsert_receiver_account(
            ReceiverAccountConfig::new("z5", Some("new-secret"), "Studio Z5")
                .expect("account should build"),
        )
        .expect("account should update");
    let accounts = store
        .receiver_accounts()
        .expect("accounts should list from sqlite");

    assert_eq!(updated.username, "z5");
    assert_eq!(updated.device_name, "Studio Z5");
    assert_eq!(updated.created_at_ms, created.created_at_ms);
    assert!(updated.updated_at_ms >= created.updated_at_ms);
    assert_ne!(updated.password_hash, created.password_hash);
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0], updated);
}

#[test]
fn sqlite_store_removes_receiver_accounts() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    store
        .upsert_receiver_account(
            ReceiverAccountConfig::new("z5", Some("secret"), "Z5 II")
                .expect("account should build"),
        )
        .expect("account should upsert");

    assert!(store
        .remove_receiver_account("z5")
        .expect("account should remove"));
    assert!(!store
        .remove_receiver_account("z5")
        .expect("missing account should be reported"));
    assert!(store
        .receiver_accounts()
        .expect("accounts should list")
        .is_empty());
}

#[test]
fn sqlite_store_records_connected_device_state() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open_state_dir(temp_dir.path()).expect("store should open");

    store
        .record_connected_device("192.168.137.56", Some(51120), None, None)
        .expect("device should connect");
    store
        .record_authenticated_device("192.168.137.56", Some("Studio Z5"), Some("z5"))
        .expect("device should authenticate");
    store
        .record_connected_device("192.168.137.44", Some(51121), None, None)
        .expect("new IP should connect");
    store
        .record_authenticated_device("192.168.137.44", Some("Studio Z5"), Some("z5"))
        .expect("new IP should authenticate");

    let devices = store
        .connected_devices()
        .expect("connected devices should load");

    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].remote_addr, "192.168.137.44");
    assert_eq!(devices[0].last_remote_port, Some(51121));
    assert_eq!(devices[0].username.as_deref(), Some("z5"));
    assert_eq!(devices[0].source_name.as_deref(), Some("Studio Z5"));
    assert_eq!(devices[0].active_connections, 1);
    assert!(devices[0].online);

    store
        .record_disconnected_device("192.168.137.44")
        .expect("device should disconnect");
    let disconnected = store
        .connected_devices()
        .expect("connected devices should load");
    assert_eq!(disconnected[0].active_connections, 0);
    assert!(!disconnected[0].online);
    assert!(disconnected[0].last_disconnected_at_ms.is_some());
}

#[test]
fn sqlite_store_persists_receiver_runtime_status() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open_state_dir(temp_dir.path()).expect("store should open");
    let status = ReceiverRuntimeStatus {
        phase: ReceiverRuntimePhase::Running,
        protocol: Some(PushProtocol::Ftp),
        auth_mode: ReceiverAuthMode::Accounts,
        local_addr: Some("127.0.0.1:2121".parse().expect("addr should parse")),
        output_dir: Some(temp_dir.path().join("output")),
        state_dir: Some(temp_dir.path().to_path_buf()),
        account_count: 1,
        message: None,
    };

    store
        .write_receiver_runtime_status(&status)
        .expect("runtime status should write");
    let loaded = store
        .read_receiver_runtime_status()
        .expect("runtime status should read")
        .expect("runtime status should exist");

    assert_eq!(loaded.phase, ReceiverRuntimePhase::Running);
    assert_eq!(loaded.protocol, Some(PushProtocol::Ftp));
    assert_eq!(loaded.auth_mode, ReceiverAuthMode::Accounts);
    assert_eq!(loaded.local_addr, status.local_addr);
    assert_eq!(loaded.output_dir, status.output_dir);
    assert_eq!(loaded.state_dir, status.state_dir);
    assert_eq!(loaded.account_count, 1);
}

#[test]
fn sqlite_store_persists_receiver_runtime_status_as_columns() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let db_path = temp_dir.path().join("state.sqlite");
    let store = SqliteStore::open(&db_path).expect("store should open");
    let status = ReceiverRuntimeStatus {
        phase: ReceiverRuntimePhase::Running,
        protocol: Some(PushProtocol::Sftp),
        auth_mode: ReceiverAuthMode::Accounts,
        local_addr: Some("127.0.0.1:2222".parse().expect("addr should parse")),
        output_dir: Some(temp_dir.path().join("output")),
        state_dir: Some(temp_dir.path().join("state")),
        account_count: 2,
        message: Some("ready".to_string()),
    };

    store
        .write_receiver_runtime_status(&status)
        .expect("runtime status should write");

    let connection = Connection::open(db_path).expect("sqlite should open");
    let columns = connection
        .prepare("PRAGMA table_info(receiver_status)")
        .expect("pragma should prepare")
        .query_map([], |row| row.get::<_, String>(1))
        .expect("columns should query")
        .collect::<Result<Vec<_>, _>>()
        .expect("columns should collect");
    let row = connection
        .query_row(
            "SELECT phase, protocol, auth_mode, local_addr, output_dir, state_dir,
                    account_count, message
             FROM receiver_status
             WHERE key = 'current'",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            },
        )
        .expect("receiver status row should query");

    assert!(!columns.iter().any(|column| column == "payload"));
    assert_eq!(row.0, "running");
    assert_eq!(row.1.as_deref(), Some("sftp"));
    assert_eq!(row.2, "accounts");
    assert_eq!(row.3.as_deref(), Some("127.0.0.1:2222"));
    assert_eq!(row.6, 2);
    assert_eq!(row.7.as_deref(), Some("ready"));
}

#[test]
fn sqlite_store_rejects_transfer_without_existing_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");

    let result = store.record_transfer(
        "missing-project",
        completed_transfer("ftp:1", "IMG_0001.JPG", 10),
    );

    assert!(result.is_err());
    assert!(result
        .err()
        .expect("error should exist")
        .to_string()
        .contains("project not found"));
}

#[test]
fn sqlite_store_indexes_assets_and_groups_by_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project_a = store
        .create_project("Wedding")
        .expect("project should create");
    let project_b = store
        .create_project("Street")
        .expect("project should create");

    store
        .record_transfer(
            &project_a.project_id,
            completed_transfer("ftp:jpg", "DCIM/100/IMG_2222.JPG", 20),
        )
        .expect("jpg transfer should record");
    store
        .record_transfer(
            &project_a.project_id,
            completed_transfer("ftp:raw", "DCIM/100/IMG_2222.NEF", 21),
        )
        .expect("raw transfer should record");
    store
        .record_transfer(
            &project_b.project_id,
            completed_transfer("ftp:other", "DCIM/100/IMG_2222.JPG", 22),
        )
        .expect("other project transfer should record");

    let page = store
        .asset_group_page(&project_a.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("groups should query");
    let group_id = page.groups[0]
        .group_id
        .as_deref()
        .expect("project group should expose stable id");
    let assets = store
        .assets_for_group(&project_a.project_id, group_id)
        .expect("group assets should query");

    assert_eq!(page.total_groups, 1);
    assert_eq!(page.groups[0].group_key, "IMG_2222");
    assert!(page.groups[0].jpeg.is_some());
    assert!(page.groups[0].raw.is_some());
    assert_eq!(page.summary.asset_count, 2);
    assert_eq!(assets.len(), 2);
    assert!(assets.iter().all(|asset| asset.group_id.is_some()));
    assert!(assets.iter().all(|asset| asset.media_kind == "photo"));

    let other_page = store
        .asset_group_page(&project_b.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("other project groups should query");
    assert_eq!(other_page.total_groups, 1);
    assert_eq!(other_page.summary.asset_count, 1);
}

#[test]
fn sqlite_store_exposes_asset_group_rollup_model() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Rollups")
        .expect("project should create");

    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:jpg", "DCIM/100/IMG_2222.JPG", 20),
        )
        .expect("jpg transfer should record");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:raw", "DCIM/100/IMG_2222.NEF", 21),
        )
        .expect("raw transfer should record");

    let groups = store
        .stored_asset_groups(&project.project_id)
        .expect("stored groups should query");

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].project_id, project.project_id);
    assert_eq!(groups[0].display_key, "IMG_2222");
    assert_eq!(groups[0].source_identity.as_deref(), Some("Studio Z5"));
    assert_eq!(groups[0].original_parent_path.as_deref(), Some("DCIM/100"));
    assert_eq!(groups[0].member_count, 2);
    assert!(groups[0].has_jpeg);
    assert!(groups[0].has_raw);
    assert!(!groups[0].has_video);
    assert_eq!(groups[0].primary_asset_id.as_deref(), Some("ftp:jpg"));
    assert_eq!(groups[0].preview_asset_id.as_deref(), Some("ftp:jpg"));
    assert_eq!(groups[0].first_received_at_ms, Some(20));
    assert_eq!(groups[0].last_received_at_ms, Some(21));
}

#[test]
fn sqlite_store_keeps_same_stem_groups_separate_by_source_and_parent() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Same Stem")
        .expect("project should create");

    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:z5", "DCIM/100/IMG_4001.JPG", 20),
        )
        .expect("first transfer should record");
    let mut second = completed_transfer("ftp:z6", "DCIM/200/IMG_4001.JPG", 21);
    second.username = Some("z6".to_string());
    second.source_name = Some("Studio Z6".to_string());
    store
        .record_transfer(&project.project_id, second)
        .expect("second transfer should record");

    let page = store
        .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("groups should query");

    assert_eq!(page.total_groups, 2);
    assert_eq!(page.groups.len(), 2);
    assert!(page.groups.iter().all(|group| group.group_id.is_some()));
    assert_ne!(page.groups[0].group_id, page.groups[1].group_id);
    assert_eq!(page.summary.asset_count, 2);

    let first_group_id = page.groups[0]
        .group_id
        .as_deref()
        .expect("first group should expose id");
    let second_group_id = page.groups[1]
        .group_id
        .as_deref()
        .expect("second group should expose id");
    let first_assets = store
        .assets_for_group(&project.project_id, first_group_id)
        .expect("first group members should query");
    let second_assets = store
        .assets_for_group(&project.project_id, second_group_id)
        .expect("second group members should query");
    let ambiguous_assets = store
        .assets_for_group(&project.project_id, "IMG_4001")
        .expect("display key should not be treated as a group identity");

    assert_eq!(first_assets.len(), 1);
    assert_eq!(second_assets.len(), 1);
    assert_ne!(first_assets[0].group_id, second_assets[0].group_id);
    assert!(ambiguous_assets.is_empty());

    assert!(page
        .summary
        .source_counts
        .iter()
        .any(|count| count.value == "Studio Z5" && count.group_count == 1));
    assert!(page
        .summary
        .source_counts
        .iter()
        .any(|count| count.value == "Studio Z6" && count.group_count == 1));
}

#[test]
fn sqlite_store_lists_project_transfers_by_latest_time() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project_a = store
        .create_project("Wedding")
        .expect("project should create");
    let project_b = store
        .create_project("Street")
        .expect("project should create");

    store
        .record_transfer(
            &project_a.project_id,
            completed_transfer("ftp:old", "DCIM/100/IMG_1111.JPG", 20),
        )
        .expect("old transfer should record");
    let mut failed = completed_transfer("ftp:failed", "DCIM/100/IMG_2222.JPG", 30);
    failed.status = TransferStatus::Failed;
    failed.error = Some("simulated failure".to_string());
    store
        .record_transfer(&project_a.project_id, failed)
        .expect("failed transfer should record");
    store
        .record_transfer(
            &project_b.project_id,
            completed_transfer("ftp:other", "DCIM/100/IMG_3333.JPG", 40),
        )
        .expect("other project transfer should record");

    let transfers = store
        .transfer_records(&project_a.project_id)
        .expect("project transfers should list");

    assert_eq!(transfers.len(), 2);
    assert_eq!(transfers[0].transfer_id, "ftp:failed");
    assert_eq!(transfers[0].status, TransferStatus::Failed);
    assert_eq!(transfers[0].error.as_deref(), Some("simulated failure"));
    assert_eq!(transfers[1].transfer_id, "ftp:old");
    assert!(transfers
        .iter()
        .all(|record| record.transfer_id != "ftp:other"));
}

#[test]
fn sqlite_store_preserves_duplicate_uploads_as_separate_assets() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Duplicates")
        .expect("project should create");

    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:first", "DCIM/100/IMG_7777.CR3", 10),
        )
        .expect("first transfer should record");
    let mut duplicate = completed_transfer("ftp:second", "DCIM/100/IMG_7777.CR3", 20);
    duplicate.final_filename = "IMG_7777 (1).CR3".to_string();
    duplicate.final_location = Some(StoredObjectLocation::local_path(
        temp_dir.path().join("IMG_7777 (1).CR3"),
    ));
    store
        .record_transfer(&project.project_id, duplicate)
        .expect("second transfer should record");

    let page = store
        .asset_group_page(
            &project.project_id,
            AssetGroupQuery {
                format: Some(ObjectFormat::Cr3),
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("groups should query");

    assert_eq!(page.total_groups, 2);
    assert_eq!(page.summary.asset_count, 2);
    assert_eq!(page.groups[0].primary.duplicate_count, Some(2));
    assert_eq!(page.groups[1].primary.duplicate_count, Some(2));
}

#[test]
fn sqlite_store_tracks_publish_queue_retry_state() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Queue")
        .expect("project should create");

    let item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:queued",
            "staged/ftp-queued.tmp",
            "IMG_9000.NEF",
            123,
        )
        .expect("publish item should enqueue");
    store
        .mark_publish_failed(&item.queue_id, "permission revoked")
        .expect("publish failure should save");

    let failed = store
        .pending_publish_items()
        .expect("pending publish items should load");

    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].queue_id, item.queue_id);
    assert_eq!(failed[0].attempt_count, 1);
    assert_eq!(failed[0].last_error.as_deref(), Some("permission revoked"));
    assert!(failed[0].next_attempt_at_ms.is_some());
    assert_eq!(failed[0].state.as_str(), "failed");
}

#[test]
fn sqlite_store_claims_pending_publish_items_for_exclusive_work() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Queue Claim")
        .expect("project should create");

    let staged = store
        .enqueue_publish(
            &project.project_id,
            "ftp:staged",
            "staged/ftp-staged.tmp",
            "IMG_9001.NEF",
            123,
        )
        .expect("staged publish item should enqueue");
    let failed = store
        .enqueue_publish(
            &project.project_id,
            "ftp:failed",
            "staged/ftp-failed.tmp",
            "IMG_9002.NEF",
            456,
        )
        .expect("failed publish item should enqueue");
    store
        .mark_publish_failed(&failed.queue_id, "permission revoked")
        .expect("publish failure should save");

    let claimed = store
        .claim_next_publish_item()
        .expect("claim should run")
        .expect("first item should be claimed");
    let pending = store
        .pending_publish_items()
        .expect("pending publish items should load");
    let deferred = store
        .claim_next_publish_item()
        .expect("deferred retry claim should run");
    let empty = store
        .claim_next_publish_item()
        .expect("empty claim should run");

    assert_eq!(claimed.queue_id, staged.queue_id);
    assert_eq!(claimed.state.as_str(), "publishing");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].queue_id, failed.queue_id);
    assert_eq!(pending[0].state.as_str(), "failed");
    assert!(pending[0].next_attempt_at_ms.is_some());
    assert!(deferred.is_none());
    assert!(empty.is_none());
}

#[test]
fn sqlite_store_releases_failed_publish_retry_delay_for_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Retry Project")
        .expect("project should create");
    let other_project = store
        .create_project("Other Retry Project")
        .expect("other project should create");

    let item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:retry",
            "staged/ftp-retry.tmp",
            "IMG_9100.NEF",
            123,
        )
        .expect("publish item should enqueue");
    let other_item = store
        .enqueue_publish(
            &other_project.project_id,
            "ftp:other-retry",
            "staged/ftp-other-retry.tmp",
            "IMG_9101.NEF",
            456,
        )
        .expect("other publish item should enqueue");
    store
        .mark_publish_failed(&item.queue_id, "permission revoked")
        .expect("publish failure should save");
    store
        .mark_publish_failed(&other_item.queue_id, "other permission revoked")
        .expect("other publish failure should save");

    assert!(store
        .claim_next_publish_item()
        .expect("deferred claim should run")
        .is_none());

    let released = store
        .release_failed_publish_retries(&project.project_id)
        .expect("failed retries should release");
    let claimed = store
        .claim_next_publish_item()
        .expect("retry claim should run")
        .expect("released item should be claimable");
    let empty = store
        .claim_next_publish_item()
        .expect("other project should remain deferred");

    assert_eq!(released, 1);
    assert_eq!(claimed.queue_id, item.queue_id);
    assert_eq!(claimed.state.as_str(), "publishing");
    assert_eq!(claimed.attempt_count, 1);
    assert_eq!(claimed.last_error, None);
    assert_eq!(claimed.next_attempt_at_ms, None);
    assert!(empty.is_none());
}

#[test]
fn sqlite_store_completes_publish_item_with_platform_location_into_asset_index() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Platform Publish")
        .expect("project should create");
    let item = store
        .enqueue_publish_with_metadata(
            &project.project_id,
            "ftp:platform",
            "staged/platform.tmp",
            "IMG_9100.JPG",
            321,
            camera_connector_core::PublishTransferMetadata {
                protocol: "ftp".to_string(),
                original_path: "DCIM/100/IMG_9100.JPG".to_string(),
                username: Some("z5".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Studio Z5".to_string()),
                started_at_ms: 42,
            },
        )
        .expect("publish item should enqueue");

    let record = store
        .complete_publish(
            &item.queue_id,
            "IMG_9100.JPG",
            StoredObjectLocation::document_uri("content://camera-connector/IMG_9100.JPG"),
        )
        .expect("publish should complete and index");

    assert_eq!(record.transfer_id, "ftp:platform");
    assert_eq!(record.original_path, "DCIM/100/IMG_9100.JPG");
    assert_eq!(record.username.as_deref(), Some("z5"));
    assert_eq!(record.source_name.as_deref(), Some("Studio Z5"));
    assert_eq!(record.final_location_kind(), Some("document_uri"));
    let page = store
        .asset_group_page(
            &project.project_id,
            camera_connector_core::AssetGroupQuery::default(),
            0,
            25,
        )
        .expect("groups should query");
    assert_eq!(page.total_groups, 1);
    assert_eq!(
        page.groups[0]
            .primary
            .storage_location
            .as_ref()
            .map(|value| value.kind()),
        Some("document_uri")
    );
    assert_eq!(
        store
            .publish_queue_summary(&project.project_id)
            .expect("summary should load")
            .completed_count,
        1
    );
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

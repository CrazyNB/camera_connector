use camera_connector_core::{
    AssetGroupQuery, ObjectFormat, ProjectStatus, SqliteStore, StoredObjectLocation,
    TransferRecord, TransferStatus,
};

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
    let assets = store
        .assets_for_group(&project_a.project_id, &page.groups[0].group_key)
        .expect("group assets should query");

    assert_eq!(page.total_groups, 1);
    assert_eq!(page.groups[0].group_key, "IMG_2222");
    assert!(page.groups[0].jpeg.is_some());
    assert!(page.groups[0].raw.is_some());
    assert_eq!(page.summary.asset_count, 2);
    assert_eq!(assets.len(), 2);
    assert!(assets.iter().all(|asset| asset.group_id.is_some()));

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
    assert_eq!(page.summary.asset_count, 2);
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
    assert_eq!(failed[0].state.as_str(), "failed");
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

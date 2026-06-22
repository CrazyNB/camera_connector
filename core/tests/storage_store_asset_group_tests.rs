use camera_connector_core::{
    AssetGroupQuery, ObjectFormat, SqliteStore, StoredObjectLocation, TransferRecord, TransferStatus,
};

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

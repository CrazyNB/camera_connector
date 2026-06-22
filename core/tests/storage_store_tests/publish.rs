use super::*;

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
        .failed_publish_items(&project.project_id, 10)
        .expect("failed publish items should load");

    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].queue_id, item.queue_id);
    assert_eq!(failed[0].attempt_count, 1);
    assert_eq!(failed[0].last_error.as_deref(), Some("permission revoked"));
    assert!(failed[0].next_attempt_at_ms.is_some());
    assert_eq!(failed[0].state.as_str(), "failed");
}

#[test]
fn sqlite_store_claims_publish_items_for_exclusive_work() {
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
        .failed_publish_items(&project.project_id, 10)
        .expect("failed publish items should load");
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

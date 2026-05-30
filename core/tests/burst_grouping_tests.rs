use std::collections::BTreeSet;

use camera_connector_core::{
    PublishTransferMetadata, SqliteStore, StoredObjectLocation, StrategyProfile,
};

#[test]
fn burst_grouping_falls_back_to_received_time_and_filename_sequence() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Burst Project")
        .expect("project should create");
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:1",
        "DCIM/100/IMG_1001.JPG",
        1000,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:2",
        "DCIM/100/IMG_1002.JPG",
        1100,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:3",
        "DCIM/100/IMG_1003.JPG",
        1200,
    );
    let first_group = store
        .stored_asset_groups(&project.project_id)
        .expect("groups should query")
        .first()
        .expect("group should exist")
        .group_id
        .clone();

    let bursts = store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group,
            &StrategyProfile::general(),
        )
        .expect("bursts should detect");

    assert_eq!(bursts.len(), 1);
    assert_eq!(bursts[0].member_count, 3);
    assert_eq!(bursts[0].recommendation_status, "pending");
}

#[test]
fn burst_grouping_does_not_cross_source_identity() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Source Project")
        .expect("project should create");
    publish_jpeg_with_user(
        &store,
        &project.project_id,
        "ftp:1",
        "DCIM/100/IMG_1001.JPG",
        "z5a",
        1000,
    );
    publish_jpeg_with_user(
        &store,
        &project.project_id,
        "ftp:2",
        "DCIM/100/IMG_1002.JPG",
        "z5b",
        1050,
    );
    let first_group = store
        .stored_asset_groups(&project.project_id)
        .expect("groups should query")
        .first()
        .expect("group should exist")
        .group_id
        .clone();

    let bursts = store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group,
            &StrategyProfile::general(),
        )
        .expect("bursts should detect");

    assert!(bursts.is_empty());
}

#[test]
fn out_of_order_upload_merges_existing_burst_groups() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Late Project")
        .expect("project should create");
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:1",
        "DCIM/100/IMG_1001.JPG",
        1000,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:3",
        "DCIM/100/IMG_1003.JPG",
        1300,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:2",
        "DCIM/100/IMG_1002.JPG",
        1150,
    );
    let late_group = store
        .stored_asset_groups(&project.project_id)
        .expect("groups should query")
        .into_iter()
        .find(|group| group.display_key == "IMG_1002")
        .expect("late group should exist")
        .group_id;

    let bursts = store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &late_group,
            &StrategyProfile::general(),
        )
        .expect("bursts should detect");

    assert_eq!(bursts.len(), 1);
    assert_eq!(bursts[0].member_count, 3);
}

#[test]
fn manual_split_removes_member_from_existing_burst_group() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Manual Split Project")
        .expect("project should create");
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:1",
        "DCIM/100/IMG_1001.JPG",
        1000,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:2",
        "DCIM/100/IMG_1002.JPG",
        1100,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:3",
        "DCIM/100/IMG_1003.JPG",
        1200,
    );
    let groups = store
        .stored_asset_groups(&project.project_id)
        .expect("groups should query");
    let split_group_id = groups
        .iter()
        .find(|group| group.display_key == "IMG_1002")
        .expect("split group should exist")
        .group_id
        .clone();
    let first_group_id = groups
        .first()
        .expect("first group should exist")
        .group_id
        .clone();
    let bursts = store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group_id,
            &StrategyProfile::general(),
        )
        .expect("bursts should detect");
    assert_eq!(bursts.len(), 1);
    assert_eq!(bursts[0].member_count, 3);

    let updated = store
        .split_burst_member(&bursts[0].burst_group_id, &split_group_id)
        .expect("member should split from burst")
        .expect("remaining burst should still exist");
    let page = store
        .asset_group_page(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let split_group = page
        .groups
        .iter()
        .find(|group| group.group_id.as_deref() == Some(split_group_id.as_str()))
        .expect("split group should still be visible");

    assert_eq!(updated.member_count, 2);
    assert_eq!(updated.grouping_version, 2);
    assert_eq!(updated.recommendation_status, "pending");
    assert_eq!(updated.user_override_state.as_deref(), Some("split"));
    assert!(!updated.member_group_ids.contains(&split_group_id));
    assert!(split_group.burst.is_none());
}

#[test]
fn manual_split_survives_later_burst_detection() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Manual Split Persistence Project")
        .expect("project should create");
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:1",
        "DCIM/100/IMG_1001.JPG",
        1000,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:2",
        "DCIM/100/IMG_1002.JPG",
        1100,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:3",
        "DCIM/100/IMG_1003.JPG",
        1200,
    );
    let groups = store
        .stored_asset_groups(&project.project_id)
        .expect("groups should query");
    let split_group_id = groups
        .iter()
        .find(|group| group.display_key == "IMG_1002")
        .expect("split group should exist")
        .group_id
        .clone();
    let first_group_id = groups
        .first()
        .expect("first group should exist")
        .group_id
        .clone();
    let bursts = store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group_id,
            &StrategyProfile::general(),
        )
        .expect("bursts should detect");
    store
        .split_burst_member(&bursts[0].burst_group_id, &split_group_id)
        .expect("member should split");

    store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group_id,
            &StrategyProfile::general(),
        )
        .expect("later detection should run");
    let page = store
        .asset_group_page(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let split_group = page
        .groups
        .iter()
        .find(|group| group.group_id.as_deref() == Some(split_group_id.as_str()))
        .expect("split group should remain visible");
    let burst_member_counts = page
        .groups
        .iter()
        .filter_map(|group| group.burst.as_ref().map(|burst| burst.member_count))
        .collect::<Vec<_>>();

    assert!(split_group.burst.is_none());
    assert_eq!(burst_member_counts, vec![2, 2]);
}

#[test]
fn manual_merge_member_group_merges_source_burst_and_survives_later_detection() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Manual Merge Persistence Project")
        .expect("project should create");
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:1",
        "DCIM/100/IMG_1001.JPG",
        1000,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:2",
        "DCIM/100/IMG_1002.JPG",
        1100,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:3",
        "DCIM/101/IMG_2001.JPG",
        5000,
    );
    publish_jpeg(
        &store,
        &project.project_id,
        "ftp:4",
        "DCIM/101/IMG_2002.JPG",
        5100,
    );
    let groups = store
        .stored_asset_groups(&project.project_id)
        .expect("groups should query");
    let first_group_id = groups
        .iter()
        .find(|group| group.display_key == "IMG_1001")
        .expect("first burst member should exist")
        .group_id
        .clone();
    let source_member_group_id = groups
        .iter()
        .find(|group| group.display_key == "IMG_2001")
        .expect("source burst member should exist")
        .group_id
        .clone();
    let bursts = store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group_id,
            &StrategyProfile::general(),
        )
        .expect("bursts should detect");
    assert_eq!(bursts.len(), 1);

    let target_burst_id = bursts[0].burst_group_id.clone();
    let before_page = store
        .asset_group_page(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load before merge");
    let source_burst_id = before_page
        .groups
        .iter()
        .find(|group| group.group_id.as_deref() == Some(source_member_group_id.as_str()))
        .and_then(|group| group.burst.as_ref())
        .expect("source burst should exist")
        .burst_group_id
        .clone();
    let merged = store
        .merge_burst_member(&target_burst_id, &source_member_group_id)
        .expect("member group should merge into target burst")
        .expect("target burst should remain");

    assert_eq!(merged.member_count, 4);
    assert_eq!(merged.recommendation_status, "pending");
    assert_eq!(merged.user_override_state.as_deref(), Some("merge"));
    assert_eq!(
        store.burst_group(&source_burst_id).expect("source lookup"),
        None
    );

    store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group_id,
            &StrategyProfile::general(),
        )
        .expect("later detection should run");
    let page = store
        .asset_group_page(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let burst_ids = page
        .groups
        .iter()
        .filter_map(|group| {
            group
                .burst
                .as_ref()
                .map(|burst| (burst.burst_group_id.clone(), burst.member_count))
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(burst_ids, BTreeSet::from([(target_burst_id, 4)]));
}

fn publish_jpeg(
    store: &SqliteStore,
    project_id: &str,
    transfer_id: &str,
    original_path: &str,
    started_at_ms: i64,
) {
    publish_jpeg_with_user(
        store,
        project_id,
        transfer_id,
        original_path,
        "z5",
        started_at_ms,
    )
}

fn publish_jpeg_with_user(
    store: &SqliteStore,
    project_id: &str,
    transfer_id: &str,
    original_path: &str,
    username: &str,
    started_at_ms: i64,
) {
    let final_filename = original_path.rsplit('/').next().unwrap();
    let item = store
        .enqueue_publish_with_metadata(
            project_id,
            transfer_id,
            &format!("staged/{final_filename}.tmp"),
            final_filename,
            100,
            PublishTransferMetadata {
                protocol: "ftp".to_string(),
                original_path: original_path.to_string(),
                username: Some(username.to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Nikon Z".to_string()),
                started_at_ms,
            },
        )
        .expect("publish should enqueue");
    store
        .complete_publish(
            &item.queue_id,
            final_filename,
            StoredObjectLocation::local_path(final_filename),
        )
        .expect("publish should complete");
}

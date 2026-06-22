use super::*;

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
            &BurstGroupingProfile::default(),
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
    assert_eq!(updated.manual_grouping_state.as_deref(), Some("split"));
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
            &BurstGroupingProfile::default(),
        )
        .expect("bursts should detect");
    store
        .split_burst_member(&bursts[0].burst_group_id, &split_group_id)
        .expect("member should split");

    store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group_id,
            &BurstGroupingProfile::default(),
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
    assert!(burst_member_counts.is_empty());
}

#[test]
fn manual_merge_member_groups_can_create_new_burst_group_from_singles() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Manual New Burst Project")
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
        "DCIM/101/IMG_3001.JPG",
        5000,
    );

    let groups = store
        .stored_asset_groups(&project.project_id)
        .expect("groups should query");
    let member_group_ids = groups
        .iter()
        .map(|group| group.group_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(member_group_ids.len(), 2);

    let merged = store
        .create_manual_burst_group(&project.project_id, &member_group_ids)
        .expect("manual burst should create")
        .expect("manual burst should exist");

    assert_eq!(merged.member_count, 2);
    assert_eq!(merged.manual_grouping_state.as_deref(), Some("merge"));
    assert_eq!(merged.recommendation_status, "pending");

    store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &member_group_ids[0],
            &BurstGroupingProfile::default(),
        )
        .expect("later detection should preserve manual merge");
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

    assert_eq!(burst_ids, BTreeSet::from([(merged.burst_group_id, 2)]));
}

#[test]
fn manual_merge_member_groups_can_merge_existing_burst_containers() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Manual Merge Burst Containers Project")
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
    let second_group_id = groups
        .iter()
        .find(|group| group.display_key == "IMG_2001")
        .expect("second burst member should exist")
        .group_id
        .clone();
    store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group_id,
            &BurstGroupingProfile::default(),
        )
        .expect("initial bursts should detect");

    let merged = store
        .create_manual_burst_group(
            &project.project_id,
            &[first_group_id.clone(), second_group_id],
        )
        .expect("manual burst should merge")
        .expect("manual burst should exist");

    assert_eq!(merged.member_count, 4);
    assert_eq!(merged.manual_grouping_state.as_deref(), Some("merge"));
    assert_eq!(merged.recommendation_status, "pending");
    assert_eq!(
        merged
            .member_group_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        groups
            .iter()
            .map(|group| group.group_id.clone())
            .collect::<BTreeSet<_>>(),
    );

    store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group_id,
            &BurstGroupingProfile::default(),
        )
        .expect("later detection should preserve manual container merge");
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

    assert_eq!(burst_ids, BTreeSet::from([(merged.burst_group_id, 4)]));
}

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use camera_connector_core::{
    BurstGroupingProfile, PublishTransferMetadata, SqliteStore, StoredObjectLocation,
    TechnicalAssessment, TechnicalAssessmentStatus, TechnicalGateStatus,
};

#[test]
fn burst_grouping_without_capture_time_uses_filename_sequence() {
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
            &BurstGroupingProfile::default(),
        )
        .expect("bursts should detect");

    assert_eq!(bursts.len(), 1);
    assert_eq!(bursts[0].member_count, 3);
    assert_eq!(bursts[0].recommendation_status, "pending");
}

#[test]
fn burst_grouping_without_capture_time_does_not_chain_by_received_time_only() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Received Time Project")
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
        "DCIM/100/IMG_1050.JPG",
        1100,
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
            &BurstGroupingProfile::default(),
        )
        .expect("bursts should detect");

    assert!(bursts.is_empty());
}

#[test]
fn burst_grouping_with_capture_time_does_not_chain_by_filename_sequence_only() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Capture Time Project")
        .expect("project should create");

    publish_jpeg_with_capture_file(
        &store,
        &project.project_id,
        "ftp:1",
        "DCIM/100/IMG_1001.JPG",
        temp_dir.path(),
        "2026:01:24 12:00:00",
        "00",
    );
    publish_jpeg_with_capture_file(
        &store,
        &project.project_id,
        "ftp:2",
        "DCIM/100/IMG_1002.JPG",
        temp_dir.path(),
        "2026:01:24 12:05:00",
        "00",
    );
    publish_jpeg_with_capture_file(
        &store,
        &project.project_id,
        "ftp:3",
        "DCIM/100/IMG_1003.JPG",
        temp_dir.path(),
        "2026:01:24 12:05:00",
        "08",
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
            &BurstGroupingProfile::default(),
        )
        .expect("bursts should detect");
    let page = store
        .asset_group_page(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let grouped_keys = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .map(|group| group.group_key.clone())
        .collect::<BTreeSet<_>>();

    assert_eq!(bursts.len(), 1);
    assert_eq!(bursts[0].member_count, 2);
    assert_eq!(
        grouped_keys,
        BTreeSet::from(["IMG_1002".to_string(), "IMG_1003".to_string()])
    );
}

#[test]
fn visual_refinement_splits_time_candidate_burst_when_frame_signature_changes() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Visual Split Project")
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
    let first_group = groups.first().expect("group should exist").group_id.clone();
    let initial_bursts = store
        .detect_bursts_for_asset_group(
            &project.project_id,
            &first_group,
            &BurstGroupingProfile::default(),
        )
        .expect("bursts should detect");
    assert_eq!(initial_bursts.len(), 1);
    assert_eq!(initial_bursts[0].member_count, 3);

    let mut ids_by_key = groups
        .iter()
        .map(|group| (group.display_key.as_str(), group.group_id.clone()))
        .collect::<Vec<_>>();
    ids_by_key.sort_by_key(|(key, _)| *key);
    store
        .save_technical_assessment(technical_assessment_with_signature(
            &ids_by_key[0].1,
            "ahash-v1:ffff0000ffff0000",
        ))
        .expect("first assessment should save");
    store
        .save_technical_assessment(technical_assessment_with_signature(
            &ids_by_key[1].1,
            "ahash-v1:ffff0000ffff0000",
        ))
        .expect("second assessment should save");
    store
        .save_technical_assessment(technical_assessment_with_signature(
            &ids_by_key[2].1,
            "ahash-v1:0000ffff0000ffff",
        ))
        .expect("third assessment should save");

    let refined = store
        .refine_burst_group_by_visual_similarity(
            &initial_bursts[0].burst_group_id,
            &BurstGroupingProfile::default(),
            "technical-v1",
        )
        .expect("visual refinement should run");
    let page = store
        .asset_group_page(&project.project_id, Default::default(), 0, 25)
        .expect("asset page should load");
    let grouped_keys = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .map(|group| group.group_key.clone())
        .collect::<BTreeSet<_>>();

    assert_eq!(refined.len(), 1);
    assert_eq!(refined[0].member_count, 2);
    assert_eq!(
        grouped_keys,
        BTreeSet::from(["IMG_1001".to_string(), "IMG_1002".to_string()])
    );
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
            &BurstGroupingProfile::default(),
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
            &BurstGroupingProfile::default(),
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
            &BurstGroupingProfile::default(),
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
    assert_eq!(merged.manual_grouping_state.as_deref(), Some("merge"));
    assert_eq!(
        store.burst_group(&source_burst_id).expect("source lookup"),
        None
    );

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

fn technical_assessment_with_signature(group_id: &str, signature: &str) -> TechnicalAssessment {
    TechnicalAssessment {
        asset_group_id: group_id.to_string(),
        assessor_version: "technical-v1".to_string(),
        status: TechnicalAssessmentStatus::Ready,
        gate_status: TechnicalGateStatus::Pass,
        defect_flags: Vec::new(),
        preview_source: Some("test".to_string()),
        visual_signature: Some(signature.to_string()),
        analyzed_at_ms: 1_000,
    }
}

fn publish_jpeg_with_capture_file(
    store: &SqliteStore,
    project_id: &str,
    transfer_id: &str,
    original_path: &str,
    output_dir: &Path,
    datetime: &str,
    subsec: &str,
) {
    let final_filename = original_path.rsplit('/').next().unwrap();
    let final_path = output_dir.join(final_filename);
    fs::write(&final_path, minimal_exif_jpeg(datetime, subsec, "+08:00"))
        .expect("test jpeg should write");
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
                username: Some("z5".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Nikon Z".to_string()),
                started_at_ms: 1000,
            },
        )
        .expect("publish should enqueue");
    store
        .complete_publish(
            &item.queue_id,
            final_filename,
            StoredObjectLocation::local_path(final_path),
        )
        .expect("publish should complete");
}

fn minimal_exif_jpeg(datetime: &str, subsec: &str, offset: &str) -> Vec<u8> {
    let datetime_bytes = nul_terminated(datetime);
    let subsec_bytes = nul_terminated(subsec);
    let offset_bytes = nul_terminated(offset);
    let ifd0_offset = 8_u32;
    let ifd0_size = 2 + 2 * 12 + 4;
    let exif_ifd_offset = ifd0_offset + ifd0_size;
    let exif_ifd_size = 2 + 4 * 12 + 4;
    let mut data_offset = exif_ifd_offset + exif_ifd_size;
    let image_datetime_offset = data_offset;
    data_offset += datetime_bytes.len() as u32;
    let original_datetime_offset = data_offset;
    data_offset += datetime_bytes.len() as u32;
    let digitized_datetime_offset = data_offset;
    data_offset += datetime_bytes.len() as u32;
    let subsec_original_offset = data_offset;
    data_offset += subsec_bytes.len() as u32;
    let offset_original_offset = data_offset;

    let mut tiff = Vec::new();
    tiff.extend_from_slice(b"II");
    tiff.extend_from_slice(&42_u16.to_le_bytes());
    tiff.extend_from_slice(&ifd0_offset.to_le_bytes());
    tiff.extend_from_slice(&2_u16.to_le_bytes());
    push_ascii_entry(
        &mut tiff,
        0x0132,
        datetime_bytes.len() as u32,
        image_datetime_offset,
    );
    push_long_entry(&mut tiff, 0x8769, exif_ifd_offset);
    tiff.extend_from_slice(&0_u32.to_le_bytes());

    tiff.extend_from_slice(&4_u16.to_le_bytes());
    push_ascii_entry(
        &mut tiff,
        0x9003,
        datetime_bytes.len() as u32,
        original_datetime_offset,
    );
    push_ascii_entry(
        &mut tiff,
        0x9004,
        datetime_bytes.len() as u32,
        digitized_datetime_offset,
    );
    push_ascii_entry(
        &mut tiff,
        0x9291,
        subsec_bytes.len() as u32,
        subsec_original_offset,
    );
    push_ascii_entry(
        &mut tiff,
        0x9011,
        offset_bytes.len() as u32,
        offset_original_offset,
    );
    tiff.extend_from_slice(&0_u32.to_le_bytes());

    tiff.extend_from_slice(&datetime_bytes);
    tiff.extend_from_slice(&datetime_bytes);
    tiff.extend_from_slice(&datetime_bytes);
    tiff.extend_from_slice(&subsec_bytes);
    tiff.extend_from_slice(&offset_bytes);

    let mut app1_payload = b"Exif\0\0".to_vec();
    app1_payload.extend_from_slice(&tiff);
    let segment_len = (app1_payload.len() + 2) as u16;
    let mut jpeg = vec![0xff, 0xd8, 0xff, 0xe1];
    jpeg.extend_from_slice(&segment_len.to_be_bytes());
    jpeg.extend_from_slice(&app1_payload);
    jpeg.extend_from_slice(&[0xff, 0xd9]);
    jpeg
}

fn push_ascii_entry(buffer: &mut Vec<u8>, tag: u16, count: u32, value_offset: u32) {
    buffer.extend_from_slice(&tag.to_le_bytes());
    buffer.extend_from_slice(&2_u16.to_le_bytes());
    buffer.extend_from_slice(&count.to_le_bytes());
    buffer.extend_from_slice(&value_offset.to_le_bytes());
}

fn push_long_entry(buffer: &mut Vec<u8>, tag: u16, value: u32) {
    buffer.extend_from_slice(&tag.to_le_bytes());
    buffer.extend_from_slice(&4_u16.to_le_bytes());
    buffer.extend_from_slice(&1_u32.to_le_bytes());
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn nul_terminated(value: &str) -> Vec<u8> {
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    bytes
}

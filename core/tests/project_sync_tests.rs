use camera_connector_core::{
    match_project_sync_snapshot, parse_project_sync_snapshot_json, CameraConnectorService,
    ModelEvaluatorKind, ObjectFormat, ProjectSyncProjectSummary, ProjectSyncSnapshot,
    ProjectSyncSnapshotAsset, ProjectSyncSnapshotGroup, ProjectSyncSnapshotModelEvaluation,
    ProjectSyncSnapshotRecommendation, ProjectSyncSnapshotUserMarks, ProjectSyncSourceDevice,
    SelectionRecommendationScope, SelectionSource, StoredAsset, StoredAssetGroup,
};

#[test]
fn project_sync_snapshot_parses_minimal_versioned_json() {
    let snapshot = parse_project_sync_snapshot_json(
        r#"{
          "schema_version": 1,
          "source_device": {
            "device_id": "phone-1",
            "device_label": "Pixel Field Kit",
            "platform": "android"
          },
          "project": {
            "project_id": "android-project-1",
            "name": "Wedding Selects",
            "exported_at_ms": 1781800000000
          },
          "assets": [{
            "asset_id": "asset-a",
            "group_id": "group-a",
            "original_filename": "IMG_1001.JPG",
            "final_filename": "IMG_1001.JPG",
            "normalized_stem": "IMG_1001",
            "original_path": "DCIM/100NIKON/IMG_1001.JPG",
            "original_parent_path": "DCIM/100NIKON",
            "format": "jpeg",
            "size_bytes": 4,
            "capture_at_ms": 1781000000000,
            "received_at_ms": 1781000001000,
            "source_identity": "camera-card-a"
          }],
          "groups": [{
            "group_id": "group-a",
            "display_key": "IMG_1001",
            "source_identity": "camera-card-a",
            "original_parent_path": "DCIM/100NIKON",
            "member_asset_ids": ["asset-a"],
            "primary_asset_id": "asset-a",
            "preview_asset_id": "asset-a",
            "has_raw": false,
            "has_jpeg": true,
            "has_video": false
          }],
          "model_evaluations": [],
          "selection_recommendations": [],
          "user_marks": []
        }"#,
    )
    .expect("snapshot should parse");

    assert_eq!(snapshot.schema_version, 1);
    assert_eq!(snapshot.source_device.device_label, "Pixel Field Kit");
    assert_eq!(snapshot.project.name, "Wedding Selects");
    assert_eq!(snapshot.assets[0].asset_id, "asset-a");
    assert_eq!(snapshot.groups[0].member_asset_ids, vec!["asset-a"]);
}

#[test]
fn project_sync_snapshot_rejects_unsupported_schema_version() {
    let error = parse_project_sync_snapshot_json(
        r#"{
          "schema_version": 2,
          "source_device": {"device_id": "phone", "device_label": "Phone", "platform": "android"},
          "project": {"project_id": "p", "name": "P", "exported_at_ms": 1},
          "assets": [],
          "groups": [],
          "model_evaluations": [],
          "selection_recommendations": [],
          "user_marks": []
        }"#,
    )
    .expect_err("version 2 should be rejected");

    assert!(error
        .to_string()
        .contains("unsupported project sync schema_version 2"));
}

#[test]
fn project_sync_matches_asset_by_filename_format_size_and_capture_time() {
    let snapshot = snapshot_with_one_asset("remote-asset", "remote-group", "IMG_2001.JPG");
    let local_asset = stored_asset("local-asset", "local-group", "IMG_2001.JPG")
        .with_original_path("different/local/root/IMG_2001.JPG");
    let local_group = stored_group("local-group", "IMG_2001");

    let result = match_project_sync_snapshot(&snapshot, &[local_asset], &[local_group]);

    assert_eq!(
        result.matched_assets.get("remote-asset"),
        Some(&"local-asset".to_string())
    );
    assert_eq!(
        result.matched_groups.get("remote-group"),
        Some(&"local-group".to_string())
    );
    assert!(result.unmatched_assets.is_empty());
    assert!(result.ambiguous_assets.is_empty());
}

#[test]
fn project_sync_does_not_treat_cross_device_original_path_as_identity() {
    let snapshot = snapshot_with_one_asset("remote-asset", "remote-group", "IMG_2003.JPG");
    let wrong_same_path = stored_asset("wrong-local-asset", "wrong-local-group", "OTHER.JPG")
        .with_original_path("DCIM/IMG_2003.JPG");
    let right_different_path =
        stored_asset("right-local-asset", "right-local-group", "IMG_2003.JPG")
            .with_original_path("desktop/imported/folder/IMG_2003.JPG");
    let wrong_group = stored_group("wrong-local-group", "OTHER");
    let right_group = stored_group("right-local-group", "IMG_2003");

    let result = match_project_sync_snapshot(
        &snapshot,
        &[wrong_same_path, right_different_path],
        &[wrong_group, right_group],
    );

    assert_eq!(
        result.matched_assets.get("remote-asset"),
        Some(&"right-local-asset".to_string())
    );
    assert_eq!(
        result.matched_groups.get("remote-group"),
        Some(&"right-local-group".to_string())
    );
    assert!(result.unmatched_assets.is_empty());
    assert!(result.ambiguous_assets.is_empty());
}

#[test]
fn service_sync_project_snapshot_applies_matched_existing_data_only() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("config.json");
    let root = temp_dir.path().join("photos");
    std::fs::create_dir_all(&root).expect("photo root should create");
    std::fs::write(root.join("IMG_4100.JPG"), [1_u8, 2, 3, 4]).expect("jpeg should write");

    let service = CameraConnectorService::new(Some(config_path));
    let project = service
        .create_project("Snapshot Sync")
        .expect("project should create");
    let scan = service
        .create_desktop_project_scan(&project.project_id, &root)
        .expect("scan should queue");
    service
        .run_desktop_project_scan(&scan.scan_id)
        .expect("scan should complete");

    let mut snapshot = snapshot_with_one_asset("remote-asset", "remote-group", "IMG_4100.JPG");
    snapshot.assets[0].size_bytes = 4;
    snapshot.user_marks.push(ProjectSyncSnapshotUserMarks {
        group_id: "remote-group".to_string(),
        favorite: Some(true),
        marked: Some(true),
    });
    snapshot
        .model_evaluations
        .push(ProjectSyncSnapshotModelEvaluation {
            evaluation_id: "remote-eval".to_string(),
            group_id: "remote-group".to_string(),
            evaluator_version: "android-model-v1".to_string(),
            status: "ready".to_string(),
            score: 91,
            tier: "excellent".to_string(),
            selectable: true,
            summary: "strong transferred pick".to_string(),
            strengths: vec!["sharp".to_string()],
            weaknesses: Vec::new(),
            technical_warnings: Vec::new(),
            prompt_pack_id: None,
            prompt_pack_version: None,
            prompt_hash: None,
            created_at_ms: 1781000002000,
            updated_at_ms: 1781000002000,
        });
    snapshot
        .selection_recommendations
        .push(ProjectSyncSnapshotRecommendation {
            recommendation_id: "remote-rec".to_string(),
            scope: "project".to_string(),
            subject_group_id: None,
            selected_group_ids: vec!["remote-group".to_string()],
            candidate_group_ids: vec!["remote-group".to_string()],
            rejected_group_ids: Vec::new(),
            status: "ready".to_string(),
            confidence: 0.92,
            reason: "best from phone review".to_string(),
            created_at_ms: 1781000003000,
            updated_at_ms: 1781000003000,
        });

    let summary = service
        .sync_project_snapshot(&project.project_id, &snapshot)
        .expect("snapshot sync should apply");

    assert_eq!(summary.matched_assets, 1);
    assert_eq!(summary.matched_groups, 1);
    assert_eq!(summary.applied_user_marks, 1);
    assert_eq!(summary.applied_model_evaluations, 1);
    assert_eq!(summary.applied_selection_recommendations, 1);
    assert_eq!(summary.unresolved_records, 0);
    assert_eq!(summary.ambiguous_records, 0);

    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    let group = page.groups.first().expect("one group should exist");
    let group_id = group.group_id.as_ref().expect("group id should exist");
    assert!(group.user_marks.favorite);
    assert!(group.user_marks.marked);

    let store = service.storage_store().expect("store should open");
    let evaluations = store
        .model_evaluations_for_asset_groups(std::slice::from_ref(group_id), "android-model-v1")
        .expect("evaluations should query");
    assert_eq!(evaluations.len(), 1);
    assert_eq!(evaluations[0].evaluator_kind, ModelEvaluatorKind::Imported);
    assert_eq!(evaluations[0].score, 91);

    let recommendation = store
        .latest_selection_recommendation(
            &project.project_id,
            SelectionRecommendationScope::Project,
            &project.project_id,
        )
        .expect("recommendation should query")
        .expect("recommendation should be saved");
    assert_eq!(recommendation.source, SelectionSource::Imported);
    assert_eq!(
        recommendation.selected_asset_group_ids,
        vec![group_id.clone()]
    );
}

fn snapshot_with_one_asset(asset_id: &str, group_id: &str, filename: &str) -> ProjectSyncSnapshot {
    let stem = filename
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(filename);
    ProjectSyncSnapshot {
        schema_version: 1,
        source_device: ProjectSyncSourceDevice {
            device_id: "phone-1".to_string(),
            device_label: "Pixel Field Kit".to_string(),
            platform: "android".to_string(),
        },
        project: ProjectSyncProjectSummary {
            project_id: "android-project-1".to_string(),
            name: "Wedding Selects".to_string(),
            exported_at_ms: 1781800000000,
        },
        assets: vec![ProjectSyncSnapshotAsset {
            asset_id: asset_id.to_string(),
            group_id: group_id.to_string(),
            original_filename: filename.to_string(),
            final_filename: filename.to_string(),
            normalized_stem: stem.to_string(),
            original_path: format!("DCIM/{filename}"),
            original_parent_path: Some("DCIM".to_string()),
            format: "jpeg".to_string(),
            size_bytes: 42,
            capture_at_ms: Some(1781000000000),
            received_at_ms: Some(1781000001000),
            source_identity: Some("camera-card-a".to_string()),
        }],
        groups: vec![ProjectSyncSnapshotGroup {
            group_id: group_id.to_string(),
            display_key: stem.to_string(),
            source_identity: Some("camera-card-a".to_string()),
            original_parent_path: Some("DCIM".to_string()),
            member_asset_ids: vec![asset_id.to_string()],
            primary_asset_id: Some(asset_id.to_string()),
            preview_asset_id: Some(asset_id.to_string()),
            has_raw: false,
            has_jpeg: true,
            has_video: false,
        }],
        model_evaluations: Vec::new(),
        selection_recommendations: Vec::new(),
        user_marks: Vec::new(),
    }
}

fn stored_asset(asset_id: &str, group_id: &str, filename: &str) -> StoredAsset {
    let stem = filename
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(filename);
    StoredAsset {
        asset_id: asset_id.to_string(),
        project_id: "desktop-project-1".to_string(),
        group_id: Some(group_id.to_string()),
        transfer_id: "desktop-scan".to_string(),
        group_role: "jpeg".to_string(),
        media_kind: "photo".to_string(),
        format: ObjectFormat::Jpeg,
        original_filename: filename.to_string(),
        final_filename: filename.to_string(),
        normalized_stem: stem.to_string(),
        original_path: filename.to_string(),
        original_parent_path: Some("DCIM".to_string()),
        final_location: None,
        size_bytes: 42,
        capture_at_ms: Some(1781000000000),
        received_at_ms: Some(1781000001000),
        published_at_ms: None,
        source_identity: Some("camera-card-a".to_string()),
        username: None,
        remote_addr: None,
        source_status: "available".to_string(),
        source_modified_at_ms: None,
        last_seen_scan_id: Some("scan-1".to_string()),
        duplicate_index: None,
        duplicate_count: None,
    }
}

fn stored_group(group_id: &str, display_key: &str) -> StoredAssetGroup {
    StoredAssetGroup {
        group_id: group_id.to_string(),
        project_id: "desktop-project-1".to_string(),
        group_identity: format!("camera-card-a:{display_key}"),
        display_key: display_key.to_string(),
        source_identity: Some("camera-card-a".to_string()),
        original_parent_path: Some("DCIM".to_string()),
        primary_asset_id: None,
        preview_asset_id: None,
        member_count: 1,
        has_raw: false,
        has_jpeg: true,
        has_video: false,
        first_capture_at_ms: Some(1781000000000),
        last_capture_at_ms: Some(1781000000000),
        first_received_at_ms: Some(1781000001000),
        last_received_at_ms: Some(1781000001000),
        created_at_ms: 1781000001000,
        updated_at_ms: 1781000001000,
    }
}

trait StoredAssetTestExt {
    fn with_original_path(self, original_path: &str) -> Self;
}

impl StoredAssetTestExt for StoredAsset {
    fn with_original_path(mut self, original_path: &str) -> Self {
        self.original_path = original_path.to_string();
        self.original_parent_path = original_path
            .rsplit_once('/')
            .map(|(parent, _)| parent.to_string());
        self
    }
}

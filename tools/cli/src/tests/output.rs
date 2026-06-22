use super::*;
use std::path::PathBuf;

#[test]
fn receive_file_command_indexes_upload_under_explicit_project() {
    let root = std::env::temp_dir().join(format!(
        "camera-connector-receive-file-{}",
        current_time_ms()
    ));
    let input = root.join("IMG_0001.CR3");
    let output = root.join("output");
    let state = root.join("state");
    std::fs::create_dir_all(&root).expect("temp root should create");
    std::fs::write(&input, [1_u8, 2, 3, 4]).expect("sample should write");

    let store =
        camera_connector_core::SqliteStore::open_state_dir(&state).expect("storage should open");
    let project = store
        .create_project("CLI Shoot")
        .expect("project should create");

    let record = handle_receive_file_command(ReceiveFileArgs {
        input,
        output,
        project_id: project.project_id.clone(),
        state: Some(state.clone()),
        source: ImportSource::FtpPush,
        username: Some("verify".to_string()),
        source_name: Some("Verify Camera".to_string()),
    })
    .expect("receive-file should index upload");

    let page = store
        .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 50)
        .expect("project assets should load");

    assert_eq!(record.final_filename, "IMG_0001.CR3");
    assert_eq!(page.summary.asset_count, 1);
    assert_eq!(page.groups[0].primary.filename, "IMG_0001.CR3");
    assert_eq!(page.groups[0].primary.username.as_deref(), Some("verify"));
    assert_eq!(
        page.groups[0].primary.display_source.as_deref(),
        Some("Verify Camera")
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn dashboard_json_output_contains_status_devices_and_assets() {
    let asset = ReceivedAsset::new("ftp:1", "IMG_0001.CR3", 42, ImportSource::FtpPush);
    let dashboard = CameraConnectorDashboard {
        receiver_status: Some(ReceiverRuntimeStatus {
            phase: camera_connector_core::ReceiverRuntimePhase::Stopped,
            protocol: Some(PushProtocol::Ftp),
            auth_mode: camera_connector_core::ReceiverAuthMode::Accounts,
            local_addr: None,
            output_dir: None,
            state_dir: None,
            account_count: 1,
            message: None,
        }),
        receiver_settings: ReceiverSettingsConfig::default(),
        paths: camera_connector_core::SystemPathsView {
            config_path: PathBuf::from("C:\\CameraConnector\\config.json"),
            state_dir: PathBuf::from("C:\\CameraConnector\\state"),
            output_dir: None,
        },
        accounts: vec![camera_connector_core::AccountView {
            username: "z5".to_string(),
            device_name: "Camera".to_string(),
            password_configured: true,
            online: true,
            active_connections: 1,
            last_remote_addr: Some("192.168.137.56".to_string()),
            last_remote_port: Some(50123),
            last_seen_at_ms: Some(20),
            last_disconnected_at_ms: None,
        }],
        transfers: camera_connector_core::TransferSummary {
            total_count: 2,
            completed_count: 1,
            failed_count: 1,
        },
        publish_queue: camera_connector_core::PublishQueueSummary {
            total_count: 3,
            pending_count: 2,
            staged_count: 1,
            publishing_count: 0,
            completed_count: 1,
            failed_count: 1,
        },
        global_assets: camera_connector_core::GlobalAssetSummary {
            photo_count: 1,
            file_count: 1,
            storage_bytes: 42,
        },
        recent_failures: vec![TransferRecordView {
            record: TransferRecord {
                transfer_id: "ftp:failed".to_string(),
                protocol: "ftp".to_string(),
                status: TransferStatus::Failed,
                original_path: "IMG_0002.CR3".to_string(),
                final_filename: "IMG_0002.CR3".to_string(),
                final_location: None,
                size_bytes: 0,
                username: Some("z5".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Camera".to_string()),
                started_at_ms: 11,
                completed_at_ms: Some(21),
                error: Some("connection reset".to_string()),
            },
            display_source: Some("Camera".to_string()),
            virtual_display_path: "Camera/IMG_0002.CR3".to_string(),
            final_location_kind: None,
            final_location_label: None,
        }],
        recent_publish_failures: Vec::new(),
        devices: vec![camera_connector_core::ConnectedDeviceView {
            device: camera_connector_core::ConnectedDevice {
                remote_addr: "192.168.137.56".to_string(),
                source_name: Some("Camera".to_string()),
                username: Some("z5".to_string()),
                online: true,
                last_seen_at_ms: 20,
                first_seen_at_ms: 10,
                last_disconnected_at_ms: None,
                active_connections: 1,
                last_remote_port: Some(50123),
            },
            display_source: "Camera".to_string(),
        }],
        assets: AssetGroupPage {
            groups: vec![ReceivedAssetGroup {
                group_id: None,
                group_key: "IMG_0001".to_string(),
                primary: asset.clone(),
                jpeg: None,
                raw: Some(asset),
                video: None,
                burst: None,
                technical_status: None,
                technical_gate_status: None,
                technical_defects: Vec::new(),
                model_status: None,
                model_score: None,
                model_tier: None,
                model_evaluator_kind: None,
                model_summary: None,
                is_model_select: false,
                is_favorite: false,
                is_flagged: false,
                user_marks: camera_connector_core::AssetUserMarks::default(),
                guest_mark: None,
            }],
            summary: AssetGroupSummary {
                group_count: 1,
                asset_count: 1,
                groups_with_jpeg: 0,
                groups_with_raw: 1,
                groups_with_video: 0,
                source_counts: Vec::new(),
                remote_addr_counts: Vec::new(),
            },
            offset: 0,
            limit: 50,
            total_groups: 1,
            has_more: false,
        },
    };

    let json = dashboard_json(&dashboard).expect("dashboard should serialize");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("dashboard json should parse");

    assert!(json.contains("\"receiver_status\""));
    assert!(json.contains("\"paths\""));
    assert!(json.contains("\"config_path\""));
    assert!(json.contains("\"state_dir\""));
    assert!(json.contains("\"global_assets\""));
    assert_eq!(parsed["global_assets"]["storage_bytes"].as_i64(), Some(42));
    assert!(json.contains("\"accounts\""));
    assert!(json.contains("\"password_configured\": true"));
    assert!(json.contains("\"online\": true"));
    assert!(json.contains("\"last_remote_addr\": \"192.168.137.56\""));
    assert!(!json.contains("password_hash"));
    assert!(json.contains("\"transfers\""));
    assert!(json.contains("\"failed_count\": 1"));
    assert!(json.contains("\"publish_queue\""));
    assert!(json.contains("\"pending_count\": 2"));
    assert!(json.contains("\"recent_failures\""));
    assert!(json.contains("\"connection reset\""));
    assert!(json.contains("\"devices\""));
    assert!(json.contains("\"assets\""));
    assert!(json.contains("\"group_key\": \"IMG_0001\""));
}

#[test]
fn transfer_view_line_prints_platform_location() {
    let view = TransferRecordView {
        record: TransferRecord {
            transfer_id: "sftp:1".to_string(),
            protocol: "sftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/IMG_0001.DNG".to_string(),
            final_filename: "IMG_0001.DNG".to_string(),
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_0001.DNG",
            )),
            size_bytes: 42,
            username: None,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Camera".to_string()),
            started_at_ms: 10,
            completed_at_ms: Some(20),
            error: None,
        },
        display_source: Some("Camera".to_string()),
        virtual_display_path: "Camera/DCIM/IMG_0001.DNG".to_string(),
        final_location_kind: Some("document_uri".to_string()),
        final_location_label: Some("content://camera-connector/IMG_0001.DNG".to_string()),
    };

    let line = transfer_view_line(&view);

    assert!(line.contains("username=-"));
    assert!(line.contains("error=-"));
    assert!(line.contains("location_kind=document_uri"));
    assert!(line.contains("location=content://camera-connector/IMG_0001.DNG"));
}

#[test]
fn transfer_view_line_prints_failure_error() {
    let view = TransferRecordView {
        record: TransferRecord {
            transfer_id: "ftp:failed".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Failed,
            original_path: "IMG_0002.CR3".to_string(),
            final_filename: "IMG_0002.CR3".to_string(),
            final_location: None,
            size_bytes: 0,
            username: Some("z5".to_string()),
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Camera".to_string()),
            started_at_ms: 11,
            completed_at_ms: Some(21),
            error: Some("connection reset".to_string()),
        },
        display_source: Some("Camera".to_string()),
        virtual_display_path: "Camera/IMG_0002.CR3".to_string(),
        final_location_kind: None,
        final_location_label: None,
    };

    let line = transfer_view_line(&view);

    assert!(line.contains("Failed"));
    assert!(line.contains("error=connection reset"));
}

#[test]
fn asset_group_line_prints_primary_storage_location() {
    let asset = ReceivedAsset::new("ftp:1", "IMG_0001.CR3", 42, ImportSource::FtpPush)
        .with_storage_location(StoredObjectLocation::document_uri(
            "content://camera-connector/IMG_0001.CR3",
        ));
    let mut asset = asset;
    asset.display_source = Some("Z5_2".to_string());
    asset.username = Some("z5".to_string());
    asset.remote_addr = Some("192.168.137.56".to_string());
    asset.original_path = Some("DCIM/IMG_0001.CR3".to_string());
    asset.virtual_display_path = Some("Z5_2/DCIM/IMG_0001.CR3".to_string());
    let group = ReceivedAssetGroup {
        group_id: Some("group-stable-1".to_string()),
        group_key: "IMG_0001".to_string(),
        primary: asset.clone(),
        jpeg: None,
        raw: Some(asset),
        video: None,
        burst: None,
        technical_status: None,
        technical_gate_status: None,
        technical_defects: Vec::new(),
        model_status: None,
        model_score: None,
        model_tier: None,
        model_evaluator_kind: None,
        model_summary: None,
        is_model_select: false,
        is_favorite: false,
        is_flagged: false,
        user_marks: camera_connector_core::AssetUserMarks::default(),
        guest_mark: None,
    };

    let line = asset_group_line(&group);

    assert!(line.contains("group_id=group-stable-1"));
    assert!(line.contains("primary_location_kind=document_uri"));
    assert!(line.contains("primary_location=content://camera-connector/IMG_0001.CR3"));
    assert!(line.contains("username=z5"));
    assert!(line.contains("source=Z5_2"));
    assert!(line.contains("remote=192.168.137.56"));
    assert!(line.contains("original=DCIM/IMG_0001.CR3"));
    assert!(line.contains("display=Z5_2/DCIM/IMG_0001.CR3"));
    assert!(line.contains("duplicate=-"));
}

#[test]
fn asset_group_summary_line_prints_filter_counts() {
    let summary = AssetGroupSummary {
        group_count: 2,
        asset_count: 3,
        groups_with_jpeg: 1,
        groups_with_raw: 1,
        groups_with_video: 1,
        source_counts: vec![AssetFacetCount {
            value: "Z5_2".to_string(),
            group_count: 2,
        }],
        remote_addr_counts: vec![AssetFacetCount {
            value: "192.168.137.56".to_string(),
            group_count: 2,
        }],
    };

    let line = asset_group_summary_line(&summary);

    assert!(line.contains("groups=2"));
    assert!(line.contains("raw_groups=1"));
    assert!(line.contains("sources=Z5_2:2"));
    assert!(line.contains("remotes=192.168.137.56:2"));
}

#[test]
fn asset_group_page_summary_line_prints_paging_state() {
    let page = AssetGroupPage {
        groups: Vec::new(),
        summary: AssetGroupSummary {
            group_count: 3,
            asset_count: 4,
            groups_with_jpeg: 1,
            groups_with_raw: 2,
            groups_with_video: 1,
            source_counts: Vec::new(),
            remote_addr_counts: Vec::new(),
        },
        offset: 1,
        limit: 1,
        total_groups: 3,
        has_more: true,
    };

    let line = asset_group_page_summary_line(&page);

    assert!(line.contains("groups=3"));
    assert!(line.contains("offset=1"));
    assert!(line.contains("limit=1"));
    assert!(line.contains("total_groups=3"));
    assert!(line.contains("has_more=true"));
}

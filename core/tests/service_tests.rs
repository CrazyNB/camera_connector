use camera_connector_core::{
    append_transfer_record, record_device_authenticated, record_device_connected,
    write_receiver_runtime_status, AssetGroupQuery, CameraConnectorConfig, CameraConnectorService,
    ImportSource, ObjectFormat, PushProtocol, ReceiverAuthMode, ReceiverConfigRequest,
    ReceiverRuntimePhase, ReceiverRuntimeStatus, StoredObjectLocation, TransferQuery,
    TransferRecord, TransferStatus,
};

#[test]
fn service_builds_receiver_config_from_saved_accounts() {
    let config_path = unique_temp_path("service-config");
    let output_dir = unique_temp_dir("service-output");
    let mut app_config = CameraConnectorConfig::default();
    app_config
        .set_account("z5", Some("secret"), "Z5_2")
        .expect("account should save");
    app_config
        .save(Some(&config_path))
        .expect("config should save");

    let service = CameraConnectorService::new(Some(config_path.clone()));
    let receiver = service
        .receiver_config(ReceiverConfigRequest {
            protocol: PushProtocol::Ftp,
            bind_host: "0.0.0.0".to_string(),
            port: 2121,
            output_dir: output_dir.clone(),
            state_dir: None,
            username: None,
            password: None,
            advertised_host: Some("192.168.137.1".to_string()),
            source_name: None,
        })
        .expect("receiver config should build");

    assert_eq!(receiver.accounts.len(), 1);
    assert_eq!(receiver.accounts[0].username, "z5");
    assert_eq!(receiver.accounts[0].device_name, "Z5_2");
    assert_eq!(receiver.advertised_host.as_deref(), Some("192.168.137.1"));
    assert_eq!(
        receiver.state_dir,
        config_path.parent().unwrap().join("state")
    );

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn service_returns_transfer_views_with_virtual_display_paths() {
    let output_dir = unique_temp_dir("service-transfers");
    std::fs::create_dir_all(&output_dir).expect("output dir should create");
    append_transfer_record(
        &output_dir,
        &TransferRecord {
            transfer_id: "ftp:1".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/100NIKON/DSC_0001.NEF".to_string(),
            final_filename: "DSC_0001.NEF".to_string(),
            final_path: Some(output_dir.join("DSC_0001.NEF")),
            final_location: Some(StoredObjectLocation::local_path(
                output_dir.join("DSC_0001.NEF"),
            )),
            size_bytes: 42,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 10,
            completed_at_ms: Some(20),
            error: None,
        },
    )
    .expect("transfer record should append");

    let service = CameraConnectorService::new(None);
    let views = service
        .transfers(&output_dir, TransferQuery::default())
        .expect("transfers should load");

    assert_eq!(views.len(), 1);
    assert_eq!(views[0].display_source.as_deref(), Some("Z5_2"));
    assert_eq!(
        views[0].virtual_display_path,
        "Z5_2/DCIM/100NIKON/DSC_0001.NEF"
    );
    assert_eq!(views[0].final_location_kind.as_deref(), Some("local_path"));
    assert!(views[0]
        .final_location_label
        .as_deref()
        .unwrap_or_default()
        .ends_with("DSC_0001.NEF"));

    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn service_maps_connected_devices_to_account_device_names() {
    let config_path = unique_temp_path("service-device-config");
    let output_dir = unique_temp_dir("service-devices");
    let mut app_config = CameraConnectorConfig::default();
    app_config
        .set_account("z5", Some("secret"), "Z5_2")
        .expect("account should save");
    app_config
        .save(Some(&config_path))
        .expect("config should save");
    record_device_connected(&output_dir, "192.168.137.56", Some(50123), None, None)
        .expect("device should connect");
    record_device_authenticated(&output_dir, "192.168.137.56", Some("Z5_2"), Some("z5"))
        .expect("device should authenticate");

    let service = CameraConnectorService::new(Some(config_path.clone()));
    let devices = service
        .connected_devices(&output_dir, None, false)
        .expect("devices should load");

    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].display_source, "Z5_2");
    assert_eq!(devices[0].device.username.as_deref(), Some("z5"));

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn service_scans_inbox_groups() {
    let output_dir = unique_temp_dir("service-inbox");
    std::fs::create_dir_all(&output_dir).expect("output dir should create");
    std::fs::write(output_dir.join("IMG_0001.JPG"), [1, 2, 3]).expect("jpg should write");
    std::fs::write(output_dir.join("IMG_0001.CR3"), [4, 5, 6]).expect("raw should write");

    let service = CameraConnectorService::new(None);
    let groups = service
        .inbox_groups(&output_dir, ImportSource::FtpPush)
        .expect("groups should load");

    assert_eq!(groups.len(), 1);
    assert!(groups[0].jpeg.is_some());
    assert!(groups[0].raw.is_some());
    assert_eq!(
        groups[0]
            .jpeg
            .as_ref()
            .and_then(|asset| asset.storage_location.as_ref())
            .map(StoredObjectLocation::kind),
        Some("local_path")
    );

    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn service_groups_received_assets_from_transfer_log_without_scanning_storage() {
    let state_dir = unique_temp_dir("service-transfer-groups");
    std::fs::create_dir_all(&state_dir).expect("state dir should create");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:jpg".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/100/IMG_2222.JPG".to_string(),
            final_filename: "IMG_2222.JPG".to_string(),
            final_path: None,
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_2222.JPG",
            )),
            size_bytes: 100,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 10,
            completed_at_ms: Some(20),
            error: None,
        },
    )
    .expect("jpg transfer should append");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "sftp:raw".to_string(),
            protocol: "sftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/100/IMG_2222.NEF".to_string(),
            final_filename: "IMG_2222.NEF".to_string(),
            final_path: None,
            final_location: Some(StoredObjectLocation::media_uri(
                "content://media/external/images/media/2222",
            )),
            size_bytes: 200,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 11,
            completed_at_ms: Some(21),
            error: None,
        },
    )
    .expect("raw transfer should append");

    let service = CameraConnectorService::new(None);
    let groups = service
        .transfer_asset_groups(&state_dir)
        .expect("transfer groups should load");

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_key, "IMG_2222");
    assert_eq!(
        groups[0].jpeg.as_ref().map(|asset| asset.filename.as_str()),
        Some("IMG_2222.JPG")
    );
    let jpeg = groups[0].jpeg.as_ref().expect("jpeg should exist");
    assert_eq!(jpeg.original_path.as_deref(), Some("DCIM/100/IMG_2222.JPG"));
    assert_eq!(jpeg.display_source.as_deref(), Some("Z5_2"));
    assert_eq!(jpeg.remote_addr.as_deref(), Some("192.168.137.56"));
    assert_eq!(
        jpeg.virtual_display_path.as_deref(),
        Some("Z5_2/DCIM/100/IMG_2222.JPG")
    );
    assert_eq!(
        groups[0].raw.as_ref().map(|asset| asset.filename.as_str()),
        Some("IMG_2222.NEF")
    );
    assert_eq!(
        groups[0]
            .jpeg
            .as_ref()
            .and_then(|asset| asset.storage_location.as_ref())
            .map(StoredObjectLocation::kind),
        Some("document_uri")
    );
    assert_eq!(
        groups[0]
            .raw
            .as_ref()
            .and_then(|asset| asset.storage_location.as_ref())
            .map(StoredObjectLocation::display_label)
            .as_deref(),
        Some("content://media/external/images/media/2222")
    );
    assert_eq!(groups[0].primary.source, ImportSource::FtpPush);

    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_filters_transfer_asset_groups_by_metadata_and_format() {
    let state_dir = unique_temp_dir("service-transfer-group-filter");
    std::fs::create_dir_all(&state_dir).expect("state dir should create");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:jpg".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/100/IMG_3000.JPG".to_string(),
            final_filename: "IMG_3000.JPG".to_string(),
            final_path: None,
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_3000.JPG",
            )),
            size_bytes: 100,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 10,
            completed_at_ms: Some(20),
            error: None,
        },
    )
    .expect("first transfer should append");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:raw".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/100/IMG_3000.NEF".to_string(),
            final_filename: "IMG_3000.NEF".to_string(),
            final_path: None,
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_3000.NEF",
            )),
            size_bytes: 200,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 11,
            completed_at_ms: Some(21),
            error: None,
        },
    )
    .expect("second transfer should append");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:other".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/101/IMG_4000.JPG".to_string(),
            final_filename: "IMG_4000.JPG".to_string(),
            final_path: None,
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_4000.JPG",
            )),
            size_bytes: 150,
            remote_addr: Some("192.168.137.44".to_string()),
            source_name: Some("X-T5".to_string()),
            started_at_ms: 12,
            completed_at_ms: Some(22),
            error: None,
        },
    )
    .expect("third transfer should append");

    let service = CameraConnectorService::new(None);
    let groups = service
        .transfer_asset_groups_with_query(
            &state_dir,
            AssetGroupQuery {
                source_name: Some("Z5_2".to_string()),
                original_path: Some("dcim/100".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                format: Some(ObjectFormat::Nef),
            },
        )
        .expect("filtered groups should load");

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_key, "IMG_3000");
    assert!(groups[0].jpeg.is_some());
    assert!(groups[0].raw.is_some());

    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_summarizes_log_backed_asset_groups_for_filter_tabs() {
    let state_dir = unique_temp_dir("service-transfer-group-summary");
    std::fs::create_dir_all(&state_dir).expect("state dir should create");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:jpg".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/100/IMG_5000.JPG".to_string(),
            final_filename: "IMG_5000.JPG".to_string(),
            final_path: None,
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_5000.JPG",
            )),
            size_bytes: 100,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 10,
            completed_at_ms: Some(20),
            error: None,
        },
    )
    .expect("jpg transfer should append");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:raw".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/100/IMG_5000.NEF".to_string(),
            final_filename: "IMG_5000.NEF".to_string(),
            final_path: None,
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_5000.NEF",
            )),
            size_bytes: 200,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 11,
            completed_at_ms: Some(21),
            error: None,
        },
    )
    .expect("raw transfer should append");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "sftp:video".to_string(),
            protocol: "sftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "PRIVATE/CLIP_1.MOV".to_string(),
            final_filename: "CLIP_1.MOV".to_string(),
            final_path: None,
            final_location: Some(StoredObjectLocation::media_uri(
                "content://media/external/video/media/1",
            )),
            size_bytes: 300,
            remote_addr: Some("192.168.137.44".to_string()),
            source_name: Some("X-T5".to_string()),
            started_at_ms: 12,
            completed_at_ms: Some(22),
            error: None,
        },
    )
    .expect("video transfer should append");

    let service = CameraConnectorService::new(None);
    let summary = service
        .transfer_asset_summary_with_query(&state_dir, AssetGroupQuery::default())
        .expect("summary should load");

    assert_eq!(summary.group_count, 2);
    assert_eq!(summary.asset_count, 3);
    assert_eq!(summary.groups_with_jpeg, 1);
    assert_eq!(summary.groups_with_raw, 1);
    assert_eq!(summary.groups_with_video, 1);
    assert_eq!(summary.source_counts[0].value, "X-T5");
    assert_eq!(summary.source_counts[0].group_count, 1);
    assert_eq!(summary.source_counts[1].value, "Z5_2");
    assert_eq!(summary.source_counts[1].group_count, 1);
    assert_eq!(summary.remote_addr_counts.len(), 2);

    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_orders_log_backed_asset_groups_by_latest_completed_time() {
    let state_dir = unique_temp_dir("service-transfer-group-order");
    std::fs::create_dir_all(&state_dir).expect("state dir should create");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:old".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "IMG_1000.CR3".to_string(),
            final_filename: "IMG_1000.CR3".to_string(),
            final_path: None,
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_1000.CR3",
            )),
            size_bytes: 10,
            remote_addr: None,
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 100,
            completed_at_ms: Some(100),
            error: None,
        },
    )
    .expect("old transfer should append");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:new".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "IMG_1001.CR3".to_string(),
            final_filename: "IMG_1001.CR3".to_string(),
            final_path: None,
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_1001.CR3",
            )),
            size_bytes: 10,
            remote_addr: None,
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 200,
            completed_at_ms: Some(200),
            error: None,
        },
    )
    .expect("new transfer should append");

    let service = CameraConnectorService::new(None);
    let groups = service
        .transfer_asset_groups(&state_dir)
        .expect("groups should load");

    assert_eq!(groups[0].group_key, "IMG_1001");
    assert_eq!(groups[1].group_key, "IMG_1000");

    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_paginates_log_backed_asset_groups_after_filtering_and_sorting() {
    let state_dir = unique_temp_dir("service-transfer-group-page");
    std::fs::create_dir_all(&state_dir).expect("state dir should create");
    for index in 0..3 {
        append_transfer_record(
            &state_dir,
            &TransferRecord {
                transfer_id: format!("ftp:{index}"),
                protocol: "ftp".to_string(),
                status: TransferStatus::Completed,
                original_path: format!("IMG_200{index}.CR3"),
                final_filename: format!("IMG_200{index}.CR3"),
                final_path: None,
                final_location: Some(StoredObjectLocation::document_uri(format!(
                    "content://camera-connector/IMG_200{index}.CR3"
                ))),
                size_bytes: 10,
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Z5_2".to_string()),
                started_at_ms: 100 + index,
                completed_at_ms: Some(100 + index),
                error: None,
            },
        )
        .expect("transfer should append");
    }

    let service = CameraConnectorService::new(None);
    let page = service
        .transfer_asset_group_page_with_query(
            &state_dir,
            AssetGroupQuery {
                source_name: Some("Z5_2".to_string()),
                ..AssetGroupQuery::default()
            },
            1,
            1,
        )
        .expect("page should load");

    assert_eq!(page.offset, 1);
    assert_eq!(page.limit, 1);
    assert_eq!(page.total_groups, 3);
    assert!(page.has_more);
    assert_eq!(page.summary.group_count, 3);
    assert_eq!(page.groups.len(), 1);
    assert_eq!(page.groups[0].group_key, "IMG_2001");

    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_reads_receiver_runtime_status() {
    let output_dir = unique_temp_dir("service-runtime-status");
    std::fs::create_dir_all(&output_dir).expect("output dir should create");
    write_receiver_runtime_status(
        &output_dir,
        &ReceiverRuntimeStatus {
            phase: ReceiverRuntimePhase::Stopped,
            protocol: Some(PushProtocol::Ftp),
            auth_mode: ReceiverAuthMode::Accounts,
            local_addr: None,
            output_dir: Some(output_dir.clone()),
            state_dir: Some(output_dir.clone()),
            account_count: 2,
            message: Some("operator stopped receiver".to_string()),
        },
    )
    .expect("runtime status should write");

    let service = CameraConnectorService::new(None);
    let status = service
        .receiver_status(&output_dir)
        .expect("runtime status should load")
        .expect("runtime status should exist");

    assert_eq!(status.phase, ReceiverRuntimePhase::Stopped);
    assert_eq!(status.auth_mode, ReceiverAuthMode::Accounts);
    assert_eq!(status.account_count, 2);

    let _ = std::fs::remove_dir_all(output_dir);
}

fn unique_temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camera-connector-{name}-{}.json", unique_suffix()))
}

fn unique_temp_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camera-connector-{name}-{}", unique_suffix()))
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

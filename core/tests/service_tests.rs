use camera_connector_core::{
    append_transfer_record, record_device_authenticated, record_device_connected,
    write_receiver_runtime_status, CameraConnectorConfig, CameraConnectorService, ImportSource,
    PushProtocol, ReceiverAuthMode, ReceiverConfigRequest, ReceiverRuntimePhase,
    ReceiverRuntimeStatus, StoredObjectLocation, TransferQuery, TransferRecord, TransferStatus,
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

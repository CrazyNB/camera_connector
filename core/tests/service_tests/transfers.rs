use super::*;

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
            final_location: Some(StoredObjectLocation::local_path(
                output_dir.join("DSC_0001.NEF"),
            )),
            size_bytes: 42,
            username: None,
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
        .diagnostic_transfers(&output_dir, TransferQuery::default())
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
fn service_summarizes_transfer_statuses() {
    let state_dir = unique_temp_dir("service-transfer-summary");
    std::fs::create_dir_all(&state_dir).expect("state dir should create");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:ok".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "IMG_0001.CR3".to_string(),
            final_filename: "IMG_0001.CR3".to_string(),
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_0001.CR3",
            )),
            size_bytes: 42,
            username: Some("z5".to_string()),
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 10,
            completed_at_ms: Some(20),
            error: None,
        },
    )
    .expect("completed transfer should append");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:failed".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Failed,
            original_path: "IMG_0002.CR3".to_string(),
            final_filename: "IMG_0002.CR3".to_string(),
            final_location: None,
            size_bytes: 0,
            username: Some("z5".to_string()),
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 11,
            completed_at_ms: Some(21),
            error: Some("connection reset".to_string()),
        },
    )
    .expect("failed transfer should append");

    let service = CameraConnectorService::new(None);
    let transfers = service
        .diagnostic_transfers(
            &state_dir,
            TransferQuery {
                username: Some("z5".to_string()),
                ..TransferQuery::default()
            },
        )
        .expect("transfers should load");

    assert_eq!(transfers.len(), 2);
    assert_eq!(
        transfers
            .iter()
            .filter(|view| view.record.status == TransferStatus::Completed)
            .count(),
        1
    );
    assert_eq!(
        transfers
            .iter()
            .filter(|view| view.record.status == TransferStatus::Failed)
            .count(),
        1
    );

    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_filters_failed_transfer_views_by_status() {
    let state_dir = unique_temp_dir("service-transfer-status-filter");
    std::fs::create_dir_all(&state_dir).expect("state dir should create");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:ok".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "IMG_0001.CR3".to_string(),
            final_filename: "IMG_0001.CR3".to_string(),
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_0001.CR3",
            )),
            size_bytes: 42,
            username: Some("z5".to_string()),
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 10,
            completed_at_ms: Some(20),
            error: None,
        },
    )
    .expect("completed transfer should append");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:failed".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Failed,
            original_path: "IMG_0002.CR3".to_string(),
            final_filename: "IMG_0002.CR3".to_string(),
            final_location: None,
            size_bytes: 0,
            username: Some("z5".to_string()),
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 11,
            completed_at_ms: Some(21),
            error: Some("connection reset".to_string()),
        },
    )
    .expect("failed transfer should append");

    let service = CameraConnectorService::new(None);
    let transfers = service
        .diagnostic_transfers(
            &state_dir,
            TransferQuery {
                status: Some(TransferStatus::Failed),
                username: Some("z5".to_string()),
                ..TransferQuery::default()
            },
        )
        .expect("failed transfers should load");

    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].record.status, TransferStatus::Failed);
    assert_eq!(
        transfers[0].record.error.as_deref(),
        Some("connection reset")
    );

    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_resolves_transfer_display_source_from_current_account_name() {
    let config_path = unique_temp_path("service-transfer-account-config");
    let state_dir = unique_temp_dir("service-transfer-account-state");
    std::fs::create_dir_all(&state_dir).expect("state dir should create");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            state_dir: Some(state_dir.clone()),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");
    service
        .set_account("z5", Some("secret"), "Renamed Z5")
        .expect("account should save");
    append_transfer_record(
        &state_dir,
        &TransferRecord {
            transfer_id: "ftp:account".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/100NIKON/DSC_0002.NEF".to_string(),
            final_filename: "DSC_0002.NEF".to_string(),
            final_location: Some(StoredObjectLocation::local_path(
                state_dir.join("DSC_0002.NEF"),
            )),
            size_bytes: 64,
            remote_addr: Some("192.168.137.56".to_string()),
            username: Some("z5".to_string()),
            source_name: Some("Old Z5 Name".to_string()),
            started_at_ms: 10,
            completed_at_ms: Some(20),
            error: None,
        },
    )
    .expect("transfer record should append");

    let transfers = service
        .diagnostic_transfers(
            &state_dir,
            TransferQuery {
                username: Some("z5".to_string()),
                ..TransferQuery::default()
            },
        )
        .expect("transfers should load");
    let groups = service
        .diagnostic_transfer_asset_groups_with_query(
            &state_dir,
            AssetGroupQuery {
                username: Some("z5".to_string()),
                ..AssetGroupQuery::default()
            },
        )
        .expect("groups should load");

    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].display_source.as_deref(), Some("Renamed Z5"));
    assert_eq!(
        transfers[0].virtual_display_path,
        "Renamed Z5/DCIM/100NIKON/DSC_0002.NEF"
    );
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].primary.username.as_deref(), Some("z5"));
    assert_eq!(
        groups[0].primary.virtual_display_path.as_deref(),
        Some("Renamed Z5/DCIM/100NIKON/DSC_0002.NEF")
    );

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

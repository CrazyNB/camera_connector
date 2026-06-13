use camera_connector_core::{
    append_transfer_record, record_device_authenticated, record_device_connected,
    write_receiver_runtime_status, AssetGroupQuery, CameraConnectorConfig, CameraConnectorService,
    CvPolicy, GuestMark, ImportSource, ModelProviderKind, ModelProviderSettings, ModelSendMode,
    ObjectFormat, PreviewSample, ProjectRecommendationMode, PushProtocol, ReceiverAuthMode,
    ReceiverConfigRequest, ReceiverRuntimePhase, ReceiverRuntimeStatus, ReceiverSettingsUpdate,
    SceneProfile, StoredObjectLocation, TechnicalAssessmentPolicy, TechnicalDefectType,
    TechnicalGateStatus, TransferQuery, TransferRecord, TransferStatus,
};

#[test]
fn guest_mark_accepts_only_share_selection_values() {
    assert_eq!(
        GuestMark::from_wire("favorite").unwrap(),
        GuestMark::Favorite
    );
    assert_eq!(GuestMark::from_wire("marked").unwrap(), GuestMark::Marked);
    assert_eq!(GuestMark::from_wire("reject").unwrap(), GuestMark::Reject);
    assert!(GuestMark::from_wire("delete").is_none());
    assert!(GuestMark::from_wire("").is_none());
}

#[test]
fn service_lan_share_asset_page_uses_saved_query() {
    let (service, config_path, state_dir) = service_with_state_dir("service-lan-share-query");
    let project = service
        .create_project("LAN Query")
        .expect("project should create");
    let keep = record_service_jpeg_group(&service, &project.project_id, "KEEP_0001.JPG", 10);
    let skip = record_service_jpeg_group(&service, &project.project_id, "SKIP_0001.JPG", 20);
    service
        .set_asset_group_user_marks(&project.project_id, &keep, Some(true), None)
        .expect("favorite should save");

    let session = service
        .create_lan_share_session(
            &project.project_id,
            AssetGroupQuery {
                favorite: Some(true),
                ..AssetGroupQuery::default()
            },
            Some("Favorites".to_string()),
        )
        .expect("share session should create");

    let page = service
        .lan_share_asset_group_page(&session.token, 0, 25)
        .expect("share page should load");

    assert_eq!(page.total_groups, 1);
    assert_eq!(page.groups[0].group_id.as_deref(), Some(keep.as_str()));
    assert_ne!(page.groups[0].group_id.as_deref(), Some(skip.as_str()));

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_guest_reject_mark_does_not_delete_or_mutate_user_marks() {
    let (service, config_path, state_dir) = service_with_state_dir("service-lan-share-reject");
    let project = service
        .create_project("LAN Reject")
        .expect("project should create");
    let group_id = record_service_jpeg_group(&service, &project.project_id, "IMG_4040.JPG", 10);
    let session = service
        .create_lan_share_session(&project.project_id, AssetGroupQuery::default(), None)
        .expect("share session should create");

    service
        .set_lan_share_guest_mark(&session.token, &group_id, Some(GuestMark::Reject))
        .expect("guest mark should save");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("page should load");
    assert_eq!(page.total_groups, 1);
    assert_eq!(page.groups[0].guest_mark, Some(GuestMark::Reject));
    assert!(!page.groups[0].user_marks.favorite);
    assert!(!page.groups[0].user_marks.marked);

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_builds_receiver_config_from_saved_accounts() {
    let config_path = unique_temp_path("service-config");
    let output_dir = unique_temp_dir("service-output");
    let mut app_config = CameraConnectorConfig::default();
    let configured_state_dir = unique_temp_dir("service-config-state");
    app_config.receiver.protocol = PushProtocol::Sftp;
    app_config.receiver.bind_host = "127.0.0.1".to_string();
    app_config.receiver.sftp_port = 2223;
    app_config.receiver.output_dir = Some(output_dir.clone());
    app_config.receiver.state_dir = Some(configured_state_dir.clone());
    app_config.receiver.advertised_host = Some("192.168.137.1".to_string());
    app_config
        .save(Some(&config_path))
        .expect("config should save");

    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_account("z5", Some("secret"), "Z5_2")
        .expect("account should save");
    let receiver = service
        .receiver_config(ReceiverConfigRequest {
            protocol: None,
            bind_host: None,
            port: None,
            output_dir: None,
            state_dir: None,
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
            defer_publish: None,
        })
        .expect("receiver config should build");

    assert_eq!(receiver.protocol, PushProtocol::Sftp);
    assert_eq!(receiver.bind_host, "127.0.0.1");
    assert_eq!(receiver.port, 2223);
    assert_eq!(receiver.output_dir, output_dir);
    assert_eq!(receiver.accounts.len(), 1);
    assert_eq!(receiver.accounts[0].username, "z5");
    assert_eq!(receiver.accounts[0].device_name, "Z5_2");
    assert_eq!(receiver.advertised_host.as_deref(), Some("192.168.137.1"));
    assert_eq!(receiver.state_dir, configured_state_dir);

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(output_dir);
    let _ = std::fs::remove_dir_all(configured_state_dir);
}

#[test]
fn service_updates_receiver_settings() {
    let config_path = unique_temp_path("service-receiver-settings");
    let output_dir = unique_temp_dir("service-receiver-settings-output");
    let state_dir = unique_temp_dir("service-receiver-settings-state");
    let service = CameraConnectorService::new(Some(config_path.clone()));

    let (settings, saved_path) = service
        .set_receiver_settings(ReceiverSettingsUpdate {
            protocol: Some(PushProtocol::Sftp),
            bind_host: Some("127.0.0.1".to_string()),
            ftp_port: Some(2122),
            sftp_port: Some(2223),
            output_dir: Some(output_dir.clone()),
            state_dir: Some(state_dir.clone()),
            advertised_host: Some("192.168.137.1".to_string()),
            source_name: Some("Studio".to_string()),
            defer_publish: Some(true),
        })
        .expect("receiver settings should save");

    assert_eq!(saved_path, config_path);
    assert_eq!(settings.protocol, PushProtocol::Sftp);
    assert_eq!(settings.bind_host, "127.0.0.1");
    assert_eq!(settings.ftp_port, 2122);
    assert_eq!(settings.sftp_port, 2223);
    assert!(settings.defer_publish);

    let loaded = CameraConnectorConfig::load(Some(&config_path)).expect("config should load");
    assert_eq!(
        loaded.receiver.output_dir.as_deref(),
        Some(output_dir.as_path())
    );
    assert_eq!(
        loaded.receiver.state_dir.as_deref(),
        Some(state_dir.as_path())
    );
    assert_eq!(
        loaded.receiver.advertised_host.as_deref(),
        Some("192.168.137.1")
    );
    assert_eq!(loaded.receiver.source_name.as_deref(), Some("Studio"));
    assert!(loaded.receiver.defer_publish);

    let _ = std::fs::remove_file(config_path);
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
    let summary = service
        .diagnostic_transfer_summary_with_query(
            &state_dir,
            TransferQuery {
                username: Some("z5".to_string()),
                ..TransferQuery::default()
            },
        )
        .expect("transfer summary should load");

    assert_eq!(summary.total_count, 2);
    assert_eq!(summary.completed_count, 1);
    assert_eq!(summary.failed_count, 1);

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

#[test]
fn service_maps_connected_devices_to_account_device_names() {
    let config_path = unique_temp_path("service-device-config");
    let output_dir = unique_temp_dir("service-devices");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            state_dir: Some(output_dir.clone()),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");
    service
        .set_account("z5", Some("secret"), "Account Z5")
        .expect("account should save");
    record_device_connected(&output_dir, "192.168.137.56", Some(50123), None, None)
        .expect("device should connect");
    record_device_authenticated(&output_dir, "192.168.137.56", Some("Old Z5"), Some("z5"))
        .expect("device should authenticate");

    let devices = service
        .connected_devices(&output_dir, None, false)
        .expect("devices should load");

    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].display_source, "Account Z5");
    assert_eq!(devices[0].device.username.as_deref(), Some("z5"));

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn service_scans_received_asset_groups() {
    let output_dir = unique_temp_dir("service-output");
    std::fs::create_dir_all(&output_dir).expect("output dir should create");
    std::fs::write(output_dir.join("IMG_0001.JPG"), [1, 2, 3]).expect("jpg should write");
    std::fs::write(output_dir.join("IMG_0001.CR3"), [4, 5, 6]).expect("raw should write");

    let service = CameraConnectorService::new(None);
    let groups = service
        .diagnostic_received_asset_groups(&output_dir, ImportSource::FtpPush)
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
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_2222.JPG",
            )),
            size_bytes: 100,
            username: None,
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
            final_location: Some(StoredObjectLocation::media_uri(
                "content://media/external/images/media/2222",
            )),
            size_bytes: 200,
            username: None,
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
            transfer_id: "ftp:txt".to_string(),
            protocol: "ftp".to_string(),
            status: TransferStatus::Completed,
            original_path: "DCIM/100/README.TXT".to_string(),
            final_filename: "README.TXT".to_string(),
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/README.TXT",
            )),
            size_bytes: 10,
            username: None,
            remote_addr: Some("192.168.137.56".to_string()),
            source_name: Some("Z5_2".to_string()),
            started_at_ms: 12,
            completed_at_ms: Some(22),
            error: None,
        },
    )
    .expect("non-media transfer should append");

    let service = CameraConnectorService::new(None);
    let groups = service
        .diagnostic_transfer_asset_groups(&state_dir)
        .expect("transfer groups should load");

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].group_key, "IMG_2222");
    assert_ne!(groups[0].primary.filename, "README.TXT");
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
fn service_marks_duplicate_assets_from_same_account_and_original_path() {
    let state_dir = unique_temp_dir("service-transfer-duplicates");
    std::fs::create_dir_all(&state_dir).expect("state dir should create");
    for (index, final_filename) in ["IMG_7777.CR3", "IMG_7777 (1).CR3"].iter().enumerate() {
        append_transfer_record(
            &state_dir,
            &TransferRecord {
                transfer_id: format!("ftp:duplicate:{index}"),
                protocol: "ftp".to_string(),
                status: TransferStatus::Completed,
                original_path: "DCIM/100CANON/IMG_7777.CR3".to_string(),
                final_filename: final_filename.to_string(),
                final_location: Some(StoredObjectLocation::document_uri(format!(
                    "content://camera-connector/{final_filename}"
                ))),
                size_bytes: 100,
                username: Some("canon".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("R5".to_string()),
                started_at_ms: 10 + index as i64,
                completed_at_ms: Some(20 + index as i64),
                error: None,
            },
        )
        .expect("transfer should append");
    }

    let service = CameraConnectorService::new(None);
    let groups = service
        .diagnostic_transfer_asset_groups_with_query(
            &state_dir,
            AssetGroupQuery {
                username: Some("canon".to_string()),
                ..AssetGroupQuery::default()
            },
        )
        .expect("groups should load");

    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].primary.filename, "IMG_7777 (1).CR3");
    assert_eq!(groups[0].primary.duplicate_index, Some(2));
    assert_eq!(groups[0].primary.duplicate_count, Some(2));
    assert_eq!(groups[1].primary.filename, "IMG_7777.CR3");
    assert_eq!(groups[1].primary.duplicate_index, Some(1));
    assert_eq!(groups[1].primary.duplicate_count, Some(2));

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
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_3000.JPG",
            )),
            size_bytes: 100,
            username: None,
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
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_3000.NEF",
            )),
            size_bytes: 200,
            username: None,
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
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_4000.JPG",
            )),
            size_bytes: 150,
            username: None,
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
        .diagnostic_transfer_asset_groups_with_query(
            &state_dir,
            AssetGroupQuery {
                source_name: Some("Z5_2".to_string()),
                original_path: Some("dcim/100".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                format: Some(ObjectFormat::Nef),
                ..AssetGroupQuery::default()
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
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_5000.JPG",
            )),
            size_bytes: 100,
            username: None,
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
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_5000.NEF",
            )),
            size_bytes: 200,
            username: None,
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
            final_location: Some(StoredObjectLocation::media_uri(
                "content://media/external/video/media/1",
            )),
            size_bytes: 300,
            username: None,
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
        .diagnostic_transfer_asset_summary_with_query(&state_dir, AssetGroupQuery::default())
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
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_1000.CR3",
            )),
            size_bytes: 10,
            username: None,
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
            final_location: Some(StoredObjectLocation::document_uri(
                "content://camera-connector/IMG_1001.CR3",
            )),
            size_bytes: 10,
            username: None,
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
        .diagnostic_transfer_asset_groups(&state_dir)
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
                final_location: Some(StoredObjectLocation::document_uri(format!(
                    "content://camera-connector/IMG_200{index}.CR3"
                ))),
                size_bytes: 10,
                username: None,
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
        .diagnostic_transfer_asset_group_page_with_query(
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

#[test]
fn service_rejects_invalid_prompt_id_when_prompt_is_selected() {
    let (service, config_path, state_dir) = service_with_state_dir("service-settings-invalid");
    let project = service
        .create_project("Invalid Prompt Settings")
        .expect("project should create");
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.prompt_pack_id = Some("missing-prompt".to_string());
    settings.updated_at_ms = 5_000;

    assert!(service.save_project_evaluation_settings(settings).is_err());

    let mut without_prompt = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should reload")
        .expect("settings should exist");
    without_prompt.prompt_pack_id = None;
    without_prompt.updated_at_ms = 5_100;
    let saved = service
        .save_project_evaluation_settings(without_prompt)
        .expect("prompt pack may be omitted");
    assert_eq!(saved.prompt_pack_id, None);

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_saves_provider_and_project_settings_with_manual_recommendation_mode() {
    let (service, config_path, state_dir) = service_with_state_dir("service-settings-manual");
    let project = service
        .create_project("Manual Settings")
        .expect("project should create");

    let provider = service
        .save_model_provider_settings(ModelProviderSettings {
            settings_id: "contains-no-secret-fields".to_string(),
            provider_kind: ModelProviderKind::OpenAi,
            provider_label: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-5.1-mini".to_string(),
            default_max_image_side: 1600,
            default_send_mode: ModelSendMode::PreviewOnly,
            default_batch_size: 3,
            configured: true,
            api_key_configured: false,
            key_alias: None,
            updated_at_ms: 6_000,
        })
        .expect("provider settings should save");
    assert_eq!(provider.settings_id, "contains-no-secret-fields");
    assert_eq!(
        service
            .model_provider_settings()
            .expect("provider should load")
            .expect("provider should exist"),
        provider
    );

    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.project_recommendation_mode = ProjectRecommendationMode::Manual;
    settings.scene_profile = SceneProfile::Portrait;
    settings.cv_policy = CvPolicy::Strict;
    settings.updated_at_ms = 6_100;
    let saved = service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");

    assert_eq!(
        saved.project_recommendation_mode,
        ProjectRecommendationMode::Manual
    );
    assert_eq!(saved.scene_profile, SceneProfile::Portrait);
    assert_eq!(saved.cv_policy, CvPolicy::Strict);

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[test]
fn service_project_evaluation_settings_can_override_technical_thresholds() {
    let (service, config_path, state_dir) =
        service_with_state_dir("service-cv-threshold-overrides");
    let project = service
        .create_project("CV Threshold Overrides")
        .expect("project should create");
    let mut settings = service
        .project_evaluation_settings(&project.project_id)
        .expect("settings should load")
        .expect("settings should exist");
    settings.cv_policy = CvPolicy::Standard;
    let mut policy = TechnicalAssessmentPolicy::standard();
    policy.clipping_high_ratio = 0.08;
    policy.clipping_high_connected_ratio = 0.08;
    settings.cv_policy_overrides = Some(policy);
    service
        .save_project_evaluation_settings(settings)
        .expect("settings should save");
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer(
                "ftp:threshold-overrides",
                "DCIM/100/IMG_THRESHOLD.JPG",
                1_000,
            ),
        )
        .expect("transfer should record");
    service
        .drain_analysis_jobs(10)
        .expect("burst analysis should drain");
    let page = service
        .project_asset_group_page_with_query(&project.project_id, Default::default(), 0, 25)
        .expect("page should load");
    let group_id = page.groups[0]
        .group_id
        .as_ref()
        .expect("group id should exist")
        .clone();

    let assessment = service
        .assess_asset_group_preview(
            &group_id,
            scattered_highlight_sample(100, 100, 900),
            "technical-v1",
        )
        .expect("assessment should save");

    assert_eq!(assessment.gate_status, TechnicalGateStatus::Warn);
    assert!(assessment
        .defect_flags
        .iter()
        .any(|flag| flag.defect_type == TechnicalDefectType::HighlightClip));

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(state_dir);
}

fn unique_temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camera-connector-{name}-{}.json", unique_suffix()))
}

fn unique_temp_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camera-connector-{name}-{}", unique_suffix()))
}

fn service_with_state_dir(
    name: &str,
) -> (
    CameraConnectorService,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let config_path = unique_temp_path(name);
    let state_dir = unique_temp_dir(name);
    let mut app_config = CameraConnectorConfig::default();
    app_config.receiver.state_dir = Some(state_dir.clone());
    app_config
        .save(Some(&config_path))
        .expect("config should save");
    (
        CameraConnectorService::new(Some(config_path.clone())),
        config_path,
        state_dir,
    )
}

fn scattered_highlight_sample(width: usize, height: usize, clipped_count: usize) -> PreviewSample {
    let mut luma = vec![128; width * height];
    for index in 0..clipped_count.min(luma.len()) {
        let pixel_index = (index * 37) % luma.len();
        luma[pixel_index] = 248;
    }
    PreviewSample {
        width,
        height,
        luma,
        red: None,
        green: None,
        blue: None,
        preview_source: Some("test".to_string()),
    }
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

fn record_service_jpeg_group(
    service: &CameraConnectorService,
    project_id: &str,
    filename: &str,
    completed_at_ms: i64,
) -> String {
    service
        .record_project_transfer(
            project_id,
            completed_transfer(
                &format!("ftp:{filename}"),
                &format!("DCIM/100/{filename}"),
                completed_at_ms,
            ),
        )
        .expect("transfer should record");
    service
        .project_asset_group_page_with_query(project_id, AssetGroupQuery::default(), 0, 100)
        .expect("page should load")
        .groups
        .into_iter()
        .find(|group| group.primary.filename == filename)
        .and_then(|group| group.group_id)
        .expect("group should exist")
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

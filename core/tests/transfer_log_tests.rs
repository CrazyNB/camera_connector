use camera_connector_core::{
    append_transfer_record, read_transfer_log, StoredObjectLocation, TransferRecord, TransferStatus,
};

#[test]
fn appends_and_reads_transfer_log_records() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let record = TransferRecord {
        transfer_id: "ftp:1".to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: "DCIM/100/IMG_0001.CR3".to_string(),
        final_filename: "IMG_0001.CR3".to_string(),
        final_path: Some(temp_dir.path().join("IMG_0001.CR3")),
        final_location: Some(StoredObjectLocation::local_path(
            temp_dir.path().join("IMG_0001.CR3"),
        )),
        size_bytes: 42,
        remote_addr: Some("192.168.137.56".to_string()),
        source_name: Some("Camera A".to_string()),
        started_at_ms: 10,
        completed_at_ms: Some(20),
        error: None,
    };

    append_transfer_record(temp_dir.path(), &record).expect("record should append");

    let records = read_transfer_log(temp_dir.path()).expect("records should read");
    assert_eq!(records, vec![record]);
}

#[test]
fn builds_virtual_display_path_from_source_and_original_path() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let record = TransferRecord {
        transfer_id: "ftp:1".to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: "BB/DSC_2552.NEF".to_string(),
        final_filename: "DSC_2552.NEF".to_string(),
        final_path: Some(temp_dir.path().join("DSC_2552.NEF")),
        final_location: Some(StoredObjectLocation::local_path(
            temp_dir.path().join("DSC_2552.NEF"),
        )),
        size_bytes: 42,
        remote_addr: Some("192.168.137.56".to_string()),
        source_name: Some("Z5_2".to_string()),
        started_at_ms: 10,
        completed_at_ms: Some(20),
        error: None,
    };

    assert_eq!(record.virtual_display_path(None), "Z5_2/BB/DSC_2552.NEF");
    assert_eq!(
        record.virtual_display_path(Some("Studio Camera")),
        "Studio Camera/BB/DSC_2552.NEF"
    );
}

#[test]
fn falls_back_to_last_ip_octet_for_virtual_display_path() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let record = TransferRecord {
        transfer_id: "ftp:1".to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: "BB/DSC_2552.NEF".to_string(),
        final_filename: "DSC_2552 (1).NEF".to_string(),
        final_path: Some(temp_dir.path().join("DSC_2552 (1).NEF")),
        final_location: Some(StoredObjectLocation::local_path(
            temp_dir.path().join("DSC_2552 (1).NEF"),
        )),
        size_bytes: 42,
        remote_addr: Some("192.168.137.56".to_string()),
        source_name: None,
        started_at_ms: 10,
        completed_at_ms: Some(20),
        error: None,
    };

    assert_eq!(
        record.virtual_display_path(None),
        "IP-056/BB/DSC_2552 (1).NEF"
    );
}

#[test]
fn transfer_log_can_record_non_path_storage_locations() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let record = TransferRecord {
        transfer_id: "sftp:1".to_string(),
        protocol: "sftp".to_string(),
        status: TransferStatus::Completed,
        original_path: "DCIM/101APPLE/IMG_0002.DNG".to_string(),
        final_filename: "IMG_0002.DNG".to_string(),
        final_path: None,
        final_location: Some(StoredObjectLocation::document_uri(
            "content://camera-connector/tree/inbox/IMG_0002.DNG",
        )),
        size_bytes: 84,
        remote_addr: Some("192.168.137.57".to_string()),
        source_name: Some("iPad Import".to_string()),
        started_at_ms: 30,
        completed_at_ms: Some(40),
        error: None,
    };

    append_transfer_record(temp_dir.path(), &record).expect("record should append");

    let records = read_transfer_log(temp_dir.path()).expect("records should read");
    assert_eq!(records, vec![record]);
}

#[test]
fn transfer_record_resolves_legacy_final_path_as_local_location() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let record = TransferRecord {
        transfer_id: "ftp:legacy".to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: "IMG_0003.NEF".to_string(),
        final_filename: "IMG_0003.NEF".to_string(),
        final_path: Some(temp_dir.path().join("IMG_0003.NEF")),
        final_location: None,
        size_bytes: 128,
        remote_addr: None,
        source_name: None,
        started_at_ms: 50,
        completed_at_ms: Some(60),
        error: None,
    };

    assert_eq!(
        record.resolved_final_location(),
        Some(StoredObjectLocation::local_path(
            temp_dir.path().join("IMG_0003.NEF")
        ))
    );
    assert_eq!(record.final_location_kind(), Some("local_path"));
}

#[test]
fn transfer_record_formats_platform_location_labels() {
    let record = TransferRecord {
        transfer_id: "sftp:ios".to_string(),
        protocol: "sftp".to_string(),
        status: TransferStatus::Completed,
        original_path: "IMG_0004.DNG".to_string(),
        final_filename: "IMG_0004.DNG".to_string(),
        final_path: None,
        final_location: Some(StoredObjectLocation::photo_asset("photos-local-id-1")),
        size_bytes: 256,
        remote_addr: None,
        source_name: None,
        started_at_ms: 70,
        completed_at_ms: Some(80),
        error: None,
    };

    assert_eq!(record.final_location_kind(), Some("photo_asset"));
    assert_eq!(
        record.final_location_label().as_deref(),
        Some("photos-local-id-1")
    );
}

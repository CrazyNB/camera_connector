use camera_connector_core::{
    append_transfer_record, read_transfer_log, TransferRecord, TransferStatus,
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
        final_path: temp_dir.path().join("IMG_0001.CR3"),
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

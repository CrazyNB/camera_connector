use camera_connector_core::{
    FtpPushServer, PushProtocol, PushReceiverConfig, PushReceiverServer, ReceiverAccount,
    SftpPushServer, SqliteStore, TransferRecord, TransferStatus,
};
use std::str::FromStr;

#[test]
fn push_protocol_rejects_ftps() {
    let result = PushProtocol::from_str("ftps");

    assert!(result.is_err());
}

#[test]
fn push_receiver_config_requires_active_project_for_storage_recording() {
    let output_dir = tempfile::tempdir().expect("output dir should create");
    let state_dir = tempfile::tempdir().expect("state dir should create");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, output_dir.path())
        .with_state_dir(state_dir.path());

    let result = config.record_storage_transfer(&TransferRecord {
        transfer_id: "ftp:fallback".to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: "IMG_0001.JPG".to_string(),
        final_filename: "IMG_0001.JPG".to_string(),
        final_location: None,
        size_bytes: 3,
        username: None,
        remote_addr: None,
        source_name: None,
        started_at_ms: 10,
        completed_at_ms: Some(20),
        error: None,
    });

    assert!(result.is_err());

    let store = SqliteStore::open_state_dir(state_dir.path()).expect("store should open");
    assert!(store
        .list_projects()
        .expect("projects should load")
        .is_empty());
}

#[tokio::test]
async fn ftp_server_rejects_blank_account_username() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path());
    config
        .accounts
        .push(ReceiverAccount::new(" ", Some("secret"), "Studio A"));

    let result = FtpPushServer::bind(config).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn ftp_server_rejects_blank_account_device_name() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path());
    config
        .accounts
        .push(ReceiverAccount::new("z5", Some("secret"), " "));

    let result = FtpPushServer::bind(config).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn push_receiver_server_binds_ftp_receiver() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path());

    let receiver = PushReceiverServer::bind(config)
        .await
        .expect("FTP receiver should bind through facade");

    assert_eq!(receiver.local_addr().ip().to_string(), "127.0.0.1");
}

#[tokio::test]
async fn push_receiver_server_binds_sftp_receiver() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Sftp, "127.0.0.1", 0, temp_dir.path());

    let receiver = PushReceiverServer::bind(config)
        .await
        .expect("SFTP receiver should bind through facade");

    assert_eq!(receiver.local_addr().ip().to_string(), "127.0.0.1");
}

#[tokio::test]
async fn sftp_server_binds_listener() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Sftp, "127.0.0.1", 0, temp_dir.path());

    let server = SftpPushServer::bind(config)
        .await
        .expect("SFTP receiver should bind listener");

    assert_eq!(server.local_addr().ip().to_string(), "127.0.0.1");
}

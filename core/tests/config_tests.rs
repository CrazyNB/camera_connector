use camera_connector_core::{FtpPushServer, PushProtocol, PushReceiverConfig, ReceiverAccount};
use std::str::FromStr;

#[test]
fn push_protocol_rejects_ftps() {
    let result = PushProtocol::from_str("ftps");

    assert!(result.is_err());
}

#[tokio::test]
async fn ftp_server_rejects_blank_account_username() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path())
        .with_account(ReceiverAccount::new(" ", Some("secret"), "Studio A"));

    let result = FtpPushServer::bind(config).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn ftp_server_rejects_blank_account_device_name() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path())
        .with_account(ReceiverAccount::new("z5", Some("secret"), " "));

    let result = FtpPushServer::bind(config).await;

    assert!(result.is_err());
}

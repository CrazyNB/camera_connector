use camera_connector_core::{FtpPushServer, PushProtocol, PushReceiverConfig, ReceiverAccount};

#[tokio::test]
async fn ftp_server_rejects_blank_account_username() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path())
        .with_account(ReceiverAccount::new(" ", Some("secret"), "Studio A"));

    let result = FtpPushServer::bind(config).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn ftp_server_rejects_invalid_account_ip() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path())
        .with_account(
            ReceiverAccount::new("z5", Some("secret"), "Studio A").with_remote_addr("not-an-ip"),
        );

    let result = FtpPushServer::bind(config).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn ftp_server_rejects_duplicate_account_ip_bindings() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path())
        .with_account(
            ReceiverAccount::new("z5", Some("secret"), "Studio A")
                .with_remote_addr("192.168.137.56"),
        )
        .with_account(
            ReceiverAccount::new("xt5", Some("secret"), "Studio B")
                .with_remote_addr("192.168.137.56"),
        );

    let result = FtpPushServer::bind(config).await;

    assert!(result.is_err());
}

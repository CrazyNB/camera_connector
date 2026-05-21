use std::sync::Arc;

use camera_connector_core::{
    read_transfer_log, PushProtocol, PushReceiverConfig, ReceiverAccount, SftpPushServer,
};
use russh::client;
use russh_sftp::{client::SftpSession, protocol::OpenFlags};
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;

struct AcceptAnyServerKey;

impl client::Handler for AcceptAnyServerKey {
    type Error = russh::Error;

    async fn check_server_key(&mut self, _: &russh::keys::PublicKey) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

#[tokio::test]
async fn sftp_server_accepts_password_upload() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config =
        PushReceiverConfig::new(PushProtocol::Sftp, "127.0.0.1", 0, temp_dir.path()).with_account(
            ReceiverAccount::new("camera", Some("secret"), "Studio Camera"),
        );
    let server = SftpPushServer::bind(config)
        .await
        .expect("SFTP server should bind");
    let local_addr = server.local_addr();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let task = tokio::spawn(server.run_until(async {
        let _ = shutdown_rx.await;
    }));

    let mut session = client::connect(
        Arc::new(client::Config::default()),
        local_addr,
        AcceptAnyServerKey,
    )
    .await
    .expect("SFTP client should connect");
    assert!(session
        .authenticate_password("camera", "secret")
        .await
        .expect("password auth should complete")
        .success());
    let channel = session
        .channel_open_session()
        .await
        .expect("session channel should open");
    channel
        .request_subsystem(true, "sftp")
        .await
        .expect("SFTP subsystem should open");
    let sftp = SftpSession::new(channel.into_stream())
        .await
        .expect("SFTP session should start");
    let mut file = sftp
        .open_with_flags(
            "DCIM/100CAM/IMG_1001.NEF",
            OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
        )
        .await
        .expect("remote file should open");
    file.write_all(&[1, 2, 3, 4])
        .await
        .expect("remote file should write");
    file.shutdown()
        .await
        .expect("remote file should close cleanly");

    shutdown_tx.send(()).ok();
    task.await
        .expect("server task should join")
        .expect("server should stop cleanly");

    assert_eq!(
        std::fs::read(temp_dir.path().join("IMG_1001.NEF")).expect("uploaded file should exist"),
        vec![1, 2, 3, 4]
    );
    let records = read_transfer_log(temp_dir.path()).expect("transfer log should read");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].protocol, "sftp");
    assert_eq!(records[0].original_path, "DCIM/100CAM/IMG_1001.NEF");
    assert_eq!(records[0].source_name.as_deref(), Some("Studio Camera"));
}

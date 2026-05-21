use std::sync::{Arc, Mutex};

use camera_connector_core::{
    read_connected_devices, read_transfer_log, PushProtocol, PushReceiverConfig, ReceiverAccount,
    SftpPushServer,
};
use russh::client;
use russh_sftp::{client::SftpSession, protocol::OpenFlags};
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};

struct AcceptAnyServerKey;

impl client::Handler for AcceptAnyServerKey {
    type Error = russh::Error;

    async fn check_server_key(&mut self, _: &russh::keys::PublicKey) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

struct CapturingServerKey {
    key: Arc<Mutex<Option<String>>>,
}

impl client::Handler for CapturingServerKey {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        *self.key.lock().expect("server key capture should lock") =
            Some(key.to_openssh().expect("server key should encode"));
        Ok(true)
    }
}

#[tokio::test]
async fn sftp_server_accepts_password_upload() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state_dir = tempfile::tempdir().expect("state dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Sftp, "127.0.0.1", 0, temp_dir.path())
        .with_state_dir(state_dir.path())
        .with_account(ReceiverAccount::new(
            "camera",
            Some("secret"),
            "Studio Camera",
        ));
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
    let connected = read_connected_devices(state_dir.path()).expect("devices should read");
    assert_eq!(connected.len(), 1);
    assert_eq!(connected[0].username.as_deref(), Some("camera"));
    assert_eq!(connected[0].source_name.as_deref(), Some("Studio Camera"));
    assert!(connected[0].online);

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
    session
        .disconnect(russh::Disconnect::ByApplication, "done", "")
        .await
        .expect("client should disconnect cleanly");

    let disconnected = wait_for_sftp_device_disconnect(state_dir.path()).await;
    assert_eq!(disconnected.len(), 1);
    assert!(!disconnected[0].online);
    assert_eq!(disconnected[0].active_connections, 0);

    shutdown_tx.send(()).ok();
    task.await
        .expect("server task should join")
        .expect("server should stop cleanly");

    assert_eq!(
        std::fs::read(temp_dir.path().join("IMG_1001.NEF")).expect("uploaded file should exist"),
        vec![1, 2, 3, 4]
    );
    let records = read_transfer_log(state_dir.path()).expect("transfer log should read");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].protocol, "sftp");
    assert_eq!(records[0].original_path, "DCIM/100CAM/IMG_1001.NEF");
    assert_eq!(records[0].username.as_deref(), Some("camera"));
    assert_eq!(records[0].source_name.as_deref(), Some("Studio Camera"));
    assert!(!temp_dir.path().join("transfer-log.jsonl").exists());
    assert!(!temp_dir.path().join("connected-devices.json").exists());
    assert!(!temp_dir.path().join("sftp-host-key").exists());
    assert!(state_dir.path().join("sftp-host-key").exists());
}

#[tokio::test]
async fn sftp_server_reuses_persisted_host_key() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");

    let first = capture_server_key(temp_dir.path()).await;
    let second = capture_server_key(temp_dir.path()).await;

    assert_eq!(first, second);
    assert!(temp_dir.path().join("sftp-host-key").exists());
}

async fn wait_for_sftp_device_disconnect(
    output_dir: &std::path::Path,
) -> Vec<camera_connector_core::ConnectedDevice> {
    for _ in 0..20 {
        let devices = read_connected_devices(output_dir).expect("devices should read");
        if devices
            .first()
            .map(|device| !device.online)
            .unwrap_or(false)
        {
            return devices;
        }
        sleep(Duration::from_millis(25)).await;
    }
    read_connected_devices(output_dir).expect("devices should read")
}

async fn capture_server_key(output_dir: &std::path::Path) -> String {
    let config = PushReceiverConfig::new(PushProtocol::Sftp, "127.0.0.1", 0, output_dir);
    let server = SftpPushServer::bind(config)
        .await
        .expect("SFTP server should bind");
    let local_addr = server.local_addr();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let task = tokio::spawn(server.run_until(async {
        let _ = shutdown_rx.await;
    }));
    let key = Arc::new(Mutex::new(None));
    let session = client::connect(
        Arc::new(client::Config::default()),
        local_addr,
        CapturingServerKey {
            key: Arc::clone(&key),
        },
    )
    .await
    .expect("SFTP client should connect");
    session
        .disconnect(russh::Disconnect::ByApplication, "done", "")
        .await
        .expect("client should disconnect cleanly");
    shutdown_tx.send(()).ok();
    task.await
        .expect("server task should join")
        .expect("server should stop cleanly");

    let captured = key
        .lock()
        .expect("server key capture should lock")
        .clone()
        .expect("server key should be captured");
    captured
}

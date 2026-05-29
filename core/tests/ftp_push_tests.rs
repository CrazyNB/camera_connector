use std::fs;
use std::net::SocketAddr;

use camera_connector_core::{
    read_connected_devices, read_transfer_log, AssetGroupQuery, FtpPushServer, PushProtocol,
    PushReceiverConfig, ReceiverAccount, SqliteStore, TransferStatus,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn ftp_server_accepts_passive_stor_upload() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state_dir = tempfile::tempdir().expect("state dir should be created");
    let store = SqliteStore::open_state_dir(state_dir.path()).expect("store should open");
    let project = store
        .create_project("FTP Upload")
        .expect("project should create");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path())
        .with_state_dir(state_dir.path())
        .with_active_project(project.project_id.clone())
        .with_account(ReceiverAccount::new("z5", Some("secret"), "Studio A"));
    let server = FtpPushServer::bind(config)
        .await
        .expect("server should bind");
    let control_addr = server.local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let server_task = tokio::spawn(async move {
        server
            .run_until(async {
                let _ = shutdown_rx.await;
            })
            .await
    });

    let mut control = BufReader::new(
        tokio::net::TcpStream::connect(control_addr)
            .await
            .expect("control connection should open"),
    );

    assert_reply(&mut control, "220").await;
    let connected = read_connected_devices(state_dir.path()).expect("devices should read");
    assert_eq!(connected.len(), 1);
    assert_eq!(connected[0].remote_addr, "127.0.0.1");
    assert_eq!(connected[0].source_name.as_deref(), None);
    assert_eq!(connected[0].username.as_deref(), None);
    assert!(connected[0].online);

    command(&mut control, "USER z5").await;
    assert_reply(&mut control, "331").await;
    command(&mut control, "PASS secret").await;
    assert_reply(&mut control, "230").await;
    let logged_in = read_connected_devices(state_dir.path()).expect("devices should read");
    assert_eq!(logged_in[0].username.as_deref(), Some("z5"));
    assert_eq!(logged_in[0].source_name.as_deref(), Some("Studio A"));
    command(&mut control, "TYPE I").await;
    assert_reply(&mut control, "200").await;
    command(&mut control, "PASV").await;
    let passive_reply = read_reply(&mut control).await;
    assert!(passive_reply.starts_with("227"), "{passive_reply}");
    let data_addr = passive_addr_from_reply(&passive_reply);

    let mut data = tokio::net::TcpStream::connect(data_addr)
        .await
        .expect("data connection should open");
    command(&mut control, "CWD DCIM").await;
    assert_reply(&mut control, "250").await;
    command(&mut control, "CWD 100CANON").await;
    assert_reply(&mut control, "250").await;
    command(&mut control, "STOR IMG_4321.CR3").await;
    assert_reply(&mut control, "150").await;
    data.write_all(&[1, 2, 3, 4, 5])
        .await
        .expect("data should write");
    assert!(
        wait_for_staged_file(state_dir.path().join("staging")).await,
        "staged upload file should exist before the data connection closes"
    );
    assert!(!temp_dir.path().join("IMG_4321.CR3").exists());
    data.shutdown().await.expect("data should close");
    assert_reply(&mut control, "226").await;

    command(&mut control, "QUIT").await;
    assert_reply(&mut control, "221").await;

    let _ = shutdown_tx.send(());
    server_task
        .await
        .expect("server task should join")
        .expect("server should stop cleanly");

    let bytes = fs::read(temp_dir.path().join("IMG_4321.CR3")).expect("uploaded file should read");
    assert_eq!(bytes, vec![1, 2, 3, 4, 5]);
    assert!(!temp_dir.path().join("DCIM").exists());

    let records = read_transfer_log(state_dir.path()).expect("transfer log should read");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].remote_addr.as_deref(), Some("127.0.0.1"));
    assert_eq!(records[0].username.as_deref(), Some("z5"));
    assert_eq!(records[0].source_name.as_deref(), Some("Studio A"));
    assert_eq!(records[0].original_path, "DCIM/100CANON/IMG_4321.CR3");
    assert_eq!(
        records[0].virtual_display_path(None),
        "Studio A/DCIM/100CANON/IMG_4321.CR3"
    );
    assert_eq!(records[0].final_filename, "IMG_4321.CR3");
    assert_eq!(records[0].size_bytes, 5);
    assert_eq!(records[0].status, TransferStatus::Completed);

    let disconnected = read_connected_devices(state_dir.path()).expect("devices should read");
    assert_eq!(disconnected.len(), 1);
    assert!(!disconnected[0].online);
    assert!(!temp_dir.path().join("transfer-log.jsonl").exists());
    assert!(!temp_dir.path().join("connected-devices.json").exists());
}

#[tokio::test]
async fn ftp_server_indexes_uploads_under_active_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state_dir = tempfile::tempdir().expect("state dir should be created");
    let store = SqliteStore::open_state_dir(state_dir.path()).expect("store should open");
    let project = store
        .create_project("FTP Project")
        .expect("project should create");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path())
        .with_state_dir(state_dir.path())
        .with_active_project(project.project_id.clone());
    let server = FtpPushServer::bind(config)
        .await
        .expect("server should bind");
    let control_addr = server.local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let server_task = tokio::spawn(async move {
        server
            .run_until(async {
                let _ = shutdown_rx.await;
            })
            .await
    });

    let mut control = BufReader::new(
        tokio::net::TcpStream::connect(control_addr)
            .await
            .expect("control connection should open"),
    );

    assert_reply(&mut control, "220").await;
    command(&mut control, "PASV").await;
    let passive_reply = read_reply(&mut control).await;
    let data_addr = passive_addr_from_reply(&passive_reply);
    let mut data = tokio::net::TcpStream::connect(data_addr)
        .await
        .expect("data connection should open");
    command(&mut control, "STOR IMG_5001.JPG").await;
    assert_reply(&mut control, "150").await;
    data.write_all(&[1, 2, 3]).await.expect("data should write");
    data.shutdown().await.expect("data should close");
    assert_reply(&mut control, "226").await;
    command(&mut control, "QUIT").await;
    assert_reply(&mut control, "221").await;

    let _ = shutdown_tx.send(());
    server_task
        .await
        .expect("server task should join")
        .expect("server should stop cleanly");

    let page = store
        .asset_group_page(
            &project.project_id,
            camera_connector_core::AssetGroupQuery::default(),
            0,
            25,
        )
        .expect("indexed groups should query");
    assert_eq!(page.total_groups, 1);
    assert_eq!(page.summary.asset_count, 1);
    assert_eq!(page.groups[0].group_key, "IMG_5001");
}

#[tokio::test]
async fn ftp_server_defers_final_publish_when_configured() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state_dir = tempfile::tempdir().expect("state dir should be created");
    let store = SqliteStore::open_state_dir(state_dir.path()).expect("store should open");
    let project = store
        .create_project("Deferred FTP Project")
        .expect("project should create");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path())
        .with_state_dir(state_dir.path())
        .with_active_project(project.project_id.clone())
        .with_deferred_publish();
    let server = FtpPushServer::bind(config)
        .await
        .expect("server should bind");
    let control_addr = server.local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let server_task = tokio::spawn(async move {
        server
            .run_until(async {
                let _ = shutdown_rx.await;
            })
            .await
    });

    let mut control = BufReader::new(
        tokio::net::TcpStream::connect(control_addr)
            .await
            .expect("control connection should open"),
    );

    assert_reply(&mut control, "220").await;
    command(&mut control, "PASV").await;
    let passive_reply = read_reply(&mut control).await;
    let data_addr = passive_addr_from_reply(&passive_reply);
    let mut data = tokio::net::TcpStream::connect(data_addr)
        .await
        .expect("data connection should open");
    command(&mut control, "CWD DCIM").await;
    assert_reply(&mut control, "250").await;
    command(&mut control, "STOR IMG_9101.JPG").await;
    assert_reply(&mut control, "150").await;
    data.write_all(&[9, 1, 0, 1])
        .await
        .expect("data should write");
    data.shutdown().await.expect("data should close");
    assert_reply(&mut control, "226").await;
    command(&mut control, "QUIT").await;
    assert_reply(&mut control, "221").await;

    let _ = shutdown_tx.send(());
    server_task
        .await
        .expect("server task should join")
        .expect("server should stop cleanly");

    assert!(!temp_dir.path().join("IMG_9101.JPG").exists());
    let records = read_transfer_log(state_dir.path()).expect("transfer log should read");
    assert!(records.is_empty());

    let summary = store
        .publish_queue_summary(&project.project_id)
        .expect("queue summary should read");
    assert_eq!(summary.staged_count, 1);
    assert_eq!(summary.pending_count, 1);
    assert_eq!(summary.completed_count, 0);
    let page = store
        .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset groups should query");
    assert_eq!(page.total_groups, 0);

    let item = store
        .claim_next_publish_item()
        .expect("publish item should claim")
        .expect("deferred publish should leave a queue item");
    assert_eq!(item.final_filename, "IMG_9101.JPG");
    assert_eq!(item.protocol.as_deref(), Some("ftp"));
    assert_eq!(item.original_path.as_deref(), Some("DCIM/IMG_9101.JPG"));
    assert!(std::path::Path::new(&item.staged_path).exists());
}

#[tokio::test]
async fn ftp_server_rejects_unknown_account_when_accounts_are_configured() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path())
        .with_account(ReceiverAccount::new("z5", Some("secret"), "Studio A"));
    let server = FtpPushServer::bind(config)
        .await
        .expect("server should bind");
    let control_addr = server.local_addr();
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let server_task = tokio::spawn(async move {
        server
            .run_until(async {
                let _ = shutdown_rx.await;
            })
            .await
    });

    let mut control = BufReader::new(
        tokio::net::TcpStream::connect(control_addr)
            .await
            .expect("control connection should open"),
    );

    assert_reply(&mut control, "220").await;
    command(&mut control, "USER wrong").await;
    assert_reply(&mut control, "530").await;
    command(&mut control, "PASV").await;
    assert_reply(&mut control, "530").await;
    command(&mut control, "QUIT").await;
    assert_reply(&mut control, "221").await;

    let _ = shutdown_tx.send(());
    server_task
        .await
        .expect("server task should join")
        .expect("server should stop cleanly");
}

async fn command(control: &mut BufReader<tokio::net::TcpStream>, line: &str) {
    control
        .get_mut()
        .write_all(format!("{line}\r\n").as_bytes())
        .await
        .expect("command should write");
}

async fn assert_reply(control: &mut BufReader<tokio::net::TcpStream>, prefix: &str) {
    let reply = read_reply(control).await;
    assert!(reply.starts_with(prefix), "{reply}");
}

async fn read_reply(control: &mut BufReader<tokio::net::TcpStream>) -> String {
    let mut line = String::new();
    control
        .read_line(&mut line)
        .await
        .expect("reply should read");
    line
}

fn passive_addr_from_reply(reply: &str) -> SocketAddr {
    let start = reply.find('(').expect("passive tuple should start") + 1;
    let end = reply.find(')').expect("passive tuple should end");
    let parts: Vec<u16> = reply[start..end]
        .split(',')
        .map(|part| part.parse().expect("passive tuple part should parse"))
        .collect();
    let port = parts[4] * 256 + parts[5];
    format!("{}.{}.{}.{}:{port}", parts[0], parts[1], parts[2], parts[3])
        .parse()
        .expect("passive address should parse")
}

async fn wait_for_staged_file(path: impl AsRef<std::path::Path>) -> bool {
    let path = path.as_ref();
    for _ in 0..20 {
        if path
            .read_dir()
            .map(|mut entries| entries.any(|entry| entry.is_ok()))
            .unwrap_or(false)
        {
            return true;
        }
        sleep(Duration::from_millis(25)).await;
    }
    false
}

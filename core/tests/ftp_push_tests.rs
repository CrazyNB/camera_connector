use std::fs;
use std::net::SocketAddr;

use camera_connector_core::{
    read_transfer_log, FtpPushServer, PushProtocol, PushReceiverConfig, TransferStatus,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::test]
async fn ftp_server_accepts_passive_stor_upload() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let config = PushReceiverConfig::new(PushProtocol::Ftp, "127.0.0.1", 0, temp_dir.path())
        .with_source_name("Studio A");
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
    command(&mut control, "USER anonymous").await;
    assert_reply(&mut control, "230").await;
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

    let records = read_transfer_log(temp_dir.path()).expect("transfer log should read");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].source_name.as_deref(), Some("Studio A"));
    assert_eq!(records[0].original_path, "DCIM/100CANON/IMG_4321.CR3");
    assert_eq!(records[0].final_filename, "IMG_4321.CR3");
    assert_eq!(records[0].size_bytes, 5);
    assert_eq!(records[0].status, TransferStatus::Completed);
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

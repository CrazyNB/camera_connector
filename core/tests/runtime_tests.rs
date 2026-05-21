use std::net::TcpListener;

use camera_connector_core::{
    read_receiver_runtime_status, write_receiver_runtime_status, CameraConnectorConfig,
    CameraConnectorRuntime, CameraConnectorService, PushProtocol, ReceiverAuthMode,
    ReceiverConfigRequest, ReceiverRuntimePhase, ReceiverRuntimeStatus,
};

#[tokio::test]
async fn runtime_starts_and_stops_ftp_receiver() {
    let output_dir = unique_temp_dir("runtime-output");
    let runtime = CameraConnectorRuntime::new(CameraConnectorService::new(None));

    assert_eq!(runtime.status().phase, ReceiverRuntimePhase::Stopped);

    let running = runtime
        .start_receiver(ReceiverConfigRequest {
            protocol: PushProtocol::Ftp,
            bind_host: "127.0.0.1".to_string(),
            port: 0,
            output_dir: output_dir.clone(),
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
        })
        .await
        .expect("receiver should start");

    assert_eq!(running.phase, ReceiverRuntimePhase::Running);
    assert_eq!(running.protocol, Some(PushProtocol::Ftp));
    assert_eq!(running.auth_mode, ReceiverAuthMode::Anonymous);
    assert!(
        running
            .local_addr
            .expect("local address should exist")
            .port()
            > 0
    );
    assert_eq!(running.output_dir.as_deref(), Some(output_dir.as_path()));
    assert_eq!(runtime.status().phase, ReceiverRuntimePhase::Running);
    assert_eq!(
        read_receiver_runtime_status(&output_dir)
            .expect("runtime status should read")
            .expect("runtime status should exist")
            .phase,
        ReceiverRuntimePhase::Running
    );

    let stopped = runtime.stop_receiver().await.expect("receiver should stop");

    assert_eq!(stopped.phase, ReceiverRuntimePhase::Stopped);
    assert!(runtime.status().local_addr.is_none());
    assert_eq!(
        read_receiver_runtime_status(&output_dir)
            .expect("runtime status should read")
            .expect("runtime status should exist")
            .phase,
        ReceiverRuntimePhase::Stopped
    );
    let _ = std::fs::remove_dir_all(output_dir);
}

#[tokio::test]
async fn runtime_status_reports_account_authentication_mode() {
    let config_path = unique_temp_path("runtime-auth-config");
    let output_dir = unique_temp_dir("runtime-auth-output");
    let mut app_config = CameraConnectorConfig::default();
    app_config
        .set_account("z5", Some("secret"), "Z5_2")
        .expect("account should save");
    app_config
        .save(Some(&config_path))
        .expect("config should save");
    let runtime =
        CameraConnectorRuntime::new(CameraConnectorService::new(Some(config_path.clone())));

    let running = runtime
        .start_receiver(ReceiverConfigRequest {
            protocol: PushProtocol::Ftp,
            bind_host: "127.0.0.1".to_string(),
            port: 0,
            output_dir: output_dir.clone(),
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
        })
        .await
        .expect("receiver should start");

    assert_eq!(running.account_count, 1);
    assert_eq!(running.auth_mode, ReceiverAuthMode::Accounts);

    runtime.stop_receiver().await.expect("receiver should stop");
    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(output_dir);
}

#[tokio::test]
async fn runtime_records_failed_status_when_port_is_unavailable() {
    let occupied = TcpListener::bind(("127.0.0.1", 0)).expect("test port should bind");
    let port = occupied
        .local_addr()
        .expect("local addr should exist")
        .port();
    let output_dir = unique_temp_dir("runtime-failed-output");
    let runtime = CameraConnectorRuntime::new(CameraConnectorService::new(None));

    let result = runtime
        .start_receiver(ReceiverConfigRequest {
            protocol: PushProtocol::Ftp,
            bind_host: "127.0.0.1".to_string(),
            port,
            output_dir: output_dir.clone(),
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
        })
        .await;

    assert!(result.is_err());
    let status = runtime.status();
    assert_eq!(status.phase, ReceiverRuntimePhase::Failed);
    assert!(status
        .message
        .as_deref()
        .unwrap_or_default()
        .contains("io error"));
    assert_eq!(
        read_receiver_runtime_status(&output_dir)
            .expect("runtime status should read")
            .expect("runtime status should exist")
            .phase,
        ReceiverRuntimePhase::Failed
    );

    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn read_receiver_runtime_status_marks_dead_running_listener_stopped() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral port should bind");
    let local_addr = listener
        .local_addr()
        .expect("listener address should exist");
    drop(listener);

    write_receiver_runtime_status(
        temp_dir.path(),
        &ReceiverRuntimeStatus {
            phase: ReceiverRuntimePhase::Running,
            protocol: Some(PushProtocol::Ftp),
            auth_mode: ReceiverAuthMode::Anonymous,
            local_addr: Some(local_addr),
            output_dir: Some(temp_dir.path().to_path_buf()),
            account_count: 0,
            message: None,
        },
    )
    .expect("runtime status should write");

    let status = read_receiver_runtime_status(temp_dir.path())
        .expect("runtime status should read")
        .expect("runtime status should exist");

    assert_eq!(status.phase, ReceiverRuntimePhase::Stopped);
    assert_eq!(
        status.message.as_deref(),
        Some("receiver process is not listening")
    );
}

fn unique_temp_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camera-connector-{name}-{}", unique_suffix()))
}

fn unique_temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("camera-connector-{name}-{}.json", unique_suffix()))
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

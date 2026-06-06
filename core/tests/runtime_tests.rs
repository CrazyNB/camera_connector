use std::net::TcpListener;

use camera_connector_core::{
    read_receiver_runtime_status, receiver_runtime_status_path, write_receiver_runtime_status,
    CameraConnectorRuntime, CameraConnectorService, PushProtocol, ReceiverAuthMode,
    ReceiverConfigRequest, ReceiverRuntimePhase, ReceiverRuntimeStatus, ReceiverSettingsUpdate,
};

#[tokio::test]
async fn runtime_starts_and_stops_ftp_receiver() {
    let config_path = unique_temp_path("runtime-config");
    let output_dir = unique_temp_dir("runtime-output");
    let state_dir = unique_temp_dir("runtime-state");
    let runtime =
        CameraConnectorRuntime::new(CameraConnectorService::new(Some(config_path.clone())));

    assert_eq!(runtime.status().phase, ReceiverRuntimePhase::Stopped);

    let running = runtime
        .start_receiver(ReceiverConfigRequest {
            protocol: Some(PushProtocol::Ftp),
            bind_host: Some("127.0.0.1".to_string()),
            port: Some(0),
            output_dir: Some(output_dir.clone()),
            state_dir: Some(state_dir.clone()),
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
            defer_publish: None,
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
    assert_eq!(running.state_dir.as_deref(), Some(state_dir.as_path()));
    assert_eq!(runtime.status().phase, ReceiverRuntimePhase::Running);
    assert_eq!(
        read_receiver_runtime_status(&state_dir)
            .expect("runtime status should read")
            .expect("runtime status should exist")
            .phase,
        ReceiverRuntimePhase::Running
    );

    let stopped = runtime.stop_receiver().await.expect("receiver should stop");

    assert_eq!(stopped.phase, ReceiverRuntimePhase::Stopped);
    assert!(runtime.status().local_addr.is_none());
    assert_eq!(
        read_receiver_runtime_status(&state_dir)
            .expect("runtime status should read")
            .expect("runtime status should exist")
            .phase,
        ReceiverRuntimePhase::Stopped
    );
    let _ = std::fs::remove_dir_all(output_dir);
    let _ = std::fs::remove_dir_all(state_dir);
    let _ = std::fs::remove_file(config_path);
}

#[tokio::test]
async fn runtime_status_reports_account_authentication_mode() {
    let config_path = unique_temp_path("runtime-auth-config");
    let output_dir = unique_temp_dir("runtime-auth-output");
    let state_dir = unique_temp_dir("runtime-auth-state");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            state_dir: Some(state_dir.clone()),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");
    service
        .set_account("z5", Some("secret"), "Z5_2")
        .expect("account should save");
    let runtime = CameraConnectorRuntime::new(service);

    let running = runtime
        .start_receiver(ReceiverConfigRequest {
            protocol: Some(PushProtocol::Ftp),
            bind_host: Some("127.0.0.1".to_string()),
            port: Some(0),
            output_dir: Some(output_dir.clone()),
            state_dir: Some(state_dir.clone()),
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
            defer_publish: None,
        })
        .await
        .expect("receiver should start");

    assert_eq!(running.account_count, 1);
    assert_eq!(running.auth_mode, ReceiverAuthMode::Accounts);

    runtime.stop_receiver().await.expect("receiver should stop");
    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir_all(output_dir);
    let _ = std::fs::remove_dir_all(state_dir);
}

#[tokio::test]
async fn runtime_records_failed_status_when_port_is_unavailable() {
    let config_path = unique_temp_path("runtime-failed-config");
    let occupied = TcpListener::bind(("127.0.0.1", 0)).expect("test port should bind");
    let port = occupied
        .local_addr()
        .expect("local addr should exist")
        .port();
    let output_dir = unique_temp_dir("runtime-failed-output");
    let state_dir = unique_temp_dir("runtime-failed-state");
    let runtime =
        CameraConnectorRuntime::new(CameraConnectorService::new(Some(config_path.clone())));

    let result = runtime
        .start_receiver(ReceiverConfigRequest {
            protocol: Some(PushProtocol::Ftp),
            bind_host: Some("127.0.0.1".to_string()),
            port: Some(port),
            output_dir: Some(output_dir.clone()),
            state_dir: Some(state_dir.clone()),
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
            defer_publish: None,
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
        read_receiver_runtime_status(&state_dir)
            .expect("runtime status should read")
            .expect("runtime status should exist")
            .phase,
        ReceiverRuntimePhase::Failed
    );

    let _ = std::fs::remove_dir_all(output_dir);
    let _ = std::fs::remove_dir_all(state_dir);
    let _ = std::fs::remove_file(config_path);
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
            state_dir: Some(temp_dir.path().to_path_buf()),
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

#[test]
fn read_receiver_runtime_status_marks_dead_stopping_listener_stopped() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let listener = TcpListener::bind("127.0.0.1:0").expect("ephemeral port should bind");
    let local_addr = listener
        .local_addr()
        .expect("listener address should exist");
    drop(listener);

    write_receiver_runtime_status(
        temp_dir.path(),
        &ReceiverRuntimeStatus {
            phase: ReceiverRuntimePhase::Stopping,
            protocol: Some(PushProtocol::Ftp),
            auth_mode: ReceiverAuthMode::Anonymous,
            local_addr: Some(local_addr),
            output_dir: Some(temp_dir.path().to_path_buf()),
            state_dir: Some(temp_dir.path().to_path_buf()),
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

#[test]
fn receiver_runtime_status_is_stored_in_sqlite() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");

    write_receiver_runtime_status(
        temp_dir.path(),
        &ReceiverRuntimeStatus {
            phase: ReceiverRuntimePhase::Stopped,
            protocol: Some(PushProtocol::Sftp),
            auth_mode: ReceiverAuthMode::Accounts,
            local_addr: None,
            output_dir: Some(temp_dir.path().join("output")),
            state_dir: Some(temp_dir.path().to_path_buf()),
            account_count: 2,
            message: Some("stopped for test".to_string()),
        },
    )
    .expect("runtime status should write");

    assert!(!receiver_runtime_status_path(temp_dir.path()).exists());
    let status = read_receiver_runtime_status(temp_dir.path())
        .expect("runtime status should read")
        .expect("runtime status should exist");

    assert_eq!(status.phase, ReceiverRuntimePhase::Stopped);
    assert_eq!(status.protocol, Some(PushProtocol::Sftp));
    assert_eq!(status.auth_mode, ReceiverAuthMode::Accounts);
    assert_eq!(status.account_count, 2);
    assert_eq!(status.message.as_deref(), Some("stopped for test"));
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

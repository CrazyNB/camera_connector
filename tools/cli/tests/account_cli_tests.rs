use std::process::Command;

use camera_connector_core::{CameraConnectorService, ReceiverSettingsUpdate, SqliteStore};

#[test]
fn account_list_prints_account_runtime_state() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("config.json");
    let state_dir = temp_dir.path().join("state");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    service
        .set_receiver_settings(ReceiverSettingsUpdate {
            state_dir: Some(state_dir.clone()),
            ..ReceiverSettingsUpdate::default()
        })
        .expect("receiver settings should save");
    service
        .set_account("z5", Some("secret"), "Studio Z5")
        .expect("account should save");
    let store = SqliteStore::open_state_dir(&state_dir).expect("store should open");
    store
        .record_connected_device("192.168.137.56", Some(51120), None, None)
        .expect("device should connect");
    store
        .record_authenticated_device("192.168.137.56", Some("Studio Z5"), Some("z5"))
        .expect("device should authenticate");

    let output = Command::new(env!("CARGO_BIN_EXE_camera-connector"))
        .arg("account")
        .arg("--config")
        .arg(&config_path)
        .arg("list")
        .output()
        .expect("account list command should run");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");

    assert!(output.status.success(), "{stderr}");
    assert!(stdout.contains("config: "));
    assert!(stdout.contains(
        "account\tusername=z5\tdevice=Studio Z5\tpassword_configured=true\tonline=true\tconnections=1\tremote=192.168.137.56\tport=51120"
    ));
    assert!(stdout.contains("last_seen_ms="));
    assert!(stdout.contains("last_disconnected_ms=-"));
    assert!(!stdout.contains("password=configured"));
}

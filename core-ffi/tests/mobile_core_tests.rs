use camera_connector_ffi::{MobileCore, MobileReceiverSettingsPatch};
use serde_json::Value;

#[test]
fn mobile_core_saves_receiver_settings_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let output_dir = temp.path().join("output");
    let state_dir = temp.path().join("state");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let json = core
        .save_receiver_settings_json(MobileReceiverSettingsPatch {
            protocol: Some("sftp".to_string()),
            bind_host: Some("0.0.0.0".to_string()),
            ftp_port: Some(2121),
            sftp_port: Some(2222),
            output_dir: Some(output_dir.to_string_lossy().into_owned()),
            state_dir: Some(state_dir.to_string_lossy().into_owned()),
            advertised_host: Some("192.168.137.1".to_string()),
            source_name: Some("Studio Camera".to_string()),
        })
        .unwrap();

    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["protocol"], "Sftp");
    assert_eq!(value["bind_host"], "0.0.0.0");
    assert_eq!(value["ftp_port"], 2121);
    assert_eq!(value["sftp_port"], 2222);
    assert_eq!(value["source_name"], "Studio Camera");
}

#[test]
fn mobile_core_rejects_ftps_protocol() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let error = core
        .save_receiver_settings_json(MobileReceiverSettingsPatch {
            protocol: Some("ftps".to_string()),
            ..MobileReceiverSettingsPatch::default()
        })
        .unwrap_err()
        .to_string();

    assert!(error.contains("invalid protocol: ftps"));
}

#[test]
fn mobile_core_saves_account_without_plaintext_password() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let json = core
        .save_device_account_json(
            "camera01".to_string(),
            Some("secret".to_string()),
            "Camera 01".to_string(),
        )
        .unwrap();

    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["username"], "camera01");
    assert_eq!(value["device_name"], "Camera 01");
    assert_eq!(value["password_configured"], true);
    assert!(!json.contains("secret"));
}

#[test]
fn mobile_core_returns_dashboard_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let state_dir = temp.path().join("state");
    std::fs::create_dir_all(&state_dir).unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.save_device_account_json(
        "camera01".to_string(),
        Some("secret".to_string()),
        "Camera 01".to_string(),
    )
    .unwrap();

    let json = core
        .dashboard_json(Some(state_dir.to_string_lossy().into_owned()), 0, 25)
        .unwrap();

    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["accounts"][0]["username"], "camera01");
    assert_eq!(value["accounts"][0]["device_name"], "Camera 01");
    assert_eq!(value["assets"]["limit"], 25);
    assert!(json.contains("config_path"));
}

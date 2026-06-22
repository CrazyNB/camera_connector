use super::*;

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
fn mobile_core_starts_and_stops_receiver_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let output_dir = temp.path().join("output");
    let state_dir = temp.path().join("state");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.save_receiver_settings_json(MobileReceiverSettingsPatch {
        protocol: Some("ftp".to_string()),
        bind_host: Some("127.0.0.1".to_string()),
        ftp_port: Some(0),
        output_dir: Some(output_dir.to_string_lossy().into_owned()),
        state_dir: Some(state_dir.to_string_lossy().into_owned()),
        ..MobileReceiverSettingsPatch::default()
    })
    .unwrap();

    let started_json = core.start_receiver_json().unwrap();
    let started: Value = serde_json::from_str(&started_json).unwrap();
    assert_eq!(started["phase"], "Running");
    assert_eq!(started["protocol"], "Ftp");
    assert!(started["local_addr"]
        .as_str()
        .unwrap()
        .starts_with("127.0.0.1:"));

    let stopped_json = core.stop_receiver_json().unwrap();
    let stopped: Value = serde_json::from_str(&stopped_json).unwrap();
    assert_eq!(stopped["phase"], "Stopped");
}

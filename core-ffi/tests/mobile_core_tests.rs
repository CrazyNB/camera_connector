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
fn mobile_core_removes_account_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.save_device_account_json(
        "camera01".to_string(),
        Some("secret".to_string()),
        "Camera 01".to_string(),
    )
    .unwrap();

    let json = core
        .remove_device_account_json("camera01".to_string())
        .unwrap();

    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["username"], "camera01");
    assert_eq!(value["removed"], true);
    let dashboard: Value = serde_json::from_str(
        &core
            .dashboard_json(
                Some(temp.path().join("state").to_string_lossy().into_owned()),
                0,
                25,
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(dashboard["accounts"].as_array().unwrap().len(), 0);
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
    assert_eq!(value["receiver_settings"]["protocol"], "Ftp");
    assert_eq!(value["accounts"][0]["username"], "camera01");
    assert_eq!(value["accounts"][0]["device_name"], "Camera 01");
    assert_eq!(value["assets"]["limit"], 25);
    assert!(json.contains("config_path"));
}

#[test]
fn mobile_core_manages_projects_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let initial_active: Value = serde_json::from_str(&core.active_project_json().unwrap()).unwrap();
    assert!(initial_active.is_null());

    let created_json = core
        .create_project_json("Wedding Shoot".to_string())
        .unwrap();
    let created: Value = serde_json::from_str(&created_json).unwrap();
    assert_eq!(created["name"], "Wedding Shoot");
    assert_eq!(created["slug"], "wedding-shoot");
    assert_eq!(created["status"], "Active");
    let project_id = created["project_id"].as_str().unwrap().to_string();

    let listed_json = core.list_projects_json().unwrap();
    let listed: Value = serde_json::from_str(&listed_json).unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["project_id"], project_id);

    let active_json = core.set_active_project_json(project_id.clone()).unwrap();
    let active: Value = serde_json::from_str(&active_json).unwrap();
    assert_eq!(active["project_id"], project_id);

    let active_again: Value = serde_json::from_str(&core.active_project_json().unwrap()).unwrap();
    assert_eq!(active_again["project_id"], project_id);

    let archived_json = core.archive_project_json(project_id.clone()).unwrap();
    let archived: Value = serde_json::from_str(&archived_json).unwrap();
    assert_eq!(archived["project_id"], project_id);
    assert_eq!(archived["status"], "Archived");
    let active_after_archive: Value =
        serde_json::from_str(&core.active_project_json().unwrap()).unwrap();
    assert!(active_after_archive.is_null());
    assert!(core.set_active_project_json(project_id.clone()).is_err());

    let restored_json = core.restore_project_json(project_id.clone()).unwrap();
    let restored: Value = serde_json::from_str(&restored_json).unwrap();
    assert_eq!(restored["project_id"], project_id);
    assert_eq!(restored["status"], "Active");
    let active_after_restore: Value =
        serde_json::from_str(&core.set_active_project_json(project_id.clone()).unwrap()).unwrap();
    assert_eq!(active_after_restore["project_id"], project_id);
}

#[test]
fn mobile_core_ensures_active_project_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let ensured_json = core.ensure_active_project_json().unwrap();
    let ensured: Value = serde_json::from_str(&ensured_json).unwrap();

    assert_eq!(ensured["project_id"], "project-inbox");
    assert_eq!(ensured["name"], "Inbox");
    let active: Value = serde_json::from_str(&core.active_project_json().unwrap()).unwrap();
    assert_eq!(active["project_id"], "project-inbox");
    let projects: Value = serde_json::from_str(&core.list_projects_json().unwrap()).unwrap();
    assert_eq!(projects.as_array().unwrap().len(), 1);
}

#[test]
fn mobile_core_returns_project_dashboard_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let created: Value = serde_json::from_str(
        &core
            .create_project_json("Catalog Session".to_string())
            .unwrap(),
    )
    .unwrap();
    let project_id = created["project_id"].as_str().unwrap();

    let dashboard_json = core
        .project_dashboard_json(project_id.to_string(), 0, 50)
        .unwrap();

    let dashboard: Value = serde_json::from_str(&dashboard_json).unwrap();
    assert_eq!(dashboard["assets"]["total_groups"], 0);
    assert_eq!(dashboard["assets"]["limit"], 50);
    assert_eq!(dashboard["transfers"]["total_count"], 0);
    assert!(dashboard["paths"]["state_dir"]
        .as_str()
        .unwrap()
        .contains("state"));
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

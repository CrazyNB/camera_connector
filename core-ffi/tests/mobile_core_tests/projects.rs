use super::*;

#[test]
fn mobile_core_persists_user_marks_and_filters_asset_groups_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Client Selects").unwrap();
    let project_id = project.project_id.clone();

    service
        .record_project_transfer(
            &project_id,
            completed_transfer("ftp:favorite", "DCIM/100/KEEP_0001.JPG", 10),
        )
        .expect("transfer should record");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project_id.clone(), "{}".to_string(), 0, 25)
            .expect("asset page should query"),
    )
    .unwrap();
    let group_id = page["groups"][0]["group_id"].as_str().unwrap().to_string();

    let marks: Value = serde_json::from_str(
        &core
            .set_asset_group_user_marks_json(
                project_id.clone(),
                group_id.clone(),
                r#"{"favorite":true}"#.to_string(),
            )
            .expect("user mark should save"),
    )
    .unwrap();
    assert_eq!(marks["favorite"], true);
    assert_eq!(marks["marked"], false);

    let favorites: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project_id, r#"{"favorite":true}"#.to_string(), 0, 25)
            .expect("favorite page should query"),
    )
    .unwrap();

    assert_eq!(favorites["total_groups"], 1);
    assert_eq!(favorites["groups"][0]["group_id"], group_id);
    assert_eq!(favorites["groups"][0]["user_marks"]["favorite"], true);
}

#[test]
fn mobile_core_creates_lan_share_and_sets_guest_mark_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile LAN").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:lan", "DCIM/100/IMG_7001.JPG", 10),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let session: Value = serde_json::from_str(
        &core
            .create_lan_share_session_json(
                project.project_id.clone(),
                r#"{"collection":"all","sort":"latest_received"}"#.to_string(),
                "Client link".to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(session["project_id"], project.project_id);
    assert_eq!(session["title"], "Client link");
    let token = session["token"].as_str().unwrap();

    let page: Value = serde_json::from_str(
        &core
            .lan_share_asset_group_page_json(token.to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let group_id = page["groups"][0]["group_id"].as_str().unwrap();

    let mark: Value = serde_json::from_str(
        &core
            .set_lan_share_guest_mark_json(
                token.to_string(),
                group_id.to_string(),
                r#"{"guest_mark":"reject"}"#.to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(mark["guest_mark"], "reject");
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
    let project: Value = serde_json::from_str(
        &core
            .create_project_json("Account Dashboard".to_string())
            .unwrap(),
    )
    .unwrap();
    let project_id = project["project_id"].as_str().unwrap().to_string();
    core.set_active_project_json(project_id.clone()).unwrap();
    let dashboard: Value =
        serde_json::from_str(&core.project_dashboard_json(project_id, 0, 25).unwrap()).unwrap();
    assert_eq!(dashboard["accounts"].as_array().unwrap().len(), 0);
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

    let deleted_json = core.delete_project_json(project_id.clone()).unwrap();
    let deleted: Value = serde_json::from_str(&deleted_json).unwrap();
    assert_eq!(deleted["project_id"], project_id);
    assert_eq!(deleted["deleted"], true);
    let active_after_delete: Value =
        serde_json::from_str(&core.active_project_json().unwrap()).unwrap();
    assert!(active_after_delete.is_null());
    let listed_after_delete: Value =
        serde_json::from_str(&core.list_projects_json().unwrap()).unwrap();
    assert!(listed_after_delete.as_array().unwrap().is_empty());
}

#[test]
fn mobile_core_exposes_project_capabilities_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let created: Value = serde_json::from_str(
        &core
            .create_project_json("Client Capability Shoot".to_string())
            .unwrap(),
    )
    .unwrap();
    let archived: Value = serde_json::from_str(
        &core
            .archive_project_json(created["project_id"].as_str().unwrap().to_string())
            .unwrap(),
    )
    .unwrap();

    assert_eq!(created["kind"], "User");
    assert_eq!(created["capabilities"]["can_archive"], true);
    assert_eq!(created["capabilities"]["can_rename"], true);
    assert_eq!(created["capabilities"]["can_accept_moved_groups"], true);
    assert_eq!(archived["capabilities"]["can_be_active_project"], false);
    assert_eq!(archived["capabilities"]["can_archive"], false);
    assert_eq!(archived["capabilities"]["can_restore"], true);
}

#[test]
fn mobile_core_renames_project_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let created: Value = serde_json::from_str(
        &core
            .create_project_json("Untitled Mobile Shoot".into())
            .unwrap(),
    )
    .unwrap();
    let project_id = created["project_id"].as_str().unwrap().to_string();
    core.set_active_project_json(project_id.clone()).unwrap();

    let renamed_json = core
        .rename_project_json(project_id.clone(), "Client Mobile Shoot".into())
        .unwrap();
    let active_json = core.active_project_json().unwrap();

    let renamed: Value = serde_json::from_str(&renamed_json).unwrap();
    let active: Value = serde_json::from_str(&active_json).unwrap();
    assert_eq!(renamed["project_id"], project_id);
    assert_eq!(renamed["name"], "Client Mobile Shoot");
    assert_eq!(renamed["slug"], "client-mobile-shoot");
    assert_eq!(active["project_id"], project_id);
    assert_eq!(active["name"], "Client Mobile Shoot");
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
    assert_eq!(dashboard["global_assets"]["photo_count"], 0);
    assert_eq!(dashboard["global_assets"]["storage_bytes"], 0);
    assert_eq!(dashboard["transfers"]["total_count"], 0);
    assert!(dashboard["paths"]["state_dir"]
        .as_str()
        .unwrap()
        .contains("state"));
}

use camera_connector_core::{
    AssetGroupQuery, CameraConnectorService, PublishTransferMetadata, StoredObjectLocation,
    TransferRecord, TransferStatus,
};
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
            defer_publish: Some(true),
        })
        .unwrap();

    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["protocol"], "Sftp");
    assert_eq!(value["bind_host"], "0.0.0.0");
    assert_eq!(value["ftp_port"], 2121);
    assert_eq!(value["sftp_port"], 2222);
    assert_eq!(value["source_name"], "Studio Camera");
    assert_eq!(value["defer_publish"], true);
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
    let project: Value = serde_json::from_str(&core.ensure_active_project_json().unwrap()).unwrap();
    let project_id = project["project_id"].as_str().unwrap().to_string();
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
    assert_eq!(dashboard["transfers"]["total_count"], 0);
    assert!(dashboard["paths"]["state_dir"]
        .as_str()
        .unwrap()
        .contains("state"));
}

#[test]
fn mobile_core_claims_and_completes_publish_queue_items_as_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Publisher").unwrap();
    let store = service.storage_store().unwrap();
    let item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:mobile-publish",
            "staging/mobile-publish.tmp",
            "IMG_6001.JPG",
            42,
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let claimed_json = core.claim_next_publish_item_json().unwrap();
    let claimed: Value = serde_json::from_str(&claimed_json).unwrap();
    assert_eq!(claimed["queue_id"], item.queue_id);
    assert_eq!(claimed["state"], "Publishing");
    assert_eq!(claimed["last_error"], Value::Null);
    let empty_after_claim: Value =
        serde_json::from_str(&core.claim_next_publish_item_json().unwrap()).unwrap();
    assert!(empty_after_claim.is_null());

    let completed_json = core
        .mark_publish_completed_json(item.queue_id.clone())
        .unwrap();
    let completed: Value = serde_json::from_str(&completed_json).unwrap();
    assert_eq!(completed["queue_id"], item.queue_id);
    assert_eq!(completed["completed"], true);
    let failed_item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:mobile-failed",
            "staging/mobile-failed.tmp",
            "IMG_6002.JPG",
            43,
        )
        .unwrap();
    let claimed_failed: Value =
        serde_json::from_str(&core.claim_next_publish_item_json().unwrap()).unwrap();
    assert_eq!(claimed_failed["queue_id"], failed_item.queue_id);
    let failed_json = core
        .mark_publish_failed_json(
            failed_item.queue_id.clone(),
            "permission revoked".to_string(),
        )
        .unwrap();
    let failed: Value = serde_json::from_str(&failed_json).unwrap();
    assert_eq!(failed["queue_id"], failed_item.queue_id);
    assert_eq!(failed["failed"], true);
    let dashboard: Value = serde_json::from_str(
        &core
            .project_dashboard_json(project.project_id.clone(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(dashboard["publish_queue"]["completed_count"], 1);
    assert_eq!(dashboard["publish_queue"]["failed_count"], 1);
    assert_eq!(dashboard["publish_queue"]["pending_count"], 1);
}

#[test]
fn mobile_core_releases_failed_publish_retries_for_project() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Retry").unwrap();
    let store = service.storage_store().unwrap();
    let item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:mobile-retry",
            "staging/mobile-retry.tmp",
            "IMG_6003.JPG",
            44,
        )
        .unwrap();
    store
        .mark_publish_failed(&item.queue_id, "permission revoked")
        .unwrap();

    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let deferred: Value =
        serde_json::from_str(&core.claim_next_publish_item_json().unwrap()).unwrap();
    assert!(deferred.is_null());

    let released: Value = serde_json::from_str(
        &core
            .release_failed_publish_retries_json(project.project_id.clone())
            .unwrap(),
    )
    .unwrap();
    let claimed: Value =
        serde_json::from_str(&core.claim_next_publish_item_json().unwrap()).unwrap();

    assert_eq!(released["project_id"], project.project_id);
    assert_eq!(released["released_count"], 1);
    assert_eq!(claimed["queue_id"], item.queue_id);
    assert_eq!(claimed["state"], "Publishing");
    assert_eq!(claimed["last_error"], Value::Null);
    assert_eq!(claimed["next_attempt_at_ms"], Value::Null);
}

#[test]
fn mobile_core_completes_publish_with_platform_location_into_project_assets() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Platform Publisher").unwrap();
    let store = service.storage_store().unwrap();
    let item = store
        .enqueue_publish_with_metadata(
            &project.project_id,
            "ftp:mobile-platform-publish",
            "staging/mobile-platform-publish.tmp",
            "IMG_6010.JPG",
            42,
            PublishTransferMetadata {
                protocol: "ftp".to_string(),
                original_path: "DCIM/100/IMG_6010.JPG".to_string(),
                username: Some("z5".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Studio Z5".to_string()),
                started_at_ms: 42,
            },
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let claimed: Value =
        serde_json::from_str(&core.claim_next_publish_item_json().unwrap()).unwrap();
    assert_eq!(claimed["queue_id"], item.queue_id);

    let completed_json = core
        .complete_publish_json(
            item.queue_id.clone(),
            "IMG_6010.JPG".to_string(),
            "media_uri".to_string(),
            "content://media/external/images/media/6010".to_string(),
        )
        .unwrap();
    let completed: Value = serde_json::from_str(&completed_json).unwrap();
    assert_eq!(completed["transfer_id"], "ftp:mobile-platform-publish");
    assert_eq!(completed["final_location"]["kind"], "media_uri");

    let dashboard: Value = serde_json::from_str(
        &core
            .project_dashboard_json(project.project_id.clone(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(dashboard["assets"]["total_groups"], 1);
    assert_eq!(
        dashboard["assets"]["groups"][0]["primary"]["storage_location"]["kind"],
        "media_uri"
    );
    assert_eq!(dashboard["publish_queue"]["completed_count"], 1);
}

#[test]
fn mobile_core_returns_project_group_assets_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Members").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-jpg", "DCIM/100/IMG_5001.JPG", 20),
        )
        .unwrap();
    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .unwrap();
    let group_id = page.groups[0].group_id.clone().unwrap();

    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let json = core
        .project_group_assets_json(project.project_id.clone(), group_id.clone())
        .unwrap();
    let value: Value = serde_json::from_str(&json).unwrap();
    let assets = value.as_array().unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0]["project_id"], project.project_id);
    assert_eq!(assets[0]["group_id"], group_id);
    assert_eq!(assets[0]["final_filename"], "IMG_5001.JPG");

    let ambiguous_json = core
        .project_group_assets_json(project.project_id, "IMG_5001".to_string())
        .unwrap();
    let ambiguous: Value = serde_json::from_str(&ambiguous_json).unwrap();
    assert!(ambiguous.as_array().unwrap().is_empty());
}

#[test]
fn mobile_core_moves_project_group_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let source_project = service.create_project("Wrong Mobile Project").unwrap();
    let target_project = service.create_project("Correct Mobile Project").unwrap();
    service
        .record_project_transfer(
            &source_project.project_id,
            completed_transfer("ftp:mobile-move-jpg", "DCIM/100/IMG_6101.JPG", 20),
        )
        .unwrap();
    service
        .record_project_transfer(
            &source_project.project_id,
            completed_transfer("ftp:mobile-move-raw", "DCIM/100/IMG_6101.NEF", 21),
        )
        .unwrap();
    let source_page = service
        .project_asset_group_page_with_query(
            &source_project.project_id,
            AssetGroupQuery::default(),
            0,
            25,
        )
        .unwrap();
    let group_id = source_page.groups[0].group_id.clone().unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let moved_json = core
        .move_project_group_json(
            source_project.project_id.clone(),
            group_id.clone(),
            target_project.project_id.clone(),
        )
        .unwrap();
    let source_dashboard: Value = serde_json::from_str(
        &core
            .project_dashboard_json(source_project.project_id.clone(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let target_dashboard: Value = serde_json::from_str(
        &core
            .project_dashboard_json(target_project.project_id.clone(), 0, 25)
            .unwrap(),
    )
    .unwrap();

    let moved: Value = serde_json::from_str(&moved_json).unwrap();
    assert_eq!(moved["project_id"], target_project.project_id);
    assert_eq!(moved["display_key"], "IMG_6101");
    assert_eq!(moved["member_count"], 2);
    assert_eq!(source_dashboard["assets"]["total_groups"], 0);
    assert_eq!(target_dashboard["assets"]["total_groups"], 1);
    assert_eq!(target_dashboard["assets"]["summary"]["asset_count"], 2);
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

fn completed_transfer(
    transfer_id: &str,
    original_path: &str,
    completed_at_ms: i64,
) -> TransferRecord {
    let final_filename = original_path
        .rsplit('/')
        .next()
        .expect("filename should exist")
        .to_string();
    TransferRecord {
        transfer_id: transfer_id.to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: original_path.to_string(),
        final_filename: final_filename.clone(),
        final_path: None,
        final_location: Some(StoredObjectLocation::local_path(final_filename)),
        size_bytes: 100,
        username: Some("z5".to_string()),
        remote_addr: Some("192.168.137.56".to_string()),
        source_name: Some("Studio Z5".to_string()),
        started_at_ms: completed_at_ms - 1,
        completed_at_ms: Some(completed_at_ms),
        error: None,
    }
}

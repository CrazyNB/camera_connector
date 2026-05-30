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
fn mobile_core_lists_builtin_strategy_profiles_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let value: Value = serde_json::from_str(&core.strategy_profiles_json().unwrap()).unwrap();
    let ids = value
        .as_array()
        .unwrap()
        .iter()
        .map(|profile| profile["profile_id"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        vec!["general", "conservative", "portrait", "action", "landscape"]
    );
    assert!(value
        .as_array()
        .unwrap()
        .iter()
        .all(|profile| profile["built_in"] == true));
}

#[test]
fn mobile_core_saves_custom_strategy_profile_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    let mut profile: Value = serde_json::from_str::<Value>(&core.strategy_profiles_json().unwrap())
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|profile| profile["profile_id"] == "general")
        .unwrap()
        .clone();
    profile["profile_id"] = Value::String("custom-sharp".to_string());
    profile["name"] = Value::String("Custom Sharp".to_string());
    profile["weights"]["sharpness"] = Value::from(0.56);

    let saved: Value = serde_json::from_str(
        &core
            .save_strategy_profile_json(profile.to_string())
            .expect("custom profile should save"),
    )
    .unwrap();
    let profiles: Value = serde_json::from_str(&core.strategy_profiles_json().unwrap()).unwrap();

    assert_eq!(saved["profile_id"], "custom-sharp");
    assert_eq!(saved["built_in"], false);
    assert!(profiles
        .as_array()
        .unwrap()
        .iter()
        .any(|profile| profile["profile_id"] == "custom-sharp"
            && profile["weights"]["sharpness"] == 0.56));
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
fn mobile_core_returns_project_asset_group_page_json_with_query() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Query").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-query-jpg", "DCIM/100/IMG_7001.JPG", 20),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-query-raw", "DCIM/100/IMG_7001.NEF", 21),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:mobile-query-video", "DCIM/100/VID_7002.MP4", 22),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let raw_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(
                project.project_id.clone(),
                r#"{"role":"raw"}"#.to_string(),
                0,
                25,
            )
            .unwrap(),
    )
    .unwrap();
    let video_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(
                project.project_id,
                r#"{"role":"video"}"#.to_string(),
                0,
                25,
            )
            .unwrap(),
    )
    .unwrap();

    assert_eq!(raw_page["total_groups"], 1);
    assert_eq!(raw_page["groups"][0]["group_key"], "IMG_7001");
    assert_eq!(video_page["total_groups"], 1);
    assert_eq!(video_page["groups"][0]["group_key"], "VID_7002");
}

#[test]
fn mobile_core_drain_analysis_jobs_exposes_burst_summary() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Burst").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:burst-1", "DCIM/100/IMG_7101.JPG", 1000),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:burst-2", "DCIM/100/IMG_7102.JPG", 1100),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));

    let summary: Value = serde_json::from_str(&core.drain_analysis_jobs_json(10).unwrap()).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(summary["claimed_count"], 2);
    assert_eq!(summary["completed_count"], 2);
    assert_eq!(page["groups"][0]["burst"]["member_count"], 2);
}

#[test]
fn mobile_core_scores_preview_and_recommends_burst_group_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Recommend").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:recommend-1", "DCIM/100/IMG_7201.JPG", 1000),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:recommend-2", "DCIM/100/IMG_7202.JPG", 1100),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let first = &page["groups"][0];
    let second = &page["groups"][1];
    let burst_id = first["burst"]["burst_group_id"].as_str().unwrap();
    core.score_asset_group_preview_json(
        first["group_id"].as_str().unwrap().to_string(),
        flat_sample_json(16, 16, 128),
        "local-v1".to_string(),
    )
    .unwrap();
    core.score_asset_group_preview_json(
        second["group_id"].as_str().unwrap().to_string(),
        checkerboard_sample_json(16, 16),
        "local-v1".to_string(),
    )
    .unwrap();

    let recommendation: Value = serde_json::from_str(
        &core
            .recommend_burst_group_json(burst_id.to_string(), None)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(
        recommendation["best_asset_group_id"].as_str(),
        second["group_id"].as_str()
    );
    let review_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(
                project.project_id.clone(),
                serde_json::json!({
                    "review_queue": "unconfirmed_best",
                    "strategy_profile_id": "general",
                    "sort": "group_best_score"
                })
                .to_string(),
                0,
                25,
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(review_page["total_groups"].as_u64(), Some(1));
    assert_eq!(
        review_page["groups"][0]["group_id"].as_str(),
        recommendation["best_asset_group_id"].as_str(),
    );

    let filtered_page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(
                project.project_id,
                serde_json::json!({
                    "score_min": 80.0,
                    "sort": "group_best_score"
                })
                .to_string(),
                0,
                25,
            )
            .unwrap(),
    )
    .unwrap();
    let filtered_groups = filtered_page["groups"].as_array().unwrap();
    assert_eq!(2, filtered_groups.len());
    assert!(filtered_groups
        .iter()
        .all(|group| group["burst"]["best_score"].as_f64().unwrap_or_default() >= 0.8));
}

#[test]
fn mobile_core_returns_review_queue_summary_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Review Summary").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:review-summary-1", "DCIM/100/IMG_7301.JPG", 1000),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:review-summary-2", "DCIM/100/IMG_7302.JPG", 1100),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:review-summary-single", "DCIM/100/IMG_9301.JPG", 5000),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let burst_members = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|group| group.get("burst").is_some())
        .collect::<Vec<_>>();
    let single = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group.get("burst").is_none())
        .unwrap();
    let burst_id = burst_members[0]["burst"]["burst_group_id"]
        .as_str()
        .unwrap();
    core.score_asset_group_preview_json(
        burst_members[0]["group_id"].as_str().unwrap().to_string(),
        flat_sample_json(16, 16, 128),
        "local-v1".to_string(),
    )
    .unwrap();
    core.score_asset_group_preview_json(
        burst_members[1]["group_id"].as_str().unwrap().to_string(),
        checkerboard_sample_json(16, 16),
        "local-v1".to_string(),
    )
    .unwrap();
    core.recommend_burst_group_json(burst_id.to_string(), None)
        .unwrap();
    core.score_asset_group_preview_json(
        single["group_id"].as_str().unwrap().to_string(),
        unsupported_sample_json(),
        "local-v1".to_string(),
    )
    .unwrap();

    let summary: Value = serde_json::from_str(
        &core
            .review_queue_summary_json(project.project_id, None)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(summary["total_units"], 2);
    assert_eq!(summary["unconfirmed_best_count"], 1);
    assert_eq!(summary["low_score_candidate_count"], 1);
    assert_eq!(summary["unsupported_count"], 1);
    assert_eq!(summary["needs_review_count"], 1);
}

#[test]
fn mobile_core_accepts_recommended_best_json() {
    let fixture = recommended_burst_fixture("Mobile Accept Decision");
    let before: Value = serde_json::from_str(
        &fixture
            .core
            .review_queue_summary_json(fixture.project_id.clone(), None)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(before["unconfirmed_best_count"], 1);

    let accepted: Value = serde_json::from_str(
        &fixture
            .core
            .accept_recommended_best_json(fixture.burst_id.to_string(), None)
            .unwrap(),
    )
    .unwrap();
    let after: Value = serde_json::from_str(
        &fixture
            .core
            .review_queue_summary_json(fixture.project_id.clone(), None)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(accepted["status"], "Accepted");
    assert_eq!(after["unconfirmed_best_count"], 0);
    assert_eq!(after["needs_review_count"], 0);
    let selects: Value = serde_json::from_str(
        &fixture
            .core
            .project_selects_asset_group_page_json(fixture.project_id, None, 0, 25)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(selects["total_groups"], 1);
    assert_eq!(
        selects["groups"][0]["group_id"].as_str(),
        Some(fixture.best_group_id.as_str()),
    );
}

#[test]
fn mobile_core_restores_automatic_recommendation_json() {
    let fixture = recommended_burst_fixture("Mobile Undo Decision");
    fixture
        .core
        .accept_recommended_best_json(fixture.burst_id.to_string(), None)
        .unwrap();
    let accepted: Value = serde_json::from_str(
        &fixture
            .core
            .review_queue_summary_json(fixture.project_id.clone(), None)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(accepted["unconfirmed_best_count"], 0);

    let restored: Value = serde_json::from_str(
        &fixture
            .core
            .restore_automatic_recommendation_json(fixture.burst_id.to_string(), None)
            .unwrap(),
    )
    .unwrap();
    let after: Value = serde_json::from_str(
        &fixture
            .core
            .review_queue_summary_json(fixture.project_id, None)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(restored["status"], "Ready");
    assert_eq!(after["unconfirmed_best_count"], 1);
    assert_eq!(after["needs_review_count"], 0);
}

#[test]
fn mobile_core_overrides_recommended_best_json() {
    let fixture = recommended_burst_fixture("Mobile Override Decision");

    let overridden: Value = serde_json::from_str(
        &fixture
            .core
            .override_recommended_best_json(
                fixture.burst_id.to_string(),
                fixture.alternate_group_id.to_string(),
                None,
            )
            .unwrap(),
    )
    .unwrap();
    let summary: Value = serde_json::from_str(
        &fixture
            .core
            .review_queue_summary_json(fixture.project_id.clone(), None)
            .unwrap(),
    )
    .unwrap();
    let selects: Value = serde_json::from_str(
        &fixture
            .core
            .project_selects_asset_group_page_json(fixture.project_id, None, 0, 25)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(overridden["status"], "UserOverridden");
    assert_eq!(
        overridden["best_asset_group_id"].as_str(),
        Some(fixture.alternate_group_id.as_str()),
    );
    assert_eq!(summary["user_overridden_count"], 1);
    assert_eq!(
        selects["groups"][0]["group_id"].as_str(),
        Some(fixture.alternate_group_id.as_str()),
    );
}

#[test]
fn mobile_core_marks_burst_needs_review_json() {
    let fixture = recommended_burst_fixture("Mobile Review Decision");
    let before: Value = serde_json::from_str(
        &fixture
            .core
            .review_queue_summary_json(fixture.project_id.clone(), None)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(before["unconfirmed_best_count"], 1);

    let marked: Value = serde_json::from_str(
        &fixture
            .core
            .mark_burst_needs_review_json(fixture.burst_id.to_string(), None)
            .unwrap(),
    )
    .unwrap();
    let after: Value = serde_json::from_str(
        &fixture
            .core
            .review_queue_summary_json(fixture.project_id, None)
            .unwrap(),
    )
    .unwrap();

    assert_eq!(marked["status"], "NeedsReview");
    assert_eq!(after["unconfirmed_best_count"], 0);
    assert_eq!(after["needs_review_count"], 1);
}

#[test]
fn mobile_core_applies_extended_review_decisions_json() {
    let clear_fixture = recommended_burst_fixture("Mobile Clear Decision");
    let cleared: Value = serde_json::from_str(
        &clear_fixture
            .core
            .clear_recommendation_json(clear_fixture.burst_id.to_string(), None)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(cleared["status"], "Cleared");
    assert!(cleared["best_asset_group_id"].is_null());

    let keep_fixture = recommended_burst_fixture("Mobile Keep All Decision");
    let kept: Value = serde_json::from_str(
        &keep_fixture
            .core
            .keep_all_candidates_json(keep_fixture.burst_id.to_string(), None)
            .unwrap(),
    )
    .unwrap();
    let keep_summary: Value = serde_json::from_str(
        &keep_fixture
            .core
            .review_queue_summary_json(keep_fixture.project_id, None)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(kept["status"], "KeptAll");
    assert_eq!(keep_summary["unconfirmed_best_count"], 0);

    let low_fixture = low_score_burst_fixture("Mobile Hide Low Score Decision");
    let hidden: Value = serde_json::from_str(
        &low_fixture
            .core
            .hide_low_score_candidates_json(low_fixture.burst_id.to_string(), None)
            .unwrap(),
    )
    .unwrap();
    let low_summary: Value = serde_json::from_str(
        &low_fixture
            .core
            .review_queue_summary_json(low_fixture.project_id, None)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(hidden["status"], "LowScoreHidden");
    assert_eq!(low_summary["low_score_candidate_count"], 0);
}

#[test]
fn mobile_core_splits_burst_member_json() {
    let fixture = three_member_recommended_burst_fixture("Mobile Split Decision");

    let updated: Value = serde_json::from_str(
        &fixture
            .core
            .split_burst_member_json(
                fixture.burst_id.to_string(),
                fixture.alternate_group_id.to_string(),
            )
            .unwrap(),
    )
    .unwrap();
    let summary: Value = serde_json::from_str(
        &fixture
            .core
            .review_queue_summary_json(fixture.project_id.clone(), None)
            .unwrap(),
    )
    .unwrap();
    let page: Value = serde_json::from_str(
        &fixture
            .core
            .project_asset_group_page_json(fixture.project_id, "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let split_group = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["group_id"].as_str() == Some(fixture.alternate_group_id.as_str()))
        .expect("split group should remain visible");

    assert_eq!(updated["member_count"], 2);
    assert_eq!(updated["recommendation_status"], "pending");
    assert_eq!(summary["unconfirmed_best_count"], 0);
    assert_eq!(summary["pending_count"], 2);
    assert!(split_group.get("burst").is_none());
}

#[test]
fn mobile_core_merges_burst_member_json() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Mobile Merge Decision").unwrap();
    for (transfer_id, path, completed_at_ms) in [
        ("ftp:merge-a-1", "DCIM/240/IMG_8241.JPG", 1000),
        ("ftp:merge-a-2", "DCIM/240/IMG_8242.JPG", 1100),
        ("ftp:merge-b-1", "DCIM/241/IMG_9241.JPG", 5000),
        ("ftp:merge-b-2", "DCIM/241/IMG_9242.JPG", 5100),
    ] {
        service
            .record_project_transfer(
                &project.project_id,
                completed_transfer(transfer_id, path, completed_at_ms),
            )
            .unwrap();
    }
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let target = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| {
            group["primary"]["original_path"]
                .as_str()
                .map(|path| path.ends_with("IMG_8241.JPG"))
                .unwrap_or(false)
        })
        .expect("target member should exist");
    let source = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|group| {
            group["primary"]["original_path"]
                .as_str()
                .map(|path| path.ends_with("IMG_9241.JPG"))
                .unwrap_or(false)
        })
        .expect("source member should exist");
    let target_burst_id = target["burst"]["burst_group_id"].as_str().unwrap();
    let source_group_id = source["group_id"].as_str().unwrap();

    let merged: Value = serde_json::from_str(
        &core
            .merge_burst_member_json(target_burst_id.to_string(), source_group_id.to_string())
            .unwrap(),
    )
    .unwrap();

    assert_eq!(merged["member_count"], 4);
    assert_eq!(merged["recommendation_status"], "pending");
    assert_eq!(merged["user_override_state"], "merge");
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

fn flat_sample_json(width: usize, height: usize, value: u8) -> String {
    serde_json::json!({
        "width": width,
        "height": height,
        "luma": vec![value; width * height],
        "preview_source": "test"
    })
    .to_string()
}

fn checkerboard_sample_json(width: usize, height: usize) -> String {
    let mut luma = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            luma.push(if (x + y) % 2 == 0 { 0u8 } else { 255u8 });
        }
    }
    serde_json::json!({
        "width": width,
        "height": height,
        "luma": luma,
        "preview_source": "test"
    })
    .to_string()
}

fn unsupported_sample_json() -> String {
    serde_json::json!({
        "width": 0,
        "height": 0,
        "luma": [],
        "preview_source": "missing"
    })
    .to_string()
}

struct MobileReviewFixture {
    _temp: tempfile::TempDir,
    core: MobileCore,
    project_id: String,
    burst_id: String,
    best_group_id: String,
    alternate_group_id: String,
}

fn recommended_burst_fixture(project_name: &str) -> MobileReviewFixture {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project(project_name).unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:decision-1", "DCIM/100/IMG_7401.JPG", 1000),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:decision-2", "DCIM/100/IMG_7402.JPG", 1100),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let burst_members = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|group| group.get("burst").is_some())
        .collect::<Vec<_>>();
    let burst_id = burst_members[0]["burst"]["burst_group_id"]
        .as_str()
        .unwrap()
        .to_string();
    for member in burst_members {
        core.score_asset_group_preview_json(
            member["group_id"].as_str().unwrap().to_string(),
            checkerboard_sample_json(16, 16),
            "local-v1".to_string(),
        )
        .unwrap();
    }
    let recommendation: Value = serde_json::from_str(
        &core
            .recommend_burst_group_json(burst_id.clone(), None)
            .unwrap(),
    )
    .unwrap();
    let best_group_id = recommendation["best_asset_group_id"]
        .as_str()
        .unwrap()
        .to_string();
    let alternate_group_id = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|group| group.get("burst").is_some())
        .filter_map(|group| group["group_id"].as_str())
        .find(|group_id| *group_id != best_group_id)
        .unwrap()
        .to_string();

    MobileReviewFixture {
        _temp: temp,
        core,
        project_id: project.project_id,
        burst_id,
        best_group_id,
        alternate_group_id,
    }
}

fn low_score_burst_fixture(project_name: &str) -> MobileReviewFixture {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project(project_name).unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:low-decision-1", "DCIM/100/IMG_7501.JPG", 1000),
        )
        .unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:low-decision-2", "DCIM/100/IMG_7502.JPG", 1100),
        )
        .unwrap();
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let burst_members = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|group| group.get("burst").is_some())
        .collect::<Vec<_>>();
    let burst_id = burst_members[0]["burst"]["burst_group_id"]
        .as_str()
        .unwrap()
        .to_string();
    let first_group_id = burst_members[0]["group_id"].as_str().unwrap().to_string();
    let second_group_id = burst_members[1]["group_id"].as_str().unwrap().to_string();
    core.score_asset_group_preview_json(
        first_group_id.clone(),
        flat_sample_json(16, 16, 128),
        "local-v1".to_string(),
    )
    .unwrap();
    core.score_asset_group_preview_json(
        second_group_id.clone(),
        checkerboard_sample_json(16, 16),
        "local-v1".to_string(),
    )
    .unwrap();
    let recommendation: Value = serde_json::from_str(
        &core
            .recommend_burst_group_json(burst_id.clone(), None)
            .unwrap(),
    )
    .unwrap();
    let best_group_id = recommendation["best_asset_group_id"]
        .as_str()
        .unwrap()
        .to_string();
    let alternate_group_id = if best_group_id == first_group_id {
        second_group_id
    } else {
        first_group_id
    };

    MobileReviewFixture {
        _temp: temp,
        core,
        project_id: project.project_id,
        burst_id,
        best_group_id,
        alternate_group_id,
    }
}

fn three_member_recommended_burst_fixture(project_name: &str) -> MobileReviewFixture {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project(project_name).unwrap();
    for (transfer_id, path, completed_at_ms) in [
        ("ftp:three-decision-1", "DCIM/100/IMG_7601.JPG", 1000),
        ("ftp:three-decision-2", "DCIM/100/IMG_7602.JPG", 1100),
        ("ftp:three-decision-3", "DCIM/100/IMG_7603.JPG", 1200),
    ] {
        service
            .record_project_transfer(
                &project.project_id,
                completed_transfer(transfer_id, path, completed_at_ms),
            )
            .unwrap();
    }
    let core = MobileCore::new(Some(config_path.to_string_lossy().into_owned()));
    core.drain_analysis_jobs_json(10).unwrap();
    let page: Value = serde_json::from_str(
        &core
            .project_asset_group_page_json(project.project_id.clone(), "{}".to_string(), 0, 25)
            .unwrap(),
    )
    .unwrap();
    let burst_members = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|group| group.get("burst").is_some())
        .collect::<Vec<_>>();
    let burst_id = burst_members[0]["burst"]["burst_group_id"]
        .as_str()
        .unwrap()
        .to_string();
    for member in burst_members {
        core.score_asset_group_preview_json(
            member["group_id"].as_str().unwrap().to_string(),
            checkerboard_sample_json(16, 16),
            "local-v1".to_string(),
        )
        .unwrap();
    }
    let recommendation: Value = serde_json::from_str(
        &core
            .recommend_burst_group_json(burst_id.clone(), None)
            .unwrap(),
    )
    .unwrap();
    let best_group_id = recommendation["best_asset_group_id"]
        .as_str()
        .unwrap()
        .to_string();
    let alternate_group_id = page["groups"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|group| group.get("burst").is_some())
        .filter_map(|group| group["group_id"].as_str())
        .find(|group_id| *group_id != best_group_id)
        .unwrap()
        .to_string();

    MobileReviewFixture {
        _temp: temp,
        core,
        project_id: project.project_id,
        burst_id,
        best_group_id,
        alternate_group_id,
    }
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

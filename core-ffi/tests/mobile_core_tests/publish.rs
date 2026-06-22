use super::*;

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

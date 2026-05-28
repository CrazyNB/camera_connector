use std::ffi::{CStr, CString};

use serde_json::Value;

use camera_connector_core::{
    AssetGroupQuery, CameraConnectorService, PublishTransferMetadata, StoredObjectLocation,
    TransferRecord, TransferStatus,
};
use camera_connector_ffi::{
    camera_connector_mobile_core_active_project_json,
    camera_connector_mobile_core_archive_project_json,
    camera_connector_mobile_core_claim_next_publish_item_json,
    camera_connector_mobile_core_complete_publish_json, camera_connector_mobile_core_create,
    camera_connector_mobile_core_create_project_json, camera_connector_mobile_core_destroy,
    camera_connector_mobile_core_ensure_active_project_json,
    camera_connector_mobile_core_free_string, camera_connector_mobile_core_list_projects_json,
    camera_connector_mobile_core_mark_publish_completed_json,
    camera_connector_mobile_core_mark_publish_failed_json,
    camera_connector_mobile_core_move_project_group_json,
    camera_connector_mobile_core_project_dashboard_json,
    camera_connector_mobile_core_project_group_assets_json,
    camera_connector_mobile_core_release_failed_publish_retries_json,
    camera_connector_mobile_core_remove_device_account_json,
    camera_connector_mobile_core_restore_project_json,
    camera_connector_mobile_core_save_device_account_json,
    camera_connector_mobile_core_save_receiver_settings_json,
    camera_connector_mobile_core_set_active_project_json,
    camera_connector_mobile_core_start_receiver_json,
    camera_connector_mobile_core_stop_receiver_json,
};

#[test]
fn ffi_saves_account_and_returns_success_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let username = CString::new("camera01").unwrap();
    let password = CString::new("secret").unwrap();
    let device_name = CString::new("Camera 01").unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    assert!(!core.is_null());

    let response_ptr = unsafe {
        camera_connector_mobile_core_save_device_account_json(
            core,
            username.as_ptr(),
            password.as_ptr(),
            device_name.as_ptr(),
        )
    };
    let response = take_ffi_string(response_ptr);
    unsafe { camera_connector_mobile_core_destroy(core) };

    let value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(value["ok"], true);
    assert_eq!(value["value"]["username"], "camera01");
    assert_eq!(value["value"]["device_name"], "Camera 01");
    assert_eq!(value["value"]["password_configured"], true);
    assert!(!response.contains("secret"));
}

#[test]
fn ffi_removes_account_and_returns_success_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let username = CString::new("camera01").unwrap();
    let password = CString::new("secret").unwrap();
    let device_name = CString::new("Camera 01").unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    assert!(!core.is_null());
    take_ffi_string(unsafe {
        camera_connector_mobile_core_save_device_account_json(
            core,
            username.as_ptr(),
            password.as_ptr(),
            device_name.as_ptr(),
        )
    });

    let response = take_ffi_string(unsafe {
        camera_connector_mobile_core_remove_device_account_json(core, username.as_ptr())
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(value["ok"], true);
    assert_eq!(value["value"]["username"], "camera01");
    assert_eq!(value["value"]["removed"], true);
}

#[test]
fn ffi_saves_receiver_settings_from_json_patch() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let output_dir = temp.path().join("output");
    let state_dir = temp.path().join("state");
    let patch = CString::new(format!(
        r#"{{
            "protocol":"sftp",
            "bind_host":"0.0.0.0",
            "ftp_port":2121,
            "sftp_port":2222,
            "output_dir":{},
            "state_dir":{},
            "advertised_host":"192.168.137.1",
            "source_name":"Studio Camera",
            "defer_publish":true
        }}"#,
        serde_json::to_string(&output_dir.to_string_lossy()).unwrap(),
        serde_json::to_string(&state_dir.to_string_lossy()).unwrap(),
    ))
    .unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let response_ptr =
        unsafe { camera_connector_mobile_core_save_receiver_settings_json(core, patch.as_ptr()) };
    let response = take_ffi_string(response_ptr);
    unsafe { camera_connector_mobile_core_destroy(core) };

    let value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(value["ok"], true);
    assert_eq!(value["value"]["protocol"], "Sftp");
    assert_eq!(value["value"]["source_name"], "Studio Camera");
    assert_eq!(value["value"]["defer_publish"], true);
}

#[test]
fn ffi_returns_error_envelope_for_invalid_protocol() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let patch = CString::new(r#"{"protocol":"ftps"}"#).unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let response_ptr =
        unsafe { camera_connector_mobile_core_save_receiver_settings_json(core, patch.as_ptr()) };
    let response = take_ffi_string(response_ptr);
    unsafe { camera_connector_mobile_core_destroy(core) };

    let value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(value["ok"], false);
    assert!(value["error"]
        .as_str()
        .unwrap()
        .contains("invalid protocol: ftps"));
}

#[test]
fn ffi_manages_projects_with_envelopes() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let name = CString::new("Commercial Shoot").unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let created = take_ffi_string(unsafe {
        camera_connector_mobile_core_create_project_json(core, name.as_ptr())
    });
    let created: Value = serde_json::from_str(&created).unwrap();
    assert_eq!(created["ok"], true);
    assert_eq!(created["value"]["name"], "Commercial Shoot");
    let project_id = CString::new(created["value"]["project_id"].as_str().unwrap()).unwrap();

    let listed = take_ffi_string(unsafe { camera_connector_mobile_core_list_projects_json(core) });
    let listed: Value = serde_json::from_str(&listed).unwrap();
    assert_eq!(listed["ok"], true);
    assert_eq!(listed["value"].as_array().unwrap().len(), 1);

    let active = take_ffi_string(unsafe {
        camera_connector_mobile_core_set_active_project_json(core, project_id.as_ptr())
    });
    let active: Value = serde_json::from_str(&active).unwrap();
    assert_eq!(active["ok"], true);
    assert_eq!(active["value"]["project_id"], project_id.to_str().unwrap());

    let active_again =
        take_ffi_string(unsafe { camera_connector_mobile_core_active_project_json(core) });
    let active_again: Value = serde_json::from_str(&active_again).unwrap();
    assert_eq!(
        active_again["value"]["project_id"],
        project_id.to_str().unwrap()
    );

    let archived = take_ffi_string(unsafe {
        camera_connector_mobile_core_archive_project_json(core, project_id.as_ptr())
    });
    let archived: Value = serde_json::from_str(&archived).unwrap();
    assert_eq!(archived["ok"], true);
    assert_eq!(archived["value"]["status"], "Archived");

    let active_after_archive =
        take_ffi_string(unsafe { camera_connector_mobile_core_active_project_json(core) });
    let active_after_archive: Value = serde_json::from_str(&active_after_archive).unwrap();
    assert_eq!(active_after_archive["ok"], true);
    assert!(active_after_archive["value"].is_null());

    let select_archived = take_ffi_string(unsafe {
        camera_connector_mobile_core_set_active_project_json(core, project_id.as_ptr())
    });
    let select_archived: Value = serde_json::from_str(&select_archived).unwrap();
    assert_eq!(select_archived["ok"], false);
    assert!(select_archived["error"]
        .as_str()
        .unwrap()
        .contains("project archived"));

    let restored = take_ffi_string(unsafe {
        camera_connector_mobile_core_restore_project_json(core, project_id.as_ptr())
    });
    let restored: Value = serde_json::from_str(&restored).unwrap();
    assert_eq!(restored["ok"], true);
    assert_eq!(restored["value"]["status"], "Active");

    unsafe { camera_connector_mobile_core_destroy(core) };
}

#[test]
fn ffi_ensures_active_project_with_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let ensured =
        take_ffi_string(unsafe { camera_connector_mobile_core_ensure_active_project_json(core) });
    let ensured: Value = serde_json::from_str(&ensured).unwrap();
    assert_eq!(ensured["ok"], true);
    assert_eq!(ensured["value"]["project_id"], "project-inbox");

    let active_again =
        take_ffi_string(unsafe { camera_connector_mobile_core_active_project_json(core) });
    let active_again: Value = serde_json::from_str(&active_again).unwrap();
    assert_eq!(active_again["value"]["project_id"], "project-inbox");

    unsafe { camera_connector_mobile_core_destroy(core) };
}

#[test]
fn ffi_returns_project_dashboard_json_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let name = CString::new("Editorial Shoot").unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let created = take_ffi_string(unsafe {
        camera_connector_mobile_core_create_project_json(core, name.as_ptr())
    });
    let created: Value = serde_json::from_str(&created).unwrap();
    let project_id = CString::new(created["value"]["project_id"].as_str().unwrap()).unwrap();

    let dashboard = take_ffi_string(unsafe {
        camera_connector_mobile_core_project_dashboard_json(core, project_id.as_ptr(), 0, 25)
    });
    let dashboard: Value = serde_json::from_str(&dashboard).unwrap();
    assert_eq!(dashboard["ok"], true);
    assert_eq!(dashboard["value"]["assets"]["limit"], 25);
    assert_eq!(dashboard["value"]["assets"]["total_groups"], 0);
    assert_eq!(dashboard["value"]["transfers"]["total_count"], 0);

    unsafe { camera_connector_mobile_core_destroy(core) };
}

#[test]
fn ffi_returns_project_group_assets_json_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("FFI Members").unwrap();
    service
        .record_project_transfer(
            &project.project_id,
            completed_transfer("ftp:ffi-jpg", "DCIM/100/IMG_6001.JPG", 20),
        )
        .unwrap();
    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .unwrap();
    let group_id = page.groups[0].group_id.clone().unwrap();

    let config_path = CString::new(config_path.to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let project_id = CString::new(project.project_id).unwrap();
    let group_id = CString::new(group_id).unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let response = take_ffi_string(unsafe {
        camera_connector_mobile_core_project_group_assets_json(
            core,
            project_id.as_ptr(),
            group_id.as_ptr(),
        )
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(value["ok"], true);
    let assets = value["value"].as_array().unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0]["final_filename"], "IMG_6001.JPG");
    assert_eq!(assets[0]["group_id"], group_id.to_str().unwrap());
}

#[test]
fn ffi_moves_project_group_json_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let source_project = service.create_project("FFI Wrong Project").unwrap();
    let target_project = service.create_project("FFI Correct Project").unwrap();
    service
        .record_project_transfer(
            &source_project.project_id,
            completed_transfer("ftp:ffi-move-jpg", "DCIM/100/IMG_6102.JPG", 20),
        )
        .unwrap();
    service
        .record_project_transfer(
            &source_project.project_id,
            completed_transfer("ftp:ffi-move-raw", "DCIM/100/IMG_6102.NEF", 21),
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

    let config_path = CString::new(config_path.to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let source_project_id = CString::new(source_project.project_id.clone()).unwrap();
    let target_project_id = CString::new(target_project.project_id.clone()).unwrap();
    let group_id = CString::new(group_id).unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let response = take_ffi_string(unsafe {
        camera_connector_mobile_core_move_project_group_json(
            core,
            source_project_id.as_ptr(),
            group_id.as_ptr(),
            target_project_id.as_ptr(),
        )
    });
    let source_dashboard = take_ffi_string(unsafe {
        camera_connector_mobile_core_project_dashboard_json(core, source_project_id.as_ptr(), 0, 25)
    });
    let target_dashboard = take_ffi_string(unsafe {
        camera_connector_mobile_core_project_dashboard_json(core, target_project_id.as_ptr(), 0, 25)
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let value: Value = serde_json::from_str(&response).unwrap();
    let source_dashboard: Value = serde_json::from_str(&source_dashboard).unwrap();
    let target_dashboard: Value = serde_json::from_str(&target_dashboard).unwrap();
    assert_eq!(value["ok"], true);
    assert_eq!(value["value"]["project_id"], target_project.project_id);
    assert_eq!(value["value"]["member_count"], 2);
    assert_eq!(source_dashboard["value"]["assets"]["total_groups"], 0);
    assert_eq!(target_dashboard["value"]["assets"]["total_groups"], 1);
}

#[test]
fn ffi_claims_and_updates_publish_queue_json_envelopes() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("FFI Publisher").unwrap();
    let store = service.storage_store().unwrap();
    let item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:ffi-publish",
            "staging/ffi-publish.tmp",
            "IMG_7001.JPG",
            42,
        )
        .unwrap();

    let config_path = CString::new(config_path.to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let queue_id = CString::new(item.queue_id.clone()).unwrap();
    let error = CString::new("permission revoked").unwrap();
    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };

    let claimed =
        take_ffi_string(unsafe { camera_connector_mobile_core_claim_next_publish_item_json(core) });
    let claimed: Value = serde_json::from_str(&claimed).unwrap();
    assert_eq!(claimed["ok"], true);
    assert_eq!(claimed["value"]["queue_id"], item.queue_id);
    assert_eq!(claimed["value"]["state"], "Publishing");

    let completed = take_ffi_string(unsafe {
        camera_connector_mobile_core_mark_publish_completed_json(core, queue_id.as_ptr())
    });
    let completed: Value = serde_json::from_str(&completed).unwrap();
    assert_eq!(completed["ok"], true);
    assert_eq!(completed["value"]["completed"], true);

    let failed_item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:ffi-failed",
            "staging/ffi-failed.tmp",
            "IMG_7002.JPG",
            43,
        )
        .unwrap();
    let failed_queue_id = CString::new(failed_item.queue_id.clone()).unwrap();
    let _ =
        take_ffi_string(unsafe { camera_connector_mobile_core_claim_next_publish_item_json(core) });
    let failed = take_ffi_string(unsafe {
        camera_connector_mobile_core_mark_publish_failed_json(
            core,
            failed_queue_id.as_ptr(),
            error.as_ptr(),
        )
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let failed: Value = serde_json::from_str(&failed).unwrap();
    assert_eq!(failed["ok"], true);
    assert_eq!(failed["value"]["failed"], true);
}

#[test]
fn ffi_releases_failed_publish_retries_json_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("FFI Retry").unwrap();
    let store = service.storage_store().unwrap();
    let item = store
        .enqueue_publish(
            &project.project_id,
            "ftp:ffi-retry",
            "staging/ffi-retry.tmp",
            "IMG_7003.JPG",
            45,
        )
        .unwrap();
    store
        .mark_publish_failed(&item.queue_id, "permission revoked")
        .unwrap();

    let config_path = CString::new(config_path.to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let project_id = CString::new(project.project_id.clone()).unwrap();
    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };

    let deferred =
        take_ffi_string(unsafe { camera_connector_mobile_core_claim_next_publish_item_json(core) });
    let deferred: Value = serde_json::from_str(&deferred).unwrap();
    assert_eq!(deferred["ok"], true);
    assert!(deferred["value"].is_null());

    let released = take_ffi_string(unsafe {
        camera_connector_mobile_core_release_failed_publish_retries_json(core, project_id.as_ptr())
    });
    let claimed =
        take_ffi_string(unsafe { camera_connector_mobile_core_claim_next_publish_item_json(core) });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let released: Value = serde_json::from_str(&released).unwrap();
    let claimed: Value = serde_json::from_str(&claimed).unwrap();
    assert_eq!(released["ok"], true);
    assert_eq!(released["value"]["project_id"], project.project_id);
    assert_eq!(released["value"]["released_count"], 1);
    assert_eq!(claimed["value"]["queue_id"], item.queue_id);
    assert_eq!(claimed["value"]["state"], "Publishing");
}

#[test]
fn ffi_completes_publish_with_platform_location_json_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("FFI Platform Publisher").unwrap();
    let store = service.storage_store().unwrap();
    let item = store
        .enqueue_publish_with_metadata(
            &project.project_id,
            "ftp:ffi-platform-publish",
            "staging/ffi-platform-publish.tmp",
            "IMG_7010.JPG",
            42,
            PublishTransferMetadata {
                protocol: "ftp".to_string(),
                original_path: "DCIM/100/IMG_7010.JPG".to_string(),
                username: Some("z5".to_string()),
                remote_addr: Some("192.168.137.56".to_string()),
                source_name: Some("Studio Z5".to_string()),
                started_at_ms: 42,
            },
        )
        .unwrap();

    let config_path = CString::new(config_path.to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let queue_id = CString::new(item.queue_id.clone()).unwrap();
    let final_filename = CString::new("IMG_7010.JPG").unwrap();
    let location_kind = CString::new("document_uri").unwrap();
    let location = CString::new("content://camera-connector/IMG_7010.JPG").unwrap();
    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let _ =
        take_ffi_string(unsafe { camera_connector_mobile_core_claim_next_publish_item_json(core) });

    let completed = take_ffi_string(unsafe {
        camera_connector_mobile_core_complete_publish_json(
            core,
            queue_id.as_ptr(),
            final_filename.as_ptr(),
            location_kind.as_ptr(),
            location.as_ptr(),
        )
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let completed: Value = serde_json::from_str(&completed).unwrap();
    assert_eq!(completed["ok"], true);
    assert_eq!(
        completed["value"]["transfer_id"],
        "ftp:ffi-platform-publish"
    );
    assert_eq!(completed["value"]["final_location"]["kind"], "document_uri");
}

#[test]
fn ffi_rejects_null_core_pointer() {
    let project_id = CString::new("project-inbox").unwrap();
    let response_ptr = unsafe {
        camera_connector_mobile_core_project_dashboard_json(
            std::ptr::null(),
            project_id.as_ptr(),
            0,
            25,
        )
    };
    let response = take_ffi_string(response_ptr);
    let value: Value = serde_json::from_str(&response).unwrap();

    assert_eq!(value["ok"], false);
    assert_eq!(value["error"], "mobile core pointer is null");
}

#[test]
fn ffi_starts_and_stops_receiver_with_envelopes() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let output_dir = temp.path().join("output");
    let state_dir = temp.path().join("state");
    let patch = CString::new(format!(
        r#"{{
            "protocol":"ftp",
            "bind_host":"127.0.0.1",
            "ftp_port":0,
            "output_dir":{},
            "state_dir":{}
        }}"#,
        serde_json::to_string(&output_dir.to_string_lossy()).unwrap(),
        serde_json::to_string(&state_dir.to_string_lossy()).unwrap(),
    ))
    .unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    take_ffi_string(unsafe {
        camera_connector_mobile_core_save_receiver_settings_json(core, patch.as_ptr())
    });

    let started =
        take_ffi_string(unsafe { camera_connector_mobile_core_start_receiver_json(core) });
    let started: Value = serde_json::from_str(&started).unwrap();
    assert_eq!(started["ok"], true);
    assert_eq!(started["value"]["phase"], "Running");

    let stopped = take_ffi_string(unsafe { camera_connector_mobile_core_stop_receiver_json(core) });
    let stopped: Value = serde_json::from_str(&stopped).unwrap();
    assert_eq!(stopped["ok"], true);
    assert_eq!(stopped["value"]["phase"], "Stopped");

    unsafe { camera_connector_mobile_core_destroy(core) };
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

fn take_ffi_string(ptr: *mut std::os::raw::c_char) -> String {
    assert!(!ptr.is_null());
    let value = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { camera_connector_mobile_core_free_string(ptr) };
    value
}

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
    camera_connector_mobile_core_create_global_prompt_pack_json,
    camera_connector_mobile_core_create_project_json,
    camera_connector_mobile_core_delete_global_prompt_pack_json,
    camera_connector_mobile_core_delete_model_provider_settings_json,
    camera_connector_mobile_core_destroy, camera_connector_mobile_core_fork_prompt_pack_json,
    camera_connector_mobile_core_free_string,
    camera_connector_mobile_core_generate_project_recommendation_json,
    camera_connector_mobile_core_global_prompt_packs_json,
    camera_connector_mobile_core_latest_project_recommendation_run_status_json,
    camera_connector_mobile_core_list_projects_json,
    camera_connector_mobile_core_mark_publish_completed_json,
    camera_connector_mobile_core_mark_publish_failed_json,
    camera_connector_mobile_core_model_provider_settings_json,
    camera_connector_mobile_core_model_provider_settings_list_json,
    camera_connector_mobile_core_project_dashboard_json,
    camera_connector_mobile_core_project_evaluation_settings_json,
    camera_connector_mobile_core_project_group_assets_json,
    camera_connector_mobile_core_prompt_packs_for_project_json,
    camera_connector_mobile_core_release_failed_publish_retries_json,
    camera_connector_mobile_core_rename_project_json,
    camera_connector_mobile_core_restore_project_json,
    camera_connector_mobile_core_save_global_prompt_pack_json,
    camera_connector_mobile_core_save_model_provider_settings_json,
    camera_connector_mobile_core_save_project_evaluation_settings_json,
    camera_connector_mobile_core_save_prompt_pack_json,
    camera_connector_mobile_core_set_active_project_json,
    camera_connector_mobile_core_split_burst_member_json,
};

#[path = "mobile_ffi_tests/receiver.rs"]
mod receiver;
#[path = "mobile_ffi_tests/settings_prompt.rs"]
mod settings_prompt;

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
fn ffi_renames_project_json_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("Untitled FFI Shoot").unwrap();
    service.set_active_project(&project.project_id).unwrap();
    let config_path = CString::new(config_path.to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let project_id = CString::new(project.project_id.clone()).unwrap();
    let name = CString::new("Client FFI Shoot").unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let response = take_ffi_string(unsafe {
        camera_connector_mobile_core_rename_project_json(core, project_id.as_ptr(), name.as_ptr())
    });
    let active = take_ffi_string(unsafe { camera_connector_mobile_core_active_project_json(core) });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let value: Value = serde_json::from_str(&response).unwrap();
    let active: Value = serde_json::from_str(&active).unwrap();
    assert_eq!(value["ok"], true);
    assert_eq!(value["value"]["project_id"], project.project_id);
    assert_eq!(value["value"]["name"], "Client FFI Shoot");
    assert_eq!(value["value"]["slug"], "client-ffi-shoot");
    assert_eq!(active["value"]["project_id"], project.project_id);
    assert_eq!(active["value"]["name"], "Client FFI Shoot");
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
fn ffi_splits_burst_member_json_envelope() {
    let fixture = three_member_burst_fixture("FFI Split Decision");
    let core = unsafe { camera_connector_mobile_core_create(fixture.config_path.as_ptr()) };

    let updated = take_ffi_string(unsafe {
        camera_connector_mobile_core_split_burst_member_json(
            core,
            fixture.burst_id.as_ptr(),
            fixture.member_group_id.as_ptr(),
        )
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let updated: Value = serde_json::from_str(&updated).unwrap();
    assert_eq!(updated["ok"], true);
    assert_eq!(updated["value"]["member_count"], 2);
    assert_eq!(updated["value"]["recommendation_status"], "pending");
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
    let project_id = CString::new("project-missing").unwrap();
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

struct ThreeMemberBurstFixture {
    _temp: tempfile::TempDir,
    config_path: CString,
    burst_id: CString,
    member_group_id: CString,
}

fn three_member_burst_fixture(project_name: &str) -> ThreeMemberBurstFixture {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project(project_name).unwrap();
    for (transfer_id, path, completed_at_ms) in [
        ("ftp:ffi-three-1", "DCIM/120/IMG_7701.JPG", 1000),
        ("ftp:ffi-three-2", "DCIM/120/IMG_7702.JPG", 1100),
        ("ftp:ffi-three-3", "DCIM/120/IMG_7703.JPG", 1200),
    ] {
        service
            .record_project_transfer(
                &project.project_id,
                completed_transfer(transfer_id, path, completed_at_ms),
            )
            .unwrap();
    }
    service.drain_analysis_jobs(10).unwrap();

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .unwrap();
    let burst_members = page
        .groups
        .iter()
        .filter(|group| group.burst.is_some())
        .collect::<Vec<_>>();
    let burst_id = burst_members[0]
        .burst
        .as_ref()
        .unwrap()
        .burst_group_id
        .clone();
    let member_group_id = burst_members[1].group_id.clone().unwrap();

    ThreeMemberBurstFixture {
        _temp: temp,
        config_path: CString::new(config_path.to_string_lossy().as_bytes()).unwrap(),
        burst_id: CString::new(burst_id).unwrap(),
        member_group_id: CString::new(member_group_id).unwrap(),
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

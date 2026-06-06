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
    camera_connector_mobile_core_create_global_prompt_profile_json,
    camera_connector_mobile_core_create_project_json,
    camera_connector_mobile_core_delete_model_provider_settings_json,
    camera_connector_mobile_core_destroy, camera_connector_mobile_core_fork_prompt_profile_json,
    camera_connector_mobile_core_free_string,
    camera_connector_mobile_core_generate_project_recommendation_json,
    camera_connector_mobile_core_global_prompt_profiles_json,
    camera_connector_mobile_core_latest_project_recommendation_run_status_json,
    camera_connector_mobile_core_list_projects_json,
    camera_connector_mobile_core_mark_publish_completed_json,
    camera_connector_mobile_core_mark_publish_failed_json,
    camera_connector_mobile_core_merge_burst_member_json,
    camera_connector_mobile_core_model_provider_settings_json,
    camera_connector_mobile_core_model_provider_settings_list_json,
    camera_connector_mobile_core_move_project_group_json,
    camera_connector_mobile_core_project_dashboard_json,
    camera_connector_mobile_core_project_evaluation_settings_json,
    camera_connector_mobile_core_project_group_assets_json,
    camera_connector_mobile_core_prompt_profiles_for_project_json,
    camera_connector_mobile_core_release_failed_publish_retries_json,
    camera_connector_mobile_core_remove_device_account_json,
    camera_connector_mobile_core_rename_project_json,
    camera_connector_mobile_core_restore_project_json,
    camera_connector_mobile_core_save_device_account_json,
    camera_connector_mobile_core_save_global_prompt_version_json,
    camera_connector_mobile_core_save_model_provider_settings_json,
    camera_connector_mobile_core_save_project_evaluation_settings_json,
    camera_connector_mobile_core_save_prompt_version_json,
    camera_connector_mobile_core_save_receiver_settings_json,
    camera_connector_mobile_core_set_active_project_json,
    camera_connector_mobile_core_split_burst_member_json,
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
fn ffi_model_provider_settings_json_round_trips_without_secrets() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let patch = CString::new(
        r#"{
            "provider_kind":"openai",
            "settings_id":"photo-eval-model",
            "provider_label":"OpenAI",
            "default_model":"gpt-5.1-mini",
            "default_max_image_side":1536,
            "default_send_mode":"detail_image",
            "default_batch_size":3,
            "configured":true,
            "secret":"nope"
        }"#,
    )
    .unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let missing =
        take_ffi_string(unsafe { camera_connector_mobile_core_model_provider_settings_json(core) });
    let saved = take_ffi_string(unsafe {
        camera_connector_mobile_core_save_model_provider_settings_json(core, patch.as_ptr())
    });
    let loaded =
        take_ffi_string(unsafe { camera_connector_mobile_core_model_provider_settings_json(core) });
    let list = take_ffi_string(unsafe {
        camera_connector_mobile_core_model_provider_settings_list_json(core)
    });
    let settings_id = CString::new("photo-eval-model").unwrap();
    let deleted = take_ffi_string(unsafe {
        camera_connector_mobile_core_delete_model_provider_settings_json(core, settings_id.as_ptr())
    });
    let list_after_delete = take_ffi_string(unsafe {
        camera_connector_mobile_core_model_provider_settings_list_json(core)
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let missing: Value = serde_json::from_str(&missing).unwrap();
    let saved: Value = serde_json::from_str(&saved).unwrap();
    let loaded: Value = serde_json::from_str(&loaded).unwrap();
    let list: Value = serde_json::from_str(&list).unwrap();
    let deleted: Value = serde_json::from_str(&deleted).unwrap();
    let list_after_delete: Value = serde_json::from_str(&list_after_delete).unwrap();
    assert_eq!(missing["ok"], true);
    assert_eq!(missing["value"]["configured"], false);
    assert_eq!(saved["value"]["provider_kind"], "openai");
    assert_eq!(saved["value"]["settings_id"], "photo-eval-model");
    assert_eq!(loaded["value"]["default_batch_size"], 3);
    assert_eq!(list["value"].as_array().unwrap().len(), 1);
    assert_eq!(deleted["value"]["deleted"], true);
    assert!(list_after_delete["value"].as_array().unwrap().is_empty());
    assert!(!saved.to_string().contains("nope"));
    assert!(!loaded.to_string().contains("secret"));
}

#[test]
fn ffi_project_settings_and_prompt_profiles_json_are_available() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("FFI Evaluation Settings").unwrap();
    let config_path = CString::new(config_path.to_string_lossy().as_bytes()).unwrap();
    let project_id = CString::new(project.project_id.clone()).unwrap();
    let patch = CString::new(
        r#"{
            "model_evaluation_enabled":false,
            "auto_evaluate_on_upload":true,
            "auto_burst_recommendation_enabled":false,
            "project_recommendation_mode":"manual",
            "prompt_profile_id":null,
            "scene_profile":"landscape",
            "cv_policy":"loose",
            "allow_risky_model_selects":true,
            "max_image_side":1200,
            "batch_size":5
        }"#,
    )
    .unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let saved = take_ffi_string(unsafe {
        camera_connector_mobile_core_save_project_evaluation_settings_json(
            core,
            project_id.as_ptr(),
            patch.as_ptr(),
        )
    });
    let loaded = take_ffi_string(unsafe {
        camera_connector_mobile_core_project_evaluation_settings_json(core, project_id.as_ptr())
    });
    let profiles = take_ffi_string(unsafe {
        camera_connector_mobile_core_prompt_profiles_for_project_json(core, project_id.as_ptr())
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let saved: Value = serde_json::from_str(&saved).unwrap();
    let loaded: Value = serde_json::from_str(&loaded).unwrap();
    let profiles: Value = serde_json::from_str(&profiles).unwrap();
    assert_eq!(saved["ok"], true);
    assert_eq!(saved["value"]["project_recommendation_mode"], "manual");
    assert_eq!(loaded["value"]["scene_profile"], "landscape");
    assert!(profiles["value"]
        .as_array()
        .unwrap()
        .iter()
        .any(|profile| profile["prompt_profile_id"] == "general-default"
            && profile["built_in"] == true
            && profile["style_tags"].is_array()));
}

#[test]
fn ffi_rejects_invalid_settings_enum_json_envelopes() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("FFI Invalid Enums").unwrap();
    let config_path = CString::new(config_path.to_string_lossy().as_bytes()).unwrap();
    let project_id = CString::new(project.project_id).unwrap();
    let invalid_provider = CString::new(r#"{"provider_kind":"OpenAI"}"#).unwrap();
    let invalid_scene = CString::new(
        r#"{
            "model_evaluation_enabled":false,
            "project_recommendation_mode":"manual",
            "scene_profile":"Portrait",
            "cv_policy":"standard"
        }"#,
    )
    .unwrap();
    let invalid_mode = CString::new(
        r#"{
            "model_evaluation_enabled":false,
            "project_recommendation_mode":"automatic",
            "scene_profile":"general",
            "cv_policy":"standard"
        }"#,
    )
    .unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let provider = take_ffi_string(unsafe {
        camera_connector_mobile_core_save_model_provider_settings_json(
            core,
            invalid_provider.as_ptr(),
        )
    });
    let scene = take_ffi_string(unsafe {
        camera_connector_mobile_core_save_project_evaluation_settings_json(
            core,
            project_id.as_ptr(),
            invalid_scene.as_ptr(),
        )
    });
    let mode = take_ffi_string(unsafe {
        camera_connector_mobile_core_save_project_evaluation_settings_json(
            core,
            project_id.as_ptr(),
            invalid_mode.as_ptr(),
        )
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let provider: Value = serde_json::from_str(&provider).unwrap();
    let scene: Value = serde_json::from_str(&scene).unwrap();
    let mode: Value = serde_json::from_str(&mode).unwrap();
    assert_eq!(provider["ok"], false);
    assert!(provider["error"]
        .as_str()
        .unwrap()
        .contains("invalid provider_kind: OpenAI"));
    assert_eq!(scene["ok"], false);
    assert!(scene["error"]
        .as_str()
        .unwrap()
        .contains("invalid scene_profile: Portrait"));
    assert_eq!(mode["ok"], false);
    assert!(mode["error"]
        .as_str()
        .unwrap()
        .contains("invalid project_recommendation_mode: automatic"));
}

#[test]
fn ffi_prompt_edit_and_manual_project_recommendation_endpoints_have_envelopes() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let project_name = CString::new("FFI Prompt Recommendation").unwrap();
    let provider = CString::new(
        r#"{
            "provider_kind":"imported",
            "provider_label":"Imported",
            "default_model":"model-stub-v1",
            "default_max_image_side":1024,
            "default_send_mode":"preview_only",
            "default_batch_size":2,
            "configured":true
        }"#,
    )
    .unwrap();
    let source_profile_id = CString::new("general-default").unwrap();
    let fork_name = CString::new("FFI Editable").unwrap();
    let prompt_text = CString::new("Return concise project recommendations.").unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let created = take_ffi_string(unsafe {
        camera_connector_mobile_core_create_project_json(core, project_name.as_ptr())
    });
    let created: Value = serde_json::from_str(&created).unwrap();
    let project_id = CString::new(created["value"]["project_id"].as_str().unwrap()).unwrap();
    take_ffi_string(unsafe {
        camera_connector_mobile_core_save_model_provider_settings_json(core, provider.as_ptr())
    });

    let forked = take_ffi_string(unsafe {
        camera_connector_mobile_core_fork_prompt_profile_json(
            core,
            project_id.as_ptr(),
            source_profile_id.as_ptr(),
            fork_name.as_ptr(),
        )
    });
    let forked: Value = serde_json::from_str(&forked).unwrap();
    assert_eq!(forked["ok"], true);
    assert_eq!(forked["value"]["scope"], "project");
    let prompt_profile_id =
        CString::new(forked["value"]["prompt_profile_id"].as_str().unwrap()).unwrap();
    let project_settings = CString::new(format!(
        r#"{{
            "model_evaluation_enabled":false,
            "auto_evaluate_on_upload":false,
            "auto_burst_recommendation_enabled":true,
            "project_recommendation_mode":"manual",
            "prompt_profile_id":{},
            "model_provider_settings_id":"global",
            "scene_profile":"general",
            "cv_policy":"standard",
            "allow_risky_model_selects":false
        }}"#,
        serde_json::to_string(prompt_profile_id.to_str().unwrap()).unwrap()
    ))
    .unwrap();
    take_ffi_string(unsafe {
        camera_connector_mobile_core_save_project_evaluation_settings_json(
            core,
            project_id.as_ptr(),
            project_settings.as_ptr(),
        )
    });

    let version = take_ffi_string(unsafe {
        camera_connector_mobile_core_save_prompt_version_json(
            core,
            project_id.as_ptr(),
            prompt_profile_id.as_ptr(),
            prompt_text.as_ptr(),
        )
    });
    let recommendation = take_ffi_string(unsafe {
        camera_connector_mobile_core_generate_project_recommendation_json(core, project_id.as_ptr())
    });
    let run = take_ffi_string(unsafe {
        camera_connector_mobile_core_latest_project_recommendation_run_status_json(
            core,
            project_id.as_ptr(),
        )
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let version: Value = serde_json::from_str(&version).unwrap();
    let recommendation: Value = serde_json::from_str(&recommendation).unwrap();
    let run: Value = serde_json::from_str(&run).unwrap();
    assert_eq!(version["ok"], true);
    assert_eq!(
        version["value"]["prompt_profile_id"],
        prompt_profile_id.to_str().unwrap()
    );
    assert_eq!(recommendation["ok"], true);
    assert!(recommendation["value"]["run_id"].as_str().is_some());
    assert_eq!(run["ok"], true);
    assert_eq!(run["value"]["run_type"], "project_recommendation");
    assert_eq!(run["value"]["status"], "ready");
}

#[test]
fn ffi_creates_global_prompt_profile_with_structured_preference_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let name = CString::new("Documentary Preference").unwrap();
    let style_tags = CString::new(r#"["documentary","portrait"]"#).unwrap();
    let scene_profile = CString::new("portrait").unwrap();
    let prompt_text =
        CString::new("Prefer honest documentary moments, natural skin tone, and clear subjects.")
            .unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let created = take_ffi_string(unsafe {
        camera_connector_mobile_core_create_global_prompt_profile_json(
            core,
            name.as_ptr(),
            style_tags.as_ptr(),
            scene_profile.as_ptr(),
            prompt_text.as_ptr(),
        )
    });
    let listed =
        take_ffi_string(unsafe { camera_connector_mobile_core_global_prompt_profiles_json(core) });
    let prompt_profile_id = {
        let created: Value = serde_json::from_str(&created).unwrap();
        CString::new(created["value"]["prompt_profile_id"].as_str().unwrap()).unwrap()
    };
    let updated_text = CString::new("Prefer color restraint and quiet subject emotion.").unwrap();
    let updated = take_ffi_string(unsafe {
        camera_connector_mobile_core_save_global_prompt_version_json(
            core,
            prompt_profile_id.as_ptr(),
            updated_text.as_ptr(),
        )
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let created: Value = serde_json::from_str(&created).unwrap();
    let listed: Value = serde_json::from_str(&listed).unwrap();
    let updated: Value = serde_json::from_str(&updated).unwrap();
    assert_eq!(created["ok"], true);
    assert_eq!(created["value"]["scope"], "global");
    assert_eq!(created["value"]["name"], "Documentary Preference");
    assert_eq!(created["value"]["scene_profile"], "portrait");
    assert_eq!(created["value"]["style_tags"][0], "documentary");
    assert_eq!(created["value"]["style_tags"][1], "portrait");
    assert_eq!(created["value"]["built_in"], false);
    assert!(created["value"]["active_version_id"].as_str().is_some());
    assert!(created["value"]["active_prompt_text"]
        .as_str()
        .unwrap()
        .contains("shared_preference"));
    assert!(listed["value"].as_array().unwrap().iter().any(|profile| {
        profile["prompt_profile_id"] == created["value"]["prompt_profile_id"]
            && profile["active_prompt_text"]
                .as_str()
                .unwrap()
                .contains("honest documentary moments")
    }));
    assert_eq!(updated["ok"], true);
    assert_eq!(
        updated["value"]["prompt_profile_id"],
        created["value"]["prompt_profile_id"]
    );
    assert!(updated["value"]["active_prompt_text"]
        .as_str()
        .unwrap()
        .contains("color restraint"));
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
fn ffi_merges_burst_member_json_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("FFI Merge Decision").unwrap();
    for (transfer_id, path, completed_at_ms) in [
        ("ftp:ffi-merge-a-1", "DCIM/250/IMG_8251.JPG", 1000),
        ("ftp:ffi-merge-a-2", "DCIM/250/IMG_8252.JPG", 1100),
        ("ftp:ffi-merge-b-1", "DCIM/251/IMG_9251.JPG", 5000),
        ("ftp:ffi-merge-b-2", "DCIM/251/IMG_9252.JPG", 5100),
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
    let target = page
        .groups
        .iter()
        .find(|group| {
            group
                .primary
                .original_path
                .as_deref()
                .map(|path| path.ends_with("IMG_8251.JPG"))
                .unwrap_or(false)
        })
        .unwrap();
    let source = page
        .groups
        .iter()
        .find(|group| {
            group
                .primary
                .original_path
                .as_deref()
                .map(|path| path.ends_with("IMG_9251.JPG"))
                .unwrap_or(false)
        })
        .unwrap();
    let target_burst_id =
        CString::new(target.burst.as_ref().unwrap().burst_group_id.as_str()).unwrap();
    let source_group_id = CString::new(source.group_id.as_ref().unwrap().as_str()).unwrap();
    let config_path = CString::new(config_path.to_string_lossy().as_bytes()).unwrap();
    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };

    let merged = take_ffi_string(unsafe {
        camera_connector_mobile_core_merge_burst_member_json(
            core,
            target_burst_id.as_ptr(),
            source_group_id.as_ptr(),
        )
    });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let merged: Value = serde_json::from_str(&merged).unwrap();
    assert_eq!(merged["ok"], true);
    assert_eq!(merged["value"]["member_count"], 4);
    assert_eq!(merged["value"]["recommendation_status"], "pending");
    assert_eq!(merged["value"]["manual_grouping_state"], "merge");
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

use super::*;

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
fn ffi_project_settings_and_prompt_packs_json_are_available() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = temp.path().join("config.json");
    let service = CameraConnectorService::new(Some(config_path.clone()));
    let project = service.create_project("FFI Evaluation Settings").unwrap();
    let config_path = CString::new(config_path.to_string_lossy().as_bytes()).unwrap();
    let project_id = CString::new(project.project_id.clone()).unwrap();
    let patch = CString::new(
        r#"{
            "auto_evaluate_on_upload":true,
            "auto_burst_recommendation_enabled":false,
            "project_recommendation_mode":"manual",
            "prompt_pack_id":null,
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
        camera_connector_mobile_core_prompt_packs_for_project_json(core, project_id.as_ptr())
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
        .any(|profile| profile["prompt_pack_id"] == "general-default"
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
            "project_recommendation_mode":"manual",
            "scene_profile":"Portrait",
            "cv_policy":"standard"
        }"#,
    )
    .unwrap();
    let invalid_mode = CString::new(
        r#"{
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
    let distribution_folder = CString::new("ffi-pack").unwrap();
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
        camera_connector_mobile_core_fork_prompt_pack_json(
            core,
            project_id.as_ptr(),
            source_profile_id.as_ptr(),
            fork_name.as_ptr(),
            distribution_folder.as_ptr(),
        )
    });
    let forked: Value = serde_json::from_str(&forked).unwrap();
    assert_eq!(forked["ok"], true);
    assert_eq!(forked["value"]["built_in"], false);
    assert_eq!(forked["value"]["author"], "user");
    let prompt_pack_id = CString::new(forked["value"]["prompt_pack_id"].as_str().unwrap()).unwrap();
    let project_settings = CString::new(format!(
        r#"{{
            "auto_evaluate_on_upload":false,
            "auto_burst_recommendation_enabled":true,
            "project_recommendation_mode":"manual",
            "prompt_pack_id":{},
            "model_provider_settings_id":"global",
            "scene_profile":"general",
            "cv_policy":"standard",
            "allow_risky_model_selects":false
        }}"#,
        serde_json::to_string(prompt_pack_id.to_str().unwrap()).unwrap()
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
        let prompt_name = CString::new("Edited Project Preference").unwrap();
        let style_tags = CString::new(r#"["project","edited"]"#).unwrap();
        let scene_profile = CString::new("general").unwrap();
        camera_connector_mobile_core_save_prompt_pack_json(
            core,
            project_id.as_ptr(),
            prompt_pack_id.as_ptr(),
            prompt_name.as_ptr(),
            style_tags.as_ptr(),
            scene_profile.as_ptr(),
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
        version["value"]["prompt_pack_id"],
        prompt_pack_id.to_str().unwrap()
    );
    assert_eq!(recommendation["ok"], true);
    assert!(recommendation["value"]["run_id"].as_str().is_some());
    assert_eq!(run["ok"], true);
    assert_eq!(run["value"]["run_type"], "project_recommendation");
    assert_eq!(run["value"]["status"], "ready");
}

#[test]
fn ffi_creates_global_prompt_pack_with_structured_preference_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let name = CString::new("Documentary Preference").unwrap();
    let style_tags = CString::new(r#"["documentary","portrait"]"#).unwrap();
    let scene_profile = CString::new("portrait").unwrap();
    let distribution_folder = CString::new("documentary-pack").unwrap();
    let prompt_text =
        CString::new("Prefer honest documentary moments, natural skin tone, and clear subjects.")
            .unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let created = take_ffi_string(unsafe {
        camera_connector_mobile_core_create_global_prompt_pack_json(
            core,
            name.as_ptr(),
            style_tags.as_ptr(),
            scene_profile.as_ptr(),
            distribution_folder.as_ptr(),
            prompt_text.as_ptr(),
        )
    });
    let listed =
        take_ffi_string(unsafe { camera_connector_mobile_core_global_prompt_packs_json(core) });
    let prompt_pack_id = {
        let created: Value = serde_json::from_str(&created).unwrap();
        CString::new(created["value"]["prompt_pack_id"].as_str().unwrap()).unwrap()
    };
    let updated_name = CString::new("Documentary Preference Edited").unwrap();
    let updated_style_tags = CString::new(r#"["documentary","edited"]"#).unwrap();
    let updated_scene_profile = CString::new("landscape").unwrap();
    let updated_text = CString::new("Prefer color restraint and quiet subject emotion.").unwrap();
    let updated = take_ffi_string(unsafe {
        camera_connector_mobile_core_save_global_prompt_pack_json(
            core,
            prompt_pack_id.as_ptr(),
            updated_name.as_ptr(),
            updated_style_tags.as_ptr(),
            updated_scene_profile.as_ptr(),
            updated_text.as_ptr(),
        )
    });
    let deleted = take_ffi_string(unsafe {
        camera_connector_mobile_core_delete_global_prompt_pack_json(core, prompt_pack_id.as_ptr())
    });
    let after_delete =
        take_ffi_string(unsafe { camera_connector_mobile_core_global_prompt_packs_json(core) });
    unsafe { camera_connector_mobile_core_destroy(core) };

    let created: Value = serde_json::from_str(&created).unwrap();
    let listed: Value = serde_json::from_str(&listed).unwrap();
    let updated: Value = serde_json::from_str(&updated).unwrap();
    let deleted: Value = serde_json::from_str(&deleted).unwrap();
    let after_delete: Value = serde_json::from_str(&after_delete).unwrap();
    assert_eq!(created["ok"], true);
    assert_eq!(created["value"]["built_in"], false);
    assert_eq!(created["value"]["author"], "user");
    assert_eq!(created["value"]["name"], "Documentary Preference");
    assert_eq!(created["value"]["scene_profile"], "portrait");
    assert_eq!(created["value"]["style_tags"][0], "documentary");
    assert_eq!(created["value"]["style_tags"][1], "portrait");
    assert_eq!(created["value"]["built_in"], false);
    assert!(created["value"]["version"].as_str().is_some());
    let created_prompt_text = created["value"]["prompt_text"].as_str().unwrap();
    assert!(created_prompt_text.contains("honest documentary moments"));
    assert!(!created_prompt_text.contains("shared_preference"));
    assert!(listed["value"].as_array().unwrap().iter().any(|profile| {
        profile["prompt_pack_id"] == created["value"]["prompt_pack_id"]
            && profile["prompt_text"]
                .as_str()
                .unwrap()
                .contains("honest documentary moments")
    }));
    assert_eq!(updated["ok"], true);
    assert_eq!(
        updated["value"]["prompt_pack_id"],
        created["value"]["prompt_pack_id"]
    );
    assert_eq!(updated["value"]["name"], "Documentary Preference Edited");
    assert_eq!(updated["value"]["scene_profile"], "landscape");
    assert_eq!(updated["value"]["style_tags"][1], "edited");
    assert!(updated["value"]["prompt_text"]
        .as_str()
        .unwrap()
        .contains("color restraint"));
    assert_eq!(deleted["ok"], true);
    assert_eq!(deleted["value"]["deleted"], true);
    assert!(!after_delete["value"]
        .as_array()
        .unwrap()
        .iter()
        .any(|profile| { profile["prompt_pack_id"] == created["value"]["prompt_pack_id"] }));
}

use std::ffi::{CStr, CString};

use serde_json::Value;

use camera_connector_ffi::{
    camera_connector_mobile_core_active_project_json, camera_connector_mobile_core_create,
    camera_connector_mobile_core_create_project_json, camera_connector_mobile_core_dashboard_json,
    camera_connector_mobile_core_destroy, camera_connector_mobile_core_free_string,
    camera_connector_mobile_core_list_projects_json,
    camera_connector_mobile_core_project_dashboard_json,
    camera_connector_mobile_core_remove_device_account_json,
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
            "source_name":"Studio Camera"
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
fn ffi_returns_dashboard_json_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let config_path = CString::new(temp.path().join("config.json").to_string_lossy().as_bytes())
        .expect("config path should not contain nul");
    let state_dir = temp.path().join("state");
    std::fs::create_dir_all(&state_dir).unwrap();
    let state_dir = CString::new(state_dir.to_string_lossy().as_bytes()).unwrap();

    let core = unsafe { camera_connector_mobile_core_create(config_path.as_ptr()) };
    let response_ptr =
        unsafe { camera_connector_mobile_core_dashboard_json(core, state_dir.as_ptr(), 0, 25) };
    let response = take_ffi_string(response_ptr);
    unsafe { camera_connector_mobile_core_destroy(core) };

    let value: Value = serde_json::from_str(&response).unwrap();
    assert_eq!(value["ok"], true);
    assert_eq!(value["value"]["assets"]["limit"], 25);
    assert!(value["value"]["paths"]["config_path"].is_string());
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
fn ffi_rejects_null_core_pointer() {
    let response_ptr = unsafe {
        camera_connector_mobile_core_dashboard_json(std::ptr::null(), std::ptr::null(), 0, 25)
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

fn take_ffi_string(ptr: *mut std::os::raw::c_char) -> String {
    assert!(!ptr.is_null());
    let value = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { camera_connector_mobile_core_free_string(ptr) };
    value
}

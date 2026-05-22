use std::ffi::{CStr, CString};

use serde_json::Value;

use camera_connector_ffi::{
    camera_connector_mobile_core_create, camera_connector_mobile_core_dashboard_json,
    camera_connector_mobile_core_destroy, camera_connector_mobile_core_free_string,
    camera_connector_mobile_core_save_device_account_json,
    camera_connector_mobile_core_save_receiver_settings_json,
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
fn ffi_rejects_null_core_pointer() {
    let response_ptr = unsafe {
        camera_connector_mobile_core_dashboard_json(std::ptr::null(), std::ptr::null(), 0, 25)
    };
    let response = take_ffi_string(response_ptr);
    let value: Value = serde_json::from_str(&response).unwrap();

    assert_eq!(value["ok"], false);
    assert_eq!(value["error"], "mobile core pointer is null");
}

fn take_ffi_string(ptr: *mut std::os::raw::c_char) -> String {
    assert!(!ptr.is_null());
    let value = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { camera_connector_mobile_core_free_string(ptr) };
    value
}

use std::os::raw::c_char;

use super::interop::{core_ref, ffi_response, required_c_string};
use super::json_support::parse_json_value;
use super::MobileCore;
/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `name` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_create_project_json(
    core: *const MobileCore,
    name: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let name = required_c_string(name, "name")?;
        let project = core_ref(core)?.create_project_json(name)?;
        parse_json_value(&project)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_list_projects_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let projects = core_ref(core)?.list_projects_json()?;
        parse_json_value(&projects)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_set_active_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let project = core_ref(core)?.set_active_project_json(project_id)?;
        parse_json_value(&project)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// All string pointers must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_rename_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
    name: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let name = required_c_string(name, "name")?;
        let project = core_ref(core)?.rename_project_json(project_id, name)?;
        parse_json_value(&project)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_archive_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let project = core_ref(core)?.archive_project_json(project_id)?;
        parse_json_value(&project)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_delete_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let result = core_ref(core)?.delete_project_json(project_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_restore_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let project = core_ref(core)?.restore_project_json(project_id)?;
        parse_json_value(&project)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_active_project_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let project = core_ref(core)?.active_project_json()?;
        parse_json_value(&project)
    })
}

/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_project_dashboard_json(
    core: *const MobileCore,
    project_id: *const c_char,
    offset: u32,
    limit: u32,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let dashboard = core_ref(core)?.project_dashboard_json(project_id, offset, limit)?;
        parse_json_value(&dashboard)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` and `query_json` must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_project_asset_group_page_json(
    core: *const MobileCore,
    project_id: *const c_char,
    query_json: *const c_char,
    offset: u32,
    limit: u32,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let query_json = required_c_string(query_json, "query_json")?;
        let page =
            core_ref(core)?.project_asset_group_page_json(project_id, query_json, offset, limit)?;
        parse_json_value(&page)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id`, `query_json`, and `title` must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_create_lan_share_session_json(
    core: *const MobileCore,
    project_id: *const c_char,
    query_json: *const c_char,
    title: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let query_json = required_c_string(query_json, "query_json")?;
        let title = required_c_string(title, "title")?;
        let session =
            core_ref(core)?.create_lan_share_session_json(project_id, query_json, title)?;
        parse_json_value(&session)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `share_id` must be a valid, null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_stop_lan_share_session_json(
    core: *const MobileCore,
    share_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let share_id = required_c_string(share_id, "share_id")?;
        let session = core_ref(core)?.stop_lan_share_session_json(share_id)?;
        parse_json_value(&session)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `token` must be a valid, null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_lan_share_asset_group_page_json(
    core: *const MobileCore,
    token: *const c_char,
    offset: u32,
    limit: u32,
) -> *mut c_char {
    ffi_response(|| {
        let token = required_c_string(token, "token")?;
        let page = core_ref(core)?.lan_share_asset_group_page_json(token, offset, limit)?;
        parse_json_value(&page)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `token`, `asset_group_id`, and `patch_json` must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_set_lan_share_guest_mark_json(
    core: *const MobileCore,
    token: *const c_char,
    asset_group_id: *const c_char,
    patch_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let token = required_c_string(token, "token")?;
        let asset_group_id = required_c_string(asset_group_id, "asset_group_id")?;
        let patch_json = required_c_string(patch_json, "patch_json")?;
        let mark =
            core_ref(core)?.set_lan_share_guest_mark_json(token, asset_group_id, patch_json)?;
        parse_json_value(&mark)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` and `group_id` must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_project_group_assets_json(
    core: *const MobileCore,
    project_id: *const c_char,
    group_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let group_id = required_c_string(group_id, "group_id")?;
        let assets = core_ref(core)?.project_group_assets_json(project_id, group_id)?;
        parse_json_value(&assets)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// All string pointers must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_set_asset_group_user_marks_json(
    core: *const MobileCore,
    project_id: *const c_char,
    group_id: *const c_char,
    patch_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let group_id = required_c_string(group_id, "group_id")?;
        let patch_json = required_c_string(patch_json, "patch_json")?;
        let marks =
            core_ref(core)?.set_asset_group_user_marks_json(project_id, group_id, patch_json)?;
        parse_json_value(&marks)
    })
}

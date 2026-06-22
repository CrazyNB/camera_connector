use std::os::raw::c_char;

use super::interop::{core_ref, ffi_response, required_c_string};
use super::json_support::parse_json_value;
use super::MobileCore;

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_prompt_packs_for_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let profiles = core_ref(core)?.prompt_packs_for_project_json(project_id)?;
        parse_json_value(&profiles)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_global_prompt_packs_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let profiles = core_ref(core)?.global_prompt_packs_json()?;
        parse_json_value(&profiles)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// String pointers must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_create_global_prompt_pack_json(
    core: *const MobileCore,
    name: *const c_char,
    style_tags_json: *const c_char,
    scene_profile: *const c_char,
    distribution_folder: *const c_char,
    prompt_text: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let name = required_c_string(name, "name")?;
        let style_tags_json = required_c_string(style_tags_json, "style_tags_json")?;
        let scene_profile = required_c_string(scene_profile, "scene_profile")?;
        let distribution_folder = required_c_string(distribution_folder, "distribution_folder")?;
        let prompt_text = required_c_string(prompt_text, "prompt_text")?;
        let profile = core_ref(core)?.create_global_prompt_pack_json(
            name,
            style_tags_json,
            scene_profile,
            distribution_folder,
            prompt_text,
        )?;
        parse_json_value(&profile)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// String pointers must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_global_prompt_pack_json(
    core: *const MobileCore,
    prompt_pack_id: *const c_char,
    name: *const c_char,
    style_tags_json: *const c_char,
    scene_profile: *const c_char,
    prompt_text: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let prompt_pack_id = required_c_string(prompt_pack_id, "prompt_pack_id")?;
        let name = required_c_string(name, "name")?;
        let style_tags_json = required_c_string(style_tags_json, "style_tags_json")?;
        let scene_profile = required_c_string(scene_profile, "scene_profile")?;
        let prompt_text = required_c_string(prompt_text, "prompt_text")?;
        let profile = core_ref(core)?.save_global_prompt_pack_json(
            prompt_pack_id,
            name,
            style_tags_json,
            scene_profile,
            prompt_text,
        )?;
        parse_json_value(&profile)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `prompt_pack_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_delete_global_prompt_pack_json(
    core: *const MobileCore,
    prompt_pack_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let prompt_pack_id = required_c_string(prompt_pack_id, "prompt_pack_id")?;
        let result = core_ref(core)?.delete_global_prompt_pack_json(prompt_pack_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `distribution_folder` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_delete_global_prompt_package_json(
    core: *const MobileCore,
    distribution_folder: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let distribution_folder = required_c_string(distribution_folder, "distribution_folder")?;
        let result = core_ref(core)?.delete_global_prompt_package_json(distribution_folder)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// String pointers must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_fork_prompt_pack_json(
    core: *const MobileCore,
    project_id: *const c_char,
    source_profile_id: *const c_char,
    name: *const c_char,
    distribution_folder: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let source_profile_id = required_c_string(source_profile_id, "source_profile_id")?;
        let name = required_c_string(name, "name")?;
        let distribution_folder = required_c_string(distribution_folder, "distribution_folder")?;
        let profile = core_ref(core)?.fork_prompt_pack_json(
            project_id,
            source_profile_id,
            name,
            distribution_folder,
        )?;
        parse_json_value(&profile)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// String pointers must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_prompt_pack_json(
    core: *const MobileCore,
    project_id: *const c_char,
    prompt_pack_id: *const c_char,
    name: *const c_char,
    style_tags_json: *const c_char,
    scene_profile: *const c_char,
    prompt_text: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let prompt_pack_id = required_c_string(prompt_pack_id, "prompt_pack_id")?;
        let name = required_c_string(name, "name")?;
        let style_tags_json = required_c_string(style_tags_json, "style_tags_json")?;
        let scene_profile = required_c_string(scene_profile, "scene_profile")?;
        let prompt_text = required_c_string(prompt_text, "prompt_text")?;
        let version = core_ref(core)?.save_prompt_pack_json(
            project_id,
            prompt_pack_id,
            name,
            style_tags_json,
            scene_profile,
            prompt_text,
        )?;
        parse_json_value(&version)
    })
}

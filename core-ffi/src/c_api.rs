use std::ffi::CString;
use std::os::raw::c_char;

use serde_json::json;

use super::interop::{core_ref, ffi_response, optional_c_string, required_c_string};
use super::json_support::parse_json_value;
use super::{MobileCore, MobileReceiverSettingsPatch};

/// # Safety
///
/// `config_path` must be either null or a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_create(
    config_path: *const c_char,
) -> *mut MobileCore {
    let config_path = optional_c_string(config_path).ok().flatten();
    Box::into_raw(Box::new(MobileCore::new(config_path)))
}

/// # Safety
///
/// `core` must be a pointer returned by `camera_connector_mobile_core_create`.
/// Passing the same pointer more than once is invalid.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_destroy(core: *mut MobileCore) {
    if !core.is_null() {
        drop(Box::from_raw(core));
    }
}

/// # Safety
///
/// `value` must be a pointer returned by one of this crate's string-returning
/// FFI functions. Passing the same pointer more than once is invalid.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_free_string(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_config_path(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| Ok(json!(core_ref(core)?.config_path())))
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_default_state_dir(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| Ok(json!(core_ref(core)?.default_state_dir())))
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_claim_next_publish_item_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let item = core_ref(core)?.claim_next_publish_item_json()?;
        parse_json_value(&item)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `queue_id` must be a valid, null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_mark_publish_completed_json(
    core: *const MobileCore,
    queue_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let queue_id = required_c_string(queue_id, "queue_id")?;
        let result = core_ref(core)?.mark_publish_completed_json(queue_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// All string pointers must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_complete_publish_json(
    core: *const MobileCore,
    queue_id: *const c_char,
    final_filename: *const c_char,
    location_kind: *const c_char,
    location: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let queue_id = required_c_string(queue_id, "queue_id")?;
        let final_filename = required_c_string(final_filename, "final_filename")?;
        let location_kind = required_c_string(location_kind, "location_kind")?;
        let location = required_c_string(location, "location")?;
        let result = core_ref(core)?.complete_publish_json(
            queue_id,
            final_filename,
            location_kind,
            location,
        )?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `queue_id` and `error` must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_mark_publish_failed_json(
    core: *const MobileCore,
    queue_id: *const c_char,
    error: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let queue_id = required_c_string(queue_id, "queue_id")?;
        let error = required_c_string(error, "error")?;
        let result = core_ref(core)?.mark_publish_failed_json(queue_id, error)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_release_failed_publish_retries_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let result = core_ref(core)?.release_failed_publish_retries_json(project_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_drain_analysis_jobs_json(
    core: *const MobileCore,
    limit: u32,
) -> *mut c_char {
    ffi_response(|| {
        let result = core_ref(core)?.drain_analysis_jobs_json(limit)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_drain_analysis_jobs_with_provider_configured_json(
    core: *const MobileCore,
    limit: u32,
    provider_configured: bool,
) -> *mut c_char {
    ffi_response(|| {
        let result = core_ref(core)?
            .drain_analysis_jobs_with_provider_configured_json(limit, provider_configured)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `request_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_enqueue_model_evaluation_for_asset_groups_json(
    core: *const MobileCore,
    request_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let request_json = required_c_string(request_json, "request_json")?;
        let result =
            core_ref(core)?.enqueue_model_evaluation_for_asset_groups_json(request_json)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `request_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_evaluate_asset_groups_with_model_inputs_json(
    core: *const MobileCore,
    request_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let request_json = required_c_string(request_json, "request_json")?;
        let result = core_ref(core)?.evaluate_asset_groups_with_model_inputs_json(request_json)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `request_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_recommend_burst_group_with_candidate_visuals_json(
    core: *const MobileCore,
    request_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let request_json = required_c_string(request_json, "request_json")?;
        let result =
            core_ref(core)?.recommend_burst_group_with_candidate_visuals_json(request_json)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// String pointers must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_assess_asset_group_preview_json(
    core: *const MobileCore,
    asset_group_id: *const c_char,
    sample_json: *const c_char,
    assessor_version: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let asset_group_id = required_c_string(asset_group_id, "asset_group_id")?;
        let sample_json = required_c_string(sample_json, "sample_json")?;
        let assessor_version = required_c_string(assessor_version, "assessor_version")?;
        let result = core_ref(core)?.assess_asset_group_preview_json(
            asset_group_id,
            sample_json,
            assessor_version,
        )?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `burst_group_id` and `member_group_id` must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_split_burst_member_json(
    core: *const MobileCore,
    burst_group_id: *const c_char,
    member_group_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let burst_group_id = required_c_string(burst_group_id, "burst_group_id")?;
        let member_group_id = required_c_string(member_group_id, "member_group_id")?;
        let result = core_ref(core)?.split_burst_member_json(burst_group_id, member_group_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `request_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_create_manual_burst_group_json(
    core: *const MobileCore,
    request_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let request_json = required_c_string(request_json, "request_json")?;
        let result = core_ref(core)?.create_manual_burst_group_json(&request_json)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_model_provider_settings_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let settings = core_ref(core)?.model_provider_settings_json()?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_model_provider_settings_list_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let settings = core_ref(core)?.model_provider_settings_list_json()?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `settings_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_model_provider_settings_json(
    core: *const MobileCore,
    settings_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let settings_json = required_c_string(settings_json, "settings_json")?;
        let settings = core_ref(core)?.save_model_provider_settings_json(settings_json)?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `settings_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_delete_model_provider_settings_json(
    core: *const MobileCore,
    settings_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let settings_id = required_c_string(settings_id, "settings_id")?;
        let result = core_ref(core)?.delete_model_provider_settings_json(settings_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_project_evaluation_settings_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let settings = core_ref(core)?.project_evaluation_settings_json(project_id)?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` and `settings_json` must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_project_evaluation_settings_json(
    core: *const MobileCore,
    project_id: *const c_char,
    settings_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let settings_json = required_c_string(settings_json, "settings_json")?;
        let settings =
            core_ref(core)?.save_project_evaluation_settings_json(project_id, settings_json)?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// String pointers must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_assess_asset_group_preview_with_provider_configured_json(
    core: *const MobileCore,
    asset_group_id: *const c_char,
    sample_json: *const c_char,
    assessor_version: *const c_char,
    provider_configured: bool,
) -> *mut c_char {
    ffi_response(|| {
        let asset_group_id = required_c_string(asset_group_id, "asset_group_id")?;
        let sample_json = required_c_string(sample_json, "sample_json")?;
        let assessor_version = required_c_string(assessor_version, "assessor_version")?;
        let result = core_ref(core)?.assess_asset_group_preview_with_provider_configured_json(
            asset_group_id,
            sample_json,
            assessor_version,
            provider_configured,
        )?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_generate_project_recommendation_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let recommendation = core_ref(core)?.generate_project_recommendation_json(project_id)?;
        parse_json_value(&recommendation)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `request_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_generate_project_recommendation_with_candidate_visuals_json(
    core: *const MobileCore,
    request_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let request_json = required_c_string(request_json, "request_json")?;
        let recommendation = core_ref(core)?
            .generate_project_recommendation_with_candidate_visuals_json(request_json)?;
        parse_json_value(&recommendation)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_latest_project_recommendation_run_status_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let run = core_ref(core)?.latest_project_recommendation_run_status_json(project_id)?;
        parse_json_value(&run)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_should_schedule_subject_assessment_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let result = core_ref(core)?.should_schedule_subject_assessment_json(project_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `assessment_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_subject_assessment_json(
    core: *const MobileCore,
    assessment_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let assessment_json = required_c_string(assessment_json, "assessment_json")?;
        let assessment = core_ref(core)?.save_subject_assessment_json(assessment_json)?;
        parse_json_value(&assessment)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` and `group_ids_json` must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_subject_assessments_for_asset_groups_json(
    core: *const MobileCore,
    project_id: *const c_char,
    group_ids_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let group_ids_json = required_c_string(group_ids_json, "group_ids_json")?;
        let assessments = core_ref(core)?
            .subject_assessments_for_asset_groups_json(project_id, group_ids_json)?;
        parse_json_value(&assessments)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `patch_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_receiver_settings_json(
    core: *const MobileCore,
    patch_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let patch_json = required_c_string(patch_json, "patch_json")?;
        let patch = serde_json::from_str::<MobileReceiverSettingsPatch>(&patch_json)?;
        let settings = core_ref(core)?.save_receiver_settings_json(patch)?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `username` and `device_name` must be valid, null-terminated UTF-8 C strings.
/// `password` must be either null or a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_device_account_json(
    core: *const MobileCore,
    username: *const c_char,
    password: *const c_char,
    device_name: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let username = required_c_string(username, "username")?;
        let password = optional_c_string(password)?;
        let device_name = required_c_string(device_name, "device_name")?;
        let account = core_ref(core)?.save_device_account_json(username, password, device_name)?;
        parse_json_value(&account)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `username` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_remove_device_account_json(
    core: *const MobileCore,
    username: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let username = required_c_string(username, "username")?;
        let removed = core_ref(core)?.remove_device_account_json(username)?;
        parse_json_value(&removed)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_start_receiver_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let status = core_ref(core)?.start_receiver_json()?;
        parse_json_value(&status)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_stop_receiver_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let status = core_ref(core)?.stop_receiver_json()?;
        parse_json_value(&status)
    })
}

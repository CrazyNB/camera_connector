use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jstring, JNI_TRUE};
use jni::EnvUnowned;

use super::interop::{
    java_response, mobile_core_from_handle, optional_java_string, required_java_string,
};
use super::json_support::parse_json_value;
use super::MobileCore;

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_create(
    mut env: EnvUnowned,
    _class: JClass,
    config_path: JString,
) -> jlong {
    env.with_env(|env| -> Result<jlong, jni::errors::Error> {
        let config_path = optional_java_string(env, config_path).unwrap_or(None);
        Ok(Box::into_raw(Box::new(MobileCore::new(config_path))) as jlong)
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

/// # Safety
///
/// `handle` must be a pointer value returned by
/// `Java_com_cameraconnector_app_core_NativeMobileCore_create`. Passing the
/// same handle more than once is invalid.
#[no_mangle]
pub unsafe extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_destroy(
    _env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        drop(Box::from_raw(handle as *mut MobileCore));
    }
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_claimNextPublishItemJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let item = mobile_core_from_handle(handle)?.claim_next_publish_item_json()?;
            parse_json_value(&item)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_completePublishJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    queue_id: JString,
    final_filename: JString,
    location_kind: JString,
    location: JString,
) -> jstring {
    env.with_env(|env| {
        let queue_id = required_java_string(env, queue_id, "queue_id");
        let final_filename = required_java_string(env, final_filename, "final_filename");
        let location_kind = required_java_string(env, location_kind, "location_kind");
        let location = required_java_string(env, location, "location");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.complete_publish_json(
                queue_id?,
                final_filename?,
                location_kind?,
                location?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_markPublishFailedJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    queue_id: JString,
    error: JString,
) -> jstring {
    env.with_env(|env| {
        let queue_id = required_java_string(env, queue_id, "queue_id");
        let error = required_java_string(env, error, "error");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.mark_publish_failed_json(queue_id?, error?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_releaseFailedPublishRetriesJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .release_failed_publish_retries_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_drainAnalysisJobsWithProviderConfiguredJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    limit: jint,
    provider_configured: jboolean,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .drain_analysis_jobs_with_provider_configured_json(
                    limit.max(0) as u32,
                    provider_configured == JNI_TRUE,
                )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_enqueueModelEvaluationForAssetGroupsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    request_json: JString,
) -> jstring {
    env.with_env(|env| {
        let request_json = required_java_string(env, request_json, "request_json");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .enqueue_model_evaluation_for_asset_groups_json(request_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_evaluateAssetGroupsWithModelInputsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    request_json: JString,
) -> jstring {
    env.with_env(|env| {
        let request_json = required_java_string(env, request_json, "request_json");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .evaluate_asset_groups_with_model_inputs_json(request_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_recommendBurstGroupWithCandidateVisualsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    request_json: JString,
) -> jstring {
    env.with_env(|env| {
        let request_json = required_java_string(env, request_json, "request_json");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .recommend_burst_group_with_candidate_visuals_json(request_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_assessAssetGroupPreviewWithProviderConfiguredJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    asset_group_id: JString,
    sample_json: JString,
    assessor_version: JString,
    provider_configured: jboolean,
) -> jstring {
    env.with_env(|env| {
        let asset_group_id = required_java_string(env, asset_group_id, "asset_group_id");
        let sample_json = required_java_string(env, sample_json, "sample_json");
        let assessor_version = required_java_string(env, assessor_version, "assessor_version");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .assess_asset_group_preview_with_provider_configured_json(
                    asset_group_id?,
                    sample_json?,
                    assessor_version?,
                    provider_configured == JNI_TRUE,
                )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_splitBurstMemberJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    burst_group_id: JString,
    member_group_id: JString,
) -> jstring {
    env.with_env(|env| {
        let burst_group_id = required_java_string(env, burst_group_id, "burst_group_id");
        let member_group_id = required_java_string(env, member_group_id, "member_group_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .split_burst_member_json(burst_group_id?, member_group_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_createManualBurstGroupJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    request_json: JString,
) -> jstring {
    env.with_env(|env| {
        let request_json = required_java_string(env, request_json, "request_json");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.create_manual_burst_group_json(&request_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_modelProviderSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.model_provider_settings_json()?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_modelProviderSettingsListJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.model_provider_settings_list_json()?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveModelProviderSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    settings_json: JString,
) -> jstring {
    env.with_env(|env| {
        let settings_json = required_java_string(env, settings_json, "settings_json");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .save_model_provider_settings_json(settings_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_deleteModelProviderSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    settings_id: JString,
) -> jstring {
    env.with_env(|env| {
        let settings_id = required_java_string(env, settings_id, "settings_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .delete_model_provider_settings_json(settings_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_projectEvaluationSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.project_evaluation_settings_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveProjectEvaluationSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    settings_json: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let settings_json = required_java_string(env, settings_json, "settings_json");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .save_project_evaluation_settings_json(project_id?, settings_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_generateProjectRecommendationJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .generate_project_recommendation_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_generateProjectRecommendationWithCandidateVisualsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    request_json: JString,
) -> jstring {
    env.with_env(|env| {
        let request_json = required_java_string(env, request_json, "request_json");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .generate_project_recommendation_with_candidate_visuals_json(request_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_latestProjectRecommendationRunStatusJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .latest_project_recommendation_run_status_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_shouldScheduleSubjectAssessmentJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .should_schedule_subject_assessment_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveSubjectAssessmentJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    assessment_json: JString,
) -> jstring {
    env.with_env(|env| {
        let assessment_json = required_java_string(env, assessment_json, "assessment_json");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.save_subject_assessment_json(assessment_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_subjectAssessmentsForAssetGroupsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    group_ids_json: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let group_ids_json = required_java_string(env, group_ids_json, "group_ids_json");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .subject_assessments_for_asset_groups_json(project_id?, group_ids_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jlong, jstring};
use jni::EnvUnowned;

use super::interop::{java_response, mobile_core_from_handle, required_java_string};
use super::json_support::parse_json_value;
#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_createProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    name: JString,
) -> jstring {
    env.with_env(|env| {
        let name = required_java_string(env, name, "name");
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.create_project_json(name?)?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_listProjectsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let projects = mobile_core_from_handle(handle)?.list_projects_json()?;
            parse_json_value(&projects)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_setActiveProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.set_active_project_json(project_id?)?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_renameProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    name: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let name = required_java_string(env, name, "name");
        java_response(env, || {
            let project =
                mobile_core_from_handle(handle)?.rename_project_json(project_id?, name?)?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_archiveProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.archive_project_json(project_id?)?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_deleteProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.delete_project_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_restoreProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.restore_project_json(project_id?)?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_activeProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.active_project_json()?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_projectDashboardJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    offset: jint,
    limit: jint,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let dashboard = mobile_core_from_handle(handle)?.project_dashboard_json(
                project_id?,
                offset.max(0) as u32,
                limit.max(0) as u32,
            )?;
            parse_json_value(&dashboard)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_projectAssetGroupPageJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    query_json: JString,
    offset: jint,
    limit: jint,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let query_json = required_java_string(env, query_json, "query_json");
        java_response(env, || {
            let page = mobile_core_from_handle(handle)?.project_asset_group_page_json(
                project_id?,
                query_json?,
                offset.max(0) as u32,
                limit.max(0) as u32,
            )?;
            parse_json_value(&page)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_createLanShareSessionJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    query_json: JString,
    title: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let query_json = required_java_string(env, query_json, "query_json");
        let title = required_java_string(env, title, "title");
        java_response(env, || {
            let session = mobile_core_from_handle(handle)?.create_lan_share_session_json(
                project_id?,
                query_json?,
                title?,
            )?;
            parse_json_value(&session)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_stopLanShareSessionJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    share_id: JString,
) -> jstring {
    env.with_env(|env| {
        let share_id = required_java_string(env, share_id, "share_id");
        java_response(env, || {
            let session =
                mobile_core_from_handle(handle)?.stop_lan_share_session_json(share_id?)?;
            parse_json_value(&session)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_lanShareAssetGroupPageJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    token: JString,
    offset: jint,
    limit: jint,
) -> jstring {
    env.with_env(|env| {
        let token = required_java_string(env, token, "token");
        java_response(env, || {
            let page = mobile_core_from_handle(handle)?.lan_share_asset_group_page_json(
                token?,
                offset.max(0) as u32,
                limit.max(0) as u32,
            )?;
            parse_json_value(&page)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_setLanShareGuestMarkJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    token: JString,
    asset_group_id: JString,
    patch_json: JString,
) -> jstring {
    env.with_env(|env| {
        let token = required_java_string(env, token, "token");
        let asset_group_id = required_java_string(env, asset_group_id, "asset_group_id");
        let patch_json = required_java_string(env, patch_json, "patch_json");
        java_response(env, || {
            let mark = mobile_core_from_handle(handle)?.set_lan_share_guest_mark_json(
                token?,
                asset_group_id?,
                patch_json?,
            )?;
            parse_json_value(&mark)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_projectGroupAssetsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    group_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let group_id = required_java_string(env, group_id, "group_id");
        java_response(env, || {
            let assets = mobile_core_from_handle(handle)?
                .project_group_assets_json(project_id?, group_id?)?;
            parse_json_value(&assets)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_deleteProjectGroupJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    group_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let group_id = required_java_string(env, group_id, "group_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .delete_project_group_json(project_id?, group_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_setAssetGroupUserMarksJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    group_id: JString,
    patch_json: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let group_id = required_java_string(env, group_id, "group_id");
        let patch_json = required_java_string(env, patch_json, "patch_json");
        java_response(env, || {
            let marks = mobile_core_from_handle(handle)?.set_asset_group_user_marks_json(
                project_id?,
                group_id?,
                patch_json?,
            )?;
            parse_json_value(&marks)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

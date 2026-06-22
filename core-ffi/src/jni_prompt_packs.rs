use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JString};
use jni::sys::{jlong, jstring};
use jni::EnvUnowned;

use super::interop::{java_response, mobile_core_from_handle, required_java_string};
use super::json_support::parse_json_value;

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_PromptPacksForProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.prompt_packs_for_project_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_globalPromptPacksJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.global_prompt_packs_json()?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_forkGlobalPromptPackJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    source_profile_id: JString,
    name: JString,
    distribution_folder: JString,
) -> jstring {
    env.with_env(|env| {
        let source_profile_id = required_java_string(env, source_profile_id, "source_profile_id");
        let name = required_java_string(env, name, "name");
        let distribution_folder =
            required_java_string(env, distribution_folder, "distribution_folder");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.fork_global_prompt_pack_json(
                source_profile_id?,
                name?,
                distribution_folder?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_createGlobalPromptPackJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    name: JString,
    style_tags_json: JString,
    scene_profile: JString,
    distribution_folder: JString,
    prompt_text: JString,
) -> jstring {
    env.with_env(|env| {
        let name = required_java_string(env, name, "name");
        let style_tags_json = required_java_string(env, style_tags_json, "style_tags_json");
        let scene_profile = required_java_string(env, scene_profile, "scene_profile");
        let distribution_folder =
            required_java_string(env, distribution_folder, "distribution_folder");
        let prompt_text = required_java_string(env, prompt_text, "prompt_text");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.create_global_prompt_pack_json(
                name?,
                style_tags_json?,
                scene_profile?,
                distribution_folder?,
                prompt_text?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveGlobalPromptPackJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    prompt_pack_id: JString,
    name: JString,
    style_tags_json: JString,
    scene_profile: JString,
    prompt_text: JString,
) -> jstring {
    env.with_env(|env| {
        let prompt_pack_id = required_java_string(env, prompt_pack_id, "prompt_pack_id");
        let name = required_java_string(env, name, "name");
        let style_tags_json = required_java_string(env, style_tags_json, "style_tags_json");
        let scene_profile = required_java_string(env, scene_profile, "scene_profile");
        let prompt_text = required_java_string(env, prompt_text, "prompt_text");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.save_global_prompt_pack_json(
                prompt_pack_id?,
                name?,
                style_tags_json?,
                scene_profile?,
                prompt_text?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_deleteGlobalPromptPackJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    prompt_pack_id: JString,
) -> jstring {
    env.with_env(|env| {
        let prompt_pack_id = required_java_string(env, prompt_pack_id, "prompt_pack_id");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.delete_global_prompt_pack_json(prompt_pack_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_deleteGlobalPromptPackageJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    distribution_folder: JString,
) -> jstring {
    env.with_env(|env| {
        let distribution_folder =
            required_java_string(env, distribution_folder, "distribution_folder");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .delete_global_prompt_package_json(distribution_folder?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_forkPromptPackJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    source_profile_id: JString,
    name: JString,
    distribution_folder: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let source_profile_id = required_java_string(env, source_profile_id, "source_profile_id");
        let name = required_java_string(env, name, "name");
        let distribution_folder =
            required_java_string(env, distribution_folder, "distribution_folder");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.fork_prompt_pack_json(
                project_id?,
                source_profile_id?,
                name?,
                distribution_folder?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_savePromptPackJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    prompt_pack_id: JString,
    name: JString,
    style_tags_json: JString,
    scene_profile: JString,
    prompt_text: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let prompt_pack_id = required_java_string(env, prompt_pack_id, "prompt_pack_id");
        let name = required_java_string(env, name, "name");
        let style_tags_json = required_java_string(env, style_tags_json, "style_tags_json");
        let scene_profile = required_java_string(env, scene_profile, "scene_profile");
        let prompt_text = required_java_string(env, prompt_text, "prompt_text");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.save_prompt_pack_json(
                project_id?,
                prompt_pack_id?,
                name?,
                style_tags_json?,
                scene_profile?,
                prompt_text?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

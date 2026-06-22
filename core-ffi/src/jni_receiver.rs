use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JString};
use jni::sys::{jlong, jstring};
use jni::EnvUnowned;

use super::interop::{
    java_response, mobile_core_from_handle, optional_java_string, required_java_string,
};
use super::json_support::parse_json_value;
use super::MobileReceiverSettingsPatch;

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveReceiverSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    patch_json: JString,
) -> jstring {
    env.with_env(|env| {
        let patch_json = required_java_string(env, patch_json, "patch_json");
        java_response(env, || {
            let patch = serde_json::from_str::<MobileReceiverSettingsPatch>(&patch_json?)?;
            let settings = mobile_core_from_handle(handle)?.save_receiver_settings_json(patch)?;
            parse_json_value(&settings)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveDeviceAccountJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    username: JString,
    password: JString,
    device_name: JString,
) -> jstring {
    env.with_env(|env| {
        let username = required_java_string(env, username, "username");
        let password = optional_java_string(env, password);
        let device_name = required_java_string(env, device_name, "device_name");
        java_response(env, || {
            let account = mobile_core_from_handle(handle)?.save_device_account_json(
                username?,
                password?,
                device_name?,
            )?;
            parse_json_value(&account)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_removeDeviceAccountJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    username: JString,
) -> jstring {
    env.with_env(|env| {
        let username = required_java_string(env, username, "username");
        java_response(env, || {
            let removed = mobile_core_from_handle(handle)?.remove_device_account_json(username?)?;
            parse_json_value(&removed)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_startReceiverJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let status = mobile_core_from_handle(handle)?.start_receiver_json()?;
            parse_json_value(&status)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_stopReceiverJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let status = mobile_core_from_handle(handle)?.stop_receiver_json()?;
            parse_json_value(&status)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use jni::objects::JString;
use jni::sys::{jlong, jstring};
use jni::Env;
use serde_json::{json, Value};

use super::{MobileCore, MobileCoreError, MobileCoreResult};

pub(crate) fn core_ref<'a>(core: *const MobileCore) -> MobileCoreResult<&'a MobileCore> {
    if core.is_null() {
        Err(MobileCoreError::NullCore)
    } else {
        Ok(unsafe { &*core })
    }
}

pub(crate) fn optional_c_string(value: *const c_char) -> MobileCoreResult<Option<String>> {
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(c_string(value, "optional")?))
    }
}

pub(crate) fn required_c_string(
    value: *const c_char,
    name: &'static str,
) -> MobileCoreResult<String> {
    if value.is_null() {
        Err(MobileCoreError::NullInput(name))
    } else {
        c_string(value, name)
    }
}

fn c_string(value: *const c_char, name: &'static str) -> MobileCoreResult<String> {
    unsafe { CStr::from_ptr(value) }
        .to_str()
        .map(ToOwned::to_owned)
        .map_err(|_| MobileCoreError::InvalidUtf8(name))
}

pub(crate) fn ffi_response(action: impl FnOnce() -> MobileCoreResult<Value>) -> *mut c_char {
    let response = response_value(action());
    string_to_ffi(
        serde_json::to_string(&response)
            .unwrap_or_else(|error| format!(r#"{{"ok":false,"value":null,"error":"{}"}}"#, error)),
    )
}

pub(crate) fn string_to_ffi(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(value) => value.into_raw(),
        Err(_) => CString::new(
            r#"{"ok":false,"value":null,"error":"response contains an interior nul byte"}"#,
        )
        .expect("static error response should not contain nul")
        .into_raw(),
    }
}

pub(crate) fn mobile_core_from_handle<'a>(handle: jlong) -> MobileCoreResult<&'a MobileCore> {
    if handle == 0 {
        Err(MobileCoreError::NullCore)
    } else {
        Ok(unsafe { &*(handle as *const MobileCore) })
    }
}

pub(crate) fn optional_java_string(
    env: &mut Env,
    value: JString,
) -> MobileCoreResult<Option<String>> {
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(value.try_to_string(env)?))
    }
}

pub(crate) fn required_java_string(
    env: &mut Env,
    value: JString,
    name: &'static str,
) -> MobileCoreResult<String> {
    optional_java_string(env, value)?.ok_or(MobileCoreError::NullInput(name))
}

pub(crate) fn java_response(
    env: &mut Env,
    action: impl FnOnce() -> MobileCoreResult<Value>,
) -> Result<jstring, jni::errors::Error> {
    let response = response_value(action());
    let raw = serde_json::to_string(&response)
        .unwrap_or_else(|error| format!(r#"{{"ok":false,"value":null,"error":"{}"}}"#, error));
    JString::from_str(env, raw).map(|value| value.into_raw())
}

fn response_value(result: MobileCoreResult<Value>) -> Value {
    match result {
        Ok(value) => json!({
            "ok": true,
            "value": value,
            "error": Value::Null,
        }),
        Err(error) => json!({
            "ok": false,
            "value": Value::Null,
            "error": error.to_string(),
        }),
    }
}

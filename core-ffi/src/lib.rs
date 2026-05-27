use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;

use camera_connector_core::{
    AssetGroupQuery, CameraConnectorDashboard, CameraConnectorRuntime, CameraConnectorService,
    ImporterError, PushProtocol, ReceiverConfigRequest, ReceiverSettingsConfig,
    ReceiverSettingsUpdate, StoredObjectLocation,
};
use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jlong, jstring};
use jni::{Env, EnvUnowned};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, thiserror::Error)]
pub enum MobileCoreError {
    #[error("{0}")]
    Core(#[from] ImporterError),
    #[error("invalid protocol: {0}")]
    InvalidProtocol(String),
    #[error("invalid storage location kind: {0}")]
    InvalidLocationKind(String),
    #[error("mobile core pointer is null")]
    NullCore,
    #[error("input pointer is null: {0}")]
    NullInput(&'static str),
    #[error("input is not valid UTF-8: {0}")]
    InvalidUtf8(&'static str),
    #[error("response contains an interior nul byte")]
    InteriorNul,
    #[error("{0}")]
    Jni(#[from] jni::errors::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

pub type MobileCoreResult<T> = std::result::Result<T, MobileCoreError>;

#[derive(Debug, Clone)]
pub struct MobileCore {
    service: CameraConnectorService,
    runtime: CameraConnectorRuntime,
    async_runtime: std::sync::Arc<tokio::runtime::Runtime>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MobileReceiverSettingsPatch {
    pub protocol: Option<String>,
    pub bind_host: Option<String>,
    pub ftp_port: Option<u16>,
    pub sftp_port: Option<u16>,
    pub output_dir: Option<String>,
    pub state_dir: Option<String>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
    pub defer_publish: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileAccountView {
    pub username: String,
    pub device_name: String,
    pub password_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileRemoveAccountView {
    pub username: String,
    pub removed: bool,
}

impl MobileCore {
    pub fn new(config_path: Option<String>) -> Self {
        let service = CameraConnectorService::new(config_path.map(PathBuf::from));
        Self {
            runtime: CameraConnectorRuntime::new(service.clone()),
            service,
            async_runtime: std::sync::Arc::new(
                tokio::runtime::Runtime::new().expect("mobile async runtime should initialize"),
            ),
        }
    }

    pub fn config_path(&self) -> String {
        self.service.config_path().to_string_lossy().into_owned()
    }

    pub fn default_state_dir(&self) -> String {
        self.service.state_dir().to_string_lossy().into_owned()
    }

    pub fn create_project_json(&self, name: String) -> MobileCoreResult<String> {
        let project = self.service.create_project(name)?;
        Ok(serde_json::to_string(&project)?)
    }

    pub fn list_projects_json(&self) -> MobileCoreResult<String> {
        let projects = self.service.list_projects()?;
        Ok(serde_json::to_string(&projects)?)
    }

    pub fn set_active_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        self.service.set_active_project(&project_id)?;
        let project = self
            .service
            .active_project()?
            .ok_or_else(|| ImporterError::internal("active project was not found after update"))?;
        Ok(serde_json::to_string(&project)?)
    }

    pub fn archive_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        let project = self.service.archive_project(&project_id)?;
        Ok(serde_json::to_string(&project)?)
    }

    pub fn restore_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        let project = self.service.restore_project(&project_id)?;
        Ok(serde_json::to_string(&project)?)
    }

    pub fn active_project_json(&self) -> MobileCoreResult<String> {
        let project = self.service.active_project()?;
        Ok(serde_json::to_string(&project)?)
    }

    pub fn ensure_active_project_json(&self) -> MobileCoreResult<String> {
        let project = self.service.ensure_active_project()?;
        Ok(serde_json::to_string(&project)?)
    }

    pub fn project_dashboard_json(
        &self,
        project_id: String,
        offset: u32,
        limit: u32,
    ) -> MobileCoreResult<String> {
        let dashboard: CameraConnectorDashboard = self.service.project_dashboard(
            &project_id,
            AssetGroupQuery::default(),
            offset as usize,
            limit as usize,
            false,
        )?;
        Ok(serde_json::to_string(&dashboard)?)
    }

    pub fn project_group_assets_json(
        &self,
        project_id: String,
        group_id: String,
    ) -> MobileCoreResult<String> {
        let assets = self.service.project_group_assets(&project_id, &group_id)?;
        Ok(serde_json::to_string(&assets)?)
    }

    pub fn claim_next_publish_item_json(&self) -> MobileCoreResult<String> {
        let item = self.service.claim_next_publish_item()?;
        Ok(serde_json::to_string(&item)?)
    }

    pub fn mark_publish_completed_json(&self, queue_id: String) -> MobileCoreResult<String> {
        self.service.mark_publish_completed(&queue_id)?;
        Ok(serde_json::to_string(&json!({
            "queue_id": queue_id,
            "completed": true,
        }))?)
    }

    pub fn complete_publish_json(
        &self,
        queue_id: String,
        final_filename: String,
        location_kind: String,
        location: String,
    ) -> MobileCoreResult<String> {
        let final_location = parse_storage_location(location_kind, location)?;
        let record = self
            .service
            .complete_publish(&queue_id, &final_filename, final_location)?;
        Ok(serde_json::to_string(&record)?)
    }

    pub fn mark_publish_failed_json(
        &self,
        queue_id: String,
        error: String,
    ) -> MobileCoreResult<String> {
        self.service.mark_publish_failed(&queue_id, &error)?;
        Ok(serde_json::to_string(&json!({
            "queue_id": queue_id,
            "failed": true,
        }))?)
    }

    pub fn save_receiver_settings_json(
        &self,
        patch: MobileReceiverSettingsPatch,
    ) -> MobileCoreResult<String> {
        let (settings, _) = self.service.set_receiver_settings(patch.try_into()?)?;
        Ok(serde_json::to_string(&settings)?)
    }

    pub fn save_device_account_json(
        &self,
        username: String,
        password: Option<String>,
        device_name: String,
    ) -> MobileCoreResult<String> {
        let (account, _) = self
            .service
            .set_account(username, password.as_deref(), device_name)?;
        let password_configured = account.password_configured();
        let view = MobileAccountView {
            username: account.username,
            device_name: account.device_name,
            password_configured,
        };
        Ok(serde_json::to_string(&view)?)
    }

    pub fn remove_device_account_json(&self, username: String) -> MobileCoreResult<String> {
        let (removed, _) = self.service.remove_account(&username)?;
        Ok(serde_json::to_string(&MobileRemoveAccountView {
            username,
            removed,
        })?)
    }

    pub fn start_receiver_json(&self) -> MobileCoreResult<String> {
        let status =
            self.async_runtime
                .block_on(self.runtime.start_receiver(ReceiverConfigRequest {
                    protocol: None,
                    bind_host: None,
                    port: None,
                    output_dir: None,
                    state_dir: None,
                    username: None,
                    password: None,
                    advertised_host: None,
                    source_name: None,
                    defer_publish: None,
                }))?;
        Ok(serde_json::to_string(&status)?)
    }

    pub fn stop_receiver_json(&self) -> MobileCoreResult<String> {
        let status = self.async_runtime.block_on(self.runtime.stop_receiver())?;
        Ok(serde_json::to_string(&status)?)
    }
}

impl TryFrom<MobileReceiverSettingsPatch> for ReceiverSettingsUpdate {
    type Error = MobileCoreError;

    fn try_from(patch: MobileReceiverSettingsPatch) -> MobileCoreResult<Self> {
        Ok(Self {
            protocol: patch.protocol.map(parse_protocol).transpose()?,
            bind_host: patch.bind_host,
            ftp_port: patch.ftp_port,
            sftp_port: patch.sftp_port,
            output_dir: patch.output_dir.map(PathBuf::from),
            state_dir: patch.state_dir.map(PathBuf::from),
            advertised_host: patch.advertised_host,
            source_name: patch.source_name,
            defer_publish: patch.defer_publish,
        })
    }
}

fn parse_protocol(protocol: String) -> MobileCoreResult<PushProtocol> {
    match protocol.trim().to_ascii_lowercase().as_str() {
        "ftp" => Ok(PushProtocol::Ftp),
        "sftp" => Ok(PushProtocol::Sftp),
        _ => Err(MobileCoreError::InvalidProtocol(protocol)),
    }
}

fn parse_storage_location(
    kind: String,
    location: String,
) -> MobileCoreResult<StoredObjectLocation> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "local_path" => Ok(StoredObjectLocation::local_path(location)),
        "document_uri" => Ok(StoredObjectLocation::document_uri(location)),
        "media_uri" => Ok(StoredObjectLocation::media_uri(location)),
        "photo_asset" => Ok(StoredObjectLocation::photo_asset(location)),
        _ => Err(MobileCoreError::InvalidLocationKind(kind)),
    }
}

#[allow(dead_code)]
fn _assert_settings_config_is_serializable(settings: &ReceiverSettingsConfig) -> String {
    serde_json::to_string(settings).expect("receiver settings should serialize")
}

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

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_ensure_active_project_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let project = core_ref(core)?.ensure_active_project_json()?;
        parse_json_value(&project)
    })
}

/// # Safety
///
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

fn parse_json_value(json: &str) -> MobileCoreResult<Value> {
    Ok(serde_json::from_str(json)?)
}

fn core_ref<'a>(core: *const MobileCore) -> MobileCoreResult<&'a MobileCore> {
    if core.is_null() {
        Err(MobileCoreError::NullCore)
    } else {
        Ok(unsafe { &*core })
    }
}

fn optional_c_string(value: *const c_char) -> MobileCoreResult<Option<String>> {
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(c_string(value, "optional")?))
    }
}

fn required_c_string(value: *const c_char, name: &'static str) -> MobileCoreResult<String> {
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

fn ffi_response(action: impl FnOnce() -> MobileCoreResult<Value>) -> *mut c_char {
    let response = match action() {
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
    };
    string_to_ffi(
        serde_json::to_string(&response)
            .unwrap_or_else(|error| format!(r#"{{"ok":false,"value":null,"error":"{}"}}"#, error)),
    )
}

fn string_to_ffi(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(value) => value.into_raw(),
        Err(_) => CString::new(
            r#"{"ok":false,"value":null,"error":"response contains an interior nul byte"}"#,
        )
        .expect("static error response should not contain nul")
        .into_raw(),
    }
}

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
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_ensureActiveProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.ensure_active_project_json()?;
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
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_markPublishCompletedJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    queue_id: JString,
) -> jstring {
    env.with_env(|env| {
        let queue_id = required_java_string(env, queue_id, "queue_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.mark_publish_completed_json(queue_id?)?;
            parse_json_value(&result)
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

fn mobile_core_from_handle<'a>(handle: jlong) -> MobileCoreResult<&'a MobileCore> {
    if handle == 0 {
        Err(MobileCoreError::NullCore)
    } else {
        Ok(unsafe { &*(handle as *const MobileCore) })
    }
}

fn optional_java_string(env: &mut Env, value: JString) -> MobileCoreResult<Option<String>> {
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(value.try_to_string(env)?))
    }
}

fn required_java_string(
    env: &mut Env,
    value: JString,
    name: &'static str,
) -> MobileCoreResult<String> {
    optional_java_string(env, value)?.ok_or(MobileCoreError::NullInput(name))
}

fn java_response(
    env: &mut Env,
    action: impl FnOnce() -> MobileCoreResult<Value>,
) -> Result<jstring, jni::errors::Error> {
    let response = match action() {
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
    };
    let raw = serde_json::to_string(&response)
        .unwrap_or_else(|error| format!(r#"{{"ok":false,"value":null,"error":"{}"}}"#, error));
    JString::from_str(env, raw).map(|value| value.into_raw())
}

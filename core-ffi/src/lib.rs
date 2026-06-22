#![allow(clippy::missing_safety_doc)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

mod c_api;
mod c_projects;
mod c_prompt_packs;
mod error;
mod interop;
mod jni_api;
mod jni_projects;
mod jni_prompt_packs;
mod jni_receiver;
mod json_support;
mod mobile_analysis;
mod mobile_prompt_packs;
mod parsing;
mod patch;

use camera_connector_core::{
    AssetGroupPage, AssetGroupQuery, CameraConnectorDashboard, CameraConnectorRuntime,
    CameraConnectorService, ImporterError, ReceiverConfigRequest,
};
use json_support::{
    asset_group_query_from_json, guest_mark_from_patch, non_blank, project_json, project_list_json,
    project_option_json, user_marks_json,
};
use parsing::parse_storage_location;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use c_api::*;
pub use c_projects::*;
pub use c_prompt_packs::*;
pub use error::{MobileCoreError, MobileCoreResult};
pub use jni_receiver::*;

#[derive(Debug, Clone)]
pub struct MobileCore {
    service: CameraConnectorService,
    runtime: CameraConnectorRuntime,
    async_runtime: Arc<tokio::runtime::Runtime>,
    action_clock_ms: Arc<AtomicI64>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MobileUserMarksPatch {
    pub favorite: Option<bool>,
    pub marked: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MobileGuestMarkPatch {
    pub guest_mark: Option<String>,
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
            async_runtime: Arc::new(
                tokio::runtime::Runtime::new().expect("mobile async runtime should initialize"),
            ),
            action_clock_ms: Arc::new(AtomicI64::new(0)),
        }
    }

    fn next_action_time_ms(&self) -> i64 {
        let now = current_time_ms();
        let mut previous = self.action_clock_ms.load(Ordering::Relaxed);
        loop {
            let next = now.max(previous.saturating_add(1));
            match self.action_clock_ms.compare_exchange_weak(
                previous,
                next,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return next,
                Err(observed) => previous = observed,
            }
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
        project_json(project)
    }

    pub fn rename_project_json(
        &self,
        project_id: String,
        name: String,
    ) -> MobileCoreResult<String> {
        let project = self.service.rename_project(&project_id, name)?;
        project_json(project)
    }

    pub fn list_projects_json(&self) -> MobileCoreResult<String> {
        let projects = self.service.list_projects()?;
        project_list_json(projects)
    }

    pub fn set_active_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        self.service.set_active_project(&project_id)?;
        let project = self
            .service
            .active_project()?
            .ok_or_else(|| ImporterError::internal("active project was not found after update"))?;
        project_json(project)
    }

    pub fn archive_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        let project = self.service.archive_project(&project_id)?;
        project_json(project)
    }

    pub fn delete_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        let deleted = self.service.delete_project(&project_id)?;
        Ok(serde_json::to_string(&json!({
            "project_id": project_id,
            "deleted": deleted,
        }))?)
    }

    pub fn restore_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        let project = self.service.restore_project(&project_id)?;
        project_json(project)
    }

    pub fn active_project_json(&self) -> MobileCoreResult<String> {
        let project = self.service.active_project()?;
        project_option_json(project)
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

    pub fn project_asset_group_page_json(
        &self,
        project_id: String,
        query_json: String,
        offset: u32,
        limit: u32,
    ) -> MobileCoreResult<String> {
        let query = asset_group_query_from_json(&query_json)?;
        let page: AssetGroupPage = self.service.project_asset_group_page_with_query(
            &project_id,
            query,
            offset as usize,
            limit as usize,
        )?;
        Ok(serde_json::to_string(&page)?)
    }

    pub fn create_lan_share_session_json(
        &self,
        project_id: String,
        query_json: String,
        title: String,
    ) -> MobileCoreResult<String> {
        let query = asset_group_query_from_json(&query_json)?;
        let session =
            self.service
                .create_lan_share_session(&project_id, query, non_blank(title))?;
        Ok(serde_json::to_string(&session)?)
    }

    pub fn stop_lan_share_session_json(&self, share_id: String) -> MobileCoreResult<String> {
        let session = self.service.stop_lan_share_session(&share_id)?;
        Ok(serde_json::to_string(&session)?)
    }

    pub fn lan_share_asset_group_page_json(
        &self,
        token: String,
        offset: u32,
        limit: u32,
    ) -> MobileCoreResult<String> {
        let page =
            self.service
                .lan_share_asset_group_page(&token, offset as usize, limit as usize)?;
        Ok(serde_json::to_string(&page)?)
    }

    pub fn set_lan_share_guest_mark_json(
        &self,
        token: String,
        asset_group_id: String,
        patch_json: String,
    ) -> MobileCoreResult<String> {
        let patch: MobileGuestMarkPatch = if patch_json.trim().is_empty() {
            MobileGuestMarkPatch::default()
        } else {
            serde_json::from_str(&patch_json)?
        };
        let guest_mark = guest_mark_from_patch(patch.guest_mark)?;
        let mark = self
            .service
            .set_lan_share_guest_mark(&token, &asset_group_id, guest_mark)?;
        Ok(serde_json::to_string(&mark.map(|mark| {
            json!({
                "share_id": mark.share_id,
                "project_id": mark.project_id,
                "asset_group_id": mark.asset_group_id,
                "guest_mark": mark.guest_mark,
                "updated_at_ms": mark.updated_at_ms,
            })
        }))?)
    }

    pub fn project_group_assets_json(
        &self,
        project_id: String,
        group_id: String,
    ) -> MobileCoreResult<String> {
        let assets = self.service.project_group_assets(&project_id, &group_id)?;
        Ok(serde_json::to_string(&assets)?)
    }

    pub fn delete_project_group_json(
        &self,
        project_id: String,
        group_id: String,
    ) -> MobileCoreResult<String> {
        let deleted = self
            .service
            .delete_project_asset_group(&project_id, &group_id)?;
        Ok(serde_json::to_string(&json!({
            "project_id": project_id,
            "group_id": group_id,
            "deleted": deleted,
        }))?)
    }

    pub fn set_asset_group_user_marks_json(
        &self,
        project_id: String,
        group_id: String,
        patch_json: String,
    ) -> MobileCoreResult<String> {
        let patch: MobileUserMarksPatch = if patch_json.trim().is_empty() {
            MobileUserMarksPatch::default()
        } else {
            serde_json::from_str(&patch_json)?
        };
        let marks = self.service.set_asset_group_user_marks(
            &project_id,
            &group_id,
            patch.favorite,
            patch.marked,
        )?;
        user_marks_json(marks)
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

    pub fn release_failed_publish_retries_json(
        &self,
        project_id: String,
    ) -> MobileCoreResult<String> {
        let released_count = self.service.release_failed_publish_retries(&project_id)?;
        Ok(serde_json::to_string(&json!({
            "project_id": project_id,
            "released_count": released_count,
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

fn current_time_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}

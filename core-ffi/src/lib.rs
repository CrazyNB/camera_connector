use std::path::PathBuf;

use camera_connector_core::{
    AssetGroupQuery, CameraConnectorDashboard, CameraConnectorService, ImporterError, PushProtocol,
    ReceiverSettingsConfig, ReceiverSettingsUpdate,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum MobileCoreError {
    #[error("{0}")]
    Core(#[from] ImporterError),
    #[error("invalid protocol: {0}")]
    InvalidProtocol(String),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

pub type MobileCoreResult<T> = std::result::Result<T, MobileCoreError>;

#[derive(Debug, Clone)]
pub struct MobileCore {
    service: CameraConnectorService,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileAccountView {
    pub username: String,
    pub device_name: String,
    pub password_configured: bool,
}

impl MobileCore {
    pub fn new(config_path: Option<String>) -> Self {
        Self {
            service: CameraConnectorService::new(config_path.map(PathBuf::from)),
        }
    }

    pub fn config_path(&self) -> String {
        self.service.config_path().to_string_lossy().into_owned()
    }

    pub fn default_state_dir(&self) -> String {
        self.service.state_dir().to_string_lossy().into_owned()
    }

    pub fn dashboard_json(
        &self,
        state_dir: Option<String>,
        offset: u32,
        limit: u32,
    ) -> MobileCoreResult<String> {
        let state_dir = state_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| self.service.state_dir());
        let dashboard: CameraConnectorDashboard = self.service.dashboard(
            state_dir,
            AssetGroupQuery::default(),
            offset as usize,
            limit as usize,
            false,
        )?;
        Ok(serde_json::to_string(&dashboard)?)
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

#[allow(dead_code)]
fn _assert_settings_config_is_serializable(settings: &ReceiverSettingsConfig) -> String {
    serde_json::to_string(settings).expect("receiver settings should serialize")
}

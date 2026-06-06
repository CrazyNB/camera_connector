use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::PasswordHash;
use serde::{Deserialize, Serialize};

use crate::{
    ImporterError, PublishQueueItem, PublishTransferMetadata, Result, SqliteStore, TransferRecord,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PushProtocol {
    #[default]
    Ftp,
    Sftp,
}

impl FromStr for PushProtocol {
    type Err = ImporterError;

    fn from_str(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "ftp" => Ok(Self::Ftp),
            "sftp" => Ok(Self::Sftp),
            _ => Err(ImporterError::UnsupportedProtocol),
        }
    }
}

impl std::fmt::Display for PushProtocol {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ftp => formatter.write_str("ftp"),
            Self::Sftp => formatter.write_str("sftp"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiverAccount {
    pub username: String,
    pub password: Option<ReceiverPassword>,
    pub device_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiverPassword {
    Plain(String),
    Argon2id(String),
}

impl ReceiverPassword {
    pub fn plain(value: impl Into<String>) -> Self {
        Self::Plain(value.into())
    }

    pub fn argon2id(hash: impl Into<String>) -> Self {
        Self::Argon2id(hash.into())
    }

    pub fn hash(password: &str) -> Result<Self> {
        let salt = password_hash::SaltString::generate(&mut rand_core::OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| Self::Argon2id(hash.to_string()))
            .map_err(|error| ImporterError::internal(error.to_string()))
    }

    pub fn verify(&self, candidate: &str) -> Result<bool> {
        match self {
            Self::Plain(expected) => Ok(expected == candidate),
            Self::Argon2id(hash) => {
                let parsed = PasswordHash::new(hash)
                    .map_err(|error| ImporterError::internal(error.to_string()))?;
                match Argon2::default().verify_password(candidate.as_bytes(), &parsed) {
                    Ok(()) => Ok(true),
                    Err(password_hash::Error::Password) => Ok(false),
                    Err(error) => Err(ImporterError::internal(error.to_string())),
                }
            }
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Plain(_) => Ok(()),
            Self::Argon2id(hash) => PasswordHash::new(hash)
                .map(|_| ())
                .map_err(|error| ImporterError::internal(error.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiverAccountConfig {
    pub username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    #[serde(default, skip_serializing)]
    pub password: Option<String>,
    pub device_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraConnectorConfig {
    #[serde(default)]
    pub receiver: ReceiverSettingsConfig,
    #[serde(default)]
    pub accounts: BTreeMap<String, ReceiverAccountConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_providers: Vec<ModelProviderSettingsConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelProviderSettingsConfig {
    #[serde(default = "default_model_provider_settings_id")]
    pub settings_id: String,
    #[serde(default)]
    pub provider_kind: String,
    #[serde(default)]
    pub provider_label: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub default_model: String,
    #[serde(default = "default_model_max_image_side")]
    pub default_max_image_side: i64,
    #[serde(default = "default_model_send_mode")]
    pub default_send_mode: String,
    #[serde(default = "default_model_batch_size")]
    pub default_batch_size: i64,
    #[serde(default)]
    pub configured: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_alias: Option<String>,
    #[serde(default)]
    pub updated_at_ms: i64,
}

impl Default for ModelProviderSettingsConfig {
    fn default() -> Self {
        Self {
            settings_id: default_model_provider_settings_id(),
            provider_kind: "none".to_string(),
            provider_label: String::new(),
            base_url: String::new(),
            default_model: String::new(),
            default_max_image_side: default_model_max_image_side(),
            default_send_mode: default_model_send_mode(),
            default_batch_size: default_model_batch_size(),
            configured: false,
            api_key: None,
            key_alias: None,
            updated_at_ms: 0,
        }
    }
}

fn default_model_provider_settings_id() -> String {
    "global".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiverSettingsConfig {
    #[serde(default)]
    pub protocol: PushProtocol,
    #[serde(default = "default_bind_host")]
    pub bind_host: String,
    #[serde(default = "default_ftp_port")]
    pub ftp_port: u16,
    #[serde(default = "default_sftp_port")]
    pub sftp_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advertised_host: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,
    #[serde(default)]
    pub defer_publish: bool,
}

impl Default for ReceiverSettingsConfig {
    fn default() -> Self {
        Self {
            protocol: PushProtocol::Ftp,
            bind_host: default_bind_host(),
            ftp_port: default_ftp_port(),
            sftp_port: default_sftp_port(),
            output_dir: None,
            state_dir: None,
            advertised_host: None,
            source_name: None,
            defer_publish: false,
        }
    }
}

impl CameraConnectorConfig {
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        let path = resolved_config_path(config_path);
        if !path.exists() {
            return Ok(Self::default());
        }

        let bytes = fs::read(&path)?;
        let mut config: Self = serde_json::from_slice(&bytes)
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        config.accounts = config
            .accounts
            .into_values()
            .map(ReceiverAccountConfig::validated)
            .map(|result| result.map(|account| (account.username.clone(), account)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok(config)
    }

    pub fn save(&self, config_path: Option<&Path>) -> Result<PathBuf> {
        let path = resolved_config_path(config_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self)
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        fs::write(&path, json)?;
        Ok(path)
    }

    pub fn resolved_path(config_path: Option<&Path>) -> PathBuf {
        resolved_config_path(config_path)
    }

    pub fn default_path() -> PathBuf {
        default_config_path()
    }

    pub fn default_state_dir(config_path: Option<&Path>) -> PathBuf {
        resolved_config_path(config_path)
            .parent()
            .map(|parent| parent.join("state"))
            .unwrap_or_else(|| PathBuf::from("camera-connector-state"))
    }

    pub fn default_output_dir() -> PathBuf {
        default_output_dir()
    }

    pub fn set_account(
        &mut self,
        username: impl Into<String>,
        password: Option<&str>,
        device_name: impl Into<String>,
    ) -> Result<&ReceiverAccountConfig> {
        let account = ReceiverAccountConfig::new(username, password, device_name)?;
        let username = account.username.clone();
        self.accounts.insert(username.clone(), account);
        self.accounts
            .get(&username)
            .ok_or_else(|| ImporterError::internal("saved account is missing"))
    }

    pub fn remove_account(&mut self, username: &str) -> Option<ReceiverAccountConfig> {
        self.accounts.remove(username)
    }

    pub fn effective_accounts(
        self,
        username: Option<&str>,
        password: Option<&str>,
        device_name: Option<&str>,
    ) -> Result<Vec<ReceiverAccount>> {
        let mut accounts = self
            .accounts
            .into_values()
            .map(ReceiverAccountConfig::into_receiver_account)
            .collect::<Vec<_>>();

        if let Some(username) = username {
            let transient =
                ReceiverAccountConfig::new(username, password, device_name.unwrap_or(username))?
                    .into_receiver_account();
            accounts.retain(|account| account.username != username);
            accounts.push(transient);
        }

        Ok(accounts)
    }
}

impl ReceiverAccountConfig {
    pub fn new(
        username: impl Into<String>,
        password: Option<&str>,
        device_name: impl Into<String>,
    ) -> Result<Self> {
        Self {
            username: username.into(),
            password_hash: password
                .map(ReceiverPassword::hash)
                .transpose()?
                .map(|password| {
                    let ReceiverPassword::Argon2id(hash) = password else {
                        unreachable!("ReceiverPassword::hash always returns an argon2id hash")
                    };
                    hash
                }),
            password: None,
            device_name: device_name.into(),
        }
        .validated()
    }

    pub fn password_configured(&self) -> bool {
        self.password_hash.is_some() || self.password.is_some()
    }

    pub fn into_receiver_account(self) -> ReceiverAccount {
        ReceiverAccount {
            username: self.username,
            password: self.password_hash.map(ReceiverPassword::argon2id),
            device_name: self.device_name,
        }
    }

    pub fn validated(mut self) -> Result<Self> {
        self.username = normalized_required("account username", &self.username)?;
        self.device_name = normalized_required("account device name", &self.device_name)?;
        if self.password_hash.is_none() {
            if let Some(password) = self.password.take() {
                let ReceiverPassword::Argon2id(hash) = ReceiverPassword::hash(&password)? else {
                    unreachable!("ReceiverPassword::hash always returns an argon2id hash")
                };
                self.password_hash = Some(hash);
            }
        }
        self.password = None;
        self.clone().into_receiver_account().validate()?;
        Ok(self)
    }
}

impl ReceiverAccount {
    pub fn new(
        username: impl Into<String>,
        password: Option<impl Into<String>>,
        device_name: impl Into<String>,
    ) -> Self {
        Self {
            username: username.into(),
            password: password.map(ReceiverPassword::plain),
            device_name: device_name.into(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.username.trim().is_empty() {
            return Err(ImporterError::internal("account username cannot be empty"));
        }
        if self.device_name.trim().is_empty() {
            return Err(ImporterError::internal(
                "account device name cannot be empty",
            ));
        }
        if let Some(password) = &self.password {
            password.validate()?;
        }
        Ok(())
    }
}

fn normalized_required(field: &str, value: &str) -> Result<String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(ImporterError::internal(format!("{field} cannot be empty")));
    }
    Ok(normalized)
}

fn resolved_config_path(config_path: Option<&Path>) -> PathBuf {
    config_path
        .map(Path::to_path_buf)
        .unwrap_or_else(default_config_path)
}

fn default_config_path() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata)
            .join("CameraConnector")
            .join("config.json");
    }
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        return PathBuf::from(home)
            .join(".camera-connector")
            .join("config.json");
    }
    PathBuf::from("camera-connector-config.json")
}

fn default_output_dir() -> PathBuf {
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        return PathBuf::from(profile)
            .join("Pictures")
            .join("CameraConnector");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("Pictures").join("CameraConnector");
    }
    PathBuf::from("CameraConnector")
}

fn default_bind_host() -> String {
    "0.0.0.0".to_string()
}

fn default_ftp_port() -> u16 {
    2121
}

fn default_sftp_port() -> u16 {
    2222
}

fn default_model_max_image_side() -> i64 {
    1024
}

fn default_model_send_mode() -> String {
    "preview_only".to_string()
}

fn default_model_batch_size() -> i64 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushReceiverConfig {
    pub protocol: PushProtocol,
    pub bind_host: String,
    pub port: u16,
    pub output_dir: PathBuf,
    pub state_dir: PathBuf,
    pub username: Option<String>,
    pub password: Option<String>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
    pub accounts: Vec<ReceiverAccount>,
    pub active_project_id: Option<String>,
    pub defer_publish: bool,
}

impl PushReceiverConfig {
    pub fn new(
        protocol: PushProtocol,
        bind_host: impl Into<String>,
        port: u16,
        output_dir: impl AsRef<Path>,
    ) -> Self {
        let output_dir = output_dir.as_ref().to_path_buf();
        Self {
            protocol,
            bind_host: bind_host.into(),
            port,
            state_dir: output_dir.clone(),
            output_dir,
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
            accounts: Vec::new(),
            active_project_id: None,
            defer_publish: false,
        }
    }

    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    pub fn with_state_dir(mut self, state_dir: impl AsRef<Path>) -> Self {
        self.state_dir = state_dir.as_ref().to_path_buf();
        self
    }

    pub fn with_advertised_host(mut self, host: impl Into<String>) -> Self {
        self.advertised_host = Some(host.into());
        self
    }

    pub fn with_source_name(mut self, source_name: impl Into<String>) -> Self {
        self.source_name = Some(source_name.into());
        self
    }

    pub fn with_account(mut self, account: ReceiverAccount) -> Self {
        self.accounts.push(account);
        self
    }

    pub fn with_active_project(mut self, project_id: impl Into<String>) -> Self {
        self.active_project_id = Some(project_id.into());
        self
    }

    pub fn with_deferred_publish(mut self) -> Self {
        self.defer_publish = true;
        self
    }

    pub fn staging_dir(&self) -> PathBuf {
        self.state_dir.join("staging")
    }

    pub fn resolved_source_name(&self, _remote_addr: Option<&str>) -> Option<String> {
        self.source_name.clone()
    }

    pub fn account_for_username(&self, username: &str) -> Option<&ReceiverAccount> {
        self.accounts
            .iter()
            .find(|account| account.username == username)
    }

    pub fn validate_accounts(&self) -> Result<()> {
        for account in &self.accounts {
            account.validate()?;
        }
        Ok(())
    }

    pub fn record_storage_transfer(&self, record: &TransferRecord) -> Result<()> {
        let project_id = self.resolve_storage_project_id()?;
        SqliteStore::open_state_dir(&self.state_dir)?.record_transfer(&project_id, record.clone())
    }

    pub fn enqueue_publish(
        &self,
        transfer_id: &str,
        staged_path: &str,
        final_filename: &str,
        size_bytes: u64,
    ) -> Result<PublishQueueItem> {
        let store = SqliteStore::open_state_dir(&self.state_dir)?;
        let project_id = self.resolve_storage_project_id_with_store(&store)?;
        store.enqueue_publish(
            &project_id,
            transfer_id,
            staged_path,
            final_filename,
            size_bytes,
        )
    }

    pub fn enqueue_publish_with_metadata(
        &self,
        transfer_id: &str,
        staged_path: &str,
        final_filename: &str,
        size_bytes: u64,
        metadata: PublishTransferMetadata,
    ) -> Result<PublishQueueItem> {
        let store = SqliteStore::open_state_dir(&self.state_dir)?;
        let project_id = self.resolve_storage_project_id_with_store(&store)?;
        store.enqueue_publish_with_metadata(
            &project_id,
            transfer_id,
            staged_path,
            final_filename,
            size_bytes,
            metadata,
        )
    }

    pub fn mark_publish_completed(&self, queue_id: &str) -> Result<()> {
        SqliteStore::open_state_dir(&self.state_dir)?.mark_publish_completed(queue_id)
    }

    pub fn mark_publish_failed(&self, queue_id: &str, error: &str) -> Result<()> {
        SqliteStore::open_state_dir(&self.state_dir)?.mark_publish_failed(queue_id, error)
    }

    fn resolve_storage_project_id(&self) -> Result<String> {
        let store = SqliteStore::open_state_dir(&self.state_dir)?;
        self.resolve_storage_project_id_with_store(&store)
    }

    fn resolve_storage_project_id_with_store(&self, store: &SqliteStore) -> Result<String> {
        match self.active_project_id.as_deref() {
            Some(project_id) => {
                store
                    .list_projects()?
                    .into_iter()
                    .find(|project| project.project_id == project_id)
                    .ok_or_else(|| ImporterError::internal("active project not found"))?;
                Ok(project_id.to_string())
            }
            None => Err(ImporterError::internal(
                "no active project selected; enter a project before receiving files",
            )),
        }
    }
}

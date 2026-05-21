use std::path::{Path, PathBuf};
use std::str::FromStr;

use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use password_hash::PasswordHash;
use serde::{Deserialize, Serialize};

use crate::{ImporterError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PushProtocol {
    Ftp,
    Sftp,
    Ftps,
}

impl FromStr for PushProtocol {
    type Err = ImporterError;

    fn from_str(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "ftp" => Ok(Self::Ftp),
            "sftp" => Ok(Self::Sftp),
            "ftps" => Ok(Self::Ftps),
            _ => Err(ImporterError::UnsupportedProtocol),
        }
    }
}

impl std::fmt::Display for PushProtocol {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ftp => formatter.write_str("ftp"),
            Self::Sftp => formatter.write_str("sftp"),
            Self::Ftps => formatter.write_str("ftps"),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushReceiverConfig {
    pub protocol: PushProtocol,
    pub bind_host: String,
    pub port: u16,
    pub output_dir: PathBuf,
    pub username: Option<String>,
    pub password: Option<String>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
    pub accounts: Vec<ReceiverAccount>,
}

impl PushReceiverConfig {
    pub fn new(
        protocol: PushProtocol,
        bind_host: impl Into<String>,
        port: u16,
        output_dir: impl AsRef<Path>,
    ) -> Self {
        Self {
            protocol,
            bind_host: bind_host.into(),
            port,
            output_dir: output_dir.as_ref().to_path_buf(),
            username: None,
            password: None,
            advertised_host: None,
            source_name: None,
            accounts: Vec::new(),
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
}

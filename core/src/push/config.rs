use std::path::{Path, PathBuf};
use std::str::FromStr;

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
pub struct PushReceiverConfig {
    pub protocol: PushProtocol,
    pub bind_host: String,
    pub port: u16,
    pub output_dir: PathBuf,
    pub username: Option<String>,
    pub password: Option<String>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
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
}

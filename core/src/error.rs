use std::io;

use thiserror::Error;
use tokio::time::error::Elapsed;

pub type Result<T> = std::result::Result<T, ImporterError>;

#[derive(Debug, Error)]
pub enum ImporterError {
    #[error("network unavailable")]
    NetworkUnavailable,
    #[error("connection timeout")]
    ConnectionTimeout,
    #[error("unsupported protocol")]
    UnsupportedProtocol,
    #[error("authentication failed")]
    AuthenticationFailed,
    #[error("invalid upload path")]
    InvalidUploadPath,
    #[error("receive interrupted")]
    ReceiveInterrupted,
    #[error("storage permission denied")]
    StoragePermissionDenied,
    #[error("local network permission denied")]
    LocalNetworkPermissionDenied,
    #[error("unknown receiver command")]
    UnknownReceiverCommand,
    #[error("internal error: {message}")]
    InternalError { message: String },
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

impl ImporterError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NetworkUnavailable => "NetworkUnavailable",
            Self::ConnectionTimeout => "ConnectionTimeout",
            Self::UnsupportedProtocol => "UnsupportedProtocol",
            Self::AuthenticationFailed => "AuthenticationFailed",
            Self::InvalidUploadPath => "InvalidUploadPath",
            Self::ReceiveInterrupted => "ReceiveInterrupted",
            Self::StoragePermissionDenied => "StoragePermissionDenied",
            Self::LocalNetworkPermissionDenied => "LocalNetworkPermissionDenied",
            Self::UnknownReceiverCommand => "UnknownReceiverCommand",
            Self::InternalError { .. } => "InternalError",
            Self::Io(_) => "Io",
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }
}

impl From<Elapsed> for ImporterError {
    fn from(_: Elapsed) -> Self {
        Self::ConnectionTimeout
    }
}

use std::io;

use thiserror::Error;
use tokio::time::error::Elapsed;

pub type Result<T> = std::result::Result<T, ImporterError>;

#[derive(Debug, Error)]
pub enum ImporterError {
    #[error("network unavailable")]
    NetworkUnavailable,
    #[error("camera not found")]
    CameraNotFound,
    #[error("connection timeout")]
    ConnectionTimeout,
    #[error("ptp/ip init failed")]
    PtpInitFailed,
    #[error("ptp session open failed")]
    SessionOpenFailed,
    #[error("unsupported operation")]
    UnsupportedOperation,
    #[error("object not found")]
    ObjectNotFound,
    #[error("thumbnail unavailable")]
    ThumbnailUnavailable,
    #[error("download interrupted")]
    DownloadInterrupted,
    #[error("storage permission denied")]
    StoragePermissionDenied,
    #[error("local network permission denied")]
    LocalNetworkPermissionDenied,
    #[error("unknown camera response")]
    UnknownCameraResponse,
    #[error("internal error: {message}")]
    InternalError { message: String },
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

impl ImporterError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NetworkUnavailable => "NetworkUnavailable",
            Self::CameraNotFound => "CameraNotFound",
            Self::ConnectionTimeout => "ConnectionTimeout",
            Self::PtpInitFailed => "PtpInitFailed",
            Self::SessionOpenFailed => "SessionOpenFailed",
            Self::UnsupportedOperation => "UnsupportedOperation",
            Self::ObjectNotFound => "ObjectNotFound",
            Self::ThumbnailUnavailable => "ThumbnailUnavailable",
            Self::DownloadInterrupted => "DownloadInterrupted",
            Self::StoragePermissionDenied => "StoragePermissionDenied",
            Self::LocalNetworkPermissionDenied => "LocalNetworkPermissionDenied",
            Self::UnknownCameraResponse => "UnknownCameraResponse",
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

use camera_connector_core::ImporterError;

#[derive(Debug, thiserror::Error)]
pub enum MobileCoreError {
    #[error("{0}")]
    Core(#[from] ImporterError),
    #[error("invalid protocol: {0}")]
    InvalidProtocol(String),
    #[error("invalid storage location kind: {0}")]
    InvalidLocationKind(String),
    #[error("invalid {field}: {value}")]
    InvalidConfigValue { field: &'static str, value: String },
    #[error("invalid asset format: {0}")]
    InvalidAssetFormat(String),
    #[error("invalid asset role: {0}")]
    InvalidAssetRole(String),
    #[error("invalid guest mark: {0}")]
    InvalidGuestMark(String),
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

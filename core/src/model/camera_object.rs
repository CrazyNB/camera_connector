use serde::{Deserialize, Serialize};

use super::ObjectFormat;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CameraObject {
    pub handle: u32,
    pub storage_id: u32,
    pub filename: String,
    pub size_bytes: u64,
    pub format: ObjectFormat,
    pub capture_time_ms: Option<i64>,
    pub modified_time_ms: Option<i64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub thumb_available: bool,
    pub downloaded: bool,
    pub group_key: Option<String>,
}

impl CameraObject {
    pub fn new(handle: u32, storage_id: u32, filename: impl Into<String>, size_bytes: u64) -> Self {
        let filename = filename.into();
        let format = ObjectFormat::from_filename(&filename);
        let group_key = group_key_from_filename(&filename);

        Self {
            handle,
            storage_id,
            filename,
            size_bytes,
            format,
            capture_time_ms: None,
            modified_time_ms: None,
            width: None,
            height: None,
            thumb_available: false,
            downloaded: false,
            group_key,
        }
    }
}

pub(crate) fn group_key_from_filename(filename: &str) -> Option<String> {
    let (stem, _) = filename.rsplit_once('.')?;
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_ascii_uppercase())
    }
}

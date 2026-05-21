use serde::{Deserialize, Serialize};

use super::ObjectFormat;
use crate::StoredObjectLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImportSource {
    FtpPush,
    SftpPush,
    ManualDrop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceivedAsset {
    pub id: String,
    pub filename: String,
    pub size_bytes: u64,
    pub format: ObjectFormat,
    pub source: ImportSource,
    pub received_time_ms: Option<i64>,
    pub capture_time_ms: Option<i64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub group_key: Option<String>,
    pub storage_location: Option<StoredObjectLocation>,
    pub original_path: Option<String>,
    pub display_source: Option<String>,
    pub remote_addr: Option<String>,
    pub virtual_display_path: Option<String>,
}

impl ReceivedAsset {
    pub fn new(
        id: impl Into<String>,
        filename: impl Into<String>,
        size_bytes: u64,
        source: ImportSource,
    ) -> Self {
        let filename = filename.into();
        let format = ObjectFormat::from_filename(&filename);
        let group_key = group_key_from_filename(&filename);

        Self {
            id: id.into(),
            filename,
            size_bytes,
            format,
            source,
            received_time_ms: None,
            capture_time_ms: None,
            width: None,
            height: None,
            group_key,
            storage_location: None,
            original_path: None,
            display_source: None,
            remote_addr: None,
            virtual_display_path: None,
        }
    }

    pub fn with_storage_location(mut self, location: StoredObjectLocation) -> Self {
        self.storage_location = Some(location);
        self
    }
}

pub(crate) fn group_key_from_filename(filename: &str) -> Option<String> {
    let name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    let (stem, _) = name.rsplit_once('.')?;
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_ascii_uppercase())
    }
}

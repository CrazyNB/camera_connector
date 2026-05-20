use serde::{Deserialize, Serialize};

use super::ObjectFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImportSource {
    FtpPush,
    SftpPush,
    FtpsPush,
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
        }
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

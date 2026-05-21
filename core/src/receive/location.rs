use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StoredObjectLocation {
    LocalPath { path: PathBuf },
    DocumentUri { uri: String },
    MediaUri { uri: String },
    PhotoAsset { local_identifier: String },
}

impl StoredObjectLocation {
    pub fn local_path(path: impl Into<PathBuf>) -> Self {
        Self::LocalPath { path: path.into() }
    }

    pub fn document_uri(uri: impl Into<String>) -> Self {
        Self::DocumentUri { uri: uri.into() }
    }

    pub fn media_uri(uri: impl Into<String>) -> Self {
        Self::MediaUri { uri: uri.into() }
    }

    pub fn photo_asset(local_identifier: impl Into<String>) -> Self {
        Self::PhotoAsset {
            local_identifier: local_identifier.into(),
        }
    }

    pub fn as_local_path(&self) -> Option<&Path> {
        match self {
            Self::LocalPath { path } => Some(path.as_path()),
            _ => None,
        }
    }
}

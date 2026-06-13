use rand::{distr::Alphanumeric, RngExt};
use serde::{Deserialize, Serialize};

use crate::AssetGroupQuery;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuestMark {
    Favorite,
    Marked,
    Reject,
}

impl GuestMark {
    pub fn from_wire(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "favorite" => Some(Self::Favorite),
            "marked" => Some(Self::Marked),
            "reject" => Some(Self::Reject),
            _ => None,
        }
    }

    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Favorite => "favorite",
            Self::Marked => "marked",
            Self::Reject => "reject",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanShareSession {
    pub share_id: String,
    pub project_id: String,
    pub token: String,
    pub query: AssetGroupQuery,
    pub title: Option<String>,
    pub active: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub stopped_at_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanShareGuestMark {
    pub share_id: String,
    pub project_id: String,
    pub asset_group_id: String,
    pub guest_mark: GuestMark,
    pub updated_at_ms: i64,
}

pub fn generate_lan_share_token() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

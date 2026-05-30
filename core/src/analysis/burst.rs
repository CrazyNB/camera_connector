use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BurstGroup {
    pub burst_group_id: String,
    pub project_id: String,
    pub source_identity: Option<String>,
    pub started_at_ms: Option<i64>,
    pub ended_at_ms: Option<i64>,
    pub member_group_ids: Vec<String>,
    pub member_count: usize,
    pub grouping_version: i64,
    pub recommendation_status: String,
    pub user_override_state: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

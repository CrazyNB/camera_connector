use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BurstGroupingProfile {
    pub grouping_version: String,
    pub burst_window_ms: i64,
    pub min_group_size: usize,
    pub visual_continuity_threshold: f64,
}

impl Default for BurstGroupingProfile {
    fn default() -> Self {
        Self {
            grouping_version: "burst-grouping-v1".to_string(),
            burst_window_ms: 1200,
            min_group_size: 2,
            visual_continuity_threshold: 0.80,
        }
    }
}

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
    pub manual_grouping_state: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

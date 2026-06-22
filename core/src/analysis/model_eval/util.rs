use std::collections::BTreeSet;

use crate::{ImporterError, Result};

pub(super) fn valid_unique_ids(
    ids: Vec<String>,
    known_ids: &BTreeSet<String>,
    blocked_ids: &BTreeSet<String>,
) -> Vec<String> {
    let mut seen = BTreeSet::new();
    ids.into_iter()
        .filter(|id| known_ids.contains(id) && !blocked_ids.contains(id))
        .filter(|id| seen.insert(id.clone()))
        .collect()
}

pub(super) fn chat_completions_endpoint(base_url: &str) -> Result<String> {
    let base_url = base_url.trim().trim_end_matches('/');
    if base_url.is_empty() {
        return Err(ImporterError::internal("model provider base URL is empty"));
    }
    if base_url.ends_with("/chat/completions") {
        Ok(base_url.to_string())
    } else {
        Ok(format!("{base_url}/chat/completions"))
    }
}

pub(super) fn model_evaluation_id(project_id: &str, asset_group_id: &str, evaluator_version: &str) -> String {
    let key = format!("{project_id}:{asset_group_id}:{evaluator_version}");
    let mut hash = 1469598103934665603_u64;
    for byte in key.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("model-evaluation-{hash:016x}")
}

pub(super) fn model_evaluation_run_id(
    project_id: &str,
    asset_group_id: &str,
    evaluator_version: &str,
    now_ms: i64,
) -> String {
    let key = format!("{project_id}:{asset_group_id}:{evaluator_version}:{now_ms}");
    let mut hash = 1469598103934665603_u64;
    for byte in key.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("evaluation-run-{hash:016x}")
}

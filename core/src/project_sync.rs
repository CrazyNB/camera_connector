use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{ImporterError, ObjectFormat, Result, StoredAsset, StoredAssetGroup};

pub const PROJECT_SYNC_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshot {
    pub schema_version: i64,
    pub source_device: ProjectSyncSourceDevice,
    pub project: ProjectSyncProjectSummary,
    #[serde(default)]
    pub assets: Vec<ProjectSyncSnapshotAsset>,
    #[serde(default)]
    pub groups: Vec<ProjectSyncSnapshotGroup>,
    #[serde(default)]
    pub model_evaluations: Vec<ProjectSyncSnapshotModelEvaluation>,
    #[serde(default)]
    pub selection_recommendations: Vec<ProjectSyncSnapshotRecommendation>,
    #[serde(default)]
    pub user_marks: Vec<ProjectSyncSnapshotUserMarks>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncSourceDevice {
    pub device_id: String,
    pub device_label: String,
    pub platform: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncProjectSummary {
    pub project_id: String,
    pub name: String,
    pub exported_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshotAsset {
    pub asset_id: String,
    pub group_id: String,
    pub original_filename: String,
    pub final_filename: String,
    pub normalized_stem: String,
    pub original_path: String,
    pub original_parent_path: Option<String>,
    pub format: String,
    pub size_bytes: u64,
    pub capture_at_ms: Option<i64>,
    pub received_at_ms: Option<i64>,
    pub source_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshotGroup {
    pub group_id: String,
    pub display_key: String,
    pub source_identity: Option<String>,
    pub original_parent_path: Option<String>,
    pub member_asset_ids: Vec<String>,
    pub primary_asset_id: Option<String>,
    pub preview_asset_id: Option<String>,
    pub has_raw: bool,
    pub has_jpeg: bool,
    pub has_video: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshotModelEvaluation {
    pub evaluation_id: String,
    pub group_id: String,
    pub evaluator_version: String,
    pub status: String,
    pub score: i64,
    pub tier: String,
    pub selectable: bool,
    pub summary: String,
    #[serde(default)]
    pub strengths: Vec<String>,
    #[serde(default)]
    pub weaknesses: Vec<String>,
    #[serde(default)]
    pub technical_warnings: Vec<String>,
    pub prompt_pack_id: Option<String>,
    pub prompt_pack_version: Option<String>,
    pub prompt_hash: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshotRecommendation {
    pub recommendation_id: String,
    pub scope: String,
    pub subject_group_id: Option<String>,
    #[serde(default)]
    pub selected_group_ids: Vec<String>,
    #[serde(default)]
    pub candidate_group_ids: Vec<String>,
    #[serde(default)]
    pub rejected_group_ids: Vec<String>,
    pub status: String,
    pub confidence: f64,
    pub reason: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshotUserMarks {
    pub group_id: String,
    pub favorite: Option<bool>,
    pub marked: Option<bool>,
}

pub fn parse_project_sync_snapshot_json(value: &str) -> Result<ProjectSyncSnapshot> {
    let snapshot: ProjectSyncSnapshot = serde_json::from_str(value).map_err(|error| {
        ImporterError::internal(format!("invalid project sync snapshot: {error}"))
    })?;
    if snapshot.schema_version != PROJECT_SYNC_SCHEMA_VERSION {
        return Err(ImporterError::internal(format!(
            "unsupported project sync schema_version {}",
            snapshot.schema_version
        )));
    }
    Ok(snapshot)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSyncMatchSummary {
    pub matched_assets: BTreeMap<String, String>,
    pub unmatched_assets: Vec<String>,
    pub ambiguous_assets: Vec<String>,
    pub matched_groups: BTreeMap<String, String>,
    pub unmatched_groups: Vec<String>,
    pub ambiguous_groups: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncApplySummary {
    pub matched_assets: usize,
    pub matched_groups: usize,
    pub applied_user_marks: usize,
    pub applied_model_evaluations: usize,
    pub applied_selection_recommendations: usize,
    pub unresolved_records: usize,
    pub ambiguous_records: usize,
}

pub fn match_project_sync_snapshot(
    snapshot: &ProjectSyncSnapshot,
    local_assets: &[StoredAsset],
    local_groups: &[StoredAssetGroup],
) -> ProjectSyncMatchSummary {
    let mut matched_assets = BTreeMap::new();
    let mut group_ids_by_snapshot_asset = BTreeMap::new();
    let mut unmatched_assets = Vec::new();
    let mut ambiguous_assets = Vec::new();

    for snapshot_asset in &snapshot.assets {
        match unique_asset_candidate(matching_assets(snapshot_asset, local_assets)) {
            CandidateResolution::Matched((_, asset)) => {
                matched_assets.insert(snapshot_asset.asset_id.clone(), asset.asset_id.clone());
                if let Some(group_id) = &asset.group_id {
                    group_ids_by_snapshot_asset
                        .insert(snapshot_asset.asset_id.clone(), group_id.clone());
                }
            }
            CandidateResolution::Unmatched => {
                unmatched_assets.push(snapshot_asset.asset_id.clone())
            }
            CandidateResolution::Ambiguous => {
                ambiguous_assets.push(snapshot_asset.asset_id.clone())
            }
        }
    }

    let mut matched_groups = BTreeMap::new();
    let mut unmatched_groups = Vec::new();
    let mut ambiguous_groups = Vec::new();

    for snapshot_group in &snapshot.groups {
        match unique_group_candidate(matching_groups(
            snapshot_group,
            local_groups,
            &group_ids_by_snapshot_asset,
        )) {
            CandidateResolution::Matched((_, group)) => {
                matched_groups.insert(snapshot_group.group_id.clone(), group.group_id.clone());
            }
            CandidateResolution::Unmatched => {
                unmatched_groups.push(snapshot_group.group_id.clone())
            }
            CandidateResolution::Ambiguous => {
                ambiguous_groups.push(snapshot_group.group_id.clone())
            }
        }
    }

    ProjectSyncMatchSummary {
        matched_assets,
        unmatched_assets,
        ambiguous_assets,
        matched_groups,
        unmatched_groups,
        ambiguous_groups,
    }
}

enum CandidateResolution<T> {
    Matched(T),
    Unmatched,
    Ambiguous,
}

fn unique_asset_candidate(
    candidates: Vec<(u8, &StoredAsset)>,
) -> CandidateResolution<(u8, &StoredAsset)> {
    if candidates.is_empty() {
        return CandidateResolution::Unmatched;
    }
    let best_rank = candidates[0].0;
    let best = candidates
        .into_iter()
        .filter(|(rank, _)| *rank == best_rank)
        .collect::<Vec<_>>();
    if best.len() == 1 {
        CandidateResolution::Matched(best[0])
    } else {
        CandidateResolution::Ambiguous
    }
}

fn unique_group_candidate(
    candidates: Vec<(u8, &StoredAssetGroup)>,
) -> CandidateResolution<(u8, &StoredAssetGroup)> {
    if candidates.is_empty() {
        return CandidateResolution::Unmatched;
    }
    let best_rank = candidates[0].0;
    let best = candidates
        .into_iter()
        .filter(|(rank, _)| *rank == best_rank)
        .collect::<Vec<_>>();
    if best.len() == 1 {
        CandidateResolution::Matched(best[0])
    } else {
        CandidateResolution::Ambiguous
    }
}

fn matching_assets<'a>(
    snapshot_asset: &ProjectSyncSnapshotAsset,
    local_assets: &'a [StoredAsset],
) -> Vec<(u8, &'a StoredAsset)> {
    let Ok(snapshot_format) = ObjectFormat::from_str(&snapshot_asset.format) else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    for asset in local_assets {
        if asset.format != snapshot_format {
            continue;
        }
        if filename_matches(asset, snapshot_asset)
            && asset.size_bytes == snapshot_asset.size_bytes
            && asset.capture_at_ms == snapshot_asset.capture_at_ms
        {
            candidates.push((1, asset));
            continue;
        }
        if asset
            .normalized_stem
            .eq_ignore_ascii_case(&snapshot_asset.normalized_stem)
            && asset.size_bytes == snapshot_asset.size_bytes
            && asset.capture_at_ms == snapshot_asset.capture_at_ms
        {
            candidates.push((2, asset));
            continue;
        }
        if filename_matches(asset, snapshot_asset) && asset.size_bytes == snapshot_asset.size_bytes
        {
            candidates.push((3, asset));
            continue;
        }
        if asset
            .normalized_stem
            .eq_ignore_ascii_case(&snapshot_asset.normalized_stem)
            && asset.size_bytes == snapshot_asset.size_bytes
        {
            candidates.push((4, asset));
            continue;
        }
        if asset
            .normalized_stem
            .eq_ignore_ascii_case(&snapshot_asset.normalized_stem)
        {
            candidates.push((5, asset));
        }
    }
    candidates.sort_by_key(|(rank, asset)| (*rank, asset.asset_id.clone()));
    candidates
}

fn filename_matches(asset: &StoredAsset, snapshot_asset: &ProjectSyncSnapshotAsset) -> bool {
    asset
        .original_filename
        .eq_ignore_ascii_case(&snapshot_asset.original_filename)
        || asset
            .original_filename
            .eq_ignore_ascii_case(&snapshot_asset.final_filename)
        || asset
            .final_filename
            .eq_ignore_ascii_case(&snapshot_asset.original_filename)
        || asset
            .final_filename
            .eq_ignore_ascii_case(&snapshot_asset.final_filename)
}

fn matching_groups<'a>(
    snapshot_group: &ProjectSyncSnapshotGroup,
    local_groups: &'a [StoredAssetGroup],
    group_ids_by_snapshot_asset: &BTreeMap<String, String>,
) -> Vec<(u8, &'a StoredAssetGroup)> {
    let matched_member_groups = snapshot_group
        .member_asset_ids
        .iter()
        .filter_map(|asset_id| group_ids_by_snapshot_asset.get(asset_id))
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut candidates = Vec::new();
    if !snapshot_group.member_asset_ids.is_empty()
        && matched_member_groups.len() == 1
        && matched_member_groups.len() == snapshot_group.member_asset_ids.len()
    {
        let local_group_id = matched_member_groups
            .iter()
            .next()
            .expect("one member group");
        candidates.extend(
            local_groups
                .iter()
                .filter(|group| &group.group_id == local_group_id)
                .map(|group| (1, group)),
        );
    }
    if matched_member_groups.len() == 1 {
        let local_group_id = matched_member_groups
            .iter()
            .next()
            .expect("one member group");
        candidates.extend(
            local_groups
                .iter()
                .filter(|group| &group.group_id == local_group_id)
                .map(|group| (2, group)),
        );
    }
    for group in local_groups {
        if group.source_identity == snapshot_group.source_identity
            && group
                .display_key
                .eq_ignore_ascii_case(&snapshot_group.display_key)
        {
            candidates.push((3, group));
            continue;
        }
        if group
            .display_key
            .eq_ignore_ascii_case(&snapshot_group.display_key)
        {
            candidates.push((4, group));
        }
    }
    candidates.sort_by_key(|(rank, group)| (*rank, group.group_id.clone()));
    candidates
}

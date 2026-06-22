use std::collections::BTreeMap;

use crate::{
    AssetFacetCount, AssetGroupQuery, AssetGroupSort, AssetGroupSummary, ReceivedAsset,
    ReceivedAssetGroup,
};

pub(super) fn summarize_asset_groups(groups: &[ReceivedAssetGroup]) -> AssetGroupSummary {
    let mut source_counts = BTreeMap::<String, usize>::new();
    let mut remote_addr_counts = BTreeMap::<String, usize>::new();
    for group in groups {
        if let Some(source) = group.primary.display_source.as_ref() {
            *source_counts.entry(source.clone()).or_default() += 1;
        }
        if let Some(remote_addr) = group.primary.remote_addr.as_ref() {
            *remote_addr_counts.entry(remote_addr.clone()).or_default() += 1;
        }
    }
    AssetGroupSummary {
        group_count: groups.len(),
        asset_count: groups.iter().map(|group| group_assets(group).len()).sum(),
        groups_with_jpeg: groups.iter().filter(|group| group.jpeg.is_some()).count(),
        groups_with_raw: groups.iter().filter(|group| group.raw.is_some()).count(),
        groups_with_video: groups.iter().filter(|group| group.video.is_some()).count(),
        source_counts: facet_counts(source_counts),
        remote_addr_counts: facet_counts(remote_addr_counts),
    }
}

pub(super) fn asset_group_matches(group: &ReceivedAssetGroup, query: &AssetGroupQuery) -> bool {
    group_assets(group).into_iter().any(|asset| {
        query
            .username
            .as_ref()
            .map(|expected| asset.username.as_ref() == Some(expected))
            .unwrap_or(true)
            && query
                .source_name
                .as_ref()
                .map(|expected| asset.display_source.as_ref() == Some(expected))
                .unwrap_or(true)
            && query
                .remote_addr
                .as_ref()
                .map(|expected| asset.remote_addr.as_ref() == Some(expected))
                .unwrap_or(true)
            && query
                .original_path
                .as_ref()
                .map(|expected| {
                    asset
                        .original_path
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&expected.to_ascii_lowercase())
                })
                .unwrap_or(true)
            && query
                .format
                .map(|expected| asset.format == expected)
                .unwrap_or(true)
            && query
                .role
                .map(|expected| asset.format.role() == expected)
                .unwrap_or(true)
    })
}

pub(super) fn asset_group_matches_analysis(
    group: &ReceivedAssetGroup,
    query: &AssetGroupQuery,
) -> bool {
    let favorite_matches = query
        .favorite
        .map(|expected| group.user_marks.favorite == expected)
        .unwrap_or(true);
    let marked_matches = query
        .marked
        .map(|expected| group.user_marks.marked == expected)
        .unwrap_or(true);
    let user_mark_any_matches = user_mark_any_matches(group, &query.user_mark_any);
    let guest_mark_matches = guest_mark_matches(group, query.guest_mark.as_deref());
    let collection_matches = query
        .collection
        .as_ref()
        .map(|collection| asset_group_matches_collection(group, collection))
        .unwrap_or(true);
    let score_matches = query
        .min_model_score
        .map(|minimum| {
            group_best_score(group)
                .map(score_for_threshold)
                .map(|score| score >= minimum)
                .unwrap_or(false)
        })
        .unwrap_or(true);

    favorite_matches
        && marked_matches
        && user_mark_any_matches
        && guest_mark_matches
        && collection_matches
        && score_matches
}

pub(super) fn sort_asset_groups_for_query(groups: &mut [ReceivedAssetGroup], sort: AssetGroupSort) {
    match sort {
        AssetGroupSort::LatestReceived => {}
        AssetGroupSort::Filename => {
            groups.sort_by(|left, right| {
                left.group_key
                    .cmp(&right.group_key)
                    .then_with(|| left.group_id.cmp(&right.group_id))
            });
        }
        AssetGroupSort::ModelScore => {
            groups.sort_by(|left, right| {
                let left_score = group_best_score(left);
                let right_score = group_best_score(right);
                let left_own_score = left.model_score.map(|score| score as f64);
                let right_own_score = right.model_score.map(|score| score as f64);
                score_sort_key(right_score)
                    .cmp(&score_sort_key(left_score))
                    .then_with(|| {
                        score_sort_key(right_own_score).cmp(&score_sort_key(left_own_score))
                    })
                    .then_with(|| {
                        group_received_sort_time(right).cmp(&group_received_sort_time(left))
                    })
                    .then_with(|| left.group_key.cmp(&right.group_key))
            });
        }
    }
}

fn user_mark_any_matches(group: &ReceivedAssetGroup, marks: &[String]) -> bool {
    if marks.is_empty() {
        return true;
    }
    marks
        .iter()
        .any(|mark| match mark.trim().to_ascii_lowercase().as_str() {
            "favorite" | "favorites" => group.user_marks.favorite,
            "marked" | "mark" | "flag" | "flagged" => group.user_marks.marked,
            _ => false,
        })
}

fn guest_mark_matches(group: &ReceivedAssetGroup, mark: Option<&str>) -> bool {
    match mark.map(|value| value.trim().to_ascii_lowercase()) {
        None => true,
        Some(value) if value.is_empty() || value == "all" => true,
        Some(value) if value == "none" || value == "unmarked" => group.guest_mark.is_none(),
        Some(value) => group
            .guest_mark
            .map(|guest_mark| guest_mark.as_wire() == value.as_str())
            .unwrap_or(false),
    }
}

fn asset_group_matches_collection(group: &ReceivedAssetGroup, collection: &str) -> bool {
    match normalized_asset_collection_key(collection).as_str() {
        "all" => true,
        "model_selects" => group.is_model_select,
        "favorites" => group.is_favorite,
        "marked" => group.is_flagged,
        "technical_risk" => {
            matches!(
                group.technical_gate_status.as_deref(),
                Some("warn" | "reject" | "inconclusive" | "unsupported")
            ) || matches!(group.model_tier.as_deref(), Some("weak" | "reject"))
        }
        "pending_analysis" => {
            group.model_status.is_none()
                || matches!(group.model_status.as_deref(), Some("pending" | "running"))
                || matches!(
                    group.technical_status.as_deref(),
                    Some("pending" | "analyzing")
                )
        }
        _ => true,
    }
}

fn normalized_asset_collection_key(collection: &str) -> String {
    match collection.trim().to_ascii_lowercase().as_str() {
        "" | "all" => "all".to_string(),
        "model_select" | "model_selects" | "algorithm_select" | "algorithm_selects" => {
            "model_selects".to_string()
        }
        "favorite" | "favorites" => "favorites".to_string(),
        "mark" | "marked" | "flag" | "flagged" => "marked".to_string(),
        "technical_risk" | "risk" => "technical_risk".to_string(),
        "pending_analysis" | "analysis_pending" => "pending_analysis".to_string(),
        value => value.to_string(),
    }
}

fn group_best_score(group: &ReceivedAssetGroup) -> Option<f64> {
    group
        .burst
        .as_ref()
        .and_then(|burst| burst.best_score)
        .or_else(|| group.model_score.map(|score| score as f64))
}

fn score_sort_key(score: Option<f64>) -> i64 {
    score
        .filter(|value| value.is_finite())
        .map(|value| (value * 1_000_000.0).round() as i64)
        .unwrap_or(i64::MIN)
}

fn score_for_threshold(score: f64) -> i64 {
    if score > 1.0 {
        score.round() as i64
    } else {
        (score * 100.0).round() as i64
    }
}

fn group_received_sort_time(group: &ReceivedAssetGroup) -> Option<i64> {
    group_assets(group)
        .into_iter()
        .filter_map(|asset| asset.capture_time_ms.or(asset.received_time_ms))
        .max()
}

fn group_assets(group: &ReceivedAssetGroup) -> Vec<&ReceivedAsset> {
    let mut assets = Vec::new();
    push_unique_asset(&mut assets, &group.primary);
    if let Some(asset) = group.jpeg.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    if let Some(asset) = group.raw.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    if let Some(asset) = group.video.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    assets
}

fn push_unique_asset<'a>(assets: &mut Vec<&'a ReceivedAsset>, asset: &'a ReceivedAsset) {
    if !assets.iter().any(|existing| existing.id == asset.id) {
        assets.push(asset);
    }
}

fn facet_counts(counts: BTreeMap<String, usize>) -> Vec<AssetFacetCount> {
    counts
        .into_iter()
        .map(|(value, group_count)| AssetFacetCount { value, group_count })
        .collect()
}

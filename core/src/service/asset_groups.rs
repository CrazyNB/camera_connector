use std::collections::BTreeMap;

use crate::{
    ConnectedDevice, ImportSource, PublishQueueItem, ReceivedAsset, ReceivedAssetGroup,
    ReceiverAccountConfig, TransferRecord,
};

use super::{AssetFacetCount, AssetGroupQuery, AssetGroupSummary, DuplicateInfo, TransferQuery};

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

fn facet_counts(counts: BTreeMap<String, usize>) -> Vec<AssetFacetCount> {
    counts
        .into_iter()
        .map(|(value, group_count)| AssetFacetCount { value, group_count })
        .collect()
}

pub(super) fn asset_group_matches(group: &ReceivedAssetGroup, query: &AssetGroupQuery) -> bool {
    let asset_matches = group_assets(group)
        .into_iter()
        .any(|asset| asset_matches(asset, query));
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
    let score_matches = query
        .min_model_score
        .map(|minimum| {
            group_best_score(group)
                .map(|score| score >= minimum)
                .unwrap_or(false)
        })
        .unwrap_or(true);
    asset_matches
        && favorite_matches
        && marked_matches
        && user_mark_any_matches
        && guest_mark_matches
        && score_matches
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

fn group_best_score(group: &ReceivedAssetGroup) -> Option<i64> {
    group
        .burst
        .as_ref()
        .and_then(|burst| burst.best_score)
        .map(score_for_threshold)
        .or(group.model_score)
}

fn score_for_threshold(score: f64) -> i64 {
    if score > 1.0 {
        score.round() as i64
    } else {
        (score * 100.0).round() as i64
    }
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

fn asset_matches(asset: &ReceivedAsset, query: &AssetGroupQuery) -> bool {
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
}

pub(super) fn asset_from_transfer_record(
    record: TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
    duplicates: &BTreeMap<String, DuplicateInfo>,
) -> ReceivedAsset {
    let storage_location = record.resolved_final_location();
    let display_source = record_display_source(&record, accounts);
    let virtual_display_path = record.virtual_display_path(display_source.as_deref());
    let mut asset = ReceivedAsset::new(
        record.transfer_id.clone(),
        record.final_filename.clone(),
        record.size_bytes,
        import_source_from_protocol(&record.protocol),
    );
    asset.received_time_ms = record.completed_at_ms.or(Some(record.started_at_ms));
    asset.storage_location = storage_location;
    asset.original_path = Some(record.original_path);
    asset.username = record.username;
    asset.display_source = display_source;
    asset.remote_addr = record.remote_addr;
    asset.virtual_display_path = Some(virtual_display_path);
    if let Some(duplicate) = duplicates.get(&record.transfer_id) {
        asset.duplicate_index = Some(duplicate.index);
        asset.duplicate_count = Some(duplicate.count);
    }
    asset
}

pub(super) fn duplicate_info_by_transfer_id(
    records: &[TransferRecord],
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> BTreeMap<String, DuplicateInfo> {
    let mut duplicate_keys = BTreeMap::<String, Vec<&TransferRecord>>::new();
    for record in records {
        if let Some(key) = duplicate_key(record, accounts) {
            duplicate_keys.entry(key).or_default().push(record);
        }
    }

    let mut duplicates = BTreeMap::new();
    for duplicate_records in duplicate_keys.values_mut() {
        if duplicate_records.len() < 2 {
            continue;
        }
        duplicate_records.sort_by_key(|record| {
            (
                record.completed_at_ms.unwrap_or(record.started_at_ms),
                record.started_at_ms,
                record.transfer_id.clone(),
            )
        });
        let count = duplicate_records.len();
        for (index, record) in duplicate_records.iter().enumerate() {
            duplicates.insert(
                record.transfer_id.clone(),
                DuplicateInfo {
                    index: index + 1,
                    count,
                },
            );
        }
    }
    duplicates
}

fn duplicate_key(
    record: &TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    let original_path = normalized_duplicate_segment(&record.original_path)?;
    let identity = record
        .username
        .as_deref()
        .and_then(normalized_duplicate_segment)
        .or_else(|| {
            record_display_source(record, accounts)
                .and_then(|value| normalized_duplicate_segment(&value))
        })
        .or_else(|| {
            record
                .remote_addr
                .as_deref()
                .and_then(normalized_duplicate_segment)
        })
        .unwrap_or_else(|| "-".to_string());
    Some(format!("{identity}\t{original_path}"))
}

fn normalized_duplicate_segment(value: &str) -> Option<String> {
    let normalized = value.trim().replace('\\', "/").to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn import_source_from_protocol(protocol: &str) -> ImportSource {
    match protocol.to_ascii_lowercase().as_str() {
        "ftp" => ImportSource::FtpPush,
        "sftp" => ImportSource::SftpPush,
        "desktop_scan" => ImportSource::DesktopScan,
        _ => ImportSource::ManualDrop,
    }
}

pub(super) fn transfer_matches(
    record: &TransferRecord,
    query: &TransferQuery,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> bool {
    query
        .status
        .map(|expected| record.status == expected)
        .unwrap_or(true)
        && query
            .username
            .as_ref()
            .map(|expected| record.username.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .source_name
            .as_ref()
            .map(|expected| record_display_source(record, accounts).as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .remote_addr
            .as_ref()
            .map(|expected| record.remote_addr.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .transfer_id
            .as_ref()
            .map(|expected| record.transfer_id.contains(expected))
            .unwrap_or(true)
        && query
            .original_path
            .as_ref()
            .map(|expected| {
                record
                    .original_path
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
            })
            .unwrap_or(true)
        && query
            .final_filename
            .as_ref()
            .map(|expected| {
                record
                    .final_filename
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
            })
            .unwrap_or(true)
}

pub(super) fn record_display_source(
    record: &TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    record
        .username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| record.source_name.clone())
}

pub(super) fn publish_item_display_source(
    item: &PublishQueueItem,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    item.username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| item.source_name.clone())
}

pub(super) fn device_matches(
    device: &ConnectedDevice,
    username: Option<&str>,
    online: bool,
) -> bool {
    (!online || device.online)
        && username
            .map(|expected| device.username.as_deref() == Some(expected))
            .unwrap_or(true)
}

pub(super) fn device_display_source(
    device: &ConnectedDevice,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    device
        .username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| device.source_name.clone())
}

pub(super) fn remote_addr_display_label(remote_addr: &str) -> String {
    if let Some(last_octet) = remote_addr
        .rsplit('.')
        .next()
        .filter(|value| !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit()))
        .and_then(|value| value.parse::<u8>().ok())
    {
        return format!("IP-{last_octet:03}");
    }

    let digits = remote_addr
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>();
    if digits.is_empty() {
        "IP".to_string()
    } else {
        let start = digits.len().saturating_sub(3);
        format!("IP-{:0>3}", &digits[start..])
    }
}

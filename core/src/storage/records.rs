use rusqlite::Row;

use crate::{
    ImportSource, ImporterError, ObjectFormat, ReceivedAsset, Result, StoredObjectLocation,
    TransferRecord, TransferStatus,
};

use super::{
    Project, ProjectStatus, PublishQueueItem, PublishState, StoredAsset, StoredAssetGroup,
};

pub(super) fn project_from_row(row: &Row<'_>) -> std::result::Result<Project, rusqlite::Error> {
    let status: String = row.get(3)?;
    Ok(Project {
        project_id: row.get(0)?,
        name: row.get(1)?,
        slug: row.get(2)?,
        status: ProjectStatus::from_str(&status),
        created_at_ms: row.get(4)?,
        updated_at_ms: row.get(5)?,
        archived_at_ms: row.get(6)?,
        default_output_target_id: row.get(7)?,
    })
}

pub(super) fn received_asset_from_row(
    row: &Row<'_>,
) -> std::result::Result<ReceivedAsset, rusqlite::Error> {
    let stored = stored_asset_from_row(row)?;
    let mut asset = ReceivedAsset::new(
        stored.asset_id,
        stored.final_filename,
        stored.size_bytes,
        import_source_from_transfer_id(&stored.transfer_id),
    );
    asset.format = stored.format;
    asset.group_key = Some(stored.normalized_stem);
    asset.storage_location = stored.final_location;
    asset.original_path = Some(stored.original_path);
    asset.username = stored.username;
    asset.display_source = stored.source_identity;
    asset.remote_addr = stored.remote_addr;
    asset.received_time_ms = stored.received_at_ms;
    asset.capture_time_ms = stored.capture_at_ms;
    asset.source_status = Some(stored.source_status);
    asset.source_modified_at_ms = stored.source_modified_at_ms;
    asset.last_seen_scan_id = stored.last_seen_scan_id;
    asset.duplicate_index = stored.duplicate_index;
    asset.duplicate_count = stored.duplicate_count;
    asset.virtual_display_path = asset.display_source.as_deref().map(|source| {
        format!(
            "{source}/{}",
            asset.original_path.as_deref().unwrap_or(&asset.filename)
        )
    });
    Ok(asset)
}

pub(super) fn stored_asset_from_row(
    row: &Row<'_>,
) -> std::result::Result<StoredAsset, rusqlite::Error> {
    let media_kind: String = row.get(5)?;
    let format: String = row.get(6)?;
    let final_location_payload: Option<String> = row.get(12)?;
    Ok(StoredAsset {
        asset_id: row.get(0)?,
        project_id: row.get(1)?,
        group_id: row.get(2)?,
        transfer_id: row.get(3)?,
        group_role: row.get(4)?,
        media_kind,
        format: parse_format(&format),
        original_filename: row.get(7)?,
        final_filename: row.get(8)?,
        normalized_stem: row.get(9)?,
        original_path: row.get(10)?,
        original_parent_path: row.get(11)?,
        final_location: parse_location(final_location_payload)?,
        size_bytes: row.get::<_, i64>(13)? as u64,
        capture_at_ms: row.get(14)?,
        received_at_ms: row.get(15)?,
        published_at_ms: row.get(16)?,
        source_identity: row.get(17)?,
        username: row.get(18)?,
        remote_addr: row.get(19)?,
        source_status: row.get(20)?,
        source_modified_at_ms: row.get(21)?,
        last_seen_scan_id: row.get(22)?,
        duplicate_index: row.get::<_, Option<i64>>(23)?.map(|value| value as usize),
        duplicate_count: row.get::<_, Option<i64>>(24)?.map(|value| value as usize),
    })
}

pub(super) fn stored_asset_group_from_row(
    row: &Row<'_>,
) -> std::result::Result<StoredAssetGroup, rusqlite::Error> {
    Ok(StoredAssetGroup {
        group_id: row.get(0)?,
        project_id: row.get(1)?,
        group_identity: row.get(2)?,
        display_key: row.get(3)?,
        source_identity: row.get(4)?,
        original_parent_path: row.get(5)?,
        primary_asset_id: row.get(6)?,
        preview_asset_id: row.get(7)?,
        member_count: row.get::<_, i64>(8)? as usize,
        has_raw: row.get::<_, i64>(9)? != 0,
        has_jpeg: row.get::<_, i64>(10)? != 0,
        has_video: row.get::<_, i64>(11)? != 0,
        first_capture_at_ms: row.get(12)?,
        last_capture_at_ms: row.get(13)?,
        first_received_at_ms: row.get(14)?,
        last_received_at_ms: row.get(15)?,
        created_at_ms: row.get(16)?,
        updated_at_ms: row.get(17)?,
    })
}

pub(super) fn transfer_record_from_row(
    row: &Row<'_>,
) -> std::result::Result<TransferRecord, rusqlite::Error> {
    let status: String = row.get(2)?;
    let final_location_payload: Option<String> = row.get(5)?;
    Ok(TransferRecord {
        transfer_id: row.get(0)?,
        protocol: row.get(1)?,
        status: parse_transfer_status(&status),
        original_path: row.get(3)?,
        final_filename: row.get(4)?,
        final_location: parse_location(final_location_payload)?,
        size_bytes: row.get::<_, i64>(6)? as u64,
        username: row.get(7)?,
        remote_addr: row.get(8)?,
        source_name: row.get(9)?,
        started_at_ms: row.get(10)?,
        completed_at_ms: row.get(11)?,
        error: row.get(12)?,
    })
}

pub(super) fn publish_item_from_row(
    row: &Row<'_>,
) -> std::result::Result<PublishQueueItem, rusqlite::Error> {
    let state: String = row.get(12)?;
    Ok(PublishQueueItem {
        queue_id: row.get(0)?,
        project_id: row.get(1)?,
        transfer_id: row.get(2)?,
        staged_path: row.get(3)?,
        final_filename: row.get(4)?,
        size_bytes: row.get::<_, i64>(5)? as u64,
        protocol: row.get(6)?,
        original_path: row.get(7)?,
        username: row.get(8)?,
        remote_addr: row.get(9)?,
        source_name: row.get(10)?,
        started_at_ms: row.get(11)?,
        state: PublishState::from_str(&state),
        attempt_count: row.get::<_, i64>(13)? as u32,
        last_error: row.get(14)?,
        next_attempt_at_ms: row.get(15)?,
        created_at_ms: row.get(16)?,
        updated_at_ms: row.get(17)?,
    })
}

pub(super) fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&Row<'_>) -> std::result::Result<T, rusqlite::Error>>,
) -> std::result::Result<Vec<T>, rusqlite::Error> {
    rows.collect()
}

pub(super) fn final_location_json(
    location: Option<&StoredObjectLocation>,
) -> std::result::Result<Option<String>, rusqlite::Error> {
    location
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
}

fn parse_location(
    value: Option<String>,
) -> std::result::Result<Option<StoredObjectLocation>, rusqlite::Error> {
    value
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
}

pub(super) fn normalized_required(field: &str, value: &str) -> Result<String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(ImporterError::internal(format!("{field} cannot be empty")));
    }
    Ok(normalized)
}

pub(super) fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

pub(super) fn slugify(value: &str) -> String {
    let slug = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "project".to_string()
    } else {
        slug
    }
}

pub(super) fn stable_key(value: &str) -> String {
    let mut hash = 1469598103934665603_u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}

pub(super) fn asset_group_identity(
    project_id: &str,
    source_identity: Option<&str>,
    original_parent_path: Option<&str>,
    normalized_stem: &str,
) -> String {
    format!(
        "{}\t{}\t{}\t{}",
        project_id,
        source_identity.unwrap_or_default(),
        original_parent_path.unwrap_or_default(),
        normalized_stem
    )
}

pub(super) fn normalized_stem(filename: &str) -> Option<String> {
    let name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    let (stem, _) = name.rsplit_once('.')?;
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_ascii_uppercase())
    }
}

pub(super) fn original_parent_path(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let parent = normalized
        .rsplit_once('/')
        .map(|(parent, _)| parent.trim_matches('/'))?;
    if parent.is_empty() {
        None
    } else {
        Some(parent.to_string())
    }
}

pub(super) fn original_filename(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(path)
        .to_string()
}

pub(super) fn source_identity(record: &TransferRecord) -> Option<String> {
    record
        .source_name
        .clone()
        .or_else(|| record.username.clone())
        .or_else(|| record.remote_addr.clone())
}

pub(super) fn duplicate_key(record: &TransferRecord) -> Option<String> {
    let original = normalized_duplicate_segment(&record.original_path)?;
    let identity = source_identity(record)
        .and_then(|value| normalized_duplicate_segment(&value))
        .unwrap_or_else(|| "-".to_string());
    Some(format!("{identity}\t{original}"))
}

fn normalized_duplicate_segment(value: &str) -> Option<String> {
    let normalized = value.trim().replace('\\', "/").to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub(super) fn transfer_status_name(status: TransferStatus) -> &'static str {
    match status {
        TransferStatus::Completed => "completed",
        TransferStatus::Failed => "failed",
    }
}

fn parse_transfer_status(value: &str) -> TransferStatus {
    match value {
        "failed" => TransferStatus::Failed,
        _ => TransferStatus::Completed,
    }
}

pub(super) fn format_name(format: ObjectFormat) -> &'static str {
    match format {
        ObjectFormat::Jpeg => "jpeg",
        ObjectFormat::Nef => "nef",
        ObjectFormat::Nrw => "nrw",
        ObjectFormat::Cr2 => "cr2",
        ObjectFormat::Cr3 => "cr3",
        ObjectFormat::Arw => "arw",
        ObjectFormat::Raf => "raf",
        ObjectFormat::Rw2 => "rw2",
        ObjectFormat::Orf => "orf",
        ObjectFormat::Pef => "pef",
        ObjectFormat::Dng => "dng",
        ObjectFormat::Mov => "mov",
        ObjectFormat::Mp4 => "mp4",
        ObjectFormat::Tiff => "tiff",
        ObjectFormat::Unknown => "unknown",
    }
}

fn parse_format(value: &str) -> ObjectFormat {
    match value {
        "jpeg" => ObjectFormat::Jpeg,
        "nef" => ObjectFormat::Nef,
        "nrw" => ObjectFormat::Nrw,
        "cr2" => ObjectFormat::Cr2,
        "cr3" => ObjectFormat::Cr3,
        "arw" => ObjectFormat::Arw,
        "raf" => ObjectFormat::Raf,
        "rw2" => ObjectFormat::Rw2,
        "orf" => ObjectFormat::Orf,
        "pef" => ObjectFormat::Pef,
        "dng" => ObjectFormat::Dng,
        "mov" => ObjectFormat::Mov,
        "mp4" => ObjectFormat::Mp4,
        "tiff" => ObjectFormat::Tiff,
        _ => ObjectFormat::Unknown,
    }
}

pub(super) fn group_role(format: ObjectFormat) -> &'static str {
    if format == ObjectFormat::Jpeg {
        "jpeg"
    } else if format.is_raw() {
        "raw"
    } else if format.is_video() {
        "video"
    } else {
        "other"
    }
}

pub(super) fn media_kind(format: ObjectFormat) -> &'static str {
    if format.is_video() {
        "video"
    } else if format.is_photo() {
        "photo"
    } else {
        "unknown"
    }
}

fn import_source_from_transfer_id(transfer_id: &str) -> ImportSource {
    if transfer_id.starts_with("desktop-scan-") {
        ImportSource::DesktopScan
    } else if transfer_id.starts_with("sftp:") {
        ImportSource::SftpPush
    } else if transfer_id.starts_with("ftp:") {
        ImportSource::FtpPush
    } else {
        ImportSource::ManualDrop
    }
}

use rusqlite::{params, Connection, OptionalExtension};

use crate::media_metadata::extract_capture_time_ms;
use crate::{
    AssetUserMarks, DesktopSourceStatus, GuestMark, ObjectFormat, ReceivedAsset,
    StoredObjectLocation, TransferRecord, DESKTOP_SCAN_PROTOCOL,
};

use super::projects::ensure_project_exists;
use super::records::{
    asset_group_identity, collect_rows, current_time_ms, duplicate_key, final_location_json,
    format_name, group_role, media_kind, normalized_stem, original_filename, original_parent_path,
    received_asset_from_row, source_identity, stable_key, stored_asset_group_from_row,
    transfer_status_name,
};
use super::{sqlite_data_error, StoredAssetGroup};

pub(super) fn stored_asset_groups_for_project(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<Vec<StoredAssetGroup>, rusqlite::Error> {
    ensure_project_exists(connection, project_id)?;
    let mut statement = connection.prepare(
        "SELECT group_id, project_id, group_identity, display_key, source_identity,
                original_parent_path, primary_asset_id, preview_asset_id, member_count,
                has_raw, has_jpeg, has_video, first_capture_at_ms, last_capture_at_ms,
                first_received_at_ms, last_received_at_ms, created_at_ms, updated_at_ms
         FROM asset_groups
         WHERE project_id = ?1
         ORDER BY COALESCE(last_received_at_ms, updated_at_ms) DESC,
                  display_key ASC,
                  group_id ASC",
    )?;
    let rows = statement.query_map(params![project_id], stored_asset_group_from_row)?;
    collect_rows(rows)
}

pub(super) fn stored_asset_group_by_id(
    connection: &Connection,
    project_id: &str,
    group_id: &str,
) -> std::result::Result<Option<StoredAssetGroup>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT group_id, project_id, group_identity, display_key, source_identity,
                    original_parent_path, primary_asset_id, preview_asset_id, member_count,
                    has_raw, has_jpeg, has_video, first_capture_at_ms, last_capture_at_ms,
                    first_received_at_ms, last_received_at_ms, created_at_ms, updated_at_ms
             FROM asset_groups
             WHERE project_id = ?1 AND group_id = ?2",
            params![project_id, group_id],
            stored_asset_group_from_row,
        )
        .optional()
}

pub(super) fn trailing_sequence_number(value: &str) -> Option<i64> {
    let reversed_digits = value
        .chars()
        .rev()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if reversed_digits.is_empty() {
        return None;
    }
    reversed_digits
        .chars()
        .rev()
        .collect::<String>()
        .parse()
        .ok()
}

pub(super) fn received_assets_for_group(
    connection: &Connection,
    project_id: &str,
    group_id: &str,
) -> std::result::Result<Vec<ReceivedAsset>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT asset_id, project_id, group_id, transfer_id, group_role,
                media_kind, format, original_filename, final_filename, normalized_stem, original_path,
                original_parent_path, final_location_payload, size_bytes, capture_at_ms,
                received_at_ms, published_at_ms, source_identity, username, remote_addr,
                source_status, source_modified_at_ms, last_seen_scan_id, duplicate_index,
                duplicate_count
         FROM assets
         WHERE project_id = ?1 AND group_id = ?2
         ORDER BY CASE group_role
                    WHEN 'jpeg' THEN 0
                    WHEN 'raw' THEN 1
                    WHEN 'video' THEN 2
                    ELSE 3
                  END ASC,
                  published_at_ms ASC,
                  asset_id ASC",
    )?;
    let rows = statement.query_map(params![project_id, group_id], received_asset_from_row)?;
    collect_rows(rows)
}

pub(super) fn user_marks_for_asset_group(
    connection: &Connection,
    project_id: &str,
    group_id: &str,
) -> std::result::Result<AssetUserMarks, rusqlite::Error> {
    connection
        .query_row(
            "SELECT favorite, marked
             FROM asset_group_user_marks
             WHERE project_id = ?1 AND group_id = ?2",
            params![project_id, group_id],
            |row| {
                Ok(AssetUserMarks {
                    favorite: row.get::<_, bool>(0)?,
                    marked: row.get::<_, bool>(1)?,
                })
            },
        )
        .optional()
        .map(|marks| marks.unwrap_or_default())
}

pub(super) fn guest_mark_for_asset_group(
    connection: &Connection,
    project_id: &str,
    group_id: &str,
) -> std::result::Result<Option<GuestMark>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT marks.guest_mark
             FROM lan_share_guest_marks marks
             JOIN lan_share_sessions sessions ON sessions.share_id = marks.share_id
             WHERE marks.project_id = ?1
               AND marks.asset_group_id = ?2
               AND sessions.active = 1
             ORDER BY marks.updated_at_ms DESC, sessions.created_at_ms DESC
             LIMIT 1",
            params![project_id, group_id],
            |row| {
                let raw: String = row.get(0)?;
                GuestMark::from_wire(&raw)
                    .ok_or_else(|| sqlite_data_error(format!("invalid guest mark: {raw}")))
            },
        )
        .optional()
}

pub(super) fn insert_transfer(
    connection: &Connection,
    project_id: &str,
    record: &TransferRecord,
) -> std::result::Result<(), rusqlite::Error> {
    let final_location = record.resolved_final_location();
    let final_location_payload = final_location_json(final_location.as_ref())?;
    connection.execute(
        "INSERT OR REPLACE INTO transfers (
            transfer_id, project_id, protocol, status, original_path, final_filename,
            final_location_kind, final_location_payload, size_bytes, username, remote_addr,
            source_name, started_at_ms, completed_at_ms, error
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        params![
            record.transfer_id,
            project_id,
            record.protocol,
            transfer_status_name(record.status),
            record.original_path,
            record.final_filename,
            final_location.as_ref().map(StoredObjectLocation::kind),
            final_location_payload,
            record.size_bytes as i64,
            record.username,
            record.remote_addr,
            record.source_name,
            record.started_at_ms,
            record.completed_at_ms,
            record.error,
        ],
    )?;
    Ok(())
}

pub(super) fn insert_asset_for_transfer(
    connection: &Connection,
    project_id: &str,
    record: &TransferRecord,
) -> std::result::Result<Option<String>, rusqlite::Error> {
    let format = ObjectFormat::from_filename(&record.final_filename);
    if !format.is_supported_media() {
        return Ok(None);
    }

    let now = current_time_ms();
    let normalized_stem =
        normalized_stem(&record.final_filename).unwrap_or_else(|| record.final_filename.clone());
    let asset_original_parent_path = original_parent_path(&record.original_path);
    let group_original_parent_path = if record.protocol == DESKTOP_SCAN_PROTOCOL {
        None
    } else {
        asset_original_parent_path.clone()
    };
    let source_identity = source_identity(record);
    let group_identity = asset_group_identity(
        project_id,
        source_identity.as_deref(),
        group_original_parent_path.as_deref(),
        &normalized_stem,
    );
    let group_id = format!("group-{}", stable_key(&group_identity));
    let final_location = record.resolved_final_location();
    let final_location_payload = final_location_json(final_location.as_ref())?;
    let group_role = group_role(format).to_string();
    let published_at_ms = record.completed_at_ms.or(Some(record.started_at_ms));
    let capture_at_ms = final_location
        .as_ref()
        .and_then(|location| location.as_local_path())
        .and_then(extract_capture_time_ms);
    let duplicate_key = duplicate_key(record);
    let previous_duplicate_key = connection
        .query_row(
            "SELECT duplicate_key FROM assets WHERE transfer_id = ?1",
            params![record.transfer_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten();

    connection.execute(
        "INSERT INTO asset_groups (
            group_id, project_id, group_identity, display_key, source_identity, original_parent_path,
            created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(group_identity) DO UPDATE SET updated_at_ms = excluded.updated_at_ms",
        params![
            group_id,
            project_id,
            group_identity,
            normalized_stem,
            source_identity,
            group_original_parent_path,
            now,
            now,
        ],
    )?;

    connection.execute(
        "INSERT OR REPLACE INTO assets (
            asset_id, project_id, group_id, transfer_id, group_role, media_kind, format,
            original_filename, final_filename, normalized_stem, original_path, original_parent_path,
            final_location_kind, final_location_payload, size_bytes, capture_at_ms, received_at_ms,
            published_at_ms, source_identity, username, remote_addr, source_status,
            source_modified_at_ms, last_seen_scan_id, duplicate_key
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)",
        params![
            record.transfer_id,
            project_id,
            group_id,
            record.transfer_id,
            group_role,
            media_kind(format),
            format_name(format),
            original_filename(&record.original_path),
            record.final_filename,
            normalized_stem,
            record.original_path,
            asset_original_parent_path,
            final_location.as_ref().map(StoredObjectLocation::kind),
            final_location_payload,
            record.size_bytes as i64,
            capture_at_ms,
            published_at_ms,
            published_at_ms,
            source_identity.clone(),
            record.username,
            record.remote_addr,
            DesktopSourceStatus::Available.as_str(),
            None::<i64>,
            None::<String>,
            duplicate_key,
        ],
    )?;
    refresh_group_rollup(connection, &group_id)?;
    refresh_duplicate_info_for_key(connection, project_id, previous_duplicate_key.as_deref())?;
    if previous_duplicate_key.as_deref() != duplicate_key.as_deref() {
        refresh_duplicate_info_for_key(connection, project_id, duplicate_key.as_deref())?;
    }
    Ok(Some(group_id))
}

pub(super) fn refresh_group_rollup(
    connection: &Connection,
    group_id: &str,
) -> std::result::Result<(), rusqlite::Error> {
    let primary_asset_id: Option<String> = connection
        .query_row(
            "SELECT asset_id
             FROM assets
             WHERE group_id = ?1
             ORDER BY CASE group_role
                        WHEN 'jpeg' THEN 0
                        WHEN 'raw' THEN 1
                        WHEN 'video' THEN 2
                        ELSE 3
                      END ASC,
                      published_at_ms ASC,
                      asset_id ASC
             LIMIT 1",
            params![group_id],
            |row| row.get(0),
        )
        .optional()?;
    let preview_asset_id: Option<String> = connection
        .query_row(
            "SELECT asset_id FROM assets WHERE group_id = ?1 AND group_role = 'jpeg'
             ORDER BY published_at_ms ASC, asset_id ASC LIMIT 1",
            params![group_id],
            |row| row.get(0),
        )
        .optional()?
        .or_else(|| primary_asset_id.clone());

    connection.execute(
        "UPDATE asset_groups
         SET primary_asset_id = ?2,
             preview_asset_id = ?3,
             member_count = (SELECT COUNT(*) FROM assets WHERE group_id = ?1),
             has_raw = EXISTS(SELECT 1 FROM assets WHERE group_id = ?1 AND group_role = 'raw'),
             has_jpeg = EXISTS(SELECT 1 FROM assets WHERE group_id = ?1 AND group_role = 'jpeg'),
             has_video = EXISTS(SELECT 1 FROM assets WHERE group_id = ?1 AND group_role = 'video'),
             first_capture_at_ms = (SELECT MIN(capture_at_ms) FROM assets WHERE group_id = ?1),
             last_capture_at_ms = (SELECT MAX(capture_at_ms) FROM assets WHERE group_id = ?1),
             first_received_at_ms = (SELECT MIN(received_at_ms) FROM assets WHERE group_id = ?1),
             last_received_at_ms = (SELECT MAX(received_at_ms) FROM assets WHERE group_id = ?1),
             updated_at_ms = ?4
         WHERE group_id = ?1",
        params![
            group_id,
            primary_asset_id,
            preview_asset_id,
            current_time_ms()
        ],
    )?;
    Ok(())
}

pub(super) fn refresh_duplicate_info(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<(), rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT duplicate_key FROM assets
         WHERE project_id = ?1 AND duplicate_key IS NOT NULL
         GROUP BY duplicate_key
         HAVING COUNT(*) > 1",
    )?;
    let duplicate_keys = statement
        .query_map(params![project_id], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    drop(statement);

    connection.execute(
        "UPDATE assets SET duplicate_index = NULL, duplicate_count = NULL WHERE project_id = ?1",
        params![project_id],
    )?;

    for duplicate_key in duplicate_keys {
        let mut assets = connection
            .prepare(
                "SELECT asset_id FROM assets
                 WHERE project_id = ?1 AND duplicate_key = ?2
                 ORDER BY published_at_ms ASC, asset_id ASC",
            )?
            .query_map(params![project_id, duplicate_key], |row| {
                row.get::<_, String>(0)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let count = assets.len() as i64;
        for (index, asset_id) in assets.drain(..).enumerate() {
            connection.execute(
                "UPDATE assets SET duplicate_index = ?1, duplicate_count = ?2 WHERE asset_id = ?3",
                params![index as i64 + 1, count, asset_id],
            )?;
        }
    }
    Ok(())
}

fn refresh_duplicate_info_for_key(
    connection: &Connection,
    project_id: &str,
    duplicate_key: Option<&str>,
) -> std::result::Result<(), rusqlite::Error> {
    let Some(duplicate_key) = duplicate_key else {
        return Ok(());
    };
    let mut assets = connection
        .prepare(
            "SELECT asset_id FROM assets
             WHERE project_id = ?1 AND duplicate_key = ?2
             ORDER BY published_at_ms ASC, asset_id ASC",
        )?
        .query_map(params![project_id, duplicate_key], |row| {
            row.get::<_, String>(0)
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let count = assets.len() as i64;
    if count <= 1 {
        connection.execute(
            "UPDATE assets
             SET duplicate_index = NULL, duplicate_count = NULL
             WHERE project_id = ?1 AND duplicate_key = ?2",
            params![project_id, duplicate_key],
        )?;
        return Ok(());
    }

    for (index, asset_id) in assets.drain(..).enumerate() {
        connection.execute(
            "UPDATE assets SET duplicate_index = ?1, duplicate_count = ?2 WHERE asset_id = ?3",
            params![index as i64 + 1, count, asset_id],
        )?;
    }
    Ok(())
}

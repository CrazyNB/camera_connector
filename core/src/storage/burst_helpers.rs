use std::collections::{BTreeMap, BTreeSet};

use rusqlite::{params, Connection, OptionalExtension};

use crate::{BurstGroup, BurstGroupingProfile, StoredAssetGroup};

use super::collect_rows;

pub(super) fn burst_source_identity_for_group(
    connection: &Connection,
    project_id: &str,
    group_id: &str,
) -> std::result::Result<Option<String>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT COALESCE(NULLIF(username, ''), NULLIF(source_identity, ''), NULLIF(remote_addr, ''))
             FROM assets
             WHERE project_id = ?1 AND group_id = ?2
             ORDER BY CASE group_role
                        WHEN 'jpeg' THEN 0
                        WHEN 'raw' THEN 1
                        WHEN 'video' THEN 2
                        ELSE 3
                      END ASC,
                      published_at_ms ASC,
                      asset_id ASC
             LIMIT 1",
            params![project_id, group_id],
            |row| row.get(0),
        )
        .optional()
        .map(|value| value.flatten())
}

pub(super) fn common_burst_source_identity(
    connection: &Connection,
    project_id: &str,
    groups: &[StoredAssetGroup],
) -> std::result::Result<Option<String>, rusqlite::Error> {
    let mut common: Option<String> = None;
    for group in groups {
        let source_identity =
            burst_source_identity_for_group(connection, project_id, &group.group_id)?
                .or_else(|| group.source_identity.clone());
        match (&common, source_identity) {
            (None, Some(value)) => common = Some(value),
            (Some(left), Some(right)) if left == &right => {}
            (Some(_), Some(_)) => return Ok(None),
            (_, None) => {}
        }
    }
    Ok(common)
}

pub(super) fn burst_group_by_id(
    connection: &Connection,
    burst_group_id: &str,
) -> std::result::Result<Option<BurstGroup>, rusqlite::Error> {
    let Some(mut burst) = connection
        .query_row(
            "SELECT burst_group_id, project_id, source_identity, started_at_ms, ended_at_ms,
                    member_count, grouping_version, recommendation_status, manual_grouping_state,
                    created_at_ms, updated_at_ms
             FROM burst_groups
             WHERE burst_group_id = ?1",
            params![burst_group_id],
            |row| {
                Ok(BurstGroup {
                    burst_group_id: row.get(0)?,
                    project_id: row.get(1)?,
                    source_identity: row.get(2)?,
                    started_at_ms: row.get(3)?,
                    ended_at_ms: row.get(4)?,
                    member_count: row.get::<_, i64>(5)? as usize,
                    member_group_ids: Vec::new(),
                    grouping_version: row.get(6)?,
                    recommendation_status: row.get(7)?,
                    manual_grouping_state: row.get(8)?,
                    created_at_ms: row.get(9)?,
                    updated_at_ms: row.get(10)?,
                })
            },
        )
        .optional()?
    else {
        return Ok(None);
    };

    let mut statement = connection.prepare(
        "SELECT bgm.member_group_id
         FROM burst_group_members bgm
         LEFT JOIN asset_groups ag ON ag.group_id = bgm.member_group_id
         WHERE bgm.burst_group_id = ?1
         ORDER BY COALESCE(ag.first_capture_at_ms, ag.first_received_at_ms, ag.created_at_ms) ASC,
                  ag.display_key ASC,
                  bgm.member_group_id ASC",
    )?;
    let rows = statement.query_map(params![burst_group_id], |row| row.get(0))?;
    burst.member_group_ids = collect_rows(rows)?;
    burst.member_count = burst.member_group_ids.len();
    Ok(Some(burst))
}

pub(super) fn burst_group_for_member_group(
    connection: &Connection,
    project_id: &str,
    member_group_id: &str,
) -> std::result::Result<Option<BurstGroup>, rusqlite::Error> {
    let burst_group_id = connection
        .query_row(
            "SELECT bg.burst_group_id
             FROM burst_group_members bgm
             JOIN burst_groups bg ON bg.burst_group_id = bgm.burst_group_id
             WHERE bg.project_id = ?1 AND bgm.member_group_id = ?2
             ORDER BY bg.updated_at_ms DESC, bg.burst_group_id DESC
             LIMIT 1",
            params![project_id, member_group_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    burst_group_id
        .map(|burst_group_id| burst_group_by_id(connection, &burst_group_id))
        .transpose()
        .map(|value| value.flatten())
}

pub(super) fn manual_split_excluded_member_group_ids(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<BTreeSet<String>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT member_group_id
         FROM burst_member_manual_edits
         WHERE project_id = ?1 AND action = 'split_exclude'",
    )?;
    let rows = statement.query_map(params![project_id], |row| row.get::<_, String>(0))?;
    collect_rows(rows).map(|values| values.into_iter().collect())
}

pub(super) fn manual_merge_member_groups(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<BTreeMap<String, Vec<String>>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT manual_group_id, member_group_id
         FROM burst_member_manual_edits
         WHERE project_id = ?1
           AND action = 'merge_include'
           AND manual_group_id IS NOT NULL
         ORDER BY manual_group_id ASC, member_group_id ASC",
    )?;
    let rows = statement.query_map(params![project_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut groups = BTreeMap::new();
    for row in rows {
        let (manual_group_id, member_group_id) = row?;
        groups
            .entry(manual_group_id)
            .or_insert_with(Vec::new)
            .push(member_group_id);
    }
    Ok(groups)
}

pub(super) fn visual_hash_from_signature(value: Option<&str>) -> Option<u64> {
    let value = value?;
    let hex = value.strip_prefix("ahash-v1:")?;
    u64::from_str_radix(hex, 16).ok()
}

pub(super) fn visual_hash_similarity(left: u64, right: u64) -> f64 {
    1.0 - ((left ^ right).count_ones() as f64 / 64.0)
}

pub(super) fn visual_burst_continuity_threshold(profile: &BurstGroupingProfile) -> f64 {
    profile.visual_continuity_threshold.clamp(0.70, 0.90)
}

pub(super) fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

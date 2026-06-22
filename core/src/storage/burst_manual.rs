use rusqlite::{params, Connection};

use crate::{BurstGroup, SelectionRecommendationScope, SelectionRecommendationStatus};

use super::burst_helpers::{
    burst_group_by_id, burst_group_for_member_group, common_burst_source_identity,
    push_unique_string,
};
use super::bursts::insert_burst_group;
use super::{
    current_time_ms, ensure_project_exists, sqlite_data_error, stable_key, stored_asset_group_by_id,
};

pub(super) fn split_burst_member_for_connection(
    connection: &Connection,
    burst_group_id: &str,
    member_group_id: &str,
) -> std::result::Result<Option<BurstGroup>, rusqlite::Error> {
    let Some(burst) = burst_group_by_id(connection, burst_group_id)? else {
        return Ok(None);
    };
    let member_group_id = member_group_id.trim();
    if member_group_id.is_empty() {
        return Err(sqlite_data_error("member group id cannot be empty"));
    }
    if !burst
        .member_group_ids
        .iter()
        .any(|group_id| group_id == member_group_id)
    {
        return Err(sqlite_data_error("member group is not in burst group"));
    }

    let remaining_member_ids = burst
        .member_group_ids
        .iter()
        .filter(|group_id| group_id.as_str() != member_group_id)
        .cloned()
        .collect::<Vec<_>>();
    let now = current_time_ms();

    connection.execute(
        "DELETE FROM selection_recommendations
         WHERE project_id = ?1 AND scope = ?2 AND subject_id = ?3",
        params![
            burst.project_id,
            SelectionRecommendationScope::BurstGroup.as_str(),
            burst_group_id
        ],
    )?;
    connection.execute(
        "INSERT OR REPLACE INTO burst_member_manual_edits (
            project_id, member_group_id, action, manual_group_id, created_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            burst.project_id,
            member_group_id,
            "split_exclude",
            None::<String>,
            now
        ],
    )?;

    if remaining_member_ids.len() < 2 {
        connection.execute(
            "DELETE FROM burst_group_members WHERE burst_group_id = ?1",
            params![burst_group_id],
        )?;
        connection.execute(
            "DELETE FROM burst_groups WHERE burst_group_id = ?1",
            params![burst_group_id],
        )?;
        return Ok(None);
    }

    connection.execute(
        "DELETE FROM burst_group_members WHERE burst_group_id = ?1",
        params![burst_group_id],
    )?;
    for member_group_id in remaining_member_ids.iter() {
        connection.execute(
            "INSERT INTO burst_group_members (burst_group_id, member_group_id)
             VALUES (?1, ?2)",
            params![burst_group_id, member_group_id],
        )?;
    }
    connection.execute(
        "UPDATE burst_groups
         SET member_count = ?1,
             grouping_version = grouping_version + 1,
             recommendation_status = ?2,
             manual_grouping_state = ?3,
             updated_at_ms = ?4
         WHERE burst_group_id = ?5",
        params![
            remaining_member_ids.len() as i64,
            SelectionRecommendationStatus::Pending.as_str(),
            "split",
            now,
            burst_group_id,
        ],
    )?;

    burst_group_by_id(connection, burst_group_id)
}

pub(super) fn create_manual_burst_group_for_connection(
    connection: &Connection,
    project_id: &str,
    member_group_ids: &[String],
) -> std::result::Result<Option<BurstGroup>, rusqlite::Error> {
    ensure_project_exists(connection, project_id)?;

    let mut expanded_member_ids = Vec::new();
    let mut affected_burst_ids = Vec::new();
    let mut requested_container_ids = Vec::new();
    for raw_member_group_id in member_group_ids {
        let member_group_id = raw_member_group_id.trim();
        if member_group_id.is_empty() {
            continue;
        }
        if stored_asset_group_by_id(connection, project_id, member_group_id)?.is_none() {
            return Err(sqlite_data_error(
                "member group not found in target project",
            ));
        }
        if let Some(source_burst) =
            burst_group_for_member_group(connection, project_id, member_group_id)?
        {
            push_unique_string(
                &mut requested_container_ids,
                source_burst.burst_group_id.clone(),
            );
            push_unique_string(&mut affected_burst_ids, source_burst.burst_group_id.clone());
            for source_member_group_id in source_burst.member_group_ids {
                push_unique_string(&mut expanded_member_ids, source_member_group_id);
            }
        } else {
            push_unique_string(&mut requested_container_ids, member_group_id.to_string());
            push_unique_string(&mut expanded_member_ids, member_group_id.to_string());
        }
    }

    if requested_container_ids.len() < 2 || expanded_member_ids.len() < 2 {
        return Ok(None);
    }

    let mut stable_member_ids = expanded_member_ids.clone();
    stable_member_ids.sort();
    let manual_burst_group_id = format!(
        "manual-burst-{}",
        stable_key(&format!("{project_id}\t{}", stable_member_ids.join(",")))
    );
    let now = current_time_ms();

    let mut cleanup_burst_ids = affected_burst_ids.clone();
    push_unique_string(&mut cleanup_burst_ids, manual_burst_group_id.clone());
    for burst_group_id in cleanup_burst_ids.iter() {
        connection.execute(
            "DELETE FROM selection_recommendations
             WHERE project_id = ?1 AND scope = ?2 AND subject_id = ?3",
            params![
                project_id,
                SelectionRecommendationScope::BurstGroup.as_str(),
                burst_group_id,
            ],
        )?;
        connection.execute(
            "DELETE FROM burst_group_members WHERE burst_group_id = ?1",
            params![burst_group_id],
        )?;
        connection.execute(
            "DELETE FROM burst_groups WHERE burst_group_id = ?1",
            params![burst_group_id],
        )?;
    }

    let mut member_groups = Vec::new();
    for member_group_id in expanded_member_ids.iter() {
        if let Some(group) = stored_asset_group_by_id(connection, project_id, member_group_id)? {
            member_groups.push(group);
        }
        connection.execute(
            "DELETE FROM burst_member_manual_edits
             WHERE project_id = ?1 AND member_group_id = ?2
               AND action IN ('split_exclude', 'merge_include')",
            params![project_id, member_group_id],
        )?;
        connection.execute(
            "INSERT INTO burst_member_manual_edits (
                project_id, member_group_id, action, manual_group_id, created_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                project_id,
                member_group_id,
                "merge_include",
                manual_burst_group_id,
                now,
            ],
        )?;
    }

    member_groups.sort_by(|left, right| {
        (
            left.first_capture_at_ms
                .or(left.first_received_at_ms)
                .or(Some(left.created_at_ms)),
            left.display_key.as_str(),
            left.group_id.as_str(),
        )
            .cmp(&(
                right
                    .first_capture_at_ms
                    .or(right.first_received_at_ms)
                    .or(Some(right.created_at_ms)),
                right.display_key.as_str(),
                right.group_id.as_str(),
            ))
    });
    let sorted_member_group_ids = member_groups
        .iter()
        .map(|group| group.group_id.clone())
        .collect::<Vec<_>>();
    let started_at_ms = member_groups
        .iter()
        .filter_map(|group| group.first_capture_at_ms.or(group.first_received_at_ms))
        .min();
    let ended_at_ms = member_groups
        .iter()
        .filter_map(|group| group.first_capture_at_ms.or(group.first_received_at_ms))
        .max();
    let source_identity = common_burst_source_identity(connection, project_id, &member_groups)?;
    let burst = BurstGroup {
        burst_group_id: manual_burst_group_id,
        project_id: project_id.to_string(),
        source_identity,
        started_at_ms,
        ended_at_ms,
        member_count: sorted_member_group_ids.len(),
        member_group_ids: sorted_member_group_ids,
        grouping_version: 1,
        recommendation_status: SelectionRecommendationStatus::Pending.as_str().to_string(),
        manual_grouping_state: Some("merge".to_string()),
        created_at_ms: now,
        updated_at_ms: now,
    };
    insert_burst_group(connection, &burst)?;
    Ok(Some(burst))
}

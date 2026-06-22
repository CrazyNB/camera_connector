use std::collections::{BTreeMap, BTreeSet};

use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    AnalysisEntityType, BurstGroup, BurstGroupingProfile, ReceivedAssetBurstSummary, Result,
    SelectionRecommendationScope, SelectionRecommendationStatus, TechnicalAssessmentStatus,
};

use super::analysis::{
    latest_selection_recommendation_for_connection, technical_assessments_for_asset_group_ids,
};
use super::burst_helpers::{
    burst_group_by_id, burst_group_for_member_group, burst_source_identity_for_group,
    common_burst_source_identity, manual_merge_member_groups,
    manual_split_excluded_member_group_ids, visual_burst_continuity_threshold,
    visual_hash_from_signature, visual_hash_similarity,
};
use super::burst_manual::{
    create_manual_burst_group_for_connection, split_burst_member_for_connection,
};
use super::{
    current_time_ms, ensure_project_exists, stable_key, stored_asset_group_by_id,
    stored_asset_groups_for_project, trailing_sequence_number, SqliteStore, StoredAssetGroup,
};

impl SqliteStore {
    pub fn burst_group(&self, burst_group_id: &str) -> Result<Option<BurstGroup>> {
        self.with_read_connection(|connection| burst_group_by_id(connection, burst_group_id))
    }

    pub fn burst_group_for_asset_group(&self, asset_group_id: &str) -> Result<Option<BurstGroup>> {
        self.with_read_connection(|connection| {
            let project_id = connection
                .query_row(
                    "SELECT project_id FROM asset_groups WHERE group_id = ?1",
                    params![asset_group_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            project_id
                .map(|project_id| {
                    burst_group_for_member_group(connection, &project_id, asset_group_id)
                })
                .transpose()
                .map(|value| value.flatten())
        })
    }

    pub fn split_burst_member(
        &self,
        burst_group_id: &str,
        member_group_id: &str,
    ) -> Result<Option<BurstGroup>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let result =
                split_burst_member_for_connection(&transaction, burst_group_id, member_group_id)?;
            transaction.commit()?;
            Ok(result)
        })
    }

    pub fn create_manual_burst_group(
        &self,
        project_id: &str,
        member_group_ids: &[String],
    ) -> Result<Option<BurstGroup>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let result = create_manual_burst_group_for_connection(
                &transaction,
                project_id,
                member_group_ids,
            )?;
            transaction.commit()?;
            Ok(result)
        })
    }

    pub fn detect_bursts_for_asset_group(
        &self,
        project_id: &str,
        asset_group_id: &str,
        profile: &BurstGroupingProfile,
    ) -> Result<Vec<BurstGroup>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            ensure_project_exists(&transaction, project_id)?;
            let groups = stored_asset_groups_for_project(&transaction, project_id)?;
            if !groups.iter().any(|group| group.group_id == asset_group_id) {
                transaction.commit()?;
                return Ok(Vec::new());
            }

            let bursts =
                rebuild_burst_groups_for_project(&transaction, project_id, groups, profile)?;
            transaction.commit()?;
            Ok(bursts
                .into_iter()
                .filter(|burst| {
                    burst
                        .member_group_ids
                        .iter()
                        .any(|member_group_id| member_group_id == asset_group_id)
                })
                .collect())
        })
    }

    pub fn refine_burst_group_by_visual_similarity(
        &self,
        burst_group_id: &str,
        profile: &BurstGroupingProfile,
        assessor_version: &str,
    ) -> Result<Vec<BurstGroup>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let Some(burst) = burst_group_by_id(&transaction, burst_group_id)? else {
                transaction.commit()?;
                return Ok(Vec::new());
            };
            let assessments = technical_assessments_for_asset_group_ids(
                &transaction,
                &burst.member_group_ids,
                assessor_version,
            )?;
            let assessment_by_group_id = assessments
                .iter()
                .map(|assessment| (assessment.asset_group_id.as_str(), assessment))
                .collect::<BTreeMap<_, _>>();
            let mut visual_hashes = Vec::new();
            for member_group_id in &burst.member_group_ids {
                let Some(assessment) = assessment_by_group_id.get(member_group_id.as_str()) else {
                    transaction.commit()?;
                    return Ok(vec![burst]);
                };
                if assessment.status != TechnicalAssessmentStatus::Ready {
                    transaction.commit()?;
                    return Ok(vec![burst]);
                }
                let Some(hash) = visual_hash_from_signature(assessment.visual_signature.as_deref())
                else {
                    transaction.commit()?;
                    return Ok(vec![burst]);
                };
                visual_hashes.push((member_group_id.clone(), hash));
            }

            let threshold = visual_burst_continuity_threshold(profile);
            let mut runs: Vec<Vec<String>> = Vec::new();
            let mut current_run: Vec<String> = Vec::new();
            let mut previous_hash: Option<u64> = None;
            for (member_group_id, hash) in visual_hashes {
                let continues = previous_hash
                    .map(|previous| visual_hash_similarity(previous, hash) >= threshold)
                    .unwrap_or(true);
                if !continues && !current_run.is_empty() {
                    runs.push(std::mem::take(&mut current_run));
                }
                current_run.push(member_group_id);
                previous_hash = Some(hash);
            }
            if !current_run.is_empty() {
                runs.push(current_run);
            }
            if runs.len() <= 1 {
                transaction.commit()?;
                return Ok(vec![burst]);
            }

            transaction.execute(
                "DELETE FROM selection_recommendations
                 WHERE project_id = ?1 AND scope = ?2 AND subject_id = ?3",
                params![
                    burst.project_id,
                    SelectionRecommendationScope::BurstGroup.as_str(),
                    burst_group_id
                ],
            )?;
            transaction.execute(
                "DELETE FROM background_jobs
                 WHERE entity_type = ?1 AND entity_id = ?2 AND status IN ('pending', 'failed')",
                params![AnalysisEntityType::BurstGroup.as_str(), burst_group_id],
            )?;
            transaction.execute(
                "DELETE FROM burst_group_members WHERE burst_group_id = ?1",
                params![burst_group_id],
            )?;
            transaction.execute(
                "DELETE FROM burst_groups WHERE burst_group_id = ?1",
                params![burst_group_id],
            )?;

            let now = current_time_ms();
            let mut refined_bursts = Vec::new();
            for run in runs {
                if run.len() < profile.min_group_size {
                    continue;
                }
                let mut member_groups = Vec::new();
                for member_group_id in &run {
                    if let Some(group) =
                        stored_asset_group_by_id(&transaction, &burst.project_id, member_group_id)?
                    {
                        member_groups.push(group);
                    }
                }
                if member_groups.len() < profile.min_group_size {
                    continue;
                }
                let member_group_ids = member_groups
                    .iter()
                    .map(|group| group.group_id.clone())
                    .collect::<Vec<_>>();
                let stable_members = member_group_ids.join(",");
                let refined_burst_group_id = format!(
                    "burst-{}",
                    stable_key(&format!(
                        "{}\t{}\t{}",
                        burst.project_id, profile.grouping_version, stable_members
                    ))
                );
                let started_at_ms = member_groups
                    .iter()
                    .filter_map(|group| group.first_capture_at_ms.or(group.first_received_at_ms))
                    .min();
                let ended_at_ms = member_groups
                    .iter()
                    .filter_map(|group| group.first_capture_at_ms.or(group.first_received_at_ms))
                    .max();
                let refined = BurstGroup {
                    burst_group_id: refined_burst_group_id,
                    project_id: burst.project_id.clone(),
                    source_identity: common_burst_source_identity(
                        &transaction,
                        &burst.project_id,
                        &member_groups,
                    )?
                    .or_else(|| burst.source_identity.clone()),
                    started_at_ms,
                    ended_at_ms,
                    member_count: member_group_ids.len(),
                    member_group_ids,
                    grouping_version: burst.grouping_version + 1,
                    recommendation_status: SelectionRecommendationStatus::Pending
                        .as_str()
                        .to_string(),
                    manual_grouping_state: None,
                    created_at_ms: now,
                    updated_at_ms: now,
                };
                insert_burst_group(&transaction, &refined)?;
                refined_bursts.push(refined);
            }
            transaction.commit()?;
            Ok(refined_bursts)
        })
    }
}

#[derive(Debug, Clone)]
struct BurstCandidate {
    group: StoredAssetGroup,
    source_identity: Option<String>,
    sequence_number: Option<i64>,
    event_time_ms: Option<i64>,
    event_time_is_capture: bool,
}

fn rebuild_burst_groups_for_project(
    connection: &Connection,
    project_id: &str,
    groups: Vec<StoredAssetGroup>,
    profile: &BurstGroupingProfile,
) -> std::result::Result<Vec<BurstGroup>, rusqlite::Error> {
    connection.execute(
        "DELETE FROM burst_group_members
         WHERE burst_group_id IN (
            SELECT burst_group_id FROM burst_groups WHERE project_id = ?1
         )",
        params![project_id],
    )?;
    connection.execute(
        "DELETE FROM burst_groups WHERE project_id = ?1",
        params![project_id],
    )?;

    let manual_merge_groups = manual_merge_member_groups(connection, project_id)?;
    let manual_merge_member_group_ids = manual_merge_groups
        .values()
        .flat_map(|member_group_ids| member_group_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let split_excluded_member_group_ids =
        manual_split_excluded_member_group_ids(connection, project_id)?;
    let mut candidates = groups
        .into_iter()
        .filter(|group| group.has_jpeg || group.has_raw)
        .filter(|group| !split_excluded_member_group_ids.contains(&group.group_id))
        .filter(|group| !manual_merge_member_group_ids.contains(&group.group_id))
        .map(|group| {
            let source_identity =
                burst_source_identity_for_group(connection, project_id, &group.group_id)?
                    .or_else(|| group.source_identity.clone());
            let sequence_number = trailing_sequence_number(&group.display_key);
            let (event_time_ms, event_time_is_capture) = match group.first_capture_at_ms {
                Some(value) => (Some(value), true),
                None => (
                    group.first_received_at_ms.or(Some(group.created_at_ms)),
                    false,
                ),
            };
            Ok(BurstCandidate {
                group,
                source_identity,
                sequence_number,
                event_time_ms,
                event_time_is_capture,
            })
        })
        .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()?;

    candidates.sort_by(|left, right| {
        (
            left.source_identity.as_deref().unwrap_or_default(),
            left.group
                .original_parent_path
                .as_deref()
                .unwrap_or_default(),
            left.sequence_number.unwrap_or(i64::MAX),
            left.event_time_ms.unwrap_or(i64::MAX),
            left.group.display_key.as_str(),
            left.group.group_id.as_str(),
        )
            .cmp(&(
                right.source_identity.as_deref().unwrap_or_default(),
                right
                    .group
                    .original_parent_path
                    .as_deref()
                    .unwrap_or_default(),
                right.sequence_number.unwrap_or(i64::MAX),
                right.event_time_ms.unwrap_or(i64::MAX),
                right.group.display_key.as_str(),
                right.group.group_id.as_str(),
            ))
    });

    let mut runs: Vec<Vec<BurstCandidate>> = Vec::new();
    let mut current_run: Vec<BurstCandidate> = Vec::new();
    for candidate in candidates {
        let continues_current_run = current_run
            .last()
            .map(|previous| burst_candidates_are_adjacent(previous, &candidate, profile))
            .unwrap_or(false);
        if !continues_current_run && !current_run.is_empty() {
            runs.push(std::mem::take(&mut current_run));
        }
        current_run.push(candidate);
    }
    if !current_run.is_empty() {
        runs.push(current_run);
    }

    let now = current_time_ms();
    let mut bursts = Vec::new();
    for run in runs {
        if run.len() < profile.min_group_size {
            continue;
        }
        let member_group_ids = run
            .iter()
            .map(|candidate| candidate.group.group_id.clone())
            .collect::<Vec<_>>();
        let stable_members = member_group_ids.join(",");
        let burst_group_id = format!(
            "burst-{}",
            stable_key(&format!(
                "{project_id}\t{}\t{stable_members}",
                profile.grouping_version
            ))
        );
        let started_at_ms = run
            .iter()
            .filter_map(|candidate| candidate.event_time_ms)
            .min();
        let ended_at_ms = run
            .iter()
            .filter_map(|candidate| candidate.event_time_ms)
            .max();
        let source_identity = run
            .first()
            .and_then(|candidate| candidate.source_identity.clone());
        let burst = BurstGroup {
            burst_group_id: burst_group_id.clone(),
            project_id: project_id.to_string(),
            source_identity,
            started_at_ms,
            ended_at_ms,
            member_count: member_group_ids.len(),
            member_group_ids,
            grouping_version: 1,
            recommendation_status: SelectionRecommendationStatus::Pending.as_str().to_string(),
            manual_grouping_state: None,
            created_at_ms: now,
            updated_at_ms: now,
        };
        insert_burst_group(connection, &burst)?;
        bursts.push(burst);
    }

    for (burst_group_id, member_group_ids) in manual_merge_groups {
        let mut member_groups = Vec::new();
        for member_group_id in member_group_ids {
            if let Some(group) = stored_asset_group_by_id(connection, project_id, &member_group_id)?
            {
                member_groups.push(group);
            }
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
        if member_groups.len() < profile.min_group_size {
            continue;
        }

        let member_group_ids = member_groups
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
            burst_group_id,
            project_id: project_id.to_string(),
            source_identity,
            started_at_ms,
            ended_at_ms,
            member_count: member_group_ids.len(),
            member_group_ids,
            grouping_version: 1,
            recommendation_status: SelectionRecommendationStatus::Pending.as_str().to_string(),
            manual_grouping_state: Some("merge".to_string()),
            created_at_ms: now,
            updated_at_ms: now,
        };
        insert_burst_group(connection, &burst)?;
        bursts.push(burst);
    }

    Ok(bursts)
}

fn burst_candidates_are_adjacent(
    previous: &BurstCandidate,
    candidate: &BurstCandidate,
    profile: &BurstGroupingProfile,
) -> bool {
    if previous.source_identity != candidate.source_identity
        || previous.group.original_parent_path != candidate.group.original_parent_path
    {
        return false;
    }

    let time_is_adjacent = previous.event_time_is_capture
        && candidate.event_time_is_capture
        && previous
            .event_time_ms
            .zip(candidate.event_time_ms)
            .map(|(left, right)| right >= left && right - left <= profile.burst_window_ms)
            .unwrap_or(false);

    if previous.event_time_is_capture || candidate.event_time_is_capture {
        return time_is_adjacent;
    }

    previous
        .sequence_number
        .zip(candidate.sequence_number)
        .map(|(left, right)| right > left && right - left <= 1)
        .unwrap_or(false)
}

pub(super) fn insert_burst_group(
    connection: &Connection,
    burst: &BurstGroup,
) -> std::result::Result<(), rusqlite::Error> {
    connection.execute(
        "INSERT INTO burst_groups (
            burst_group_id, project_id, source_identity, started_at_ms, ended_at_ms,
            member_count, grouping_version, recommendation_status, manual_grouping_state,
            created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            burst.burst_group_id,
            burst.project_id,
            burst.source_identity,
            burst.started_at_ms,
            burst.ended_at_ms,
            burst.member_count as i64,
            burst.grouping_version,
            burst.recommendation_status,
            burst.manual_grouping_state,
            burst.created_at_ms,
            burst.updated_at_ms,
        ],
    )?;

    for member_group_id in burst.member_group_ids.iter() {
        connection.execute(
            "INSERT INTO burst_group_members (burst_group_id, member_group_id)
             VALUES (?1, ?2)",
            params![burst.burst_group_id, member_group_id],
        )?;
    }
    Ok(())
}

pub(super) fn burst_summary_for_asset_group(
    connection: &Connection,
    project_id: &str,
    asset_group_id: &str,
) -> std::result::Result<Option<ReceivedAssetBurstSummary>, rusqlite::Error> {
    let summary = connection
        .query_row(
            "SELECT bg.burst_group_id, bg.member_count, bg.recommendation_status
             FROM burst_group_members bgm
             JOIN burst_groups bg ON bg.burst_group_id = bgm.burst_group_id
             WHERE bg.project_id = ?1 AND bgm.member_group_id = ?2
             ORDER BY bg.updated_at_ms DESC, bg.burst_group_id DESC
             LIMIT 1",
            params![project_id, asset_group_id],
            |row| {
                Ok(ReceivedAssetBurstSummary {
                    burst_group_id: row.get(0)?,
                    member_count: row.get::<_, i64>(1)? as usize,
                    recommendation_status: row.get(2)?,
                    best_asset_group_id: None,
                    best_score: None,
                })
            },
        )
        .optional()?;
    summary
        .map(|mut summary| {
            let mut selected_asset_group_id = None;
            if let Some(recommendation) = latest_selection_recommendation_for_connection(
                connection,
                project_id,
                SelectionRecommendationScope::BurstGroup,
                &summary.burst_group_id,
            )? {
                summary.recommendation_status = recommendation.status.as_str().to_string();
                selected_asset_group_id = recommendation.selected_asset_group_ids.first().cloned();
                summary.best_asset_group_id = selected_asset_group_id.clone();
            }
            summary.best_score = selected_asset_group_id
                .as_deref()
                .map(|asset_group_id| burst_selected_model_score(connection, asset_group_id))
                .transpose()?
                .flatten();
            Ok(summary)
        })
        .transpose()
}

fn burst_selected_model_score(
    connection: &Connection,
    asset_group_id: &str,
) -> std::result::Result<Option<f64>, rusqlite::Error> {
    connection.query_row(
        "SELECT MAX(me.score)
         FROM model_evaluations me
         WHERE me.asset_group_id = ?1
           AND me.status = 'ready'
           AND me.updated_at_ms = (
               SELECT MAX(latest.updated_at_ms)
               FROM model_evaluations latest
               WHERE latest.asset_group_id = me.asset_group_id
           )",
        params![asset_group_id],
        |row| {
            row.get::<_, Option<i64>>(0)
                .map(|score| score.map(|value| value as f64))
        },
    )
}

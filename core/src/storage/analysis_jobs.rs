use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::{
    AnalysisEntityType, AnalysisJob, AnalysisJobStatus, AnalysisJobType, NewAnalysisJob, Result,
    SceneProfile,
};

use super::{collect_rows, current_time_ms, sqlite_data_error, stable_key, SqliteStore};

impl SqliteStore {
    pub fn enqueue_analysis_job(&self, job: NewAnalysisJob) -> Result<AnalysisJob> {
        self.with_connection(|connection| enqueue_analysis_job_for_connection(connection, job))
    }

    pub fn claim_analysis_jobs(&self, now_ms: i64, limit: usize) -> Result<Vec<AnalysisJob>> {
        self.with_connection(|connection| {
            claim_analysis_jobs_for_connection(connection, now_ms, limit)
        })
    }

    pub fn complete_analysis_job(&self, job_id: &str) -> Result<()> {
        self.with_connection(|connection| {
            let changed = connection.execute(
                "UPDATE background_jobs
                 SET status = ?1, last_error = NULL, next_attempt_at_ms = NULL, updated_at_ms = ?2
                 WHERE job_id = ?3",
                params![
                    AnalysisJobStatus::Completed.as_str(),
                    current_time_ms(),
                    job_id
                ],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "analysis job not found".to_string(),
                ));
            }
            Ok(())
        })
    }

    pub fn fail_analysis_job(
        &self,
        job_id: &str,
        error: &str,
        next_attempt_at_ms: i64,
    ) -> Result<()> {
        self.with_connection(|connection| {
            let changed = connection.execute(
                "UPDATE background_jobs
                 SET status = ?1, attempts = attempts + 1, last_error = ?2,
                     next_attempt_at_ms = ?3, updated_at_ms = ?4
                 WHERE job_id = ?5",
                params![
                    AnalysisJobStatus::Failed.as_str(),
                    error,
                    next_attempt_at_ms,
                    current_time_ms(),
                    job_id,
                ],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "analysis job not found".to_string(),
                ));
            }
            Ok(())
        })
    }
}

pub(super) fn enqueue_detect_burst_job_for_connection(
    connection: &Connection,
    project_id: &str,
    asset_group_id: &str,
) -> std::result::Result<AnalysisJob, rusqlite::Error> {
    enqueue_analysis_job_for_connection(
        connection,
        NewAnalysisJob::new(
            project_id,
            AnalysisJobType::DetectBurstForAssetGroup,
            AnalysisEntityType::AssetGroup,
            asset_group_id,
            &format!("burst:{project_id}:{asset_group_id}"),
        ),
    )
}

pub(super) fn enqueue_portrait_subject_assessment_job_for_connection(
    connection: &Connection,
    project_id: &str,
    asset_group_id: &str,
) -> std::result::Result<AnalysisJob, rusqlite::Error> {
    let mut job = NewAnalysisJob::new(
        project_id,
        AnalysisJobType::AssessPortraitSubject,
        AnalysisEntityType::AssetGroup,
        asset_group_id,
        &format!("subject:portrait:{project_id}:{asset_group_id}"),
    );
    job.priority = 15;
    enqueue_analysis_job_for_connection(connection, job)
}

pub(super) fn should_schedule_subject_assessment_for_project(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<bool, rusqlite::Error> {
    let scene_profile = connection
        .query_row(
            "SELECT scene_profile
             FROM project_evaluation_settings
             WHERE project_id = ?1",
            params![project_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(scene_profile
        .as_deref()
        .map(SceneProfile::from_str)
        .unwrap_or(SceneProfile::General)
        == SceneProfile::Portrait)
}

fn enqueue_analysis_job_for_connection(
    connection: &Connection,
    job: NewAnalysisJob,
) -> std::result::Result<AnalysisJob, rusqlite::Error> {
    if let Some(existing) = connection
        .query_row(
            "SELECT job_id, project_id, job_type, entity_type, entity_id, dedupe_key, status,
                    priority, attempts, next_attempt_at_ms, last_error, created_at_ms, updated_at_ms
             FROM background_jobs
             WHERE dedupe_key = ?1 AND status != 'completed'
             ORDER BY created_at_ms ASC
             LIMIT 1",
            params![job.dedupe_key],
            analysis_job_from_row,
        )
        .optional()?
    {
        return Ok(existing);
    }
    let now = current_time_ms();
    let job_id = format!("analysis-job-{}-{}", now, stable_key(&job.dedupe_key));
    connection.execute(
        "INSERT INTO background_jobs (
            job_id, project_id, job_type, entity_type, entity_id, dedupe_key, status, priority,
            attempts, next_attempt_at_ms, last_error, created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, NULL, ?10, ?10)",
        params![
            job_id,
            job.project_id,
            job.job_type.as_str(),
            job.entity_type.as_str(),
            job.entity_id,
            job.dedupe_key,
            AnalysisJobStatus::Pending.as_str(),
            job.priority,
            job.next_attempt_at_ms,
            now,
        ],
    )?;
    analysis_job_by_id(connection, &job_id)?
        .ok_or_else(|| sqlite_data_error("analysis job not found"))
}

fn claim_analysis_jobs_for_connection(
    connection: &mut Connection,
    now_ms: i64,
    limit: usize,
) -> std::result::Result<Vec<AnalysisJob>, rusqlite::Error> {
    let transaction = connection.unchecked_transaction()?;
    let jobs = {
        let mut statement = transaction.prepare(
            "SELECT job_id, project_id, job_type, entity_type, entity_id, dedupe_key, status,
                    priority, attempts, next_attempt_at_ms, last_error, created_at_ms, updated_at_ms
             FROM background_jobs
             WHERE status IN ('pending', 'failed')
               AND (next_attempt_at_ms IS NULL OR next_attempt_at_ms <= ?1)
             ORDER BY priority DESC, created_at_ms ASC, job_id ASC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![now_ms, limit as i64], analysis_job_from_row)?;
        collect_rows(rows)?
    };
    for job in &jobs {
        transaction.execute(
            "UPDATE background_jobs
             SET status = ?1, updated_at_ms = ?2
             WHERE job_id = ?3",
            params![AnalysisJobStatus::Running.as_str(), now_ms, job.job_id],
        )?;
    }
    transaction.commit()?;
    Ok(jobs
        .into_iter()
        .map(|job| AnalysisJob {
            status: AnalysisJobStatus::Running,
            updated_at_ms: now_ms,
            ..job
        })
        .collect())
}

fn analysis_job_by_id(
    connection: &Connection,
    job_id: &str,
) -> std::result::Result<Option<AnalysisJob>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT job_id, project_id, job_type, entity_type, entity_id, dedupe_key, status,
                    priority, attempts, next_attempt_at_ms, last_error, created_at_ms, updated_at_ms
             FROM background_jobs
             WHERE job_id = ?1",
            params![job_id],
            analysis_job_from_row,
        )
        .optional()
}

fn analysis_job_from_row(row: &Row<'_>) -> std::result::Result<AnalysisJob, rusqlite::Error> {
    let job_type: String = row.get(2)?;
    let entity_type: String = row.get(3)?;
    let status: String = row.get(6)?;
    Ok(AnalysisJob {
        job_id: row.get(0)?,
        project_id: row.get(1)?,
        job_type: AnalysisJobType::from_str(&job_type),
        entity_type: AnalysisEntityType::from_str(&entity_type),
        entity_id: row.get(4)?,
        dedupe_key: row.get(5)?,
        status: AnalysisJobStatus::from_str(&status),
        priority: row.get(7)?,
        attempts: row.get(8)?,
        next_attempt_at_ms: row.get(9)?,
        last_error: row.get(10)?,
        created_at_ms: row.get(11)?,
        updated_at_ms: row.get(12)?,
    })
}

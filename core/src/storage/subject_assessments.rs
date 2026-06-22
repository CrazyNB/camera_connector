use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::{EvaluationRunStatus, Result, SubjectAssessment};

use super::projects::ensure_project_exists;
use super::records::collect_rows;
use super::{sqlite_data_error, SqliteStore};

impl SqliteStore {
    pub fn save_subject_assessment(
        &self,
        assessment: SubjectAssessment,
    ) -> Result<SubjectAssessment> {
        self.with_connection(|connection| {
            save_subject_assessment_for_connection(connection, assessment)
        })
    }

    pub fn subject_assessments_for_asset_groups(
        &self,
        project_id: &str,
        group_ids: &[String],
    ) -> Result<Vec<SubjectAssessment>> {
        self.with_read_connection(|connection| {
            subject_assessments_for_asset_groups(connection, project_id, group_ids)
        })
    }
}

fn save_subject_assessment_for_connection(
    connection: &Connection,
    assessment: SubjectAssessment,
) -> std::result::Result<SubjectAssessment, rusqlite::Error> {
    ensure_project_exists(connection, &assessment.project_id)?;
    ensure_asset_group_exists_in_project(
        connection,
        &assessment.project_id,
        &assessment.asset_group_id,
    )?;
    ensure_subject_assessment_identity_is_stable(connection, &assessment)?;
    validate_subject_assessment_json(&assessment)?;
    connection.execute(
        "INSERT INTO subject_assessments (
            assessment_id, project_id, asset_group_id, subject_type, detector_kind,
            detector_version, status, gate_status, regions_json, signals_json, summary,
            created_at_ms, updated_at_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(assessment_id) DO UPDATE SET
            project_id = excluded.project_id,
            asset_group_id = excluded.asset_group_id,
            subject_type = excluded.subject_type,
            detector_kind = excluded.detector_kind,
            detector_version = excluded.detector_version,
            status = excluded.status,
            gate_status = excluded.gate_status,
            regions_json = excluded.regions_json,
            signals_json = excluded.signals_json,
            summary = excluded.summary,
            updated_at_ms = excluded.updated_at_ms",
        params![
            assessment.assessment_id,
            assessment.project_id,
            assessment.asset_group_id,
            assessment.subject_type,
            assessment.detector_kind,
            assessment.detector_version,
            assessment.status.as_str(),
            assessment.gate_status,
            assessment.regions_json,
            assessment.signals_json,
            assessment.summary,
            assessment.created_at_ms,
            assessment.updated_at_ms,
        ],
    )?;
    subject_assessment_by_id(connection, &assessment.assessment_id)?.ok_or_else(|| {
        rusqlite::Error::InvalidParameterName("subject assessment not found".to_string())
    })
}

fn ensure_asset_group_exists_in_project(
    connection: &Connection,
    project_id: &str,
    asset_group_id: &str,
) -> std::result::Result<(), rusqlite::Error> {
    let exists = connection.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM asset_groups WHERE project_id = ?1 AND group_id = ?2
         )",
        params![project_id, asset_group_id],
        |row| row.get::<_, bool>(0),
    )?;
    if exists {
        Ok(())
    } else {
        Err(sqlite_data_error(
            "subject assessment asset group not found in project",
        ))
    }
}

fn ensure_subject_assessment_identity_is_stable(
    connection: &Connection,
    assessment: &SubjectAssessment,
) -> std::result::Result<(), rusqlite::Error> {
    let existing = connection
        .query_row(
            "SELECT project_id, asset_group_id
             FROM subject_assessments
             WHERE assessment_id = ?1",
            params![assessment.assessment_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    if let Some((project_id, asset_group_id)) = existing {
        if project_id != assessment.project_id || asset_group_id != assessment.asset_group_id {
            return Err(sqlite_data_error(
                "subject assessment id cannot move between asset groups",
            ));
        }
    }
    Ok(())
}

fn validate_subject_assessment_json(
    assessment: &SubjectAssessment,
) -> std::result::Result<(), rusqlite::Error> {
    let regions =
        serde_json::from_str::<serde_json::Value>(&assessment.regions_json).map_err(|error| {
            sqlite_data_error(format!("invalid subject assessment regions_json: {error}"))
        })?;
    if !regions.is_array() {
        return Err(sqlite_data_error(
            "subject assessment regions_json must be a JSON array",
        ));
    }
    let signals =
        serde_json::from_str::<serde_json::Value>(&assessment.signals_json).map_err(|error| {
            sqlite_data_error(format!("invalid subject assessment signals_json: {error}"))
        })?;
    if !signals.is_object() {
        return Err(sqlite_data_error(
            "subject assessment signals_json must be a JSON object",
        ));
    }
    Ok(())
}

fn subject_assessment_by_id(
    connection: &Connection,
    assessment_id: &str,
) -> std::result::Result<Option<SubjectAssessment>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT assessment_id, project_id, asset_group_id, subject_type, detector_kind,
                    detector_version, status, gate_status, regions_json, signals_json, summary,
                    created_at_ms, updated_at_ms
             FROM subject_assessments
             WHERE assessment_id = ?1",
            params![assessment_id],
            subject_assessment_from_row,
        )
        .optional()
}

fn subject_assessments_for_asset_groups(
    connection: &Connection,
    project_id: &str,
    group_ids: &[String],
) -> std::result::Result<Vec<SubjectAssessment>, rusqlite::Error> {
    let mut assessments = Vec::new();
    for group_id in group_ids {
        let mut statement = connection.prepare(
            "SELECT assessment_id, project_id, asset_group_id, subject_type, detector_kind,
                    detector_version, status, gate_status, regions_json, signals_json, summary,
                    created_at_ms, updated_at_ms
             FROM subject_assessments
             WHERE project_id = ?1 AND asset_group_id = ?2
             ORDER BY created_at_ms DESC, assessment_id DESC",
        )?;
        let rows =
            statement.query_map(params![project_id, group_id], subject_assessment_from_row)?;
        assessments.extend(collect_rows(rows)?);
    }
    Ok(assessments)
}

fn subject_assessment_from_row(
    row: &Row<'_>,
) -> std::result::Result<SubjectAssessment, rusqlite::Error> {
    let status: String = row.get(6)?;
    Ok(SubjectAssessment {
        assessment_id: row.get(0)?,
        project_id: row.get(1)?,
        asset_group_id: row.get(2)?,
        subject_type: row.get(3)?,
        detector_kind: row.get(4)?,
        detector_version: row.get(5)?,
        status: EvaluationRunStatus::from_str(&status),
        gate_status: row.get(7)?,
        regions_json: row.get(8)?,
        signals_json: row.get(9)?,
        summary: row.get(10)?,
        created_at_ms: row.get(11)?,
        updated_at_ms: row.get(12)?,
    })
}

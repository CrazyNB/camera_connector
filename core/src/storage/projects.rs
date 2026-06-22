use rusqlite::{params, Connection, OptionalExtension};

use super::{
    collect_rows, current_time_ms, normalized_required, project_from_row,
    save_project_evaluation_settings_for_connection, slugify, stored_asset_from_row, Project,
    ProjectStatus, Result, SqliteStore, StoredAsset,
};
use crate::ProjectEvaluationSettings;

impl SqliteStore {
    pub fn create_project(&self, name: impl AsRef<str>) -> Result<Project> {
        let name = normalized_required("project name", name.as_ref())?;
        let now = current_time_ms();
        let slug = slugify(&name);
        let project = Project {
            project_id: format!("project-{now}-{slug}"),
            name,
            slug,
            status: ProjectStatus::Active,
            created_at_ms: now,
            updated_at_ms: now,
            archived_at_ms: None,
            default_output_target_id: None,
        };
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO projects (
                    project_id, name, slug, status, created_at_ms, updated_at_ms,
                    archived_at_ms, default_output_target_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    project.project_id,
                    project.name,
                    project.slug,
                    project.status.as_str(),
                    project.created_at_ms,
                    project.updated_at_ms,
                    project.archived_at_ms,
                    project.default_output_target_id,
                ],
            )?;
            save_project_evaluation_settings_for_connection(
                connection,
                ProjectEvaluationSettings::default_for_project(&project.project_id, now),
            )?;
            Ok(project)
        })
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        self.with_read_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT project_id, name, slug, status, created_at_ms, updated_at_ms,
                        archived_at_ms, default_output_target_id
                 FROM projects
                 ORDER BY updated_at_ms DESC, name ASC",
            )?;
            let rows = statement.query_map([], project_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn rename_project(&self, project_id: &str, name: impl AsRef<str>) -> Result<Project> {
        let name = normalized_required("project name", name.as_ref())?;
        let slug = slugify(&name);
        self.with_connection(|connection| {
            let now = current_time_ms();
            ensure_project_exists(connection, project_id)?;
            connection.execute(
                "UPDATE projects
                 SET name = ?1, slug = ?2, updated_at_ms = ?3
                 WHERE project_id = ?4",
                params![name, slug, now, project_id],
            )?;
            project_by_id(connection, project_id)?.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName("project not found".to_string())
            })
        })
    }

    pub fn archive_project(&self, project_id: &str) -> Result<Project> {
        self.with_connection(|connection| {
            let now = current_time_ms();
            ensure_project_exists(connection, project_id)?;
            connection.execute(
                "UPDATE projects
                 SET status = ?1, archived_at_ms = ?2, updated_at_ms = ?2
                 WHERE project_id = ?3",
                params![ProjectStatus::Archived.as_str(), now, project_id],
            )?;
            project_by_id(connection, project_id)?.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName("project not found".to_string())
            })
        })
    }

    pub fn restore_project(&self, project_id: &str) -> Result<Project> {
        self.with_connection(|connection| {
            let now = current_time_ms();
            ensure_project_exists(connection, project_id)?;
            connection.execute(
                "UPDATE projects
                 SET status = ?1, archived_at_ms = NULL, updated_at_ms = ?2
                 WHERE project_id = ?3",
                params![ProjectStatus::Active.as_str(), now, project_id],
            )?;
            project_by_id(connection, project_id)?.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName("project not found".to_string())
            })
        })
    }

    pub fn delete_project(&self, project_id: &str) -> Result<Option<Vec<StoredAsset>>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            if project_by_id(&transaction, project_id)?.is_none() {
                transaction.commit()?;
                return Ok(None);
            }

            let assets = {
                let mut statement = transaction.prepare(
                    "SELECT asset_id, project_id, group_id, transfer_id, group_role,
                            media_kind, format, original_filename, final_filename, normalized_stem, original_path,
                            original_parent_path, final_location_payload, size_bytes, capture_at_ms,
                            received_at_ms, published_at_ms, source_identity, username, remote_addr,
                            source_status, source_modified_at_ms, last_seen_scan_id, duplicate_index,
                            duplicate_count
                     FROM assets
                     WHERE project_id = ?1
                     ORDER BY published_at_ms ASC, asset_id ASC",
                )?;
                let rows = statement.query_map(params![project_id], stored_asset_from_row)?;
                collect_rows(rows)?
            };

            transaction.execute(
                "DELETE FROM technical_assessments
                 WHERE asset_group_id IN (
                    SELECT group_id FROM asset_groups WHERE project_id = ?1
                 )",
                params![project_id],
            )?;
            transaction.execute(
                "DELETE FROM burst_group_members
                 WHERE burst_group_id IN (
                    SELECT burst_group_id FROM burst_groups WHERE project_id = ?1
                 )
                    OR member_group_id IN (
                    SELECT group_id FROM asset_groups WHERE project_id = ?1
                 )",
                params![project_id],
            )?;
            for table in [
                "burst_member_manual_edits",
                "asset_group_user_marks",
                "lan_share_guest_marks",
                "lan_share_sessions",
                "subject_assessments",
                "model_evaluations",
                "selection_recommendations",
                "background_jobs",
                "publish_queue",
                "evaluation_runs",
                "assets",
                "transfers",
                "burst_groups",
                "asset_groups",
                "project_evaluation_settings",
                "projects",
            ] {
                transaction.execute(
                    &format!("DELETE FROM {table} WHERE project_id = ?1"),
                    params![project_id],
                )?;
            }
            transaction.commit()?;
            Ok(Some(assets))
        })
    }
}

pub(super) fn ensure_project_exists(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<(), rusqlite::Error> {
    if project_by_id(connection, project_id)?.is_none() {
        return Err(rusqlite::Error::InvalidParameterName(
            "project not found".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn ensure_project_is_active(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<Project, rusqlite::Error> {
    let project = project_by_id(connection, project_id)?
        .ok_or_else(|| rusqlite::Error::InvalidParameterName("project not found".to_string()))?;
    if project.status == ProjectStatus::Archived {
        return Err(rusqlite::Error::InvalidParameterName(
            "project archived".to_string(),
        ));
    }
    Ok(project)
}

pub(super) fn project_by_id(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<Option<Project>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT project_id, name, slug, status, created_at_ms, updated_at_ms,
                    archived_at_ms, default_output_target_id
             FROM projects
             WHERE project_id = ?1",
            params![project_id],
            project_from_row,
        )
        .optional()
}

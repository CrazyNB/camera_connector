use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension, Row};

use super::{
    collect_rows, ensure_project_exists, ensure_project_is_active, insert_asset_for_transfer,
    insert_transfer, refresh_group_rollup, sqlite_data_error, SqliteStore, StoredObjectLocation,
};
use crate::{
    desktop_scan_root_key, desktop_scan_root_label, desktop_scan_transfer_id,
    DesktopScanIndexResult, DesktopScanPhase, DesktopScanRun, DesktopScannedFile,
    DesktopSourceStatus, Result, TransferRecord, TransferStatus, DESKTOP_SCAN_PROTOCOL,
};

#[derive(Debug, Clone)]
pub struct DesktopScanRunUpdate<'a> {
    pub scan_id: &'a str,
    pub phase: DesktopScanPhase,
    pub files_seen: usize,
    pub assets_indexed: usize,
    pub groups_updated: usize,
    pub error: Option<&'a str>,
    pub now_ms: i64,
}

impl SqliteStore {
    pub fn create_desktop_scan_run(
        &self,
        project_id: &str,
        root_path: impl AsRef<Path>,
        now_ms: i64,
    ) -> Result<DesktopScanRun> {
        let root_path = root_path.as_ref().to_path_buf();
        let root_path_value = root_path.to_string_lossy().to_string();
        let root_key = desktop_scan_root_key(project_id, &root_path);
        let root_label = desktop_scan_root_label(&root_path);
        let scan_id = format!("desktop-scan-run-{root_key}-{now_ms}");
        self.with_connection(|connection| {
            ensure_project_is_active(connection, project_id)?;
            connection.execute(
                "INSERT INTO desktop_scan_runs (
                    scan_id, project_id, root_path, root_key, root_label, phase,
                    files_seen, assets_indexed, groups_updated, started_at_ms, updated_at_ms,
                    completed_at_ms, error
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, 0, ?7, ?7, NULL, NULL)",
                params![
                    scan_id,
                    project_id,
                    root_path_value,
                    root_key,
                    root_label,
                    DesktopScanPhase::Queued.as_str(),
                    now_ms,
                ],
            )?;
            desktop_scan_run_by_id(connection, &scan_id)?
                .ok_or_else(|| sqlite_data_error("desktop scan run not found"))
        })
    }

    pub fn desktop_scan_run(&self, scan_id: &str) -> Result<Option<DesktopScanRun>> {
        self.with_connection(|connection| desktop_scan_run_by_id(connection, scan_id))
    }

    pub fn latest_desktop_scan_run(&self, project_id: &str) -> Result<Option<DesktopScanRun>> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            connection
                .query_row(
                    "SELECT scan_id, project_id, root_path, root_key, root_label, phase,
                            files_seen, assets_indexed, groups_updated, started_at_ms, updated_at_ms,
                            completed_at_ms, error
                     FROM desktop_scan_runs
                     WHERE project_id = ?1
                     ORDER BY updated_at_ms DESC, started_at_ms DESC, scan_id DESC
                     LIMIT 1",
                    params![project_id],
                    desktop_scan_run_from_row,
                )
                .optional()
        })
    }

    pub fn update_desktop_scan_run(
        &self,
        update: DesktopScanRunUpdate<'_>,
    ) -> Result<DesktopScanRun> {
        let DesktopScanRunUpdate {
            scan_id,
            phase,
            files_seen,
            assets_indexed,
            groups_updated,
            error,
            now_ms,
        } = update;
        self.with_connection(|connection| {
            let completed_at_ms = match phase {
                DesktopScanPhase::Completed
                | DesktopScanPhase::Failed
                | DesktopScanPhase::Cancelled => Some(now_ms),
                _ => None,
            };
            let changed = connection.execute(
                "UPDATE desktop_scan_runs
                 SET phase = ?2,
                     files_seen = ?3,
                     assets_indexed = ?4,
                     groups_updated = ?5,
                     updated_at_ms = ?6,
                     completed_at_ms = ?7,
                     error = ?8
                 WHERE scan_id = ?1",
                params![
                    scan_id,
                    phase.as_str(),
                    files_seen as i64,
                    assets_indexed as i64,
                    groups_updated as i64,
                    now_ms,
                    completed_at_ms,
                    error,
                ],
            )?;
            if changed == 0 {
                return Err(sqlite_data_error("desktop scan run not found"));
            }
            desktop_scan_run_by_id(connection, scan_id)?
                .ok_or_else(|| sqlite_data_error("desktop scan run not found"))
        })
    }

    pub fn record_desktop_scan_files(
        &self,
        scan_id: &str,
        files: &[DesktopScannedFile],
        now_ms: i64,
    ) -> Result<DesktopScanIndexResult> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let run = desktop_scan_run_by_id(&transaction, scan_id)?
                .ok_or_else(|| sqlite_data_error("desktop scan run not found"))?;
            ensure_project_is_active(&transaction, &run.project_id)?;
            transaction.execute(
                "UPDATE desktop_scan_runs
                 SET phase = ?2,
                     files_seen = ?3,
                     updated_at_ms = ?4
                 WHERE scan_id = ?1",
                params![
                    scan_id,
                    DesktopScanPhase::Indexing.as_str(),
                    files.len() as i64,
                    now_ms,
                ],
            )?;

            let mut group_ids = BTreeSet::new();
            let mut assets_indexed = 0usize;
            let source_name = run.root_path.display().to_string();

            for file in files {
                let transfer_id =
                    desktop_scan_transfer_id(&run.project_id, &run.root_path, &file.relative_path);
                let previous_state = desktop_scan_asset_source_state(&transaction, &transfer_id)?;
                let source_status = match previous_state {
                    Some((previous_modified_at_ms, previous_size_bytes))
                        if previous_modified_at_ms != Some(file.modified_at_ms)
                            || previous_size_bytes != file.size_bytes =>
                    {
                        DesktopSourceStatus::Changed
                    }
                    _ => DesktopSourceStatus::Available,
                };
                let record = TransferRecord {
                    transfer_id: transfer_id.clone(),
                    protocol: DESKTOP_SCAN_PROTOCOL.to_string(),
                    status: TransferStatus::Completed,
                    original_path: file.relative_path.clone(),
                    final_filename: file.original_filename.clone(),
                    final_location: Some(StoredObjectLocation::local_path(file.local_path.clone())),
                    size_bytes: file.size_bytes,
                    username: None,
                    remote_addr: None,
                    source_name: Some(source_name.clone()),
                    started_at_ms: run.started_at_ms,
                    completed_at_ms: Some(now_ms),
                    error: None,
                };
                insert_transfer(&transaction, &run.project_id, &record)?;
                if let Some(group_id) =
                    insert_asset_for_transfer(&transaction, &run.project_id, &record)?
                {
                    transaction.execute(
                        "UPDATE assets
                         SET source_status = ?1,
                             source_modified_at_ms = ?2,
                             last_seen_scan_id = ?3,
                             capture_at_ms = COALESCE(capture_at_ms, ?4)
                         WHERE transfer_id = ?5",
                        params![
                            source_status.as_str(),
                            file.modified_at_ms,
                            scan_id,
                            file.capture_time_ms,
                            transfer_id,
                        ],
                    )?;
                    group_ids.insert(group_id);
                    assets_indexed += 1;
                }
            }

            let root_prefix = format!(
                "desktop-scan-{}-%",
                desktop_scan_root_key(&run.project_id, &run.root_path)
            );
            let missing_group_ids = desktop_scan_missing_group_ids(
                &transaction,
                &run.project_id,
                &root_prefix,
                scan_id,
            )?;
            if !missing_group_ids.is_empty() {
                transaction.execute(
                    "UPDATE assets
                     SET source_status = ?4
                     WHERE project_id = ?1
                       AND transfer_id LIKE ?2
                       AND (last_seen_scan_id IS NULL OR last_seen_scan_id <> ?3)",
                    params![
                        run.project_id,
                        root_prefix,
                        scan_id,
                        DesktopSourceStatus::Missing.as_str(),
                    ],
                )?;
                for group_id in missing_group_ids {
                    refresh_group_rollup(&transaction, &group_id)?;
                    group_ids.insert(group_id);
                }
            }

            let group_ids = group_ids.into_iter().collect::<Vec<_>>();
            transaction.execute(
                "UPDATE desktop_scan_runs
                 SET phase = ?2,
                     files_seen = ?3,
                     assets_indexed = ?4,
                     groups_updated = ?5,
                     updated_at_ms = ?6,
                     completed_at_ms = ?6,
                     error = NULL
                 WHERE scan_id = ?1",
                params![
                    scan_id,
                    DesktopScanPhase::Completed.as_str(),
                    files.len() as i64,
                    assets_indexed as i64,
                    group_ids.len() as i64,
                    now_ms,
                ],
            )?;
            transaction.commit()?;
            Ok(DesktopScanIndexResult {
                assets_indexed,
                group_ids,
            })
        })
    }
}

fn desktop_scan_run_by_id(
    connection: &Connection,
    scan_id: &str,
) -> std::result::Result<Option<DesktopScanRun>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT scan_id, project_id, root_path, root_key, root_label, phase,
                    files_seen, assets_indexed, groups_updated, started_at_ms, updated_at_ms,
                    completed_at_ms, error
             FROM desktop_scan_runs
             WHERE scan_id = ?1",
            params![scan_id],
            desktop_scan_run_from_row,
        )
        .optional()
}

fn desktop_scan_asset_source_state(
    connection: &Connection,
    transfer_id: &str,
) -> std::result::Result<Option<(Option<i64>, u64)>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT source_modified_at_ms, size_bytes
             FROM assets
             WHERE transfer_id = ?1",
            params![transfer_id],
            |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, i64>(1)? as u64)),
        )
        .optional()
}

fn desktop_scan_missing_group_ids(
    connection: &Connection,
    project_id: &str,
    root_transfer_prefix: &str,
    scan_id: &str,
) -> std::result::Result<Vec<String>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT DISTINCT group_id
         FROM assets
         WHERE project_id = ?1
           AND transfer_id LIKE ?2
           AND (last_seen_scan_id IS NULL OR last_seen_scan_id <> ?3)",
    )?;
    let rows = statement.query_map(params![project_id, root_transfer_prefix, scan_id], |row| {
        row.get::<_, String>(0)
    })?;
    collect_rows(rows)
}

fn desktop_scan_run_from_row(
    row: &Row<'_>,
) -> std::result::Result<DesktopScanRun, rusqlite::Error> {
    let phase: String = row.get(5)?;
    Ok(DesktopScanRun {
        scan_id: row.get(0)?,
        project_id: row.get(1)?,
        root_path: PathBuf::from(row.get::<_, String>(2)?),
        root_key: row.get(3)?,
        root_label: row.get(4)?,
        phase: DesktopScanPhase::from_str(&phase),
        files_seen: row.get::<_, i64>(6)? as usize,
        assets_indexed: row.get::<_, i64>(7)? as usize,
        groups_updated: row.get::<_, i64>(8)? as usize,
        started_at_ms: row.get(9)?,
        updated_at_ms: row.get(10)?,
        completed_at_ms: row.get(11)?,
        error: row.get(12)?,
    })
}

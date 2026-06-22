use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

mod analysis;
mod analysis_jobs;
mod asset_groups;
mod asset_index;
mod burst_helpers;
mod burst_manual;
mod bursts;
mod desktop_scan;
mod pipeline;
mod projects;
mod publish;
mod receiver;
mod records;
mod schema;
mod subject_assessments;
mod types;

use analysis::{
    apply_model_evaluation_summary, apply_technical_summary, is_model_selected_asset_group,
    save_project_evaluation_settings_for_connection,
};
use analysis_jobs::{
    enqueue_detect_burst_job_for_connection,
    enqueue_portrait_subject_assessment_job_for_connection,
    should_schedule_subject_assessment_for_project,
};
use asset_groups::{
    asset_group_matches, asset_group_matches_analysis, sort_asset_groups_for_query,
    summarize_asset_groups,
};
use asset_index::{
    guest_mark_for_asset_group, insert_asset_for_transfer, insert_transfer,
    received_assets_for_group, refresh_duplicate_info, refresh_group_rollup,
    stored_asset_group_by_id, stored_asset_groups_for_project, trailing_sequence_number,
    user_marks_for_asset_group,
};
use bursts::burst_summary_for_asset_group;
use projects::{ensure_project_exists, ensure_project_is_active, project_by_id};
use records::{
    collect_rows, current_time_ms, normalized_required, project_from_row, publish_item_from_row,
    slugify, stable_key, stored_asset_from_row, transfer_record_from_row,
};
use rusqlite::{params, Connection, OptionalExtension, Row};
use schema::initialize_schema;

use crate::{
    generate_lan_share_token, group_received_assets, AssetGroupPage, AssetGroupQuery,
    AssetUserMarks, GuestMark, ImporterError, LanShareGuestMark, LanShareSession, Result,
    StoredObjectLocation, TransferRecord, TransferStatus,
};

pub use desktop_scan::DesktopScanRunUpdate;
pub use pipeline::{LocalFolderObjectStore, LocalStagedUpload, LocalStagingStore, StagedObject};
pub use types::*;

const DB_FILENAME: &str = "camera-connector.sqlite3";
const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(15);

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let store = Self {
            access_lock: sqlite_access_lock(&path),
            db_path: path,
        };
        store.with_write_connection(|connection| initialize_schema(connection, &store.db_path))?;
        Ok(store)
    }

    pub fn open_state_dir(state_dir: impl AsRef<Path>) -> Result<Self> {
        Self::open(state_dir.as_ref().join(DB_FILENAME))
    }

    pub fn state_dir(&self) -> PathBuf {
        self.db_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn record_transfer(&self, project_id: &str, record: TransferRecord) -> Result<()> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            ensure_project_is_active(&transaction, project_id)?;
            insert_transfer(&transaction, project_id, &record)?;
            if record.status == TransferStatus::Completed {
                if let Some(asset_group_id) =
                    insert_asset_for_transfer(&transaction, project_id, &record)?
                {
                    enqueue_detect_burst_job_for_connection(
                        &transaction,
                        project_id,
                        &asset_group_id,
                    )?;
                    if should_schedule_subject_assessment_for_project(&transaction, project_id)? {
                        enqueue_portrait_subject_assessment_job_for_connection(
                            &transaction,
                            project_id,
                            &asset_group_id,
                        )?;
                    }
                }
            }
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn asset_group_page(
        &self,
        project_id: &str,
        query: AssetGroupQuery,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        self.with_read_connection(|connection| {
            let stored_groups = stored_asset_groups_for_project(connection, project_id)?;
            let mut groups = Vec::new();
            for stored_group in stored_groups {
                let assets =
                    received_assets_for_group(connection, project_id, &stored_group.group_id)?;
                if let Some(mut group) = group_received_assets(assets).into_iter().next() {
                    group.group_id = Some(stored_group.group_id);
                    if asset_group_matches(&group, &query) {
                        if let Some(group_id) = group.group_id.clone() {
                            group.burst =
                                burst_summary_for_asset_group(connection, project_id, &group_id)?;
                            apply_technical_summary(connection, &group_id, &mut group)?;
                            apply_model_evaluation_summary(connection, &group_id, &mut group)?;
                            group.is_model_select = is_model_selected_asset_group(
                                connection,
                                project_id,
                                &group_id,
                                group
                                    .burst
                                    .as_ref()
                                    .map(|burst| burst.burst_group_id.as_str()),
                            )?;
                            group.user_marks =
                                user_marks_for_asset_group(connection, project_id, &group_id)?;
                            group.is_favorite = group.user_marks.favorite;
                            group.is_flagged = group.user_marks.marked;
                            group.guest_mark =
                                guest_mark_for_asset_group(connection, project_id, &group_id)?;
                        }
                        if asset_group_matches_analysis(&group, &query) {
                            groups.push(group);
                        }
                    }
                }
            }

            sort_asset_groups_for_query(&mut groups, query.sort);
            let total_groups = groups.len();
            let summary = summarize_asset_groups(&groups);
            let page_groups = groups
                .into_iter()
                .skip(offset)
                .take(limit)
                .collect::<Vec<_>>();

            Ok(AssetGroupPage {
                groups: page_groups,
                summary,
                offset,
                limit,
                total_groups,
                has_more: offset.saturating_add(limit) < total_groups,
            })
        })
    }

    pub fn assets_for_group(&self, project_id: &str, group_id: &str) -> Result<Vec<StoredAsset>> {
        self.with_read_connection(|connection| {
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
            let rows = statement.query_map(params![project_id, group_id], stored_asset_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn delete_asset_group(
        &self,
        project_id: &str,
        group_id: &str,
    ) -> Result<Option<Vec<StoredAsset>>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            ensure_project_exists(&transaction, project_id)?;
            let mut statement = transaction.prepare(
                "SELECT asset_id, project_id, group_id, transfer_id, group_role,
                        media_kind, format, original_filename, final_filename, normalized_stem, original_path,
                        original_parent_path, final_location_payload, size_bytes, capture_at_ms,
                        received_at_ms, published_at_ms, source_identity, username, remote_addr,
                        source_status, source_modified_at_ms, last_seen_scan_id, duplicate_index,
                        duplicate_count
                 FROM assets
                 WHERE project_id = ?1 AND group_id = ?2
                 ORDER BY published_at_ms ASC, asset_id ASC",
            )?;
            let rows = statement.query_map(params![project_id, group_id], stored_asset_from_row)?;
            let assets = collect_rows(rows)?;
            drop(statement);
            if assets.is_empty() {
                transaction.commit()?;
                return Ok(None);
            }

            let affected_burst_ids = transaction
                .prepare(
                    "SELECT burst_group_id FROM burst_group_members WHERE member_group_id = ?1",
                )?
                .query_map(params![group_id], |row| row.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            transaction.execute(
                "DELETE FROM selection_recommendations
                 WHERE project_id = ?1
                   AND (scope = 'project' OR subject_id = ?2)",
                params![project_id, group_id],
            )?;
            for burst_group_id in &affected_burst_ids {
                transaction.execute(
                    "DELETE FROM selection_recommendations
                     WHERE project_id = ?1 AND subject_id = ?2",
                    params![project_id, burst_group_id],
                )?;
                transaction.execute(
                    "DELETE FROM background_jobs
                     WHERE project_id = ?1 AND entity_type = 'burst_group' AND entity_id = ?2",
                    params![project_id, burst_group_id],
                )?;
            }
            transaction.execute(
                "DELETE FROM background_jobs
                 WHERE project_id = ?1 AND entity_type = 'asset_group' AND entity_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM technical_assessments WHERE asset_group_id = ?1",
                params![group_id],
            )?;
            transaction.execute(
                "DELETE FROM model_evaluations WHERE project_id = ?1 AND asset_group_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM subject_assessments WHERE project_id = ?1 AND asset_group_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM asset_group_user_marks WHERE project_id = ?1 AND group_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM lan_share_guest_marks WHERE project_id = ?1 AND asset_group_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM burst_member_manual_edits WHERE project_id = ?1 AND member_group_id = ?2",
                params![project_id, group_id],
            )?;
            transaction.execute(
                "DELETE FROM burst_group_members WHERE member_group_id = ?1",
                params![group_id],
            )?;
            for asset in &assets {
                transaction.execute(
                    "DELETE FROM publish_queue WHERE project_id = ?1 AND transfer_id = ?2",
                    params![project_id, asset.transfer_id],
                )?;
                transaction.execute(
                    "DELETE FROM assets WHERE project_id = ?1 AND asset_id = ?2",
                    params![project_id, asset.asset_id],
                )?;
                transaction.execute(
                    "DELETE FROM transfers WHERE project_id = ?1 AND transfer_id = ?2",
                    params![project_id, asset.transfer_id],
                )?;
            }
            transaction.execute(
                "DELETE FROM asset_groups WHERE project_id = ?1 AND group_id = ?2",
                params![project_id, group_id],
            )?;
            for burst_group_id in &affected_burst_ids {
                transaction.execute(
                    "UPDATE burst_groups
                     SET member_count = (
                         SELECT COUNT(*) FROM burst_group_members WHERE burst_group_id = ?1
                     ),
                         updated_at_ms = ?2
                     WHERE burst_group_id = ?1",
                    params![burst_group_id, current_time_ms()],
                )?;
                transaction.execute(
                    "DELETE FROM burst_groups
                     WHERE burst_group_id = ?1 AND member_count < 2",
                    params![burst_group_id],
                )?;
            }
            transaction.execute(
                "UPDATE projects SET updated_at_ms = ?1 WHERE project_id = ?2",
                params![current_time_ms(), project_id],
            )?;
            refresh_duplicate_info(&transaction, project_id)?;
            transaction.commit()?;
            Ok(Some(assets))
        })
    }

    pub fn set_asset_group_user_marks(
        &self,
        project_id: &str,
        group_id: &str,
        favorite: Option<bool>,
        marked: Option<bool>,
    ) -> Result<AssetUserMarks> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            if stored_asset_group_by_id(connection, project_id, group_id)?.is_none() {
                return Err(rusqlite::Error::InvalidParameterName(
                    "asset group not found".to_string(),
                ));
            }
            let existing = user_marks_for_asset_group(connection, project_id, group_id)?;
            let next = AssetUserMarks {
                favorite: favorite.unwrap_or(existing.favorite),
                marked: marked.unwrap_or(existing.marked),
            };
            let now = current_time_ms();
            connection.execute(
                "INSERT INTO asset_group_user_marks (
                    project_id, group_id, favorite, marked, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?5)
                 ON CONFLICT(project_id, group_id) DO UPDATE SET
                    favorite = excluded.favorite,
                    marked = excluded.marked,
                    updated_at_ms = excluded.updated_at_ms",
                params![project_id, group_id, next.favorite, next.marked, now,],
            )?;
            Ok(next)
        })
    }

    pub fn create_lan_share_session(
        &self,
        project_id: &str,
        query: AssetGroupQuery,
        title: Option<String>,
        now_ms: i64,
    ) -> Result<LanShareSession> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let token = generate_lan_share_token();
            let share_id = format!("share-{now_ms}-{}", stable_key(&token));
            let query_json = asset_group_query_json(&query)?;
            connection.execute(
                "INSERT INTO lan_share_sessions (
                    share_id, project_id, token, query_json, title, active,
                    created_at_ms, updated_at_ms, stopped_at_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6, NULL)",
                params![
                    &share_id,
                    project_id,
                    &token,
                    query_json,
                    title.as_deref(),
                    now_ms,
                ],
            )?;
            lan_share_session_by_id(connection, &share_id)?
                .ok_or_else(|| sqlite_data_error("lan share session not found after insert"))
        })
    }

    pub fn lan_share_session_by_token(&self, token: &str) -> Result<Option<LanShareSession>> {
        self.with_read_connection(|connection| lan_share_session_by_token(connection, token))
    }

    pub fn stop_lan_share_session(
        &self,
        share_id: &str,
        now_ms: i64,
    ) -> Result<Option<LanShareSession>> {
        self.with_connection(|connection| {
            connection.execute(
                "UPDATE lan_share_sessions
                 SET active = 0, updated_at_ms = ?1, stopped_at_ms = ?1
                 WHERE share_id = ?2",
                params![now_ms, share_id],
            )?;
            lan_share_session_by_id(connection, share_id)
        })
    }

    pub fn set_lan_share_guest_mark(
        &self,
        share_id: &str,
        project_id: &str,
        asset_group_id: &str,
        guest_mark: Option<GuestMark>,
        now_ms: i64,
    ) -> Result<Option<LanShareGuestMark>> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            if lan_share_session_by_id(connection, share_id)?.is_none() {
                return Err(sqlite_data_error("lan share session not found"));
            }
            if stored_asset_group_by_id(connection, project_id, asset_group_id)?.is_none() {
                return Err(sqlite_data_error("asset group not found"));
            }
            if let Some(guest_mark) = guest_mark {
                connection.execute(
                    "INSERT INTO lan_share_guest_marks (
                        share_id, project_id, asset_group_id, guest_mark, updated_at_ms
                     ) VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(share_id, asset_group_id) DO UPDATE SET
                        project_id = excluded.project_id,
                        guest_mark = excluded.guest_mark,
                        updated_at_ms = excluded.updated_at_ms",
                    params![
                        share_id,
                        project_id,
                        asset_group_id,
                        guest_mark.as_wire(),
                        now_ms,
                    ],
                )?;
                return Ok(Some(LanShareGuestMark {
                    share_id: share_id.to_string(),
                    project_id: project_id.to_string(),
                    asset_group_id: asset_group_id.to_string(),
                    guest_mark,
                    updated_at_ms: now_ms,
                }));
            }
            connection.execute(
                "DELETE FROM lan_share_guest_marks
                 WHERE share_id = ?1 AND project_id = ?2 AND asset_group_id = ?3",
                params![share_id, project_id, asset_group_id],
            )?;
            Ok(None)
        })
    }

    pub fn stored_asset_groups(&self, project_id: &str) -> Result<Vec<StoredAssetGroup>> {
        self.with_read_connection(|connection| {
            stored_asset_groups_for_project(connection, project_id)
        })
    }

    pub fn global_asset_summary(&self) -> Result<GlobalAssetSummary> {
        self.with_read_connection(|connection| {
            connection.query_row(
                "SELECT
                        (SELECT COUNT(*) FROM asset_groups),
                        (SELECT COUNT(*) FROM assets),
                        COALESCE((SELECT SUM(size_bytes) FROM assets), 0)",
                [],
                |row| {
                    Ok(GlobalAssetSummary {
                        photo_count: row.get::<_, i64>(0)?.max(0) as usize,
                        file_count: row.get::<_, i64>(1)?.max(0) as usize,
                        storage_bytes: row.get::<_, i64>(2)?.max(0) as u64,
                    })
                },
            )
        })
    }

    pub fn project_id_for_asset_group(&self, asset_group_id: &str) -> Result<Option<String>> {
        self.with_read_connection(|connection| {
            connection
                .query_row(
                    "SELECT project_id FROM asset_groups WHERE group_id = ?1 LIMIT 1",
                    params![asset_group_id],
                    |row| row.get::<_, String>(0),
                )
                .optional()
        })
    }

    pub fn transfer_records(&self, project_id: &str) -> Result<Vec<TransferRecord>> {
        self.with_read_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let mut statement = connection.prepare(
                "SELECT transfer_id, protocol, status, original_path, final_filename,
                        final_location_payload, size_bytes, username, remote_addr, source_name,
                        started_at_ms, completed_at_ms, error
                 FROM transfers
                 WHERE project_id = ?1
                 ORDER BY COALESCE(completed_at_ms, started_at_ms) DESC,
                          started_at_ms DESC,
                          transfer_id DESC",
            )?;
            let rows = statement.query_map(params![project_id], transfer_record_from_row)?;
            collect_rows(rows)
        })
    }

    fn with_connection<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> std::result::Result<T, rusqlite::Error>,
    ) -> Result<T> {
        self.with_write_connection(operation)
    }

    fn with_read_connection<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> std::result::Result<T, rusqlite::Error>,
    ) -> Result<T> {
        let mut connection = self.open_configured_connection()?;
        connection
            .execute_batch("PRAGMA query_only = ON;")
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        operation(&mut connection).map_err(|error| ImporterError::internal(error.to_string()))
    }

    fn with_write_connection<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> std::result::Result<T, rusqlite::Error>,
    ) -> Result<T> {
        let _access_guard = self
            .access_lock
            .lock()
            .map_err(|_| ImporterError::internal("sqlite access lock poisoned"))?;
        let mut connection = self.open_configured_connection()?;
        operation(&mut connection).map_err(|error| ImporterError::internal(error.to_string()))
    }

    fn open_configured_connection(&self) -> Result<Connection> {
        let connection = Connection::open(&self.db_path)
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        connection
            .busy_timeout(SQLITE_BUSY_TIMEOUT)
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        Ok(connection)
    }
}

static SQLITE_ACCESS_LOCKS: OnceLock<Mutex<BTreeMap<PathBuf, Arc<Mutex<()>>>>> = OnceLock::new();

fn sqlite_access_lock(db_path: &Path) -> Arc<Mutex<()>> {
    let key = sqlite_lock_key(db_path);
    let mut locks = SQLITE_ACCESS_LOCKS
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .expect("sqlite access lock registry should not be poisoned");
    locks
        .entry(key)
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

fn sqlite_lock_key(db_path: &Path) -> PathBuf {
    let file_name = db_path.file_name().unwrap_or_default();
    db_path
        .parent()
        .and_then(|parent| std::fs::canonicalize(parent).ok())
        .map(|parent| parent.join(file_name))
        .unwrap_or_else(|| db_path.to_path_buf())
}

fn lan_share_session_by_id(
    connection: &Connection,
    share_id: &str,
) -> std::result::Result<Option<LanShareSession>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT share_id, project_id, token, query_json, title, active,
                    created_at_ms, updated_at_ms, stopped_at_ms
             FROM lan_share_sessions
             WHERE share_id = ?1",
            params![share_id],
            lan_share_session_from_row,
        )
        .optional()
}

fn lan_share_session_by_token(
    connection: &Connection,
    token: &str,
) -> std::result::Result<Option<LanShareSession>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT share_id, project_id, token, query_json, title, active,
                    created_at_ms, updated_at_ms, stopped_at_ms
             FROM lan_share_sessions
             WHERE token = ?1",
            params![token],
            lan_share_session_from_row,
        )
        .optional()
}

fn lan_share_session_from_row(
    row: &Row<'_>,
) -> std::result::Result<LanShareSession, rusqlite::Error> {
    let query_json: String = row.get(3)?;
    Ok(LanShareSession {
        share_id: row.get(0)?,
        project_id: row.get(1)?,
        token: row.get(2)?,
        query: asset_group_query_from_json(query_json)?,
        title: row.get(4)?,
        active: row.get(5)?,
        created_at_ms: row.get(6)?,
        updated_at_ms: row.get(7)?,
        stopped_at_ms: row.get(8)?,
    })
}

fn asset_group_query_json(value: &AssetGroupQuery) -> std::result::Result<String, rusqlite::Error> {
    serde_json::to_string(value).map_err(|error| sqlite_data_error(error.to_string()))
}

fn asset_group_query_from_json(
    value: String,
) -> std::result::Result<AssetGroupQuery, rusqlite::Error> {
    serde_json::from_str(&value).map_err(|error| sqlite_data_error(error.to_string()))
}

fn sqlite_data_error(error: impl ToString) -> rusqlite::Error {
    rusqlite::Error::InvalidParameterName(error.to_string())
}

#[cfg(test)]
mod read_write_concurrency_tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn read_connections_do_not_wait_for_write_transaction_gate() {
        let temp_dir = tempfile::tempdir().expect("temp dir should create");
        let store =
            SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
        store
            .create_project("Concurrent Reads")
            .expect("project should create");

        let writer_store = store.clone();
        let (locked_tx, locked_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let writer = thread::spawn(move || {
            writer_store
                .with_write_connection(|connection| {
                    let transaction = connection.unchecked_transaction()?;
                    transaction.execute(
                        "UPDATE projects
                         SET updated_at_ms = updated_at_ms
                         WHERE project_id = (SELECT project_id FROM projects LIMIT 1)",
                        [],
                    )?;
                    locked_tx
                        .send(())
                        .expect("lock signal should send from writer");
                    release_rx
                        .recv()
                        .expect("writer should wait for release signal");
                    transaction.commit()?;
                    Ok(())
                })
                .expect("writer should finish");
        });

        locked_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("writer should enter write transaction");
        let started_at = Instant::now();
        let projects = store
            .list_projects()
            .expect("read should not wait for writer");
        let elapsed = started_at.elapsed();

        release_tx
            .send(())
            .expect("release signal should send to writer");
        writer.join().expect("writer thread should join");

        assert_eq!(projects.len(), 1);
        assert!(
            elapsed < Duration::from_millis(200),
            "read waited for write transaction for {:?}",
            elapsed
        );
    }
}

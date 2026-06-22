use rusqlite::{params, OptionalExtension};

use super::{
    collect_rows, current_time_ms, enqueue_detect_burst_job_for_connection, ensure_project_exists,
    ensure_project_is_active, insert_asset_for_transfer, insert_transfer, publish_item_from_row,
    PublishQueueItem, PublishQueueSummary, PublishState, PublishTransferMetadata, Result,
    SqliteStore, StoredObjectLocation, TransferRecord, TransferStatus,
};

const FAILED_PUBLISH_RETRY_DELAY_MS: i64 = 30_000;

impl SqliteStore {
    pub fn enqueue_publish(
        &self,
        project_id: &str,
        transfer_id: &str,
        staged_path: &str,
        final_filename: &str,
        size_bytes: u64,
    ) -> Result<PublishQueueItem> {
        let now = current_time_ms();
        let item = PublishQueueItem {
            queue_id: format!("publish-{now}-{transfer_id}"),
            project_id: project_id.to_string(),
            transfer_id: transfer_id.to_string(),
            staged_path: staged_path.to_string(),
            final_filename: final_filename.to_string(),
            size_bytes,
            protocol: None,
            original_path: None,
            username: None,
            remote_addr: None,
            source_name: None,
            started_at_ms: None,
            state: PublishState::Staged,
            attempt_count: 0,
            last_error: None,
            next_attempt_at_ms: None,
            created_at_ms: now,
            updated_at_ms: now,
        };
        self.with_connection(|connection| {
            ensure_project_is_active(connection, project_id)?;
            connection.execute(
                "INSERT INTO publish_queue (
                    queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                    state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    item.queue_id,
                    item.project_id,
                    item.transfer_id,
                    item.staged_path,
                    item.final_filename,
                    item.size_bytes as i64,
                    item.state.as_str(),
                    item.attempt_count as i64,
                    item.last_error,
                    item.next_attempt_at_ms,
                    item.created_at_ms,
                    item.updated_at_ms,
                ],
            )?;
            Ok(item)
        })
    }

    pub fn enqueue_publish_with_metadata(
        &self,
        project_id: &str,
        transfer_id: &str,
        staged_path: &str,
        final_filename: &str,
        size_bytes: u64,
        metadata: PublishTransferMetadata,
    ) -> Result<PublishQueueItem> {
        let now = current_time_ms();
        let item = PublishQueueItem {
            queue_id: format!("publish-{now}-{transfer_id}"),
            project_id: project_id.to_string(),
            transfer_id: transfer_id.to_string(),
            staged_path: staged_path.to_string(),
            final_filename: final_filename.to_string(),
            size_bytes,
            protocol: Some(metadata.protocol),
            original_path: Some(metadata.original_path),
            username: metadata.username,
            remote_addr: metadata.remote_addr,
            source_name: metadata.source_name,
            started_at_ms: Some(metadata.started_at_ms),
            state: PublishState::Staged,
            attempt_count: 0,
            last_error: None,
            next_attempt_at_ms: None,
            created_at_ms: now,
            updated_at_ms: now,
        };
        self.with_connection(|connection| {
            ensure_project_is_active(connection, project_id)?;
            connection.execute(
                "INSERT INTO publish_queue (
                    queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                    protocol, original_path, username, remote_addr, source_name, started_at_ms,
                    state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    item.queue_id,
                    item.project_id,
                    item.transfer_id,
                    item.staged_path,
                    item.final_filename,
                    item.size_bytes as i64,
                    item.protocol,
                    item.original_path,
                    item.username,
                    item.remote_addr,
                    item.source_name,
                    item.started_at_ms,
                    item.state.as_str(),
                    item.attempt_count as i64,
                    item.last_error,
                    item.next_attempt_at_ms,
                    item.created_at_ms,
                    item.updated_at_ms,
                ],
            )?;
            Ok(item)
        })
    }

    pub fn complete_publish(
        &self,
        queue_id: &str,
        final_filename: &str,
        final_location: StoredObjectLocation,
    ) -> Result<TransferRecord> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let item = transaction.query_row(
                "SELECT queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                        protocol, original_path, username, remote_addr, source_name, started_at_ms,
                        state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 FROM publish_queue
                 WHERE queue_id = ?1",
                params![queue_id],
                publish_item_from_row,
            )?;
            ensure_project_is_active(&transaction, &item.project_id)?;
            let protocol = item.protocol.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName(
                    "publish queue item missing protocol".to_string(),
                )
            })?;
            let original_path = item.original_path.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName(
                    "publish queue item missing original path".to_string(),
                )
            })?;
            let started_at_ms = item.started_at_ms.ok_or_else(|| {
                rusqlite::Error::InvalidParameterName(
                    "publish queue item missing started time".to_string(),
                )
            })?;
            let record = TransferRecord {
                transfer_id: item.transfer_id,
                protocol,
                status: TransferStatus::Completed,
                original_path,
                final_filename: final_filename.to_string(),
                final_location: Some(final_location),
                size_bytes: item.size_bytes,
                username: item.username,
                remote_addr: item.remote_addr,
                source_name: item.source_name,
                started_at_ms,
                completed_at_ms: Some(current_time_ms()),
                error: None,
            };
            insert_transfer(&transaction, &item.project_id, &record)?;
            let asset_group_id = insert_asset_for_transfer(&transaction, &item.project_id, &record)?;
            if let Some(asset_group_id) = asset_group_id {
                enqueue_detect_burst_job_for_connection(
                    &transaction,
                    &item.project_id,
                    &asset_group_id,
                )?;
            }
            let changed = transaction.execute(
                "UPDATE publish_queue
                 SET state = ?1, last_error = NULL, next_attempt_at_ms = NULL, updated_at_ms = ?2
                 WHERE queue_id = ?3",
                params![
                    PublishState::Completed.as_str(),
                    current_time_ms(),
                    queue_id
                ],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "publish queue item not found".to_string(),
                ));
            }
            transaction.commit()?;
            Ok(record)
        })
    }

    pub fn mark_publish_failed(&self, queue_id: &str, error: &str) -> Result<()> {
        self.with_connection(|connection| {
            let now = current_time_ms();
            let next_attempt_at_ms = now.saturating_add(FAILED_PUBLISH_RETRY_DELAY_MS);
            let changed = connection.execute(
                "UPDATE publish_queue
                 SET state = ?1, attempt_count = attempt_count + 1, last_error = ?2,
                     next_attempt_at_ms = ?3, updated_at_ms = ?4
                 WHERE queue_id = ?5",
                params![
                    PublishState::Failed.as_str(),
                    error,
                    next_attempt_at_ms,
                    now,
                    queue_id
                ],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "publish queue item not found".to_string(),
                ));
            }
            Ok(())
        })
    }

    pub fn mark_publish_completed(&self, queue_id: &str) -> Result<()> {
        self.with_connection(|connection| {
            let changed = connection.execute(
                "UPDATE publish_queue
                 SET state = ?1, last_error = NULL, next_attempt_at_ms = NULL, updated_at_ms = ?2
                 WHERE queue_id = ?3",
                params![
                    PublishState::Completed.as_str(),
                    current_time_ms(),
                    queue_id
                ],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "publish queue item not found".to_string(),
                ));
            }
            Ok(())
        })
    }

    pub fn failed_publish_items(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<Vec<PublishQueueItem>> {
        self.with_read_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let mut statement = connection.prepare(
                "SELECT queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                        protocol, original_path, username, remote_addr, source_name, started_at_ms,
                        state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 FROM publish_queue
                 WHERE project_id = ?1 AND state = 'failed'
                 ORDER BY updated_at_ms DESC, created_at_ms DESC, queue_id DESC
                 LIMIT ?2",
            )?;
            let rows =
                statement.query_map(params![project_id, limit as i64], publish_item_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn release_failed_publish_retries(&self, project_id: &str) -> Result<usize> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let changed = connection.execute(
                "UPDATE publish_queue
                 SET next_attempt_at_ms = NULL, updated_at_ms = ?1
                 WHERE project_id = ?2
                   AND state = 'failed'
                   AND next_attempt_at_ms IS NOT NULL",
                params![current_time_ms(), project_id],
            )?;
            Ok(changed)
        })
    }

    pub fn claim_next_publish_item(&self) -> Result<Option<PublishQueueItem>> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            let now = current_time_ms();
            let queue_id = transaction
                .query_row(
                    "SELECT queue_id
                     FROM publish_queue
                     WHERE state = 'staged'
                        OR (
                            state = 'failed'
                            AND (next_attempt_at_ms IS NULL OR next_attempt_at_ms <= ?1)
                        )
                     ORDER BY created_at_ms ASC, queue_id ASC
                     LIMIT 1",
                    params![now],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            let Some(queue_id) = queue_id else {
                transaction.commit()?;
                return Ok(None);
            };
            transaction.execute(
                "UPDATE publish_queue
                 SET state = ?1, last_error = NULL, next_attempt_at_ms = NULL, updated_at_ms = ?2
                 WHERE queue_id = ?3
                   AND (
                       state = 'staged'
                       OR (
                           state = 'failed'
                           AND (next_attempt_at_ms IS NULL OR next_attempt_at_ms <= ?4)
                       )
                   )",
                params![PublishState::Publishing.as_str(), now, queue_id, now],
            )?;
            let item = transaction.query_row(
                "SELECT queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                        protocol, original_path, username, remote_addr, source_name, started_at_ms,
                        state, attempt_count, last_error, next_attempt_at_ms, created_at_ms, updated_at_ms
                 FROM publish_queue
                 WHERE queue_id = ?1",
                params![queue_id],
                publish_item_from_row,
            )?;
            transaction.commit()?;
            Ok(Some(item))
        })
    }

    pub fn publish_queue_summary(&self, project_id: &str) -> Result<PublishQueueSummary> {
        self.with_read_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let mut statement = connection.prepare(
                "SELECT state, COUNT(*)
                 FROM publish_queue
                 WHERE project_id = ?1
                 GROUP BY state",
            )?;
            let rows = statement.query_map(params![project_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            let mut summary = PublishQueueSummary::default();
            for row in rows {
                let (state, count) = row?;
                let count = count as usize;
                summary.total_count += count;
                match PublishState::from_str(&state) {
                    PublishState::Staged => summary.staged_count += count,
                    PublishState::Publishing => summary.publishing_count += count,
                    PublishState::Completed => summary.completed_count += count,
                    PublishState::Failed => summary.failed_count += count,
                }
            }
            summary.pending_count =
                summary.staged_count + summary.publishing_count + summary.failed_count;
            Ok(summary)
        })
    }
}

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::{
    group_received_assets, AssetFacetCount, AssetGroupPage, AssetGroupQuery, AssetGroupSummary,
    ImportSource, ImporterError, ObjectFormat, ReceivedAsset, ReceivedAssetGroup, Result,
    StoredObjectLocation, TransferRecord, TransferStatus,
};

const DB_FILENAME: &str = "camera-connector.sqlite3";
const ACTIVE_PROJECT_KEY: &str = "active_project_id";

#[derive(Debug, Clone)]
pub struct SqliteStore {
    db_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub slug: String,
    pub status: ProjectStatus,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub archived_at_ms: Option<i64>,
    pub default_output_target_id: Option<String>,
    pub default_strategy_profile_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Archived,
}

impl ProjectStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "archived" => Self::Archived,
            _ => Self::Active,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredAsset {
    pub asset_id: String,
    pub project_id: String,
    pub group_id: Option<String>,
    pub transfer_id: String,
    pub group_role: String,
    pub group_rank: i64,
    pub format: ObjectFormat,
    pub original_filename: String,
    pub final_filename: String,
    pub normalized_stem: String,
    pub original_path: String,
    pub final_location: Option<StoredObjectLocation>,
    pub size_bytes: u64,
    pub capture_at_ms: Option<i64>,
    pub received_at_ms: Option<i64>,
    pub published_at_ms: Option<i64>,
    pub source_identity: Option<String>,
    pub username: Option<String>,
    pub remote_addr: Option<String>,
    pub duplicate_index: Option<usize>,
    pub duplicate_count: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishQueueItem {
    pub queue_id: String,
    pub project_id: String,
    pub transfer_id: String,
    pub staged_path: String,
    pub final_filename: String,
    pub size_bytes: u64,
    pub state: PublishState,
    pub attempt_count: u32,
    pub last_error: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublishState {
    Staged,
    Publishing,
    Completed,
    Failed,
}

impl PublishState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Staged => "staged",
            Self::Publishing => "publishing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    fn from_str(value: &str) -> Self {
        match value {
            "publishing" => Self::Publishing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::Staged,
        }
    }
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let store = Self { db_path: path };
        store.with_connection(|connection| initialize_schema(connection))?;
        Ok(store)
    }

    pub fn open_state_dir(state_dir: impl AsRef<Path>) -> Result<Self> {
        Self::open(state_dir.as_ref().join(DB_FILENAME))
    }

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
            default_strategy_profile_id: None,
        };
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO projects (
                    project_id, name, slug, status, created_at_ms, updated_at_ms,
                    archived_at_ms, default_output_target_id, default_strategy_profile_id
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    project.project_id,
                    project.name,
                    project.slug,
                    project.status.as_str(),
                    project.created_at_ms,
                    project.updated_at_ms,
                    project.archived_at_ms,
                    project.default_output_target_id,
                    project.default_strategy_profile_id,
                ],
            )?;
            Ok(project)
        })
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT project_id, name, slug, status, created_at_ms, updated_at_ms,
                        archived_at_ms, default_output_target_id, default_strategy_profile_id
                 FROM projects
                 ORDER BY updated_at_ms DESC, name ASC",
            )?;
            let rows = statement.query_map([], project_from_row)?;
            collect_rows(rows)
        })
    }

    pub fn set_active_project(&self, project_id: &str) -> Result<()> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            connection.execute(
                "INSERT INTO app_state (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![ACTIVE_PROJECT_KEY, project_id],
            )?;
            Ok(())
        })
    }

    pub fn active_project(&self) -> Result<Option<Project>> {
        self.with_connection(|connection| {
            let project_id = connection
                .query_row(
                    "SELECT value FROM app_state WHERE key = ?1",
                    params![ACTIVE_PROJECT_KEY],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            let Some(project_id) = project_id else {
                return Ok(None);
            };
            connection
                .query_row(
                    "SELECT project_id, name, slug, status, created_at_ms, updated_at_ms,
                            archived_at_ms, default_output_target_id, default_strategy_profile_id
                     FROM projects
                     WHERE project_id = ?1",
                    params![project_id],
                    project_from_row,
                )
                .optional()
                .map_err(Into::into)
        })
    }

    pub fn record_transfer(&self, project_id: &str, record: TransferRecord) -> Result<()> {
        self.with_connection(|connection| {
            let transaction = connection.unchecked_transaction()?;
            ensure_project_exists(&transaction, project_id)?;
            insert_transfer(&transaction, project_id, &record)?;
            if record.status == TransferStatus::Completed {
                insert_asset_for_transfer(&transaction, project_id, &record)?;
                refresh_duplicate_info(&transaction, project_id)?;
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
        let assets = self.assets_for_project(project_id)?;
        let groups = group_received_assets(assets)
            .into_iter()
            .filter(|group| asset_group_matches(group, &query))
            .collect::<Vec<_>>();
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
    }

    pub fn assets_for_group(
        &self,
        project_id: &str,
        group_id_or_display_key: &str,
    ) -> Result<Vec<StoredAsset>> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT asset_id, project_id, group_id, transfer_id, group_role, group_rank,
                        format, original_filename, final_filename, normalized_stem, original_path,
                        final_location_payload, size_bytes, capture_at_ms, received_at_ms,
                        published_at_ms, source_identity, username, remote_addr,
                        duplicate_index, duplicate_count
                 FROM assets
                 WHERE group_id IN (
                    SELECT group_id FROM asset_groups
                    WHERE project_id = ?1 AND (group_id = ?2 OR display_key = ?2)
                 )
                 ORDER BY group_rank ASC, published_at_ms ASC, asset_id ASC",
            )?;
            let rows = statement.query_map(
                params![project_id, group_id_or_display_key],
                stored_asset_from_row,
            )?;
            collect_rows(rows)
        })
    }

    pub fn transfer_counts(&self, project_id: &str) -> Result<(usize, usize, usize)> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let total = count_transfers(connection, project_id, None)?;
            let completed = count_transfers(connection, project_id, Some("completed"))?;
            let failed = count_transfers(connection, project_id, Some("failed"))?;
            Ok((total as usize, completed as usize, failed as usize))
        })
    }

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
            state: PublishState::Staged,
            attempt_count: 0,
            last_error: None,
            created_at_ms: now,
            updated_at_ms: now,
        };
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            connection.execute(
                "INSERT INTO publish_queue (
                    queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                    state, attempt_count, last_error, created_at_ms, updated_at_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
                    item.created_at_ms,
                    item.updated_at_ms,
                ],
            )?;
            Ok(item)
        })
    }

    pub fn mark_publish_failed(&self, queue_id: &str, error: &str) -> Result<()> {
        self.with_connection(|connection| {
            let changed = connection.execute(
                "UPDATE publish_queue
                 SET state = ?1, attempt_count = attempt_count + 1, last_error = ?2, updated_at_ms = ?3
                 WHERE queue_id = ?4",
                params![PublishState::Failed.as_str(), error, current_time_ms(), queue_id],
            )?;
            if changed == 0 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "publish queue item not found".to_string(),
                ));
            }
            Ok(())
        })
    }

    pub fn pending_publish_items(&self) -> Result<Vec<PublishQueueItem>> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT queue_id, project_id, transfer_id, staged_path, final_filename, size_bytes,
                        state, attempt_count, last_error, created_at_ms, updated_at_ms
                 FROM publish_queue
                 WHERE state IN ('staged', 'failed')
                 ORDER BY created_at_ms ASC, queue_id ASC",
            )?;
            let rows = statement.query_map([], publish_item_from_row)?;
            collect_rows(rows)
        })
    }

    fn assets_for_project(&self, project_id: &str) -> Result<Vec<ReceivedAsset>> {
        self.with_connection(|connection| {
            ensure_project_exists(connection, project_id)?;
            let mut statement = connection.prepare(
                "SELECT asset_id, project_id, group_id, transfer_id, group_role, group_rank,
                        format, original_filename, final_filename, normalized_stem, original_path,
                        final_location_payload, size_bytes, capture_at_ms, received_at_ms,
                        published_at_ms, source_identity, username, remote_addr,
                        duplicate_index, duplicate_count
                 FROM assets
                 WHERE project_id = ?1
                 ORDER BY published_at_ms DESC, asset_id ASC",
            )?;
            let rows = statement.query_map(params![project_id], received_asset_from_row)?;
            collect_rows(rows)
        })
    }

    fn with_connection<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> std::result::Result<T, rusqlite::Error>,
    ) -> Result<T> {
        let mut connection = Connection::open(&self.db_path)
            .map_err(|error| ImporterError::internal(error.to_string()))?;
        operation(&mut connection).map_err(|error| ImporterError::internal(error.to_string()))
    }
}

fn count_transfers(
    connection: &Connection,
    project_id: &str,
    status: Option<&str>,
) -> std::result::Result<i64, rusqlite::Error> {
    match status {
        Some(status) => connection.query_row(
            "SELECT COUNT(*) FROM transfers WHERE project_id = ?1 AND status = ?2",
            params![project_id, status],
            |row| row.get(0),
        ),
        None => connection.query_row(
            "SELECT COUNT(*) FROM transfers WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        ),
    }
}

fn initialize_schema(connection: &Connection) -> std::result::Result<(), rusqlite::Error> {
    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS projects (
            project_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL,
            archived_at_ms INTEGER,
            default_output_target_id TEXT,
            default_strategy_profile_id TEXT
        );

        CREATE TABLE IF NOT EXISTS app_state (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS transfers (
            transfer_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            protocol TEXT NOT NULL,
            status TEXT NOT NULL,
            original_path TEXT NOT NULL,
            final_filename TEXT NOT NULL,
            final_location_kind TEXT,
            final_location_payload TEXT,
            size_bytes INTEGER NOT NULL,
            username TEXT,
            remote_addr TEXT,
            source_name TEXT,
            started_at_ms INTEGER NOT NULL,
            completed_at_ms INTEGER,
            error TEXT
        );

        CREATE TABLE IF NOT EXISTS asset_groups (
            group_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            group_identity TEXT NOT NULL UNIQUE,
            display_key TEXT NOT NULL,
            source_identity TEXT,
            original_parent_path TEXT,
            primary_asset_id TEXT,
            preview_asset_id TEXT,
            member_count INTEGER NOT NULL DEFAULT 0,
            has_raw INTEGER NOT NULL DEFAULT 0,
            has_jpeg INTEGER NOT NULL DEFAULT 0,
            has_video INTEGER NOT NULL DEFAULT 0,
            first_capture_at_ms INTEGER,
            last_capture_at_ms INTEGER,
            first_received_at_ms INTEGER,
            last_received_at_ms INTEGER,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS assets (
            asset_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            group_id TEXT NOT NULL REFERENCES asset_groups(group_id),
            transfer_id TEXT NOT NULL UNIQUE REFERENCES transfers(transfer_id),
            group_role TEXT NOT NULL,
            group_rank INTEGER NOT NULL,
            media_kind TEXT NOT NULL,
            format TEXT NOT NULL,
            original_filename TEXT NOT NULL,
            final_filename TEXT NOT NULL,
            normalized_stem TEXT NOT NULL,
            original_path TEXT NOT NULL,
            original_parent_path TEXT,
            final_location_kind TEXT,
            final_location_payload TEXT,
            size_bytes INTEGER NOT NULL,
            capture_at_ms INTEGER,
            received_at_ms INTEGER,
            published_at_ms INTEGER,
            source_identity TEXT,
            username TEXT,
            remote_addr TEXT,
            duplicate_key TEXT,
            duplicate_index INTEGER,
            duplicate_count INTEGER
        );

        CREATE TABLE IF NOT EXISTS publish_queue (
            queue_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            transfer_id TEXT NOT NULL,
            staged_path TEXT NOT NULL,
            final_filename TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            state TEXT NOT NULL,
            attempt_count INTEGER NOT NULL,
            last_error TEXT,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_assets_project_group ON assets(project_id, group_id);
        CREATE INDEX IF NOT EXISTS idx_asset_groups_project ON asset_groups(project_id, updated_at_ms);
        CREATE INDEX IF NOT EXISTS idx_publish_queue_state ON publish_queue(state, created_at_ms);
        ",
    )
}

fn ensure_project_exists(
    connection: &Connection,
    project_id: &str,
) -> std::result::Result<(), rusqlite::Error> {
    let exists: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM projects WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .optional()?;
    if exists.is_none() {
        return Err(rusqlite::Error::InvalidParameterName(
            "project not found".to_string(),
        ));
    }
    Ok(())
}

fn insert_transfer(
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

fn insert_asset_for_transfer(
    connection: &Connection,
    project_id: &str,
    record: &TransferRecord,
) -> std::result::Result<(), rusqlite::Error> {
    let format = ObjectFormat::from_filename(&record.final_filename);
    if !format.is_supported_media() {
        return Ok(());
    }

    let now = current_time_ms();
    let normalized_stem =
        normalized_stem(&record.final_filename).unwrap_or_else(|| record.final_filename.clone());
    let original_parent_path = original_parent_path(&record.original_path);
    let source_identity = source_identity(record);
    let group_identity = format!(
        "{}\t{}\t{}\t{}",
        project_id,
        source_identity.clone().unwrap_or_default(),
        original_parent_path.clone().unwrap_or_default(),
        normalized_stem
    );
    let group_id = format!("group-{}", stable_key(&group_identity));
    let final_location = record.resolved_final_location();
    let final_location_payload = final_location_json(final_location.as_ref())?;
    let group_role = group_role(format).to_string();
    let group_rank = group_rank(format);
    let published_at_ms = record.completed_at_ms.or(Some(record.started_at_ms));
    let duplicate_key = duplicate_key(record);

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
            original_parent_path,
            now,
            now,
        ],
    )?;

    connection.execute(
        "INSERT OR REPLACE INTO assets (
            asset_id, project_id, group_id, transfer_id, group_role, group_rank, media_kind, format,
            original_filename, final_filename, normalized_stem, original_path, original_parent_path,
            final_location_kind, final_location_payload, size_bytes, capture_at_ms, received_at_ms,
            published_at_ms, source_identity, username, remote_addr, duplicate_key
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)",
        params![
            record.transfer_id,
            project_id,
            group_id,
            record.transfer_id,
            group_role,
            group_rank,
            media_kind(format),
            format_name(format),
            original_filename(&record.original_path),
            record.final_filename,
            normalized_stem,
            record.original_path,
            original_parent_path,
            final_location.as_ref().map(StoredObjectLocation::kind),
            final_location_payload,
            record.size_bytes as i64,
            Option::<i64>::None,
            published_at_ms,
            published_at_ms,
            source_identity.clone(),
            record.username,
            record.remote_addr,
            duplicate_key,
        ],
    )?;
    refresh_group_rollup(connection, &group_id)?;
    Ok(())
}

fn refresh_group_rollup(
    connection: &Connection,
    group_id: &str,
) -> std::result::Result<(), rusqlite::Error> {
    let primary_asset_id: Option<String> = connection
        .query_row(
            "SELECT asset_id FROM assets WHERE group_id = ?1 ORDER BY group_rank ASC, published_at_ms ASC, asset_id ASC LIMIT 1",
            params![group_id],
            |row| row.get(0),
        )
        .optional()?;
    let preview_asset_id: Option<String> = connection
        .query_row(
            "SELECT asset_id FROM assets WHERE group_id = ?1 AND group_role = 'jpeg'
             ORDER BY group_rank ASC, published_at_ms ASC, asset_id ASC LIMIT 1",
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

fn refresh_duplicate_info(
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

fn project_from_row(row: &Row<'_>) -> std::result::Result<Project, rusqlite::Error> {
    let status: String = row.get(3)?;
    Ok(Project {
        project_id: row.get(0)?,
        name: row.get(1)?,
        slug: row.get(2)?,
        status: ProjectStatus::from_str(&status),
        created_at_ms: row.get(4)?,
        updated_at_ms: row.get(5)?,
        archived_at_ms: row.get(6)?,
        default_output_target_id: row.get(7)?,
        default_strategy_profile_id: row.get(8)?,
    })
}

fn received_asset_from_row(row: &Row<'_>) -> std::result::Result<ReceivedAsset, rusqlite::Error> {
    let stored = stored_asset_from_row(row)?;
    let mut asset = ReceivedAsset::new(
        stored.asset_id,
        stored.final_filename,
        stored.size_bytes,
        import_source_from_transfer_id(&stored.transfer_id),
    );
    asset.format = stored.format;
    asset.group_key = Some(stored.normalized_stem);
    asset.storage_location = stored.final_location;
    asset.original_path = Some(stored.original_path);
    asset.username = stored.username;
    asset.display_source = stored.source_identity;
    asset.remote_addr = stored.remote_addr;
    asset.received_time_ms = stored.received_at_ms;
    asset.capture_time_ms = stored.capture_at_ms;
    asset.duplicate_index = stored.duplicate_index;
    asset.duplicate_count = stored.duplicate_count;
    asset.virtual_display_path = asset.display_source.as_deref().map(|source| {
        format!(
            "{source}/{}",
            asset.original_path.as_deref().unwrap_or(&asset.filename)
        )
    });
    Ok(asset)
}

fn stored_asset_from_row(row: &Row<'_>) -> std::result::Result<StoredAsset, rusqlite::Error> {
    let format: String = row.get(6)?;
    let final_location_payload: Option<String> = row.get(11)?;
    Ok(StoredAsset {
        asset_id: row.get(0)?,
        project_id: row.get(1)?,
        group_id: row.get(2)?,
        transfer_id: row.get(3)?,
        group_role: row.get(4)?,
        group_rank: row.get(5)?,
        format: parse_format(&format),
        original_filename: row.get(7)?,
        final_filename: row.get(8)?,
        normalized_stem: row.get(9)?,
        original_path: row.get(10)?,
        final_location: parse_location(final_location_payload)?,
        size_bytes: row.get::<_, i64>(12)? as u64,
        capture_at_ms: row.get(13)?,
        received_at_ms: row.get(14)?,
        published_at_ms: row.get(15)?,
        source_identity: row.get(16)?,
        username: row.get(17)?,
        remote_addr: row.get(18)?,
        duplicate_index: row.get::<_, Option<i64>>(19)?.map(|value| value as usize),
        duplicate_count: row.get::<_, Option<i64>>(20)?.map(|value| value as usize),
    })
}

fn publish_item_from_row(row: &Row<'_>) -> std::result::Result<PublishQueueItem, rusqlite::Error> {
    let state: String = row.get(6)?;
    Ok(PublishQueueItem {
        queue_id: row.get(0)?,
        project_id: row.get(1)?,
        transfer_id: row.get(2)?,
        staged_path: row.get(3)?,
        final_filename: row.get(4)?,
        size_bytes: row.get::<_, i64>(5)? as u64,
        state: PublishState::from_str(&state),
        attempt_count: row.get::<_, i64>(7)? as u32,
        last_error: row.get(8)?,
        created_at_ms: row.get(9)?,
        updated_at_ms: row.get(10)?,
    })
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&Row<'_>) -> std::result::Result<T, rusqlite::Error>>,
) -> std::result::Result<Vec<T>, rusqlite::Error> {
    rows.collect()
}

fn summarize_asset_groups(groups: &[ReceivedAssetGroup]) -> AssetGroupSummary {
    let mut source_counts = BTreeMap::<String, usize>::new();
    let mut remote_addr_counts = BTreeMap::<String, usize>::new();
    for group in groups {
        if let Some(source) = group.primary.display_source.as_ref() {
            *source_counts.entry(source.clone()).or_default() += 1;
        }
        if let Some(remote_addr) = group.primary.remote_addr.as_ref() {
            *remote_addr_counts.entry(remote_addr.clone()).or_default() += 1;
        }
    }
    AssetGroupSummary {
        group_count: groups.len(),
        asset_count: groups.iter().map(|group| group_assets(group).len()).sum(),
        groups_with_jpeg: groups.iter().filter(|group| group.jpeg.is_some()).count(),
        groups_with_raw: groups.iter().filter(|group| group.raw.is_some()).count(),
        groups_with_video: groups.iter().filter(|group| group.video.is_some()).count(),
        source_counts: facet_counts(source_counts),
        remote_addr_counts: facet_counts(remote_addr_counts),
    }
}

fn asset_group_matches(group: &ReceivedAssetGroup, query: &AssetGroupQuery) -> bool {
    group_assets(group).into_iter().any(|asset| {
        query
            .username
            .as_ref()
            .map(|expected| asset.username.as_ref() == Some(expected))
            .unwrap_or(true)
            && query
                .source_name
                .as_ref()
                .map(|expected| asset.display_source.as_ref() == Some(expected))
                .unwrap_or(true)
            && query
                .remote_addr
                .as_ref()
                .map(|expected| asset.remote_addr.as_ref() == Some(expected))
                .unwrap_or(true)
            && query
                .original_path
                .as_ref()
                .map(|expected| {
                    asset
                        .original_path
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(&expected.to_ascii_lowercase())
                })
                .unwrap_or(true)
            && query
                .format
                .map(|expected| asset.format == expected)
                .unwrap_or(true)
    })
}

fn group_assets(group: &ReceivedAssetGroup) -> Vec<&ReceivedAsset> {
    let mut assets = Vec::new();
    push_unique_asset(&mut assets, &group.primary);
    if let Some(asset) = group.jpeg.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    if let Some(asset) = group.raw.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    if let Some(asset) = group.video.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    assets
}

fn push_unique_asset<'a>(assets: &mut Vec<&'a ReceivedAsset>, asset: &'a ReceivedAsset) {
    if !assets.iter().any(|existing| existing.id == asset.id) {
        assets.push(asset);
    }
}

fn facet_counts(counts: BTreeMap<String, usize>) -> Vec<AssetFacetCount> {
    counts
        .into_iter()
        .map(|(value, group_count)| AssetFacetCount { value, group_count })
        .collect()
}

fn final_location_json(
    location: Option<&StoredObjectLocation>,
) -> std::result::Result<Option<String>, rusqlite::Error> {
    location
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
}

fn parse_location(
    value: Option<String>,
) -> std::result::Result<Option<StoredObjectLocation>, rusqlite::Error> {
    value
        .map(|json| serde_json::from_str(&json))
        .transpose()
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
}

fn normalized_required(field: &str, value: &str) -> Result<String> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(ImporterError::internal(format!("{field} cannot be empty")));
    }
    Ok(normalized)
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn slugify(value: &str) -> String {
    let slug = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "project".to_string()
    } else {
        slug
    }
}

fn stable_key(value: &str) -> String {
    let mut hash = 1469598103934665603_u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}

fn normalized_stem(filename: &str) -> Option<String> {
    let name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    let (stem, _) = name.rsplit_once('.')?;
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_ascii_uppercase())
    }
}

fn original_parent_path(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let parent = normalized
        .rsplit_once('/')
        .map(|(parent, _)| parent.trim_matches('/'))?;
    if parent.is_empty() {
        None
    } else {
        Some(parent.to_string())
    }
}

fn original_filename(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn source_identity(record: &TransferRecord) -> Option<String> {
    record
        .username
        .clone()
        .or_else(|| record.source_name.clone())
        .or_else(|| record.remote_addr.clone())
}

fn duplicate_key(record: &TransferRecord) -> Option<String> {
    let original = normalized_duplicate_segment(&record.original_path)?;
    let identity = source_identity(record)
        .and_then(|value| normalized_duplicate_segment(&value))
        .unwrap_or_else(|| "-".to_string());
    Some(format!("{identity}\t{original}"))
}

fn normalized_duplicate_segment(value: &str) -> Option<String> {
    let normalized = value.trim().replace('\\', "/").to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn transfer_status_name(status: TransferStatus) -> &'static str {
    match status {
        TransferStatus::Completed => "completed",
        TransferStatus::Failed => "failed",
    }
}

fn format_name(format: ObjectFormat) -> &'static str {
    match format {
        ObjectFormat::Jpeg => "jpeg",
        ObjectFormat::Nef => "nef",
        ObjectFormat::Nrw => "nrw",
        ObjectFormat::Cr2 => "cr2",
        ObjectFormat::Cr3 => "cr3",
        ObjectFormat::Arw => "arw",
        ObjectFormat::Raf => "raf",
        ObjectFormat::Rw2 => "rw2",
        ObjectFormat::Orf => "orf",
        ObjectFormat::Pef => "pef",
        ObjectFormat::Dng => "dng",
        ObjectFormat::Mov => "mov",
        ObjectFormat::Mp4 => "mp4",
        ObjectFormat::Tiff => "tiff",
        ObjectFormat::Unknown => "unknown",
    }
}

fn parse_format(value: &str) -> ObjectFormat {
    match value {
        "jpeg" => ObjectFormat::Jpeg,
        "nef" => ObjectFormat::Nef,
        "nrw" => ObjectFormat::Nrw,
        "cr2" => ObjectFormat::Cr2,
        "cr3" => ObjectFormat::Cr3,
        "arw" => ObjectFormat::Arw,
        "raf" => ObjectFormat::Raf,
        "rw2" => ObjectFormat::Rw2,
        "orf" => ObjectFormat::Orf,
        "pef" => ObjectFormat::Pef,
        "dng" => ObjectFormat::Dng,
        "mov" => ObjectFormat::Mov,
        "mp4" => ObjectFormat::Mp4,
        "tiff" => ObjectFormat::Tiff,
        _ => ObjectFormat::Unknown,
    }
}

fn group_role(format: ObjectFormat) -> &'static str {
    if format == ObjectFormat::Jpeg {
        "jpeg"
    } else if format.is_raw() {
        "raw"
    } else if format.is_video() {
        "video"
    } else {
        "other"
    }
}

fn group_rank(format: ObjectFormat) -> i64 {
    match group_role(format) {
        "jpeg" => 0,
        "raw" => 1,
        "video" => 2,
        _ => 3,
    }
}

fn media_kind(format: ObjectFormat) -> &'static str {
    if format.is_video() {
        "video"
    } else if format.is_supported_media() {
        "photo"
    } else {
        "unknown"
    }
}

fn import_source_from_transfer_id(transfer_id: &str) -> ImportSource {
    if transfer_id.starts_with("sftp:") {
        ImportSource::SftpPush
    } else if transfer_id.starts_with("ftp:") {
        ImportSource::FtpPush
    } else {
        ImportSource::ManualDrop
    }
}

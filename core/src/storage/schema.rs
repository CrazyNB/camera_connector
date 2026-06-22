use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use rusqlite::Connection;

use super::sqlite_lock_key;

pub(super) fn initialize_schema(
    connection: &Connection,
    db_path: &Path,
) -> std::result::Result<(), rusqlite::Error> {
    ensure_wal_mode(connection, db_path)?;
    connection.execute_batch(
        "
        PRAGMA synchronous = NORMAL;
        PRAGMA wal_autocheckpoint = 1000;
        ",
    )?;
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
            default_output_target_id TEXT
        );

        CREATE TABLE IF NOT EXISTS project_evaluation_settings (
            project_id TEXT PRIMARY KEY REFERENCES projects(project_id),
            auto_evaluate_on_upload INTEGER NOT NULL,
            auto_burst_recommendation_enabled INTEGER NOT NULL,
            project_recommendation_mode TEXT NOT NULL,
            prompt_pack_id TEXT,
            model_provider_settings_id TEXT,
            scene_profile TEXT NOT NULL,
            cv_policy TEXT NOT NULL,
            cv_policy_overrides_json TEXT,
            allow_risky_model_selects INTEGER NOT NULL,
            max_image_side INTEGER,
            batch_size INTEGER,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS evaluation_runs (
            run_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            run_type TEXT NOT NULL,
            trigger TEXT NOT NULL,
            status TEXT NOT NULL,
            provider_kind TEXT NOT NULL,
            provider_model TEXT NOT NULL,
            prompt_pack_id TEXT,
            prompt_pack_version TEXT,
            prompt_hash TEXT,
            settings_snapshot_json TEXT NOT NULL,
            error_message TEXT,
            started_at_ms INTEGER,
            completed_at_ms INTEGER,
            created_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS subject_assessments (
            assessment_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            asset_group_id TEXT NOT NULL,
            subject_type TEXT NOT NULL,
            detector_kind TEXT NOT NULL,
            detector_version TEXT NOT NULL,
            status TEXT NOT NULL,
            gate_status TEXT NOT NULL,
            regions_json TEXT NOT NULL,
            signals_json TEXT NOT NULL,
            summary TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS receiver_accounts (
            username TEXT PRIMARY KEY,
            password_hash TEXT,
            device_name TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS connected_devices (
            remote_addr TEXT PRIMARY KEY,
            source_name TEXT,
            username TEXT,
            first_seen_at_ms INTEGER NOT NULL,
            last_seen_at_ms INTEGER NOT NULL,
            last_disconnected_at_ms INTEGER,
            last_remote_port INTEGER,
            active_connections INTEGER NOT NULL,
            online INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS receiver_status (
            key TEXT PRIMARY KEY,
            phase TEXT NOT NULL,
            protocol TEXT,
            auth_mode TEXT NOT NULL,
            local_addr TEXT,
            output_dir TEXT,
            state_dir TEXT,
            account_count INTEGER NOT NULL,
            message TEXT,
            updated_at_ms INTEGER NOT NULL
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

        CREATE TABLE IF NOT EXISTS desktop_scan_runs (
            scan_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            root_path TEXT NOT NULL,
            root_key TEXT NOT NULL,
            root_label TEXT NOT NULL,
            phase TEXT NOT NULL,
            files_seen INTEGER NOT NULL,
            assets_indexed INTEGER NOT NULL,
            groups_updated INTEGER NOT NULL,
            started_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL,
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
            source_status TEXT NOT NULL DEFAULT 'available',
            source_modified_at_ms INTEGER,
            last_seen_scan_id TEXT,
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
            protocol TEXT,
            original_path TEXT,
            username TEXT,
            remote_addr TEXT,
            source_name TEXT,
            started_at_ms INTEGER,
            state TEXT NOT NULL,
            attempt_count INTEGER NOT NULL,
            last_error TEXT,
            next_attempt_at_ms INTEGER,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS background_jobs (
            job_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            job_type TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            dedupe_key TEXT NOT NULL,
            status TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            attempts INTEGER NOT NULL DEFAULT 0,
            next_attempt_at_ms INTEGER,
            last_error TEXT,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS burst_groups (
            burst_group_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            source_identity TEXT,
            started_at_ms INTEGER,
            ended_at_ms INTEGER,
            member_count INTEGER NOT NULL,
            grouping_version INTEGER NOT NULL,
            recommendation_status TEXT NOT NULL,
            manual_grouping_state TEXT,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS burst_group_members (
            burst_group_id TEXT NOT NULL REFERENCES burst_groups(burst_group_id) ON DELETE CASCADE,
            member_group_id TEXT NOT NULL REFERENCES asset_groups(group_id),
            PRIMARY KEY(burst_group_id, member_group_id)
        );

        CREATE TABLE IF NOT EXISTS burst_member_manual_edits (
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            member_group_id TEXT NOT NULL REFERENCES asset_groups(group_id),
            action TEXT NOT NULL,
            manual_group_id TEXT,
            created_at_ms INTEGER NOT NULL,
            PRIMARY KEY(project_id, member_group_id, action)
        );

        CREATE TABLE IF NOT EXISTS asset_group_user_marks (
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            group_id TEXT NOT NULL REFERENCES asset_groups(group_id) ON DELETE CASCADE,
            favorite INTEGER NOT NULL DEFAULT 0,
            marked INTEGER NOT NULL DEFAULT 0,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL,
            PRIMARY KEY(project_id, group_id)
        );

        CREATE TABLE IF NOT EXISTS lan_share_sessions (
            share_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            token TEXT NOT NULL UNIQUE,
            query_json TEXT NOT NULL,
            title TEXT,
            active INTEGER NOT NULL,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL,
            stopped_at_ms INTEGER
        );

        CREATE TABLE IF NOT EXISTS lan_share_guest_marks (
            share_id TEXT NOT NULL REFERENCES lan_share_sessions(share_id) ON DELETE CASCADE,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            asset_group_id TEXT NOT NULL REFERENCES asset_groups(group_id) ON DELETE CASCADE,
            guest_mark TEXT NOT NULL,
            updated_at_ms INTEGER NOT NULL,
            PRIMARY KEY(share_id, asset_group_id)
        );

        CREATE TABLE IF NOT EXISTS technical_assessments (
            asset_group_id TEXT NOT NULL,
            assessor_version TEXT NOT NULL,
            status TEXT NOT NULL,
            gate_status TEXT NOT NULL,
            defect_flags_json TEXT NOT NULL,
            preview_source TEXT,
            visual_signature TEXT,
            analyzed_at_ms INTEGER NOT NULL,
            PRIMARY KEY(asset_group_id, assessor_version)
        );

        CREATE TABLE IF NOT EXISTS model_evaluations (
            evaluation_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            project_id TEXT NOT NULL,
            asset_group_id TEXT NOT NULL,
            evaluator_kind TEXT NOT NULL,
            evaluator_version TEXT NOT NULL,
            status TEXT NOT NULL,
            score INTEGER NOT NULL,
            tier TEXT NOT NULL,
            selectable INTEGER NOT NULL,
            summary TEXT NOT NULL,
            strengths_json TEXT NOT NULL,
            weaknesses_json TEXT NOT NULL,
            technical_warnings_json TEXT NOT NULL,
            prompt_pack_id TEXT,
            prompt_pack_version TEXT,
            prompt_hash TEXT,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS selection_recommendations (
            recommendation_id TEXT PRIMARY KEY,
            run_id TEXT,
            scope TEXT NOT NULL,
            project_id TEXT NOT NULL,
            subject_id TEXT NOT NULL,
            selected_asset_group_ids_json TEXT NOT NULL,
            candidate_asset_group_ids_json TEXT NOT NULL,
            rejected_asset_group_ids_json TEXT NOT NULL,
            source TEXT NOT NULL,
            status TEXT NOT NULL,
            confidence REAL NOT NULL,
            reason TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_assets_project_group ON assets(project_id, group_id);
        CREATE INDEX IF NOT EXISTS idx_asset_groups_project ON asset_groups(project_id, updated_at_ms);
        CREATE INDEX IF NOT EXISTS idx_desktop_scan_runs_project ON desktop_scan_runs(project_id, started_at_ms);
        CREATE INDEX IF NOT EXISTS idx_connected_devices_username ON connected_devices(username);
        CREATE INDEX IF NOT EXISTS idx_connected_devices_sort ON connected_devices(online, last_seen_at_ms);
        CREATE INDEX IF NOT EXISTS idx_receiver_accounts_enabled ON receiver_accounts(enabled, updated_at_ms);
        CREATE INDEX IF NOT EXISTS idx_publish_queue_state ON publish_queue(state, created_at_ms);
        CREATE INDEX IF NOT EXISTS idx_background_jobs_claim ON background_jobs(status, priority, next_attempt_at_ms, created_at_ms);
        CREATE INDEX IF NOT EXISTS idx_background_jobs_dedupe ON background_jobs(dedupe_key);
        CREATE INDEX IF NOT EXISTS idx_evaluation_runs_project ON evaluation_runs(project_id, run_type, status, created_at_ms);
        CREATE INDEX IF NOT EXISTS idx_subject_assessments_group ON subject_assessments(project_id, asset_group_id, subject_type);
        CREATE INDEX IF NOT EXISTS idx_burst_groups_project ON burst_groups(project_id, updated_at_ms);
        CREATE INDEX IF NOT EXISTS idx_burst_members_group ON burst_group_members(member_group_id, burst_group_id);
        CREATE INDEX IF NOT EXISTS idx_burst_member_manual_edits_project ON burst_member_manual_edits(project_id, action, member_group_id);
        CREATE INDEX IF NOT EXISTS idx_asset_group_user_marks_project ON asset_group_user_marks(project_id, favorite, marked);
        CREATE INDEX IF NOT EXISTS idx_lan_share_sessions_token ON lan_share_sessions(token);
        CREATE INDEX IF NOT EXISTS idx_lan_share_guest_marks_project ON lan_share_guest_marks(project_id, asset_group_id);
        CREATE INDEX IF NOT EXISTS idx_technical_assessments_status ON technical_assessments(status, gate_status);
        CREATE INDEX IF NOT EXISTS idx_model_evaluations_project ON model_evaluations(project_id, status, tier);
        CREATE INDEX IF NOT EXISTS idx_model_evaluations_asset_group ON model_evaluations(asset_group_id, evaluator_version);
        CREATE INDEX IF NOT EXISTS idx_recommendations_scope ON selection_recommendations(project_id, scope, subject_id, status);
        ",
    )?;
    ensure_desktop_scan_asset_columns(connection)?;
    connection.execute(
        "CREATE INDEX IF NOT EXISTS idx_assets_desktop_scan
         ON assets(project_id, transfer_id, last_seen_scan_id)",
        [],
    )?;
    Ok(())
}

static SQLITE_WAL_CONFIGURED_PATHS: OnceLock<Mutex<BTreeSet<PathBuf>>> = OnceLock::new();

fn ensure_wal_mode(
    connection: &Connection,
    db_path: &Path,
) -> std::result::Result<(), rusqlite::Error> {
    let key = sqlite_lock_key(db_path);
    {
        let configured = SQLITE_WAL_CONFIGURED_PATHS
            .get_or_init(|| Mutex::new(BTreeSet::new()))
            .lock()
            .expect("sqlite WAL registry should not be poisoned");
        if configured.contains(&key) {
            return Ok(());
        }
    }

    connection.query_row("PRAGMA journal_mode = WAL", [], |row| {
        row.get::<_, String>(0)
    })?;

    SQLITE_WAL_CONFIGURED_PATHS
        .get_or_init(|| Mutex::new(BTreeSet::new()))
        .lock()
        .expect("sqlite WAL registry should not be poisoned")
        .insert(key);
    Ok(())
}

fn ensure_desktop_scan_asset_columns(
    connection: &Connection,
) -> std::result::Result<(), rusqlite::Error> {
    add_column_if_missing(
        connection,
        "assets",
        "source_status",
        "TEXT NOT NULL DEFAULT 'available'",
    )?;
    add_column_if_missing(connection, "assets", "source_modified_at_ms", "INTEGER")?;
    add_column_if_missing(connection, "assets", "last_seen_scan_id", "TEXT")?;
    Ok(())
}

fn add_column_if_missing(
    connection: &Connection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> std::result::Result<(), rusqlite::Error> {
    let columns = table_columns(connection, table_name)?;
    if columns.is_empty() || columns.contains(column_name) {
        return Ok(());
    }
    connection.execute(
        &format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_definition}"),
        [],
    )?;
    Ok(())
}

fn table_columns(
    connection: &Connection,
    table_name: &str,
) -> std::result::Result<BTreeSet<String>, rusqlite::Error> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    let mut columns = BTreeSet::new();
    for row in rows {
        columns.insert(row?);
    }
    Ok(columns)
}

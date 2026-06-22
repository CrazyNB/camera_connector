use camera_connector_core::{
    AssetGroupQuery, AssetGroupSort, AssetUserMarks, GuestMark, ProjectCapabilities,
    ProjectKind, ProjectStatus, SqliteStore, StoredObjectLocation, TransferRecord, TransferStatus,
};
use rusqlite::Connection;
use std::{thread, time::Duration};

#[path = "storage_store_tests/publish.rs"]
mod publish;
#[path = "storage_store_tests/receiver.rs"]
mod receiver;

fn table_has_column(connection: &Connection, table_name: &str, column_name: &str) -> bool {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table_name})"))
        .expect("table info should prepare");
    let rows = statement
        .query_map([], |row| row.get::<_, String>(1))
        .expect("table info should query");
    let column_names: Vec<_> = rows.filter_map(Result::ok).collect();
    column_names
        .iter()
        .any(|name| name.eq_ignore_ascii_case(column_name))
}

fn table_exists(connection: &Connection, table_name: &str) -> bool {
    connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table_name],
            |_| Ok(()),
        )
        .is_ok()
}

#[test]
fn sqlite_store_opens_in_wal_mode() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let db_path = temp_dir.path().join("state.sqlite");
    let _store = SqliteStore::open(&db_path).expect("store should open");
    let connection = Connection::open(db_path).expect("sqlite connection should open");

    let journal_mode: String = connection
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .expect("journal mode should query");

    assert_eq!(journal_mode.to_ascii_lowercase(), "wal");
}

#[test]
fn sqlite_store_schema_does_not_persist_rank_columns() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let db_path = temp_dir.path().join("state.sqlite");
    let _store = SqliteStore::open(&db_path).expect("store should open");
    let connection = Connection::open(db_path).expect("sqlite connection should open");

    assert!(!table_has_column(&connection, "assets", "group_rank"));
    assert!(!table_has_column(
        &connection,
        "burst_group_members",
        "member_rank"
    ));
}

#[test]
fn sqlite_store_schema_excludes_app_runtime_configuration_tables() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let db_path = temp_dir.path().join("state.sqlite");
    let _store = SqliteStore::open(&db_path).expect("store should open");
    let connection = Connection::open(&db_path).expect("connection should open");

    assert!(!table_exists(&connection, "app_state"));
    assert!(!table_exists(&connection, "model_provider_settings"));
}

#[test]
fn sqlite_store_schema_uses_selection_recommendations_for_model_results_only() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let db_path = temp_dir.path().join("state.sqlite");
    let _store = SqliteStore::open(&db_path).expect("store should open");
    let connection = Connection::open(&db_path).expect("connection should open");

    for column in [
        "recommendation_id",
        "run_id",
        "scope",
        "project_id",
        "subject_id",
        "selected_asset_group_ids_json",
        "candidate_asset_group_ids_json",
        "rejected_asset_group_ids_json",
        "source",
        "status",
        "confidence",
        "reason",
        "created_at_ms",
        "updated_at_ms",
    ] {
        assert!(
            table_has_column(&connection, "selection_recommendations", column),
            "selection_recommendations should have model recommendation column {column}"
        );
    }
}

#[test]
fn sqlite_store_waits_for_short_concurrent_write_locks() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let db_path = temp_dir.path().join("state.sqlite");
    let store = SqliteStore::open(&db_path).expect("store should open");
    store
        .create_project("Concurrent Project")
        .expect("project should create");

    let lock_connection = Connection::open(&db_path).expect("lock connection should open");
    lock_connection
        .execute("BEGIN EXCLUSIVE TRANSACTION", [])
        .expect("exclusive transaction should start");

    let query_db_path = db_path.clone();
    let handle = thread::spawn(move || {
        let query_store =
            SqliteStore::open(&query_db_path).expect("store should wait for lock release");
        query_store
            .list_projects()
            .expect("projects should list after lock release")
    });

    thread::sleep(Duration::from_millis(100));
    lock_connection
        .execute("COMMIT", [])
        .expect("exclusive transaction should commit");

    let projects = handle.join().expect("query thread should finish");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, "Concurrent Project");
}

#[test]
fn sqlite_store_creates_projects() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");

    let project = store
        .create_project("Studio Product Shoot")
        .expect("project should create");
    let projects = store.list_projects().expect("projects should list");

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].project_id, project.project_id);
    assert_eq!(projects[0].status.as_str(), "active");
}

#[test]
fn sqlite_store_archives_and_restores_projects() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Archive Me")
        .expect("project should create");

    let archived = store
        .archive_project(&project.project_id)
        .expect("project should archive");

    assert_eq!(archived.status, ProjectStatus::Archived);
    assert!(archived.archived_at_ms.is_some());

    let restored = store
        .restore_project(&project.project_id)
        .expect("project should restore");

    assert_eq!(restored.status, ProjectStatus::Active);
    assert!(restored.archived_at_ms.is_none());
}

#[test]
fn sqlite_store_deletes_project_and_all_owned_storage_rows() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Delete Whole Project")
        .expect("project should create");
    let other_project = store
        .create_project("Keep Project")
        .expect("other project should create");

    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:delete-jpg", "DCIM/100/IMG_7001.JPG", 1000),
        )
        .expect("project transfer should record");
    store
        .record_transfer(
            &other_project.project_id,
            completed_transfer("ftp:keep-jpg", "DCIM/100/IMG_8001.JPG", 1001),
        )
        .expect("other project transfer should record");

    let deleted_assets = store
        .delete_project(&project.project_id)
        .expect("project should delete")
        .expect("project should exist");

    assert_eq!(deleted_assets.len(), 1);
    assert!(store
        .list_projects()
        .expect("projects should reload")
        .iter()
        .all(|item| item.project_id != project.project_id));
    assert!(store
        .stored_asset_groups(&project.project_id)
        .expect_err("deleted project should not be queryable")
        .to_string()
        .contains("project not found"));
    assert_eq!(
        store
            .global_asset_summary()
            .expect("global asset summary should reload")
            .photo_count,
        1
    );
    assert_eq!(
        store
            .stored_asset_groups(&other_project.project_id)
            .expect("other project groups should remain")
            .len(),
        1
    );
}

#[test]
fn sqlite_store_renames_projects_without_changing_identity() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Untitled Shoot")
        .expect("project should create");

    let renamed = store
        .rename_project(&project.project_id, "Studio Product Shoot")
        .expect("project should rename");

    assert_eq!(renamed.project_id, project.project_id);
    assert_eq!(renamed.name, "Studio Product Shoot");
    assert_eq!(renamed.slug, "studio-product-shoot");
    assert_eq!(renamed.status, ProjectStatus::Active);
    assert_eq!(renamed.created_at_ms, project.created_at_ms);
    assert!(renamed.updated_at_ms >= project.updated_at_ms);
}

#[test]
fn sqlite_store_exposes_project_lifecycle_capabilities() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let active = store
        .create_project("Client Shoot")
        .expect("project should create");
    let archived = store
        .archive_project(&active.project_id)
        .expect("project should archive");

    assert_eq!(active.kind(), ProjectKind::User);
    assert_eq!(
        active.capabilities(),
        ProjectCapabilities {
            can_be_active_project: true,
            can_archive: true,
            can_rename: true,
            can_restore: false,
            can_accept_moved_groups: true,
        }
    );
    assert_eq!(
        archived.capabilities(),
        ProjectCapabilities {
            can_be_active_project: false,
            can_archive: false,
            can_rename: true,
            can_restore: true,
            can_accept_moved_groups: false,
        }
    );
}

#[test]
fn sqlite_store_rejects_transfer_without_existing_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");

    let result = store.record_transfer(
        "missing-project",
        completed_transfer("ftp:1", "IMG_0001.JPG", 10),
    );

    assert!(result.is_err());
    assert!(result
        .expect_err("error should exist")
        .to_string()
        .contains("project not found"));
}

#[test]
fn sqlite_store_indexes_assets_and_groups_by_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project_a = store
        .create_project("Wedding")
        .expect("project should create");
    let project_b = store
        .create_project("Street")
        .expect("project should create");

    store
        .record_transfer(
            &project_a.project_id,
            completed_transfer("ftp:jpg", "DCIM/100/IMG_2222.JPG", 20),
        )
        .expect("jpg transfer should record");
    store
        .record_transfer(
            &project_a.project_id,
            completed_transfer("ftp:raw", "DCIM/100/IMG_2222.NEF", 21),
        )
        .expect("raw transfer should record");
    store
        .record_transfer(
            &project_b.project_id,
            completed_transfer("ftp:other", "DCIM/100/IMG_2222.JPG", 22),
        )
        .expect("other project transfer should record");

    let page = store
        .asset_group_page(&project_a.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("groups should query");
    let group_id = page.groups[0]
        .group_id
        .as_deref()
        .expect("project group should expose stable id");
    let assets = store
        .assets_for_group(&project_a.project_id, group_id)
        .expect("group assets should query");

    assert_eq!(page.total_groups, 1);
    assert_eq!(page.groups[0].group_key, "IMG_2222");
    assert!(page.groups[0].jpeg.is_some());
    assert!(page.groups[0].raw.is_some());
    assert_eq!(page.summary.asset_count, 2);
    assert_eq!(assets.len(), 2);
    assert!(assets.iter().all(|asset| asset.group_id.is_some()));
    assert!(assets.iter().all(|asset| asset.media_kind == "photo"));
    assert!(assets
        .iter()
        .all(|asset| asset.original_parent_path.as_deref() == Some("DCIM/100")));

    let other_page = store
        .asset_group_page(&project_b.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("other project groups should query");
    assert_eq!(other_page.total_groups, 1);
    assert_eq!(other_page.summary.asset_count, 1);
}

#[test]
fn sqlite_store_summarizes_global_project_assets() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project_a = store
        .create_project("Wedding")
        .expect("project should create");
    let project_b = store
        .create_project("Street")
        .expect("project should create");

    store
        .record_transfer(
            &project_a.project_id,
            completed_transfer("ftp:jpg", "DCIM/100/IMG_2222.JPG", 20),
        )
        .expect("jpg transfer should record");
    store
        .record_transfer(
            &project_a.project_id,
            completed_transfer("ftp:raw", "DCIM/100/IMG_2222.NEF", 21),
        )
        .expect("raw transfer should record");
    store
        .record_transfer(
            &project_b.project_id,
            completed_transfer("ftp:other", "DCIM/100/IMG_3333.JPG", 22),
        )
        .expect("other project transfer should record");

    let summary = store
        .global_asset_summary()
        .expect("global summary should query");

    assert_eq!(summary.photo_count, 2);
    assert_eq!(summary.file_count, 3);
    assert_eq!(summary.storage_bytes, 300);
}

#[test]
fn sqlite_store_persists_user_marks_and_filters_asset_groups() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Client Selects")
        .expect("project should create");

    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:favorite", "DCIM/100/KEEP_0001.JPG", 20),
        )
        .expect("favorite transfer should record");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:marked", "DCIM/100/MARK_0001.JPG", 21),
        )
        .expect("marked transfer should record");

    let page = store
        .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("groups should query");
    let favorite_group_id = page.groups[0].group_id.clone().expect("group id");
    let marked_group_id = page.groups[1].group_id.clone().expect("group id");

    let favorite_marks = store
        .set_asset_group_user_marks(&project.project_id, &favorite_group_id, Some(true), None)
        .expect("favorite mark should save");
    let marked_marks = store
        .set_asset_group_user_marks(&project.project_id, &marked_group_id, None, Some(true))
        .expect("marked flag should save");

    assert_eq!(
        favorite_marks,
        AssetUserMarks {
            favorite: true,
            marked: false,
        }
    );
    assert_eq!(
        marked_marks,
        AssetUserMarks {
            favorite: false,
            marked: true,
        }
    );

    let favorites = store
        .asset_group_page(
            &project.project_id,
            AssetGroupQuery {
                favorite: Some(true),
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("favorite groups should query");
    let marked = store
        .asset_group_page(
            &project.project_id,
            AssetGroupQuery {
                marked: Some(true),
                ..AssetGroupQuery::default()
            },
            0,
            25,
        )
        .expect("marked groups should query");

    assert_eq!(favorites.total_groups, 1);
    assert_eq!(
        favorites.groups[0].group_id.as_deref(),
        Some(favorite_group_id.as_str())
    );
    assert!(favorites.groups[0].user_marks.favorite);
    assert!(!favorites.groups[0].user_marks.marked);
    assert_eq!(marked.total_groups, 1);
    assert_eq!(
        marked.groups[0].group_id.as_deref(),
        Some(marked_group_id.as_str())
    );
    assert!(!marked.groups[0].user_marks.favorite);
    assert!(marked.groups[0].user_marks.marked);
}

#[test]
fn sqlite_store_creates_lan_share_session_with_query() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("LAN Share")
        .expect("project should create");

    let session = store
        .create_lan_share_session(
            &project.project_id,
            AssetGroupQuery {
                collection: Some("favorites".to_string()),
                sort: AssetGroupSort::ModelScore,
                ..AssetGroupQuery::default()
            },
            Some("Client selects".to_string()),
            1_000,
        )
        .expect("share session should create");

    assert_eq!(session.project_id, project.project_id);
    assert_eq!(session.query.collection.as_deref(), Some("favorites"));
    assert_eq!(session.query.sort, AssetGroupSort::ModelScore);
    assert!(session.active);
    assert_eq!(session.title.as_deref(), Some("Client selects"));
    assert_eq!(session.token.len(), 32);
}

#[test]
fn sqlite_store_sets_and_clears_lan_share_guest_mark_without_user_marks() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Guest Marks")
        .expect("project should create");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:guest-mark", "DCIM/100/IMG_9001.JPG", 20),
        )
        .expect("transfer should record");
    let page = store
        .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("groups should query");
    let group_id = page.groups[0].group_id.clone().expect("group id");
    let session = store
        .create_lan_share_session(&project.project_id, AssetGroupQuery::default(), None, 1_000)
        .expect("share session should create");

    let mark = store
        .set_lan_share_guest_mark(
            &session.share_id,
            &project.project_id,
            &group_id,
            Some(GuestMark::Reject),
            2_000,
        )
        .expect("guest mark should save");
    assert_eq!(mark.unwrap().guest_mark, GuestMark::Reject);

    let page = store
        .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("groups should query");
    assert_eq!(page.groups[0].guest_mark, Some(GuestMark::Reject));
    assert!(!page.groups[0].user_marks.favorite);
    assert!(!page.groups[0].user_marks.marked);

    let cleared = store
        .set_lan_share_guest_mark(
            &session.share_id,
            &project.project_id,
            &group_id,
            None,
            3_000,
        )
        .expect("guest mark should clear");
    assert!(cleared.is_none());
}

#[test]
fn sqlite_store_deletes_asset_group_without_leaving_database_locked() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Delete Project")
        .expect("project should create");

    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:delete-jpg", "DCIM/100/IMG_7001.JPG", 1000),
        )
        .expect("jpg transfer should record");
    store
        .record_transfer(
            &project.project_id,
            completed_transfer("ftp:delete-raw", "DCIM/100/IMG_7001.NEF", 1001),
        )
        .expect("raw transfer should record");
    let group_id = store
        .stored_asset_groups(&project.project_id)
        .expect("groups should load")
        .into_iter()
        .find(|group| group.display_key == "IMG_7001")
        .expect("group should exist")
        .group_id;

    let deleted = store
        .delete_asset_group(&project.project_id, &group_id)
        .expect("delete should not leave sqlite locked")
        .expect("delete should find group");

    assert_eq!(deleted.len(), 2);
    assert!(store
        .stored_asset_groups(&project.project_id)
        .expect("groups should reload")
        .is_empty());
    assert!(store
        .transfer_records(&project.project_id)
        .expect("transfers should reload")
        .is_empty());
}

fn completed_transfer(
    transfer_id: &str,
    original_path: &str,
    completed_at_ms: i64,
) -> TransferRecord {
    let final_filename = original_path
        .rsplit('/')
        .next()
        .expect("filename should exist")
        .to_string();
    TransferRecord {
        transfer_id: transfer_id.to_string(),
        protocol: "ftp".to_string(),
        status: TransferStatus::Completed,
        original_path: original_path.to_string(),
        final_filename: final_filename.clone(),
        final_location: Some(StoredObjectLocation::local_path(final_filename)),
        size_bytes: 100,
        username: Some("z5".to_string()),
        remote_addr: Some("192.168.137.56".to_string()),
        source_name: Some("Studio Z5".to_string()),
        started_at_ms: completed_at_ms - 1,
        completed_at_ms: Some(completed_at_ms),
        error: None,
    }
}

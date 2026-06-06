# Desktop Workbench MVP Vertical Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a real desktop workbench MVP that creates a project, scans a local folder through formal desktop scan semantics, displays grouped assets, and wires evaluation and recommendation actions.

**Architecture:** Add source-aware project assets and first-class desktop scan tables to `camera_connector_core`, then expose them through a thin Tauri command gateway in `apps/desktop/src-tauri`. The frontend is a Vite TypeScript workbench that depends on gateway DTOs and never talks to SQLite or receiver/import tables directly.

**Tech Stack:** Rust 2021, rusqlite, existing `camera_connector_core`, Tauri 2, Vite, TypeScript, CSS, npm.

---

## File Structure

- Create `core/src/desktop_scan.rs`: desktop scan DTOs, enums, file discovery helpers, and stable ids for scan records.
- Modify `core/src/lib.rs`: export desktop scan types.
- Modify `core/src/model/received_asset.rs`: add optional source metadata fields to project asset DTOs.
- Modify `core/src/storage/mod.rs`: migrate `assets` to source-aware provenance, add desktop scan tables, persist scan runs and scanned assets, expose scan status queries.
- Modify `core/src/service.rs`: add desktop scan service methods and schedule analysis jobs after scan completion according to `ProjectEvaluationSettings`.
- Create `core/tests/desktop_scan_tests.rs`: core tests for formal scan provenance, grouping, scan status, missing/changed files, and settings-driven jobs.
- Modify root `Cargo.toml`: add `apps/desktop/src-tauri` workspace member and Tauri workspace dependencies.
- Create `apps/desktop/package.json`, `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/src/styles.css`: desktop frontend.
- Create `apps/desktop/src-tauri/Cargo.toml`, `apps/desktop/src-tauri/build.rs`, `apps/desktop/src-tauri/tauri.conf.json`, `apps/desktop/src-tauri/capabilities/default.json`, `apps/desktop/src-tauri/src/main.rs`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src-tauri/src/commands.rs`: Tauri shell and command gateway.

## Task 1: Source-Aware Asset Provenance

**Files:**
- Modify: `core/src/storage/mod.rs`
- Modify: `core/src/model/received_asset.rs`
- Modify: `core/src/lib.rs`
- Create: `core/tests/desktop_scan_tests.rs`

- [ ] **Step 1: Write the failing provenance test**

Add this initial test to `core/tests/desktop_scan_tests.rs`:

```rust
use camera_connector_core::{
    AssetGroupQuery, DesktopScannedAssetInput, DesktopSourceStatus, SqliteStore,
    StoredObjectLocation,
};

#[test]
fn desktop_scanned_asset_does_not_create_transfer_record() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Desktop Scan")
        .expect("project should create");
    let source = store
        .upsert_desktop_scan_source(&project.project_id, temp_dir.path(), 10_000)
        .expect("scan source should save");
    let scan = store
        .create_desktop_scan_run(&project.project_id, &source.scan_source_id, temp_dir.path(), 10_001)
        .expect("scan run should create");
    let photo = temp_dir.path().join("IMG_1001.JPG");
    std::fs::write(&photo, [1_u8, 2, 3, 4]).expect("sample should write");

    let result = store
        .record_desktop_scanned_assets(
            &scan.scan_id,
            &[DesktopScannedAssetInput {
                local_path: photo.clone(),
                original_filename: "IMG_1001.JPG".to_string(),
                relative_path: "IMG_1001.JPG".to_string(),
                normalized_stem: "IMG_1001".to_string(),
                size_bytes: 4,
                modified_at_ms: 10_002,
                capture_time_ms: None,
            }],
            10_003,
        )
        .expect("desktop asset should index");

    assert_eq!(result.assets_indexed, 1);
    assert_eq!(result.group_ids.len(), 1);
    assert_eq!(
        store
            .transfer_records(&project.project_id)
            .expect("transfer records should query")
            .len(),
        0
    );

    let page = store
        .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should query");
    assert_eq!(page.summary.asset_count, 1);
    assert_eq!(page.groups[0].group_key, "IMG_1001");
    assert_eq!(page.groups[0].primary.source_kind.as_deref(), Some("desktop_scan"));
    assert_eq!(
        page.groups[0].primary.source_status.as_deref(),
        Some(DesktopSourceStatus::Available.as_str())
    );
    assert_eq!(
        page.groups[0].primary.storage_location,
        Some(StoredObjectLocation::local_path(photo.clone()))
    );
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```powershell
cargo test -p camera_connector_core --test desktop_scan_tests desktop_scanned_asset_does_not_create_transfer_record
```

Expected: FAIL with unresolved imports for `DesktopScannedAssetInput`, `DesktopSourceStatus`, and missing `SqliteStore` methods.

- [ ] **Step 3: Add source fields to asset DTOs**

In `core/src/model/received_asset.rs`, extend `ReceivedAsset`:

```rust
pub struct ReceivedAsset {
    pub id: String,
    pub filename: String,
    pub size_bytes: u64,
    pub format: ObjectFormat,
    pub source: ImportSource,
    pub received_time_ms: Option<i64>,
    pub capture_time_ms: Option<i64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub group_key: Option<String>,
    pub storage_location: Option<StoredObjectLocation>,
    pub original_path: Option<String>,
    pub username: Option<String>,
    pub display_source: Option<String>,
    pub remote_addr: Option<String>,
    pub virtual_display_path: Option<String>,
    pub duplicate_index: Option<usize>,
    pub duplicate_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_record_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_modified_at_ms: Option<i64>,
}
```

Add these defaults to `ReceivedAsset::new`:

```rust
source_kind: None,
source_record_id: None,
source_status: None,
source_modified_at_ms: None,
```

- [ ] **Step 4: Add desktop scan source types**

Create `core/src/desktop_scan.rs`:

```rust
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopScanPhase {
    Queued,
    Scanning,
    Indexing,
    Completed,
    Failed,
    Cancelled,
}

impl DesktopScanPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Scanning => "scanning",
            Self::Indexing => "indexing",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "scanning" => Self::Scanning,
            "indexing" => Self::Indexing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Queued,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopSourceStatus {
    Available,
    Missing,
    Changed,
}

impl DesktopSourceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Missing => "missing",
            Self::Changed => "changed",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "missing" => Self::Missing,
            "changed" => Self::Changed,
            _ => Self::Available,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopScanSource {
    pub scan_source_id: String,
    pub project_id: String,
    pub root_path: PathBuf,
    pub root_label: String,
    pub status: DesktopSourceStatus,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopScanRun {
    pub scan_id: String,
    pub project_id: String,
    pub scan_source_id: String,
    pub root_path: PathBuf,
    pub phase: DesktopScanPhase,
    pub files_seen: usize,
    pub assets_indexed: usize,
    pub groups_updated: usize,
    pub started_at_ms: i64,
    pub updated_at_ms: i64,
    pub completed_at_ms: Option<i64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopScannedAssetInput {
    pub local_path: PathBuf,
    pub original_filename: String,
    pub relative_path: String,
    pub normalized_stem: String,
    pub size_bytes: u64,
    pub modified_at_ms: i64,
    pub capture_time_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopScanIndexResult {
    pub assets_indexed: usize,
    pub group_ids: Vec<String>,
}
```

Modify `core/src/lib.rs`:

```rust
pub mod desktop_scan;

pub use desktop_scan::{
    DesktopScanIndexResult, DesktopScanPhase, DesktopScanRun, DesktopScanSource,
    DesktopScannedAssetInput, DesktopSourceStatus,
};
```

- [ ] **Step 5: Make `assets` source-aware without creating transfer rows**

In `core/src/storage/mod.rs`, add these fields to `StoredAsset`:

```rust
pub transfer_id: Option<String>,
pub source_kind: String,
pub source_record_id: String,
pub source_status: String,
pub source_modified_at_ms: Option<i64>,
```

This replaces the current `pub transfer_id: String` field. Transfer-backed assets store `Some(transfer_id)`. Desktop scanned assets store `None`.

Update `initialize_schema` so `assets` has nullable `transfer_id` and explicit provenance:

```sql
CREATE TABLE IF NOT EXISTS assets (
    asset_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    group_id TEXT NOT NULL REFERENCES asset_groups(group_id),
    transfer_id TEXT UNIQUE REFERENCES transfers(transfer_id),
    source_kind TEXT NOT NULL DEFAULT 'transfer',
    source_record_id TEXT NOT NULL,
    source_status TEXT NOT NULL DEFAULT 'available',
    source_modified_at_ms INTEGER,
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
    duplicate_key TEXT,
    duplicate_index INTEGER,
    duplicate_count INTEGER,
    UNIQUE(project_id, source_kind, source_record_id)
);
```

Before `CREATE TABLE IF NOT EXISTS assets`, call a migration helper that rebuilds old `assets` tables:

```rust
migrate_assets_to_source_provenance(connection)?;
```

Implement the helper in `core/src/storage/mod.rs`:

```rust
fn migrate_assets_to_source_provenance(
    connection: &Connection,
) -> std::result::Result<(), rusqlite::Error> {
    let columns = table_columns(connection, "assets")?;
    if columns.is_empty() || columns.contains("source_kind") {
        return Ok(());
    }
    connection.execute_batch(
        "
        ALTER TABLE assets RENAME TO assets_legacy_source_migration;
        CREATE TABLE assets (
            asset_id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(project_id),
            group_id TEXT NOT NULL REFERENCES asset_groups(group_id),
            transfer_id TEXT UNIQUE REFERENCES transfers(transfer_id),
            source_kind TEXT NOT NULL DEFAULT 'transfer',
            source_record_id TEXT NOT NULL,
            source_status TEXT NOT NULL DEFAULT 'available',
            source_modified_at_ms INTEGER,
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
            duplicate_key TEXT,
            duplicate_index INTEGER,
            duplicate_count INTEGER,
            UNIQUE(project_id, source_kind, source_record_id)
        );
        INSERT INTO assets (
            asset_id, project_id, group_id, transfer_id, source_kind, source_record_id,
            source_status, source_modified_at_ms, group_role, media_kind, format,
            original_filename, final_filename, normalized_stem, original_path,
            original_parent_path, final_location_kind, final_location_payload, size_bytes,
            capture_at_ms, received_at_ms, published_at_ms, source_identity, username,
            remote_addr, duplicate_key, duplicate_index, duplicate_count
        )
        SELECT
            asset_id, project_id, group_id, transfer_id, 'transfer', transfer_id,
            'available', NULL, group_role, media_kind, format, original_filename,
            final_filename, normalized_stem, original_path, original_parent_path,
            final_location_kind, final_location_payload, size_bytes, capture_at_ms,
            received_at_ms, published_at_ms, source_identity, username, remote_addr,
            duplicate_key, duplicate_index, duplicate_count
        FROM assets_legacy_source_migration;
        DROP TABLE assets_legacy_source_migration;
        ",
    )?;
    Ok(())
}
```

Update every `SELECT` that reads `assets` to include `source_kind`, `source_record_id`, `source_status`, and `source_modified_at_ms` immediately after `transfer_id`. Update `stored_asset_from_row` index positions accordingly, and set the new `ReceivedAsset` fields in `received_asset_from_row`.

Update `received_asset_from_row` so desktop scanned assets do not require a transfer id:

```rust
let source = stored
    .transfer_id
    .as_deref()
    .map(import_source_from_transfer_id)
    .unwrap_or(ImportSource::ManualDrop);
let mut asset = ReceivedAsset::new(
    stored.asset_id,
    stored.final_filename,
    stored.size_bytes,
    source,
);
asset.source_kind = Some(stored.source_kind);
asset.source_record_id = Some(stored.source_record_id);
asset.source_status = Some(stored.source_status);
asset.source_modified_at_ms = stored.source_modified_at_ms;
```

Update `asset_transfer_ids_for_group` to ignore desktop scanned assets:

```sql
SELECT transfer_id
FROM assets
WHERE project_id = ?1 AND group_id = ?2 AND transfer_id IS NOT NULL
ORDER BY CASE group_role
         WHEN 'jpeg' THEN 0
         WHEN 'raw' THEN 1
         WHEN 'video' THEN 2
         ELSE 3
         END ASC,
         published_at_ms ASC,
         asset_id ASC
```

Update `insert_asset_for_transfer` to insert:

```rust
source_kind = "transfer";
source_record_id = record.transfer_id.clone();
source_status = DesktopSourceStatus::Available.as_str();
source_modified_at_ms = None::<i64>;
```

- [ ] **Step 6: Add desktop scan tables and storage methods**

Add these tables to `initialize_schema`:

```sql
CREATE TABLE IF NOT EXISTS desktop_scan_sources (
    scan_source_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    root_path TEXT NOT NULL,
    root_label TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    UNIQUE(project_id, root_path)
);

CREATE TABLE IF NOT EXISTS desktop_scan_runs (
    scan_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    scan_source_id TEXT NOT NULL REFERENCES desktop_scan_sources(scan_source_id),
    root_path TEXT NOT NULL,
    phase TEXT NOT NULL,
    files_seen INTEGER NOT NULL,
    assets_indexed INTEGER NOT NULL,
    groups_updated INTEGER NOT NULL,
    started_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    completed_at_ms INTEGER,
    error TEXT
);

CREATE TABLE IF NOT EXISTS desktop_scanned_assets (
    asset_id TEXT PRIMARY KEY REFERENCES assets(asset_id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    scan_source_id TEXT NOT NULL REFERENCES desktop_scan_sources(scan_source_id),
    scan_id TEXT NOT NULL REFERENCES desktop_scan_runs(scan_id),
    local_path TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    normalized_stem TEXT NOT NULL,
    object_format TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    modified_at_ms INTEGER NOT NULL,
    capture_time_ms INTEGER,
    source_status TEXT NOT NULL,
    indexed_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    UNIQUE(project_id, scan_source_id, local_path)
);
```

Add indexes:

```sql
CREATE INDEX IF NOT EXISTS idx_desktop_scan_runs_project ON desktop_scan_runs(project_id, updated_at_ms);
CREATE INDEX IF NOT EXISTS idx_desktop_scanned_assets_source ON desktop_scanned_assets(project_id, scan_source_id, source_status);
```

Add storage methods on `impl SqliteStore`:

```rust
pub fn upsert_desktop_scan_source(
    &self,
    project_id: &str,
    root_path: impl AsRef<Path>,
    now_ms: i64,
) -> Result<DesktopScanSource>;

pub fn create_desktop_scan_run(
    &self,
    project_id: &str,
    scan_source_id: &str,
    root_path: impl AsRef<Path>,
    now_ms: i64,
) -> Result<DesktopScanRun>;

pub fn update_desktop_scan_run(
    &self,
    scan_id: &str,
    phase: DesktopScanPhase,
    files_seen: usize,
    assets_indexed: usize,
    groups_updated: usize,
    error: Option<&str>,
    now_ms: i64,
) -> Result<DesktopScanRun>;

pub fn latest_desktop_scan_run(&self, project_id: &str) -> Result<Option<DesktopScanRun>>;

pub fn record_desktop_scanned_assets(
    &self,
    scan_id: &str,
    inputs: &[DesktopScannedAssetInput],
    now_ms: i64,
) -> Result<DesktopScanIndexResult>;
```

Use these deterministic ids:

```rust
fn desktop_scan_source_id(project_id: &str, root_path: &str) -> String {
    format!("desktop-source-{}", stable_key(&format!("{project_id}\t{root_path}")))
}

fn desktop_scan_id(project_id: &str, scan_source_id: &str, now_ms: i64) -> String {
    format!("desktop-scan-{}", stable_key(&format!("{project_id}\t{scan_source_id}\t{now_ms}")))
}

fn desktop_asset_id(project_id: &str, scan_source_id: &str, local_path: &str) -> String {
    format!("desktop-asset-{}", stable_key(&format!("{project_id}\t{scan_source_id}\t{local_path}")))
}
```

In `record_desktop_scanned_assets`, insert or update both `desktop_scanned_assets` and `assets`. Desktop asset rows must set:

```rust
transfer_id: None::<String>,
source_kind: "desktop_scan",
source_record_id: asset_id.clone(),
source_status: "available" | "changed",
source_modified_at_ms: Some(input.modified_at_ms),
final_location: StoredObjectLocation::local_path(input.local_path.clone()),
source_identity: source.root_label,
original_path: input.relative_path,
original_parent_path: original_parent_path(&input.relative_path),
received_at_ms: None::<i64>,
published_at_ms: Some(now_ms),
```

After all inserts, mark previously indexed assets for the same `scan_source_id` as `missing` when their `asset_id` was not seen in this scan. Refresh group rollups and duplicate info for all touched groups.

- [ ] **Step 7: Run the test and verify it passes**

Run:

```powershell
cargo test -p camera_connector_core --test desktop_scan_tests desktop_scanned_asset_does_not_create_transfer_record
```

Expected: PASS.

- [ ] **Step 8: Run existing asset and storage regression tests**

Run:

```powershell
cargo test -p camera_connector_core --test asset_query_tests
cargo test -p camera_connector_core --test storage_store_tests
```

Expected: PASS.

- [ ] **Step 9: Commit**

```powershell
git add core/src/desktop_scan.rs core/src/lib.rs core/src/model/received_asset.rs core/src/storage/mod.rs core/tests/desktop_scan_tests.rs
git commit -m "feat: add source-aware desktop scan storage"
```

## Task 2: Core Desktop Folder Scanner And Analysis Scheduling

**Files:**
- Modify: `core/src/desktop_scan.rs`
- Modify: `core/src/service.rs`
- Modify: `core/src/storage/mod.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/desktop_scan_tests.rs`

- [ ] **Step 1: Add failing tests for folder scan, grouping, status, and missing/changed state**

Append these tests to `core/tests/desktop_scan_tests.rs`:

```rust
use camera_connector_core::{CameraConnectorService, DesktopScanPhase};

#[test]
fn service_scans_folder_and_groups_raw_jpeg_video_by_stem() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("config.json");
    let root = temp_dir.path().join("photos");
    std::fs::create_dir_all(&root).expect("photo root should create");
    std::fs::write(root.join("IMG_2001.JPG"), [1_u8]).expect("jpeg should write");
    std::fs::write(root.join("IMG_2001.NEF"), [2_u8]).expect("raw should write");
    std::fs::write(root.join("IMG_2001.MOV"), [3_u8]).expect("video should write");

    let service = CameraConnectorService::new(Some(config_path));
    let project = service.create_project("Desktop Folder").expect("project should create");
    let scan = service
        .create_desktop_project_scan(&project.project_id, &root)
        .expect("scan should queue");

    let result = service
        .run_desktop_project_scan(&scan.scan_id)
        .expect("scan should complete");

    assert_eq!(result.scan.phase, DesktopScanPhase::Completed);
    assert_eq!(result.scan.files_seen, 3);
    assert_eq!(result.index.assets_indexed, 3);

    let page = service
        .project_asset_group_page_with_query(
            &project.project_id,
            AssetGroupQuery::default(),
            0,
            25,
        )
        .expect("assets should query");
    assert_eq!(page.total_groups, 1);
    assert!(page.groups[0].jpeg.is_some());
    assert!(page.groups[0].raw.is_some());
    assert!(page.groups[0].video.is_some());
}

#[test]
fn rescan_marks_missing_and_changed_without_deleting_group_marks() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("config.json");
    let root = temp_dir.path().join("photos");
    std::fs::create_dir_all(&root).expect("photo root should create");
    let first = root.join("IMG_3001.JPG");
    let second = root.join("IMG_3002.JPG");
    std::fs::write(&first, [1_u8]).expect("first should write");
    std::fs::write(&second, [2_u8]).expect("second should write");

    let service = CameraConnectorService::new(Some(config_path));
    let project = service.create_project("Rescan").expect("project should create");
    let scan = service
        .create_desktop_project_scan(&project.project_id, &root)
        .expect("scan should queue");
    service.run_desktop_project_scan(&scan.scan_id).expect("scan should run");
    let first_group = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page")
        .groups
        .into_iter()
        .find(|group| group.group_key == "IMG_3001")
        .expect("first group should exist")
        .group_id
        .expect("group id should exist");
    service
        .set_asset_group_user_marks(&project.project_id, &first_group, Some(true), Some(true))
        .expect("marks should save");

    std::fs::remove_file(&first).expect("first should remove");
    std::fs::write(&second, [2_u8, 3_u8]).expect("second should change");
    let rescan = service
        .create_desktop_project_scan(&project.project_id, &root)
        .expect("rescan should queue");
    service.run_desktop_project_scan(&rescan.scan_id).expect("rescan should run");

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should query");
    assert_eq!(page.total_groups, 2);
    let missing = page
        .groups
        .iter()
        .find(|group| group.group_key == "IMG_3001")
        .expect("missing group should remain");
    assert_eq!(missing.primary.source_status.as_deref(), Some("missing"));
    assert!(missing.user_marks.favorite);
    assert!(missing.user_marks.marked);
    let changed = page
        .groups
        .iter()
        .find(|group| group.group_key == "IMG_3002")
        .expect("changed group should remain");
    assert_eq!(changed.primary.source_status.as_deref(), Some("changed"));
}
```

- [ ] **Step 2: Run the tests and verify they fail**

Run:

```powershell
cargo test -p camera_connector_core --test desktop_scan_tests service_scans_folder_and_groups_raw_jpeg_video_by_stem
cargo test -p camera_connector_core --test desktop_scan_tests rescan_marks_missing_and_changed_without_deleting_group_marks
```

Expected: FAIL with missing `CameraConnectorService::create_desktop_project_scan` and `run_desktop_project_scan`.

- [ ] **Step 3: Add file discovery helpers**

In `core/src/desktop_scan.rs`, add:

```rust
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::ObjectFormat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopDiscoveredFile {
    pub local_path: PathBuf,
    pub original_filename: String,
    pub relative_path: String,
    pub normalized_stem: String,
    pub size_bytes: u64,
    pub modified_at_ms: i64,
}

pub fn discover_desktop_media_files(root: &Path) -> crate::Result<Vec<DesktopDiscoveredFile>> {
    let mut files = Vec::new();
    visit_media_dir(root, root, &mut files)?;
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

fn visit_media_dir(
    root: &Path,
    current: &Path,
    files: &mut Vec<DesktopDiscoveredFile>,
) -> crate::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            visit_media_dir(root, &path, files)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().into_owned();
        let format = ObjectFormat::from_filename(&filename);
        if !format.is_supported_media() {
            continue;
        }
        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let normalized_stem = filename
            .rsplit_once('.')
            .map(|(stem, _)| stem.to_ascii_uppercase())
            .filter(|stem| !stem.is_empty())
            .unwrap_or_else(|| filename.to_ascii_uppercase());
        let modified_at_ms = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis() as i64)
            .unwrap_or_default();
        files.push(DesktopDiscoveredFile {
            local_path: path,
            original_filename: filename,
            relative_path,
            normalized_stem,
            size_bytes: metadata.len(),
            modified_at_ms,
        });
    }
    Ok(())
}
```

- [ ] **Step 4: Add service scan methods**

In `core/src/service.rs`, import desktop scan helpers:

```rust
use crate::{
    discover_desktop_media_files, DesktopScanIndexResult, DesktopScanPhase, DesktopScanRun,
    DesktopScannedAssetInput,
};
```

Add result DTO:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopProjectScanResult {
    pub scan: DesktopScanRun,
    pub index: DesktopScanIndexResult,
}
```

Add methods on `impl CameraConnectorService`:

```rust
pub fn create_desktop_project_scan(
    &self,
    project_id: &str,
    root_path: impl AsRef<Path>,
) -> Result<DesktopScanRun> {
    let now = current_time_ms();
    let store = self.storage_store()?;
    let source = store.upsert_desktop_scan_source(project_id, root_path.as_ref(), now)?;
    store.create_desktop_scan_run(project_id, &source.scan_source_id, root_path, now)
}

pub fn latest_desktop_project_scan(&self, project_id: &str) -> Result<Option<DesktopScanRun>> {
    self.storage_store()?.latest_desktop_scan_run(project_id)
}

pub fn run_desktop_project_scan(&self, scan_id: &str) -> Result<DesktopProjectScanResult> {
    let store = self.storage_store()?;
    let scan = store.desktop_scan_run(scan_id)?.ok_or_else(|| {
        crate::ImporterError::internal(format!("desktop scan not found: {scan_id}"))
    })?;
    let started = current_time_ms();
    store.update_desktop_scan_run(
        scan_id,
        DesktopScanPhase::Scanning,
        0,
        0,
        0,
        None,
        started,
    )?;
    let discovered = match discover_desktop_media_files(&scan.root_path) {
        Ok(files) => files,
        Err(error) => {
            let now = current_time_ms();
            store.update_desktop_scan_run(
                scan_id,
                DesktopScanPhase::Failed,
                0,
                0,
                0,
                Some(&error.to_string()),
                now,
            )?;
            return Err(error);
        }
    };
    store.update_desktop_scan_run(
        scan_id,
        DesktopScanPhase::Indexing,
        discovered.len(),
        0,
        0,
        None,
        current_time_ms(),
    )?;
    let inputs = discovered
        .into_iter()
        .map(|file| DesktopScannedAssetInput {
            local_path: file.local_path.clone(),
            original_filename: file.original_filename,
            relative_path: file.relative_path,
            normalized_stem: file.normalized_stem,
            size_bytes: file.size_bytes,
            modified_at_ms: file.modified_at_ms,
            capture_time_ms: crate::media_metadata::extract_capture_time_ms(&file.local_path),
        })
        .collect::<Vec<_>>();
    let index = store.record_desktop_scanned_assets(scan_id, &inputs, current_time_ms())?;
    self.enqueue_desktop_scan_analysis_jobs(&scan.project_id, &index.group_ids)?;
    let completed = store.update_desktop_scan_run(
        scan_id,
        DesktopScanPhase::Completed,
        inputs.len(),
        index.assets_indexed,
        index.group_ids.len(),
        None,
        current_time_ms(),
    )?;
    Ok(DesktopProjectScanResult {
        scan: completed,
        index,
    })
}
```

- [ ] **Step 5: Export service scan result and drain summary**

Modify the `pub use service::{ ... }` block in `core/src/lib.rs` so it includes the new `DesktopProjectScanResult` and the existing `AnalysisDrainSummary`:

```rust
pub use service::{
    AccountView, AnalysisDrainSummary, AssetFacetCount, AssetGroupModelEvaluationInput,
    AssetGroupPage, AssetGroupQuery, AssetGroupSort, AssetGroupSummary, CameraConnectorDashboard,
    CameraConnectorService, ConnectedDeviceView, DesktopProjectScanResult,
    PublishQueueFailureView, ReceiverConfigRequest, ReceiverSettingsUpdate, SystemPathsView,
    TransferQuery, TransferRecordView, TransferSummary,
};
```

- [ ] **Step 6: Add scan completion analysis scheduling**

Add this private method in `core/src/service.rs`:

```rust
fn enqueue_desktop_scan_analysis_jobs(
    &self,
    project_id: &str,
    group_ids: &[String],
) -> Result<usize> {
    let store = self.storage_store()?;
    let settings = store
        .project_evaluation_settings(project_id)?
        .unwrap_or_else(|| ProjectEvaluationSettings::default_for_project(project_id, current_time_ms()));
    let mut enqueued = 0;
    let provider_configured = self.provider_configured_for_model_work()?;
    for group_id in group_ids {
        let mut technical = NewAnalysisJob::new(
            project_id,
            AnalysisJobType::AssessAssetGroupTechnicalQuality,
            AnalysisEntityType::AssetGroup,
            group_id,
            &format!("desktop-scan-technical:{project_id}:{group_id}:technical-v1"),
        );
        technical.priority = 20;
        store.enqueue_analysis_job(technical)?;
        enqueued += 1;
        if settings.model_evaluation_enabled
            && settings.auto_evaluate_on_upload
            && provider_configured
        {
            let mut model = NewAnalysisJob::new(
                project_id,
                AnalysisJobType::EvaluateAssetGroupWithModel,
                AnalysisEntityType::AssetGroup,
                group_id,
                &format!("desktop-scan-model:{project_id}:{group_id}"),
            );
            model.priority = 30;
            store.enqueue_analysis_job(model)?;
            enqueued += 1;
        }
    }
    Ok(enqueued)
}
```

Keep project-scope recommendation out of this method.

- [ ] **Step 7: Run desktop scan tests**

Run:

```powershell
cargo test -p camera_connector_core --test desktop_scan_tests
```

Expected: PASS.

- [ ] **Step 8: Run analysis job regressions**

Run:

```powershell
cargo test -p camera_connector_core --test analysis_job_tests
```

Expected: PASS, including existing project-recommendation automatic-drain tests.

- [ ] **Step 9: Commit**

```powershell
git add core/src/desktop_scan.rs core/src/service.rs core/src/storage/mod.rs core/src/lib.rs core/tests/desktop_scan_tests.rs
git commit -m "feat: scan desktop folders into project assets"
```

## Task 3: Desktop Command Gateway

**Files:**
- Modify: `Cargo.toml`
- Create: `apps/desktop/src-tauri/Cargo.toml`
- Create: `apps/desktop/src-tauri/build.rs`
- Create: `apps/desktop/src-tauri/tauri.conf.json`
- Create: `apps/desktop/src-tauri/capabilities/default.json`
- Create: `apps/desktop/src-tauri/src/main.rs`
- Create: `apps/desktop/src-tauri/src/lib.rs`
- Create: `apps/desktop/src-tauri/src/commands.rs`

- [ ] **Step 1: Add workspace member and dependencies**

Modify root `Cargo.toml`:

```toml
[workspace]
members = [
    "core",
    "core-ffi",
    "tools/cli",
    "apps/desktop/src-tauri",
]
resolver = "2"

[workspace.dependencies]
tauri = { version = "2", features = [] }
tauri-build = { version = "2", features = [] }
tauri-plugin-dialog = "2"
```

- [ ] **Step 2: Create Tauri Cargo manifest**

Create `apps/desktop/src-tauri/Cargo.toml`:

```toml
[package]
name = "camera-connector-desktop"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[lib]
name = "camera_connector_desktop"
crate-type = ["staticlib", "cdylib", "rlib"]

[[bin]]
name = "camera-connector-desktop"
path = "src/main.rs"

[build-dependencies]
tauri-build.workspace = true

[dependencies]
camera_connector_core = { path = "../../../core" }
serde.workspace = true
serde_json.workspace = true
tauri.workspace = true
tauri-plugin-dialog.workspace = true
```

Create `apps/desktop/src-tauri/build.rs`:

```rust
fn main() {
    tauri_build::build();
}
```

- [ ] **Step 3: Add Tauri config and capabilities**

Create `apps/desktop/src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Camera Connector",
  "version": "0.1.0",
  "identifier": "com.cameraconnector.desktop",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "Camera Connector",
        "width": 1280,
        "height": 820,
        "minWidth": 980,
        "minHeight": 640
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": false,
    "targets": "all"
  }
}
```

Create `apps/desktop/src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default desktop permissions for Camera Connector MVP",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default"
  ]
}
```

- [ ] **Step 4: Add command DTOs and commands**

Create `apps/desktop/src-tauri/src/commands.rs`:

```rust
use std::path::PathBuf;
use std::sync::Mutex;

use camera_connector_core::{
    AnalysisDrainSummary, AssetGroupPage, AssetGroupQuery, CameraConnectorDashboard,
    CameraConnectorService, DesktopScanRun, Project, SelectionRecommendation, StoredAsset,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

pub struct DesktopState {
    pub service: CameraConnectorService,
    pub scan_lock: Mutex<()>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetPageRequest {
    pub project_id: String,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserMarksRequest {
    pub project_id: String,
    pub group_id: String,
    pub favorite: Option<bool>,
    pub marked: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BurstRecommendationRequest {
    pub burst_group_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopError {
    pub code: String,
    pub message: String,
}

fn desktop_error(error: camera_connector_core::ImporterError) -> DesktopError {
    DesktopError {
        code: error.code().to_string(),
        message: error.to_string(),
    }
}

#[tauri::command]
pub fn create_project(state: State<'_, DesktopState>, name: String) -> Result<Project, DesktopError> {
    state.service.create_project(name).map_err(desktop_error)
}

#[tauri::command]
pub fn list_projects(state: State<'_, DesktopState>) -> Result<Vec<Project>, DesktopError> {
    state.service.list_projects().map_err(desktop_error)
}

#[tauri::command]
pub fn select_project(state: State<'_, DesktopState>, project_id: String) -> Result<(), DesktopError> {
    state.service.set_active_project(&project_id).map_err(desktop_error)
}

#[tauri::command]
pub fn start_project_scan(
    app: AppHandle,
    state: State<'_, DesktopState>,
    project_id: String,
    root_path: String,
) -> Result<DesktopScanRun, DesktopError> {
    let scan = state
        .service
        .create_desktop_project_scan(&project_id, PathBuf::from(root_path))
        .map_err(desktop_error)?;
    let service = state.service.clone();
    let scan_id = scan.scan_id.clone();
    tauri::async_runtime::spawn(async move {
        let result = service.run_desktop_project_scan(&scan_id);
        let _ = app.emit("desktop-scan-finished", result.is_ok());
    });
    Ok(scan)
}

#[tauri::command]
pub fn get_scan_status(
    state: State<'_, DesktopState>,
    project_id: String,
) -> Result<Option<DesktopScanRun>, DesktopError> {
    state.service.latest_desktop_project_scan(&project_id).map_err(desktop_error)
}

#[tauri::command]
pub fn get_project_asset_page(
    state: State<'_, DesktopState>,
    request: AssetPageRequest,
) -> Result<AssetGroupPage, DesktopError> {
    state
        .service
        .project_asset_group_page_with_query(
            &request.project_id,
            AssetGroupQuery::default(),
            request.offset,
            request.limit,
        )
        .map_err(desktop_error)
}

#[tauri::command]
pub fn get_project_group_detail(
    state: State<'_, DesktopState>,
    project_id: String,
    group_id: String,
) -> Result<Vec<StoredAsset>, DesktopError> {
    state.service.project_group_assets(&project_id, &group_id).map_err(desktop_error)
}

#[tauri::command]
pub fn save_group_user_marks(
    state: State<'_, DesktopState>,
    request: UserMarksRequest,
) -> Result<camera_connector_core::AssetUserMarks, DesktopError> {
    state
        .service
        .set_asset_group_user_marks(
            &request.project_id,
            &request.group_id,
            request.favorite,
            request.marked,
        )
        .map_err(desktop_error)
}

#[tauri::command]
pub fn drain_analysis_jobs(state: State<'_, DesktopState>, limit: usize) -> Result<AnalysisDrainSummary, DesktopError> {
    state.service.drain_analysis_jobs(limit).map_err(desktop_error)
}

#[tauri::command]
pub fn recommend_burst_group(
    state: State<'_, DesktopState>,
    request: BurstRecommendationRequest,
) -> Result<SelectionRecommendation, DesktopError> {
    state
        .service
        .recommend_burst_group_from_model(&request.burst_group_id)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn generate_project_recommendation(
    state: State<'_, DesktopState>,
    project_id: String,
) -> Result<SelectionRecommendation, DesktopError> {
    state.service.generate_project_recommendation(&project_id, current_time_ms()).map_err(desktop_error)
}

#[tauri::command]
pub fn get_project_dashboard(
    state: State<'_, DesktopState>,
    project_id: String,
) -> Result<CameraConnectorDashboard, DesktopError> {
    state
        .service
        .project_dashboard(&project_id, AssetGroupQuery::default(), 0, 50, false)
        .map_err(desktop_error)
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}
```

- [ ] **Step 5: Wire the app entrypoint**

Create `apps/desktop/src-tauri/src/lib.rs`:

```rust
mod commands;

use std::sync::Mutex;

use camera_connector_core::CameraConnectorService;
use commands::DesktopState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(DesktopState {
            service: CameraConnectorService::new(None),
            scan_lock: Mutex::new(()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::select_project,
            commands::start_project_scan,
            commands::get_scan_status,
            commands::get_project_asset_page,
            commands::get_project_group_detail,
            commands::save_group_user_marks,
            commands::drain_analysis_jobs,
            commands::recommend_burst_group,
            commands::generate_project_recommendation,
            commands::get_project_dashboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running camera connector desktop");
}
```

Create `apps/desktop/src-tauri/src/main.rs`:

```rust
fn main() {
    camera_connector_desktop::run();
}
```

- [ ] **Step 6: Check the desktop backend**

Run:

```powershell
cargo check -p camera-connector-desktop
```

Expected: PASS.

- [ ] **Step 7: Commit**

```powershell
git add Cargo.toml apps/desktop/src-tauri
git commit -m "feat: add desktop tauri command gateway"
```

## Task 4: Desktop Frontend Workbench

**Files:**
- Create: `apps/desktop/package.json`
- Create: `apps/desktop/tsconfig.json`
- Create: `apps/desktop/index.html`
- Create: `apps/desktop/src/main.ts`
- Create: `apps/desktop/src/styles.css`

- [ ] **Step 1: Create frontend package files**

Create `apps/desktop/package.json`:

```json
{
  "name": "camera-connector-desktop",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite --host 127.0.0.1 --port 1420",
    "build": "tsc && vite build",
    "preview": "vite preview --host 127.0.0.1 --port 1420",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0"
  }
}
```

Create `apps/desktop/tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "Bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "strict": true
  },
  "include": ["src"]
}
```

Create `apps/desktop/index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Camera Connector</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 2: Add typed command client and state model**

Create `apps/desktop/src/main.ts` with these imports, types, and command wrappers:

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import "./styles.css";

type Project = {
  project_id: string;
  name: string;
  slug: string;
  status: string;
};

type ReceivedAsset = {
  id: string;
  filename: string;
  size_bytes: number;
  format: string;
  group_key?: string;
  storage_location?: { kind: string; value: string };
  original_path?: string;
  source_kind?: string;
  source_record_id?: string;
  source_status?: string;
  model_status?: string;
};

type AssetGroup = {
  group_id?: string;
  group_key: string;
  primary: ReceivedAsset;
  jpeg?: ReceivedAsset;
  raw?: ReceivedAsset;
  video?: ReceivedAsset;
  burst?: {
    burst_group_id: string;
    member_count: number;
    recommendation_status: string;
    best_asset_group_id?: string;
    best_score?: number;
  };
  technical_status?: string;
  technical_gate_status?: string;
  model_status?: string;
  model_score?: number;
  model_tier?: string;
  model_summary?: string;
  is_favorite: boolean;
  is_flagged: boolean;
};

type AssetGroupPage = {
  groups: AssetGroup[];
  total_groups: number;
  has_more: boolean;
};

type DesktopScanRun = {
  scan_id: string;
  project_id: string;
  root_path: string;
  phase: string;
  files_seen: number;
  assets_indexed: number;
  groups_updated: number;
  error?: string;
};

type AppState = {
  projects: Project[];
  selectedProjectId: string | null;
  selectedGroupId: string | null;
  assetPage: AssetGroupPage | null;
  scan: DesktopScanRun | null;
  busy: boolean;
  error: string | null;
};

const state: AppState = {
  projects: [],
  selectedProjectId: null,
  selectedGroupId: null,
  assetPage: null,
  scan: null,
  busy: false,
  error: null,
};

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) throw new Error("missing app root");

async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (error) {
    const message = typeof error === "object" && error && "message" in error
      ? String((error as { message: unknown }).message)
      : String(error);
    throw new Error(message);
  }
}
```

- [ ] **Step 3: Add project and scan actions**

Append to `apps/desktop/src/main.ts`:

```ts
async function loadProjects() {
  state.projects = await command<Project[]>("list_projects");
  if (!state.selectedProjectId && state.projects.length > 0) {
    state.selectedProjectId = state.projects[0].project_id;
  }
  await refreshSelectedProject();
}

async function createProject() {
  const name = window.prompt("Project name", "Desktop Shoot");
  if (!name?.trim()) return;
  const project = await command<Project>("create_project", { name: name.trim() });
  state.projects = [project, ...state.projects.filter((item) => item.project_id !== project.project_id)];
  state.selectedProjectId = project.project_id;
  await command<void>("select_project", { projectId: project.project_id });
  await refreshSelectedProject();
}

async function selectProject(projectId: string) {
  state.selectedProjectId = projectId;
  state.selectedGroupId = null;
  await command<void>("select_project", { projectId });
  await refreshSelectedProject();
}

async function chooseAndScanFolder() {
  if (!state.selectedProjectId) return;
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected !== "string") return;
  state.scan = await command<DesktopScanRun>("start_project_scan", {
    projectId: state.selectedProjectId,
    rootPath: selected,
  });
  render();
  pollScanUntilDone();
}

async function pollScanUntilDone() {
  if (!state.selectedProjectId) return;
  const poll = async () => {
    if (!state.selectedProjectId) return;
    state.scan = await command<DesktopScanRun | null>("get_scan_status", {
      projectId: state.selectedProjectId,
    });
    await refreshAssets();
    render();
    if (state.scan && !["completed", "failed", "cancelled"].includes(state.scan.phase)) {
      window.setTimeout(poll, 1000);
    }
  };
  await poll();
}

async function refreshSelectedProject() {
  await refreshScan();
  await refreshAssets();
  render();
}

async function refreshScan() {
  if (!state.selectedProjectId) {
    state.scan = null;
    return;
  }
  state.scan = await command<DesktopScanRun | null>("get_scan_status", {
    projectId: state.selectedProjectId,
  });
}

async function refreshAssets() {
  if (!state.selectedProjectId) {
    state.assetPage = null;
    return;
  }
  state.assetPage = await command<AssetGroupPage>("get_project_asset_page", {
    request: {
      project_id: state.selectedProjectId,
      offset: 0,
      limit: 100,
    },
  });
}
```

- [ ] **Step 4: Add analysis and recommendation actions**

Append to `apps/desktop/src/main.ts`:

```ts
async function drainAnalysis() {
  await command("drain_analysis_jobs", { limit: 25 });
  await refreshSelectedProject();
}

async function recommendSelectedBurst() {
  const group = selectedGroup();
  const burstId = group?.burst?.burst_group_id;
  if (!burstId) return;
  await command("recommend_burst_group", {
    request: { burst_group_id: burstId },
  });
  await refreshSelectedProject();
}

async function generateProjectRecommendation() {
  if (!state.selectedProjectId) return;
  await command("generate_project_recommendation", {
    projectId: state.selectedProjectId,
  });
  await refreshSelectedProject();
}

async function toggleFavorite(group: AssetGroup) {
  if (!state.selectedProjectId || !group.group_id) return;
  await command("save_group_user_marks", {
    request: {
      project_id: state.selectedProjectId,
      group_id: group.group_id,
      favorite: !group.is_favorite,
      marked: null,
    },
  });
  await refreshAssets();
  render();
}

function selectedGroup(): AssetGroup | null {
  return state.assetPage?.groups.find((group) => group.group_id === state.selectedGroupId) ?? null;
}
```

- [ ] **Step 5: Add rendering**

Append to `apps/desktop/src/main.ts`:

```ts
function render() {
  const selectedProject = state.projects.find((project) => project.project_id === state.selectedProjectId);
  const groups = state.assetPage?.groups ?? [];
  const selected = selectedGroup() ?? groups[0] ?? null;
  if (selected && !state.selectedGroupId) state.selectedGroupId = selected.group_id ?? null;

  app.innerHTML = `
    <main class="shell">
      <aside class="sidebar">
        <div class="brand">
          <span class="brand-mark">CC</span>
          <div>
            <strong>Camera Connector</strong>
            <small>Desktop Workbench</small>
          </div>
        </div>
        <button class="primary wide" data-action="create-project">New Project</button>
        <nav class="project-list">
          ${state.projects.map((project) => `
            <button class="project ${project.project_id === state.selectedProjectId ? "selected" : ""}" data-project="${project.project_id}">
              <span>${escapeHtml(project.name)}</span>
              <small>${escapeHtml(project.slug)}</small>
            </button>
          `).join("")}
        </nav>
      </aside>
      <section class="workbench">
        <header class="topbar">
          <div>
            <h1>${escapeHtml(selectedProject?.name ?? "No project")}</h1>
            <p>${scanLine()}</p>
          </div>
          <div class="actions">
            <button data-action="scan" ${selectedProject ? "" : "disabled"}>Choose Folder</button>
            <button data-action="drain" ${selectedProject ? "" : "disabled"}>Run Jobs</button>
            <button data-action="project-recommend" ${selectedProject ? "" : "disabled"}>Project Recommend</button>
          </div>
        </header>
        <section class="content">
          <div class="grid-panel">
            <div class="grid-header">
              <strong>${state.assetPage?.total_groups ?? 0} groups</strong>
              <span>${state.error ? escapeHtml(state.error) : ""}</span>
            </div>
            <div class="asset-grid">
              ${groups.map((group) => assetCard(group)).join("")}
            </div>
          </div>
          <aside class="detail">
            ${selected ? detailPanel(selected) : `<div class="empty">Select a folder to scan photos.</div>`}
          </aside>
        </section>
      </section>
    </main>
  `;

  bindEvents();
}

function assetCard(group: AssetGroup): string {
  const selected = group.group_id === state.selectedGroupId;
  return `
    <button class="asset-card ${selected ? "selected" : ""}" data-group="${group.group_id ?? ""}">
      <div class="thumb">${formatBadge(group)}</div>
      <strong>${escapeHtml(group.group_key)}</strong>
      <span>${escapeHtml(group.primary.filename)}</span>
      <div class="badges">
        ${group.primary.source_status ? `<em>${escapeHtml(group.primary.source_status)}</em>` : ""}
        ${group.technical_gate_status ? `<em>${escapeHtml(group.technical_gate_status)}</em>` : ""}
        ${typeof group.model_score === "number" ? `<em>${group.model_score}</em>` : ""}
        ${group.burst?.recommendation_status ? `<em>${escapeHtml(group.burst.recommendation_status)}</em>` : ""}
        ${group.is_favorite ? `<em>favorite</em>` : ""}
      </div>
    </button>
  `;
}

function detailPanel(group: AssetGroup): string {
  return `
    <div class="detail-head">
      <div>
        <h2>${escapeHtml(group.group_key)}</h2>
        <p>${escapeHtml(group.primary.original_path ?? group.primary.filename)}</p>
      </div>
      <button data-action="favorite">${group.is_favorite ? "Unfavorite" : "Favorite"}</button>
    </div>
    <dl>
      <dt>Source</dt><dd>${escapeHtml(group.primary.source_kind ?? "-")} / ${escapeHtml(group.primary.source_status ?? "-")}</dd>
      <dt>Technical</dt><dd>${escapeHtml(group.technical_gate_status ?? group.technical_status ?? "-")}</dd>
      <dt>Model</dt><dd>${typeof group.model_score === "number" ? `${group.model_score} ${escapeHtml(group.model_tier ?? "")}` : "-"}</dd>
      <dt>Recommendation</dt><dd>${escapeHtml(group.burst?.recommendation_status ?? "-")}</dd>
      <dt>Summary</dt><dd>${escapeHtml(group.model_summary ?? "-")}</dd>
    </dl>
    <button class="wide" data-action="burst-recommend" ${group.burst ? "" : "disabled"}>Recommend Burst</button>
  `;
}

function bindEvents() {
  app.querySelector<HTMLButtonElement>("[data-action='create-project']")?.addEventListener("click", wrap(createProject));
  app.querySelector<HTMLButtonElement>("[data-action='scan']")?.addEventListener("click", wrap(chooseAndScanFolder));
  app.querySelector<HTMLButtonElement>("[data-action='drain']")?.addEventListener("click", wrap(drainAnalysis));
  app.querySelector<HTMLButtonElement>("[data-action='project-recommend']")?.addEventListener("click", wrap(generateProjectRecommendation));
  app.querySelector<HTMLButtonElement>("[data-action='burst-recommend']")?.addEventListener("click", wrap(recommendSelectedBurst));
  app.querySelector<HTMLButtonElement>("[data-action='favorite']")?.addEventListener("click", wrap(async () => {
    const group = selectedGroup();
    if (group) await toggleFavorite(group);
  }));
  app.querySelectorAll<HTMLButtonElement>("[data-project]").forEach((button) => {
    button.addEventListener("click", wrap(() => selectProject(button.dataset.project ?? "")));
  });
  app.querySelectorAll<HTMLButtonElement>("[data-group]").forEach((button) => {
    button.addEventListener("click", () => {
      state.selectedGroupId = button.dataset.group ?? null;
      render();
    });
  });
}

function wrap(fn: () => Promise<void>) {
  return async () => {
    state.error = null;
    try {
      await fn();
    } catch (error) {
      state.error = error instanceof Error ? error.message : String(error);
      render();
    }
  };
}

function scanLine(): string {
  if (!state.scan) return "No scan has run for this project.";
  return `${state.scan.phase} / ${state.scan.files_seen} seen / ${state.scan.assets_indexed} indexed / ${state.scan.groups_updated} groups${state.scan.error ? ` / ${state.scan.error}` : ""}`;
}

function formatBadge(group: AssetGroup): string {
  const badges = [
    group.jpeg ? "JPG" : "",
    group.raw ? "RAW" : "",
    group.video ? "VID" : "",
  ].filter(Boolean);
  return badges.join(" + ") || group.primary.format.toUpperCase();
}

function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, (char) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    "\"": "&quot;",
    "'": "&#039;",
  }[char] ?? char));
}

listen("desktop-scan-finished", () => {
  void refreshSelectedProject();
});

void loadProjects();
```

- [ ] **Step 6: Add styles**

Create `apps/desktop/src/styles.css`:

```css
:root {
  color: #1f2933;
  background: #eef1f4;
  font: 14px/1.4 Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

* { box-sizing: border-box; }
body { margin: 0; min-width: 980px; min-height: 640px; }
button { font: inherit; }

.shell {
  display: grid;
  grid-template-columns: 260px minmax(0, 1fr);
  min-height: 100vh;
}

.sidebar {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 18px;
  border-right: 1px solid #d7dde3;
  background: #f8fafb;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-mark {
  display: grid;
  place-items: center;
  width: 34px;
  height: 34px;
  border-radius: 6px;
  color: white;
  background: #245b75;
  font-weight: 700;
}

.brand small,
.project small,
.topbar p,
.asset-card span,
.detail p {
  color: #6b7785;
}

.primary,
button {
  border: 1px solid #c6d0d8;
  border-radius: 6px;
  background: #ffffff;
  color: #1f2933;
  padding: 8px 10px;
  cursor: pointer;
}

button:disabled {
  color: #9aa5b1;
  cursor: default;
  background: #edf1f4;
}

.primary {
  border-color: #245b75;
  background: #245b75;
  color: white;
}

.wide { width: 100%; }

.project-list {
  display: grid;
  gap: 8px;
}

.project {
  display: grid;
  gap: 2px;
  width: 100%;
  text-align: left;
}

.project.selected,
.asset-card.selected {
  border-color: #245b75;
  box-shadow: 0 0 0 1px #245b75 inset;
}

.workbench {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  min-width: 0;
}

.topbar {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  padding: 18px 22px;
  border-bottom: 1px solid #d7dde3;
  background: #ffffff;
}

.topbar h1 {
  margin: 0 0 4px;
  font-size: 22px;
}

.topbar p { margin: 0; }

.actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.content {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 340px;
  min-height: 0;
}

.grid-panel {
  min-width: 0;
  padding: 18px;
  overflow: auto;
}

.grid-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 12px;
}

.asset-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 10px;
}

.asset-card {
  display: grid;
  gap: 8px;
  min-height: 150px;
  text-align: left;
  align-content: start;
}

.thumb {
  display: grid;
  place-items: center;
  height: 74px;
  border-radius: 5px;
  background: #dde5eb;
  color: #245b75;
  font-weight: 700;
}

.badges {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.badges em {
  border-radius: 999px;
  background: #e8edf1;
  color: #43515f;
  padding: 2px 6px;
  font-style: normal;
  font-size: 12px;
}

.detail {
  padding: 18px;
  border-left: 1px solid #d7dde3;
  background: #ffffff;
  overflow: auto;
}

.detail-head {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: 12px;
}

.detail h2 {
  margin: 0 0 4px;
  font-size: 20px;
}

.detail p {
  margin: 0;
  overflow-wrap: anywhere;
}

dl {
  display: grid;
  grid-template-columns: 110px minmax(0, 1fr);
  gap: 10px 12px;
  margin: 18px 0;
}

dt {
  color: #6b7785;
}

dd {
  margin: 0;
  overflow-wrap: anywhere;
}

.empty {
  display: grid;
  place-items: center;
  min-height: 260px;
  color: #6b7785;
}
```

- [ ] **Step 7: Build the frontend**

Run:

```powershell
npm install --prefix apps/desktop
npm run build --prefix apps/desktop
```

Expected: PASS.

- [ ] **Step 8: Run the desktop app**

Run:

```powershell
npm run tauri --prefix apps/desktop -- dev
```

Expected: A Tauri window opens with project sidebar, scan controls, asset grid, and detail pane.

- [ ] **Step 9: Commit**

```powershell
git add apps/desktop/package.json apps/desktop/package-lock.json apps/desktop/tsconfig.json apps/desktop/index.html apps/desktop/src
git commit -m "feat: add desktop workbench ui"
```

## Task 5: End-To-End MVP Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-06-07-desktop-workbench-mvp-vertical-slice-implementation.md`

- [ ] **Step 1: Run full Rust checks**

Run:

```powershell
cargo test -p camera_connector_core
cargo test -p camera-connector-ffi
cargo test -p camera-connector-cli
cargo check -p camera-connector-desktop
```

Expected: PASS.

- [ ] **Step 2: Run frontend build**

Run:

```powershell
npm run build --prefix apps/desktop
```

Expected: PASS.

- [ ] **Step 3: Manual scan verification**

Create a sample folder:

```powershell
$sample = "$env:TEMP\\camera-connector-desktop-sample"
New-Item -ItemType Directory -Force -Path $sample
Set-Content -Path "$sample\\IMG_9001.JPG" -Value "jpeg"
Set-Content -Path "$sample\\IMG_9001.NEF" -Value "raw"
Set-Content -Path "$sample\\IMG_9002.JPG" -Value "jpeg"
```

Run:

```powershell
npm run tauri --prefix apps/desktop -- dev
```

Expected manual result:

- create a project named `Desktop MVP`;
- choose the sample folder;
- scan status reaches `completed`;
- grid shows two groups;
- `IMG_9001` shows both `JPG` and `RAW` badges;
- detail panel shows `source_kind` as `desktop_scan`;
- favorite toggle persists after refresh.

- [ ] **Step 4: Manual missing/changed verification**

In PowerShell while the app is open:

```powershell
Remove-Item "$env:TEMP\\camera-connector-desktop-sample\\IMG_9002.JPG"
Set-Content -Path "$env:TEMP\\camera-connector-desktop-sample\\IMG_9001.JPG" -Value "jpeg changed"
```

Run scan again from the same folder.

Expected manual result:

- `IMG_9002` remains visible with `missing` source state;
- `IMG_9001` remains visible with `changed` source state;
- previous favorite or marked state remains attached to the group.

- [ ] **Step 5: Manual recommendation verification**

If model provider settings are configured:

- click `Run Jobs`;
- wait for model scores to appear;
- click `Recommend Burst` on a burst group;
- confirm recommendation status updates in the grid and detail pane;
- click `Project Recommend`;
- confirm project recommendation is produced only after the manual action.

If model provider settings are not configured:

- click `Run Jobs`;
- confirm scanned assets remain browseable;
- confirm model-dependent actions return a visible setup/configuration error instead of hiding the assets.

- [ ] **Step 6: Record verification notes in the plan**

Append a `Verification Notes` section to this file:

```markdown
## Verification Notes

- `cargo test -p camera_connector_core`: PASS
- `cargo test -p camera-connector-ffi`: PASS
- `cargo test -p camera-connector-cli`: PASS
- `cargo check -p camera-connector-desktop`: PASS
- `npm run build --prefix apps/desktop`: PASS
- Manual desktop scan: PASS
- Manual missing/changed rescan: PASS
- Manual recommendation flow with provider configured: PASS
- Manual recommendation flow without provider configured: PASS, showing `model provider is not configured`
```

- [ ] **Step 7: Commit verification notes**

```powershell
git add docs/superpowers/plans/2026-06-07-desktop-workbench-mvp-vertical-slice-implementation.md
git commit -m "docs: record desktop mvp verification"
```

## Self-Review Checklist

- Spec coverage: The plan covers formal desktop scan provenance, project creation, local folder scanning, scan status, missing/changed file state, asset display, evaluation wiring, recommendation wiring, and desktop UI.
- Non-goals preserved: The plan does not use transfer/import tables as a desktop scan adapter, does not implement Android package import, does not implement preview cache tables, and does not make project-scope recommendations automatic.
- Conflict control: Core changes are additive except the required `assets` provenance migration, which preserves existing transfer assets and keeps receiver behavior unchanged.
- Type consistency: The same DTO names are used throughout: `DesktopScanSource`, `DesktopScanRun`, `DesktopScannedAssetInput`, `DesktopScanIndexResult`, `DesktopProjectScanResult`, `DesktopScanPhase`, and `DesktopSourceStatus`.

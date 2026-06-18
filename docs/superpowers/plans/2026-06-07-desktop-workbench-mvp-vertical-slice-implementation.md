# Desktop Workbench MVP Vertical Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a real desktop workbench MVP that creates a project, scans a local folder through formal `desktop_scan` acquisition events, displays grouped assets, and wires evaluation and recommendation actions.

**Architecture:** Expand `transfers` from receiver-only network transfer records into project asset acquisition events. Desktop scanning records one `desktop_scan_runs` row for the batch and stable `transfers(protocol = "desktop_scan")` rows for discovered files, then reuses the existing `assets` and `asset_groups` path.

**Tech Stack:** Rust 2021, rusqlite, existing `camera_connector_core`, Tauri 2, Vite, TypeScript, CSS, npm.

---

## File Structure

- Create `core/src/desktop_scan.rs`: desktop scan DTOs, scan phases, source status enum, file discovery helpers, and stable desktop scan transfer ids.
- Modify `core/src/lib.rs`: export desktop scan types and desktop scan result DTOs.
- Modify `core/src/model/received_asset.rs`: add optional scan state fields to project asset DTOs.
- Modify `core/src/storage/mod.rs`: add `desktop_scan_runs`, add scan state columns to `assets`, index `desktop_scan` transfers through the existing asset path, and mark missing/changed assets.
- Modify `core/src/service.rs`: add desktop scan service methods and schedule analysis jobs after scan completion according to `ProjectEvaluationSettings`.
- Create `core/tests/desktop_scan_tests.rs`: core tests for `desktop_scan` transfer provenance, grouping, scan status, missing/changed files, and settings-driven jobs.
- Modify root `Cargo.toml`: add `apps/desktop/src-tauri` workspace member and Tauri workspace dependencies.
- Create `apps/desktop/package.json`, `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/src/styles.css`: desktop frontend.
- Create `apps/desktop/src-tauri/Cargo.toml`, `apps/desktop/src-tauri/build.rs`, `apps/desktop/src-tauri/tauri.conf.json`, `apps/desktop/src-tauri/capabilities/default.json`, `apps/desktop/src-tauri/src/main.rs`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src-tauri/src/commands.rs`: Tauri shell and command gateway.

## Task 1: Formal `desktop_scan` Transfer Acquisition

**Files:**
- Create: `core/src/desktop_scan.rs`
- Modify: `core/src/lib.rs`
- Modify: `core/src/model/received_asset.rs`
- Modify: `core/src/storage/mod.rs`
- Test: `core/tests/desktop_scan_tests.rs`

- [ ] **Step 1: Write the failing provenance test**

Create `core/tests/desktop_scan_tests.rs`:

```rust
use camera_connector_core::{
    AssetGroupQuery, DesktopScannedFile, DesktopSourceStatus, SqliteStore, StoredObjectLocation,
    TransferStatus,
};

#[test]
fn desktop_scan_indexes_local_file_as_desktop_scan_transfer() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let store = SqliteStore::open(temp_dir.path().join("state.sqlite")).expect("store should open");
    let project = store
        .create_project("Desktop Scan")
        .expect("project should create");
    let root = temp_dir.path().join("photos");
    std::fs::create_dir_all(&root).expect("root should create");
    let photo = root.join("IMG_1001.JPG");
    std::fs::write(&photo, [1_u8, 2, 3, 4]).expect("sample should write");

    let scan = store
        .create_desktop_scan_run(&project.project_id, &root, 10_000)
        .expect("scan run should create");
    let result = store
        .record_desktop_scan_files(
            &scan.scan_id,
            &[DesktopScannedFile {
                local_path: photo.clone(),
                relative_path: "IMG_1001.JPG".to_string(),
                original_filename: "IMG_1001.JPG".to_string(),
                normalized_stem: "IMG_1001".to_string(),
                size_bytes: 4,
                modified_at_ms: 10_001,
                capture_time_ms: None,
            }],
            10_002,
        )
        .expect("desktop scan file should index");

    assert_eq!(result.assets_indexed, 1);
    assert_eq!(result.group_ids.len(), 1);

    let transfers = store
        .transfer_records(&project.project_id)
        .expect("transfer records should query");
    assert_eq!(transfers.len(), 1);
    assert!(transfers[0].transfer_id.starts_with("desktop-scan-"));
    assert_eq!(transfers[0].protocol, "desktop_scan");
    assert_eq!(transfers[0].status, TransferStatus::Completed);
    assert_eq!(transfers[0].original_path, "IMG_1001.JPG");
    assert_eq!(transfers[0].final_location, Some(StoredObjectLocation::local_path(photo.clone())));

    let page = store
        .asset_group_page(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("asset page should query");
    assert_eq!(page.summary.asset_count, 1);
    assert_eq!(page.groups[0].group_key, "IMG_1001");
    assert_eq!(
        page.groups[0].primary.source_status.as_deref(),
        Some(DesktopSourceStatus::Available.as_str())
    );
    assert_eq!(
        page.groups[0].primary.last_seen_scan_id.as_deref(),
        Some(scan.scan_id.as_str())
    );
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```powershell
cargo test -p camera_connector_core --test desktop_scan_tests desktop_scan_indexes_local_file_as_desktop_scan_transfer
```

Expected: FAIL with unresolved imports for `DesktopScannedFile`, `DesktopSourceStatus`, and missing `SqliteStore` methods.

- [ ] **Step 3: Add desktop scan DTOs and stable transfer ids**

Create `core/src/desktop_scan.rs`:

```rust
use std::path::{Path, PathBuf};

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
pub struct DesktopScanRun {
    pub scan_id: String,
    pub project_id: String,
    pub root_path: PathBuf,
    pub root_label: String,
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
pub struct DesktopScannedFile {
    pub local_path: PathBuf,
    pub relative_path: String,
    pub original_filename: String,
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

pub fn desktop_scan_root_key(project_id: &str, root_path: &Path) -> String {
    stable_key(&format!("{project_id}\t{}", root_path.display()))
}

pub fn desktop_scan_transfer_id(project_id: &str, root_path: &Path, relative_path: &str) -> String {
    let root_key = desktop_scan_root_key(project_id, root_path);
    let file_key = stable_key(&relative_path.replace('\\', "/").to_ascii_lowercase());
    format!("desktop-scan-{root_key}-{file_key}")
}

pub fn stable_key(value: &str) -> String {
    let mut hash = 1469598103934665603_u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("{hash:016x}")
}
```

Modify `core/src/lib.rs`:

```rust
pub mod desktop_scan;

pub use desktop_scan::{
    desktop_scan_root_key, desktop_scan_transfer_id, DesktopScanIndexResult,
    DesktopScanPhase, DesktopScanRun, DesktopScannedFile, DesktopSourceStatus,
};
```

- [ ] **Step 4: Add scan state to project asset DTOs**

In `core/src/model/received_asset.rs`, add fields to `ReceivedAsset`:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub source_status: Option<String>,
#[serde(default, skip_serializing_if = "Option::is_none")]
pub source_modified_at_ms: Option<i64>,
#[serde(default, skip_serializing_if = "Option::is_none")]
pub last_seen_scan_id: Option<String>,
```

Add defaults in `ReceivedAsset::new`:

```rust
source_status: None,
source_modified_at_ms: None,
last_seen_scan_id: None,
```

- [ ] **Step 5: Add `assets` scan state columns**

In `core/src/storage/mod.rs`, add fields to `StoredAsset`:

```rust
pub source_status: String,
pub source_modified_at_ms: Option<i64>,
pub last_seen_scan_id: Option<String>,
```

In `initialize_schema`, add columns to `assets`:

```sql
source_status TEXT NOT NULL DEFAULT 'available',
source_modified_at_ms INTEGER,
last_seen_scan_id TEXT,
```

Add a migration helper before the `CREATE TABLE IF NOT EXISTS assets` statement:

```rust
add_column_if_missing(connection, "assets", "source_status", "TEXT NOT NULL DEFAULT 'available'")?;
add_column_if_missing(connection, "assets", "source_modified_at_ms", "INTEGER")?;
add_column_if_missing(connection, "assets", "last_seen_scan_id", "TEXT")?;
```

Add the helper near `table_columns`:

```rust
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
```

Update every `SELECT` that reads `assets` to include:

```sql
source_status, source_modified_at_ms, last_seen_scan_id
```

Update `stored_asset_from_row` index positions and set these fields in `received_asset_from_row`:

```rust
asset.source_status = Some(stored.source_status);
asset.source_modified_at_ms = stored.source_modified_at_ms;
asset.last_seen_scan_id = stored.last_seen_scan_id;
```

Update `insert_asset_for_transfer` so regular receiver assets set:

```rust
source_status = DesktopSourceStatus::Available.as_str();
source_modified_at_ms = None::<i64>;
last_seen_scan_id = None::<String>;
```

- [ ] **Step 6: Add `desktop_scan_runs` storage**

Add this table to `initialize_schema`:

```sql
CREATE TABLE IF NOT EXISTS desktop_scan_runs (
    scan_id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(project_id),
    root_path TEXT NOT NULL,
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

CREATE INDEX IF NOT EXISTS idx_desktop_scan_runs_project
ON desktop_scan_runs(project_id, updated_at_ms);
```

Add storage methods:

```rust
pub fn create_desktop_scan_run(
    &self,
    project_id: &str,
    root_path: impl AsRef<Path>,
    now_ms: i64,
) -> Result<DesktopScanRun>;

pub fn desktop_scan_run(&self, scan_id: &str) -> Result<Option<DesktopScanRun>>;

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
```

Use this scan id:

```rust
let scan_id = format!(
    "desktop-scan-run-{}",
    crate::desktop_scan::stable_key(&format!("{project_id}\t{}\t{now_ms}", root_path.display()))
);
```

- [ ] **Step 7: Add desktop scan indexing through transfer records**

Add storage method:

```rust
pub fn record_desktop_scan_files(
    &self,
    scan_id: &str,
    files: &[DesktopScannedFile],
    now_ms: i64,
) -> Result<DesktopScanIndexResult>;
```

Implementation rules:

- Load the `DesktopScanRun` by `scan_id`.
- For each file, compute stable transfer id with `desktop_scan_transfer_id(project_id, root_path, relative_path)`.
- Read any existing asset row for that transfer id before updating.
- Source status is `changed` when the previous `source_modified_at_ms` differs from `file.modified_at_ms` or previous `size_bytes` differs from `file.size_bytes`; otherwise `available`.
- Create a `TransferRecord`:

```rust
TransferRecord {
    transfer_id,
    protocol: "desktop_scan".to_string(),
    status: TransferStatus::Completed,
    original_path: file.relative_path.clone(),
    final_filename: file.original_filename.clone(),
    final_location: Some(StoredObjectLocation::local_path(file.local_path.clone())),
    size_bytes: file.size_bytes,
    username: None,
    remote_addr: None,
    source_name: Some(scan.root_label.clone()),
    started_at_ms: scan.started_at_ms,
    completed_at_ms: Some(now_ms),
    error: None,
}
```

- Call the existing `insert_transfer` and `insert_asset_for_transfer` helpers.
- After insertion, update the matching `assets` row:

```sql
UPDATE assets
SET source_status = ?1,
    source_modified_at_ms = ?2,
    last_seen_scan_id = ?3
WHERE transfer_id = ?4
```

- Mark missing files for the same root after all current files are indexed. Use the stable root prefix:

```rust
let root_prefix = format!("desktop-scan-{}-", desktop_scan_root_key(project_id, &scan.root_path));
```

Then update desktop-scan assets under that root that were not seen in the current scan:

```sql
UPDATE assets
SET source_status = 'missing'
WHERE project_id = ?1
  AND transfer_id LIKE ?2
  AND (last_seen_scan_id IS NULL OR last_seen_scan_id <> ?3)
```

- Refresh group rollups and duplicate info for touched groups.
- Return unique touched group ids.

- [ ] **Step 8: Run the provenance test**

Run:

```powershell
cargo test -p camera_connector_core --test desktop_scan_tests desktop_scan_indexes_local_file_as_desktop_scan_transfer
```

Expected: PASS.

- [ ] **Step 9: Run storage regressions and commit**

Run:

```powershell
cargo test -p camera_connector_core --test asset_query_tests
cargo test -p camera_connector_core --test storage_store_tests
```

Expected: PASS.

Commit:

```powershell
git add core/src/desktop_scan.rs core/src/lib.rs core/src/model/received_asset.rs core/src/storage/mod.rs core/tests/desktop_scan_tests.rs
git commit -m "feat: add desktop scan transfer acquisition"
```

## Task 2: Core Desktop Folder Scanner And Analysis Scheduling

**Files:**
- Modify: `core/src/desktop_scan.rs`
- Modify: `core/src/service.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/desktop_scan_tests.rs`

- [ ] **Step 1: Add failing folder scan and rescan tests**

Append to `core/tests/desktop_scan_tests.rs`:

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
    let result = service.run_desktop_project_scan(&scan.scan_id).expect("scan should complete");

    assert_eq!(result.scan.phase, DesktopScanPhase::Completed);
    assert_eq!(result.scan.files_seen, 3);
    assert_eq!(result.index.assets_indexed, 3);

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
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
    let missing = page.groups.iter().find(|group| group.group_key == "IMG_3001").expect("missing group");
    assert_eq!(missing.primary.source_status.as_deref(), Some("missing"));
    assert!(missing.user_marks.favorite);
    assert!(missing.user_marks.marked);
    let changed = page.groups.iter().find(|group| group.group_key == "IMG_3002").expect("changed group");
    assert_eq!(changed.primary.source_status.as_deref(), Some("changed"));
}
```

- [ ] **Step 2: Run the tests and verify they fail**

Run:

```powershell
cargo test -p camera_connector_core --test desktop_scan_tests service_scans_folder_and_groups_raw_jpeg_video_by_stem
cargo test -p camera_connector_core --test desktop_scan_tests rescan_marks_missing_and_changed_without_deleting_group_marks
```

Expected: FAIL with missing service scan methods.

- [ ] **Step 3: Add media file discovery**

Append to `core/src/desktop_scan.rs`:

```rust
use std::fs;
use std::time::UNIX_EPOCH;

use crate::ObjectFormat;

pub fn discover_desktop_media_files(root: &Path) -> crate::Result<Vec<DesktopScannedFile>> {
    let mut files = Vec::new();
    visit_media_dir(root, root, &mut files)?;
    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(files)
}

fn visit_media_dir(root: &Path, current: &Path, files: &mut Vec<DesktopScannedFile>) -> crate::Result<()> {
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
        files.push(DesktopScannedFile {
            local_path: path,
            relative_path,
            original_filename: filename,
            normalized_stem,
            size_bytes: metadata.len(),
            modified_at_ms,
            capture_time_ms: None,
        });
    }
    Ok(())
}
```

- [ ] **Step 4: Add service scan methods**

In `core/src/service.rs`, import:

```rust
use crate::{discover_desktop_media_files, DesktopScanIndexResult, DesktopScanPhase, DesktopScanRun};
```

Add DTO:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesktopProjectScanResult {
    pub scan: DesktopScanRun,
    pub index: DesktopScanIndexResult,
}
```

Add methods:

```rust
pub fn create_desktop_project_scan(
    &self,
    project_id: &str,
    root_path: impl AsRef<Path>,
) -> Result<DesktopScanRun> {
    self.storage_store()?.create_desktop_scan_run(project_id, root_path, current_time_ms())
}

pub fn latest_desktop_project_scan(&self, project_id: &str) -> Result<Option<DesktopScanRun>> {
    self.storage_store()?.latest_desktop_scan_run(project_id)
}

pub fn run_desktop_project_scan(&self, scan_id: &str) -> Result<DesktopProjectScanResult> {
    let store = self.storage_store()?;
    let scan = store.desktop_scan_run(scan_id)?.ok_or_else(|| {
        crate::ImporterError::internal(format!("desktop scan not found: {scan_id}"))
    })?;
    store.update_desktop_scan_run(scan_id, DesktopScanPhase::Scanning, 0, 0, 0, None, current_time_ms())?;
    let files = discover_desktop_media_files(&scan.root_path)?;
    store.update_desktop_scan_run(scan_id, DesktopScanPhase::Indexing, files.len(), 0, 0, None, current_time_ms())?;
    let index = store.record_desktop_scan_files(scan_id, &files, current_time_ms())?;
    self.enqueue_desktop_scan_analysis_jobs(&scan.project_id, &index.group_ids)?;
    let completed = store.update_desktop_scan_run(
        scan_id,
        DesktopScanPhase::Completed,
        files.len(),
        index.assets_indexed,
        index.group_ids.len(),
        None,
        current_time_ms(),
    )?;
    Ok(DesktopProjectScanResult { scan: completed, index })
}
```

Export from `core/src/lib.rs`:

```rust
pub use service::{
    AccountView, AnalysisDrainSummary, AssetFacetCount, AssetGroupModelEvaluationInput,
    AssetGroupPage, AssetGroupQuery, AssetGroupSort, AssetGroupSummary, CameraConnectorDashboard,
    CameraConnectorService, ConnectedDeviceView, DesktopProjectScanResult,
    PublishQueueFailureView, ReceiverConfigRequest, ReceiverSettingsUpdate, SystemPathsView,
    TransferQuery, TransferRecordView, TransferSummary,
};
```

- [ ] **Step 5: Add scan completion analysis scheduling**

Add private method in `core/src/service.rs`:

```rust
fn enqueue_desktop_scan_analysis_jobs(&self, project_id: &str, group_ids: &[String]) -> Result<usize> {
    let store = self.storage_store()?;
    let settings = store
        .project_evaluation_settings(project_id)?
        .unwrap_or_else(|| ProjectEvaluationSettings::default_for_project(project_id, current_time_ms()));
    let provider_configured = self.provider_configured_for_model_work()?;
    let mut enqueued = 0;
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
        if settings.model_evaluation_enabled && settings.auto_evaluate_on_upload && provider_configured {
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

Do not enqueue `GenerateProjectRecommendation` from scan completion.

- [ ] **Step 6: Run tests and commit**

Run:

```powershell
cargo test -p camera_connector_core --test desktop_scan_tests
cargo test -p camera_connector_core --test analysis_job_tests
```

Expected: PASS.

Commit:

```powershell
git add core/src/desktop_scan.rs core/src/service.rs core/src/storage/mod.rs core/src/lib.rs core/tests/desktop_scan_tests.rs
git commit -m "feat: scan desktop folders through desktop transfers"
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

- [ ] **Step 2: Create Tauri backend files**

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

- [ ] **Step 3: Add command gateway**

Create `apps/desktop/src-tauri/src/commands.rs`:

```rust
use std::path::PathBuf;

use camera_connector_core::{
    AnalysisDrainSummary, AssetGroupPage, AssetGroupQuery, CameraConnectorDashboard,
    CameraConnectorService, DesktopScanRun, Project, SelectionRecommendation, StoredAsset,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

pub struct DesktopState {
    pub service: CameraConnectorService,
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

#[derive(Debug, Clone, Serialize)]
pub struct DesktopError {
    pub code: String,
    pub message: String,
}

fn desktop_error(error: camera_connector_core::ImporterError) -> DesktopError {
    DesktopError { code: error.code().to_string(), message: error.to_string() }
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
pub fn get_scan_status(state: State<'_, DesktopState>, project_id: String) -> Result<Option<DesktopScanRun>, DesktopError> {
    state.service.latest_desktop_project_scan(&project_id).map_err(desktop_error)
}

#[tauri::command]
pub fn get_project_asset_page(state: State<'_, DesktopState>, request: AssetPageRequest) -> Result<AssetGroupPage, DesktopError> {
    state
        .service
        .project_asset_group_page_with_query(&request.project_id, AssetGroupQuery::default(), request.offset, request.limit)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn get_project_group_detail(state: State<'_, DesktopState>, project_id: String, group_id: String) -> Result<Vec<StoredAsset>, DesktopError> {
    state.service.project_group_assets(&project_id, &group_id).map_err(desktop_error)
}

#[tauri::command]
pub fn save_group_user_marks(state: State<'_, DesktopState>, request: UserMarksRequest) -> Result<camera_connector_core::AssetUserMarks, DesktopError> {
    state
        .service
        .set_asset_group_user_marks(&request.project_id, &request.group_id, request.favorite, request.marked)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn drain_analysis_jobs(state: State<'_, DesktopState>, limit: usize) -> Result<AnalysisDrainSummary, DesktopError> {
    state.service.drain_analysis_jobs(limit).map_err(desktop_error)
}

#[tauri::command]
pub fn recommend_burst_group(state: State<'_, DesktopState>, burst_group_id: String) -> Result<SelectionRecommendation, DesktopError> {
    state.service.recommend_burst_group_from_model(&burst_group_id).map_err(desktop_error)
}

#[tauri::command]
pub fn generate_project_recommendation(state: State<'_, DesktopState>, project_id: String) -> Result<SelectionRecommendation, DesktopError> {
    state.service.generate_project_recommendation(&project_id, current_time_ms()).map_err(desktop_error)
}

#[tauri::command]
pub fn get_project_dashboard(state: State<'_, DesktopState>, project_id: String) -> Result<CameraConnectorDashboard, DesktopError> {
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

Create `apps/desktop/src-tauri/src/lib.rs`:

```rust
mod commands;

use camera_connector_core::CameraConnectorService;
use commands::DesktopState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(DesktopState {
            service: CameraConnectorService::new(None),
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

- [ ] **Step 4: Check and commit backend**

Run:

```powershell
cargo check -p camera-connector-desktop
```

Expected: PASS.

Commit:

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

- [ ] **Step 1: Create frontend package**

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

Create `tsconfig.json`, `index.html`, `src/main.ts`, and `src/styles.css` as a single-screen operational workbench: project sidebar, scan controls, scan progress strip, grouped asset grid, and detail panel. The UI should display `source_status`, model score, technical status, recommendation status, and user marks. It should call Tauri commands only through typed wrappers.

- [ ] **Step 2: Build and run**

Run:

```powershell
npm install --prefix apps/desktop
npm run build --prefix apps/desktop
npm run tauri --prefix apps/desktop -- dev
```

Expected: A Tauri window opens with project sidebar, scan controls, asset grid, and detail pane.

- [ ] **Step 3: Commit**

```powershell
git add apps/desktop/package.json apps/desktop/package-lock.json apps/desktop/tsconfig.json apps/desktop/index.html apps/desktop/src
git commit -m "feat: add desktop workbench ui"
```

## Task 5: End-To-End MVP Verification

**Files:**
- Modify: `docs/superpowers/plans/2026-06-07-desktop-workbench-mvp-vertical-slice-implementation.md`

- [ ] **Step 1: Run full checks**

Run:

```powershell
cargo test -p camera_connector_core
cargo test -p camera-connector-ffi
cargo test -p camera-connector-cli
cargo check -p camera-connector-desktop
npm run build --prefix apps/desktop
```

Expected: PASS.

- [ ] **Step 2: Manual scan verification**

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
- transfer records for scanned files have `protocol = desktop_scan`;
- favorite toggle persists after refresh.

- [ ] **Step 3: Manual missing/changed verification**

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

- [ ] **Step 4: Manual recommendation verification**

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
- confirm model-dependent actions return `model provider is not configured` instead of hiding assets.

- [ ] **Step 5: Commit verification notes**

Append:

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

Commit:

```powershell
git add docs/superpowers/plans/2026-06-07-desktop-workbench-mvp-vertical-slice-implementation.md
git commit -m "docs: record desktop mvp verification"
```

## Self-Review Checklist

- Spec coverage: The plan covers formal `desktop_scan` transfer acquisition, project creation, local folder scanning, scan status, missing/changed file state, asset display, evaluation wiring, recommendation wiring, and desktop UI.
- Non-goals preserved: The plan does not add `desktop_scan_sources`, does not add `desktop_scanned_assets`, does not implement Android package import, does not implement preview cache tables, and does not make project-scope recommendations automatic.
- Conflict control: Core changes are additive: `desktop_scan_runs`, asset scan-state columns, `desktop_scan` transfer protocol handling, and desktop service methods. `assets.transfer_id` remains non-null.
- Type consistency: The same DTO names are used throughout: `DesktopScanRun`, `DesktopScannedFile`, `DesktopScanIndexResult`, `DesktopProjectScanResult`, `DesktopScanPhase`, and `DesktopSourceStatus`.

# Scan-Integrated Project Sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the first JSON-snapshot version of scan-integrated project sync: load synchronized project facts, match them to desktop-scanned local assets, and apply matched marks/model context without adding any database tables or columns.

**Architecture:** Add a pure core `project_sync` module for snapshot parsing, in-memory matching, summary counts, and conversion into existing core models. Add a `CameraConnectorService` method that reads current project assets/groups through existing storage APIs and writes only existing durable concepts: asset group user marks, model evaluations, and selection recommendations. Add one desktop command that loads a local JSON snapshot path and returns compact transient counts.

**Tech Stack:** Rust, serde/serde_json, existing `camera_connector_core` storage/service APIs, Tauri command adapter, Cargo tests.

---

## File Structure

- Create `core/src/project_sync.rs`: snapshot DTOs, validation, ordered matching, sync summary, and pure helpers. This file must not open SQLite or own persistence.
- Modify `core/src/lib.rs`: export the new module and public sync types.
- Modify `core/src/service.rs`: add a service method that loads local project assets/groups, calls the pure matcher, and persists only existing marks/evaluations/recommendations.
- Create `core/tests/project_sync_tests.rs`: focused tests for parser, matcher, application behavior, ambiguity, and idempotency.
- Modify `apps/desktop/src-tauri/src/commands.rs`: add request/response DTOs, blocking helper, and Tauri command for syncing from a local JSON snapshot path.
- Modify `apps/desktop/src-tauri/src/lib.rs`: register the new command.

Do not modify `core/src/storage/mod.rs` schema creation, migrations, or table shapes. If an implementation step appears to need a new table or column, stop and redesign the step around transient command results.

## Task 1: Core Snapshot Types And Parser

**Files:**
- Create: `core/src/project_sync.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/project_sync_tests.rs`

- [ ] **Step 1: Write the failing parser tests**

Create `core/tests/project_sync_tests.rs` with:

```rust
use camera_connector_core::{
    parse_project_sync_snapshot_json, ProjectSyncSnapshot, ProjectSyncSnapshotAsset,
    ProjectSyncSnapshotGroup,
};

#[test]
fn project_sync_snapshot_parses_minimal_versioned_json() {
    let snapshot = parse_project_sync_snapshot_json(
        r#"{
          "schema_version": 1,
          "source_device": {
            "device_id": "phone-1",
            "device_label": "Pixel Field Kit",
            "platform": "android"
          },
          "project": {
            "project_id": "android-project-1",
            "name": "Wedding Selects",
            "exported_at_ms": 1781800000000
          },
          "assets": [{
            "asset_id": "asset-a",
            "group_id": "group-a",
            "original_filename": "IMG_1001.JPG",
            "final_filename": "IMG_1001.JPG",
            "normalized_stem": "IMG_1001",
            "original_path": "DCIM/100NIKON/IMG_1001.JPG",
            "original_parent_path": "DCIM/100NIKON",
            "format": "jpeg",
            "size_bytes": 4,
            "capture_at_ms": 1781000000000,
            "received_at_ms": 1781000001000,
            "source_identity": "camera-card-a"
          }],
          "groups": [{
            "group_id": "group-a",
            "display_key": "IMG_1001",
            "source_identity": "camera-card-a",
            "original_parent_path": "DCIM/100NIKON",
            "member_asset_ids": ["asset-a"],
            "primary_asset_id": "asset-a",
            "preview_asset_id": "asset-a",
            "has_raw": false,
            "has_jpeg": true,
            "has_video": false
          }],
          "model_evaluations": [],
          "selection_recommendations": [],
          "user_marks": []
        }"#,
    )
    .expect("snapshot should parse");

    assert_eq!(snapshot.schema_version, 1);
    assert_eq!(snapshot.source_device.device_label, "Pixel Field Kit");
    assert_eq!(snapshot.project.name, "Wedding Selects");
    assert_eq!(snapshot.assets[0].asset_id, "asset-a");
    assert_eq!(snapshot.groups[0].member_asset_ids, vec!["asset-a"]);
}

#[test]
fn project_sync_snapshot_rejects_unsupported_schema_version() {
    let error = parse_project_sync_snapshot_json(
        r#"{
          "schema_version": 2,
          "source_device": {"device_id": "phone", "device_label": "Phone", "platform": "android"},
          "project": {"project_id": "p", "name": "P", "exported_at_ms": 1},
          "assets": [],
          "groups": [],
          "model_evaluations": [],
          "selection_recommendations": [],
          "user_marks": []
        }"#,
    )
    .expect_err("version 2 should be rejected");

    assert!(error.to_string().contains("unsupported project sync schema_version 2"));
}
```

- [ ] **Step 2: Run parser tests to verify RED**

Run:

```powershell
cargo test -p camera_connector_core --test project_sync_tests project_sync_snapshot
```

Expected: compile fails because `parse_project_sync_snapshot_json`, `ProjectSyncSnapshot`, `ProjectSyncSnapshotAsset`, and `ProjectSyncSnapshotGroup` do not exist.

- [ ] **Step 3: Add core snapshot structs and parser**

Create `core/src/project_sync.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::{ImporterError, Result};

pub const PROJECT_SYNC_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshot {
    pub schema_version: i64,
    pub source_device: ProjectSyncSourceDevice,
    pub project: ProjectSyncProjectSummary,
    #[serde(default)]
    pub assets: Vec<ProjectSyncSnapshotAsset>,
    #[serde(default)]
    pub groups: Vec<ProjectSyncSnapshotGroup>,
    #[serde(default)]
    pub model_evaluations: Vec<ProjectSyncSnapshotModelEvaluation>,
    #[serde(default)]
    pub selection_recommendations: Vec<ProjectSyncSnapshotRecommendation>,
    #[serde(default)]
    pub user_marks: Vec<ProjectSyncSnapshotUserMarks>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncSourceDevice {
    pub device_id: String,
    pub device_label: String,
    pub platform: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncProjectSummary {
    pub project_id: String,
    pub name: String,
    pub exported_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshotAsset {
    pub asset_id: String,
    pub group_id: String,
    pub original_filename: String,
    pub final_filename: String,
    pub normalized_stem: String,
    pub original_path: String,
    pub original_parent_path: Option<String>,
    pub format: String,
    pub size_bytes: u64,
    pub capture_at_ms: Option<i64>,
    pub received_at_ms: Option<i64>,
    pub source_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshotGroup {
    pub group_id: String,
    pub display_key: String,
    pub source_identity: Option<String>,
    pub original_parent_path: Option<String>,
    pub member_asset_ids: Vec<String>,
    pub primary_asset_id: Option<String>,
    pub preview_asset_id: Option<String>,
    pub has_raw: bool,
    pub has_jpeg: bool,
    pub has_video: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshotModelEvaluation {
    pub evaluation_id: String,
    pub group_id: String,
    pub evaluator_version: String,
    pub status: String,
    pub score: i64,
    pub tier: String,
    pub selectable: bool,
    pub summary: String,
    #[serde(default)]
    pub strengths: Vec<String>,
    #[serde(default)]
    pub weaknesses: Vec<String>,
    #[serde(default)]
    pub technical_warnings: Vec<String>,
    pub prompt_pack_id: Option<String>,
    pub prompt_pack_version: Option<String>,
    pub prompt_hash: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshotRecommendation {
    pub recommendation_id: String,
    pub scope: String,
    pub subject_group_id: Option<String>,
    #[serde(default)]
    pub selected_group_ids: Vec<String>,
    #[serde(default)]
    pub candidate_group_ids: Vec<String>,
    #[serde(default)]
    pub rejected_group_ids: Vec<String>,
    pub status: String,
    pub confidence: f64,
    pub reason: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncSnapshotUserMarks {
    pub group_id: String,
    pub favorite: Option<bool>,
    pub marked: Option<bool>,
}

pub fn parse_project_sync_snapshot_json(value: &str) -> Result<ProjectSyncSnapshot> {
    let snapshot: ProjectSyncSnapshot = serde_json::from_str(value)
        .map_err(|error| ImporterError::internal(format!("invalid project sync snapshot: {error}")))?;
    if snapshot.schema_version != PROJECT_SYNC_SCHEMA_VERSION {
        return Err(ImporterError::internal(format!(
            "unsupported project sync schema_version {}",
            snapshot.schema_version
        )));
    }
    Ok(snapshot)
}
```

Modify `core/src/lib.rs`:

```rust
pub mod project_sync;
```

Add this export block near the existing `pub use` blocks:

```rust
pub use project_sync::{
    parse_project_sync_snapshot_json, ProjectSyncProjectSummary, ProjectSyncSnapshot,
    ProjectSyncSnapshotAsset, ProjectSyncSnapshotGroup, ProjectSyncSnapshotModelEvaluation,
    ProjectSyncSnapshotRecommendation, ProjectSyncSnapshotUserMarks, ProjectSyncSourceDevice,
    PROJECT_SYNC_SCHEMA_VERSION,
};
```

- [ ] **Step 4: Run parser tests to verify GREEN**

Run:

```powershell
cargo test -p camera_connector_core --test project_sync_tests project_sync_snapshot
```

Expected: both parser tests pass.

- [ ] **Step 5: Commit Task 1**

Run:

```powershell
git add core/src/project_sync.rs core/src/lib.rs core/tests/project_sync_tests.rs
git commit -m "Add project sync snapshot parser"
```

## Task 2: In-Memory Asset And Group Matcher

**Files:**
- Modify: `core/src/project_sync.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/project_sync_tests.rs`

- [ ] **Step 1: Add failing matcher tests**

Append to `core/tests/project_sync_tests.rs`:

```rust
use camera_connector_core::{
    match_project_sync_snapshot, ObjectFormat, ProjectSyncMatchStatus, StoredAsset,
    StoredAssetGroup, StoredObjectLocation,
};

#[test]
fn project_sync_matches_asset_by_filename_format_size_and_capture_time() {
    let snapshot = snapshot_with_one_asset("remote-a", "remote-g", "IMG_2001.JPG", "IMG_2001", 4, Some(55));
    let local_group = stored_group("local-g", "IMG_2001");
    let local_asset = stored_asset("local-a", "local-g", "IMG_2001.JPG", "IMG_2001", ObjectFormat::Jpeg, 4, Some(55))
        .with_original_path("different/local/root/IMG_2001.JPG");

    let report = match_project_sync_snapshot(
        &snapshot,
        &[local_group],
        &[local_asset],
    );

    assert_eq!(
        report.asset_matches.get("remote-a").map(|matched| matched.local_asset_id.as_str()),
        Some("local-a")
    );
    assert_eq!(
        report.group_matches.get("remote-g").map(|matched| matched.local_group_id.as_str()),
        Some("local-g")
    );
    assert_eq!(report.summary.matched_assets, 1);
    assert_eq!(report.summary.matched_groups, 1);
    assert_eq!(report.summary.ambiguous_assets, 0);
}

#[test]
fn project_sync_does_not_treat_cross_device_original_path_as_identity() {
    let snapshot = snapshot_with_one_asset("remote-a", "remote-g", "IMG_2003.JPG", "IMG_2003", 4, Some(77));
    let local_group = stored_group("local-g", "IMG_2003");
    let wrong_same_path = stored_asset("local-wrong", "local-g", "OTHER.JPG", "OTHER", ObjectFormat::Jpeg, 4, Some(77))
        .with_original_path("DCIM/IMG_2003.JPG");
    let right_different_path = stored_asset("local-right", "local-g", "IMG_2003.JPG", "IMG_2003", ObjectFormat::Jpeg, 4, Some(77))
        .with_original_path("desktop/imported/folder/IMG_2003.JPG");

    let report = match_project_sync_snapshot(
        &snapshot,
        &[local_group],
        &[wrong_same_path, right_different_path],
    );

    assert_eq!(
        report.asset_matches.get("remote-a").map(|matched| matched.local_asset_id.as_str()),
        Some("local-right")
    );
}

#[test]
fn project_sync_refuses_ambiguous_lower_confidence_asset_match() {
    let snapshot = snapshot_with_one_asset("remote-a", "remote-g", "IMG_2002.JPG", "IMG_2002", 4, None);
    let local_group = stored_group("local-g", "IMG_2002");
    let first = stored_asset("local-a", "local-g", "IMG_2002.JPG", "IMG_2002", ObjectFormat::Jpeg, 4, None);
    let second = stored_asset("local-b", "local-g", "COPY_IMG_2002.JPG", "IMG_2002", ObjectFormat::Jpeg, 4, None);

    let report = match_project_sync_snapshot(
        &snapshot,
        &[local_group],
        &[first, second],
    );

    assert!(report.asset_matches.get("remote-a").is_none());
    assert_eq!(
        report.asset_status.get("remote-a"),
        Some(&ProjectSyncMatchStatus::Ambiguous)
    );
    assert_eq!(report.summary.ambiguous_assets, 1);
    assert_eq!(report.summary.matched_assets, 0);
}

fn snapshot_with_one_asset(
    asset_id: &str,
    group_id: &str,
    filename: &str,
    stem: &str,
    size_bytes: u64,
    capture_at_ms: Option<i64>,
) -> ProjectSyncSnapshot {
    ProjectSyncSnapshot {
        schema_version: 1,
        source_device: camera_connector_core::ProjectSyncSourceDevice {
            device_id: "phone".to_string(),
            device_label: "Phone".to_string(),
            platform: "android".to_string(),
        },
        project: camera_connector_core::ProjectSyncProjectSummary {
            project_id: "remote-project".to_string(),
            name: "Remote Project".to_string(),
            exported_at_ms: 1,
        },
        assets: vec![ProjectSyncSnapshotAsset {
            asset_id: asset_id.to_string(),
            group_id: group_id.to_string(),
            original_filename: filename.to_string(),
            final_filename: filename.to_string(),
            normalized_stem: stem.to_string(),
            original_path: format!("DCIM/{filename}"),
            original_parent_path: Some("DCIM".to_string()),
            format: "jpeg".to_string(),
            size_bytes,
            capture_at_ms,
            received_at_ms: Some(2),
            source_identity: Some("camera-card".to_string()),
        }],
        groups: vec![ProjectSyncSnapshotGroup {
            group_id: group_id.to_string(),
            display_key: stem.to_string(),
            source_identity: Some("camera-card".to_string()),
            original_parent_path: Some("DCIM".to_string()),
            member_asset_ids: vec![asset_id.to_string()],
            primary_asset_id: Some(asset_id.to_string()),
            preview_asset_id: Some(asset_id.to_string()),
            has_raw: false,
            has_jpeg: true,
            has_video: false,
        }],
        model_evaluations: Vec::new(),
        selection_recommendations: Vec::new(),
        user_marks: Vec::new(),
    }
}

fn stored_group(group_id: &str, display_key: &str) -> StoredAssetGroup {
    StoredAssetGroup {
        group_id: group_id.to_string(),
        project_id: "local-project".to_string(),
        group_identity: format!("group-{display_key}"),
        display_key: display_key.to_string(),
        source_identity: Some("camera-card".to_string()),
        original_parent_path: Some("DCIM".to_string()),
        primary_asset_id: Some(format!("asset-{group_id}")),
        preview_asset_id: Some(format!("asset-{group_id}")),
        member_count: 1,
        has_raw: false,
        has_jpeg: true,
        has_video: false,
        first_capture_at_ms: None,
        last_capture_at_ms: None,
        first_received_at_ms: None,
        last_received_at_ms: None,
        created_at_ms: 1,
        updated_at_ms: 1,
    }
}

fn stored_asset(
    asset_id: &str,
    group_id: &str,
    filename: &str,
    stem: &str,
    format: ObjectFormat,
    size_bytes: u64,
    capture_at_ms: Option<i64>,
) -> StoredAsset {
    StoredAsset {
        asset_id: asset_id.to_string(),
        project_id: "local-project".to_string(),
        group_id: Some(group_id.to_string()),
        transfer_id: asset_id.to_string(),
        group_role: "jpeg".to_string(),
        media_kind: "photo".to_string(),
        format,
        original_filename: filename.to_string(),
        final_filename: filename.to_string(),
        normalized_stem: stem.to_string(),
        original_path: filename.to_string(),
        original_parent_path: Some("DCIM".to_string()),
        final_location: Some(StoredObjectLocation::local_path(format!("C:/photos/{filename}"))),
        size_bytes,
        capture_at_ms,
        received_at_ms: Some(2),
        published_at_ms: Some(2),
        source_identity: Some("camera-card".to_string()),
        username: None,
        remote_addr: None,
        source_status: "available".to_string(),
        source_modified_at_ms: Some(3),
        last_seen_scan_id: Some("scan".to_string()),
        duplicate_index: None,
        duplicate_count: None,
    }
}

trait StoredAssetTestExt {
    fn with_original_path(self, original_path: &str) -> Self;
}

impl StoredAssetTestExt for StoredAsset {
    fn with_original_path(mut self, original_path: &str) -> Self {
        self.original_path = original_path.to_string();
        self.original_parent_path = original_path.rsplit_once('/').map(|(parent, _)| parent.to_string());
        self
    }
}
```

- [ ] **Step 2: Run matcher tests to verify RED**

Run:

```powershell
cargo test -p camera_connector_core --test project_sync_tests project_sync_matches
```

Expected: compile fails because `match_project_sync_snapshot`, `ProjectSyncMatchStatus`, and report types do not exist.

- [ ] **Step 3: Add matcher report types and ordered matching**

Modify `core/src/project_sync.rs`:

```rust
use std::collections::{BTreeMap, BTreeSet};

use crate::{ImporterError, ObjectFormat, Result, StoredAsset, StoredAssetGroup};
```

Add after snapshot structs:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectSyncMatchStatus {
    Matched,
    Unmatched,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncAssetMatch {
    pub snapshot_asset_id: String,
    pub local_asset_id: String,
    pub local_group_id: String,
    pub confidence_rank: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncGroupMatch {
    pub snapshot_group_id: String,
    pub local_group_id: String,
    pub confidence_rank: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncSummary {
    pub matched_assets: usize,
    pub unmatched_assets: usize,
    pub ambiguous_assets: usize,
    pub matched_groups: usize,
    pub unmatched_groups: usize,
    pub ambiguous_groups: usize,
    pub applied_user_marks: usize,
    pub imported_model_evaluations: usize,
    pub imported_selection_recommendations: usize,
    pub skipped_records: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSyncMatchReport {
    pub summary: ProjectSyncSummary,
    pub asset_matches: BTreeMap<String, ProjectSyncAssetMatch>,
    pub group_matches: BTreeMap<String, ProjectSyncGroupMatch>,
    pub asset_status: BTreeMap<String, ProjectSyncMatchStatus>,
    pub group_status: BTreeMap<String, ProjectSyncMatchStatus>,
}

pub fn match_project_sync_snapshot(
    snapshot: &ProjectSyncSnapshot,
    local_groups: &[StoredAssetGroup],
    local_assets: &[StoredAsset],
) -> ProjectSyncMatchReport {
    let mut report = ProjectSyncMatchReport::default();
    let mut group_ids_by_snapshot_asset = BTreeMap::new();

    for snapshot_asset in &snapshot.assets {
        let candidates = matching_assets(snapshot_asset, local_assets);
        match unique_candidate(candidates) {
            CandidateResolution::Matched((rank, asset)) => {
                if let Some(group_id) = asset.group_id.clone() {
                    report.asset_status.insert(snapshot_asset.asset_id.clone(), ProjectSyncMatchStatus::Matched);
                    report.asset_matches.insert(
                        snapshot_asset.asset_id.clone(),
                        ProjectSyncAssetMatch {
                            snapshot_asset_id: snapshot_asset.asset_id.clone(),
                            local_asset_id: asset.asset_id.clone(),
                            local_group_id: group_id.clone(),
                            confidence_rank: rank,
                        },
                    );
                    group_ids_by_snapshot_asset.insert(snapshot_asset.asset_id.clone(), group_id);
                    report.summary.matched_assets += 1;
                } else {
                    report.asset_status.insert(snapshot_asset.asset_id.clone(), ProjectSyncMatchStatus::Unmatched);
                    report.summary.unmatched_assets += 1;
                }
            }
            CandidateResolution::Ambiguous => {
                report.asset_status.insert(snapshot_asset.asset_id.clone(), ProjectSyncMatchStatus::Ambiguous);
                report.summary.ambiguous_assets += 1;
            }
            CandidateResolution::Unmatched => {
                report.asset_status.insert(snapshot_asset.asset_id.clone(), ProjectSyncMatchStatus::Unmatched);
                report.summary.unmatched_assets += 1;
            }
        }
    }

    for snapshot_group in &snapshot.groups {
        let candidates = matching_groups(snapshot_group, local_groups, &group_ids_by_snapshot_asset);
        match unique_group_candidate(candidates) {
            CandidateResolution::Matched((rank, group)) => {
                report.group_status.insert(snapshot_group.group_id.clone(), ProjectSyncMatchStatus::Matched);
                report.group_matches.insert(
                    snapshot_group.group_id.clone(),
                    ProjectSyncGroupMatch {
                        snapshot_group_id: snapshot_group.group_id.clone(),
                        local_group_id: group.group_id.clone(),
                        confidence_rank: rank,
                    },
                );
                report.summary.matched_groups += 1;
            }
            CandidateResolution::Ambiguous => {
                report.group_status.insert(snapshot_group.group_id.clone(), ProjectSyncMatchStatus::Ambiguous);
                report.summary.ambiguous_groups += 1;
            }
            CandidateResolution::Unmatched => {
                report.group_status.insert(snapshot_group.group_id.clone(), ProjectSyncMatchStatus::Unmatched);
                report.summary.unmatched_groups += 1;
            }
        }
    }

    report
}

enum CandidateResolution<T> {
    Matched(T),
    Ambiguous,
    Unmatched,
}

fn unique_candidate<'a>(candidates: Vec<(u8, &'a StoredAsset)>) -> CandidateResolution<(u8, &'a StoredAsset)> {
    if candidates.is_empty() {
        return CandidateResolution::Unmatched;
    }
    let best_rank = candidates[0].0;
    let best = candidates
        .into_iter()
        .filter(|(rank, _)| *rank == best_rank)
        .collect::<Vec<_>>();
    if best.len() == 1 {
        CandidateResolution::Matched(best[0])
    } else {
        CandidateResolution::Ambiguous
    }
}

fn unique_group_candidate<'a>(candidates: Vec<(u8, &'a StoredAssetGroup)>) -> CandidateResolution<(u8, &'a StoredAssetGroup)> {
    if candidates.is_empty() {
        return CandidateResolution::Unmatched;
    }
    let best_rank = candidates[0].0;
    let best = candidates
        .into_iter()
        .filter(|(rank, _)| *rank == best_rank)
        .collect::<Vec<_>>();
    if best.len() == 1 {
        CandidateResolution::Matched(best[0])
    } else {
        CandidateResolution::Ambiguous
    }
}

fn matching_assets<'a>(
    snapshot_asset: &ProjectSyncSnapshotAsset,
    local_assets: &'a [StoredAsset],
) -> Vec<(u8, &'a StoredAsset)> {
    let snapshot_format = ObjectFormat::from_str(&snapshot_asset.format);
    let mut candidates = Vec::new();
    for asset in local_assets {
        if asset.format != snapshot_format {
            continue;
        }
        if filename_matches(asset, snapshot_asset)
            && asset.size_bytes == snapshot_asset.size_bytes
            && asset.capture_at_ms == snapshot_asset.capture_at_ms
        {
            candidates.push((1, asset));
            continue;
        }
        if asset.normalized_stem.eq_ignore_ascii_case(&snapshot_asset.normalized_stem)
            && asset.size_bytes == snapshot_asset.size_bytes
            && asset.capture_at_ms == snapshot_asset.capture_at_ms
        {
            candidates.push((2, asset));
            continue;
        }
        if filename_matches(asset, snapshot_asset)
            && asset.size_bytes == snapshot_asset.size_bytes
        {
            candidates.push((3, asset));
            continue;
        }
        if asset.normalized_stem.eq_ignore_ascii_case(&snapshot_asset.normalized_stem)
            && asset.size_bytes == snapshot_asset.size_bytes
        {
            candidates.push((4, asset));
            continue;
        }
        if asset.normalized_stem.eq_ignore_ascii_case(&snapshot_asset.normalized_stem) {
            candidates.push((5, asset));
        }
    }
    candidates.sort_by_key(|(rank, asset)| (*rank, asset.asset_id.clone()));
    candidates
}

fn filename_matches(asset: &StoredAsset, snapshot_asset: &ProjectSyncSnapshotAsset) -> bool {
    asset.original_filename.eq_ignore_ascii_case(&snapshot_asset.original_filename)
        || asset.original_filename.eq_ignore_ascii_case(&snapshot_asset.final_filename)
        || asset.final_filename.eq_ignore_ascii_case(&snapshot_asset.original_filename)
        || asset.final_filename.eq_ignore_ascii_case(&snapshot_asset.final_filename)
}

fn matching_groups<'a>(
    snapshot_group: &ProjectSyncSnapshotGroup,
    local_groups: &'a [StoredAssetGroup],
    group_ids_by_snapshot_asset: &BTreeMap<String, String>,
) -> Vec<(u8, &'a StoredAssetGroup)> {
    let matched_member_groups = snapshot_group
        .member_asset_ids
        .iter()
        .filter_map(|asset_id| group_ids_by_snapshot_asset.get(asset_id))
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut candidates = Vec::new();
    if matched_member_groups.len() == 1 && matched_member_groups.len() == snapshot_group.member_asset_ids.len() {
        let local_group_id = matched_member_groups.iter().next().expect("one member group");
        candidates.extend(local_groups.iter().filter(|group| &group.group_id == local_group_id).map(|group| (1, group)));
    }
    if matched_member_groups.len() == 1 {
        let local_group_id = matched_member_groups.iter().next().expect("one member group");
        candidates.extend(local_groups.iter().filter(|group| &group.group_id == local_group_id).map(|group| (2, group)));
    }
    for group in local_groups {
        if group.source_identity == snapshot_group.source_identity
            && group.display_key.eq_ignore_ascii_case(&snapshot_group.display_key)
        {
            candidates.push((3, group));
            continue;
        }
        if group.display_key.eq_ignore_ascii_case(&snapshot_group.display_key) {
            candidates.push((4, group));
        }
    }
    candidates.sort_by_key(|(rank, group)| (*rank, group.group_id.clone()));
    candidates
}
```

Modify `core/src/lib.rs` export block for project sync:

```rust
pub use project_sync::{
    match_project_sync_snapshot, parse_project_sync_snapshot_json, ProjectSyncAssetMatch,
    ProjectSyncGroupMatch, ProjectSyncMatchReport, ProjectSyncMatchStatus,
    ProjectSyncProjectSummary, ProjectSyncSnapshot, ProjectSyncSnapshotAsset,
    ProjectSyncSnapshotGroup, ProjectSyncSnapshotModelEvaluation,
    ProjectSyncSnapshotRecommendation, ProjectSyncSnapshotUserMarks, ProjectSyncSourceDevice,
    ProjectSyncSummary, PROJECT_SYNC_SCHEMA_VERSION,
};
```

- [ ] **Step 4: Run matcher tests to verify GREEN**

Run:

```powershell
cargo test -p camera_connector_core --test project_sync_tests project_sync_matches
```

Expected: matcher tests pass.

- [ ] **Step 5: Run parser tests again**

Run:

```powershell
cargo test -p camera_connector_core --test project_sync_tests project_sync_snapshot
```

Expected: parser tests still pass.

- [ ] **Step 6: Commit Task 2**

Run:

```powershell
git add core/src/project_sync.rs core/src/lib.rs core/tests/project_sync_tests.rs
git commit -m "Add project sync matcher"
```

## Task 3: Service Sync Application With Existing Tables Only

**Files:**
- Modify: `core/src/project_sync.rs`
- Modify: `core/src/service.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/project_sync_tests.rs`

- [ ] **Step 1: Add failing service application tests**

Append to `core/tests/project_sync_tests.rs`:

```rust
use camera_connector_core::{
    CameraConnectorService, ModelEvaluatorKind, SelectionRecommendationScope,
};

#[test]
fn service_project_sync_applies_marks_and_imported_model_context_to_matched_groups() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("config.json");
    let root = temp_dir.path().join("photos");
    std::fs::create_dir_all(&root).expect("photo root should create");
    std::fs::write(root.join("IMG_3100.JPG"), [1_u8, 2, 3, 4]).expect("photo should write");

    let service = CameraConnectorService::new(Some(config_path));
    let project = service.create_project("Project Sync").expect("project should create");
    let scan = service
        .create_desktop_project_scan(&project.project_id, &root)
        .expect("scan should create");
    service.run_desktop_project_scan(&scan.scan_id).expect("scan should run");

    let snapshot = snapshot_with_one_asset("remote-a", "remote-g", "IMG_3100.JPG", "IMG_3100", 4, None)
        .with_user_mark("remote-g", Some(true), Some(false))
        .with_model_evaluation("eval-remote", "remote-g", 91)
        .with_project_recommendation("rec-remote", vec!["remote-g"]);

    let summary = service
        .sync_project_snapshot(&project.project_id, &snapshot)
        .expect("sync should apply");

    assert_eq!(summary.matched_assets, 1);
    assert_eq!(summary.matched_groups, 1);
    assert_eq!(summary.applied_user_marks, 1);
    assert_eq!(summary.imported_model_evaluations, 1);
    assert_eq!(summary.imported_selection_recommendations, 1);
    assert_eq!(summary.skipped_records, 0);

    let page = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("page should query");
    let group = &page.groups[0];
    let group_id = group.group_id.as_ref().expect("group id should exist");
    assert!(group.user_marks.favorite);
    assert!(!group.user_marks.marked);
    assert_eq!(group.model_evaluation.as_ref().map(|evaluation| evaluation.score), Some(91));

    let recommendation = service
        .storage_store()
        .expect("store should open")
        .latest_selection_recommendation(
            &project.project_id,
            SelectionRecommendationScope::Project,
            &project.project_id,
        )
        .expect("recommendation should query")
        .expect("recommendation should import");
    assert_eq!(recommendation.selected_asset_group_ids, vec![group_id.clone()]);
}

#[test]
fn service_project_sync_is_idempotent_for_same_snapshot() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let config_path = temp_dir.path().join("config.json");
    let root = temp_dir.path().join("photos");
    std::fs::create_dir_all(&root).expect("photo root should create");
    std::fs::write(root.join("IMG_3200.JPG"), [1_u8, 2, 3, 4]).expect("photo should write");

    let service = CameraConnectorService::new(Some(config_path));
    let project = service.create_project("Idempotent Sync").expect("project should create");
    let scan = service
        .create_desktop_project_scan(&project.project_id, &root)
        .expect("scan should create");
    service.run_desktop_project_scan(&scan.scan_id).expect("scan should run");

    let snapshot = snapshot_with_one_asset("remote-a", "remote-g", "IMG_3200.JPG", "IMG_3200", 4, None)
        .with_model_evaluation("eval-remote", "remote-g", 88);

    service.sync_project_snapshot(&project.project_id, &snapshot).expect("first sync");
    let second = service.sync_project_snapshot(&project.project_id, &snapshot).expect("second sync");

    assert_eq!(second.imported_model_evaluations, 1);
    let group_id = service
        .project_asset_group_page_with_query(&project.project_id, AssetGroupQuery::default(), 0, 25)
        .expect("page should query")
        .groups[0]
        .group_id
        .clone()
        .expect("group id should exist");
    let evaluations = service
        .storage_store()
        .expect("store should open")
        .model_evaluations_for_asset_groups(&[group_id], "imported:phone:eval-remote")
        .expect("evaluations should query");
    assert_eq!(evaluations.len(), 1);
}

trait SnapshotTestExt {
    fn with_user_mark(self, group_id: &str, favorite: Option<bool>, marked: Option<bool>) -> Self;
    fn with_model_evaluation(self, evaluation_id: &str, group_id: &str, score: i64) -> Self;
    fn with_project_recommendation(self, recommendation_id: &str, selected_group_ids: Vec<&str>) -> Self;
}

impl SnapshotTestExt for ProjectSyncSnapshot {
    fn with_user_mark(mut self, group_id: &str, favorite: Option<bool>, marked: Option<bool>) -> Self {
        self.user_marks.push(camera_connector_core::ProjectSyncSnapshotUserMarks {
            group_id: group_id.to_string(),
            favorite,
            marked,
        });
        self
    }

    fn with_model_evaluation(mut self, evaluation_id: &str, group_id: &str, score: i64) -> Self {
        self.model_evaluations.push(camera_connector_core::ProjectSyncSnapshotModelEvaluation {
            evaluation_id: evaluation_id.to_string(),
            group_id: group_id.to_string(),
            evaluator_version: format!("imported:phone:{evaluation_id}"),
            status: "ready".to_string(),
            score,
            tier: "excellent".to_string(),
            selectable: true,
            summary: "Imported model summary".to_string(),
            strengths: vec!["Strong expression".to_string()],
            weaknesses: Vec::new(),
            technical_warnings: Vec::new(),
            prompt_pack_id: Some("remote-pack".to_string()),
            prompt_pack_version: Some("1".to_string()),
            prompt_hash: Some("hash".to_string()),
            created_at_ms: 10,
            updated_at_ms: 11,
        });
        self
    }

    fn with_project_recommendation(mut self, recommendation_id: &str, selected_group_ids: Vec<&str>) -> Self {
        self.selection_recommendations.push(camera_connector_core::ProjectSyncSnapshotRecommendation {
            recommendation_id: recommendation_id.to_string(),
            scope: "project".to_string(),
            subject_group_id: None,
            selected_group_ids: selected_group_ids.into_iter().map(str::to_string).collect(),
            candidate_group_ids: Vec::new(),
            rejected_group_ids: Vec::new(),
            status: "ready".to_string(),
            confidence: 0.91,
            reason: "Imported project recommendation".to_string(),
            created_at_ms: 12,
            updated_at_ms: 13,
        });
        self
    }
}
```

- [ ] **Step 2: Run service application tests to verify RED**

Run:

```powershell
cargo test -p camera_connector_core --test project_sync_tests service_project_sync
```

Expected: compile fails because `CameraConnectorService::sync_project_snapshot` and conversion helpers do not exist.

- [ ] **Step 3: Add sync application helpers**

Modify `core/src/project_sync.rs` imports:

```rust
use crate::{
    ImporterError, ModelEvaluation, ModelEvaluationStatus, ModelEvaluationTier, ModelEvaluatorKind,
    ObjectFormat, Result, SelectionRecommendation, SelectionRecommendationScope,
    SelectionRecommendationStatus, SelectionSource, StoredAsset, StoredAssetGroup,
};
```

Add conversion helpers:

```rust
pub fn imported_model_evaluation(
    snapshot: &ProjectSyncSnapshot,
    record: &ProjectSyncSnapshotModelEvaluation,
    local_project_id: &str,
    local_group_id: &str,
) -> ModelEvaluation {
    let evaluator_version = if record.evaluator_version.trim().is_empty() {
        format!("imported:{}:{}", snapshot.source_device.device_id, record.evaluation_id)
    } else {
        record.evaluator_version.clone()
    };
    ModelEvaluation {
        evaluation_id: stable_sync_id(
            "imported-evaluation",
            &[local_project_id, local_group_id, &snapshot.source_device.device_id, &record.evaluation_id],
        ),
        run_id: stable_sync_id(
            "imported-run",
            &[local_project_id, local_group_id, &snapshot.source_device.device_id, &record.evaluation_id],
        ),
        project_id: local_project_id.to_string(),
        asset_group_id: local_group_id.to_string(),
        evaluator_kind: ModelEvaluatorKind::Imported,
        evaluator_version,
        status: ModelEvaluationStatus::from_str(&record.status),
        score: record.score,
        tier: ModelEvaluationTier::from_str(&record.tier),
        selectable: record.selectable,
        summary: record.summary.clone(),
        strengths: record.strengths.clone(),
        weaknesses: record.weaknesses.clone(),
        technical_warnings: record.technical_warnings.clone(),
        prompt_pack_id: record.prompt_pack_id.clone(),
        prompt_pack_version: record.prompt_pack_version.clone(),
        prompt_hash: record.prompt_hash.clone(),
        created_at_ms: record.created_at_ms,
        updated_at_ms: record.updated_at_ms,
    }
}

pub fn imported_selection_recommendation(
    snapshot: &ProjectSyncSnapshot,
    record: &ProjectSyncSnapshotRecommendation,
    local_project_id: &str,
    subject_id: String,
    selected_asset_group_ids: Vec<String>,
    candidate_asset_group_ids: Vec<String>,
    rejected_asset_group_ids: Vec<String>,
) -> SelectionRecommendation {
    SelectionRecommendation {
        recommendation_id: stable_sync_id(
            "imported-recommendation",
            &[local_project_id, &snapshot.source_device.device_id, &record.recommendation_id],
        ),
        run_id: None,
        scope: SelectionRecommendationScope::from_str(&record.scope),
        project_id: local_project_id.to_string(),
        subject_id,
        selected_asset_group_ids,
        candidate_asset_group_ids,
        rejected_asset_group_ids,
        source: SelectionSource::Imported,
        status: SelectionRecommendationStatus::from_str(&record.status),
        confidence: record.confidence,
        reason: record.reason.clone(),
        created_at_ms: record.created_at_ms,
        updated_at_ms: record.updated_at_ms,
    }
}

fn stable_sync_id(prefix: &str, parts: &[&str]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{prefix}-{:016x}", hash)
}
```

- [ ] **Step 4: Add service method using only existing storage APIs**

Modify `core/src/service.rs` imports at the top to include:

```rust
    imported_model_evaluation, imported_selection_recommendation, match_project_sync_snapshot,
    ProjectSyncSnapshot, ProjectSyncSummary,
```

Add this method inside `impl CameraConnectorService` near other project asset methods:

```rust
    pub fn sync_project_snapshot(
        &self,
        project_id: &str,
        snapshot: &ProjectSyncSnapshot,
    ) -> Result<ProjectSyncSummary> {
        let store = self.storage_store()?;
        let groups = store.stored_asset_groups(project_id)?;
        let mut assets = Vec::new();
        for group in &groups {
            assets.extend(store.assets_for_group(project_id, &group.group_id)?);
        }

        let report = match_project_sync_snapshot(snapshot, &groups, &assets);
        let mut summary = report.summary.clone();

        for mark in &snapshot.user_marks {
            let Some(group_match) = report.group_matches.get(&mark.group_id) else {
                summary.skipped_records += 1;
                continue;
            };
            store.set_asset_group_user_marks(
                project_id,
                &group_match.local_group_id,
                mark.favorite,
                mark.marked,
            )?;
            summary.applied_user_marks += 1;
        }

        for evaluation in &snapshot.model_evaluations {
            let Some(group_match) = report.group_matches.get(&evaluation.group_id) else {
                summary.skipped_records += 1;
                continue;
            };
            store.save_model_evaluation(imported_model_evaluation(
                snapshot,
                evaluation,
                project_id,
                &group_match.local_group_id,
            ))?;
            summary.imported_model_evaluations += 1;
        }

        for recommendation in &snapshot.selection_recommendations {
            let Some((subject_id, selected, candidate, rejected)) =
                map_recommendation_groups(project_id, recommendation, &report.group_matches)
            else {
                summary.skipped_records += 1;
                continue;
            };
            store.save_selection_recommendation(imported_selection_recommendation(
                snapshot,
                recommendation,
                project_id,
                subject_id,
                selected,
                candidate,
                rejected,
            ))?;
            summary.imported_selection_recommendations += 1;
        }

        Ok(summary)
    }
```

Add this private helper near other free helpers in `core/src/service.rs`:

```rust
fn map_recommendation_groups(
    project_id: &str,
    recommendation: &crate::ProjectSyncSnapshotRecommendation,
    group_matches: &std::collections::BTreeMap<String, crate::ProjectSyncGroupMatch>,
) -> Option<(String, Vec<String>, Vec<String>, Vec<String>)> {
    let selected = map_snapshot_group_ids(&recommendation.selected_group_ids, group_matches)?;
    let candidate = map_snapshot_group_ids(&recommendation.candidate_group_ids, group_matches)?;
    let rejected = map_snapshot_group_ids(&recommendation.rejected_group_ids, group_matches)?;
    let subject_id = match crate::SelectionRecommendationScope::from_str(&recommendation.scope) {
        crate::SelectionRecommendationScope::Project => project_id.to_string(),
        crate::SelectionRecommendationScope::BurstGroup => {
            let subject_group_id = recommendation.subject_group_id.as_ref()?;
            group_matches.get(subject_group_id)?.local_group_id.clone()
        }
    };
    Some((subject_id, selected, candidate, rejected))
}

fn map_snapshot_group_ids(
    snapshot_group_ids: &[String],
    group_matches: &std::collections::BTreeMap<String, crate::ProjectSyncGroupMatch>,
) -> Option<Vec<String>> {
    snapshot_group_ids
        .iter()
        .map(|group_id| group_matches.get(group_id).map(|matched| matched.local_group_id.clone()))
        .collect()
}
```

- [ ] **Step 5: Export application helper types**

Modify `core/src/lib.rs` project sync export block:

```rust
pub use project_sync::{
    imported_model_evaluation, imported_selection_recommendation, match_project_sync_snapshot,
    parse_project_sync_snapshot_json, ProjectSyncAssetMatch, ProjectSyncGroupMatch,
    ProjectSyncMatchReport, ProjectSyncMatchStatus, ProjectSyncProjectSummary,
    ProjectSyncSnapshot, ProjectSyncSnapshotAsset, ProjectSyncSnapshotGroup,
    ProjectSyncSnapshotModelEvaluation, ProjectSyncSnapshotRecommendation,
    ProjectSyncSnapshotUserMarks, ProjectSyncSourceDevice, ProjectSyncSummary,
    PROJECT_SYNC_SCHEMA_VERSION,
};
```

- [ ] **Step 6: Run service tests to verify GREEN**

Run:

```powershell
cargo test -p camera_connector_core --test project_sync_tests service_project_sync
```

Expected: service sync tests pass.

- [ ] **Step 7: Run all project sync tests**

Run:

```powershell
cargo test -p camera_connector_core --test project_sync_tests
```

Expected: all project sync tests pass.

- [ ] **Step 8: Commit Task 3**

Run:

```powershell
git add core/src/project_sync.rs core/src/service.rs core/src/lib.rs core/tests/project_sync_tests.rs
git commit -m "Apply project sync snapshots to existing project data"
```

## Task 4: Desktop Command For Local JSON Snapshot Sync

**Files:**
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Test: `apps/desktop/src-tauri/src/commands.rs`

- [ ] **Step 1: Add failing desktop command helper test**

Append inside the existing `#[cfg(test)] mod tests` in `apps/desktop/src-tauri/src/commands.rs`:

```rust
    #[test]
    fn sync_project_snapshot_from_path_returns_compact_counts() {
        let temp_dir = unique_temp_dir("desktop-project-sync");
        let config_path = temp_dir.join("config.json");
        let root = temp_dir.join("photos");
        fs::create_dir_all(&root).expect("photo root should create");
        fs::write(root.join("IMG_4100.JPG"), [1_u8, 2, 3, 4]).expect("photo should write");

        let service = CameraConnectorService::new(Some(config_path));
        let project = service.create_project("Desktop Sync Command").expect("project should create");
        let scan = service
            .create_desktop_project_scan(&project.project_id, &root)
            .expect("scan should create");
        service.run_desktop_project_scan(&scan.scan_id).expect("scan should run");

        let snapshot_path = temp_dir.join("snapshot.json");
        fs::write(
            &snapshot_path,
            r#"{
              "schema_version": 1,
              "source_device": {"device_id": "phone", "device_label": "Phone", "platform": "android"},
              "project": {"project_id": "remote", "name": "Remote", "exported_at_ms": 1},
              "assets": [{
                "asset_id": "remote-a",
                "group_id": "remote-g",
                "original_filename": "IMG_4100.JPG",
                "final_filename": "IMG_4100.JPG",
                "normalized_stem": "IMG_4100",
                "original_path": "IMG_4100.JPG",
                "original_parent_path": null,
                "format": "jpeg",
                "size_bytes": 4,
                "capture_at_ms": null,
                "received_at_ms": 1,
                "source_identity": null
              }],
              "groups": [{
                "group_id": "remote-g",
                "display_key": "IMG_4100",
                "source_identity": null,
                "original_parent_path": null,
                "member_asset_ids": ["remote-a"],
                "primary_asset_id": "remote-a",
                "preview_asset_id": "remote-a",
                "has_raw": false,
                "has_jpeg": true,
                "has_video": false
              }],
              "model_evaluations": [],
              "selection_recommendations": [],
              "user_marks": [{"group_id": "remote-g", "favorite": true, "marked": true}]
            }"#,
        )
        .expect("snapshot should write");

        let response = sync_project_snapshot_from_path_blocking(
            &service,
            SyncProjectSnapshotRequest {
                project_id: project.project_id.clone(),
                snapshot_path: snapshot_path.to_string_lossy().to_string(),
            },
        )
        .expect("sync command helper should succeed");

        assert_eq!(response.matched_assets, 1);
        assert_eq!(response.matched_groups, 1);
        assert_eq!(response.applied_user_marks, 1);
        assert_eq!(response.unresolved_records, 0);
    }
```

- [ ] **Step 2: Run desktop command test to verify RED**

Run:

```powershell
cargo test -p camera-connector-desktop sync_project_snapshot_from_path_returns_compact_counts
```

Expected: compile fails because `SyncProjectSnapshotRequest`, response type, and helper do not exist.

- [ ] **Step 3: Add request/response DTOs and helper**

Modify imports in `apps/desktop/src-tauri/src/commands.rs` to include:

```rust
    parse_project_sync_snapshot_json, ProjectSyncSummary,
```

Add near other request/response structs:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct SyncProjectSnapshotRequest {
    pub project_id: String,
    pub snapshot_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncProjectSnapshotResponse {
    pub matched_assets: usize,
    pub unmatched_assets: usize,
    pub ambiguous_assets: usize,
    pub matched_groups: usize,
    pub unmatched_groups: usize,
    pub ambiguous_groups: usize,
    pub applied_user_marks: usize,
    pub imported_model_evaluations: usize,
    pub imported_selection_recommendations: usize,
    pub skipped_records: usize,
    pub unresolved_records: usize,
}
```

Add helper functions near other command helpers:

```rust
fn desktop_project_sync_response(summary: ProjectSyncSummary) -> SyncProjectSnapshotResponse {
    let unresolved_records = summary
        .unmatched_assets
        .saturating_add(summary.ambiguous_assets)
        .saturating_add(summary.unmatched_groups)
        .saturating_add(summary.ambiguous_groups)
        .saturating_add(summary.skipped_records);
    SyncProjectSnapshotResponse {
        matched_assets: summary.matched_assets,
        unmatched_assets: summary.unmatched_assets,
        ambiguous_assets: summary.ambiguous_assets,
        matched_groups: summary.matched_groups,
        unmatched_groups: summary.unmatched_groups,
        ambiguous_groups: summary.ambiguous_groups,
        applied_user_marks: summary.applied_user_marks,
        imported_model_evaluations: summary.imported_model_evaluations,
        imported_selection_recommendations: summary.imported_selection_recommendations,
        skipped_records: summary.skipped_records,
        unresolved_records,
    }
}

fn sync_project_snapshot_from_path_blocking(
    service: &CameraConnectorService,
    request: SyncProjectSnapshotRequest,
) -> Result<SyncProjectSnapshotResponse, DesktopError> {
    let snapshot_json = fs::read_to_string(PathBuf::from(request.snapshot_path))
        .map_err(|error| DesktopError {
            code: "project_sync".to_string(),
            message: format!("project sync snapshot could not be read: {error}"),
        })?;
    let snapshot = parse_project_sync_snapshot_json(&snapshot_json).map_err(desktop_error)?;
    service
        .sync_project_snapshot(&request.project_id, &snapshot)
        .map(desktop_project_sync_response)
        .map_err(desktop_error)
}
```

Add Tauri command near other project commands:

```rust
#[tauri::command]
pub async fn sync_project_snapshot_from_path(
    state: State<'_, DesktopState>,
    request: SyncProjectSnapshotRequest,
) -> Result<SyncProjectSnapshotResponse, DesktopError> {
    let service = state.service.clone();
    tauri::async_runtime::spawn_blocking(move || {
        sync_project_snapshot_from_path_blocking(&service, request)
    })
    .await
    .map_err(|error| DesktopError {
        code: "project_sync".to_string(),
        message: format!("project sync task failed: {error}"),
    })?
}
```

- [ ] **Step 4: Register command**

Modify `apps/desktop/src-tauri/src/lib.rs`:

```rust
            commands::sync_project_snapshot_from_path,
```

Place it after `commands::start_project_scan` or near `commands::get_scan_status`.

- [ ] **Step 5: Run desktop command test to verify GREEN**

Run:

```powershell
cargo test -p camera-connector-desktop sync_project_snapshot_from_path_returns_compact_counts
```

Expected: test passes.

- [ ] **Step 6: Commit Task 4**

Run:

```powershell
git add apps/desktop/src-tauri/src/commands.rs apps/desktop/src-tauri/src/lib.rs
git commit -m "Add desktop project sync snapshot command"
```

## Task 5: Final Verification And No-Schema Guard

**Files:**
- Verify only; do not edit unless checks find issues.

- [ ] **Step 1: Run focused core tests**

Run:

```powershell
cargo test -p camera_connector_core --test project_sync_tests
```

Expected: all project sync tests pass.

- [ ] **Step 2: Run desktop command test**

Run:

```powershell
cargo test -p camera-connector-desktop sync_project_snapshot_from_path_returns_compact_counts
```

Expected: desktop command test passes.

- [ ] **Step 3: Run related desktop scan regressions**

Run:

```powershell
cargo test -p camera_connector_core --test desktop_scan_tests
```

Expected: existing desktop scan tests pass. This protects scan/index behavior after adding sync.

- [ ] **Step 4: Run formatting and package checks**

Run:

```powershell
cargo fmt --all --check
cargo check -p camera_connector_core
cargo check -p camera-connector-desktop
```

Expected: formatting and both package checks pass.

- [ ] **Step 5: Verify no database schema changes were made**

Run:

```powershell
git diff -- core/src/storage/mod.rs
rg -n "project_sync|sync_snapshot|import_session|unresolved" core/src/storage/mod.rs
```

Expected: `git diff` prints nothing for `core/src/storage/mod.rs`; `rg` finds no sync/import schema references in storage.

- [ ] **Step 6: Diff hygiene**

Run:

```powershell
git diff --check
git status --short
```

Expected: no whitespace errors. Status shows only intended project sync source/test/desktop command files if not yet committed.

- [ ] **Step 7: Commit any verification fixes**

If Step 1 through Step 6 required fixes, commit them:

```powershell
git add core/src/project_sync.rs core/src/lib.rs core/src/service.rs core/tests/project_sync_tests.rs apps/desktop/src-tauri/src/commands.rs apps/desktop/src-tauri/src/lib.rs
git commit -m "Verify scan integrated project sync"
```

If no files changed after Task 4, do not create an empty commit.

## Self-Review

- Spec coverage: Tasks 1 through 4 cover local JSON snapshot loading, scan-indexed matching, user mark/model context application, compact transient counts, and desktop command access. LAN discovery remains explicitly deferred by the approved spec.
- No DB schema: The plan never modifies storage schema or adds sync/import tables, columns, sessions, unresolved bindings, or persisted reports. Durable writes reuse `asset_group_user_marks`, `model_evaluations`, and `selection_recommendations`.
- Type consistency: The plan uses `ProjectSyncSnapshot*` for JSON DTOs, `ProjectSyncMatchReport` for transient mapping, `ProjectSyncSummary` for counts, and `SyncProjectSnapshotResponse` for the desktop command response.

# Desktop Project Workbench Design

## Goal

Define the first Windows and macOS desktop product slice for Camera Connector as
a project-scoped scanning, review, and smart-selection workbench.

The desktop app should not start as another receiver-first native application.
Its primary job is to scan an existing local photo library, create or hydrate a
Camera Connector project, generate local preview cache, and run the same
project evaluation and recommendation semantics that Android uses.

The design also connects the existing project package migration protocol to the
desktop workbench. A desktop project can be created from a local folder alone or
from an Android-exported project package plus a local photo root. Both paths end
in the same core project, asset group, model evaluation, selection
recommendation, and user mark model.

## Product Positioning

Desktop is the large-screen project workbench.

Android remains the mobile receiver and field import surface. It owns Android
foreground service behavior, notification permission, and SAF publishing.

Desktop focuses on:

- scanning existing photo folders without moving originals;
- reviewing RAW/JPEG/video groups on a large screen;
- running configured technical assessment, model evaluation, and recommendation
  jobs;
- importing Android project context when the same files already exist in a
  desktop photo library;
- preserving the same project semantics across Android and desktop.

Desktop receiver support can remain available through CLI and future app
surfaces, but it is not the first desktop workbench milestone.

## Decisions

- Use a Tauri desktop shell for the first Windows/macOS app direction.
- Keep Rust core as the source of truth for project state, indexing, evaluation
  settings, analysis jobs, model evaluations, selection recommendations, and
  user marks.
- Treat desktop UI as a thin adapter over core service commands, similar in
  spirit to the Android gateway.
- New desktop projects use in-place indexing by default. The app records local
  file locations and metadata; it does not copy, move, rename, or reorganize
  originals.
- Project evaluation settings drive automation on desktop. Scan completion
  should schedule technical assessment, model evaluation, and burst
  recommendation only according to the selected project settings.
- Project-level recommendation remains manual. Scanning and background drains
  must not create or replace project-scope recommendations automatically.
- Preview cache is a local derived state owned by Camera Connector. It is
  separate from project facts and excluded from cross-device package identity.
- Android project package import remains project-scoped. It maps exported
  package-local ids to local scanned files and groups, then applies model
  evaluation, recommendation, and user mark records through the same project
  model used by desktop-created projects.

## Non-Goals

- Do not implement desktop UI in this spec.
- Do not implement import/export in this spec.
- Do not make desktop the primary receiver path.
- Do not add cloud sync or bidirectional conflict merge.
- Do not require file copying into a managed library.
- Do not implement full RAW decoding or image editing.
- Do not make preview cache records part of the Android package protocol.
- Do not require a cross-platform universal photo id column in SQLite.

## Architecture

```text
Desktop UI (Tauri)
  -> Desktop command gateway
  -> CameraConnectorService
  -> SqliteStore
  -> Scan jobs + preview cache jobs + analysis jobs
  -> Project dashboard/read models
```

The desktop command gateway should expose stable, UI-oriented commands rather
than leaking internal table shapes. The gateway can be implemented inside the
Tauri Rust side and call the existing core service directly.

Suggested command groups:

- `projects`: create, list, rename, archive, select, import package.
- `scan`: start folder scan, read scan status, cancel scan, rescan project.
- `assets`: read project dashboard, asset pages, group details, user marks.
- `preview`: get cached thumbnail or preview path, enqueue preview generation.
- `analysis`: drain jobs, enqueue manual model evaluation, run recommendations.
- `settings`: provider profiles, prompt profiles, project evaluation settings.

Core remains responsible for project correctness. UI commands should not
manually mutate analysis tables or asset grouping state.

## Desktop Project Creation Flow

1. User creates a desktop project.
2. User selects a local photo root.
3. Desktop stores the project and scan root in app-private state.
4. Scanner walks supported media files under the root.
5. Scanner records file-backed asset facts:
   - local path;
   - original filename;
   - normalized stem;
   - format;
   - size bytes;
   - modified time;
   - capture time when available;
   - source root identity.
6. Core groups RAW/JPEG/video assets using the existing normalized-stem
   semantics.
7. Desktop starts background thumbnail generation for all discovered groups.
8. Desktop schedules analysis work according to `ProjectEvaluationSettings`.
9. Project dashboard shows scan progress, preview readiness, technical risk,
   model status, recommendation state, and user marks.

The local photo root should not be treated as an output target. It is an
external library root that the project indexes. Missing or moved files should be
reported as unresolved source files instead of silently deleting project facts.

## Android Package Import Flow

1. User opens a project package exported from Android.
2. Desktop validates `manifest.json` and schema version.
3. User selects a local photo root.
4. Desktop scans supported media files under the selected root.
5. Importer computes package match keys and local file match keys using the
   existing package migration protocol.
6. Importer builds a temporary mapping:

```text
package_asset_id -> local asset or unresolved candidate
package_group_id -> local group or unresolved candidate
```

7. Importer creates or updates a local desktop project.
8. Importer applies package model evaluations, selection recommendations, and
   user marks to matched local groups/assets.
9. Ambiguous, missing, and partially matched package records appear in an import
   report.
10. The resulting project can continue to scan, generate previews, evaluate, and
    recommend using the same desktop workflow as a newly created project.

The package remains a transfer of project context, not a file ownership model.
Package-local ids are references for import mapping. They are not universal
photo identities.

## Project Evaluation Automation

Desktop must use the same project settings semantics as Android:

- `model_evaluation_enabled` controls whether model work can be scheduled.
- `auto_evaluate_on_upload` should be interpreted on desktop as
  `auto_evaluate_on_scan`.
- `auto_burst_recommendation_enabled` controls automatic burst-level
  recommendation after enough model evaluation data exists.
- `project_recommendation_mode` keeps project-level recommendation manual for
  the first desktop workbench.
- Missing provider or API key is explicit. Technical assessment and preview
  generation continue, while model evaluation and model recommendation are
  skipped or disabled with visible setup state.

The scan worker should not bypass these settings. Manual actions may enqueue
model evaluation or recommendation work even when automatic scheduling is off,
as long as provider capability exists.

## In-Place Asset Model

Desktop indexed files should use `StoredObjectLocation` with `local_path`.
Unlike receiver imports, in-place indexed assets are not completed transfers
from a camera upload. They need enough provenance to distinguish scanned local
files from push-imported files.

Use scan-specific source records for desktop indexing instead of pretending
scanned files are receiver transfers. The transfer table should remain the
receiver/import audit trail. Desktop scan records should feed the same
project-scoped asset and group indexing path so the read models stay shared.

The important contract is:

- project assets remain project-scoped;
- groups use the same grouping rules;
- local file paths remain platform-local facts;
- scan roots and cache records do not become cross-device package identity;
- Android package matching uses factual fields, not desktop-only ids.

## Preview Cache

Original files stay in place. Camera Connector owns derived preview cache.

Cache kinds:

- `thumb`: small image for grid display, generated after scan for every
  candidate that can provide a preview.
- `preview`: larger image for detail view and model visual input, generated
  lazily.

Recommended first sizes:

- `thumb`: about 320 px on the long edge.
- `preview`: about 1600 to 2048 px on the long edge.

Recommended source order:

1. RAW+JPEG group: use the JPEG member for group preview.
2. JPEG-only: decode the JPEG.
3. RAW-only: try embedded preview if available; otherwise mark preview
   unavailable.
4. Video: show placeholder in the first milestone; first-frame generation can
   be a later enhancement.

Preview cache key should be deterministic and cheap:

```text
hash(canonical_path, size_bytes, modified_at_ms)
```

This avoids full-file hashing during scan. A later implementation can add a
partial content hash for stronger moved-file detection if needed.

### Preview Cache Table

Preview cache is local derived state, not project fact. Keep it in a separate
table so UI and workers can answer whether a cache entry is ready, stale,
failed, or missing without probing the filesystem repeatedly.

Minimal table shape:

```text
asset_preview_cache
  asset_id
  cache_kind               thumb | preview
  source_location_kind     local_path
  source_location
  source_size_bytes
  source_modified_at_ms
  cache_key
  cache_path
  width
  height
  status                   ready | stale | failed | missing_source
  error
  generated_at_ms
```

Rules:

- If source path, size, or modified time changes, mark existing cache rows stale.
- If the source file is missing, mark rows `missing_source`.
- If preview extraction fails, record `failed` with an error and avoid tight
  retry loops.
- Deleting cache files should be recoverable by regenerating from source files.
- Cache rows and cache files are not exported in project packages.

## Scan Status

Desktop needs visible scan progress because large photo folders can take time.

Suggested scan status fields:

```text
scan_id
project_id
root_path
phase          queued | scanning | indexing | previewing | completed | failed | cancelled
files_seen
assets_indexed
groups_updated
thumbs_queued
thumbs_ready
started_at_ms
updated_at_ms
message
```

The first implementation can keep only one active scan per project. Rescan
should reconcile newly found, changed, missing, and unchanged files without
destroying model evaluations or user marks for still-matched groups.

## Error Handling

- Missing photo root: keep project visible and mark source unavailable.
- Permission denied while scanning: stop scan with an explicit root-level error.
- Individual unreadable file: skip the file, record a scan warning, continue.
- Unsupported format: record as unsupported or ignore according to existing
  object format rules.
- Preview decode failure: mark preview cache `failed`; keep the asset indexed.
- Package schema mismatch: reject import with a version error.
- Package match ambiguity: keep unresolved until user confirms a match.
- Provider not configured: continue scan and technical work; disable or skip
  model work with visible setup state.

## Testing Strategy

Core tests:

- desktop project creation indexes local files without copying originals;
- scanner groups RAW/JPEG/video by normalized stem;
- rescan preserves user marks, model evaluations, and recommendations for
  unchanged matched groups;
- missing files are reported without deleting project records;
- preview cache rows become stale when size or modified time changes;
- failed preview extraction does not block asset display;
- Android package import maps records to scanned local assets using the ordered
  matching policy;
- ambiguous package matches produce an import report instead of attaching data.

Desktop gateway tests:

- Tauri command adapters call service methods and return UI-friendly JSON;
- scan status can be polled while a scan is running;
- dashboard includes preview readiness and unresolved source state.

Manual verification:

- create a desktop project from a local RAW+JPEG folder;
- verify grid thumbnails generate in the background;
- open detail and verify preview generation is lazy;
- enable provider settings and project evaluation settings, then verify model
  evaluation and burst recommendation follow project settings;
- import an Android package against a local photo folder and review matched,
  unmatched, and ambiguous records.

## Milestones

### Milestone 1: Core Desktop Scan Foundation

- Add in-place desktop scan source support.
- Index local files into project assets and groups.
- Add scan status and minimal rescan behavior.
- Add preview cache table and deterministic cache path contract.

### Milestone 2: Preview Worker And Desktop Gateway

- Generate all thumbnails after scan.
- Generate detail previews lazily.
- Expose project, scan, preview, and dashboard commands for Tauri.
- Keep receiver runtime out of the first desktop UI milestone.

### Milestone 3: Project Evaluation Automation

- Map `auto_evaluate_on_upload` to desktop scan completion.
- Schedule technical assessment and model evaluation according to project
  settings.
- Keep project recommendation manual.
- Surface no-provider setup state.

### Milestone 4: Android Package Import

- Implement package reader and schema validation.
- Scan local root and compute match candidates.
- Apply model evaluations, recommendations, and user marks to matched groups.
- Report unresolved and ambiguous records.

### Milestone 5: Desktop UI Polish

- Build the Tauri project workbench UI.
- Add project list, scan setup, photo grid, detail review, filters, settings,
  and import report surfaces.
- Add platform packaging for Windows and macOS.

## Open Questions For Implementation Planning

- Which image library should generate JPEG thumbnails and embedded RAW previews
  in Rust for the first milestone?
- Scan roots should be stored as scan source records and may also be surfaced in
  project settings as the current default root.
- How much of the Android `CoreGateway` DTO shape should be mirrored by the
  desktop Tauri command JSON?
- The first cache policy should keep all generated thumbnails and previews until
  the user clears or regenerates cache; automatic quota eviction can wait.

These questions are implementation-planning details. They do not change the
product decision that desktop is an in-place project workbench with local
preview cache and shared project evaluation semantics.

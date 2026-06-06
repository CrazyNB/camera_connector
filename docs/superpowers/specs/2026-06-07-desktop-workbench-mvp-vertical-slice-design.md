# Desktop Workbench MVP Vertical Slice Design

## Goal

Build the first usable desktop workbench MVP as a real end-to-end product slice:
create a desktop project, choose a local photo root, scan local files into a
project-scoped asset model, evaluate groups, generate recommendations, and show
the results in a desktop UI.

This slice must use formal desktop scan semantics by expanding `transfers` into
the project asset acquisition event log. Receiver FTP/SFTP transfers remain
network acquisition events. Desktop scan records become local-library
acquisition events with `protocol = desktop_scan`.

## Product Scope

The MVP proves the desktop product shape:

- create and select a project from the desktop app;
- select one local folder as the project scan root;
- scan supported media files in place without moving originals;
- write discovered files into project assets and RAW/JPEG/video groups through
  `desktop_scan` transfer records;
- show scan progress and errors;
- show a large-screen asset grid and group detail view;
- run technical assessment, model evaluation, and burst recommendation through
  existing project evaluation semantics;
- keep project-level recommendation manual;
- show recommendation, score, status, and user mark state in the desktop UI.

The MVP is allowed to add core storage and service APIs when they are required
for correct desktop semantics. It should keep those changes narrow and avoid
large storage rewrites that are not needed for the end-to-end slice.

## Non-Goals

- Do not introduce a separate scan source model for the MVP.
- Do not introduce a duplicate per-file desktop scanned assets table.
- Do not use receiver-specific UI labels for desktop scan transfer records.
- Do not implement Android package import in this slice.
- Do not implement full preview cache tables in this slice.
- Do not implement automatic quota eviction for cached files.
- Do not implement multiple scan roots per project.
- Do not implement full moved-file reconciliation.
- Do not make project-scope recommendations automatic.
- Do not turn the desktop app into a receiver-first application.

## Architecture

```text
Tauri desktop app
  -> desktop command gateway
  -> CameraConnectorService
  -> SqliteStore
  -> desktop scan runs
  -> transfers with protocol = desktop_scan
  -> project assets and asset groups
  -> analysis jobs and recommendation read models
```

The desktop UI should depend on gateway DTOs, not on database tables. The
gateway exposes commands shaped for the workbench:

- `create_project`
- `list_projects`
- `select_project`
- `start_project_scan`
- `get_scan_status`
- `get_project_asset_page`
- `get_project_group_detail`
- `save_group_user_marks`
- `drain_analysis_jobs`
- `evaluate_asset_groups`
- `recommend_burst_group`
- `generate_project_recommendation`
- `get_project_dashboard`

The gateway may be implemented in the Tauri Rust side and call core directly.
Core remains responsible for project correctness, grouping, settings, analysis
jobs, recommendations, and user marks.

## Desktop Scan Acquisition Model

Desktop scan is a first-class acquisition path.

The MVP does not add a separate `sources` model. A scan run records the
batch-level operation. Each discovered supported file is recorded as a
`transfers` row with `protocol = desktop_scan`, then indexed through the
existing project asset and group path.

```text
desktop_scan_runs
  scan_id
  project_id
  root_path
  root_label
  phase
  files_seen
  assets_indexed
  groups_updated
  started_at_ms
  updated_at_ms
  completed_at_ms
  error

transfers
  transfer_id              desktop-scan:<stable_root_key>:<stable_file_key>
  project_id
  protocol                 desktop_scan
  status                   completed | failed
  original_path            relative path under the scanned root
  final_filename
  final_location           local_path absolute path
  size_bytes
  source_name              root label
  started_at_ms            scan started time
  completed_at_ms          file indexed time

assets
  existing project asset fields
  source_status            available | missing | changed
  source_modified_at_ms
  last_seen_scan_id
```

Project groups still use the same normalized-stem grouping rules as Android and
receiver assets. `StoredObjectLocation::local_path` remains the location
representation for local source files.

This keeps the MVP model small:

- no separate `desktop_scan_sources` table;
- no separate `desktop_scanned_assets` table;
- no nullable `assets.transfer_id` migration;
- scan provenance remains attached to the existing transfer-backed asset path.

## Scan Status

The MVP keeps one active scan per project.

Supported phases:

```text
queued | scanning | indexing | completed | failed | cancelled
```

The UI should poll `get_scan_status` and show:

- current phase;
- files seen;
- assets indexed;
- groups updated;
- the latest message or error.

The scan worker should continue past individual unreadable or unsupported files
when possible. A root-level permission failure should fail the scan with a
visible error.

## Missing And Changed Files

The MVP must not delete project facts simply because a source file is missing.

`assets.source_status` should support:

```text
available | missing | changed
```

For the first slice:

- newly scanned files create or update a `desktop_scan` transfer and become
  `available`;
- unchanged matched files stay `available`;
- a previously indexed path that is absent during a scan becomes `missing`;
- a previously indexed desktop scan asset whose size or modified time changed
  becomes `changed`;
- user marks, model evaluations, and recommendations remain attached to still
  matched project groups.

The desktop UI only needs to display missing or changed state. It does not need
to offer manual remapping in this MVP.

## Evaluation And Recommendation Flow

Scan completion should schedule or enable work according to
`ProjectEvaluationSettings`:

- technical assessment may run for indexed groups;
- `model_evaluation_enabled` controls model evaluation scheduling;
- Android's `auto_evaluate_on_upload` is interpreted as
  `auto_evaluate_on_scan`;
- `auto_burst_recommendation_enabled` controls automatic burst recommendation;
- project-level recommendation remains manual.

If no model provider or API key is configured, the UI must still show scanned
assets and technical state. Model evaluation and model recommendation actions
should show an explicit setup-disabled state.

Manual actions may run model evaluation, burst recommendation, or project
recommendation as long as provider capability exists.

## Desktop UI MVP

The desktop app starts as the actual workbench, not a marketing screen.

Required views:

- Project sidebar: create, list, and select projects.
- Scan setup: choose a local folder and start scan for the selected project.
- Scan progress strip: phase, counts, and error state.
- Asset grid: grouped assets with filename, format badges, source state,
  technical status, model score, recommendation status, and user mark badges.
- Group detail: all files in the group, local path, source state, technical
  findings, model summary, recommendation rationale, and user mark controls.
- Settings panel: provider setup status and project evaluation settings needed
  by the MVP.

The UI may use local file paths for thumbnails only when the platform permits
safe display. A formal preview cache can be added later; the MVP may show file
type placeholders for RAW and video groups if direct preview generation is not
ready.

## Error Handling

- Missing selected root: keep the project visible and show the root as
  unavailable.
- Permission denied at root: fail the scan with a root-level error.
- Individual unreadable file: record a warning and continue scanning.
- Unsupported format: skip or record according to existing object format rules.
- Changed file: mark source status as `changed`; keep project facts.
- Missing file: mark source status as `missing`; keep project facts.
- Provider not configured: disable model actions with setup guidance while
  leaving scan and asset browsing usable.

## Testing Strategy

Core tests:

- desktop scan indexes local files by creating `desktop_scan` transfer records;
- desktop scan records do not appear as receiver FTP/SFTP activity in desktop
  UI summaries;
- RAW/JPEG/video files with the same normalized stem form one project group;
- scan status moves through scanning and completed phases;
- missing files are marked without deleting project assets or user marks;
- changed files are marked using size or modified time;
- scan completion obeys project evaluation settings;
- automatic drains do not create project-scope recommendations.

Desktop gateway tests:

- project commands return UI-friendly DTOs;
- scan command creates a scan run and `desktop_scan` transfers;
- asset page includes source status state;
- analysis commands call existing core evaluation and recommendation paths.

Manual verification:

- create a desktop project from a local folder containing RAW+JPEG pairs;
- confirm the app shows grouped files in the grid;
- confirm scan progress reaches completed;
- configure provider settings and run model evaluation;
- run burst recommendation and see the recommended asset in the grid/detail;
- run manual project recommendation and confirm it is not created
  automatically by scan completion;
- delete or rename one indexed file, scan again, and confirm the group remains
  visible with missing or changed source state.

## Implementation Boundary

The first implementation plan should start with the smallest formal core
surface that lets the Tauri app complete the product flow:

1. core desktop scan run storage and `desktop_scan` transfer indexing APIs;
2. desktop command gateway;
3. Tauri app shell and workbench UI;
4. analysis and recommendation command wiring;
5. manual verification on a local sample folder.

Any core changes should be additive where possible. Existing Android, receiver,
transfer, model evaluation, and recommendation behavior should remain
unchanged.

# Architecture

Camera Connector is a shared-core, multi-shell application. The Rust core owns
product semantics and durable state. Desktop, Android, and CLI code are adapters
that render, invoke, or validate those shared semantics.

## Runtime Shape

```text
Desktop TypeScript UI
  -> Tauri commands
    -> camera_connector_core::CameraConnectorService
      -> SQLite / receiver runtime / analysis / scan / sync

Android Compose UI
  -> CoreGateway
    -> NativeCoreGateway / JNI
      -> core-ffi::MobileCore
        -> camera_connector_core::CameraConnectorService

CLI
  -> CameraConnectorService / CameraConnectorRuntime
```

## Semantic Boundaries

### Receiver Facts

Owns how files arrive.

Files:

- `core/src/push/`: FTP and engineering SFTP receiver implementations.
- `core/src/receive/`: transfer logs, connected devices, receive sinks, and
  storage locations.
- `core/src/runtime.rs`: start, stop, status, and stale listener detection.
- `core/src/service/receiver.rs`: service-level receiver settings and views.

This layer records who connected, which protocol was used, what was uploaded,
and whether the transfer succeeded. It should not decide which photos are good
or which project assets are selected.

### Asset Facts

Owns what imported files are.

Files:

- `core/src/model/`: object formats, received assets, asset groups, and marks.
- `core/src/storage/asset_index.rs`: transfer-to-asset indexing and duplicates.
- `core/src/storage/asset_groups.rs`: query, filter, sort, and summarize groups.
- `core/src/storage/bursts.rs` and `burst_manual.rs`: burst grouping facts and
  user corrections.

This layer groups RAW/JPEG/video by normalized stem, records duplicate state,
and exposes asset-group read models.

### Project Scope

Owns the user-created work boundary.

Files:

- `core/src/storage/projects.rs`
- `core/src/service/projects.rs`
- `core/src/service/desktop_scan.rs`
- `core/src/desktop_scan.rs`

Every import, scan, dashboard, and evaluation belongs to an explicit project.
There is no default system project fallback.

### Human Decisions

Owns independent human judgment.

Files:

- `core/src/storage/asset_index.rs`: asset group user marks.
- `core/src/storage/burst_manual.rs`: manual burst edits.
- `core/src/storage/mod.rs`: asset group delete and LAN guest marks.
- Android UI files under `apps/android/app/src/main/java/com/cameraconnector/app/ui/`
- Desktop action files such as `apps/desktop/src/appActions.ts`

Favorites, marked state, guest marks, manual burst edits, and deletion are human
state. They must not mutate model-evaluation or recommendation rows.

### Local Technical Assessment

Owns objective local risk and gate context.

Files:

- `core/src/analysis/technical.rs`
- `core/src/storage/subject_assessments.rs`
- `apps/desktop/src-tauri/src/desktop_cv.rs`
- `apps/android/app/src/main/java/com/cameraconnector/app/service/SmartSelectionAnalysisWorker.kt`

This layer records blur, clipping, noise, face/portrait risk, and technical gate
state. It is not the final photographic score.

### Model Evaluation

Owns provider-backed photographic evaluation.

Files:

- `core/src/analysis/model_eval.rs`
- `core/src/analysis/model_eval/`
- `core/src/service/model_providers.rs`
- `core/src/service/prompt_packs.rs`
- `core/src/storage/analysis.rs`
- `core/src/storage/analysis_jobs.rs`

This layer stores provider settings, prompt packs, model scores, tiers,
summaries, and analysis-job state. Missing provider/API key must leave uploads,
grouping, publishing, thumbnails, and local technical assessment working.

### Selection Recommendation

Owns model recommendation output.

Files:

- `core/src/analysis/recommendation.rs`
- `core/src/service/analysis_recommendations.rs`
- `core/src/storage/analysis.rs`

`selection_recommendations` stores model recommendations only. Burst
recommendations may be automatic when enabled; project recommendations are
manual actions.

### Publish And Final Storage

Owns staged bytes, final platform storage, and retry state.

Files:

- `core/src/storage/pipeline.rs`
- `core/src/storage/publish.rs`
- `core/src/service/publish.rs`
- `apps/android/app/src/main/java/com/cameraconnector/app/storage/`

The receiver stages bytes first. A publish worker claims pending work, writes to
the final object store, records a `StoredObjectLocation`, and exposes failed
write state for retry.

### Sharing And Project Sync

Owns project state moving between devices or viewers.

Files:

- `core/src/lan_share.rs`
- `core/src/project_sync.rs`
- `apps/android/app/src/main/java/com/cameraconnector/app/share/`
- `apps/desktop/src/lanProjectSync.ts`
- `apps/desktop/src-tauri/src/lan_discovery.rs`

LAN share and project sync operate on project facts; they are not the receiver
upload path.

### Platform Shells

Own UI, lifecycle, permissions, and platform APIs.

Desktop:

- `apps/desktop/src/`: TypeScript workbench state, rendering, previews, and UI
  interactions.
- `apps/desktop/src-tauri/src/`: Tauri commands, thumbnailing, LAN discovery,
  and desktop CV adapters.

Android:

- `apps/android/app/src/main/java/com/cameraconnector/app/MainActivity.kt`
- `apps/android/app/src/main/java/com/cameraconnector/app/core/`
- `apps/android/app/src/main/java/com/cameraconnector/app/service/`
- `apps/android/app/src/main/java/com/cameraconnector/app/storage/`
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/`

CLI:

- `tools/cli/src/main.rs`
- `tools/cli/src/cli_args.rs`
- `tools/cli/src/cli_support.rs`

Platform shells should not reimplement transfer logs, grouping, duplicate
policy, account identity, recommendation semantics, or storage schema rules.

## Main Data Tables

The SQLite schema lives in `core/src/storage/schema.rs`. Important tables:

- `projects`
- `project_evaluation_settings`
- `receiver_accounts`
- `connected_devices`
- `receiver_status`
- `transfers`
- `desktop_scan_runs`
- `asset_groups`
- `assets`
- `publish_queue`
- `background_jobs`
- `burst_groups`
- `burst_group_members`
- `asset_group_user_marks`
- `lan_share_sessions`
- `lan_share_guest_marks`
- `technical_assessments`
- `model_evaluations`
- `selection_recommendations`

## Ownership Rules

- Put durable product behavior in `core`.
- Put app-facing orchestration in `CameraConnectorService`.
- Put platform permissions, foreground services, SAF, Tauri command glue, and UI
  state in platform shells.
- Prefer JSON/DTO boundaries at FFI and Tauri edges.
- Do not let UI code infer business meaning from table names or raw transfer
  logs when a service read model exists.
- Do not collapse technical risk, model score, recommendation, and human marks
  into one status field.

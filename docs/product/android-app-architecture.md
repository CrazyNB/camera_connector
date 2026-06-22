# Android App Architecture

## 1. Decision

Camera Connector Android uses **Kotlin + Jetpack Compose + Material 3** as the native shell, with the existing **Rust core** embedded behind a small gateway boundary.

This is the first mobile target. It should prove the product loop on Android before any cross-platform abstraction is introduced.

The repository-level architecture and semantic module ownership live in
`../architecture.md`.

## 2. Why Native Android

Camera Connector depends on Android-native surfaces that are central to the product, not incidental implementation details:

- A long-running FTP receiver needs an Android foreground service and a persistent notification.
- Output storage needs platform-specific handling through MediaStore or the Storage Access Framework.
- The app must expose the active host, port, protocol, account identity, transfer health, and failure diagnostics while the receiver is running.
- Network behavior depends on hotspot/LAN state, local IP selection, and permissions.

Flutter, React Native, or Tauri Mobile would still require native modules for these capabilities. For the first Android slice, that extra bridge would slow down receiver validation.

## 3. Module Shape

```text
apps/android/
  app/
    MainActivity
    Compose UI shell
    ReceiverForegroundService
    Android storage gateway
    Core gateway interface
    NativeMobileCore Kotlin adapter
    NativeCoreGateway dashboard adapter

core/
  Existing Rust service, receiver, transfer log, accounts, grouping

core-ffi/
  Mobile-facing Rust facade, cdylib/staticlib target, C ABI and JSON DTO boundary

future generated bindings/
  UniFFI or JNI Kotlin bindings over core-ffi
```

The current implementation uses JNI through `core-ffi`. Future generated
bindings must preserve the same gateway boundary instead of letting Compose bind
directly to core storage or Rust internals.

## 3.1 Semantic Responsibilities

Android owns platform mechanics:

- Activity, navigation, Compose rendering, and UI state.
- Foreground service lifecycle and notification permission.
- SAF document tree selection and persisted URI permission.
- Local Android preview decoding and publish worker execution.
- User-facing camera setup copy and diagnostics presentation.

The Rust core owns product semantics:

- Receiver facts: runtime status, authentication, connected devices, transfers.
- Asset facts: final storage locations, RAW/JPEG/video roles, grouping,
  duplicates, and source metadata.
- Project scope: active project, dashboard reads, scans, and sync.
- Human decisions: favorite, marked, guest marks, manual burst edits, and delete.
- Local technical assessment, model evaluation, selection recommendation, and
  background analysis jobs.
- Publish queue claim/retry state.

Android should map these concepts into UI models without collapsing them into a
single "smart selection" or "import status" field.

## 4. Android Responsibilities

The Android layer owns:

- Activity navigation and Compose rendering.
- Foreground service lifecycle.
- Android notification channels and notification permission guidance.
- Storage permission request and persisted document-tree URI.
- App-private config/state file locations.
- User-facing setup instructions for camera-side FTP profiles.

The Android layer should not reimplement:

- Transfer log semantics.
- RAW/JPEG/video grouping.
- Duplicate detection.
- Account identity rules.
- Receiver protocol behavior.

Those remain core responsibilities.

## 5. Core Gateway Boundary

UI code talks to a Kotlin `CoreGateway` interface:

- `observeDashboard()`
- `observeProjects()`
- `createProject()`
- `setActiveProject()`
- `archiveProject()`
- `restoreProject()`
- `splitBurstMember()`
- `createManualBurstGroup()`
- `startReceiver()`
- `stopReceiver()`
- `saveReceiverSettings()`
- `saveDeviceAccount()`
- `removeDeviceAccount()`
- `retryFailedPublishes()`
- `loadModelProviderSettings()` / `saveModelProviderSettings()`
- `loadProjectEvaluationSettings()` / `saveProjectEvaluationSettings()`
- `loadPromptPacks(projectId)` / `loadGlobalPromptPacks()`
- `createGlobalPromptPack()`
- `forkPromptPack()`
- `savePromptPack()`
- `generateProjectRecommendation()`
- `latestProjectRecommendationRunStatus()`
- `shouldScheduleSubjectAssessment()`
- `saveSubjectAssessment()` / `loadSubjectAssessments(projectId, groupIds)`

The preview gateway remains only for lightweight IDE rendering. Product
verification uses the native gateway and the embedded Rust core. This keeps the
UI from depending directly on FFI-generated types and allows the product shell
to evolve without duplicating core logic.

The Rust side now has a `camera-connector-ffi` crate that exposes a mobile-facing `MobileCore` facade. Its first contract is JSON-based so it can be verified in the normal Rust workspace before Android SDK/NDK builds are available.

The same crate also exports a narrow C ABI:

- `camera_connector_mobile_core_create`
- `camera_connector_mobile_core_destroy`
- `camera_connector_mobile_core_free_string`
- `camera_connector_mobile_core_config_path`
- `camera_connector_mobile_core_default_state_dir`
- `camera_connector_mobile_core_create_project_json`
- `camera_connector_mobile_core_list_projects_json`
- `camera_connector_mobile_core_set_active_project_json`
- `camera_connector_mobile_core_rename_project_json`
- `camera_connector_mobile_core_archive_project_json`
- `camera_connector_mobile_core_restore_project_json`
- `camera_connector_mobile_core_active_project_json`
- `camera_connector_mobile_core_project_dashboard_json`
- `camera_connector_mobile_core_project_group_assets_json`
- `camera_connector_mobile_core_split_burst_member_json`
- `camera_connector_mobile_core_create_manual_burst_group_json`
- `camera_connector_mobile_core_claim_next_publish_item_json` (internal write-queue API)
- `camera_connector_mobile_core_mark_publish_completed_json` (internal write-queue API)
- `camera_connector_mobile_core_complete_publish_json` (internal write-queue API)
- `camera_connector_mobile_core_mark_publish_failed_json` (internal write-queue API)
- `camera_connector_mobile_core_release_failed_publish_retries_json`
- `camera_connector_mobile_core_save_receiver_settings_json`
- `camera_connector_mobile_core_save_device_account_json`
- `camera_connector_mobile_core_remove_device_account_json`
- `camera_connector_mobile_core_drain_analysis_jobs_with_provider_configured_json`
- `camera_connector_mobile_core_assess_asset_group_preview_with_provider_configured_json`
- `camera_connector_mobile_core_model_provider_settings_json`
- `camera_connector_mobile_core_save_model_provider_settings_json`
- `camera_connector_mobile_core_project_evaluation_settings_json`
- `camera_connector_mobile_core_save_project_evaluation_settings_json`
- `camera_connector_mobile_core_prompt_packs_for_project_json`
- `camera_connector_mobile_core_global_prompt_packs_json`
- `camera_connector_mobile_core_fork_prompt_pack_json`
- `camera_connector_mobile_core_save_prompt_pack_json`
- `camera_connector_mobile_core_save_prompt_version_json`
- `camera_connector_mobile_core_generate_project_recommendation_json`
- `camera_connector_mobile_core_latest_project_recommendation_run_status_json`
- `camera_connector_mobile_core_should_schedule_subject_assessment_json`
- `camera_connector_mobile_core_save_subject_assessment_json`
- `camera_connector_mobile_core_subject_assessments_for_asset_groups_json`
- `camera_connector_mobile_core_start_receiver_json`
- `camera_connector_mobile_core_stop_receiver_json`

The consumable C contract lives in:

```text
core-ffi/include/camera_connector_mobile.h
```

Every string-returning call returns a JSON envelope:

```json
{"ok":true,"value":{},"error":null}
```

or:

```json
{"ok":false,"value":null,"error":"message"}
```

Android JNI bindings keep this envelope at the native boundary, then map it into typed Kotlin state before it reaches Compose screens.

The Android source now has the Kotlin side of that bridge:

- `NativeMobileCore` owns the native handle, loads `camera_connector_ffi`, calls external functions, and unwraps the JSON envelope.
- `NativeCoreGateway` ensures an active project and adapts project dashboard JSON into the existing `CoreGateway` model used by Compose.
- Project correction actions stay inside the active project: Android can split members out of a burst group or manually merge multiple asset groups/burst containers into one burst group while Rust persists the grouping decision transactionally.
- `CoreGatewayFactory` chooses either `PreviewCoreGateway` or `NativeCoreGateway` through `BuildConfig.USE_NATIVE_CORE`; release-facing verification builds use the native gateway. `PreviewCoreGateway` is an empty UI shell with no seeded project/account data, and native failures only fall back to preview when `cameraConnector.nativeCoreFallbackToPreview=true` is set explicitly.
- `ReceiverServiceController` sends receiver start/stop commands to `ReceiverForegroundService`.
- `ReceiverForegroundService` owns the long-running native receiver lifecycle and foreground notification.
- The foreground notification deep-links back into `MainActivity` and exposes a Stop action backed by an immutable service `PendingIntent`.
- Receiver service lifecycle events are logged through the `CameraConnectorReceiver` tag so adb diagnostics can separate app receiver failures from generic Android runtime crashes.
- Connected-device smoke testing builds, installs, launches, verifies package presence, and collects adb diagnostics through `scripts\smoke_android_device.ps1`.
- Emulator FTP verification covers native account creation, receiver start, passive upload of RAW/JPEG pairs, project photo-grid display, photo detail navigation, transfer rows, and adb diagnostics through `scripts\verify_android_emulator_ftp_upload.ps1`.
- The same upload verifier accepts `-RealAssetDirectory`, selects a matching RAW/JPEG filename stem from the folder, and uploads the real bytes. It has been exercised with Nikon `.NEF + .JPG` files from `D:\ps\Photos\2026\5\5.4`.
- Device account setup flows through the same gateway: Compose collects device name, camera login username, and a write-only password; `NativeCoreGateway` passes that password to the Rust core so the persisted config stores the core-generated password hash rather than plaintext.
- Model provider setup also flows through the gateway. Provider profiles are app-level configuration: profile id, provider kind, label, base URL, model name, send mode, batch size, and API key are persisted in the app-private core config JSON, not in SQLite project tables. Core DTOs returned to UI expose `api_key_configured` and optional key alias rather than echoing secret text.
- Project intelligence settings are project-scoped. Global provider/model defaults can prefill new projects, but `NativeCoreGateway` must not apply a global default change by silently changing existing project evaluation settings.
- Prompt packs exposed through the gateway are global, package-grouped, Markdown-backed photographic preference resources. Built-in packs are read-only; editing one creates a user-owned copy under an app-private `prompt-packs/<package>/...` folder. Projects only select a prompt pack id; the model request protocol, task instructions, JSON schema, and output parsing remain system-owned.
- Project scene is a first-level project setting because it affects local portrait/action/landscape risk context and model evaluation semantics. Technical risk thresholds live in the project intelligence secondary panel.
- Project recommendation is a manual gateway action. Upload drains, background analysis drains, and burst-stable processing can evaluate assets or burst groups, but they must not create project-scope recommendations.
- No-key behavior is provider-aware: upload, thumbnail generation, grouping, publishing, and local technical CV continue; model evaluation and manual project recommendation are skipped or disabled when provider capability is missing. A development `local_stub` result remains `local_stub` in core/API data, while user-facing Android labels render it as local analysis instead of exposing the raw enum.
- Semantic boundaries are deliberately separate: `technical_assessments` contains local CV risk/gate context, `model_evaluations` contains photographic model scores and summaries, `selection_recommendations` contains model recommendations only, and user favorite/mark state is independent human state. Accepting, clearing, or favoriting a photo must not mutate algorithm recommendation rows.
- Portrait subject assessment is a Core/FFI storage and interpretation contract, surfaced through subject assessment APIs. It is scheduled only for projects with `scene_profile = portrait`, acts as risk/gate/context for evaluation, and does not require Android to add an ML dependency in the current slice.
- Receiver setup also flows through the gateway: the project receiver panel edits the camera-facing setup IP and one unified camera-facing port, then starts/stops the foreground receiver directly; there is no separate save button. Android keeps FTP as the visible route in the current slice and renders future STC-style mode as disabled. The native listener can still bind through core defaults such as `0.0.0.0`, but Android should not expose bind-host tuning as a primary user setting.
- The native dashboard includes both runtime status and saved receiver settings. Android uses runtime status while the receiver is running, and falls back to saved settings while stopped so protocol/host/port changes are reflected immediately in the project receiver panel and collapsed running status.
- Emulator UI verification covers the two-level shell, Project Management startup surface, photo-first project workspace, global Projects/Accounts/Settings destinations, Settings diagnostics, account-gated receiver start, switching the emulator-only bind host to `0.0.0.0`, and receiver start/stop.
- Transfer diagnostics are mapped from the native dashboard into Compose: transfer counts remain visible and recent failed transfers show the core-provided virtual display path plus error text.
- Account connection diagnostics are also mapped: Accounts and diagnostics can show active connection count, latest remote endpoint, last seen time, and last disconnected time without treating IP address as account identity.
- Receiver runtime diagnostics are mapped as well: the receiver panel and collapsed status show phase, authentication mode, account count, and core failure message so service start failures are visible in-app.
- Android directory selection is wired at the platform boundary: `MainActivity` launches SAF document tree selection, `AndroidStorageGateway` persists the URI permission and display label, and the Settings output row shows the selection. Native imports stage into app-private storage first, then the Android write worker writes to the selected SAF tree and records `document_uri`; without a selected tree it falls back to app-private file storage. The worker resolves the final write target per item, and core defers failed write retries through `next_attempt_at_ms`, so storage permission loss remains visible without hammering the selected target while the receiver keeps running. When the user reselects or reauthorizes the output directory, Android releases failed write retry delays for the active project so queued staged files can recover immediately. Receiver status and Settings diagnostics can trigger the same project-scoped retry release path and start a one-shot write drain, so retry works even when the receiver loop is not already running.
- Android maps the native dashboard `publish_queue` summary into its UI model as write-queue state. The receiver panel and collapsed status surface pending or failed writes so storage permission loss and other recoverable write failures are visible without inspecting logs.
- The native dashboard also exposes recent project-scoped write failures. Android surfaces those failures through receiver status and Settings diagnostics so output permission problems stay actionable without a separate project Transfers page.
- UI actions that call the gateway are wrapped with local error handling. Native exceptions from start, stop, receiver settings, or account save operations appear as a dismissible local error card on the relevant project, account, or settings surface.
- UI actions also publish an in-flight label while native gateway calls are running. Related controls are disabled during that window to avoid duplicate start, stop, settings, or account operations.
- The Android shell follows a two-level project model. Global navigation owns Projects, Accounts, and Settings. Settings owns diagnostics as a secondary page. Projects opens the Figma-aligned Project Management surface by default; entering a project opens the photo-first workspace. Receiver start/stop and receiver setup live at the top of that project workspace so imports always have an explicit project scope.
- The Project Photos UI is photo-first: grid density is configured from Settings, tile metadata stays compact, JPEG previews are used when available, and group detail opens by tapping the preview. Long press enters selection mode for bulk delete, model evaluation, burst split, and manual burst merge. Newly received and unresolved material is represented through project-scoped filters and diagnostics rather than a separate receive tab.

The Rust side now exports JNI symbols for Kotlin `NativeMobileCore`:

- `create`
- `destroy`
- `projectDashboardJson`
- `projectAssetGroupPageJson`
- `createLanShareSessionJson`
- `stopLanShareSessionJson`
- `lanShareAssetGroupPageJson`
- `setLanShareGuestMarkJson`
- `projectGroupAssetsJson`
- `deleteProjectGroupJson`
- `setAssetGroupUserMarksJson`
- `claimNextPublishItemJson`
- `completePublishJson`
- `markPublishFailedJson`
- `releaseFailedPublishRetriesJson`
- `drainAnalysisJobsWithProviderConfiguredJson`
- `enqueueModelEvaluationForAssetGroupsJson`
- `evaluateAssetGroupsWithModelInputsJson`
- `recommendBurstGroupWithCandidateVisualsJson`
- `assessAssetGroupPreviewWithProviderConfiguredJson`
- `splitBurstMemberJson`
- `createManualBurstGroupJson`
- `modelProviderSettingsJson`
- `modelProviderSettingsListJson`
- `saveModelProviderSettingsJson`
- `deleteModelProviderSettingsJson`
- `projectEvaluationSettingsJson`
- `saveProjectEvaluationSettingsJson`
- `PromptPacksForProjectJson`
- `globalPromptPacksJson`
- `forkGlobalPromptPackJson`
- `createGlobalPromptPackJson`
- `saveGlobalPromptPackJson`
- `deleteGlobalPromptPackJson`
- `deleteGlobalPromptPackageJson`
- `forkPromptPackJson`
- `savePromptPackJson`
- `generateProjectRecommendationJson`
- `generateProjectRecommendationWithCandidateVisualsJson`
- `latestProjectRecommendationRunStatusJson`
- `shouldScheduleSubjectAssessmentJson`
- `saveSubjectAssessmentJson`
- `subjectAssessmentsForAssetGroupsJson`
- `createProjectJson`
- `listProjectsJson`
- `setActiveProjectJson`
- `renameProjectJson`
- `archiveProjectJson`
- `deleteProjectJson`
- `restoreProjectJson`
- `activeProjectJson`
- `saveReceiverSettingsJson`
- `saveDeviceAccountJson`
- `removeDeviceAccountJson`
- `startReceiverJson`
- `stopReceiverJson`

The JNI shim reuses the same `MobileCore` facade as the C ABI, so Android-specific binding code does not duplicate receiver, account, dashboard, transfer, or receiver lifecycle logic.

Android native packaging is produced by:

```text
scripts/build_android_native.ps1
```

The script builds `core-ffi` for `aarch64-linux-android` and `x86_64-linux-android`, then copies `libcamera_connector_ffi.so` into the matching `apps/android/app/src/main/jniLibs` ABI folders.

Android APK verification is handled by:

```text
scripts/verify_android_build.ps1
```

That script builds the native arm64 and x86_64 libraries, assembles the debug APK with native core enabled, and checks that the APK contains both packaged libraries.

Native gateway builds can be produced without editing source code:

```text
gradle :app:assembleDebug -PcameraConnector.useNativeCore=true
```

The native gateway now routes start/stop through `ReceiverForegroundService`, so Android owns the long-running foreground lifecycle while Rust owns receiver behavior and status. Account setup, receiver network settings, project state, SAF publishing, and publish queue visibility also cross the gateway, keeping authentication, import state, and dashboard persistence in the shared core. The remaining bridge work is native gateway smoke testing on a physical device.

`NativeCoreGateway` polls the native dashboard every 2 seconds while it is open. This keeps receiver status, connected accounts, transfer failures, and newly imported assets moving into Compose without coupling the UI to service internals.

Native receiver `local_addr` values are parsed into separate host and port fields before they reach Compose, so the project receiver panel can render a stable `host:port` label across FTP, future STC-style routes, IPv4, hostnames, and bracketed IPv6 addresses.

Android 13+ notification permission is treated as a receiver start prerequisite. `AndroidPermissionGateway` checks `POST_NOTIFICATIONS`, `MainActivity` owns the permission launcher, and the project receiver panel blocks Start with a visible reason until notifications are available.

## 6. Storage Strategy

MVP strategy:

- Config/state: app-private storage.
- Current native smoke output: app-private `filesDir/output`.
- User-facing output: user-selected SAF document tree when configured, otherwise app-private fallback.
- Display path: virtual camera path from transfer log, not Android filesystem path.

The Android bootstrap only seeds `output_dir` and `state_dir` into native receiver settings. It must not reset protocol, host, or ports on app startup because those are user-configurable receiver settings.

Android URI values stay inside the storage gateway. The dashboard and project asset views still use product concepts: source name, username, transfer id, original path, format, duplicate count, and final location kind.

Cross-platform project package migration is not part of the Android storage model. It remains a deferred protocol over exported project facts, documented in `docs/superpowers/specs/2026-05-28-project-package-migration-protocol-design.md`.

## 7. Receiver Lifecycle

The receiver starts through `ReceiverForegroundService`.

Rules:

- Starting receiver always creates a foreground notification.
- Stopping receiver tears down the core receiver and removes the foreground notification.
- UI treats missing listener or failed service start as stopped with a visible failure.
- If notification permission or storage permission is missing, the UI surfaces setup actions instead of silently failing.

## 8. Current Android Slice

The current Android slice is project-based:

1. Global shell with Projects, Accounts, and Settings.
2. Project Management as the startup surface, with create/select project actions.
3. Photo-first project workspace with receiver launch/status at the top.
4. Receiver control inside the active project workspace.
5. Account management as a global surface because accounts are not project-owned.
6. Settings-owned output selection and diagnostics.
7. Publish queue visibility through receiver status and diagnostics.
8. Native gateway backed by the Rust core and Android foreground service.

The next milestone is physical-device verification with a real camera: Android foreground-service lifecycle, hotspot/LAN reachability, camera FTP login, RAW/JPEG upload, SAF publish recovery, and project-scoped photo/detail/transfer visibility.

## 9. Current Version Targets

As of 2026-05-22, the Android skeleton targets:

- Android Gradle Plugin 9.2.0
- Kotlin 2.3.21
- Compose BOM 2026.05.00
- compileSdk 36
- minSdk 26

These versions are intentionally centralized in Gradle files so they can be revised before the first CI build if the local Android SDK differs.

Local Android build verification is available through:

```text
scripts/verify_android_build.ps1
```

The current Windows development setup uses JDK 17, Android SDK platform 36, build-tools 36.0.0, and Gradle 9.5.1.

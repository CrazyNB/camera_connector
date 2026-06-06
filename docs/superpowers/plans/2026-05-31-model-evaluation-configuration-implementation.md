# Model Evaluation Configuration Implementation Plan

> Status: closed as an implementation history record. The current product shape
> is defined by the source code plus
> `docs/superpowers/specs/2026-05-31-model-evaluation-configuration-design.md`
> and
> `docs/superpowers/specs/2026-05-31-smart-selection-evaluation-redesign.md`.
> Do not use the unchecked boxes below as an active task backlog.

**Goal:** Add project-level model evaluation configuration, editable versioned prompt profiles, explicit no-key behavior, manual project recommendations, and portrait-specific local CV contracts.

**Architecture:** Keep local technical assessment, model evaluation, selection recommendations, and user marks as independent concepts. Add configuration and run-snapshot models in core SQLite, expose them through FFI/Android gateway DTOs, then update the Android settings and worker flow so upload-time evaluation is project-controlled while project-level recommendation is manual-only.

**Tech Stack:** Rust 2021, `rusqlite`, `serde`, existing `core-ffi` JNI/JSON bridge, Kotlin, Jetpack Compose, app-private core config JSON for provider resources and API keys, Gradle unit tests.

---

## File Structure

- `core/src/analysis/config.rs`: model provider settings, prompt profile/version, project evaluation settings, evaluation run, and subject assessment domain types.
- `core/src/analysis/model_eval.rs`: add run/prompt identity to model evaluation records and keep local stub source explicit.
- `core/src/analysis/recommendation.rs`: add run identity and keep project recommendations manual-only at service boundaries.
- `core/src/storage/mod.rs`: SQLite schema and persistence for config, prompts, runs, subject assessments, and updated evaluation rows.
- `core/src/service.rs`: project settings defaults, prompt editing/forking, provider configured checks, manual project recommendation entry point, and upload-time job gating.
- `core/src/lib.rs` and `core/src/analysis/mod.rs`: public exports for new config models.
- `core-ffi/src/lib.rs`: JSON bridge for settings, prompt profiles, prompt editing, and manual project recommendation.
- `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`: Android DTOs and method contracts for model settings, prompts, and project recommendation.
- `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`: FFI JSON mapping.
- `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeMobileCore.kt`: native method declarations.
- `core/src/push/config.rs`: app-private model provider resources, including API key persistence outside SQLite.
- `apps/android/app/src/main/java/com/cameraconnector/app/service/SmartSelectionAnalysisWorker.kt`: obey project settings and provider configured state.
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`: global provider and project evaluation controls.
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectAssetsScreen.kt`: manual project recommendation action and no-key CTA.
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectUiModels.kt`: UI helper models for evaluation settings and provider status.

## Task 1: Core Configuration Domain And Schema

**Owned files:**
- Create: `core/src/analysis/config.rs`
- Modify: `core/src/analysis/mod.rs`
- Modify: `core/src/lib.rs`
- Modify: `core/src/storage/mod.rs`
- Test: `core/tests/evaluation_config_tests.rs`

- [ ] Add Rust enums and structs:
  - `ModelProviderKind { None, OpenAi, Custom, Imported }`
  - `ModelSendMode { PreviewOnly, DetailImage }`
  - `PromptScope { Global, Project }`
  - `SceneProfile { General, Portrait, Action, Landscape, Custom }`
  - `CvPolicy { Loose, Standard, Strict }`
  - `ProjectRecommendationMode { Manual }`
  - `EvaluationRunType { AssetEvaluation, BurstRecommendation, ProjectRecommendation }`
  - `EvaluationRunTrigger { Upload, BurstStable, Manual, Retry }`
  - `EvaluationRunStatus { Pending, Running, Ready, Failed, Skipped }`
  - `ModelProviderSettings`, `PromptProfile`, `PromptProfileVersion`,
    `ProjectEvaluationSettings`, `EvaluationRun`, `SubjectAssessment`.
- [ ] Implement `as_str` and `from_str` for each enum. Unknown scene/profile
  values must fall back to `General`; unknown policy falls back to `Standard`;
  unknown recommendation mode falls back to `Manual`.
- [ ] Add SQLite tables:
  - `model_provider_settings`
  - `prompt_profiles`
  - `prompt_profile_versions`
  - `project_evaluation_settings`
  - `evaluation_runs`
  - `subject_assessments`
- [ ] Add indexes:
  - `idx_prompt_profiles_scope_project(scope, project_id, enabled)`
  - `idx_prompt_versions_profile(prompt_profile_id, created_at_ms)`
  - `idx_evaluation_runs_project(project_id, run_type, status, created_at_ms)`
  - `idx_subject_assessments_group(project_id, asset_group_id, subject_type)`
- [ ] Seed built-in prompt profiles if none exist:
  - `general-default`: tags `["general", "balanced"]`
  - `portrait-conservative`: tags `["portrait", "conservative"]`
  - `landscape-technical`: tags `["landscape", "technical"]`
  Each built-in profile needs one prompt version and `active_version_id`.
- [ ] Add storage methods:
  - `model_provider_settings()`
  - `save_model_provider_settings(settings)`
  - `prompt_profiles_for_project(project_id)`
  - `prompt_profile_version(version_id)`
  - `save_prompt_profile(profile)`
  - `save_prompt_profile_version(version)`
  - `project_evaluation_settings(project_id)`
  - `save_project_evaluation_settings(settings)`
  - `save_evaluation_run(run)`
  - `latest_evaluation_run(project_id, run_type)`
  - `save_subject_assessment(assessment)`
  - `subject_assessments_for_asset_groups(project_id, group_ids)`
- [ ] Default project settings must be:
  - `model_evaluation_enabled = false`
  - `auto_evaluate_on_upload = false`
  - `auto_burst_recommendation_enabled = true`
  - `project_recommendation_mode = manual`
  - `scene_profile = general`
  - `cv_policy = standard`
  - `allow_risky_model_selects = false`
- [ ] Tests in `core/tests/evaluation_config_tests.rs` must verify:
  - enum string round trips and unknown fallbacks.
  - built-in prompt profiles are seeded with active versions.
  - default project settings are created for a new project and model evaluation is disabled.
  - prompt version save/query preserves prompt hash and prompt text.
  - evaluation run save/query preserves manual trigger and status.
  - subject assessment save/query round-trips a portrait face assessment.
- [ ] Run: `cargo test -p camera_connector_core evaluation_config_tests`.

## Task 2: Core Prompt Versioning And Project Settings Service

**Owned files:**
- Modify: `core/src/service.rs`
- Modify: `core/src/storage/mod.rs`
- Test: `core/tests/evaluation_config_tests.rs`
- Test: `core/tests/service_tests.rs`

- [ ] Add service APIs:
  - `model_provider_settings()`
  - `save_model_provider_settings(settings)`
  - `project_evaluation_settings(project_id)`
  - `save_project_evaluation_settings(settings)`
  - `prompt_profiles_for_project(project_id)`
  - `fork_prompt_profile_for_project(project_id, source_profile_id, name, now_ms)`
  - `save_prompt_profile_version(project_id, prompt_profile_id, prompt_text, now_ms)`
- [ ] `fork_prompt_profile_for_project` must:
  - reject archived or missing projects.
  - copy the active source prompt text.
  - create a project-scoped profile.
  - copy style tags and scene profile.
  - set `built_in = false`.
- [ ] `save_prompt_profile_version` must:
  - reject built-in global profiles.
  - compute a stable prompt hash from prompt text and output schema version.
  - create a new immutable version.
  - update the profile's `active_version_id`.
- [ ] `save_project_evaluation_settings` must:
  - force `project_recommendation_mode = manual`.
  - reject missing prompt profile ids.
  - allow `prompt_profile_id = null` only when model evaluation is disabled.
  - keep secrets out of SQLite.
- [ ] Tests must verify:
  - editing a built-in prompt fails unless forked first.
  - forking creates a project-scoped editable prompt.
  - editing the project prompt creates a new version without deleting the old one.
  - invalid prompt id is rejected when model evaluation is enabled.
  - saved settings always preserve manual project recommendation mode.
- [ ] Run: `cargo test -p camera_connector_core evaluation_config_tests service_tests`.

## Task 3: Core Evaluation Run Gating And Manual Project Recommendation

**Owned files:**
- Modify: `core/src/analysis/jobs.rs`
- Modify: `core/src/analysis/model_eval.rs`
- Modify: `core/src/analysis/recommendation.rs`
- Modify: `core/src/service.rs`
- Modify: `core/src/storage/mod.rs`
- Test: `core/tests/analysis_job_tests.rs`
- Test: `core/tests/model_evaluation_tests.rs`
- Test: `core/tests/recommendation_tests.rs`

- [ ] Extend `ModelEvaluation` with:
  - `run_id: String`
  - `prompt_profile_id: Option<String>`
  - `prompt_version_id: Option<String>`
  - `prompt_hash: Option<String>`
- [ ] Update model evaluation persistence and tests to include those fields.
- [ ] Add `provider_configured` input to the service/worker-facing evaluation drain path.
- [ ] Upload-time model evaluation must be skipped when:
  - project model evaluation is disabled.
  - auto evaluate on upload is disabled.
  - provider is not configured.
- [ ] Universal technical assessment remains enabled regardless of provider state.
- [ ] Burst recommendation may run automatically only when
  `auto_burst_recommendation_enabled = true`.
- [ ] Add service API `generate_project_recommendation(project_id, now_ms)`.
- [ ] `generate_project_recommendation` must:
  - require provider configured or imported/local-stub dev evaluator explicitly enabled.
  - create `evaluation_run(run_type = project_recommendation, trigger = manual)`.
  - build candidate set from non-burst asset groups plus burst winners.
  - write a scoped project recommendation referencing the run id.
  - never run from upload-time drains.
- [ ] Tests must verify:
  - no provider skips model evaluation but still writes technical assessment.
  - project model evaluation disabled skips model evaluation jobs.
  - auto burst recommendation obeys project setting.
  - project recommendation is not produced by automatic drains.
  - manual project recommendation creates a run snapshot and project recommendation.
- [ ] Run: `cargo test -p camera_connector_core analysis_job_tests model_evaluation_tests recommendation_tests`.

## Task 4: FFI JSON Contract

**Owned files:**
- Modify: `core-ffi/src/lib.rs`
- Modify: `core-ffi/tests/mobile_core_tests.rs`
- Modify: `core-ffi/tests/mobile_ffi_tests.rs`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeMobileCore.kt`

- [ ] Add JSON endpoints for:
  - load/save global provider settings.
  - load/save project evaluation settings.
  - list prompt profiles for project.
  - fork prompt profile.
  - save prompt version.
  - generate manual project recommendation.
  - latest project recommendation run status.
- [ ] JSON shape must include no secret value fields.
- [ ] Missing provider must be represented as `configured: false`.
- [ ] Project settings JSON must include:
  - `model_evaluation_enabled`
  - `auto_evaluate_on_upload`
  - `auto_burst_recommendation_enabled`
  - `project_recommendation_mode`
  - `prompt_profile_id`
  - `scene_profile`
  - `cv_policy`
  - `allow_risky_model_selects`
  - `max_image_side`
  - `batch_size`
- [ ] Prompt profile JSON must include:
  - `prompt_profile_id`
  - `scope`
  - `project_id`
  - `name`
  - `style_tags`
  - `scene_profile`
  - `active_version_id`
  - `built_in`
  - `enabled`
- [ ] Tests must verify round-trip JSON for provider settings, project settings,
  prompt list, prompt fork/edit, and manual project recommendation.
- [ ] Run: `cargo test -p camera-connector-ffi`.

## Task 5: Android Gateway, Settings UI, And No-Key UX

**Owned files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/storage/AndroidStorageGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectUiModels.kt`
- Test: `apps/android/app/src/test/java/com/cameraconnector/app/core/NativeDashboardMappingTest.kt`
- Test: `apps/android/app/src/test/java/com/cameraconnector/app/ui/ProjectUiModelsTest.kt`

- [x] Add Android DTOs:
  - `ModelProviderSettingsUi`
  - `PromptProfileUi`
  - `ProjectEvaluationSettingsUi`
  - `EvaluationRunUi`
- [x] Treat the visible `ModelSelects` collection as project-scope selected
  works. Burst winners can affect burst covers/badges but must not enter this
  filter unless they are selected by a project recommendation.
- [x] Add gateway methods matching the FFI contract.
- [x] `AndroidStorageGateway` must store only API-key configured state or key alias,
  not expose API key text through core DTOs.
- [x] Settings screen must show two groups:
  - Global model provider: provider state, model name, send mode, batch size.
  - Current project intelligence: model evaluation switch, auto-evaluate switch,
    auto burst recommendation switch, scene profile, prompt profile, CV policy,
    risk-photo participation.
- [x] Global defaults can prefill new project settings, but editing a global
  default must not change existing project evaluation settings.
- [x] Prompt profile list must show name and style tags.
- [x] Built-in prompt edit action must fork before editing.
- [x] No-key UX:
  - project-level model evaluation switch can be enabled only after provider is configured, or shows a provider setup CTA.
  - manual project recommendation action is disabled when provider is missing.
  - local stub source must not be labelled as real model output.
- [x] Tests must verify:
  - settings mapping defaults model evaluation off.
  - prompt style tags render in the UI model.
  - no-key state disables manual project recommendation.
  - project settings preserve manual project recommendation mode.
- [x] Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_unit_tests.ps1`.

## Task 6: Android Worker Flow And Manual Recommendation Action

**Owned files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/service/SmartSelectionAnalysisWorker.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/service/ReceiverForegroundService.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectAssetsScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/CameraConnectorApp.kt`
- Test: `apps/android/app/src/test/java/com/cameraconnector/app/service/ReceiverServiceControllerTest.kt`
- Test: `apps/android/app/src/test/java/com/cameraconnector/app/ui/ProjectUiModelsTest.kt`

- [x] Worker must load active project evaluation settings before model work.
- [x] Worker must always run local technical assessment when needed.
- [x] Worker must pass provider configured state to core drain/gating calls.
- [x] Worker must not generate project recommendations from upload-time drains.
- [x] Project photos screen must expose a manual "Generate Project Recommendation" action
  near model-select filtering or project intelligence status.
- [x] When provider is missing, action must show provider setup feedback instead
  of starting a run.
- [x] When action succeeds, show a short top toast with last run status.
- [x] Tests must verify:
  - provider missing does not block local technical assessment.
  - upload drain does not create project recommendation.
  - manual action calls gateway once and updates feedback.
- [x] Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_unit_tests.ps1`.

## Task 7: Portrait Subject Assessment Contract

**Owned files:**
- Modify: `core/src/analysis/config.rs`
- Modify: `core/src/storage/mod.rs`
- Modify: `core/src/service.rs`
- Modify: `core-ffi/src/lib.rs`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`
- Test: `core/tests/evaluation_config_tests.rs`
- Test: `core-ffi/tests/mobile_core_tests.rs`

- [ ] Add subject assessment read/write APIs in service and FFI.
- [ ] `scene_profile = portrait` must be the only project setting that enables
  portrait subject assessment scheduling.
- [ ] Subject assessment JSON must include:
  - `subject_type`
  - `detector_kind`
  - `detector_version`
  - `status`
  - `gate_status`
  - `regions`
  - `signals`
  - `summary`
- [ ] First version can store detector output from Android or imported sources.
  It does not need to bundle a new ML dependency in this task.
- [ ] Portrait assessment is a risk/gate input to model evaluation, not the final
  aesthetic score or recommendation source.
- [ ] Tests must verify:
  - general projects do not schedule portrait subject assessment.
  - portrait projects schedule it.
  - subject assessment round-trips through FFI JSON.
- [ ] Run: `cargo test -p camera_connector_core evaluation_config_tests` and
  `cargo test -p camera-connector-ffi`.

## Task 8: Documentation And Acceptance Verification

**Owned files:**
- Modify: `docs/superpowers/specs/2026-05-31-smart-selection-evaluation-redesign.md`
- Modify: `docs/superpowers/specs/2026-05-31-model-evaluation-configuration-design.md`
- Modify: `docs/product/android-app-architecture.md`
- Modify: `apps/android/README.md`

- [x] Update smart-selection docs to point to the configuration design.
- [x] Document that project-level recommendation is manual-only.
- [x] Document no-key behavior and local-stub source labelling.
- [x] Document portrait subject assessment as a contract, not a hard Android-only dependency.
- [x] Run verification:
  - `cargo test`
  - `cargo test -p camera-connector-ffi`
  - `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_unit_tests.ps1`
  - `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_build.ps1`
  - `git diff --check`

## Acceptance Matrix

| Requirement | Evidence |
| --- | --- |
| Model capability is global, model evaluation enablement is project-level | Core storage/service tests for provider settings and project settings |
| New projects default model evaluation off | `evaluation_config_tests` |
| Prompt profiles are tagged, editable, and versioned | prompt fork/edit tests and FFI JSON tests |
| Old evaluations keep prompt version/hash | model evaluation persistence tests |
| Universal CV still runs with no API key | analysis job tests |
| Missing provider is explicit | FFI JSON and Android mapping tests |
| Burst recommendation can auto-run by project setting | analysis job/recommendation tests |
| Project recommendation is manual-only | service and Android worker tests |
| Portrait assessment only runs for portrait projects | subject assessment tests |
| Favorites/marks remain independent of model recommendations | recommendation and UI mapping tests |
| Android settings expose provider/project/prompt controls | Android UI model tests |
| Build remains healthy | full verification commands in Task 8 |

## Sub-Agent Execution Slices

Use `gpt-5.5`, medium reasoning, fast bounded prompts. Agents are not alone in
the worktree; they must not revert unrelated edits and must keep to their owned
files.

Recommended first wave:

1. Core config schema worker: Task 1.
2. Core service/prompt worker: Task 2 after Task 1 lands.
3. FFI contract worker: Task 4 after Tasks 1-2 land.
4. Android settings worker: Task 5 after Task 4 lands.
5. Worker/manual recommendation worker: Task 6 after Task 3 and Task 5 land.
6. Portrait contract worker: Task 7 after Task 1 lands.
7. Documentation/verification worker: Task 8 after implementation tasks land.

Task 1 is the current critical path because downstream workers need the core models
and schema names.

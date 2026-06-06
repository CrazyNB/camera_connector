# Smart Selection Evaluation Redesign Implementation Plan

> Status: closed as an implementation history record. The current product shape
> is defined by the source code plus
> `docs/superpowers/specs/2026-05-31-smart-selection-evaluation-redesign.md`
> and
> `docs/superpowers/specs/2026-05-31-model-evaluation-configuration-design.md`.
> Do not use the unchecked boxes below as an active task backlog.

**Goal:** Replace CV-as-final-score smart selection with a split model where local CV only detects technical risks, model evaluation scores every photo work unit, recommendations choose burst winners and project-level model selections, and user marks stay independent from algorithm results.

**Architecture:** Keep the existing project, asset-group, burst-group, background-job, thumbnail, and Android project workspace foundations. Add focused core modules for technical assessments, model evaluations, selection recommendations, and user marks; then update storage, FFI, Android DTOs, workers, and UI to consume those semantics. Because the app is still in development, schema cleanup should keep only the current smart-selection semantics.

**Tech Stack:** Rust 2021, `rusqlite`, `serde`, existing `core-ffi` JSON/JNI bridge, Kotlin, Jetpack Compose, Gradle unit tests.

---

## File Structure

- `core/src/analysis/technical.rs`: local technical gate types and preview-based defect detection.
- `core/src/analysis/model_eval.rs`: model evaluation types, tiers, provider-independent records, and local deterministic stub used until a real LLM/VLM provider is configured.
- `core/src/analysis/recommendation.rs`: model recommendations with `scope` and `subject_id` for `burst_group` and `project`; keep user decisions outside algorithm recommendation status.
- `core/src/storage/mod.rs`: SQLite tables and persistence for technical assessments, model evaluations, selection recommendations, and user marks.
- `core/src/service.rs`: project-scoped queries and actions that expose model recommendation and user mark states.
- `core/src/lib.rs`: public exports for new analysis models.
- `core-ffi/src/lib.rs`: mobile JSON bridge for model evaluation and recommendation fields.
- `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`: Android DTOs and gateway method contracts.
- `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`: mapping from FFI JSON to Android DTOs.
- `apps/android/app/src/main/java/com/cameraconnector/app/service/SmartSelectionAnalysisWorker.kt`: job execution order for technical assessment, model evaluation, burst recommendation, and project recommendation refresh.
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectAssetsScreen.kt`: filters and badges for model recommendations, favorites, flags, risk, and pending analysis.
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/PhotoDetailScreen.kt`: detail page score summary, technical risk panel, and independent user action buttons.
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`: high-level model evaluation settings instead of raw CV scoring weights.

## Task 1: Core Evaluation Types

**Files:**
- Create: `core/src/analysis/technical.rs`
- Create: `core/src/analysis/model_eval.rs`
- Modify: `core/src/analysis/mod.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/evaluation_model_tests.rs`

- [ ] Add `TechnicalAssessmentStatus`, `TechnicalGateStatus`, `TechnicalDefectType`, `TechnicalDefectSeverity`, `TechnicalDefectFlag`, and `TechnicalAssessment` in `technical.rs`.
- [ ] Add `ModelEvaluatorKind`, `ModelEvaluationStatus`, `ModelEvaluationTier`, and `ModelEvaluation` in `model_eval.rs`.
- [ ] Re-export these models from `analysis/mod.rs` and `lib.rs`.
- [ ] Write tests that assert:
  - `TechnicalGateStatus::from_str("reject") == TechnicalGateStatus::Reject`.
  - unknown gate status falls back to `Inconclusive`.
  - `ModelEvaluationTier::from_score(92) == Excellent`.
  - `ModelEvaluationTier::from_score(41) == Weak`.
  - `ModelEvaluationTier::from_score(20) == Reject`.
- [ ] Run: `cargo test -p camera_connector_core evaluation_model_tests`.

## Task 2: Storage Schema And Persistence

**Files:**
- Modify: `core/src/storage/mod.rs`
- Test: `core/tests/analysis_store_tests.rs`
- Test: `core/tests/storage_store_tests.rs`

- [ ] Add SQLite tables:
  - `technical_assessments(asset_group_id, assessor_version, status, gate_status, defect_flags_json, preview_source, analyzed_at_ms)`
  - `model_evaluations(evaluation_id, project_id, asset_group_id, evaluator_kind, evaluator_version, status, score, tier, selectable, summary, strengths_json, weaknesses_json, technical_warnings_json, created_at_ms, updated_at_ms)`
  - update or replace `selection_recommendations` with scope fields: `scope`, `project_id`, `subject_id`, `selected_asset_group_ids_json`, `candidate_asset_group_ids_json`, `rejected_asset_group_ids_json`, `source`, `status`, `confidence`, `reason`, `created_at_ms`, `updated_at_ms`
  - keep `asset_group_user_marks(project_id, group_id, favorite, marked, created_at_ms, updated_at_ms)` as the concrete user marks table.
- [ ] Add indexes:
  - `idx_technical_assessments_status(status, gate_status)`
  - `idx_model_evaluations_project(project_id, status, tier)`
  - `idx_model_evaluations_asset_group(asset_group_id, evaluator_version)`
  - `idx_recommendations_scope(project_id, scope, subject_id, status)`
- [ ] Add save/query methods:
  - `save_technical_assessment`
  - `technical_assessments_for_asset_groups`
  - `save_model_evaluation`
  - `model_evaluations_for_asset_groups`
  - `save_selection_recommendation`
  - `latest_selection_recommendation_by_scope`
- [ ] Update tests so persistence round-trips one technical assessment, one model evaluation, one burst recommendation, and one project recommendation.
- [ ] Run: `cargo test -p camera_connector_core analysis_store_tests storage_store_tests`.

## Task 3: Local Technical Gate

**Files:**
- Modify: `core/src/analysis/technical.rs`
- Modify: `core/src/analysis/technical.rs`
- Test: `core/tests/technical_assessment_tests.rs`

- [x] Add `assess_preview_sample(asset_group_id, sample, assessor_version, analyzed_at_ms) -> TechnicalAssessment`.
- [x] Detect severe blur using both edge-difference/Laplacian-style detail and a simple high-frequency proxy. Mark `blur` as `high` or `severe` when both signals are weak.
- [x] Detect highlight and shadow clipping using clipped-pixel ratio plus coarse connected-area ratio. Small highlights should not create `Reject`.
- [x] Detect noisy low-information previews with a flat-region local variance estimate.
- [x] Detect severe color cast only when RGB preview support is available; for current luma-only input, do not invent a color-cast defect.
- [x] Remove local-score-as-product-result entirely. Android and FFI should call technical assessment APIs directly.
- [x] Update tests:
  - blurred preview produces a `blur` defect and `Warn` or `Reject`.
  - overexposed preview produces `highlight_clip`.
  - underexposed preview produces `shadow_clip`.
  - invalid preview produces `Unsupported`.
- [x] Run: `cargo test -p camera_connector_core technical_assessment_tests`.

## Task 4: Model Evaluation Stub And Job Contracts

**Files:**
- Modify: `core/src/analysis/model_eval.rs`
- Modify: `core/src/analysis/jobs.rs`
- Modify: `core/src/service.rs`
- Test: `core/tests/model_evaluation_tests.rs`
- Test: `core/tests/analysis_job_tests.rs`

- [ ] Add job types:
  - `assess_asset_group_technical_quality`
  - `evaluate_asset_group_with_model`
  - `recommend_burst_group_from_model`
  - `recommend_project_model_selections`
- [ ] Add deterministic local stub `evaluate_asset_group_with_stub(project_id, asset_group_id, assessment, now_ms) -> ModelEvaluation`.
- [ ] Stub rules:
  - `Reject` gate status maps to score `20`, tier `Reject`, selectable `false`.
  - `Warn` maps to score `58`, tier `Normal`, selectable `true`.
  - `Pass` maps to score `72`, tier `Good`, selectable `true`.
  - `Unsupported` maps to status `Skipped`, score `0`, tier `Reject`, selectable `false`.
- [ ] The stub is only a product placeholder until a real LLM/VLM provider is wired, but it preserves the target data flow: every asset group can receive a model evaluation.
- [ ] Publish completion should still enqueue burst detection first; burst detection or worker completion should enqueue technical assessment and model evaluation jobs for affected asset groups.
- [ ] Run: `cargo test -p camera_connector_core model_evaluation_tests analysis_job_tests`.

## Task 5: Selection Recommendations

**Files:**
- Modify: `core/src/analysis/recommendation.rs`
- Modify: `core/src/storage/mod.rs`
- Modify: `core/src/service.rs`
- Test: `core/tests/recommendation_tests.rs`

- [ ] Replace burst-only recommendation fields with `SelectionRecommendationScope`, `subject_id`, `selected_asset_group_ids`, `candidate_asset_group_ids`, and `rejected_asset_group_ids`.
- [ ] Add `recommend_burst_group_from_model_evaluations(project_id, burst_group_id, member_evaluations, assessments, now_ms)`.
- [ ] Add `recommend_project_model_selections(project_id, candidate_evaluations, burst_recommendations, now_ms)`.
- [ ] Burst rules:
  - only selectable model evaluations can become selected.
  - highest score wins, with confidence based on score gap.
  - if all members are unselectable or tier `Reject`, status is `NoSelection`.
  - alternates are selectable non-winners.
- [ ] Project rules:
  - include non-burst asset groups and burst selected winners.
  - select `Excellent` and strong `Good` items by default.
  - do not select a weak burst winner into project-level model recommendations unless model tier is at least `Good`.
- [ ] Keep algorithm recommendation status limited to pending, ready, stale, failed, and no-selection states. User state belongs in user marks.
- [ ] Run: `cargo test -p camera_connector_core recommendation_tests`.

## Task 6: Collection Filters And Query Semantics

**Files:**
- Modify: `core/src/service.rs`
- Modify: `core/src/storage/mod.rs`
- Test: `core/tests/asset_query_tests.rs`

- [ ] Rebuild collection filters from:
  - technical gate status
  - model evaluation status/tier/selectable
  - selection recommendation status
  - user marks
- [ ] Target filter labels:
  - `all`
  - `model_selects`
  - `favorites`
  - `marked`
  - `technical_risk`
  - `pending_analysis`
- [x] Remove reliance on local technical metrics for sort/filter. Use model score for model sorting and technical gate for risk filtering.
- [x] Ensure favorite/marked filters are independent from recommendation filters.
- [ ] Run: `cargo test -p camera_connector_core asset_query_tests`.

## Task 7: FFI And Android Mapping

**Files:**
- Modify: `core-ffi/src/lib.rs`
- Modify: `core-ffi/tests/mobile_core_tests.rs`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/AssetUiModels.kt`
- Test: `apps/android/app/src/test/java/com/cameraconnector/app/core/NativeDashboardMappingTest.kt`

- [ ] Add JSON fields for:
  - `technical_gate_status`
  - `technical_defects`
  - `model_score`
  - `model_tier`
  - `model_summary`
  - `is_model_select`
  - `is_favorite`
  - `is_flagged`
- [x] Keep Android UI-facing models on the current names for model select, favorite, marked, technical risk, and pending analysis.
- [x] Update mapping tests to assert Chinese/UI semantics are driven by model select, favorite, flagged, quality risk, and pending analysis.
- [x] Run: `cargo test -p camera-connector-ffi` and Android unit tests.

## Task 8: Android Worker Flow

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/service/SmartSelectionAnalysisWorker.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Test: `apps/android/app/src/test/java/com/cameraconnector/app/core/NativeDashboardMappingTest.kt`

- [x] Worker order:
  - claim burst detection jobs.
  - claim technical assessment jobs and write technical assessments.
  - claim model evaluation jobs and write model evaluations.
  - claim burst recommendation jobs.
  - debounce project recommendation refresh.
- [x] Keep preview decode and thumbnail cache behavior unchanged.
- [x] Do not block project browsing while model evaluation is pending.
- [x] Run Android unit tests.

## Task 9: Android UI Semantics

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectAssetsScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/PhotoDetailScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectUiModels.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`
- Test: `apps/android/app/src/test/java/com/cameraconnector/app/ui/ProjectUiModelsTest.kt`

- [x] Photo grid filters become: 全部, 模型优选, 收藏, 标记, 技术风险, 待分析.
- [x] Burst cards show compact count badges. Preview list uses `N`; detail/burst overview can use `M/N`.
- [x] Detail page shows:
  - model score/tier/summary as primary evaluation.
  - technical risk as secondary diagnostics.
  - favorite and flagged as human actions.
  - trash as destructive delete, visually separate from remove-from-burst.
- [x] Do not add a separate recommendation-acceptance UX. Favorite is the user signal; ignoring model select is allowed.
- [x] Settings moves away from raw CV weights and exposes model evaluation mode/profile/privacy/batch controls.
- [x] Run Android unit tests and `scripts/verify_android_build.ps1`.

## Task 10: Documentation And Cleanup

**Files:**
- Modify: `docs/superpowers/specs/2026-05-26-smart-selection-design.md`
- Modify: `docs/superpowers/specs/2026-05-31-smart-selection-evaluation-redesign.md`
- Modify: `docs/product/android-app-architecture.md`
- Modify: `apps/android/README.md`

- [x] Update docs to state that CV is only the local technical gate.
- [x] Document the distinction between model recommendations, favorites, flags, and technical risks.
- [x] Remove stale references to local technical metrics as product-facing scores.
- [x] Run `cargo test`.
- [x] Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_build.ps1`.

## Self-Check Notes

- Spec coverage: tasks cover local technical assessment, model evaluation for non-burst and burst items, burst recommendation, project recommendation, user mark separation, schema cleanup, FFI, Android worker flow, UI semantics, settings, and docs.
- Scope: LLM/VLM provider integration is intentionally represented by provider-independent records plus a local stub. Real network provider wiring should be a later focused spec after the data model and UI semantics are stable.
- Data migration: none during development; current schema is the source of truth.

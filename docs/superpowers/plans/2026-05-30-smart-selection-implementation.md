# Smart Selection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build asynchronous burst grouping, local score persistence, strategy-based recommendations, review queues, and Android grid/detail/review-mode surfaces for smart selection.

**Architecture:** Publish completion enqueues durable analysis jobs in SQLite. Rust core owns analysis models, grouping, recommendation state, review queues, job claiming, and score/recommendation queries; platform layers provide preview samples when needed and render the resulting read models. Android consumes enriched project photo and review-card queries, exposes strategy controls in Settings, and shows smart selection in grid, detail, and card-based Review mode.

**Tech Stack:** Rust 2021, `rusqlite`, `serde`, existing `core-ffi` JSON/JNI bridge, Kotlin, Jetpack Compose, Gradle unit tests.

---

## File Structure

- `core/src/analysis/mod.rs`: public analysis domain module and re-exports.
- `core/src/analysis/jobs.rs`: background job types, dedupe keys, claim/complete/fail request structs.
- `core/src/analysis/burst.rs`: local burst regrouping rules and candidate-window selection.
- `core/src/analysis/scoring.rs`: score records, score normalization, score status, and preview-sample based scoring helpers.
- `core/src/analysis/recommendation.rs`: strategy profile evaluation, recommendation states, stale/user override handling.
- `core/src/analysis/review.rs`: review queue state, card selection, progress persistence, and review decisions.
- `core/src/storage/mod.rs`: SQLite schema and persistence methods for jobs, burst groups, scores, recommendations, and overrides.
- `core/src/service.rs`: service APIs that wrap storage and expose project-scoped analysis queries.
- `core-ffi/src/lib.rs`: mobile JSON/JNI bridge for enriched asset pages, strategy profiles, and analysis controls.
- `tools/cli/src/main.rs`: CLI commands for inspecting/running analysis.
- `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`: Android-facing smart selection DTOs and gateway methods.
- `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`: JSON mapping from FFI responses into Android DTOs.
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectAssetsScreen.kt`: photo grid score/recommendation badges, score sorting/filtering controls.
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/PhotoDetailScreen.kt`: score summary, reasons, burst filmstrip, manual correction entry points.
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/ReviewModeScreen.kt`: card-based screening flow for burst and review queues.
- `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`: strategy profile and scoring-weight controls.

## Implementation Tasks

### Task 1: Analysis Schema And Core Models

**Files:**
- Create: `core/src/analysis/mod.rs`
- Create: `core/src/analysis/jobs.rs`
- Create: `core/src/analysis/scoring.rs`
- Create: `core/src/analysis/recommendation.rs`
- Modify: `core/src/lib.rs`
- Modify: `core/src/storage/mod.rs`
- Test: `core/tests/analysis_store_tests.rs`

- [ ] Add analysis structs and enums: `AnalysisJob`, `AnalysisJobType`, `AnalysisJobStatus`, `BurstGroup`, `BurstGroupMember`, `QualityScore`, `QualityAnalysisStatus`, `StrategyProfile`, `SelectionRecommendation`, `RecommendationStatus`, and `UserOverride`.
- [ ] Add SQLite tables: `background_jobs`, `burst_groups`, `burst_group_members`, `quality_scores`, `strategy_profiles`, `selection_recommendations`, and `selection_user_overrides`.
- [ ] Add indexes for claim order and project queries:
  - `idx_background_jobs_claim(status, priority, next_attempt_at_ms, created_at_ms)`
  - `idx_background_jobs_dedupe(dedupe_key)`
  - `idx_burst_groups_project(project_id, updated_at_ms)`
  - `idx_burst_members_group(burst_group_id, member_group_id)`
  - `idx_quality_scores_group(asset_group_id, scorer_version)`
  - `idx_recommendations_group(burst_group_id, strategy_profile_id, status)`
- [ ] Write failing tests in `core/tests/analysis_store_tests.rs`:
  - `store_initializes_analysis_schema`
  - `store_upserts_strategy_profile`
  - `store_persists_quality_score_with_versions`
  - `store_persists_recommendation_with_grouping_strategy_and_scorer_versions`
- [ ] Implement the minimal storage methods needed by those tests.
- [ ] Run `cargo test -p camera_connector_core analysis_store_tests`.
- [ ] Commit with message `Add smart selection analysis schema`.

### Task 2: Durable Analysis Job Queue

**Files:**
- Modify: `core/src/analysis/jobs.rs`
- Modify: `core/src/storage/mod.rs`
- Test: `core/tests/analysis_job_tests.rs`
- Test: `core/tests/storage_store_tests.rs`

- [ ] Add job helpers for dedupe keys:
  - `burst_dedupe_key(project_id, source_identity, time_bucket_ms)`
  - `score_dedupe_key(asset_group_id, scorer_version)`
  - `recommend_dedupe_key(burst_group_id, strategy_profile_id, strategy_version)`
- [ ] Implement `enqueue_analysis_job`, `claim_analysis_jobs`, `complete_analysis_job`, and `fail_analysis_job`.
- [ ] Make `enqueue_analysis_job` idempotent: an active row with the same `dedupe_key` is reused rather than duplicated.
- [ ] Update `SqliteStore::complete_publish` so successful publish and asset indexing enqueue `detect_burst_for_asset_group` after the asset group is durable.
- [ ] Write tests:
  - `complete_publish_enqueues_burst_detection_job`
  - `analysis_jobs_dedupe_by_key`
  - `analysis_jobs_claim_in_priority_order`
  - `analysis_job_failure_sets_next_attempt`
- [ ] Run `cargo test -p camera_connector_core analysis_job_tests storage_store_tests`.
- [ ] Commit with message `Queue analysis jobs after publish`.

### Task 3: Burst Group Detection And Local Regrouping

**Files:**
- Modify: `core/src/analysis/burst.rs`
- Modify: `core/src/storage/mod.rs`
- Modify: `core/src/service.rs`
- Test: `core/tests/burst_grouping_tests.rs`

- [ ] Implement burst grouping over existing `asset_groups`, not individual `assets`.
- [ ] Use source identity, original parent path, capture time, received time, normalized filename sequence, and a strategy burst window to build a local candidate window.
- [ ] Treat upload order as unreliable. A late asset group must be able to merge two nearby burst groups or split an incorrect grouping during local regrouping.
- [ ] Persist `burst_groups` and `burst_group_members`; update `grouping_version` whenever membership changes.
- [ ] Mark affected recommendations as `stale` when membership changes, while leaving existing `quality_scores` attached to their asset groups.
- [ ] Write tests:
  - `burst_grouping_uses_capture_time_when_present`
  - `burst_grouping_falls_back_to_received_time_and_filename_sequence`
  - `burst_grouping_does_not_cross_source_identity`
  - `out_of_order_upload_merges_existing_burst_groups`
  - `late_member_marks_recommendation_stale`
- [ ] Run `cargo test -p camera_connector_core burst_grouping_tests`.
- [ ] Commit with message `Detect burst groups from project assets`.

### Task 4: Quality Score Persistence And Conservative Local Scoring

**Files:**
- Modify: `core/src/analysis/scoring.rs`
- Modify: `core/src/storage/mod.rs`
- Modify: `core/src/service.rs`
- Test: `core/tests/quality_scoring_tests.rs`

- [ ] Define a platform-neutral `PreviewSample` input with width, height, and downscaled luma bytes so core scoring does not depend on Android URI handling.
- [ ] Implement first-version scores: sharpness, exposure, highlight clipping penalty, shadow clipping penalty, composition, composition confidence, similarity cluster id, overall, reasons, `analysis_status`, `exif_status`, and optional `capture_time_ms`.
- [ ] Implement composition as conservative heuristics over interest heatmap, edge-cut risk, obvious tilt, low-information ratio, and weak balance.
- [ ] Fold obvious blur into sharpness and reasons; do not add a required separate motion blur score.
- [ ] Implement unsupported fallback: missing or unreadable preview writes `analysis_status = unsupported` and a `NeedsReview` reason instead of failing the project query.
- [ ] Write tests:
  - `sharp_preview_scores_above_blurred_preview`
  - `overexposed_preview_records_highlight_penalty`
  - `underexposed_preview_records_shadow_penalty`
  - `composition_flags_edge_cut_and_low_information_area`
  - `unsupported_preview_records_needs_review_status`
- [ ] Run `cargo test -p camera_connector_core quality_scoring_tests`.
- [ ] Commit with message `Add local quality scoring model`.

### Task 5: Strategy Profiles And Recommendations

**Files:**
- Modify: `core/src/analysis/recommendation.rs`
- Modify: `core/src/storage/mod.rs`
- Modify: `core/src/service.rs`
- Test: `core/tests/recommendation_tests.rs`

- [ ] Seed built-in read-only profiles: General, Conservative, Portrait, Action, Landscape.
- [ ] Implement custom profile save/update with user-facing weight fields: sharpness, exposure, composition, highlight clipping penalty, shadow clipping penalty, near-duplicate strictness, burst time window, and low-score hiding default.
- [ ] Clamp composition weight to `0.12` by default and prevent composition from overriding a sharpness veto.
- [ ] Generate `SelectionRecommendation` from within-burst normalized scores plus absolute technical thresholds.
- [ ] Persist `scorer_version`, `strategy_version`, and `grouping_version` on each recommendation.
- [ ] Implement `user_override` behavior: manual best, clear recommendation, restore automatic recommendation, merge burst, and split burst.
- [ ] Write tests:
  - `general_profile_selects_sharpest_balanced_frame`
  - `composition_cannot_promote_low_sharpness_frame_to_best`
  - `strategy_weight_change_marks_recommendation_stale`
  - `user_best_override_survives_recommendation_job`
  - `unsupported_scores_produce_needs_review`
- [ ] Run `cargo test -p camera_connector_core recommendation_tests`.
- [ ] Commit with message `Recommend best frames from burst scores`.

### Task 6: Project Photo Query Sorting And Filtering

**Files:**
- Modify: `core/src/service.rs`
- Modify: `core/src/storage/mod.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/storage_service_tests.rs`
- Test: `core/tests/asset_query_tests.rs`

- [ ] Extend `AssetGroupQuery` with:
  - `sort: AssetGroupSort`
  - `recommendation_state`
  - `score_min`
  - `score_max`
  - `analysis_status`
- [ ] Add `AssetGroupSort::LatestReceived`, `Filename`, and `GroupBestScore`.
- [ ] Enrich `ReceivedAssetGroup` or introduce an adjacent read DTO with burst id, burst member count, group best score, recommendation state, primary reason, stale flag, and analysis status.
- [ ] Implement core sorting/filtering by group best score. Single-frame asset groups use their own score; burst groups use the highest current score among member asset groups under the active strategy profile.
- [ ] Sort unsupported scores after scored items unless the query explicitly filters for unsupported.
- [ ] Write tests:
  - `project_asset_groups_sort_by_group_best_score`
  - `project_asset_groups_filter_by_score_range`
  - `project_asset_groups_filter_by_recommendation_state`
  - `stale_scores_remain_visible_with_stale_flag`
- [ ] Run `cargo test -p camera_connector_core storage_service_tests asset_query_tests`.
- [ ] Commit with message `Query project photos by smart selection state`.

### Task 7: Review Queues And Card Flow

**Files:**
- Create: `core/src/analysis/review.rs`
- Modify: `core/src/analysis/mod.rs`
- Modify: `core/src/storage/mod.rs`
- Modify: `core/src/service.rs`
- Test: `core/tests/review_queue_tests.rs`

- [ ] Add review queue types: `needs_review`, `unconfirmed_best`, `low_score_candidates`, `near_duplicates`, `unsupported`, and `user_overridden`.
- [ ] Add persistence for review progress per project and strategy profile.
- [ ] Add persistence for review session summary facts, current-session undo entries, and shortcut-action preferences.
- [ ] Add a virtual Selects collection read model derived from accepted recommendations and manual best decisions, with room for explicit user-managed membership later.
- [ ] Store shortcut preferences in an input-agnostic action mapping rather than a touch-only swipe model, so future PC keyboard/controller bindings can reuse the same actions.
- [ ] Implement review card query that returns one burst group or single-frame group at a time, including current best, candidate filmstrip data, score/confidence, reasons, and progress counts.
- [ ] Implement review decisions: accept recommended best, pick another best, mark needs review, hide low-score candidates, keep all candidates, clear recommendation, restore automatic recommendation, split group, and merge group.
- [ ] Implement current-session undo for the latest review decisions.
- [ ] Implement review session summary counts for processed groups, accepted recommendations, manual changes, remaining review groups, and low-score candidates.
- [ ] Update accepted recommendations and manual best decisions so they appear in the project Selects collection.
- [ ] Ensure accepting or overriding a recommendation removes the group from `unconfirmed_best` unless later analysis marks it stale again.
- [ ] Write tests:
  - `needs_review_queue_includes_low_confidence_and_unsupported_groups`
  - `review_progress_survives_reopening_project`
  - `accept_recommended_best_removes_group_from_unconfirmed_queue`
  - `mark_needs_review_moves_group_to_needs_review_queue`
  - `user_pick_best_records_override_and_updates_review_card`
  - `review_undo_restores_previous_decision`
  - `review_session_summary_counts_decisions`
  - `accepted_recommendations_appear_in_selects_collection`
  - `shortcut_preferences_store_input_agnostic_actions`
- [ ] Run `cargo test -p camera_connector_core review_queue_tests`.
- [ ] Commit with message `Add smart selection review queues`.

### Task 8: FFI And CLI Surface

**Files:**
- Modify: `core-ffi/include/camera_connector_mobile.h`
- Modify: `core-ffi/src/lib.rs`
- Modify: `core-ffi/tests/mobile_core_tests.rs`
- Modify: `core-ffi/tests/mobile_ffi_tests.rs`
- Modify: `tools/cli/src/main.rs`
- Test: `core-ffi/tests/mobile_core_tests.rs`
- Test: `core-ffi/tests/mobile_ffi_tests.rs`

- [ ] Extend mobile asset group query JSON with sort, score range, recommendation state, and analysis status fields.
- [ ] Include smart selection fields in project asset group page JSON.
- [ ] Add FFI/JNI methods for listing strategy profiles, saving a custom profile, claiming analysis jobs, recording score results, running recommendation for a burst group, and manual overrides.
- [ ] Add FFI/JNI methods for loading review queue summaries, fetching the next review card, and applying review decisions.
- [ ] Add CLI commands:
  - `analysis jobs claim`
  - `analysis jobs complete`
  - `analysis run-burst --project <id>`
  - `analysis recommend --project <id>`
  - `analysis review next --project <id> --queue needs-review`
  - `analysis review accept --project <id> --group <id>`
  - `analysis selects --project <id>`
  - `assets --sort group-best-score`
- [ ] Write bridge tests:
  - `mobile_core_returns_asset_groups_with_recommendation_fields`
  - `mobile_core_accepts_group_best_score_sort`
  - `mobile_ffi_saves_custom_strategy_profile`
  - `mobile_core_returns_next_review_card`
  - `mobile_ffi_applies_review_decision`
- [ ] Run `cargo test -p camera_connector_ffi`.
- [ ] Run `cargo test -p camera_connector_cli`.
- [ ] Commit with message `Expose smart selection through FFI and CLI`.

### Task 9: Android Gateway DTOs And Mapping

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeMobileCore.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`
- Modify: `apps/android/app/src/test/java/com/cameraconnector/app/core/NativeDashboardMappingTest.kt`
- Test: `apps/android/app/src/test/java/com/cameraconnector/app/core/SmartSelectionMappingTest.kt`

- [ ] Add Android DTOs: `SmartSelectionSummary`, `SignalScore`, `StrategyProfileUi`, and `PhotoSortMode`.
- [ ] Add Android review DTOs: `ReviewQueueSummary`, `ReviewCard`, `ReviewCandidate`, and `ReviewDecision`.
- [ ] Add Android Selects DTOs for project-level accepted recommendations.
- [ ] Extend `InboxAsset` with burst count, best score, recommendation state, analysis status, stale flag, primary reason, and signal scores.
- [ ] Extend `InboxAssetQuery` with sort, recommendation state, score range, and analysis status.
- [ ] Add `CoreGateway` methods for review queues, next review card, and review decisions.
- [ ] Add `CoreGateway` methods for loading the project Selects collection.
- [ ] Map new JSON fields from `NativeCoreGateway.mapInboxAssets`.
- [ ] Write tests:
  - `mapsSmartSelectionFieldsIntoInboxAsset`
  - `serializesGroupBestScoreSortInQuery`
  - `mapsUnsupportedAnalysisAsNeedsReview`
- [ ] Run Gradle unit tests for the Android app.
- [ ] Commit with message `Map smart selection data on Android`.

### Task 10: Android Settings Strategy Controls

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ElementUi.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Test: `apps/android/app/src/test/java/com/cameraconnector/app/ui/ProjectUiModelsTest.kt`

- [ ] Add a Settings section for smart selection profile controls.
- [ ] Use user-facing Chinese labels instead of raw model names:
  - `更看重清晰度`
  - `更看重曝光`
  - `更看重构图`
  - `高光保护`
  - `阴影保护`
  - `近重复判断强度`
  - `连拍时间窗口`
  - `默认隐藏低分候选`
- [ ] Save edits as a custom profile rather than mutating built-in presets.
- [ ] Trigger recommendation refresh jobs after profile save; do not recompute unchanged quality scores.
- [ ] Add Settings controls for mapping Review mode shortcuts to actions, using input-agnostic names rather than touch-only storage.
- [ ] Add tests for weight clamping and custom profile state.
- [ ] Run Android unit tests.
- [ ] Commit with message `Add smart selection settings controls`.

### Task 11: Android Photo Grid, Detail UI, And Review Mode

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectAssetsScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/PhotoDetailScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectUiModels.kt`
- Create: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ReviewModeScreen.kt`
- Test: `apps/android/app/src/test/java/com/cameraconnector/app/ui/ProjectUiModelsTest.kt`

- [ ] Add grid sorting/filtering controls for latest, filename, and group best score.
- [ ] Add recommendation filters: best, needs review, low score, near duplicate, unsupported, and unreviewed.
- [ ] Render burst groups as stacked photo cards with a small count badge.
- [ ] Render score/recommendation info in a compact edge badge or metadata strip; never cover the center of the thumbnail.
- [ ] In detail, keep the main image dominant and show score summary near the top.
- [ ] Add per-signal rows with Chinese labels: `清晰度`, `曝光`, `构图`, `高光`, `阴影`, `相似度`.
- [ ] Add a burst member filmstrip with badges: `最佳`, `备选`, `低分`, `近重复`, `需复核`.
- [ ] Add manual correction actions for set best, clear recommendation, restore automatic recommendation, merge burst, and split burst.
- [ ] Add Review mode entry from Project Photos with queue chips: `全部待处理`, `只看推荐最佳`, `需复核`, `低分候选`, and `近重复`.
- [ ] Implement Review mode card layout: dominant recommended image, compact filmstrip, score/confidence, one or two Chinese reasons, and progress text such as `12 / 48`.
- [ ] Implement visible Review mode action buttons: accept recommended best, pick another best, mark needs review, hide low-score candidates, keep all, split group, merge group, clear recommendation, and restore automatic recommendation.
- [ ] Add swipe shortcuts only as secondary accelerators; buttons remain required.
- [ ] Add group comparison entry from Review mode and Detail mode that emphasizes sharpness, exposure, composition, and near-duplicate similarity inside the burst.
- [ ] Implement group comparison as enlarged side-by-side or quick-toggle comparison for two or three candidates.
- [ ] Add Review mode session summary when exiting or completing a queue.
- [ ] Add current-session undo control for the latest review decision.
- [ ] Add a Selects entry or filter that shows accepted recommendations for the current project.
- [ ] Keep Android shortcut UI touch-focused for now, but name actions so future PC keyboard/controller shortcuts can reuse them.
- [ ] Add tests for sort/filter state and neutral low-quality wording.
- [ ] Add tests for review queue labels, card progress formatting, decision state mapping, undo visibility, and session summary labels.
- [ ] Run Android unit tests.
- [ ] Commit with message `Add Android smart selection review mode`.

### Task 12: Verification And Documentation

**Files:**
- Modify: `docs/product/android-app-architecture.md`
- Modify: `docs/product/mobile-app-handoff.md`
- Modify: `docs/protocol.md`
- Modify: `docs/superpowers/specs/2026-05-26-smart-selection-design.md`
- Test: full workspace

- [ ] Update docs to describe asynchronous analysis jobs, burst groups, score/recommendation models, user overrides, review queues, Review mode session summary/undo/shortcut configuration, Selects collection, future keyboard/controller shortcut mapping, and group-best-score queries.
- [ ] Run `cargo test`.
- [ ] Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_build.ps1`.
- [ ] Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_emulator_ui.ps1` if an emulator is available.
- [ ] Record any unavailable emulator/device verification in the final implementation notes.
- [ ] Commit with message `Document smart selection workflow`.

## Self-Review Notes

- The plan covers async queueing, burst grouping, local scoring, recommendation models, manual overrides, review queues, Review mode, session summary, current-session undo, shortcut configuration, Selects collection, future keyboard/controller shortcut mapping, versioning, EXIF reserved fields, throttling, fallback states, group-best-score sorting/filtering, FFI/CLI, Android settings, Android preview/detail display, and docs.
- LLM remains optional and disabled by default; no task requires remote processing.
- The plan keeps upload/publish higher priority than analysis and uses durable jobs for recovery.
- The first implementation scopes similarity to the current burst group and avoids full-library visual search.

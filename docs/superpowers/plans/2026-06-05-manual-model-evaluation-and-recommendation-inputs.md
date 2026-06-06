# Manual Model Evaluation and Recommendation Inputs Implementation Plan

> Status: completed and verified. Keep this file as an implementation history
> record; use source code plus the matching smart-selection/model-evaluation
> specs as the current product contract.

**Goal:** Add manual model evaluation entry points and upgrade model recommendation requests so selection can use candidate visual previews in addition to structured evaluation data.

**Architecture:** Core remains the source of truth for task queueing, provider selection, prompt locking, and recommendation persistence. Android supplies preview image data when it has decoded media, while core validates project/provider settings and writes only `model_evaluations` and `selection_recommendations`.

**Tech Stack:** Rust core/FFI, SQLite-backed analysis jobs, Android Kotlin bridge, OpenAI-compatible JSON request adapter.

---

### Task 1: Core Manual Evaluation Queue

**Files:**
- Modify: `core/src/service.rs`
- Modify: `core-ffi/src/lib.rs`
- Test: `core/tests/analysis_job_tests.rs`
- Test: `core-ffi/tests/mobile_core_tests.rs`

- [x] **Step 1: Write failing core test**

Add a test that creates a project, records two asset groups, calls a new service method to enqueue manual model evaluation for both groups, drains the queue with a configured imported provider, and asserts both groups have saved model evaluations.

- [x] **Step 2: Run red test**

Run: `cargo test -p camera_connector_core --test analysis_job_tests manual_model_evaluation_enqueues_selected_asset_groups`

Expected: compile failure because the service method does not exist yet.

- [x] **Step 3: Implement core method**

Add `CameraConnectorService::enqueue_model_evaluation_for_asset_groups(project_id, asset_group_ids)` that validates project ownership, requires project model evaluation to be enabled, enqueues `AnalysisJobType::EvaluateAssetGroupWithModel` with one dedupe key per asset group, and returns the number of enqueued jobs.

- [x] **Step 4: Run green test**

Run: `cargo test -p camera_connector_core --test analysis_job_tests manual_model_evaluation_enqueues_selected_asset_groups`

Expected: pass.

- [x] **Step 5: Add FFI JSON test**

Add a mobile core test that calls the JSON bridge for manual evaluation enqueue and checks the returned count.

- [x] **Step 6: Implement FFI bridge**

Expose a JSON method taking `{ project_id, asset_group_ids }` and returning `{ enqueued_count }`. Do not pass API keys or provider secrets through the bridge.

### Task 2: Android Manual Evaluation Entry Points

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeMobileCore.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/PhotoDetailScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectAssetsScreen.kt`
- Test: existing Android unit mapping tests where tooling is available.

- [x] **Step 1: Add gateway API**

Add a suspend gateway method to enqueue manual model evaluation for selected asset group ids.

- [x] **Step 2: Detail action**

Add a non-blocking detail action labelled as model evaluation/re-evaluation. It should enqueue the current group and show a lightweight confirmation state.

- [x] **Step 3: List multi-select action**

Reuse existing long-press/multi-select patterns if present. If there is no stable multi-select shell, add only the gateway/API surface now and leave UI list selection for the next UI pass.

### Task 3: Recommendation Request Visual Inputs

**Files:**
- Modify: `core/src/analysis/model_eval.rs`
- Modify: `core/src/service.rs`
- Test: `core/tests/model_provider_http_tests.rs`

- [x] **Step 1: Write failing provider request test**

Add a burst recommendation test that passes candidate preview image data URLs into the recommendation function and asserts the mock provider request contains multiple `image_url` parts plus a candidate id mapping.

- [x] **Step 2: Run red test**

Run: `cargo test -p camera_connector_core --test model_provider_http_tests burst_selection_recommendation_sends_candidate_preview_images`

Expected: fail because recommendation currently sends only text context.

- [x] **Step 3: Add candidate visual input model**

Add an internal `SelectionCandidateVisualInput { asset_group_id, image_data_url }` and let provider recommendation include each preview as an image part after the text context.

- [x] **Step 4: Keep storage unchanged**

Continue writing only to `selection_recommendations`. Do not add a new table for recommendation visuals.

- [x] **Step 5: Run green tests**

Run provider HTTP tests and full workspace tests.

### Task 4: Verification

**Files:**
- All touched Rust/Kotlin files.

- [x] **Step 1: Format**

Run: `cargo fmt --all`

- [x] **Step 2: Rust workspace tests**

Run: `cargo test --workspace`

- [x] **Step 3: Android tooling check**

If `apps/android/gradlew` or system Gradle exists, run Android unit tests. If unavailable, report that Android compilation was not run in this environment.

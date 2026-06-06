# Model Provider And Prompt Profile Config Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Move model provider management into a Settings secondary page and redesign prompt profiles around shared photographic preference plus locked task-specific model protocols.

**Architecture:** App-level model provider profiles remain in app-private JSON config; projects only select a provider profile id. Prompt profiles remain SQLite-backed and versioned, but the first implementation exposes only the shared user preference while core composes separate locked prompts for model evaluation, burst selection, and project selection. User-defined input/output fields are not exposed in this version; schema fields required by app parsing stay system-owned.

**Tech Stack:** Android Jetpack Compose, Kotlin gateway DTOs, JNI FFI, Rust core storage/service/model evaluation, SQLite, app-private JSON config, cargo tests, Android unit tests.

---

## File Structure

- Modify `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`: split settings UI into model provider list/editor screen and prompt profile list/editor screen.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/ui/CameraConnectorApp.kt`: add secondary navigation state for model provider config and prompt profile creation.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`: add prompt creation DTO/methods for preview and real gateways.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`: map prompt creation/edit payloads and keep provider config mapping app-level.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeMobileCore.kt`: expose native prompt creation function if Rust FFI needs a dedicated call.
- Modify `core/src/analysis/config.rs`: add structured prompt profile content types while preserving current versioned profile model.
- Modify `core/src/analysis/model_eval.rs`: split prompt composition into evaluation, burst selection, and project selection composers.
- Modify `core/src/service.rs`: create global prompt profiles without copying an existing built-in, and pass the correct prompt block into each model task.
- Modify `core/src/storage/mod.rs`: persist the new prompt version payload without introducing compatibility branches for old development data.
- Modify `core-ffi/src/lib.rs`: expose prompt create/save fields to Android JSON.
- Test `core/tests/evaluation_config_tests.rs`, `core/tests/model_provider_http_tests.rs`, `core/tests/recommendation_tests.rs`, and Android UI/gateway tests under `apps/android/app/src/test/java`.

---

### Task 1: Model Provider Secondary Page

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/CameraConnectorApp.kt`
- Test: `scripts/verify_android_unit_tests.ps1`

- [x] **Step 1: Replace inline provider card with a settings row**

In `SettingsScreen`, keep only a menu row in the Intelligence section:

```kotlin
SettingsMenuRow(
    title = "模型服务",
    subtitle = if (modelProviderSettingsList.isEmpty()) "未配置" else "已配置 ${modelProviderSettingsList.size} 个",
    trailing = ">",
    onClick = onOpenModelProviders,
)
```

Expected behavior: Settings home no longer shows URL/model/API key fields inline.

- [x] **Step 2: Add `ModelProviderProfilesScreen`**

Create a secondary composable in `SettingsScreen.kt` that lists configured provider profiles and includes a top-right "新建" button. Each row opens an editor mode in the same screen state.

```kotlin
@Composable
internal fun ModelProviderProfilesScreen(
    settingsList: List<ModelProviderSettingsUi>,
    editingSettings: ModelProviderSettingsUi?,
    actionsEnabled: Boolean,
    actionError: String?,
    actionInFlight: String?,
    onBack: () -> Unit,
    onNew: () -> Unit,
    onEdit: (String) -> Unit,
    onSave: (ModelProviderSettingsUi) -> Unit,
    onDelete: (String) -> Unit,
    onClearActionError: () -> Unit,
    modifier: Modifier = Modifier,
)
```

Expected behavior: one page handles list, new, edit, and delete without returning to Settings home.

- [x] **Step 3: Add navigation state**

In `CameraConnectorApp.kt`, add:

```kotlin
var settingsModelProvidersOpen by rememberSaveable { mutableStateOf(false) }
var editingModelProviderId by rememberSaveable { mutableStateOf<String?>(null) }
```

Wire back handling so Back exits editor first, then the secondary page.

- [x] **Step 4: Verify Android compile**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_unit_tests.ps1
```

Expected: Android unit tests compile and pass.

---

### Task 2: Prompt Profile New/Create Flow

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/CameraConnectorApp.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeMobileCore.kt`
- Modify: `core/src/service.rs`
- Modify: `core-ffi/src/lib.rs`
- Test: `core/tests/evaluation_config_tests.rs`

- [x] **Step 1: Add failing Rust service test for creating a global prompt**

Add a test that calls a new service method:

```rust
#[test]
fn service_creates_global_prompt_profile_from_user_preference() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(temp_dir.path()).expect("service should create");

    let profile = service
        .create_global_prompt_profile(
            "纪实偏好",
            vec!["纪实".to_string(), "人像".to_string()],
            SceneProfile::General,
            "偏纪实，优先情绪、主体清晰和自然肤色。",
            10_000,
        )
        .expect("prompt profile should create");

    assert_eq!(profile.scope, PromptScope::Global);
    assert!(!profile.built_in);
    assert_eq!(profile.name, "纪实偏好");
    assert!(profile.active_version_id.is_some());
}
```

Run:

```powershell
cargo test -p camera_connector_core --test evaluation_config_tests service_creates_global_prompt_profile_from_user_preference -- --nocapture
```

Expected: fail because `create_global_prompt_profile` does not exist.

- [x] **Step 2: Implement core create method**

Add `CameraConnectorService::create_global_prompt_profile(...)` in `core/src/service.rs`. It creates `PromptProfile` with `PromptScope::Global`, `built_in = false`, `enabled = true`, then creates the first `PromptProfileVersion`.

Use the current output schema version constant and current `stable_prompt_hash`.

- [x] **Step 3: Add FFI and Kotlin gateway method**

Expose a native JSON function:

```kotlin
fun createGlobalPromptProfile(name: String, styleTagsJson: String, sceneProfile: String, promptText: String): JSONObject
```

Add gateway API:

```kotlin
suspend fun createGlobalPromptProfile(
    name: String,
    styleTags: List<String>,
    sceneProfile: String,
    promptText: String,
): PromptProfileUi
```

- [x] **Step 4: Add New button in prompt list**

`PromptProfilesScreen` gets:

```kotlin
onCreatePromptProfile: () -> Unit
```

The header row shows "新建"; click opens editor with `profile = null` and create mode.

- [x] **Step 5: Verify**

Run:

```powershell
cargo test -p camera_connector_core --test evaluation_config_tests
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_unit_tests.ps1
```

Expected: tests pass.

---

### Task 3: Prompt Profile Content Model

**Files:**
- Modify: `core/src/analysis/config.rs`
- Modify: `core/src/analysis/model_eval.rs`
- Modify: `core/src/service.rs`
- Modify: `core/src/storage/mod.rs`
- Modify: `core-ffi/src/lib.rs`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Test: `core/tests/evaluation_config_tests.rs`

- [x] **Step 1: Define structured prompt content**

Add a serializable model in `core/src/analysis/config.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptProfileContent {
    pub shared_preference: String,
    pub evaluation_instruction: Option<String>,
    pub burst_selection_instruction: Option<String>,
    pub project_selection_instruction: Option<String>,
}
```

Keep existing `PromptProfileVersion.prompt_text` as the persisted string for this phase. Store `PromptProfileContent` as JSON text in `prompt_text`.

- [x] **Step 2: Store all seeded and saved prompts as structured JSON**

Implement:

```rust
pub fn prompt_profile_content_json(shared_preference: &str) -> Result<String> {
    serde_json::to_string(&PromptProfileContent {
        shared_preference: shared_preference.trim().to_string(),
        evaluation_instruction: None,
        burst_selection_instruction: None,
        project_selection_instruction: None,
    })
    .map_err(|error| ImporterError::internal(error.to_string()))
}
```

Update built-in prompt seeding and all prompt save paths to persist this JSON string. During development, old plain-text prompt rows can be reset with the rest of the app data; do not add migration branches.

- [x] **Step 3: Update UI DTO**

Add nullable fields to `PromptProfileUi`:

```kotlin
val sharedPreference: String? = null,
val evaluationInstruction: String? = null,
val burstSelectionInstruction: String? = null,
val projectSelectionInstruction: String? = null,
```

Mapping reads structured JSON when present; otherwise maps `activePromptText` to `sharedPreference`.

- [x] **Step 4: Verify**

Run:

```powershell
cargo test -p camera_connector_core --test evaluation_config_tests
cargo test -p camera_connector_core --test model_provider_http_tests
```

Expected: tests pass and existing plain prompts still compose correctly.

---

### Task 4: Split Model Prompt Composition

**Files:**
- Modify: `core/src/analysis/model_eval.rs`
- Modify: `core/src/service.rs`
- Test: `core/tests/model_provider_http_tests.rs`
- Test: `core/tests/recommendation_tests.rs`

- [x] **Step 1: Add failing tests for different task prompts**

Add HTTP provider tests asserting:

- model evaluation request contains shared preference and evaluation instruction;
- burst recommendation request contains shared preference and burst selection instruction;
- project recommendation request contains shared preference and project selection instruction.

Use captured request body assertions already present in `model_provider_http_tests.rs`.

- [x] **Step 2: Replace `user_rubric: &str` with structured content**

Change function inputs:

```rust
pub fn evaluate_asset_group_with_model_provider(..., prompt_content: &PromptProfileContent)
pub fn recommend_selection_with_model_provider(..., prompt_content: &PromptProfileContent)
```

For selection, add an internal enum:

```rust
enum SelectionPromptTask {
    Burst,
    Project,
}
```

- [x] **Step 3: Compose locked prompts by task**

Evaluation prompt:

```text
Shared photographic preference:
{shared_preference}

Evaluation task instruction:
{evaluation_instruction_or_default}
```

Burst selection prompt:

```text
Shared photographic preference:
{shared_preference}

Burst selection instruction:
Pick the best frame within a visually similar burst. Prioritize decisive moment, focus, expression, gesture, subject clarity, and avoid severe technical defects.
```

Project selection prompt:

```text
Shared photographic preference:
{shared_preference}

Project selection instruction:
Select a coherent project-level set. Prefer strong standalone images, diversity, representative coverage, and avoid near-duplicates unless they add clear value.
```

- [x] **Step 4: Keep output protocol locked**

Do not add user-defined output fields. Keep the required evaluation fields and required selection fields in the locked system prompt.

- [x] **Step 5: Verify**

Run:

```powershell
cargo test -p camera_connector_core --test model_provider_http_tests
cargo test -p camera_connector_core --test recommendation_tests
```

Expected: provider request tests show task-specific prompt content and recommendation parsing remains stable.

---

### Task 5: Prompt Editor UI First Version

**Files:**
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/ui/ProjectUiModels.kt`
- Test: Android unit tests or compile verification script.

- [x] **Step 1: Replace one big prompt text box with first-version sections**

In `PromptProfileEditorScreen`, show:

- name
- style tags text field
- scene profile chips
- "我的摄影偏好" editable multiline field
- collapsed read-only "系统评价规则"
- collapsed read-only "连拍优选规则"
- collapsed read-only "项目优选规则"
- read-only "输入输出协议由系统锁定"

- [x] **Step 2: Do not expose custom schema fields**

Show a short locked protocol card:

```text
评价输出：分数、等级、是否可选、摘要、优点、缺点、技术风险
优选输出：选中、备选、淘汰、置信度、理由
```

No add-field UI is included in this version.

- [x] **Step 3: Save structured content JSON**

When saving, serialize the editor state into the core prompt content shape:

```json
{
  "shared_preference": "...",
  "evaluation_instruction": null,
  "burst_selection_instruction": null,
  "project_selection_instruction": null
}
```

- [x] **Step 4: Verify**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_unit_tests.ps1
```

Expected: Kotlin compile and tests pass.

---

### Task 6: End-To-End Verification And Install

**Files:**
- Modify only if verification reveals a defect.

- [x] **Step 1: Run Rust verification**

Run:

```powershell
cargo test -p camera_connector_core --test evaluation_config_tests
cargo test -p camera_connector_core --test model_provider_http_tests
cargo test -p camera_connector_core --test recommendation_tests
cargo fmt --all -- --check
```

Expected: all pass.

- [x] **Step 2: Run Android verification**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_unit_tests.ps1
```

Expected: all pass.

- [x] **Step 3: Install to emulator**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install_android_debug.ps1 -Serial emulator-5554
```

Expected: APK installs and launches.

- [x] **Step 3.5: Run emulator UI route smoke**

Run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_emulator_ui.ps1 -Serial emulator-5554 -SkipInstall
```

Expected: current project-management shell, receiver launcher, Settings secondary
pages, model provider URL/model/API key fields, prompt profile list/new editor,
localized default style tags, and diagnostics route are discoverable on the
emulator.

- [x] **Step 4: Manual smoke check**

In app:

1. Open Settings.
2. Open 模型服务.
3. Create a provider with URL, model, and API key.
4. Return to Settings.
5. Open 提示词库.
6. Create a prompt profile with shared preference.
7. Open a project config page.
8. Select provider and prompt profile.
9. Trigger model evaluation or project recommendation.

Expected: project uses selected provider and selected prompt profile; missing API key disables model work but does not block upload or local CV.

Verified on emulator `emulator-5554` with project `Real Verify` after configuring
provider `openai`:

- `asset_evaluation` manual run completed with `status=ready`, provider
  `openai`, model `gpt-5.4-mini-2026-03-17`, prompt
  `portrait-conservative`.
- `project_recommendation` manual run completed with `status=ready`, provider
  `openai`, model `gpt-5.4-mini-2026-03-17`, prompt
  `portrait-conservative`.
- `selection_recommendations` persisted a project-scoped model recommendation
  with `source=llm`, `status=no_selection`, and rejected the only candidate
  group because the test image was not a usable portrait candidate.

---

## Deferred Scope

- User-defined input/output fields.
- UI rendering of custom model result fields.
- Prompt schema version migration for released user data.
- Per-project prompt override editor separate from global prompt library.
- Advanced editing for task-specific instructions as default-visible UI.


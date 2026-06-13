# Model Provider And Prompt Pack Config Implementation Plan

> Status: completed and verified. Keep this file as implementation history, not
> an active backlog.
>
> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Move model provider management into a Settings secondary page and redesign prompt packs around shared photographic preference plus locked task-specific model protocols.

**Architecture:** App-level model provider profiles remain in app-private JSON config; projects only select a provider profile id. Prompt packs are app-private, file-backed, package-grouped resources under `prompt-packs/<package>/<pack>/` with Markdown preference text in `Prompt.md`; built-in packs are read-only and user edits create user-owned copies. Projects select a prompt pack id while core composes separate locked prompts for model evaluation, burst selection, and project selection. User-defined input/output fields are not exposed in this version; schema fields required by app parsing stay system-owned.

**Tech Stack:** Android Jetpack Compose, Kotlin gateway DTOs, JNI FFI, Rust core storage/service/model evaluation, app-private JSON/file config, SQLite for project state, cargo tests, Android unit tests.

---

## File Structure

- Modify `apps/android/app/src/main/java/com/cameraconnector/app/ui/SettingsScreen.kt`: split settings UI into model provider list/editor screen and prompt pack list/editor screen.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/ui/CameraConnectorApp.kt`: add secondary navigation state for model provider config and prompt pack creation.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`: add prompt creation DTO/methods for preview and real gateways.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeCoreGateway.kt`: map prompt creation/edit payloads and keep provider config mapping app-level.
- Modify `apps/android/app/src/main/java/com/cameraconnector/app/core/NativeMobileCore.kt`: expose native prompt creation function if Rust FFI needs a dedicated call.
- Modify `core/src/analysis/config.rs`: keep locked task-specific prompt composition over a user-editable Markdown preference block.
- Modify `core/src/analysis/model_eval.rs`: split prompt composition into evaluation, burst selection, and project selection composers.
- Modify `core/src/service.rs`: create/fork/save global prompt packs and pass the correct preference block into each model task.
- Modify `core/src/storage/mod.rs`: keep prompt packs in app-private state files; do not add compatibility branches for discarded development data.
- Modify `core-ffi/src/lib.rs`: expose prompt pack create/fork/save fields to Android JSON.
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

### Task 2: Prompt Pack New/Create Flow

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
fn service_creates_global_prompt_pack_from_user_preference() {
    let temp_dir = tempfile::tempdir().expect("temp dir should create");
    let service = CameraConnectorService::new(temp_dir.path()).expect("service should create");

    let profile = service
        .create_global_prompt_pack(
            "纪实偏好",
            vec!["纪实".to_string(), "人像".to_string()],
            SceneProfile::General,
            "偏纪实，优先情绪、主体清晰和自然肤色。",
            10_000,
        )
        .expect("prompt pack should create");

    assert_eq!(profile.scope, PromptScope::Global);
    assert!(!profile.built_in);
    assert_eq!(profile.name, "纪实偏好");
    assert!(profile.active_version_id.is_some());
}
```

Run:

```powershell
cargo test -p camera_connector_core --test evaluation_config_tests service_creates_global_prompt_pack_from_user_preference -- --nocapture
```

Expected: fail because `create_global_prompt_pack` does not exist.

- [x] **Step 2: Implement core create method**

Add `CameraConnectorService::create_global_prompt_pack(...)` in `core/src/service.rs`. It creates a user-owned prompt pack under the selected package folder, writes the editable preference text to `Prompt.md`, and returns the prompt pack metadata used by Android.

Keep input/output protocol and task-specific instructions system-owned. Do not create SQLite prompt pack/version rows.

- [x] **Step 3: Add FFI and Kotlin gateway method**

Expose a native JSON function:

```kotlin
fun createGlobalPromptPack(name: String, styleTagsJson: String, sceneProfile: String, promptText: String): JSONObject
```

Add gateway API:

```kotlin
suspend fun createGlobalPromptPack(
    name: String,
    styleTags: List<String>,
    sceneProfile: String,
    promptText: String,
): PromptPackUi
```

- [x] **Step 4: Add New button in prompt list**

`PromptPacksScreen` gets:

```kotlin
onCreatePromptPack: () -> Unit
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

### Task 3: Prompt Pack Markdown Content

**Files:**
- Modify: `core/src/analysis/config.rs`
- Modify: `core/src/analysis/model_eval.rs`
- Modify: `core/src/service.rs`
- Modify: `core/src/storage/mod.rs`
- Modify: `core-ffi/src/lib.rs`
- Modify: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Test: `core/tests/evaluation_config_tests.rs`

- [x] **Step 1: Keep editable content as Markdown**

Use the user-editable `Prompt.md` text as the photographic preference block. The app may wrap it in an internal serialized state file for pack metadata, but the distributed and user-authored artifact remains Markdown:

```
prompt-packs/<package>/<pack>/Prompt.md
```

System prompt text, task instructions, and input/output JSON schema are generated by core and are not user-editable in this phase.

- [x] **Step 2: Store all seeded and saved prompts as prompt packs**

Update built-in prompt seeding and all prompt save paths to persist package metadata plus `Prompt.md`. During development, discarded prompt table data can be reset with the rest of the app data; do not add migration branches.

- [x] **Step 3: Update UI DTO**

Keep `PromptPackUi.activePromptText` / `sharedPreference` mapped to the Markdown preference text so existing Android selection and preview screens can render or edit the same content:

```kotlin
val activePromptText: String? = null
val sharedPreference: String? = null
```

Mapping should not require users to see or edit JSON.

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

- model evaluation request contains the prompt pack preference and locked evaluation instruction;
- burst recommendation request contains the prompt pack preference and locked burst selection instruction;
- project recommendation request contains the prompt pack preference and locked project selection instruction.

Use captured request body assertions already present in `model_provider_http_tests.rs`.

- [x] **Step 2: Replace `user_rubric: &str` with prompt pack preference input**

Change function inputs:

```rust
pub fn evaluate_asset_group_with_model_provider(..., prompt_preference_markdown: &str)
pub fn recommend_selection_with_model_provider(..., prompt_preference_markdown: &str)
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
Photographic preference:
{prompt_preference_markdown}

Locked evaluation task instruction:
{system_owned_evaluation_instruction}
```

Burst selection prompt:

```text
Photographic preference:
{prompt_preference_markdown}

Locked burst selection instruction:
Pick the best frame within a visually similar burst. Prioritize decisive moment, focus, expression, gesture, subject clarity, and avoid severe technical defects.
```

Project selection prompt:

```text
Photographic preference:
{prompt_preference_markdown}

Locked project selection instruction:
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

In `PromptPackEditorScreen`, show:

- name
- style tags text field
- scene profile chips
- "我的摄影偏好" editable multiline field
- read-only "输入输出协议由系统锁定"

- [x] **Step 2: Do not expose custom schema fields**

Show a short locked protocol card:

```text
评价输出：分数、等级、是否可选、摘要、优点、缺点、技术风险
优选输出：选中、备选、淘汰、置信度、理由
```

No add-field UI is included in this version.

- [x] **Step 3: Save Markdown preference content**

When saving, write only the editable photography preference Markdown and pack metadata through the core prompt pack API:

```text
prompt-packs/<package>/<pack>/Prompt.md
```

The locked system prompt, task instructions, and output schema remain generated by core at request time.

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
pages, model provider URL/model/API key fields, prompt pack list/new editor,
localized default style tags, and diagnostics route are discoverable on the
emulator.

- [x] **Step 4: Manual smoke check**

In app:

1. Open Settings.
2. Open 模型服务.
3. Create a provider with URL, model, and API key.
4. Return to Settings.
5. Open 提示词库.
6. Create a prompt pack with shared preference.
7. Open a project config page.
8. Select provider and prompt pack.
9. Trigger model evaluation or project recommendation.

Expected: project uses selected provider and selected prompt pack; missing API key disables model work but does not block upload or local CV.

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


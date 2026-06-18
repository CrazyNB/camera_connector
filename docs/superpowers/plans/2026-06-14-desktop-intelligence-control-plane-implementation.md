# Desktop Intelligence Control Plane Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the desktop command and TypeScript control-plane foundation for provider profiles, prompt packs, project evaluation settings, and manual model evaluation.

**Architecture:** Desktop remains a thin adapter over `CameraConnectorService`. Tauri commands expose desktop-friendly DTOs with snake_case string values so the frontend does not depend on Rust enum variant names. The first UI slice is intentionally compact: a loadable intelligence state and actions; full prompt editing UI can build on it later.

**Tech Stack:** Rust/Tauri commands, `camera_connector_core`, TypeScript/Vite desktop app, Node logic tests, Cargo checks.

---

### Task 1: Desktop Intelligence DTOs And Tauri Commands

**Files:**
- Modify: `apps/desktop/src-tauri/src/commands.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`

- [ ] **Step 1: Add command DTOs near existing request structs**

Add request and response structs for provider settings, prompt packs, project evaluation settings, and manual model evaluation. Use string fields for enum-like values:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct SaveModelProviderSettingsRequest {
    pub settings_id: String,
    pub provider_kind: String,
    pub provider_label: String,
    pub base_url: String,
    pub default_model: String,
    pub default_max_image_side: i64,
    pub default_send_mode: String,
    pub default_batch_size: i64,
    pub configured: bool,
    pub api_key: Option<String>,
    pub key_alias: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopModelProviderSettings {
    pub settings_id: String,
    pub provider_kind: String,
    pub provider_label: String,
    pub base_url: String,
    pub default_model: String,
    pub default_max_image_side: i64,
    pub default_send_mode: String,
    pub default_batch_size: i64,
    pub configured: bool,
    pub api_key_configured: bool,
    pub key_alias: Option<String>,
    pub updated_at_ms: i64,
}
```

- [ ] **Step 2: Add mapping helpers**

Implement helpers that map core enums through their existing `from_str` and `as_str` methods:

```rust
fn desktop_model_provider_settings(
    settings: camera_connector_core::ModelProviderSettings,
) -> DesktopModelProviderSettings {
    DesktopModelProviderSettings {
        settings_id: settings.settings_id,
        provider_kind: settings.provider_kind.as_str().to_string(),
        provider_label: settings.provider_label,
        base_url: settings.base_url,
        default_model: settings.default_model,
        default_max_image_side: settings.default_max_image_side,
        default_send_mode: settings.default_send_mode.as_str().to_string(),
        default_batch_size: settings.default_batch_size,
        configured: settings.configured,
        api_key_configured: settings.api_key_configured,
        key_alias: settings.key_alias,
        updated_at_ms: settings.updated_at_ms,
    }
}
```

- [ ] **Step 3: Add Tauri commands**

Add commands that call existing core service APIs:

```rust
#[tauri::command]
pub fn get_model_provider_settings_list(
    state: State<'_, DesktopState>,
) -> Result<Vec<DesktopModelProviderSettings>, DesktopError> {
    state
        .service
        .model_provider_settings_list()
        .map(|items| items.into_iter().map(desktop_model_provider_settings).collect())
        .map_err(desktop_error)
}
```

Repeat this pattern for save/delete provider, load/save project evaluation settings, list/fork/create/save/delete prompt packs, and enqueue model evaluation.

- [ ] **Step 4: Register commands**

Add the new command names to `tauri::generate_handler!` in `apps/desktop/src-tauri/src/lib.rs`.

- [ ] **Step 5: Verify Rust compile**

Run:

```powershell
cargo check -p camera-connector-desktop
```

Expected: command signatures compile and no serde or missing import errors remain.

### Task 2: TypeScript API And State Foundation

**Files:**
- Modify: `apps/desktop/src/main.ts`
- Create: `apps/desktop/src/intelligence.ts`
- Create: `apps/desktop/test/intelligence.test.ts`
- Modify: `apps/desktop/package.json`

- [ ] **Step 1: Extract frontend types and helpers**

Create `apps/desktop/src/intelligence.ts` with TypeScript types matching the Tauri DTOs and pure helpers:

```ts
export type ModelProviderSettings = {
  settings_id: string;
  provider_kind: string;
  provider_label: string;
  base_url: string;
  default_model: string;
  default_max_image_side: number;
  default_send_mode: string;
  default_batch_size: number;
  configured: boolean;
  api_key_configured: boolean;
  key_alias?: string | null;
  updated_at_ms: number;
};

export type ProjectEvaluationSettings = {
  project_id: string;
  auto_evaluate_on_upload: boolean;
  auto_burst_recommendation_enabled: boolean;
  project_recommendation_mode: "manual";
  prompt_pack_id?: string | null;
  model_provider_settings_id?: string | null;
  scene_profile: string;
  cv_policy: string;
  allow_risky_model_selects: boolean;
  max_image_side?: number | null;
  batch_size?: number | null;
  updated_at_ms: number;
};

export function intelligenceSetupState(
  providers: ModelProviderSettings[],
  settings: ProjectEvaluationSettings | null,
  promptCount: number,
) {
  const selectedProvider = providers.find(
    (provider) => provider.settings_id === settings?.model_provider_settings_id,
  );
  return {
    providerReady: Boolean(selectedProvider?.configured && selectedProvider.api_key_configured),
    promptReady: Boolean(settings?.prompt_pack_id),
    promptCount,
    autoEvaluate: Boolean(settings?.auto_evaluate_on_upload),
  };
}
```

- [ ] **Step 2: Add failing tests**

Create `apps/desktop/test/intelligence.test.ts`:

```ts
import assert from "node:assert/strict";
import test from "node:test";
import { intelligenceSetupState } from "../src/intelligence.js";

test("intelligenceSetupState requires configured provider with api key", () => {
  assert.deepEqual(
    intelligenceSetupState(
      [{ settings_id: "p1", provider_kind: "openai", provider_label: "OpenAI", base_url: "", default_model: "gpt", default_max_image_side: 1536, default_send_mode: "preview_only", default_batch_size: 1, configured: true, api_key_configured: false, updated_at_ms: 0 }],
      { project_id: "project", auto_evaluate_on_upload: true, auto_burst_recommendation_enabled: true, project_recommendation_mode: "manual", prompt_pack_id: "general-default", model_provider_settings_id: "p1", scene_profile: "general", cv_policy: "standard", allow_risky_model_selects: false, updated_at_ms: 0 },
      1,
    ),
    { providerReady: false, promptReady: true, promptCount: 1, autoEvaluate: true },
  );
});
```

- [ ] **Step 3: Wire API wrappers in `main.ts`**

Add invoke wrappers for:

```ts
getModelProviderSettingsList()
saveModelProviderSettings(request)
deleteModelProviderSettings(settingsId)
getProjectEvaluationSettings(projectId)
saveProjectEvaluationSettings(settings)
getGlobalPromptPacks()
getProjectPromptPacks(projectId)
createGlobalPromptPack(request)
forkGlobalPromptPack(request)
saveGlobalPromptPack(request)
deleteGlobalPromptPack(promptPackId)
enqueueModelEvaluation(projectId, groupIds)
```

- [ ] **Step 4: Add state fields**

Extend `AppState` with:

```ts
intelligenceProviders: ModelProviderSettings[];
intelligenceSettings: ProjectEvaluationSettings | null;
promptPacks: PromptPack[];
intelligenceOpen: boolean;
```

Load these alongside the selected project in `refreshCurrentProject`.

- [ ] **Step 5: Update logic test command**

Append `.tmp-test/test/intelligence.test.js` to `npm run test:logic`.

- [ ] **Step 6: Verify TypeScript tests**

Run:

```powershell
npm.cmd run test:logic
```

Expected: all logic tests pass.

### Task 3: Minimal Desktop Control Surface

**Files:**
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/src/styles.css`
- Test: `npm.cmd run build`

- [ ] **Step 1: Add compact summary render**

Add a small project intelligence summary in the left/project control area with provider, prompt, and auto-evaluation state.

- [ ] **Step 2: Add editor drawer shell**

Add a modal or drawer that can toggle auto evaluation, auto burst recommendation, risk participation, scene profile, provider profile, and prompt pack selection. Save through `saveProjectEvaluationSettings`.

- [ ] **Step 3: Add manual model evaluation action**

Add an action for the current group and viewer current group that calls `enqueueModelEvaluation(projectId, [groupId])`, then drains analysis jobs and refreshes the project.

- [ ] **Step 4: Verify build**

Run:

```powershell
npm.cmd run build
```

Expected: TypeScript and Vite build pass.

### Task 4: Final Verification

**Files:**
- All files changed in Tasks 1-3

- [ ] **Step 1: Rust formatting and check**

Run:

```powershell
cargo fmt --all --check
cargo check -p camera-connector-desktop
```

Expected: both pass.

- [ ] **Step 2: Frontend tests and build**

Run:

```powershell
npm.cmd run test:logic
npm.cmd run build
```

Expected: both pass.

- [ ] **Step 3: Diff hygiene**

Run:

```powershell
git diff --check
git status --short
```

Expected: no whitespace errors. Status shows only intended desktop intelligence implementation files plus pre-existing worktree changes.

## Self-Review

- Spec coverage: provider profiles, prompt packs, project evaluation settings, manual model evaluation, no-key behavior, and semantic separation are covered.
- Placeholder scan: no TBD/TODO placeholders are present.
- Type consistency: desktop DTO fields use snake_case strings that match Android/Core JSON conventions.

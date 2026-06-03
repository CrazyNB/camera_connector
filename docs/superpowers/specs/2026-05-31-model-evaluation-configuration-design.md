# Model Evaluation Configuration Design

## Status

Approved direction from product discussion on 2026-05-31.

This document extends `2026-05-31-smart-selection-evaluation-redesign.md`.
The earlier smart-selection redesign defines the split between local technical
gate, model evaluation, scoped recommendations, and user marks. This document
locks down how those abilities are configured, how prompts are managed, when
recommendations run, and how portrait-specific local CV fits the project model.

## Problem

The app now has the right high-level intelligence direction, but the control
surface is still underspecified:

- Model evaluation and model recommendation must be configurable.
- Evaluation prompts must be editable, versioned, and named by style tags.
- Group-level recommendation and project-level recommendation have different
  trigger semantics.
- API-key absence must be explicit and must not silently pretend a local stub is
  a real model result.
- General local CV should stay universal, while portrait projects need extra
  subject-aware checks.

## Decisions

### Configuration Scope

Use three configuration scopes.

```text
Global app settings
  -> provider capability, credentials, default model, default prompt library

Project evaluation settings
  -> whether this project uses model evaluation and which prompt/profile to use

Run snapshot
  -> immutable record of the prompt/model/settings used for one evaluation or
     recommendation run
```

The main product decision is:

```text
Model capability = global
Model evaluation enablement = project-level
Burst recommendation enablement = project-level
Project recommendation = manual run
```

Global settings may define defaults for newly created projects, but those
defaults are copied into the project settings at creation time. After a project
exists, changing a global default must not silently flip that project's model
evaluation behavior.

New projects should default to model evaluation disabled unless the user
explicitly chooses a different project template/default. If global provider
settings are configured, the create/edit project flow can expose an explicit
"enable model evaluation" switch.

The project setting is the source of truth for whether automatic model
evaluation runs. The global provider state only answers whether model work can
run at all. In other words, provider/model capability is global, but model
evaluation enablement is project-level. Global defaults affect new projects only
and must not silently mutate existing project settings.

### Recommendation Triggers

Group-level recommendation:

- Can run automatically after a burst group is stable.
- Is controlled by the project's `auto_burst_recommendation_enabled`.
- Produces `selection_recommendation.scope = burst_group`.

Project-level recommendation:

- Must be manually triggered by the user.
- Produces `selection_recommendation.scope = project`.
- Must not be produced by upload drains, background drains, or burst-stable
  events.
- Must never be continuously rewritten in the background.
- Can be regenerated, but each regeneration creates a new run record.
- Is disabled when provider capability is missing, except for imported
  evaluation/recommendation data that is already present.

### Human State And Model State

Keep these states separate:

- Model select: algorithm recommendation.
- Favorite: user collection/preference.
- Marked: user backup/candidate marker.

Favorite and marked actions must never overwrite model recommendations.
Model recommendations must never imply user acceptance.

## Data Model

The project is still in development, so schema changes can replace current
development-stage smart-selection tables without historical compatibility logic.

### Non-Secret Global Model Settings

Store non-secret global defaults in app/core settings.

```text
model_provider_settings
  settings_id TEXT PRIMARY KEY
  provider_kind TEXT NOT NULL        -- none | openai | custom | imported
  provider_label TEXT NOT NULL
  default_model TEXT NOT NULL
  default_max_image_side INTEGER NOT NULL
  default_send_mode TEXT NOT NULL    -- preview_only | review_image
  default_batch_size INTEGER NOT NULL
  configured INTEGER NOT NULL        -- provider usable from app point of view
  updated_at_ms INTEGER NOT NULL
```

Sensitive API keys are not stored in SQLite. Android stores them in encrypted
app storage / keystore and passes only capability state into core APIs.

### Prompt Profiles

Prompt profiles are named, tagged, and versioned.

```text
prompt_profiles
  prompt_profile_id TEXT PRIMARY KEY
  scope TEXT NOT NULL                -- global | project
  project_id TEXT NULL
  name TEXT NOT NULL                 -- e.g. Portrait Conservative
  style_tags_json TEXT NOT NULL      -- ["portrait", "conservative"]
  scene_profile TEXT NOT NULL        -- general | portrait | action | landscape | custom
  active_version_id TEXT NULL
  built_in INTEGER NOT NULL
  enabled INTEGER NOT NULL
  created_at_ms INTEGER NOT NULL
  updated_at_ms INTEGER NOT NULL

prompt_profile_versions
  prompt_version_id TEXT PRIMARY KEY
  prompt_profile_id TEXT NOT NULL
  prompt_text TEXT NOT NULL
  output_schema_version TEXT NOT NULL
  prompt_hash TEXT NOT NULL
  created_at_ms INTEGER NOT NULL
```

Editing a built-in prompt creates a project-scoped copy. Editing any prompt
creates a new version. Existing evaluations keep their old prompt version and
prompt hash.

Every model evaluation and recommendation run records the active prompt profile
id, prompt version id, and prompt hash in its run snapshot. Built-in prompts are
read-only; editing one means forking it into an editable project-scoped profile
first, then creating a new immutable version.

The output schema remains app-controlled. Users edit the evaluation language and
rubric, not the JSON contract.

Prompt names and style tags are product-facing organization tools. A profile can
be named by intent, such as `Portrait Conservative` or `Documentary Street`, and
tagged with short style labels such as `portrait`, `conservative`, `film`,
`landscape`, or `low-saturation`. Tags do not change behavior by themselves; the
selected prompt version and project scene profile determine behavior.

### Project Evaluation Settings

```text
project_evaluation_settings
  project_id TEXT PRIMARY KEY
  model_evaluation_enabled INTEGER NOT NULL
  auto_evaluate_on_upload INTEGER NOT NULL
  auto_burst_recommendation_enabled INTEGER NOT NULL
  project_recommendation_mode TEXT NOT NULL  -- manual
  prompt_profile_id TEXT NULL
  scene_profile TEXT NOT NULL                -- general | portrait | action | landscape | custom
  cv_policy TEXT NOT NULL                    -- loose | standard | strict
  allow_risky_model_selects INTEGER NOT NULL
  max_image_side INTEGER NULL
  batch_size INTEGER NULL
  updated_at_ms INTEGER NOT NULL
```

`project_recommendation_mode` is intentionally constrained to `manual` for the
current product direction.

### Evaluation Runs

Each model or recommendation execution gets an immutable run snapshot.

```text
evaluation_runs
  run_id TEXT PRIMARY KEY
  project_id TEXT NOT NULL
  run_type TEXT NOT NULL              -- asset_evaluation | burst_recommendation | project_recommendation
  trigger TEXT NOT NULL               -- upload | burst_stable | manual | retry
  status TEXT NOT NULL                -- pending | running | ready | failed | skipped
  provider_kind TEXT NOT NULL
  provider_model TEXT NOT NULL
  prompt_profile_id TEXT NULL
  prompt_version_id TEXT NULL
  prompt_hash TEXT NULL
  settings_snapshot_json TEXT NOT NULL
  error_message TEXT NULL
  started_at_ms INTEGER NULL
  completed_at_ms INTEGER NULL
  created_at_ms INTEGER NOT NULL
```

`model_evaluations` and `selection_recommendations` reference `run_id`.

### Model Evaluations

Extend the existing target model evaluation shape with run and prompt identity.

```text
model_evaluations
  evaluation_id TEXT PRIMARY KEY
  run_id TEXT NOT NULL
  project_id TEXT NOT NULL
  asset_group_id TEXT NOT NULL
  evaluator_kind TEXT NOT NULL        -- llm_vlm | local_stub | imported
  evaluator_version TEXT NOT NULL
  status TEXT NOT NULL
  score INTEGER NOT NULL
  tier TEXT NOT NULL
  selectable INTEGER NOT NULL
  summary TEXT NOT NULL
  strengths_json TEXT NOT NULL
  weaknesses_json TEXT NOT NULL
  technical_warnings_json TEXT NOT NULL
  prompt_profile_id TEXT NULL
  prompt_version_id TEXT NULL
  prompt_hash TEXT NULL
  created_at_ms INTEGER NOT NULL
  updated_at_ms INTEGER NOT NULL
```

The local stub remains a development/provider fallback only. The UI must not
label stub results as real model evaluation. Any row produced by this path must
store `evaluator_kind = local_stub` and expose that source to UI/API callers.

### Portrait Subject Assessments

General technical assessment remains universal. Portrait projects add a
subject-aware assessment. The trigger is the project `scene_profile`, not
automatic guessing from uploaded photos.

```text
subject_assessments
  assessment_id TEXT PRIMARY KEY
  project_id TEXT NOT NULL
  asset_group_id TEXT NOT NULL
  subject_type TEXT NOT NULL          -- face | person
  detector_kind TEXT NOT NULL         -- android_mlkit | opencv | imported | none
  detector_version TEXT NOT NULL
  status TEXT NOT NULL                -- pending | ready | failed | skipped
  gate_status TEXT NOT NULL           -- pass | warn | reject | needs_review | unsupported
  regions_json TEXT NOT NULL          -- face boxes and landmarks if available
  signals_json TEXT NOT NULL          -- closed eyes, face exposure, face color cast, face sharpness
  summary TEXT NOT NULL
  created_at_ms INTEGER NOT NULL
  updated_at_ms INTEGER NOT NULL
```

Android can implement a detector later with an on-device pre-trained face
detector, but this contract does not require Android to add an ML dependency
now. Core/FFI own only the storage and interpretation contract so Android, PC,
or imported sources can provide detector outputs.

Portrait subject assessment is still a gate/risk signal. It can flag closed
eyes, face-region exposure failure, face-region color cast, and face sharpness
risk, but it must not be treated as the final aesthetic score. The model
evaluation prompt may consume these signals as context.

Only projects with `scene_profile = portrait` schedule portrait subject
assessment. General, action, landscape, and custom projects do not schedule it
unless they are explicitly changed to the portrait scene profile.

## Pipeline

### Upload Path

```text
publish completed
  -> create persistent thumbnail
  -> detect/refine burst group
  -> enqueue universal technical assessment
  -> if project.scene_profile == portrait:
       enqueue portrait subject assessment
  -> if project.model_evaluation_enabled
       and project.auto_evaluate_on_upload
       and provider is configured:
       enqueue asset model evaluation
  -> if provider missing:
       mark model work as skipped/unconfigured, keep technical gate visible
  -> if auto_burst_recommendation_enabled:
       enqueue burst recommendation when members have evaluations or timeout
```

### Manual Project Recommendation

```text
user taps Generate Project Selects
  -> verify provider configured
  -> create evaluation_run(run_type = project_recommendation, trigger = manual)
  -> candidate set = non-burst asset groups + burst winners
  -> optionally include marked/favorite context as user signals, not as truth
  -> call model provider or imported evaluator
  -> write scoped selection_recommendation(scope = project)
```

If provider is missing, the action is disabled with a direct settings CTA.

Project recommendation must not be triggered from upload drains, background
analysis drains, or burst-stable events.

### Prompt Edits

```text
user edits prompt profile
  -> create new prompt_profile_version
  -> set active_version_id
  -> mark project model evaluations stale by prompt hash mismatch
  -> do not delete old evaluations
```

## UI Design

### Global Settings

Global settings show:

- Provider connection status.
- API key configuration entry.
- Default model.
- Default send mode: preview only / larger review image.
- Prompt library management.

### Project Settings

Project settings show:

- Model evaluation switch.
- Auto evaluate on upload switch.
- Auto group recommendation switch.
- Scene profile selector.
- Prompt profile selector and edit action.
- CV policy selector.
- Risk-photo participation switch.
- Manual project recommendation status and last run metadata.

### Project Photos

Photo grid filters remain concise:

- All.
- Model Selects.
- Favorites.
- Marked.
- Quality Risk.
- Pending Analysis.

The Model Selects filter means project-scope selected works. Burst winners can
drive burst covers and group-detail badges, but they must not appear in the
project-level Model Selects filter unless a project recommendation selected them.
Project recommendation is visible as a normal filter/collection and is not
merged into favorites.

## No-Key Behavior

When API key/provider is missing:

- Upload, thumbnail, grouping, and local CV still run.
- Model evaluation jobs are not sent to a remote provider.
- Model evaluation rows are skipped/disabled rather than fabricated.
- Project-level recommendation action is disabled.
- UI shows "model provider not configured" rather than "waiting forever".
- Existing imported evaluations remain readable.
- Development local stub can be used only when explicitly enabled as a dev
  evaluator; it must show source `local_stub`.
- Project-level "Generate Project Selects" remains disabled until provider
  capability exists.

## Acceptance Criteria

### Configuration

- Global provider settings can represent configured and unconfigured states
  without storing an API key in SQLite.
- Project settings can independently enable/disable model evaluation.
- A project can select scene profile, prompt profile, CV policy, batch size, and
  image send mode overrides.
- New projects default to model evaluation disabled.

### Prompt Profiles

- Built-in prompt profiles can be listed with style tags.
- Editing a built-in prompt creates a project-scoped editable copy.
- Editing any prompt creates a new immutable version.
- Model evaluations store prompt profile id, prompt version id, and prompt hash.
- A prompt edit does not rewrite old model evaluation rows.

### Pipeline

- Universal technical assessment runs even when model evaluation is disabled.
- Portrait subject assessment runs only for portrait projects.
- Auto model evaluation runs only when project setting is enabled and provider
  is configured.
- Auto burst recommendation is project-configurable.
- Project recommendation is manual-only.
- Missing provider state is explicit and testable.

### Recommendation Semantics

- Group recommendation and project recommendation use separate scopes.
- A burst winner does not automatically become a project select.
- User favorite and marked state do not mutate model recommendation rows.
- Project recommendation regeneration creates a new run snapshot.

### UI

- Settings exposes global model/provider defaults and project-level evaluation
  settings.
- Prompt names and style tags are visible and editable.
- Project recommendation is triggered by a manual action.
- No-key state points the user to global provider settings.
- Stub/imported/real model sources are visually distinguishable.

## Non-Goals

- Building a cloud account system.
- Auto-deleting photos.
- Treating local CV as final aesthetic judgment.
- Running project-level recommendation automatically in the background.
- Implementing historical migration for older development-stage smart-selection
  data.

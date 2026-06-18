# Desktop Intelligence Control Plane Design

## Goal

Bring the desktop workbench to feature parity with Android for project
intelligence: technical risk, project evaluation settings, prompt pack
management, model provider configuration, and LLM/VLM evaluation.

This is a control-plane slice. It should make desktop use the same core
semantics as Android without creating a separate desktop-only prompt,
evaluation, or recommendation model.

## Context

The desktop review surface already scans local folders, displays grouped
assets, shows technical/model/recommendation status fields, drains analysis
jobs, and can trigger burst or project recommendation actions.

The missing piece is configuration and evaluation orchestration. Android already
uses core gateway methods for provider profiles, prompt packs, project
evaluation settings, manual model evaluation, and risk-aware recommendation
state. Desktop should expose the same capabilities through Tauri commands and a
compact project intelligence UI.

## Decisions

- Core remains the source of truth for provider settings, prompt packs, project
  evaluation settings, analysis jobs, technical assessments, model evaluations,
  selection recommendations, and user marks.
- Desktop Tauri commands should mirror Android gateway semantics rather than
  exposing table-level mutation.
- Project intelligence settings are project-scoped. Global defaults may prefill
  a new project, but changing global provider or prompt configuration must not
  silently rewrite existing projects.
- `auto_evaluate_on_upload` is interpreted by desktop as
  `auto_evaluate_on_scan`.
- Project recommendation remains manual. Scan completion and background drains
  may evaluate assets or burst groups, but they must not create project-scope
  recommendations automatically.
- No-key behavior is explicit: scanning, previews, grouping, local technical
  risk, and human marks continue; model evaluation and model recommendation are
  skipped or disabled with visible setup state.
- There is no compatibility or migration work in this slice. During current
  desktop development, stale local state can be deleted and recreated from the
  current schema.

## Semantic Boundaries

These concepts must stay separate in desktop UI and commands:

- `technical_assessments`: local CV risk and gate context. This is not a final
  photographic score.
- `model_evaluations`: model photographic score, tier, selectable flag,
  summary, strengths, weaknesses, technical warnings, provider metadata, and
  prompt snapshot metadata.
- `selection_recommendations`: model-generated selections for burst or project
  scopes.
- user favorite and mark state: human choices that never mutate model
  evaluation or recommendation rows.

Accepting, clearing, favoriting, marking, deleting, or removing a photo from a
burst must not rewrite model evaluation rows. New model work should produce new
evaluation or recommendation records through core service APIs.

## Desktop Command Surface

Add Tauri commands in the desktop gateway for the existing core service APIs:

- `get_model_provider_settings_list`
- `save_model_provider_settings`
- `delete_model_provider_settings`
- `get_project_evaluation_settings`
- `save_project_evaluation_settings`
- `get_global_prompt_packs`
- `get_project_prompt_packs`
- `create_global_prompt_pack`
- `fork_global_prompt_pack`
- `save_global_prompt_pack`
- `delete_global_prompt_pack`
- `enqueue_model_evaluation_for_asset_groups`

Keep the existing commands:

- `drain_analysis_jobs`
- `recommend_burst_group`
- `generate_project_recommendation`

The command layer should translate only request/response DTOs and errors. It
should not manually insert rows into analysis, prompt, or recommendation tables.

## Desktop UI Model

Do not bring back a multi-step wizard. The review page remains the primary
workspace. Intelligence configuration appears as a compact project control
surface:

- A project intelligence summary near the project/source controls shows provider
  status, selected prompt, scene profile, auto-evaluation status, and risk
  policy.
- A details drawer or modal edits the selected project's intelligence settings:
  scene profile, provider profile, prompt pack, auto model evaluation,
  automatic burst recommendation, risky-selection participation, and technical
  risk threshold policy.
- Prompt management is secondary. Users can list prompt packs, preview Markdown,
  fork built-in packs, and edit user-owned prompt preference text.
- The review grid and viewer show risk, model evaluation, and recommendation
  status as photo metadata, not as setup controls.
- Manual model evaluation remains available on selected groups and in viewer
  mode. Manual project recommendation remains an explicit top-level action.

The UI should avoid teaching the user internal workflow phases. The mental model
is: choose a project and folder, then review while intelligence progressively
becomes available.

## Prompt Pack Rules

Prompt packs are global, package-grouped, Markdown-backed photographic
preference resources.

- Built-in packs are read-only.
- Editing a built-in pack forks it into a user-owned prompt pack.
- Projects select a prompt pack id.
- The model request protocol, task instructions, JSON schema, and output
  parsing remain generated by core.
- Desktop must not expose arbitrary system prompt editing in this slice.

## Evaluation Flow

On scan completion:

1. Core groups assets.
2. Desktop refreshes the project dashboard.
3. Desktop drains or schedules analysis according to project evaluation
   settings.
4. Local technical assessment may run without a provider.
5. Model evaluation runs only when project settings enable it and the selected
   provider is configured.
6. Burst recommendation may run automatically only when project settings enable
   it and enough model evaluation data exists.
7. Project recommendation remains manual.

Manual actions may enqueue model evaluation or recommendation work even when
automatic evaluation is off, as long as provider capability exists.

## Error And Empty States

- Missing provider: show setup required; do not block scanning or local risk.
- Missing API key: show provider incomplete; do not send model work.
- Missing prompt pack: block model work for that project until a prompt is
  selected or a valid default is saved.
- Provider failure: keep existing technical assessment and human marks; surface
  the model run error and allow retry.
- Unsupported media for model input: keep the asset available for review and
  technical/source status; mark model evaluation as skipped or unsupported
  through core status.

## Implementation Slices

1. Gateway parity: add Tauri commands and TypeScript API wrappers for provider
   profiles, prompt packs, project settings, and manual model evaluation enqueue.
2. Project intelligence state: load provider list, prompt packs, and project
   settings with the selected project; save changes through core.
3. Minimal desktop control surface: add a compact intelligence summary and an
   editor drawer/modal without changing the review layout.
4. Evaluation actions: wire manual model evaluation for the current group and
   selected groups; keep project recommendation manual.
5. Prompt management: list, preview, fork, edit, save, and delete user-owned
   prompt packs.

## Testing

- Core tests remain the authority for settings validation, prompt pack behavior,
  provider readiness, analysis jobs, and recommendation semantics.
- Desktop Rust tests should verify Tauri command DTO mapping where logic exists.
- Desktop TypeScript tests should cover project settings state transitions,
  disabled states for no-provider/no-key, and prompt-pack selection behavior.
- Existing scan, grouping, RAW preview, virtual grid, and viewer tests should
  remain unchanged by this control-plane slice.

## Acceptance Criteria

- Desktop can list, create/update/delete provider profiles without exposing API
  keys back to UI.
- Desktop can load and save project evaluation settings for the active project.
- Desktop can list prompt packs, select one for a project, and fork/edit
  user-owned prompt preference text.
- Desktop can manually enqueue model evaluation for asset groups.
- Automatic scan-driven analysis obeys project evaluation settings.
- No-provider and no-key states are visible but do not block scan, preview,
  grouping, local technical risk, or human marks.
- Technical risk, model evaluation, model recommendation, and user marks remain
  distinct in UI state and storage.

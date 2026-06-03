# Smart Selection Evaluation Redesign

## Status

Approved direction from product discussion on 2026-05-31.

This document supersedes the scoring and recommendation semantics in
`2026-05-26-smart-selection-design.md`. The older document still describes useful
infrastructure such as background jobs, burst grouping, project-scoped browsing,
and non-destructive user actions, but its "local CV score as recommendation"
model is no longer the target product direction.

Configuration, prompt versioning, API-key/no-key behavior, manual project
recommendation triggers, and portrait-specific subject assessment are defined in
`2026-05-31-model-evaluation-configuration-design.md`.

## Problem

The current smart selection path produces a local numeric score and uses it to
pick recommendations. That makes the UI look as if the app can judge whether a
photo is good or bad from simple CV metrics. This is misleading.

The product needs two different kinds of intelligence:

- A local, deterministic quality gate that detects obvious technical failures.
- A model-based photographic evaluation layer that can judge individual photos,
  compare burst members, and choose project-level recommended works.

The recommendation surface must also separate machine results from human intent.
User favorites and user marks must not overwrite model recommendations, and model
recommendations must not imply that the user has accepted or collected a photo.

## Core Decision

Use this model:

```text
Local CV = technical failure gate
LLM/VLM = objective photographic evaluation and recommendation
User actions = independent human curation state
```

Every photo work unit should pass through model evaluation, including non-burst
single photos, when model evaluation is enabled for the project and a provider is
configured. Burst groups receive an additional within-group recommendation.
Project-level recommendations are manual-only and are never produced by upload
or background drains.

## Terms

### Asset

A single stored file, such as a JPG, RAW, HEIC, or video file.

### Asset Group

A photo work unit. A JPG and matching RAW pair belong to the same asset group.
Most UI and recommendation operations should target asset groups, not individual
files.

### Burst Group

A set of asset groups captured close together and considered candidates for
within-group comparison. A burst group can contain two or more asset groups.
Single non-burst photos remain normal asset groups.

### Technical Assessment

Local CV result for one asset group. It reports risks and confidence, not a final
aesthetic score.

### Model Evaluation

LLM/VLM evaluation for one asset group. It produces an objective photographic
score, tier, explanation, and optional warnings.

### Selection Recommendation

A model recommendation in a scope:

- `burst_group`: choose the best member inside one burst group.
- `project`: choose recommended works across the project.

### User Mark

Human curation state, independent of model results. Initial marks:

- `favorite`: user-collected or user-preferred photo.
- `flagged`: replacement for "review"; means user-marked candidate/alternate.

## Pipeline

```text
Publish complete
  -> detect or refine burst groups
  -> generate persistent preview thumbnail
  -> run local technical assessment
  -> if project model evaluation is enabled and provider is configured:
       run model evaluation for each asset group
  -> if project auto burst recommendation is enabled:
       run burst-group recommendation for groups with 2+ members
  -> do not generate project recommendation here
```

This pipeline must remain asynchronous and durable. Upload, publish, and project
browsing have priority over analysis. Analysis jobs should be idempotent and safe
to resume after app restart.

When no API key/provider capability exists, upload, thumbnails, grouping, and
local technical assessment still run. Model evaluation and project-level
recommendation are skipped or disabled, and no fake model result is written. If a
development local stub is explicitly enabled, its result source must be stored
and displayed as `local_stub`.

## Local Technical Assessment

Local CV should answer: "Is this photo technically risky or probably unusable?"

It should not answer: "Is this a good photo?"

### First Version Signals

- Severe defocus or severe blur.
- Large dead-white regions.
- Large dead-black regions.
- Very high noise.
- Severe color cast.
- Unsupported or missing preview.

### Blur And Defocus

Use multiple cheap signals instead of a single sharpness score:

- Laplacian variance for local edge detail.
- FFT high-frequency energy ratio for global detail loss.
- Optional future directional blur signal for motion smear.

The output should distinguish warning-level softness from likely unusable severe
blur.

### Dead White And Dead Black

Use both pixel ratio and connected-area ratio:

- Small specular highlights should not reject a photo.
- Large clipped regions on the subject or most of the frame should become a
  strong risk.

### Noise

Estimate noise in low-gradient regions. Separate luminance noise and chroma noise
when possible. EXIF ISO can be recorded as context but should not be the sole
reason for rejection.

### Color Cast

Use gray-world/channel imbalance and neutral-region deviation. Severe color cast
is usually a warning unless it makes the photo clearly unusable.

### Output Shape

```text
technical_assessment
  asset_group_id
  assessor_version
  status: pending | analyzing | ready | failed | unsupported
  gate_status: pass | warn | reject | needs_review | unsupported
  defect_flags[]
    type: blur | highlight_clip | shadow_clip | noise | color_cast | unsupported
    severity: low | medium | high | severe
    confidence: 0.0..1.0
    metrics_json
    reason
  preview_source
  analyzed_at_ms
```

The UI can show this as "质量风险", not as a final score.

## Model Evaluation

Every asset group should receive a model evaluation when LLM/VLM evaluation is
enabled for the project or profile. This includes non-burst single photos.

The model receives:

- A display-safe preview image or contact sheet crop.
- Technical assessment flags.
- Basic EXIF and capture metadata when available.
- User-selected evaluation profile.
- Optional user preference text.

The model outputs structured data:

```text
model_evaluation
  evaluation_id
  project_id
  asset_group_id
  evaluator_kind: llm_vlm | local_stub | imported
  evaluator_version
  status: pending | running | ready | failed | skipped
  score: 0..100
  tier: excellent | good | normal | weak | reject
  selectable: true | false
  summary
  strengths[]
  weaknesses[]
  technical_warnings[]
  created_at_ms
```

The score is a photographic evaluation score. It is not the CV score. It should
cover:

- Technical completion: clarity, exposure, color, noise, usability.
- Subject strength: subject clarity, pose, expression, action moment.
- Composition order: framing, visual weight, edge distractions.
- Light and color: mood, direction, layer, color harmony.
- Work value: whether this is worth keeping, sharing, editing, or presenting.

The model can mark a technically passable image as weak if it lacks subject or
work value. It can also keep a warning-level image if the moment is strong, but a
local severe-reject flag should normally block "selectable" unless the user
allows risky recommendations.

## Burst-Group Recommendation

Burst recommendation compares members of one burst group.

Input:

- Burst member asset groups.
- Their technical assessments.
- Their model evaluations.
- A preview contact sheet when calling an LLM/VLM.

Output:

```text
selection_recommendation
  recommendation_id
  scope: burst_group
  project_id
  subject_id: burst_group_id
  selected_asset_group_id
  alternate_asset_group_ids[]
  rejected_asset_group_ids[]
  source: llm_vlm | local_stub | imported
  status: pending | ready | stale | failed | no_selection
  confidence: 0.0..1.0
  reason
  created_at_ms
```

Important rule:

The burst winner does not have to be a project-level recommended work. A photo
can be "best in this weak burst" while still failing to become a project select.

If the whole burst is bad, the result should be `no_selection`, not a forced best
pick.

## Project-Level Recommendation

Project-level recommendation chooses final model-recommended works.

Candidate set:

- Non-burst asset groups.
- Burst winners from `burst_group` recommendations.
- Optional alternates if the user enables broader review.

Output:

```text
selection_recommendation
  scope: project
  subject_id: project_id
  selected_asset_group_ids[]
  candidate_asset_group_ids[]
  rejected_asset_group_ids[]
  source
  status
  reason
```

The UI presents these selected asset groups as "模型优选". The source can be
stored internally as `project` or `burst_group`, but the visible product language
should be consistent.

Project-level recommendation is always triggered by an explicit user action such
as "Generate Project Selects". Upload drains, background analysis drains, and
burst-stable events must not create or refresh `scope = project`
recommendations. Each manual regeneration records its own evaluation run.

## User Actions

User actions are not model recommendations.

Initial actions:

- Favorite: user collection/preference. UI label: `收藏`.
- Flagged: user candidate/alternate marker. UI label: `标记`.
- Remove from burst group.
- Delete file/photo, with explicit destructive confirmation.

Rules:

- Favorite does not overwrite model recommendation.
- Flagged does not overwrite model recommendation.
- Accepting or cancelling an algorithm recommendation is not needed as a
  separate action. If the user likes it, they can favorite it. If they dislikes
  it, they can ignore it, flag another photo, or favorite another photo.
- Manual burst split/merge remains stronger than automatic regrouping.

## Data Model Direction

Replace or reinterpret existing smart-selection tables as follows.

### New/Target Tables

```text
technical_assessments
model_evaluations
selection_recommendations
user_marks
```

### Existing Concepts To Retire Or Rename

- `quality_scores` should no longer mean final quality. It should be replaced by
  `technical_assessments`, or kept only as an implementation detail during a
  short transition.
- `overall` should not be exposed as final good/bad score.
- `rank` should not be stored as durable truth. Ordering should be derived from
  model score, recommendation status, capture time, and UI query.
- `selection_user_overrides` should become explicit user marks and grouping
  corrections, not "accepted model best" state.

The project is still in development, so no historical-data compatibility layer is
required.

## Job Types

Recommended jobs:

```text
detect_burst_for_asset_group
assess_asset_group_technical_quality
evaluate_asset_group_with_model
recommend_burst_group
recommend_project_selects
```

LLM/VLM jobs should be debounced and batched:

- Burst evaluation can run after a burst group is stable for a short idle window.
- Project recommendation is not refreshed automatically after model evaluations.
- A manual "重新生成模型优选" action creates a project-level recommendation run.

For cost and latency, prefer contact-sheet based requests:

- One contact sheet for a burst group.
- Batched contact sheets for non-burst project candidates.
- Store structured output per asset group and per recommendation scope.

## UI Semantics

### Photo Grid

Primary filters:

- 全部
- 模型优选
- 收藏
- 标记
- 质量风险
- 待分析

Burst cards:

- Show one cover image.
- Show compact count badge, such as `5` or `2/5` only in group-detail contexts.
- Cover image should be the burst recommendation if ready, otherwise the best
  available preview/capture-order cover.

### Detail Page

Show:

- Dominant photo preview.
- User actions: favorite, flagged, remove from group, delete.
- Model evaluation score and summary when available.
- Model recommendation badge when this asset group is a burst winner or project
  select.
- Technical risk panel as secondary diagnostic information.

### Burst Overview

The burst overview is still useful. It should show group members, their model
scores, technical risks, and the recommended member. It should not require a
separate "screening mode" to understand a group.

### Project Selects

Project-level model selected works should be accessible as a normal filter or
collection. It should not be mixed with user favorites.

## Settings

Settings should expose high-level model behavior rather than raw CV weights.

First useful controls:

- Global provider capability and model defaults. Capability is app-wide, and
  global defaults only prefill newly created projects.
- Enable model evaluation for this project.
- Auto-evaluate on upload for this project.
- Auto burst recommendation for this project.
- Scene profile: general / portrait / action / landscape / custom.
- Prompt profile with editable style-tagged versions.
- Allow risk photos to participate in model selection.
- Project recommendation mode: manual only.
- Maximum images per model batch.
- Privacy option: send preview only / allow larger review image.

CV thresholds can remain advanced settings. They are less important than the
model evaluation profile in the target product.

Changing global provider/model defaults must not silently enable, disable, or
otherwise rewrite model evaluation settings for existing projects. Existing
projects use their own project evaluation settings as the source of truth.

## Acceptance Criteria

- Non-burst asset groups receive model evaluations when model evaluation is
  enabled.
- Burst groups receive within-group recommendations.
- Project-level recommendations can select both non-burst photos and burst
  winners when the user manually generates project selects.
- The UI distinguishes model recommendation, model score, technical risk, user
  favorite, and user flag.
- A weak burst can have a group winner but no project-level selected work.
- A fully bad burst can produce `no_selection`.
- Local CV severe failures are visible as risks and can block model selection by
  default.
- User favorite/flag actions never mutate model recommendation rows.
- The job queue remains asynchronous and does not block upload, publish, or
  browsing.
- Upload and background analysis never generate project-level recommendations.

## Non-Goals

- Automatic deletion.
- Treating CV metrics as final photographic quality.
- Requiring LLM/VLM for local technical risk detection.
- Forcing a best pick when the whole group is bad.
- Maintaining compatibility with earlier development-stage smart-selection data.

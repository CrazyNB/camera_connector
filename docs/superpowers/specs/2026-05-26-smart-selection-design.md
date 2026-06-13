# Smart Selection Design

## Status

Superseded by the 2026-05-31 smart-selection evaluation redesign. This document
now records the current product direction and removes the older separate
filtering workflow architecture.

## Current Model

Smart selection is split into four independent concepts:

- Burst grouping: deterministic grouping over `asset_groups`.
- Technical gate: local CV detects objective risk and unsupported inputs.
- Model evaluation: subjective photography evaluation from a configured model,
  imported evaluator, or explicit local stub in development.
- User marks: human choices such as favorite and marked.

`selection_recommendations` is reserved for model recommendations only. It is not
a local CV score table and it is not a user decision table.

## Recommendation Scopes

Recommendations use a single table with explicit scope:

- `scope = burst_group`, `subject_id = burst_group_id`: choose one or more model
  winners inside a burst group.
- `scope = project`, `subject_id = project_id`: choose project-level model
  selects from single photos and burst winners.

User actions never overwrite the model result. If a user favorites or marks a
photo, that state is stored separately and can be shown alongside the model
result.

## Technical Gate

Local CV should stay conservative and objective:

- Severe blur or defocus.
- Large highlight clipping.
- Large shadow clipping.
- High noise or low-information preview.
- Severe color cast when RGB data is available.
- Unsupported preview or RAW-only inputs.

Portrait projects may enable subject-aware checks such as face box presence,
closed-eye risk, face underexposure, face overexposure, and obvious color cast.

The technical gate may block or warn before model recommendation, but it does
not produce final aesthetic rankings.

## Model Evaluation

Every supported photo work unit can receive a model evaluation. The model output
should include:

- score or tier
- selectable flag
- concise summary
- strengths
- weaknesses
- technical warnings

If no provider is configured, model work is explicitly skipped or shown as
unconfigured. The app must not silently pretend a local stub is a real provider.

## Collections

The project photo list uses collection filters:

- `all`
- `model_selects`
- `favorites`
- `marked`
- `technical_risk`
- `pending_analysis`

These are query semantics, not independent workflow state.

## UI Direction

Project Photos remains the primary browsing surface:

- Burst groups aggregate into one card and show a compact count badge.
- The card cover uses the model winner when available, otherwise the best
  available preview.
- Tapping a burst card opens a group overview; tapping a member opens detail.
- Detail view provides carousel-style group browsing.
- The main photo stays dominant. Scores, model summaries, technical warnings,
  favorite, marked, remove-from-group, and delete actions stay compact.

Fast browsing and filtering are not a standalone mode. Any future interaction in
that direction must be designed on top of detail browsing, collection filters,
model recommendations, and user marks.

## Persistence

Persist:

- background analysis jobs
- burst groups and members
- technical assessments
- model evaluations
- selection recommendations
- asset-group user marks
- prompt packs, project evaluation settings, provider capability metadata,
  and evaluation run snapshots

Do not persist:

- plain provider API keys in SQLite
- local-CV final ranking decisions
- separate filtering workflow progress
- separate filtering workflow decision history
- user-confirmed automatic recommendation states

## Non-Goals

- Automatic deletion of originals.
- Mandatory cloud or model processing.
- Full RAW decoding.
- Local CV aesthetic scoring as the final product result.
- A separate filtering workflow state machine.

# Smart Selection Evaluation Redesign Implementation Record

> Status: completed and closed as an implementation history record. Do not use
> this file as an active backlog.

## Current References

Use these files as the current product contract:

- `docs/superpowers/specs/2026-05-31-smart-selection-evaluation-redesign.md`
- `docs/superpowers/specs/2026-05-31-model-evaluation-configuration-design.md`
- `docs/product/android-app-architecture.md`
- `docs/product/PRD.md`

## Completed Shape

- Local CV is a technical risk gate, not a final photography score.
- Model evaluations hold photographic score, tier, summary, and selectable
  semantics for asset groups.
- `selection_recommendations` stores model recommendations only, with explicit
  `burst_group` and `project` scopes.
- User favorites and marks are independent human state and never mutate model
  recommendation rows.
- Upload and manual actions enqueue asynchronous analysis work; browsing remains
  non-blocking.
- Photo list filters are `all`, `model_selects`, `favorites`, `marked`,
  `technical_risk`, and `pending_analysis`.
- Detail and group overview surfaces show model score, recommendation state,
  technical risk, favorite, mark, remove-from-group, merge/split, and destructive
  delete as separate concepts.

## Closed Implementation Areas

- Core technical assessment, model evaluation, recommendation, and user-mark
  data contracts.
- SQLite persistence for technical assessments, model evaluations,
  recommendations, and marks.
- Android FFI/DTO mapping for model score, recommendation state, risk state,
  favorites, and marks.
- Android worker flow for burst detection, technical assessment, model
  evaluation, and burst recommendation.
- Android photo grid and detail UI semantics for the current model.
- Documentation cleanup for the split between local technical risk, model
  recommendation, and human curation.

## Explicitly Not Active

- Standalone old filtering/screening mode.
- Local CV final aesthetic ranking.
- Recommendation acceptance/cancellation state.
- Durable rank fields for recommendation order.
- Migration of discarded development-stage smart-selection snapshots.

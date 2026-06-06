# Smart Selection Implementation Plan

> Superseded by the 2026-05-31 evaluation redesign. This file is kept only as a
> pointer for task references.

## Current Direction

Smart selection uses model evaluation for photography judgment and local CV for
technical gates. Photo browsing, burst overview, model-select filters, favorite,
and marked actions are all built on the same project asset model.

Current semantics:

- Upload and publish enqueue asynchronous analysis jobs.
- Burst grouping works at the `asset_group` level.
- Local CV is a technical gate only: severe blur, clipping, noise, color risk,
  unsupported preview, and optional portrait-specific checks.
- Model evaluation is the source of subjective photography evaluation.
- `selection_recommendations` stores model recommendations only.
- Recommendation rows use `scope` and `subject_id` to represent burst-group or
  project recommendations.
- User choices are independent marks on asset groups, such as favorite and
  marked.
- Photo list filters use collection keys: `all`, `model_selects`, `favorites`,
  `marked`, `technical_risk`, and `pending_analysis`.
- Burst split/merge remain direct grouping operations and do not create
  recommendation decisions.
- Detail view is the main browsing surface. Group members are browsed through the
  detail carousel and group overview, not through a standalone filtering workflow.

## Current References

Use the current specs and closed implementation records for context:

- `docs/superpowers/plans/2026-05-31-smart-selection-evaluation-redesign-implementation.md`
- `docs/superpowers/plans/2026-05-31-model-evaluation-configuration-implementation.md`
- `docs/superpowers/specs/2026-05-31-smart-selection-evaluation-redesign.md`
- `docs/superpowers/specs/2026-05-31-model-evaluation-configuration-design.md`

## Retired Interaction Paths

Do not rebuild a standalone filtering workflow. If a future workflow needs fast
browsing, design it as a fresh interaction on top of the current model-evaluation
and user-mark semantics.

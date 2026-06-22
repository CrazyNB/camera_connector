# Product

## Register

product

## Users

Photographers and image operators importing camera files over a local network
and reviewing grouped assets on desktop or Android. They are in a focused work
session: create or select a project, receive or scan files, inspect grouped
RAW/JPEG/video assets, keep marks, and run evaluation or recommendation actions
when the project is ready.

## Product Purpose

Camera Connector turns local camera push imports and desktop folder scans into
formal project asset events, then gives the user a calm workbench for review,
source status, technical risk, model evaluation, recommendation, and human
selection. Success means the user can understand the current project state at a
glance and move through the next action without hunting through command-shaped
controls.

## Product Semantics

Keep these concepts separate in product copy, UI state, and implementation:

1. Receiver facts: connections, runtime status, and transfer records.
2. Asset facts: file format, storage location, grouping, duplicates, and source
   metadata.
3. Project scope: the explicit shooting project that owns imports, scans, and
   dashboard views.
4. Human decisions: favorite, marked, guest marks, manual burst edits, and
   delete actions.
5. Local technical assessment: objective risk/gate context, not a final score.
6. Model evaluation: provider-backed photographic score, tier, and summary.
7. Selection recommendation: model recommendation output only.
8. Publishing: staged bytes, final platform storage, and retry state.
9. Sharing/sync: project snapshots, LAN share sessions, and guest marks.

The module-level version of this split lives in `docs/architecture.md`.

## Brand Personality

Quiet, precise, capable.

## Anti-references

Do not feel like a backend demo, command dashboard, marketing landing page, or decorative AI-generated interface. Avoid flat button sprawl, oversized empty cards, purple gradients, novelty motion, fake operating-system chrome, and dense panels without workflow hierarchy.

## Design Principles

1. Lead with the next task.
2. Keep the interface visually calm while preserving scan and review state.
3. Use familiar desktop affordances before inventing new ones.
4. Make source status and review decisions visible without turning the screen into diagnostics.
5. Preserve trust through clear empty, loading, error, and disabled states.
6. Never blur algorithm output into human choice: technical risk, model score,
   recommendation, favorite, and marked state remain distinct.

## Accessibility & Inclusion

Target WCAG AA contrast for text and controls. Respect reduced-motion preferences. Do not rely on color alone for available, changed, missing, failed, or selected states.

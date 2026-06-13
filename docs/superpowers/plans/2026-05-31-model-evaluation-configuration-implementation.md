# Superseded Model Evaluation Configuration Plan

Status: superseded and closed.

This implementation plan described an earlier development design for model
evaluation configuration. It is intentionally not an active backlog and should
not be used for new work.

Current product and implementation references:

- `docs/superpowers/specs/2026-05-31-model-evaluation-configuration-design.md`
- `docs/superpowers/specs/2026-05-31-smart-selection-evaluation-redesign.md`
- `docs/superpowers/plans/2026-06-06-model-provider-and-prompt-pack-config.md`
- `docs/product/android-app-architecture.md`
- `docs/product/mobile-app-handoff.md`

Current direction:

- Model provider settings are app-level JSON configuration, not project table
  state.
- Prompt packs are app-private, package-grouped, Markdown-backed preference
  resources under `prompt-packs/<package>/<pack>/`.
- Projects select a provider profile id and prompt pack id; they do not copy API
  keys or prompt protocol into project state.
- Prompt request/response schemas, model task instructions, and parsing
  contracts remain system-owned.
- SQLite stores project state, evaluation runs, model evaluation results,
  technical assessments, subject assessments, and model recommendations.

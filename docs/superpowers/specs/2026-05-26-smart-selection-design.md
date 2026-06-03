# Smart Selection Design

> Superseded for scoring/recommendation semantics by
> `2026-05-31-smart-selection-evaluation-redesign.md`. Keep using this document
> for background-job, burst-grouping, project-query, and non-destructive action
> context, but do not treat local CV scores as the final recommendation model.

## Goal

Add a general-purpose smart selection layer that groups burst sequences, scores imported assets with local computer vision signals, applies user-configurable selection strategies, and writes non-destructive recommendations back into the dashboard.

LLM review is optional. It is not a required pipeline step. The default product path must work offline with deterministic grouping and local scoring.

## Context

Camera Connector already groups matching RAW/JPEG/video assets by filename stem through `ReceivedAssetGroup`. The storage optimization direction adds an `AssetIndex`, which gives this feature a natural home for persisted analysis results and paged queries.

Smart selection should sit after import and indexing:

```text
Receiver + Storage
  -> AssetIndex
  -> BackgroundJob: detect_burst
  -> BurstGroup detection / local regrouping
  -> BackgroundJob: score_asset_group
  -> Local CV scoring
  -> BackgroundJob: recommend_burst_group
  -> Strategy Profile evaluation
  -> Optional LLM review only when explicitly enabled
  -> Non-destructive marks and recommendations
  -> UI grouped display
```

This feature must not block receiving, publishing, or transfer-log writing. Publish completion should enqueue analysis work immediately, but burst detection, scoring, and recommendation run asynchronously through a durable job queue.

## Product Positioning

The feature is an import assistant, not an automatic culling tool.

It should help users answer:

- Which frame is likely best in this burst?
- Which frames are obviously blurred, underexposed, overexposed, or near-duplicates?
- Which alternatives are worth checking before deleting or exporting?
- Why was a frame recommended?

The system should never delete originals by default. It should mark, rank, hide, or recommend, and leave destructive actions to explicit user commands.

## Decision

Implement smart selection as five separable capabilities:

1. **Analysis job queue:** durable, SQLite-backed jobs connect publish completion to burst detection, scoring, and recommendation without blocking upload or publish.
2. **Burst grouping:** deterministic rules group nearby `ReceivedAssetGroup` entries into `BurstGroup` records and can locally recompute groups when late or out-of-order uploads arrive.
3. **Local scoring:** lightweight CV and metadata scorers produce cached `ImageQualityScore` records.
4. **Strategy profiles:** user-configurable weights and thresholds turn scores into recommendations.
5. **Optional LLM review:** an explicit opt-in step can review a small candidate set and produce a recommendation or explanation.

The first shippable version should include the job queue, burst grouping, local scoring, strategy profiles, and Android display surfaces. The LLM interface can be designed early but should remain disabled by default.

## Analysis Job Queue

Smart selection should use a separate background job queue rather than reusing the publish queue. Publishing owns file durability; analysis owns post-publish interpretation.

```text
background_jobs
  job_id
  project_id
  job_type
  entity_type
  entity_id
  dedupe_key
  status
  priority
  attempts
  next_attempt_at_ms
  last_error
  created_at_ms
  updated_at_ms
```

Initial job types:

- `detect_burst_for_asset_group`
- `score_asset_group`
- `recommend_burst_group`

Publish completion inserts a `detect_burst_for_asset_group` job after the new `asset_group` is durable. The burst detector updates affected `burst_groups`, then enqueues score jobs for affected member asset groups. Once scoring is complete or skipped for unsupported previews, the scorer enqueues recommendation jobs for affected burst groups.

Jobs must be idempotent and deduplicated:

- Burst detection dedupe key: `burst:{project_id}:{source_identity}:{time_bucket}`
- Scoring dedupe key: `score:{asset_group_id}:{scorer_version}`
- Recommendation dedupe key: `recommend:{burst_group_id}:{strategy_profile_id}:{strategy_version}`

Workers claim small batches, mark transient failures with `next_attempt_at_ms`, and never block receiver or publish queue progress.

## Burst Grouping

Burst grouping operates above existing RAW/JPEG grouping.

Input:

- `ReceivedAssetGroup` entries from `AssetIndex`.
- Source metadata: username, display source, remote address, original path.
- Capture timestamp when available.
- Received timestamp fallback.
- Normalized filename stem and sequence number when parseable.

Grouping rules:

- Only group assets from the same source identity.
- Prefer capture timestamps over received timestamps.
- Use a configurable burst time window, defaulting to a conservative value such as 1200 ms.
- Use filename sequence adjacency as a supporting signal.
- Keep RAW/JPEG pairs together by grouping existing `ReceivedAssetGroup` ids, not individual files.
- Do not merge video assets into photo bursts unless explicitly configured later.
- Treat upload order as unreliable. Out-of-order uploads trigger local regrouping across a small candidate window around capture time, filename sequence, and received time.
- Allow groups to merge or split when late files arrive. Existing recommendations become stale when membership changes, while reusable quality scores remain attached to their asset groups.

Output:

```text
BurstGroup
  burst_group_id
  source_identity
  started_at_ms
  ended_at_ms
  member_group_ids
  member_count
  grouping_version
  recommendation_status
  user_override_state
  created_at_ms
  updated_at_ms
```

Single-frame groups may remain normal assets in the UI. They do not need a visible burst container unless the user enables "show all groups".

User correction must be a first-class path. Users can merge or split burst groups, override the best frame, clear a recommendation, or restore the automatic recommendation. These actions write explicit `user_override` records so later analysis jobs do not silently undo user intent.

Future visual-refinement TODO:

- Time-window grouping is only the first candidate pass. It can still be too coarse when unrelated frames share close capture timestamps.
- Keep the first implemented visual pass conservative: use lightweight preview signatures to split obvious visual discontinuities inside an existing time candidate burst.
- Upgrade the visual continuity model when grouping quality becomes the next bottleneck:
  - Store a dedicated visual signature field instead of overloading recommendation or similarity labels.
  - Combine multiple cheap signals, such as average hash, perceptual hash, color histogram distance, and simple subject-region continuity.
  - Tune thresholds per strategy profile, for example stricter for action bursts and looser for landscape sequences.
  - Keep this work asynchronous and preview-based; do not decode full RAW or block receiver/upload flow.
  - Treat visual grouping as an automatic suggestion layer. User grouping corrections must remain stronger than later automatic regrouping.

## Local CV Scoring

The default scorer should be local, explainable, and cacheable. It should analyze the best available preview, not decode full RAW files.

Preferred image input order:

1. JPEG member from the asset group.
2. Embedded JPEG preview from RAW when available.
3. Thumbnail or platform preview.
4. Full image decode only when safe and configured.

Initial scoring signals:

- `sharpness`: edge/laplacian-like detail score.
- `exposure`: balance of brightness, shadows, and highlights.
- `highlight_clipping`: blown highlight penalty.
- `shadow_clipping`: crushed shadow penalty.
- `similarity`: near-duplicate relationship inside the burst.
- `composition`: low-weight composition helper based on obvious composition risks, not aesthetic taste.
- `face_eye_quality`: deferred optional module when a local face/eye detector is available.

The first version should fold obvious motion blur into `sharpness` and reasons rather than introducing a separate required score. This keeps the model explainable while leaving room for a later motion-specific scorer.

Composition scoring must stay conservative. It should build an interest heatmap from edges, local contrast, and detail density on a downscaled preview, then derive:

- Subject/interest region position against center and rule-of-thirds anchors.
- Edge-cut risk when high-interest content touches frame borders.
- Obvious horizon or dominant-line tilt when stable lines exist.
- Low-information area ratio for excessive empty sky, wall, black, or white regions.
- Left/right and top/bottom balance as a weak signal.

Composition output includes `composition_score`, `composition_confidence`, and reasons. It is only an auxiliary ranking signal; low sharpness can still veto a best-pick recommendation even when composition is strong.

Output:

```text
ImageQualityScore
  asset_group_id
  preview_source
  scorer_version
  analysis_status
  exif_status
  capture_time_ms
  sharpness
  exposure
  highlight_clipping_penalty
  shadow_clipping_penalty
  composition
  composition_confidence
  similarity_cluster_id
  overall
  reasons
  analyzed_at_ms
```

Scores should be comparable only within the same scorer version and strategy. Versioning is required so old scores can be invalidated after algorithm changes.

The first version can use received time and filename sequence before EXIF extraction is complete, but the model should reserve `capture_time_ms` and `exif_status` so EXIF-backed grouping can be added without reshaping the feature.

## Strategy Profiles

The product should ship with safe built-in profiles and allow users to customize them.

Built-in profiles:

- `General`: balanced sharpness, exposure, blur, and diversity.
- `Conservative`: only flags obvious blur, exposure failure, or near-duplicates.
- `Portrait`: deferred profile that can strengthen face and eye quality when a local detector is available.
- `Action`: stronger sharpness weighting and stricter blur-related reasons.
- `Landscape`: stronger exposure, clipping, and composition weighting.
- `Custom`: user-edited weights and thresholds.

Settings must expose profile controls without requiring the user to understand internal model names. The first Android settings surface should allow users to adjust:

- Sharpness weight.
- Exposure weight.
- Composition weight.
- Highlight clipping penalty.
- Shadow clipping penalty.
- Near-duplicate strictness.
- Burst time window.
- Low-score hiding default.

Built-in profiles are read-only presets. User edits create or update a custom profile and trigger recommendation jobs for affected projects; existing quality scores are reused unless the scorer version changes.

Example shape:

```json
{
  "id": "general",
  "name": "General",
  "burst_window_ms": 1200,
  "min_group_size": 2,
  "weights": {
    "sharpness": 0.40,
    "exposure": 0.22,
    "composition": 0.12,
    "highlight_clipping_penalty": -0.14,
    "shadow_clipping_penalty": -0.08,
    "diversity": 0.04
  },
  "thresholds": {
    "reject_if_sharpness_below": 0.25,
    "composition_max_weight": 0.12,
    "flag_if_overall_below": 0.40,
    "near_duplicate_similarity_above": 0.92,
    "max_llm_candidates_per_group": 5
  },
  "actions": {
    "auto_delete": false,
    "auto_hide_low_score": false,
    "mark_best": true,
    "keep_raw_pairs": true
  },
  "llm": {
    "enabled": false,
    "mode": "manual_review",
    "send_low_res_preview_only": true
  }
}
```

`auto_delete` must default to false in every built-in profile. If destructive actions are added later, they require explicit confirmation and a recovery story. Composition weight should default below or equal to `0.12` so it can influence close calls without selecting a technically weak frame as best.

## Recommendation Model

Strategy evaluation turns scores into non-destructive recommendations.

```text
SelectionRecommendation
  recommendation_id
  burst_group_id
  strategy_profile_id
  scorer_version
  strategy_version
  grouping_version
  best_asset_group_id
  alternate_asset_group_ids
  low_score_asset_group_ids
  near_duplicate_asset_group_ids
  source
  status
  confidence
  reasons
  llm_review_id
  created_at_ms
```

Recommendation states:

- `Best`: primary candidate in the group.
- `Alternate`: worth checking.
- `LowScore`: likely blurred or technically weak.
- `DuplicateCandidate`: visually close to a stronger frame.
- `NeedsReview`: model confidence is low or scorers disagree.

Recommendations are marks. They do not mutate original files or remove transfer records.

When burst membership, strategy weights, or scorer versions change, the current recommendation becomes `stale` and a new `recommend_burst_group` job is queued. UI can keep showing the stale recommendation with a subtle "updating" state until the replacement is ready.

Recommendations should combine absolute technical scores with within-burst normalization. The best candidate should primarily be selected by relative rank inside the current burst group, because low-light or difficult scenes may have lower absolute scores while still containing a clearly strongest frame.

Quality and recommendation state should be explicit:

- `pending`
- `analyzing`
- `ready`
- `stale`
- `failed`
- `unsupported`
- `user_overridden`

Unsupported preview reads, missing JPEG previews, SAF read failures, or decode failures should degrade to `unsupported` or `NeedsReview`; they must not break project browsing or import.

## Optional LLM Review

LLM review is opt-in and can be run manually or through a strategy profile that explicitly enables it. It is never required for the default flow.

LLM input should be limited:

- Top K candidates per burst, not the whole import.
- Low-resolution previews unless the user opts into larger images.
- CV scores, EXIF summary, capture order, and strategy profile.
- User preference text, such as "prioritize sharp eyes" or "prefer conservative picks".

LLM output:

- Best candidate.
- Alternate candidates.
- Short explanation.
- Confidence.
- Optional disagreement with local scoring.

Privacy and cost rules:

- Default LLM setting is disabled.
- User must opt in before any image preview leaves the device.
- Profiles can require low-resolution preview only.
- A local-only mode must remain fully functional.
- The UI must distinguish local CV recommendations from LLM-reviewed recommendations.

## UI Direction

Project Photos should remain usable as a normal photo grid. Smart selection adds progressive layers:

- Burst groups appear as stacked photo cards with a count badge.
- The selected best frame appears as the cover image when available.
- Detail view shows group members, score breakdown, reasons, and alternatives.
- Users can switch strategy profile and re-run analysis for a group or import.
- Users can mark best, clear recommendation, hide low-score candidates, or export selected frames.
- Low confidence groups show "Needs review" rather than pretending the choice is certain.
- Manual corrections are available from detail or selection mode: merge burst, split burst, set best, clear recommendation, and restore automatic recommendation.

Project Photos should have two main interaction modes:

- **Grid mode:** browsing, searching, sorting, filtering, batch selection, and project-level management.
- **Review mode:** a card-based screening flow for burst groups and review queues, similar to study-card or swipe-card workflows.

Preview page display rules:

- Do not cover the photo subject with heavy score UI.
- Put burst count and recommendation state in small edge badges.
- Use a compact score strip below the image or in a reserved metadata area, not over the center of the thumbnail.
- Show the primary score and one short Chinese reason when space allows, such as `清晰度高`, `曝光均衡`, or `需要复核`.
- If analysis is pending, show a quiet pending state rather than a loading overlay on the photo.
- Low-quality labels should use neutral wording such as `低分候选` or `建议复核`, not destructive wording such as delete or trash.

Project Photos sorting and filtering:

- Support sorting by latest received time, filename, and best score.
- Support filtering by recommendation state: best, needs review, low score, near duplicate, unsupported, and unreviewed.
- Support a "group best score" filter/range that compares each visible photo or burst by the highest current score inside the group.
- For single-frame asset groups, the group best score is that frame's current score.
- For burst groups, the group best score is the highest score among member asset groups under the active strategy profile, normally the recommended best frame.
- Sort/filter must come from the core query model rather than Android-only in-memory logic, so CLI, Android, and future desktop views stay consistent.
- Stale scores can remain visible but should be marked updating; unsupported scores should sort after scored items unless the user explicitly filters for unsupported.

Review queue rules:

- Review queues are core query results, not Android-only local state.
- Built-in queues: `needs_review`, `unconfirmed_best`, `low_score_candidates`, `near_duplicates`, `unsupported`, and `user_overridden`.
- A group enters `needs_review` when confidence is low, analysis is unsupported or failed, scorer signals disagree, grouping changed after a recommendation, or the user explicitly marks it for later review.
- Queue progress should be persisted per project and strategy profile so leaving and reopening the app does not lose the user's place.
- Accepting or overriding a recommendation removes the group from `unconfirmed_best` unless later analysis makes it stale again.

Review mode display rules:

- One card represents one burst group or one single-frame group that needs attention.
- The main image is the current recommended best or the highest-scored candidate.
- A compact filmstrip shows group members with badges such as `最佳`, `备选`, `低分`, `近重复`, and `需复核`.
- The card shows score, confidence, and one or two Chinese reasons without covering the main subject.
- The header shows project progress, such as `12 / 48`, and the active queue name.
- The user can switch between `全部待处理`, `只看推荐最佳`, `需复核`, `低分候选`, and `近重复` queues.

Review mode actions:

- Accept recommended best.
- Pick another member as best.
- Mark as needs review.
- Hide low-score candidates.
- Keep all candidates.
- Split group.
- Merge with adjacent group or selected group.
- Clear recommendation.
- Restore automatic recommendation.

Swipe gestures can exist as shortcuts, but visible buttons remain required for discoverability and accessibility. Suggested shortcuts are right swipe for accepting the recommendation, left swipe for needs review, up swipe for group comparison, and down swipe for keep all or skip.

Review mode should include session ergonomics:

- A session summary appears when the user exits or completes a queue, showing processed groups, accepted recommendations, manual changes, remaining review groups, and low-score candidates.
- Current-session undo is required for the most recent review decisions. The first version can keep undo scoped to the active session instead of exposing a full historical timeline.
- Shortcut actions should be configurable from Settings. Defaults can follow the suggested swipe mapping, but users can remap gestures such as right, left, up, and down swipe to review actions.
- Group comparison can open from a card or detail view. It should support enlarged side-by-side or quick-toggle comparison for two or three candidates, focused on sharpness, exposure, composition, and near-duplicate differences.
- Accepted recommendations should feed a project-level Selects collection. The first version can treat Selects as a virtual collection backed by recommendation/review decisions, but the model should leave room for explicit user-managed selects and future export.
- Shortcut preferences should not be touch-only. Store action mappings in an input-agnostic shape so future PC keyboard or controller bindings can map actions such as accept, needs review, next candidate, previous candidate, compare, and undo without changing the review model.

Detail page display rules:

- The main image remains dominant.
- Score summary should be visible near the top, beside or below the main image depending on viewport width.
- Show per-signal scores with Chinese labels: `清晰度`, `曝光`, `构图`, `高光`, `阴影`, `相似度`.
- Show recommendation reasons prominently enough to explain the choice, but keep full diagnostic details collapsible.
- For burst groups, provide a member carousel or filmstrip with badges for `最佳`, `备选`, `低分`, `近重复`, and `需复核`.
- Long press and explicit action controls remain available for manual override.

Group comparison should be available from Review mode and Detail mode. It should emphasize relative differences inside the burst, especially sharpness, exposure, composition, and near-duplicate similarity, rather than forcing users back into the full grid.

No visible UI should imply that hidden or low-score assets were deleted.

## Data Persistence

Smart selection data belongs beside or inside `AssetIndex`.

Persist:

- Strategy profiles and user edits.
- Background analysis jobs.
- Burst group membership.
- Quality score records with scorer version.
- Recommendations and reasons.
- User override records for manual grouping and recommendation corrections.
- Review queue progress and user review decisions.
- Review session summary facts, current-session undo stack, and shortcut-action preferences.
- Selects collection membership or virtual-select facts derived from accepted recommendations.
- LLM review records when used.

Do not persist:

- Plain LLM API keys in core config.
- Full image previews sent to remote services.
- Destructive deletion decisions without explicit user action records.

## Background Processing

Analysis should be interruptible and resumable.

Rules:

- Receiver and publish queue have priority over analysis.
- Publish completion enqueues analysis jobs, but analysis workers execute asynchronously in small batches.
- Pause on low battery or thermal pressure if Android reports constraints.
- Throttle analysis during large imports by project and priority; UI-triggered jobs can receive higher priority, but upload and publish still win.
- Cache scores by asset group id, preview location, file size, and scorer version.
- Recompute when the scorer version changes or the user changes a strategy profile.
- Late or out-of-order uploads should only re-run burst detection for the local candidate window, not the whole project.
- Membership changes reuse existing quality scores and only rerun recommendations unless the affected asset group lacks a current score.
- Similarity is scoped to the current burst group in the first version. Do not build full-library visual search as part of this feature.

## Testing Strategy

Unit tests:

- Burst grouping by timestamp window.
- Burst grouping by filename sequence fallback.
- No grouping across different sources.
- RAW/JPEG group ids stay together.
- Strategy profile weight evaluation.
- Recommendation state assignment.
- Out-of-order uploads merge or split burst groups through local regrouping.
- Composition scoring reasons for subject position, edge cut risk, tilt, and low-information regions.
- Profile weight edits enqueue recommendation jobs without recomputing unchanged quality scores.
- User overrides survive later automatic analysis runs.
- Group best score sorting and filtering use the active strategy recommendation.
- Unsupported preview analysis produces `NeedsReview` without failing the query.
- LLM disabled path never requires LLM output.

Fixture tests:

- Synthetic sharp vs blurred image pairs.
- Synthetic overexposed and underexposed images.
- Near-duplicate image hashes.
- Mixed RAW/JPEG/video import records.

Integration tests:

- Import records flow into `BurstGroup` and `SelectionRecommendation`.
- Dashboard can show burst count and best candidate.
- Re-running a profile updates recommendations without duplicating records.
- Legacy imports without capture time still group by received time and filename sequence.
- Publish completion enqueues analysis jobs without blocking publish completion.

Android tests:

- Burst cards render count badges and best candidate covers.
- Preview cards show compact score badges or strips without covering the main subject area.
- Detail view shows local scores, signal labels, reasons, alternatives, and burst members.
- Settings exposes strategy profile weights and threshold controls.
- Project Photos supports sorting/filtering by group best score and recommendation state.
- Strategy profile changes trigger re-analysis.
- LLM review controls are hidden or disabled when LLM is off.

## Acceptance Criteria

The first version is accepted when:

- Existing RAW/JPEG grouping still works unchanged.
- Imported photo sequences can be grouped into burst groups using deterministic rules.
- Local scoring produces cached, explainable scores for supported previews.
- A default General profile can mark a best candidate without calling an LLM.
- Users can select or edit a strategy profile.
- Settings can adjust scoring weights and trigger recommendation refresh.
- Preview and detail views show score information clearly without obscuring photo content.
- Burst groups have optimized preview and detail presentations.
- Review mode lets users process recommendation and review queues one group at a time.
- Photo list queries can sort/filter by group best score from core data.
- Users can manually override grouping and recommendation decisions.
- Recommendations are non-destructive and reversible.
- LLM review is clearly optional and disabled by default.
- Dashboard/UI can display burst groups and best candidates without blocking receiver uploads.

## Non-Goals

- Automatic deletion of originals.
- Mandatory cloud or LLM processing.
- Full RAW decoding.
- Professional-grade aesthetic judging in the first version.
- Replacing user review for low-confidence groups.
- Blocking import, publish, or receiver lifecycle on analysis.

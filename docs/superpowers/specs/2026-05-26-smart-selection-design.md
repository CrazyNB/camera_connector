# Smart Selection Design

## Goal

Add a general-purpose smart selection layer that groups burst sequences, scores imported assets with local computer vision signals, applies user-configurable selection strategies, and writes non-destructive recommendations back into the dashboard.

LLM review is optional. It is not a required pipeline step. The default product path must work offline with deterministic grouping and local scoring.

## Context

Camera Connector already groups matching RAW/JPEG/video assets by filename stem through `ReceivedAssetGroup`. The storage optimization direction adds an `AssetIndex`, which gives this feature a natural home for persisted analysis results and paged queries.

Smart selection should sit after import and indexing:

```text
Receiver + Storage
  -> AssetIndex
  -> BurstGroup detection
  -> Local CV scoring
  -> User Strategy Profile
  -> Optional LLM review
  -> Non-destructive marks and recommendations
  -> UI grouped display
```

This feature must not block receiving, publishing, or transfer-log writing. It can run lazily after import, in the background, or on user request.

## Product Positioning

The feature is an import assistant, not an automatic culling tool.

It should help users answer:

- Which frame is likely best in this burst?
- Which frames are obviously blurred, underexposed, overexposed, or near-duplicates?
- Which alternatives are worth checking before deleting or exporting?
- Why was a frame recommended?

The system should never delete originals by default. It should mark, rank, hide, or recommend, and leave destructive actions to explicit user commands.

## Decision

Implement smart selection as four separable capabilities:

1. **Burst grouping:** deterministic rules group nearby `ReceivedAssetGroup` entries into `BurstGroup` records.
2. **Local scoring:** lightweight CV and metadata scorers produce cached `ImageQualityScore` records.
3. **Strategy profiles:** user-configurable weights and thresholds turn scores into recommendations.
4. **Optional LLM review:** an explicit opt-in step can review a small candidate set and produce a recommendation or explanation.

The first shippable version should include burst grouping, local scoring, and strategy profiles. The LLM interface can be designed early but should remain disabled by default.

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

Output:

```text
BurstGroup
  group_id
  source_identity
  started_at_ms
  ended_at_ms
  member_group_ids
  member_count
  best_asset_id
  strategy_profile_id
  confidence
  created_at_ms
  updated_at_ms
```

Single-frame groups may remain normal assets in the UI. They do not need a visible burst container unless the user enables "show all groups".

## Local CV Scoring

The default scorer should be local, explainable, and cacheable. It should analyze the best available preview, not decode full RAW files.

Preferred image input order:

1. JPEG member from the asset group.
2. Embedded JPEG preview from RAW when available.
3. Thumbnail or platform preview.
4. Full image decode only when safe and configured.

Initial scoring signals:

- `sharpness`: edge/laplacian-like detail score.
- `motion_blur`: blur penalty.
- `exposure`: balance of brightness, shadows, and highlights.
- `highlight_clipping`: blown highlight penalty.
- `shadow_clipping`: crushed shadow penalty.
- `similarity`: near-duplicate relationship inside the burst.
- `face_eye_quality`: optional module when a local face/eye detector is available.
- `composition`: low-weight optional heuristic for future use.

Output:

```text
ImageQualityScore
  asset_id
  preview_source
  scorer_version
  sharpness
  exposure
  motion_blur_penalty
  highlight_clipping_penalty
  shadow_clipping_penalty
  similarity_cluster_id
  face_eye_quality
  composition
  overall
  reasons
  analyzed_at_ms
```

Scores should be comparable only within the same scorer version and strategy. Versioning is required so old scores can be invalidated after algorithm changes.

## Strategy Profiles

The product should ship with safe built-in profiles and allow users to customize them.

Built-in profiles:

- `General`: balanced sharpness, exposure, blur, and diversity.
- `Conservative`: only flags obvious blur, exposure failure, or near-duplicates.
- `Portrait`: stronger face and eye quality when local detection is available.
- `Action`: stronger motion blur and sharpness weighting.
- `Landscape`: stronger exposure, clipping, and composition weighting.
- `Custom`: user-edited weights and thresholds.

Example shape:

```json
{
  "id": "general",
  "name": "General",
  "burst_window_ms": 1200,
  "min_group_size": 2,
  "weights": {
    "sharpness": 0.35,
    "exposure": 0.20,
    "motion_blur_penalty": -0.25,
    "highlight_clipping_penalty": -0.15,
    "shadow_clipping_penalty": -0.10,
    "face_eye_quality": 0.10,
    "composition": 0.05,
    "diversity": 0.10
  },
  "thresholds": {
    "reject_if_sharpness_below": 0.25,
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

`auto_delete` must default to false in every built-in profile. If destructive actions are added later, they require explicit confirmation and a recovery story.

## Recommendation Model

Strategy evaluation turns scores into non-destructive recommendations.

```text
SelectionRecommendation
  recommendation_id
  burst_group_id
  strategy_profile_id
  best_asset_id
  alternate_asset_ids
  low_score_asset_ids
  near_duplicate_asset_ids
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

Inbox should remain usable as a normal photo grid. Smart selection adds progressive layers:

- Burst groups appear as stacked photo cards with a count badge.
- The selected best frame appears as the cover image when available.
- Detail view shows group members, score breakdown, reasons, and alternatives.
- Users can switch strategy profile and re-run analysis for a group or import.
- Users can mark best, clear recommendation, hide low-score candidates, or export selected frames.
- Low confidence groups show "Needs review" rather than pretending the choice is certain.

No visible UI should imply that hidden or low-score assets were deleted.

## Data Persistence

Smart selection data belongs beside or inside `AssetIndex`.

Persist:

- Strategy profiles and user edits.
- Burst group membership.
- Quality score records with scorer version.
- Recommendations and reasons.
- LLM review records when used.

Do not persist:

- Plain LLM API keys in core config.
- Full image previews sent to remote services.
- Destructive deletion decisions without explicit user action records.

## Background Processing

Analysis should be interruptible and resumable.

Rules:

- Receiver and publish queue have priority over analysis.
- Analyze newly completed imports in small batches.
- Pause on low battery or thermal pressure if Android reports constraints.
- Cache scores by asset id, preview location, file size, and scorer version.
- Recompute when the scorer version changes or the user changes a strategy profile.

## Testing Strategy

Unit tests:

- Burst grouping by timestamp window.
- Burst grouping by filename sequence fallback.
- No grouping across different sources.
- RAW/JPEG group ids stay together.
- Strategy profile weight evaluation.
- Recommendation state assignment.
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

Android tests:

- Burst cards render count badges and best candidate covers.
- Detail view shows local scores and reasons.
- Strategy profile changes trigger re-analysis.
- LLM review controls are hidden or disabled when LLM is off.

## Acceptance Criteria

The first version is accepted when:

- Existing RAW/JPEG grouping still works unchanged.
- Imported photo sequences can be grouped into burst groups using deterministic rules.
- Local scoring produces cached, explainable scores for supported previews.
- A default General profile can mark a best candidate without calling an LLM.
- Users can select or edit a strategy profile.
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

# Scan-Integrated Project Sync Design

## Goal

Bring project context from a nearby device into the desktop workbench without a
heavy import wizard. The desktop flow should feel like the existing folder scan:
choose or discover a project source, scan the local photo root, index local
files, match synchronized project facts to local assets, and quietly fill the
available context.

The first implementation slice may use a local JSON project snapshot instead of
real LAN discovery. That keeps the data model, scan integration, and matching
rules testable before the network source is added.

## Product Direction

Desktop remains the large-screen review workbench. Sync is an input to the same
project view, not a separate report-driven migration surface.

The user experience should be:

1. Select an existing desktop project or create one.
2. Choose a project sync source.
3. Choose or confirm the local photo root.
4. Run the normal desktop scan.
5. Let desktop match synchronized facts to indexed local files and groups.
6. Review the same grid/detail surface with additional synchronized context.

For the first slice, "project sync source" is a local JSON snapshot file. Later,
LAN discovery and device project pull should provide the same snapshot shape.

## Non-Goals

- Do not build a multi-step import wizard.
- Do not require the user to review a large report before continuing.
- Do not add portable identity columns to SQLite.
- Do not add any sync-specific SQLite tables, columns, or persisted import
  session models.
- Do not implement bidirectional conflict merge.
- Do not copy, move, rename, or reorganize original photo files.
- Do not block scan or local review when synchronized facts cannot be matched.
- Do not expose LAN networking in the first JSON-snapshot implementation slice.

## Relationship To Existing Specs

This design replaces the user-facing direction of the older dormant package
migration flow. The package protocol remains useful for normalized snapshot
shape and factual matching policy, but desktop should treat project sync as part
of scan/index/match rather than as an explicit import package review.

The existing desktop scan model remains the local indexing authority:

- scan runs create `desktop_scan` transfers;
- local files become project assets and groups;
- missing and changed source files keep project facts visible;
- model evaluation, recommendations, and user marks remain separate concepts.

The existing Android LAN share design remains separate. LAN share is guest
selection from a hosted browser page. Project sync is device-to-desktop project
context transfer.

## Snapshot Contract

A project sync snapshot is a versioned JSON document containing normalized
project facts. The same schema can be loaded from a local file in the first
slice or pulled from a LAN device later.

```json
{
  "schema_version": 1,
  "source_device": {
    "device_id": "android-phone-1",
    "device_label": "Pixel Field Kit",
    "platform": "android"
  },
  "project": {
    "project_id": "android-project-local-id",
    "name": "Wedding Selects",
    "exported_at_ms": 1781800000000
  },
  "assets": [],
  "groups": [],
  "model_evaluations": [],
  "selection_recommendations": [],
  "user_marks": []
}
```

Snapshot ids are source-local references. They must not be treated as universal
asset or group ids. Matching authority comes from factual fields on assets and
groups.

Asset records should include:

- `asset_id`
- `group_id`
- `original_filename`
- `final_filename`
- `normalized_stem`
- `original_path`
- `original_parent_path`
- `format`
- `size_bytes`
- `capture_at_ms`
- `received_at_ms`
- `source_identity`

Group records should include:

- `group_id`
- `display_key`
- `source_identity`
- `original_parent_path`
- `member_asset_ids`
- `primary_asset_id`
- `preview_asset_id`
- `has_raw`
- `has_jpeg`
- `has_video`

Model evaluations, selection recommendations, and user marks reference snapshot
asset and group ids. They are applied only after those ids are mapped to local
project groups or assets.

## Matching Policy

The matcher compares snapshot facts with the desktop project's indexed assets
after a scan.

Asset matching is attempted in this order:

1. `original_path + format + size_bytes + capture_at_ms`
2. `original_filename + format + size_bytes + capture_at_ms`
3. `original_filename + format + size_bytes`
4. `normalized_stem + format + size_bytes`
5. `normalized_stem + format`

Group matching is attempted in this order:

1. all member assets matched with no conflicts;
2. `source_identity + original_parent_path + normalized_stem`;
3. `original_parent_path + normalized_stem`;
4. `normalized_stem`.

A lower-confidence match can be applied only when it is unique. Ambiguous
matches are not applied automatically.

## Sync Application Rules

The sync operation applies only matched, compatible facts:

- User marks may update the local group's favorite and marked state.
- Model evaluations may be imported as model-produced evaluations associated
  with the matched local group.
- Selection recommendations may be imported only when every referenced selected,
  candidate, or rejected group maps cleanly to a local group.
- Snapshot groups without local matches appear only in the transient sync
  response counts for now; they do not create fake local photo assets,
  placeholder rows, or unresolved binding records.
- Local source status remains owned by desktop scan: `available`, `missing`, or
  `changed`.
- Human marks, guest marks, model evaluations, and model recommendations stay
  distinct. Applying one must not rewrite the others.

When a snapshot fact cannot be applied, desktop returns a compact sync outcome
for status and diagnostics but does not persist a sync report, import session,
or unresolved-match model.

All durable writes must use existing project storage concepts: asset group user
marks, model evaluations, and selection recommendations. The sync layer may
compute temporary mapping state while it runs, but it must not require a schema
migration.

## Desktop UX

The UX should stay close to desktop scan:

- The project/source panel may show a compact sync source row.
- The scan progress strip may add a short sync phase such as "matching project
  context".
- The asset grid and detail view show synchronized context through existing
  badges and metadata.
- Missing or unmatched snapshot content should appear as lightweight counts or
  source-state hints from the latest command response, not as a mandatory or
  persisted report page.

The first implementation can expose the JSON snapshot source through a small
developer-facing command or file path input. The production LAN discovery UI can
replace that source later without changing core matching behavior.

## LAN Discovery Direction

LAN discovery should be a later adapter over the same snapshot contract.

Android or another device can advertise project sync availability on the local
network. Desktop discovers devices, lists available projects, pulls the selected
project snapshot, and passes that snapshot into the same scan-integrated sync
flow used by local JSON.

The network layer must not own matching or storage semantics. It only supplies a
snapshot and device metadata.

## Error Handling

- Invalid snapshot JSON: reject the sync source with a visible setup error.
- Unsupported `schema_version`: reject with a version-specific error.
- Missing local root: allow snapshot loading, but all facts remain unmatched
  until scan/index runs.
- Ambiguous asset or group match: do not apply the fact automatically.
- Partial recommendation match: do not import that recommendation.
- Evaluation or mark references an unknown snapshot id: skip that record and
  count it as unresolved.
- Applying synchronized context must be idempotent for the same snapshot and
  local scan result.

## Testing Strategy

Core tests:

- Load and validate a minimal project sync snapshot.
- Reject unsupported schema versions and corrupt JSON.
- Match snapshot assets to desktop-scanned local assets by the ordered policy.
- Refuse ambiguous lower-confidence matches.
- Match groups through matched member assets.
- Apply user marks only to matched local groups.
- Import model evaluations only to matched local groups.
- Skip recommendations when any referenced group is unmatched.
- Re-running the same sync does not duplicate imported facts.

Desktop gateway tests:

- A command can load a local JSON snapshot path and run sync against the
  selected project.
- The response returns compact counts: matched, applied, unresolved, ambiguous,
  and skipped.

Manual verification:

- Create or select a desktop project.
- Scan a local folder containing files that correspond to the snapshot.
- Run sync from a local JSON snapshot.
- Confirm matched groups show imported marks and model context in the existing
  grid/detail UI.
- Confirm unmatched snapshot items do not block review.

## Implementation Slices

1. Core snapshot types and parser for local JSON.
2. Core matcher over existing desktop project assets and groups.
3. Core sync application for user marks, model evaluations, and recommendations.
4. Desktop command surface for local JSON snapshot sync.
5. Compact desktop status integration after scan.
6. LAN discovery and project snapshot pull as a later adapter.

The first implementation should complete slices 1 through 4 and keep slice 5
minimal. Slice 6 is intentionally deferred until the local snapshot flow proves
the data model and matching behavior.

## Acceptance Criteria

- Desktop can load a project sync snapshot from JSON.
- Desktop can scan a local root and then match snapshot facts to local indexed
  assets.
- Unique matches apply user marks and model context to the local project.
- Ambiguous and unmatched facts do not block scan or review.
- The flow does not introduce a mandatory import report screen.
- The implementation does not add sync-specific database tables, columns, or
  persisted import/session models.
- The same core sync API can later accept a LAN-pulled snapshot without changing
  matching or storage semantics.

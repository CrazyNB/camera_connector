# Project Package Migration Protocol Design

## Goal

Define a future project-level migration protocol for moving Camera Connector project context between Android and PC without changing the current SQLite storage model.

The protocol should let Android export a project package containing asset, group, score, and selection context. A PC importer can then load the package, ask the user to choose the photo library root, scan local files, and map the package records back to real files and groups.

This is a spec only. It is not part of the current storage implementation priority.

## Decisions

- Migration is project-scoped. The transferable unit is a project package, not a standalone score file and not a global gallery export.
- SQLite remains the source of local truth. It stores factual fields and local ids; it does not store extra `portable_asset_key` or `portable_group_key` columns.
- Matching keys are protocol rules derived at export/import time from existing fields such as `original_path`, `original_parent_path`, `normalized_stem`, `format`, `size_bytes`, and capture time.
- `asset_id` and `group_id` may appear in the package as package-local references, but they are not the cross-platform matching authority.
- The PC importer may allocate its own local project, group, and asset ids. Any package-id to local-id mapping is importer state, not a required field in the core Android database.

## Non-Goals

- Do not implement import/export now.
- Do not add SQLite columns for portable identity.
- Do not introduce an `ImportSession` model.
- Do not require MediaStore support on Android.
- Do not solve cloud sync, live collaboration, or bidirectional conflict merge.

## Package Shape

A project package is a directory or archive with a versioned manifest and normalized JSON indexes:

```text
project-package/
  manifest.json
  assets.json
  groups.json
  scores.json
  selections.json
  thumbnails/
```

`manifest.json` defines:

- `schema_version`
- exporter app/version/platform
- exported time
- source project summary
- match policy version

`assets.json` contains one record per file asset using fields already available in storage:

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
- `username`
- `remote_addr`

`groups.json` contains one record per RAW/JPEG/video group:

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

`scores.json` and `selections.json` reference package-local `asset_id` and `group_id`. They should not reference platform paths.

## Matching Policy

The importer computes match candidates from package fields and scanned local files.

Asset matching is attempted in order:

1. `original_path + format + size_bytes + capture_at_ms`
2. `original_filename + format + size_bytes + capture_at_ms`
3. `original_filename + format + size_bytes`
4. `normalized_stem + format + size_bytes`
5. `normalized_stem + format`

Group matching is attempted in order:

1. all member assets matched with no conflicts
2. `source_identity + original_parent_path + normalized_stem`
3. `original_parent_path + normalized_stem`
4. `normalized_stem`

Lower-confidence matches can be accepted only when they are unique. Ambiguous matches must be reported to the user for confirmation.

## Import Flow

1. User opens a project package on PC.
2. Importer reads `manifest.json` and validates `schema_version`.
3. User selects a photo library root.
4. Importer scans supported media files under the selected root.
5. Importer computes package match keys and local file match keys using the protocol rules.
6. Importer builds a temporary mapping:

```text
package_asset_id -> local_asset_or_file
package_group_id -> local_group
```

7. Importer applies score and selection records to matched local groups/assets.
8. Unmatched and ambiguous records are shown as an import report.

The temporary mapping may be persisted by the PC app if its local product model needs it, but the Android/core storage model does not require it.

## Error Handling

- Unsupported `schema_version`: reject with an explicit version error.
- Missing package file: reject as corrupt package.
- Missing local photo root: allow package inspection, but mark all asset bindings unresolved.
- No match: keep score/selection data in the package import report but do not attach it to a local file.
- Multiple matches: require manual confirmation.
- Partial group match: import matched members and mark the group as incomplete.

## Compatibility Notes

The current `group_id` is generated from project-scoped identity. It is stable inside one storage database and one package export, but it should not be treated as a universal photo identity.

The current `asset_id` equals `transfer_id`, which is also local to the original import event. It is useful as a package reference but not as the only cross-device lookup key.

The durable cross-platform contract is the protocol's ordered matching policy over factual asset fields.

## Open Implementation Timing

This spec should stay dormant until the Android import pipeline and multi-project workflow are validated. The next practical milestone remains end-to-end Android FTP upload verification and real-camera compatibility testing.

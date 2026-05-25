# Storage Model Optimization Design

## Goal

Evolve Camera Connector's storage model from a desktop-local file sink into a platform-aware import pipeline that works cleanly across desktop, Android SAF, Android MediaStore, future iOS storage, and NAS/headless targets.

The main design move is to separate **receiver staging**, **final object publishing**, **project organization**, **state/log persistence**, and **query indexing**. The receiver should keep a simple, reliable write target, while platform-specific storage rules and user-facing organization live behind dedicated layers.

## Context

The current core already has the right product rules:

- FTP/SFTP receivers stream uploads through temporary files.
- Uploaded paths are flattened and sanitized.
- Duplicate filenames are preserved.
- Transfer records keep original camera paths and virtual display paths.
- `StoredObjectLocation` can represent `local_path`, `document_uri`, `media_uri`, and `photo_asset`.
- Android can persist a selected SAF directory label, but native smoke imports still write to app-private storage.
- The current flat inbox is good for one shoot, but multiple shooting projects become hard to distinguish without a first-class project model.

The gap is that `LocalFileSink` is still the real write path. It assumes final storage behaves like a local filesystem with seek, rename, path-based existence checks, and direct scanning. Those assumptions do not hold uniformly for SAF, MediaStore, Photos, or remote object stores.

## Decision

Adopt a staged import pipeline:

```text
Camera FTP/SFTP
  -> StagingStore
  -> PublishQueue
  -> ObjectStore
  -> ProjectCatalog
  -> TransferLog + AssetIndex
  -> Dashboard/UI
```

The receiver writes to `StagingStore`, which is always reliable local app-private storage for mobile and a local temporary directory for desktop/headless. After a complete upload, a publisher moves or copies the staged object into the configured `ObjectStore`. The final saved target is recorded as `StoredObjectLocation`.

For desktop, the object store can still be a local folder. For Android, the object store can be SAF or MediaStore. For future iOS, it can map to Files or Photos identifiers.

Physical storage can remain flat for safety and compatibility. Project organization is metadata-first and query-first. Object stores may optionally create project folders or album-like collections when the platform supports it, but the core product model must not depend on folder hierarchy.

## Current Model To Preserve

These product rules must not change:

- Completed files are flat by final filename, not mirrored from camera paths.
- Original camera path remains metadata.
- Duplicate uploads are preserved without overwriting earlier completed files.
- RAW/JPEG/video grouping is based on normalized filename stem.
- Transfer rows expose display source, username, remote address, virtual display path, size, and failure text.
- Receiver metadata lives outside the user-facing inbox.
- Existing `transfer-log.jsonl` records remain readable.
- The final storage location can stay flat while projects and virtual paths provide user-facing organization.

## Target Components

### `StagingStore`

Owns receiver-facing writes.

Responsibilities:

- Reserve a staged object id for each upload.
- Create a temporary local file for streaming.
- Support sequential FTP writes.
- Support random-access SFTP writes.
- Flush and mark a staged object as complete.
- Clean stale incomplete temporary files on startup.

Mobile implementations should use app-private files. This avoids SAF or MediaStore random-write limitations while keeping receiver behavior stable.

### `PublishQueue`

Owns the transition from a completed staged object to final storage.

Responsibilities:

- Enqueue completed staged uploads.
- Retry failed publishes.
- Keep enough metadata to recover after process death.
- Report `staged`, `publishing`, `completed`, and `failed` states.
- Leave staged bytes intact until the final object is durable.

The queue should live in the SQLite state database from the foundation milestone. `transfer-log.jsonl` remains the audit trail, while retryable publish state belongs in durable indexed state.

### `ObjectStore`

Owns final user-facing storage.

Responsibilities:

- Reserve a final filename using the product duplicate policy.
- Publish bytes from a staged object.
- Return a `StoredObjectLocation`.
- Provide a display label for settings and dashboard.
- Expose enough read access for previews when the platform allows it.

Initial implementations:

- `LocalFolderObjectStore`: desktop and tests.
- `AndroidSafObjectStore`: user-selected document tree.
- `AndroidMediaStoreObjectStore`: media collection or album-style output.

The receiver should not know whether the final object is a local path, document URI, media URI, or photo asset.

### `ProjectCatalog`

Owns user-facing shooting project organization.

Responsibilities:

- Create, update, archive, and list projects.
- Track the active project used by every new import.
- Require a selected project, or an explicit system project such as "Inbox", before receiving uploads.
- Provide the required dashboard and inbox scope for viewing assets.
- Provide project-level output preferences when needed.
- Support manual reassignment of assets or groups from one project to another.
- Keep project organization independent from physical storage paths.

Project records:

```text
Project
  project_id
  name
  slug
  description
  status
  created_at_ms
  updated_at_ms
  archived_at_ms
  default_output_target_id
  default_strategy_profile_id
```

Projects are long-lived user containers such as "2026-05 Wedding", "Studio Product Shoot", or "Weekend Street Walk". They should appear in the UI as the first-level organization above imports and asset groups. Uploaded photos must not exist outside a project.

### `StateStore`

Owns serialized state mutations.

Responsibilities:

- Append transfer records.
- Update connected-device metadata.
- Update receiver runtime status.
- Persist publish queue state.
- Prevent concurrent read-modify-write corruption.

The append-only `transfer-log.jsonl` can stay as an audit output, but mutable state should move behind a single SQLite-backed state boundary. The first implementation should use serialized writes through `StateStore` so transfer, device, receiver, project, and publish queue mutations cannot corrupt each other.

### `AssetIndex`

Owns query performance.

Responsibilities:

- Materialize completed transfer records into asset rows.
- Support dashboard pagination, filtering, grouping, duplicates, and facet counts.
- Support project filters as first-class query dimensions.
- Require project-scoped dashboard and inbox queries for asset grids.
- Avoid rebuilding every asset group from the full transfer log on each poll.
- Keep `transfer-log.jsonl` as the audit trail.

The foundation implementation should use SQLite instead of a JSON index. The storage refactor is the point where the project, publish queue, transfer, asset, and dashboard query models should become durable in one schema.

Core tables:

- `projects`
- `transfers`
- `assets`
- `asset_groups`
- `publish_queue`
- `connected_devices`
- `receiver_status`

Later feature tables:

- `burst_groups`
- `quality_scores`
- `selection_recommendations`
- `strategy_profiles`

## Data Model Direction

### Project identity

Every new transfer should resolve to:

- a `project_id`
- a source identity
- a final object location

The project model is both the user-facing and operational organization boundary. Every uploaded asset, transfer, group, and publish queue item belongs to exactly one project. Assets inherit project identity from their transfer records and can move to another project only through an explicit user action.

Assignment rules:

- Use the active project selected in the UI when the receiver starts.
- If no active project exists, either block receiver start until the user chooses or creates one, or use an explicit system project such as "Inbox" when the product chooses a low-friction default.
- Keep old records without a project readable by mapping them to "Legacy Import" or "Unassigned" during migration.
- Do not create unprojected assets.

Project metadata should not replace original camera path. The original path stays diagnostic metadata. Project identity is product organization.

### Transfer lifecycle

The storage pipeline needs more than `Completed` and `Failed`.

Target internal states:

- `Receiving`: receiver has accepted an upload and is writing staged bytes.
- `Staged`: upload bytes are complete but not yet published.
- `Publishing`: final object publish is active.
- `Completed`: final object is durable and indexed.
- `Failed`: receive or publish failed.

The UI can still collapse this into simple labels, but the core needs enough state to retry and recover.

### Final location

`final_location` should become the primary location field for new records. `final_path` should remain only as a legacy compatibility field.

Rules:

- New local-folder records write `final_location = LocalPath`.
- Android SAF records write `final_location = DocumentUri`.
- Android MediaStore records write `final_location = MediaUri`.
- Future iOS Photos records write `final_location = PhotoAsset`.
- Readers continue resolving legacy `final_path` as `LocalPath`.

### Output configuration

Current `output_dir: PathBuf` is too filesystem-specific for mobile.

Target configuration should distinguish:

- `staging_dir`: app-private receiver write area.
- `state_dir`: metadata, logs, queue, and index.
- `output_target`: platform final storage target.
- `output_label`: user-facing display label.
- `active_project_id`: project used for new imports.

Desktop can map `output_target` to a local folder. Android can map it to SAF or MediaStore configuration. The native FFI boundary should avoid passing SAF URI values as if they were filesystem paths.

Project output preferences can override the global output target when configured. If a project has no output target, it inherits the app default.

## Android Storage Direction

Android should treat selected SAF/MediaStore destinations as final object stores, not as Rust `PathBuf` output directories.

Recommended flow:

1. `CoreGatewayFactory` seeds app-private `staging_dir` and `state_dir`.
2. Android persists selected SAF/MediaStore targets in `AndroidStorageGateway`.
3. The user selects an active project, or the app uses an explicit system project such as "Inbox".
4. The Rust receiver writes staged files only.
5. Android or a native platform adapter publishes staged files into the selected target.
6. The final transfer record stores project identity and platform `StoredObjectLocation`.
7. Dashboard previews open `document_uri` or `media_uri` through Android APIs.

This keeps the Rust receiver portable while letting Android own platform storage permissions.

## Failure And Recovery

The optimized model must handle these cases:

- Upload interrupted before staging completes: keep or remove incomplete temp file according to cleanup policy and record a failed receive when enough metadata exists.
- Upload completes but publish fails: keep staged file, mark publish failed, show retry action.
- App process dies after staging but before publish: recover queue on next startup.
- Final object exists: reserve a duplicate filename before publish.
- State write fails after final publish: preserve enough queue metadata to reconcile on next startup.
- SAF permission is revoked: keep staged files and prompt user to reauthorize storage.
- Active project is deleted/archived during receiver activity: prevent destructive removal until receiver stops, or require the user to move in-flight records to another project.
- Legacy imports have no project id: expose them under "Legacy Import" or "Unassigned" and allow manual reassignment.

No completed upload should disappear silently. If final publishing fails, the staged bytes remain recoverable until the user deletes them or a successful retry completes.

## Migration Plan

### Foundation milestone: complete storage schema and local compatibility

This milestone should land the durable architecture in one pass:

- Introduce SQLite-backed `StateStore` and `AssetIndex`.
- Add project tables and required project references on transfers, assets, asset groups, and publish queue rows.
- Introduce `StagingStore`, `PublishQueue`, and `ObjectStore` abstractions.
- Keep local folder publishing compatible with current desktop smoke tests.
- Continue writing readable `transfer-log.jsonl` as an audit log.
- Record `final_location` for all new records, including local paths.
- Map legacy records without project identity into "Legacy Import" or "Unassigned".
- Add cleanup for stale incomplete temp files.
- Make dashboard and inbox queries project-scoped.

### Platform checkpoints

After the foundation schema exists, platform capabilities can be enabled behind focused checkpoints:

- Stop treating Android SAF labels as `output_dir`.
- Add Android storage config for selected final targets.
- Keep native receiver writing app-private staged files.
- Add SAF and MediaStore publishers.
- Add preview opening from `document_uri` and `media_uri`.
- Add retry and recovery UI for failed publishes.
- Keep desktop local folder behavior unchanged.

### Analysis checkpoints

The same SQLite foundation should leave room for later analysis features:

- Burst groups.
- Quality scores.
- Selection recommendations.
- Strategy profiles.

## Testing Strategy

Unit tests:

- Staging temp write, complete, and cleanup.
- Publish queue retry and recovery transitions.
- Duplicate filename reservation before publish.
- Legacy `final_path` records resolving as `LocalPath`.
- New records using `final_location`.
- Asset index grouping, paging, filters, and duplicate counts.
- Project creation, archive, and reassignment.
- Legacy import migration into "Legacy Import" or "Unassigned".
- Receiver start behavior when no active project exists.
- Dashboard and inbox queries requiring a project id.
- Moving assets or groups between projects.

Integration tests:

- FTP upload through staging then local object publish.
- SFTP random writes through staging then local object publish.
- Publish failure preserves staged bytes.
- Restart recovers queued staged uploads.
- Dashboard reads from transfer log or index with identical results.
- Project assignment survives receiver restart.
- Moving assets between projects updates dashboard views without touching final files.

Android tests:

- SAF permission persistence and revocation handling.
- Publish to selected document tree.
- Publish to MediaStore.
- Preview loading from platform URI.
- Receiver continues while publish queue retries are pending.

Device smoke tests:

- Real RAW/JPEG upload with app-private staging.
- Real RAW/JPEG publish to selected Android output target.
- Large video publish failure and retry.
- Storage full or permission revoked behavior.
- Multiple shooting projects imported on one device without mixing dashboard views.

## Acceptance Criteria

The storage model optimization is accepted when:

- Receiver uploads no longer require final storage to support random access or atomic rename.
- Desktop local-folder behavior remains compatible with current smoke tests.
- New transfer records store `final_location` as the canonical final target.
- Legacy records with `final_path` still appear in transfers, inbox groups, and dashboard.
- Android can distinguish app-private staging from user-selected final output.
- A completed upload whose final publish fails remains recoverable and retryable.
- Users can create/select projects and start imports into the active project.
- Every uploaded asset is associated with a project.
- Dashboard and inbox asset views require a selected project.
- Legacy imports remain visible under a safe fallback project.
- Dashboard behavior remains familiar while adding project filters.

## Non-Goals

- Changing FTP/SFTP protocol behavior.
- Mirroring camera-side folders into final storage.
- Decoding RAW files.
- Replacing current transfer-log audit history.
- Implementing cloud sync.
- Making physical final storage mirror project hierarchy mandatory.
- Adding any extra grouping layer between project and asset before there is a proven product need.

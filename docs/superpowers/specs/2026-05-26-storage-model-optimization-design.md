# Storage Model Optimization Design

## Goal

Evolve Camera Connector's storage model from a desktop-local file sink into a platform-aware import pipeline that works cleanly across desktop, Android SAF, Android MediaStore, future iOS storage, and NAS/headless targets.

The main design move is to separate **receiver staging**, **final object publishing**, **state/log persistence**, and **query indexing**. The receiver should keep a simple, reliable write target, while platform-specific storage rules live behind a publishing layer.

## Context

The current core already has the right product rules:

- FTP/SFTP receivers stream uploads through temporary files.
- Uploaded paths are flattened and sanitized.
- Duplicate filenames are preserved.
- Transfer records keep original camera paths and virtual display paths.
- `StoredObjectLocation` can represent `local_path`, `document_uri`, `media_uri`, and `photo_asset`.
- Android can persist a selected SAF directory label, but native smoke imports still write to app-private storage.

The gap is that `LocalFileSink` is still the real write path. It assumes final storage behaves like a local filesystem with seek, rename, path-based existence checks, and direct scanning. Those assumptions do not hold uniformly for SAF, MediaStore, Photos, or remote object stores.

## Decision

Adopt a staged import pipeline:

```text
Camera FTP/SFTP
  -> StagingStore
  -> PublishQueue
  -> ObjectStore
  -> TransferLog + AssetIndex
  -> Dashboard/UI
```

The receiver writes to `StagingStore`, which is always reliable local app-private storage for mobile and a local temporary directory for desktop/headless. After a complete upload, a publisher moves or copies the staged object into the configured `ObjectStore`. The final saved target is recorded as `StoredObjectLocation`.

For desktop, the object store can still be a local flat folder. For Android, the object store can be SAF or MediaStore. For future iOS, it can map to Files or Photos identifiers.

## Current Model To Preserve

These product rules must not change:

- Completed files are flat by final filename, not mirrored from camera paths.
- Original camera path remains metadata.
- Duplicate uploads are preserved without overwriting earlier completed files.
- RAW/JPEG/video grouping is based on normalized filename stem.
- Transfer rows expose display source, username, remote address, virtual display path, size, and failure text.
- Receiver metadata lives outside the user-facing inbox.
- Existing `transfer-log.jsonl` records remain readable.

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

The queue can start as a small JSON state file or table in the state directory. If SQLite is introduced for `AssetIndex`, the queue can live in the same database.

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

### `StateStore`

Owns serialized state mutations.

Responsibilities:

- Append transfer records.
- Update connected-device metadata.
- Update receiver runtime status.
- Persist publish queue state.
- Prevent concurrent read-modify-write corruption.

The current JSON/JSONL files can stay, but writes should move behind a single state boundary. The first implementation can use a mutex or single writer queue inside the process, plus atomic file replacement for JSON state files. A later implementation can use SQLite for stronger concurrency and query performance.

### `AssetIndex`

Owns query performance.

Responsibilities:

- Materialize completed transfer records into asset rows.
- Support dashboard pagination, filtering, grouping, duplicates, and facet counts.
- Avoid rebuilding every asset group from the full transfer log on each poll.
- Keep `transfer-log.jsonl` as the audit trail.

Initial option:

- `asset-index.json` derived from the transfer log for small imports.

Preferred scalable option:

- SQLite tables for transfers, assets, groups, devices, publish queue, and receiver status.

## Data Model Direction

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

Desktop can map `output_target` to a local folder. Android can map it to SAF or MediaStore configuration. The native FFI boundary should avoid passing SAF URI values as if they were filesystem paths.

## Android Storage Direction

Android should treat selected SAF/MediaStore destinations as final object stores, not as Rust `PathBuf` output directories.

Recommended flow:

1. `CoreGatewayFactory` seeds app-private `staging_dir` and `state_dir`.
2. Android persists selected SAF/MediaStore target in `AndroidStorageGateway`.
3. The Rust receiver writes staged files only.
4. Android or a native platform adapter publishes staged files into the selected target.
5. The final transfer record stores a platform `StoredObjectLocation`.
6. Dashboard previews open `document_uri` or `media_uri` through Android APIs.

This keeps the Rust receiver portable while letting Android own platform storage permissions.

## Failure And Recovery

The optimized model must handle these cases:

- Upload interrupted before staging completes: keep or remove incomplete temp file according to cleanup policy and record a failed receive when enough metadata exists.
- Upload completes but publish fails: keep staged file, mark publish failed, show retry action.
- App process dies after staging but before publish: recover queue on next startup.
- Final object exists: reserve a duplicate filename before publish.
- State write fails after final publish: preserve enough queue metadata to reconcile on next startup.
- SAF permission is revoked: keep staged files and prompt user to reauthorize storage.

No completed upload should disappear silently. If final publishing fails, the staged bytes remain recoverable until the user deletes them or a successful retry completes.

## Migration Plan

### Phase 1: Staging-first local pipeline

- Introduce `StagingStore` and `PublishQueue` abstractions.
- Keep final publishing to local folder only.
- Continue writing readable `transfer-log.jsonl`.
- Record `final_location` for all new records, including local paths.
- Add cleanup for stale incomplete temp files.

### Phase 2: Android app-private staging with explicit final target

- Stop treating Android SAF labels as `output_dir`.
- Add Android storage config for selected final target.
- Keep native receiver writing app-private staged files.
- Publish to local/app-private object store first, then SAF or MediaStore behind feature flags.

### Phase 3: Platform object stores

- Add SAF and MediaStore publishers.
- Add preview opening from `document_uri` and `media_uri`.
- Add retry and recovery UI for failed publishes.
- Keep desktop local folder behavior unchanged.

### Phase 4: Indexed dashboard queries

- Add `AssetIndex`.
- Move dashboard pagination, filters, duplicate detection, and facets to the index.
- Keep `transfer-log.jsonl` as an append-only audit trail.
- Consider SQLite when JSON index performance or concurrency becomes limiting.

## Testing Strategy

Unit tests:

- Staging temp write, complete, and cleanup.
- Publish queue retry and recovery transitions.
- Duplicate filename reservation before publish.
- Legacy `final_path` records resolving as `LocalPath`.
- New records using `final_location`.
- Asset index grouping, paging, filters, and duplicate counts.

Integration tests:

- FTP upload through staging then local object publish.
- SFTP random writes through staging then local object publish.
- Publish failure preserves staged bytes.
- Restart recovers queued staged uploads.
- Dashboard reads from transfer log or index with identical results.

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

## Acceptance Criteria

The storage model optimization is accepted when:

- Receiver uploads no longer require final storage to support random access or atomic rename.
- Desktop local-folder behavior remains compatible with current smoke tests.
- New transfer records store `final_location` as the canonical final target.
- Legacy records with `final_path` still appear in transfers, inbox groups, and dashboard.
- Android can distinguish app-private staging from user-selected final output.
- A completed upload whose final publish fails remains recoverable and retryable.
- Dashboard behavior remains the same for users while the internal query path can move toward an index.

## Non-Goals

- Changing FTP/SFTP protocol behavior.
- Mirroring camera-side folders into final storage.
- Decoding RAW files.
- Replacing current transfer-log audit history.
- Requiring SQLite in the first storage refactor.
- Implementing cloud sync.

# Storage Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the storage foundation described in `docs/superpowers/specs/2026-05-26-storage-model-optimization-design.md` while keeping smart selection out of scope.

**Architecture:** Add a SQLite-backed storage module in `core` that owns projects, transfers, assets, asset groups, and publish queue state. Keep FTP/SFTP receiver behavior stable by first dual-writing completed transfers into SQLite, then introduce staging and local publishing behind small abstractions.

**Tech Stack:** Rust 2021, `rusqlite`, existing `serde` models, existing FTP/SFTP tests, `cargo test`.

---

### Task 1: SQLite Storage Foundation

**Files:**
- Modify: `Cargo.toml`
- Modify: `core/Cargo.toml`
- Create: `core/src/storage/mod.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/storage_store_tests.rs`

- [x] Add `rusqlite` as a workspace dependency with the bundled SQLite feature.
- [x] Write failing tests for schema creation, required project identity, project-scoped asset grouping, duplicate preservation, and publish queue retry state.
- [x] Implement `SqliteStore` with schema initialization and models for `Project`, `StoredTransfer`, `StoredAsset`, `StoredAssetGroup`, and `PublishQueueItem`.
- [x] Run `cargo test -p camera_connector_core storage_store_tests`.
- [x] Commit with message `Add SQLite storage foundation`.

### Task 2: Project-Aware Service Queries

**Files:**
- Modify: `core/src/service.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/storage_service_tests.rs`

- [x] Write failing tests showing dashboard asset queries require a project id for SQLite-backed project views.
- [x] Add service methods to create/list/archive projects, resolve the active project, and query project asset groups from SQLite.
- [x] Keep existing log-backed methods working for old smoke tests during the transition.
- [x] Run `cargo test -p camera_connector_core storage_service_tests service_tests`.
- [x] Commit with message `Add project scoped storage service`.

### Task 3: Receiver Dual-Write

**Files:**
- Modify: `core/src/push/config.rs`
- Modify: `core/src/push/ftp.rs`
- Modify: `core/src/push/sftp.rs`
- Test: `core/tests/ftp_push_tests.rs`
- Test: `core/tests/sftp_push_tests.rs`

- [x] Write failing tests proving FTP/SFTP uploads create SQLite transfer, asset, and group rows under the active project.
- [x] Add `active_project_id` to receiver config and require it for storage-indexed receiver writes, defaulting tests to an explicit Inbox project.
- [x] After a completed transfer is appended to `transfer-log.jsonl`, also record it in `SqliteStore`.
- [x] Run `cargo test -p camera_connector_core ftp_push_tests sftp_push_tests storage_store_tests`.
- [x] Commit with message `Index receiver uploads in SQLite`.

### Task 4: Staging And Local Publishing

**Files:**
- Modify: `core/src/receive/sink.rs`
- Create: `core/src/storage/pipeline.rs`
- Modify: `core/src/storage/mod.rs`
- Test: `core/tests/storage_pipeline_tests.rs`
- Test: `core/tests/receive_sink_tests.rs`

- [x] Write failing tests for local staging temp writes, publish queue enqueue, local object publish, retry after publish failure, and stale staging cleanup.
- [x] Implement `LocalStagingStore`, `LocalFolderObjectStore`, and `StoragePipeline` for local publish.
- [x] Update FTP/SFTP to write staged files and publish through the pipeline while preserving final local-folder behavior.
- [x] Run `cargo test -p camera_connector_core storage_pipeline_tests receive_sink_tests ftp_push_tests sftp_push_tests`.
- [x] Commit with message `Add staging and local publish pipeline`.

### Task 5: Full Verification And Docs

**Files:**
- Modify: `docs/protocol.md`
- Modify: `docs/product/prototype-spec.md`
- Modify: `docs/superpowers/specs/2026-05-26-storage-model-optimization-design.md`

- [x] Update docs to describe project-scoped SQLite-backed storage and local staging/publish behavior.
- [x] Run `cargo test`.
- [x] Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify.ps1`.
- [x] Commit with message `Document storage foundation`.

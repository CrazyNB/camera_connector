# Nikon Wireless Importer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Nikon wireless import system that first proves PTP/IP access through a Rust CLI, then layers Android, iOS, desktop, and Flutter UI on top.

**Architecture:** Rust owns protocol parsing, camera sessions, object models, discovery, and download orchestration. CLI validates the core against mock and real cameras. Mobile and desktop apps call the Rust core through platform adapters and UniFFI after the protocol loop is stable.

**Tech Stack:** Rust, Tokio, Clap, tracing, UniFFI, Flutter, Kotlin Android, Swift iOS, MediaStore, iOS Local Network/Photos/Files APIs.

---

## Implementation Strategy

Build in this order:

1. Rust workspace and core types.
2. PTP/IP packet parser and dataset parser.
3. Mock camera server.
4. CLI protocol validation.
5. Real-camera compatibility logging.
6. UniFFI API boundary.
7. Android manual connection demo.
8. Android gallery, thumbnails, and downloads.
9. iOS and desktop after Android proves UX and core stability.

Do not start Flutter UI before the CLI can complete at least `connect -> GetDeviceInfo` against the mock camera. Do not start batch download UX before single-file download is reliable.

## Planned File Structure

```text
core/
  Cargo.toml
  src/
    lib.rs
    error.rs
    model/
      mod.rs
      endpoint.rs
      camera_info.rs
      camera_object.rs
      object_format.rs
      asset_group.rs
    ptp_ip/
      mod.rs
      packet.rs
      transport.rs
      init.rs
    ptp/
      mod.rs
      operation.rs
      response.rs
      dataset.rs
      session.rs
    camera/
      mod.rs
      client.rs
      capability.rs
      object_repository.rs
    download/
      mod.rs
      queue.rs
      progress.rs
      sink.rs
    scanner/
      mod.rs
      port_scan.rs
  tests/
    ptp_ip_packet_tests.rs
    ptp_dataset_tests.rs
    grouping_tests.rs
tools/
  cli/
    Cargo.toml
    src/main.rs
  mock_camera/
    Cargo.toml
    src/main.rs
bindings/
  uniffi.toml
  nikon_importer.udl
apps/
  mobile_flutter/
docs/
  compatibility.md
  protocol.md
  troubleshooting.md
```

## Phase 0: Repository Setup

### Task 0.1: Create Rust Workspace

**Files:**

- Create: `Cargo.toml`
- Create: `core/Cargo.toml`
- Create: `core/src/lib.rs`
- Create: `tools/cli/Cargo.toml`
- Create: `tools/cli/src/main.rs`
- Create: `tools/mock_camera/Cargo.toml`
- Create: `tools/mock_camera/src/main.rs`

- [ ] Add a workspace manifest with members `core`, `tools/cli`, and `tools/mock_camera`.
- [ ] Add dependencies only where needed: `tokio`, `thiserror`, `tracing`, `bytes`, `clap`, `serde`, `serde_json`, and `tempfile` for tests.
- [ ] Make `tools/cli` print a version banner.
- [ ] Run `cargo check --workspace`.
- [ ] Commit with `chore: create rust workspace`.

### Task 0.2: Add Project Docs

**Files:**

- Create: `docs/protocol.md`
- Create: `docs/compatibility.md`
- Create: `docs/troubleshooting.md`

- [ ] Add protocol notes with port `15740`, two-connection model, and minimum operation codes.
- [ ] Add compatibility table columns: model, firmware, mode, port, init, info, thumb, object, notes.
- [ ] Add troubleshooting entries for no camera found, timeout, and camera sleep.
- [ ] Commit with `docs: add protocol and compatibility notes`.

## Phase 1: Core Models and Errors

### Task 1.1: Define Public Models

**Files:**

- Create: `core/src/model/endpoint.rs`
- Create: `core/src/model/camera_info.rs`
- Create: `core/src/model/camera_object.rs`
- Create: `core/src/model/object_format.rs`
- Create: `core/src/model/asset_group.rs`
- Modify: `core/src/model/mod.rs`
- Modify: `core/src/lib.rs`

- [ ] Define `CameraEndpoint`, `EndpointSource`, `CameraInfo`, `CameraObject`, `ObjectFormat`, and `CameraAssetGroup`.
- [ ] Use owned strings and primitive fields that can later cross UniFFI cleanly.
- [ ] Implement `ObjectFormat::from_filename`.
- [ ] Run `cargo test -p nikon_importer_core`.
- [ ] Commit with `feat(core): add camera domain models`.

### Task 1.2: Add RAW+JPEG Grouping Tests

**Files:**

- Create: `core/tests/grouping_tests.rs`
- Modify: `core/src/model/asset_group.rs`

- [ ] Test that `DSC_1234.JPG` and `DSC_1234.NEF` become one group.
- [ ] Test that a standalone video remains a single group.
- [ ] Test that filename case does not break grouping.
- [ ] Implement `group_camera_objects(objects) -> Vec<CameraAssetGroup>`.
- [ ] Run `cargo test -p nikon_importer_core grouping`.
- [ ] Commit with `feat(core): group related camera assets`.

### Task 1.3: Define Error Types

**Files:**

- Create: `core/src/error.rs`
- Modify: `core/src/lib.rs`

- [ ] Define `ImporterError` variants from the design document.
- [ ] Add `pub type Result<T> = std::result::Result<T, ImporterError>`.
- [ ] Implement conversions from I/O errors and timeout errors.
- [ ] Add stable error codes for UI mapping.
- [ ] Run `cargo test -p nikon_importer_core`.
- [ ] Commit with `feat(core): add importer errors`.

## Phase 2: PTP/IP and PTP Parsing

### Task 2.1: Implement PTP/IP Packet Types

**Files:**

- Create: `core/src/ptp_ip/packet.rs`
- Modify: `core/src/ptp_ip/mod.rs`
- Create: `core/tests/ptp_ip_packet_tests.rs`

- [ ] Define packet kind constants for init command request/ack, init event request/ack, command request, response, data, and event.
- [ ] Encode and decode little-endian packet length and packet type.
- [ ] Reject packets shorter than header length.
- [ ] Reject declared length larger than configured maximum.
- [ ] Run packet tests.
- [ ] Commit with `feat(ptp-ip): parse packet framing`.

### Task 2.2: Implement PTP Operation and Response Types

**Files:**

- Create: `core/src/ptp/operation.rs`
- Create: `core/src/ptp/response.rs`
- Modify: `core/src/ptp/mod.rs`

- [ ] Define operation constants for the V0.0 command set.
- [ ] Define response code handling, including success and unsupported operation.
- [ ] Add helpers for transaction IDs and parameter arrays.
- [ ] Run `cargo test -p nikon_importer_core`.
- [ ] Commit with `feat(ptp): add operation and response types`.

### Task 2.3: Implement Dataset Parsers

**Files:**

- Create: `core/src/ptp/dataset.rs`
- Create: `core/tests/ptp_dataset_tests.rs`

- [ ] Parse PTP string fields.
- [ ] Parse DeviceInfo enough to extract manufacturer, model, version, supported operations, and supported formats.
- [ ] Parse StorageInfo enough to expose storage label and capacity when available.
- [ ] Parse ObjectInfo enough to expose filename, size, format, dimensions, and timestamps when present.
- [ ] Add fixture-based tests with small hand-built byte arrays.
- [ ] Commit with `feat(ptp): parse device and object datasets`.

### Task 2.4: Implement Transport

**Files:**

- Create: `core/src/ptp_ip/transport.rs`
- Create: `core/src/ptp_ip/init.rs`

- [ ] Create async TCP connection to `CameraEndpoint`.
- [ ] Add read timeout and write timeout.
- [ ] Send InitCommandRequest.
- [ ] Read and validate InitCommandAck.
- [ ] Open event connection and validate InitEventAck.
- [ ] Expose `send_packet`, `read_packet`, and `close`.
- [ ] Run tests that use the mock camera once Task 3.1 exists.
- [ ] Commit with `feat(ptp-ip): add async transport`.

## Phase 3: Mock Camera

### Task 3.1: Add Minimal Mock Camera Server

**Files:**

- Modify: `tools/mock_camera/src/main.rs`
- Create: `tools/mock_camera/src/protocol.rs`

- [ ] Listen on `127.0.0.1:15740` by default.
- [ ] Accept command and event connections.
- [ ] Respond to init packets.
- [ ] Log received operation codes.
- [ ] Add a CLI flag for port override.
- [ ] Run `cargo run -p mock_camera -- --host 127.0.0.1 --port 15740`.
- [ ] Commit with `test: add mock ptp-ip camera server`.

### Task 3.2: Add Mock Responses

**Files:**

- Modify: `tools/mock_camera/src/protocol.rs`
- Create: `tools/mock_camera/src/fixtures.rs`

- [ ] Respond to `GetDeviceInfo` with deterministic camera info.
- [ ] Respond to `OpenSession` and `CloseSession`.
- [ ] Respond to `GetStorageIDs`.
- [ ] Respond to `GetObjectHandles` with JPEG and NEF handles.
- [ ] Respond to `GetObjectInfo`.
- [ ] Respond to `GetThumb` with a tiny JPEG fixture.
- [ ] Respond to `GetObject` with deterministic bytes and size.
- [ ] Commit with `test: add mock camera object responses`.

## Phase 4: Camera Session and Client

### Task 4.1: Add PTP Session

**Files:**

- Create: `core/src/ptp/session.rs`

- [ ] Manage `session_id`.
- [ ] Increment `transaction_id` per operation.
- [ ] Implement `open`, `close`, and `operation`.
- [ ] Map response codes into `ImporterError`.
- [ ] Add mock-camera integration tests.
- [ ] Commit with `feat(core): add ptp session`.

### Task 4.2: Add NikonCameraClient

**Files:**

- Create: `core/src/camera/client.rs`
- Create: `core/src/camera/capability.rs`
- Create: `core/src/camera/object_repository.rs`

- [ ] Implement `connect(endpoint)`.
- [ ] Implement `get_camera_info`.
- [ ] Implement `probe_capabilities`.
- [ ] Implement `list_objects`.
- [ ] Implement `get_thumbnail`.
- [ ] Implement `download_object`.
- [ ] Run against mock camera through tests.
- [ ] Commit with `feat(core): add nikon camera client`.

### Task 4.3: Add Download Sink

**Files:**

- Create: `core/src/download/sink.rs`
- Create: `core/src/download/progress.rs`

- [ ] Define a sink abstraction for file output and future platform output.
- [ ] Implement local file sink using temporary file then rename.
- [ ] Emit progress events with handle, bytes written, total bytes, and state.
- [ ] Test failed download leaves no published final file.
- [ ] Commit with `feat(download): add safe file sink`.

## Phase 5: CLI

### Task 5.1: Implement `info`

**Files:**

- Modify: `tools/cli/src/main.rs`

- [ ] Add `info --host <ip> --port <port>`.
- [ ] Connect through `NikonCameraClient`.
- [ ] Print manufacturer, model, firmware, and supported operation count.
- [ ] Test against mock camera.
- [ ] Commit with `feat(cli): add camera info command`.

### Task 5.2: Implement `scan`

**Files:**

- Create: `core/src/scanner/port_scan.rs`
- Modify: `tools/cli/src/main.rs`

- [ ] Add `scan --subnet <cidr>`.
- [ ] Probe TCP port `15740`.
- [ ] Limit concurrency.
- [ ] Print responsive endpoints as soon as they are found.
- [ ] Commit with `feat(cli): add lan scan command`.

### Task 5.3: Implement `list`

**Files:**

- Modify: `tools/cli/src/main.rs`

- [ ] Add `list --host <ip>`.
- [ ] Print handle, filename, format, size, capture time, and group key.
- [ ] Sort newest first when capture time is available.
- [ ] Test against mock camera.
- [ ] Commit with `feat(cli): list camera objects`.

### Task 5.4: Implement `thumb`

**Files:**

- Modify: `tools/cli/src/main.rs`

- [ ] Add `thumb --host <ip> --handle <id> --output <file>`.
- [ ] Fetch thumbnail bytes.
- [ ] Write to output path.
- [ ] Exit with a clear error when thumbnail is unavailable.
- [ ] Test against mock camera.
- [ ] Commit with `feat(cli): fetch object thumbnail`.

### Task 5.5: Implement `pull`

**Files:**

- Modify: `tools/cli/src/main.rs`

- [ ] Add `pull --host <ip> --handle <id> --output <dir>`.
- [ ] Download via file sink.
- [ ] Print progress.
- [ ] Verify final file exists and has expected size when object size is known.
- [ ] Test against mock camera.
- [ ] Commit with `feat(cli): download camera object`.

## Phase 6: Real-Camera Validation

### Task 6.1: Real Camera Smoke Test

**Files:**

- Modify: `docs/compatibility.md`
- Modify: `docs/troubleshooting.md`

- [ ] Run `scan` in phone hotspot mode.
- [ ] Run `info` against discovered IP.
- [ ] Run `list`.
- [ ] Run `thumb` for one JPEG handle.
- [ ] Run `pull` for one JPEG and one NEF.
- [ ] Record model, firmware, connection mode, and results.
- [ ] Commit with `docs: record first nikon compatibility result`.

### Task 6.2: Protocol Hardening

**Files:**

- Modify: `core/src/ptp_ip/transport.rs`
- Modify: `core/src/ptp/session.rs`
- Modify: `core/src/camera/client.rs`
- Modify: `docs/troubleshooting.md`

- [ ] Add timeout configuration.
- [ ] Improve unknown response logging.
- [ ] Ensure session close is attempted after failures.
- [ ] Add retry only where operation is safe.
- [ ] Re-run mock and real-camera smoke tests.
- [ ] Commit with `fix(core): harden camera session handling`.

## Phase 7: UniFFI Boundary

### Task 7.1: Define Stable FFI API

**Files:**

- Create: `bindings/uniffi.toml`
- Create: `bindings/nikon_importer.udl`
- Modify: `core/Cargo.toml`
- Modify: `core/src/lib.rs`

- [ ] Expose endpoint, camera info, camera object, object format, and capability types.
- [ ] Expose async-safe methods through a runtime wrapper if needed.
- [ ] Keep platform storage out of UniFFI at first.
- [ ] Generate Kotlin and Swift bindings locally.
- [ ] Commit with `feat(bindings): add uniffi api surface`.

### Task 7.2: Add FFI Smoke Tests

**Files:**

- Create: `bindings/tests/README.md`
- Modify: `docs/troubleshooting.md`

- [ ] Document generated binding commands.
- [ ] Validate Kotlin can call `connect`, `get_camera_info`, and `list_objects` against mock camera.
- [ ] Validate Swift can compile generated bindings when iOS work begins.
- [ ] Commit with `test(bindings): document ffi smoke flow`.

## Phase 8: Android V0.1 Demo

### Task 8.1: Create Flutter App Shell

**Files:**

- Create: `apps/mobile_flutter/pubspec.yaml`
- Create: `apps/mobile_flutter/lib/main.dart`
- Create: `apps/mobile_flutter/lib/app.dart`
- Create: `apps/mobile_flutter/lib/screens/connect_screen.dart`

- [ ] Create a minimal Flutter app with Connect screen.
- [ ] Add manual IP and port fields.
- [ ] Add connect button and state display.
- [ ] Keep visual design compact and utilitarian.
- [ ] Commit with `feat(android): add flutter connect shell`.

### Task 8.2: Add Android Platform Plugin

**Files:**

- Create: `apps/mobile_flutter/android/app/src/main/kotlin/.../NikonPlatformPlugin.kt`
- Create: `apps/mobile_flutter/android/app/src/main/kotlin/.../AndroidNetworkAdapter.kt`

- [ ] Load Rust core library.
- [ ] Call generated UniFFI binding.
- [ ] Add manual connect method.
- [ ] Return camera info to Flutter.
- [ ] Surface stable error codes.
- [ ] Commit with `feat(android): connect to rust camera core`.

### Task 8.3: Add Gallery List

**Files:**

- Create: `apps/mobile_flutter/lib/screens/gallery_screen.dart`
- Create: `apps/mobile_flutter/lib/models/camera_object_view.dart`
- Create: `apps/mobile_flutter/lib/view_models/gallery_view_model.dart`

- [ ] Call list objects after connection.
- [ ] Show filename, format badge, size, and downloaded state.
- [ ] Add filters for all, JPG, RAW, video, and not downloaded.
- [ ] Add newest-first sorting.
- [ ] Commit with `feat(android): show camera object list`.

### Task 8.4: Add Thumbnail Loading

**Files:**

- Modify: `apps/mobile_flutter/lib/screens/gallery_screen.dart`
- Modify: `apps/mobile_flutter/lib/view_models/gallery_view_model.dart`
- Modify: `apps/mobile_flutter/android/app/src/main/kotlin/.../NikonPlatformPlugin.kt`

- [ ] Request thumbnails only for visible items.
- [ ] Cache thumbnails on disk.
- [ ] Show a fallback file tile if unavailable.
- [ ] Limit concurrent thumbnail calls.
- [ ] Commit with `feat(android): load gallery thumbnails lazily`.

### Task 8.5: Add Single Download

**Files:**

- Create: `apps/mobile_flutter/lib/screens/download_screen.dart`
- Create: `apps/mobile_flutter/lib/view_models/download_view_model.dart`
- Create: `apps/mobile_flutter/android/app/src/main/kotlin/.../AndroidMediaStoreSaver.kt`

- [ ] Download one selected object.
- [ ] Save JPEG/NEF to the right platform destination.
- [ ] Show progress.
- [ ] Allow retry after failure.
- [ ] Commit with `feat(android): download selected camera object`.

## Phase 9: Android V0.2 Complete Experience

### Task 9.1: Add LAN Scan

**Files:**

- Modify: `core/src/scanner/port_scan.rs`
- Modify: `apps/mobile_flutter/lib/screens/connect_screen.dart`
- Modify: `apps/mobile_flutter/android/app/src/main/kotlin/.../AndroidNetworkAdapter.kt`

- [ ] Scan current subnet.
- [ ] Try last successful IP first.
- [ ] Add camera AP default option.
- [ ] Stream found endpoints into UI.
- [ ] Commit with `feat(android): discover cameras on lan`.

### Task 9.2: Add Batch Download Queue

**Files:**

- Create: `core/src/download/queue.rs`
- Modify: `apps/mobile_flutter/lib/view_models/download_view_model.dart`
- Create: `apps/mobile_flutter/android/app/src/main/kotlin/.../AndroidDownloadService.kt`

- [ ] Add queued, running, completed, failed, and canceled states.
- [ ] Default original download concurrency to `1`.
- [ ] Add cancel and retry.
- [ ] Add foreground service for long downloads.
- [ ] Commit with `feat(android): add batch download queue`.

## Phase 10: iOS and Desktop

### Task 10.1: iOS Manual Connection

**Files:**

- Create: `apps/mobile_flutter/ios/Runner/NikonPlatformPlugin.swift`
- Create: `apps/mobile_flutter/ios/Runner/LocalNetworkPermission.swift`
- Create: `apps/mobile_flutter/ios/Runner/IOSPhotoSaver.swift`

- [ ] Add Local Network permission copy.
- [ ] Connect through generated Swift binding.
- [ ] List objects and thumbnails.
- [ ] Save JPEG to Photos and NEF/video to Files/App Documents.
- [ ] Commit with `feat(ios): add manual camera import flow`.

### Task 10.2: Desktop CLI Packaging

**Files:**

- Modify: `tools/cli/Cargo.toml`
- Modify: `docs/troubleshooting.md`

- [ ] Add release build instructions.
- [ ] Validate Windows, macOS, and Linux output paths.
- [ ] Add `sync --skip-existing`.
- [ ] Commit with `feat(cli): add desktop sync flow`.

## Verification Checklist

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] Mock camera test: `info`, `list`, `thumb`, `pull`
- [ ] Real camera smoke test recorded in `docs/compatibility.md`
- [ ] Android manual IP test in phone hotspot mode
- [ ] Android manual IP test in same-LAN mode
- [ ] Download failure leaves no corrupt published file
- [ ] Logs contain no image bytes, full private EXIF, or precise GPS coordinates

## Definition of Done for First Useful Release

The first useful release is Android V0.1 backed by a proven Rust CLI. It is done when:

- CLI passes mock-camera tests.
- CLI can connect to at least one real Nikon camera and download JPEG/NEF.
- Android app can manually connect to that camera.
- Android app can list objects and show visible thumbnails.
- Android app can download one JPEG and one NEF with retry.
- Known unsupported camera behavior is documented rather than hidden.

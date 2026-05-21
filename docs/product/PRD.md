# Camera Connector PRD

## 1. Product Positioning

Camera Connector is a local wireless import receiver for cameras. The current route is push-based: the camera sends JPEG, RAW, and video files to a receiver running on the phone, computer, or later a NAS.

One-line positioning:

> A local camera push import receiver.

## 2. Current Technical Decision

The previous brand-specific PTP/IP pull route is deprecated for this project. Real-camera validation showed that an already-paired workflow can reject a generic direct PTP/IP client and may be owned by an official receiver process. We will not continue reverse pairing or authentication work.

Current priorities:

1. FTP push mode.
2. SFTP push mode.
3. AP mode remains the original camera-hotspot meaning, but is paused.

## 3. Users

- Field photographers who need quick local transfer without a card reader.
- RAW+JPEG shooters who need grouped imports and clear file sizes.
- Desktop/NAS users who want repeatable receiver-side ingest.
- Technical validators who need a compatibility table across camera vendors, bodies, and firmware versions.

## 4. Core Scenarios

### 4.1 Phone Hotspot Or LAN FTP Push

1. User starts the receiver on phone/computer.
2. App shows receiver IP, port, protocol, username, and import save location.
3. User configures the camera FTP upload profile.
4. Camera sends files to the receiver.
5. App atomically publishes completed files into a flat inbox, keeps receiver state/logs outside the upload location, and groups RAW/JPEG pairs.

### 4.2 Desktop Batch Receiver

1. User starts FTP receiver on Windows/macOS/Linux.
2. Camera sends selected files or a batch.
3. Receiver writes all completed files into one configured flat save location, regardless of the camera's remote upload path setting.
4. App records transfer id, original camera path, final filename, remote address, and optional user-set source name for filtering and virtual display paths.

### 4.3 AP Mode

AP mode still means the camera creates its own Wi-Fi AP and the phone/computer joins it. This is not part of the current implementation milestone.

## 5. MVP Scope

P0:

- Run a local FTP receiver.
- Print camera-facing receiver settings.
- Accept passive FTP `STOR` uploads.
- Write uploaded files through a temporary file and publish only on success.
- Flatten uploaded paths to the final filename only; do not mirror camera-side folders locally.
- Keep system config, receiver state/logs, and uploaded assets in separate locations.
- Represent saved objects as platform storage locations, not only desktop filesystem paths.
- Sanitize uploaded filenames.
- Group RAW/JPEG/video assets by filename stem.
- Recognize common RAW formats across vendors: NEF/NRW, CR2/CR3, ARW/SRF/SR2, RAF, RW2/RWL, ORF, PEF, and DNG.
- Preserve duplicate uploads without overwriting completed files.
- Scan the receiver inbox as the product's import source.
- Record a transfer log for filtering by login username, transfer id, original path, final filename, source name, and remote address.
- Display files as virtual paths such as `Z5_2/BB/DSC_2552.NEF` or `IP-056/BB/DSC_2552.NEF` while keeping local storage flat.
- Let users configure camera accounts with FTP/SFTP username, password, and device name.
- Persist camera account passwords as core-generated hashes; never store or display plaintext passwords after setup.
- Show current and recently connected devices from receiver metadata so the latest IP is visible as connection state, not account identity.
- Expose receiver runtime lifecycle from core: stopped, starting, running, stopping, and failed.
- Persist receiver runtime status as receiver metadata and detect stale `Running` status when the listener is no longer reachable.
- Expose an app dashboard read-model that combines receiver status, connected devices, asset summary, and a paged asset list.
- Provide the same account model, flat sink, transfer log, connected-device metadata, runtime status, and temporary-file publish behavior for SFTP core receiver validation.
- Record real-camera compatibility results.

P1:

- Add authentication polish.
- Add duplicate detection.
- Validate SFTP receiver with real cameras and update compatibility coverage.

P2:

- Resume AP-mode validation.
- Add mobile foreground/background service behavior.
- Add NAS/headless packaging.

## 6. Non-Goals

- PTP/IP direct import.
- Pairing reverse engineering.
- Remote camera control.
- Live View.
- Camera setting changes.
- Cloud sync.
- RAW decoding or image editing.

## 7. Functional Requirements

| ID | Requirement | Priority | Acceptance |
| --- | --- | --- | --- |
| RX-001 | Start local FTP receiver | P0 | Receiver listens on configured host and port |
| RX-002 | Show receiver settings | P0 | CLI/UI shows protocol, host, port, configured accounts, password status, import save location, and state/log location |
| RX-003 | Accept passive FTP upload | P0 | A client can upload a file through `PASV` + `STOR` |
| RX-004 | Atomic publish | P0 | Final file appears only after full upload succeeds |
| RX-005 | Flat safe path handling | P0 | `/DCIM/100CANON/IMG_1001.CR3` lands as `IMG_1001.CR3`; traversal and unsafe filename characters cannot escape output folder |
| RX-005A | Storage separation | P0 | Account config, receiver state/logs, and uploaded assets are stored separately; upload inbox contains camera files and temporary upload files only |
| RX-006 | Asset grouping | P0 | Matching JPG and RAW stems such as `IMG_1001.JPG` and `IMG_1001.CR3` appear as one group |
| RX-007 | Inbox scan | P0 | Receiver output folder can be scanned into grouped assets |
| RX-008 | Duplicate policy | P0 | Re-uploading `IMG_1001.CR3` creates `IMG_1001 (1).CR3` |
| RX-009 | Compatibility log | P0 | Each real-camera test updates `docs/compatibility.md` |
| RX-010 | Transfer log | P0 | Each completed transfer records transfer id, original path, final filename, platform final location, bytes, protocol, optional login username, remote address, and optional source name |
| RX-011 | Tag-style filters and virtual paths | P0 | Inbox and transfer views can filter by format, login username, source name, remote address, transfer id, and original path; display path resolves username to the current account device name, then falls back to source name or `IP-###` plus original path without creating local subfolders |
| RX-012 | Camera account configuration | P0 | User can list, set, and remove camera accounts with username, password, and device name; FTP and SFTP receivers authenticate against these accounts, store password hashes rather than plaintext passwords, and reject invalid account config |
| RX-013 | Connected device view | P0 | FTP and SFTP receivers record current/recent device IPs, login username, and online state; receiver startup clears stale online state from previous runs |
| RX-014 | Receiver runtime lifecycle | P0 | Core exposes start, stop, and status with phase, protocol, authentication mode, local address, output directory, account count, and failure message; persisted status survives process boundaries and stale running state is reported as stopped |
| RX-014A | Dashboard read-model | P0 | Core exposes one dashboard query for UI shells with receiver status, connected devices, filtered asset summary, and paged asset groups; CLI can emit the same model as JSON for app shells and automation |
| RX-015 | SFTP route | P1 | SSH/SFTP receiver accepts password-authenticated uploads through the same account model, flat sink, transfer log, connected-device metadata, runtime status model, and temporary-file publish behavior as FTP |
| RX-016 | Cross-platform storage backend | P1 | Core write flow uses a storage backend contract; desktop uses local paths, while Android/iOS can save through media/document/photo APIs without leaking platform URIs into receiver protocol logic |
| AP-001 | Camera AP mode | P2 | Keep original AP meaning; resume after push path works |

## 8. Success Metrics

- FTP receiver accepts a real-camera JPEG upload.
- FTP receiver accepts a real-camera RAW upload.
- Completed files have correct byte length.
- Camera-side upload folders do not create local subfolders.
- Failed uploads do not leave final files.
- Duplicate uploads do not overwrite earlier completed files.
- Transfer log filters can group files by source name, original path, remote address, and transfer id.
- User can configure the camera using only the receiver settings shown by the app.

## 9. Architecture

```mermaid
flowchart LR
  Camera["Camera\nFTP/SFTP upload profile"]
  Network["Phone hotspot / LAN\nAP later"]
  Receiver["Push Receiver\nFTP first"]
  Sink["Storage Backend\n.tmp then publish"]
  State["State/Logs\nstatus + devices + transfer log + host key"]
  Index["Asset Index\nformat + RAW/JPEG grouping"]
  CoreConfig["Core Config\naccounts + credential storage"]
  Service["Core Service\nreceiver + views"]
  Runtime["Core Runtime\nstart + stop + status"]
  UI["CLI / Mobile / Desktop UI"]

  Camera --> Network --> Receiver --> Sink --> Index --> Service --> UI
  Receiver --> State --> Service
  CoreConfig --> Receiver
  CoreConfig --> Service
  Service --> Runtime --> Receiver
  UI --> Service
  UI --> Runtime
```

The CLI is a thin operational adapter for development, validation, headless/NAS use, and field diagnostics. Product behavior belongs in core so desktop, mobile, and CLI clients share one receiver, account, config, logging, inbox, storage-location model, and view model. `CameraConnectorService` is the app-facing core entry point for building receiver config, reading inbox groups, reading transfer views, and reading connected-device views. `CameraConnectorRuntime` owns receiver lifecycle state and exposes start, stop, and status for app shells.

## 10. Milestones

1. Clean PTP/IP route from code and docs.
2. Build FTP receiver core and CLI smoke path.
3. Validate with one real camera in FTP mode.
4. Update compatibility table and receiver setup guide.
5. Validate SFTP receiver with real cameras and fill the compatibility matrix.
6. Resume AP-mode exploration only after push import is stable.


# Mobile App Handoff

## 1. Purpose

This document defines the first mobile implementation slice for Camera Connector after prototype review. The mobile app should be a thin product shell over the existing core concepts: receiver settings, camera accounts, runtime status, connected devices, transfer log, and grouped inbox assets.

The current prototype is the UX source of truth:

```text
prototypes/camera-connector/index.html
```

## 2. Product Shell

Primary tabs:

- Overview
- Inbox
- Transfers

Secondary pages:

- Receiver Settings
- Device Accounts
- Asset Detail
- Transfer Detail
- Diagnostics

Do not make Receiver Settings or Device Accounts bottom-tab destinations. They are setup and management surfaces, not daily primary tasks.

## 3. Core Capability Mapping

| Mobile surface | Core capability | Notes |
| --- | --- | --- |
| Overview | `CameraConnectorService::project_dashboard` | Single project-scoped read model for status, paths, accounts, devices, transfers, failures, and assets |
| Receiver Settings | `CameraConnectorService::set_receiver_settings` | Patch-style updates; unspecified values remain unchanged |
| Device Accounts | `set_account`, `remove_account`, dashboard `accounts` | Password is write-only during setup; display configured/not required |
| Start/Stop Receiver | `CameraConnectorRuntime::start_receiver`, `stop_receiver`, `status` | Mobile shell owns foreground/background lifecycle |
| Inbox | project dashboard `assets` or `project_asset_group_page_with_query` | Use the active project as the required viewing scope |
| Transfers | `project_transfers`, `project_transfer_summary_with_query`, `project_recent_failed_transfers` | Failed rows must show error text and retry guidance |
| Connected Devices | dashboard `accounts` and `devices` | Account is identity; IP is mutable connection state |

## 4. Platform Storage Contract

The core already separates storage concepts:

- Config: app settings and accounts.
- State/logs: transfer log, connected devices, receiver status, SFTP host key.
- Output/inbox: completed camera assets and in-progress temp writes only.

Mobile implementations should preserve this separation.

Android likely maps to:

- Config/state: app private storage.
- Output: MediaStore or Storage Access Framework document tree.
- Saved location records: `media_uri` or `document_uri`.

iOS likely maps to:

- Config/state: app container.
- Output: app container, Files document provider, or Photos.
- Saved location records: `document_uri` or `photo_asset`.

Do not expose platform URI details through FTP/SFTP protocol behavior. Receiver upload handling should keep using the storage backend contract: temporary write first, then publish final object.

## 5. Receiver Lifecycle Requirements

Mobile app must handle:

- Start receiver with configured defaults.
- Stop receiver explicitly.
- Show stale running state as stopped if listener is no longer reachable.
- Keep receiver visible when foreground service / local network permission is required.
- Surface local network permission and firewall/hotspot guidance.
- Avoid losing completed files on app background or interruption.

Android-specific risks:

- Foreground service requirement for long-running receiver.
- Hotspot/LAN IP selection and local network routing.
- SAF write permission persistence.

iOS-specific risks:

- Background execution limits for FTP/SFTP listener.
- Local network permission prompt.
- Files/Photos write authorization and asset persistence.

## 6. Minimal Mobile Slice

The first implementation slice should prove these behaviors:

1. Configure one receiver profile.
2. Configure one camera account.
3. Start FTP receiver.
4. Show camera-facing host, port, username, password configured state.
5. Accept one real camera JPEG.
6. Accept one real camera RAW.
7. Show grouped inbox row.
8. Show transfer log row.
9. Stop receiver.

SFTP can stay behind a validation flag until real camera compatibility is confirmed.

## 7. UI State Rules

Overview:

- If receiver is running, show protocol, host, port, auth mode, online accounts, transfer health.
- If receiver is stopped, show last status and primary start action.
- If recent failures exist, show the latest failure card.

Inbox:

- Default sort is latest received first.
- File rows display virtual path, not local folder hierarchy.
- Duplicate rows show duplicate index/count.
- Filters are tags, not folders.

Transfers:

- Failed rows show error text.
- Retry is instruction-only: user retries from camera.
- Final location kind is visible in details for diagnostics.

Device Accounts:

- Username and device name are stable identity.
- IP address is latest connection metadata.
- Password value is never shown after saving.

## 8. Acceptance Checklist

- Bottom navigation has only Overview, Inbox, Transfers.
- Receiver settings are reachable from Overview.
- Device accounts are reachable from Overview and Receiver Settings.
- The app can render dashboard data without extra joins in the UI layer.
- Long filenames, paths, and transfer ids wrap cleanly on small screens.
- The app can operate with no direct filesystem path for output objects.
- Failed transfer diagnostics are visible without inspecting raw logs.
- Config/state/output locations remain separate.

## 9. Open Decisions Before Native Build

- Android-first, iOS-first, or shared shell first.
- Whether the Rust core will be embedded directly or exposed through a local service boundary.
- Which Android output strategy is preferred for MVP: MediaStore album or SAF directory.
- Which iOS output strategy is acceptable for MVP: app container, Files, or Photos.
- How much background receive support is required for the first mobile validation.

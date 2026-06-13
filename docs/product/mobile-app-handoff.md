# Mobile App Handoff

## 1. Purpose

This document defines the mobile implementation slice for Camera Connector after prototype review. The mobile app should be a thin product shell over the existing core concepts: projects, receiver settings, camera accounts, runtime status, connected devices, transfer log, publish queue, and grouped project assets.

The early HTML prototype remains a historical reference:

```text
prototypes/camera-connector/index.html
```

The current UX and visual source of truth is the Figma interaction design:

```text
https://www.figma.com/design/mKSknurwc2LWS83UWe0ReA
```

## 2. Product Shell

Global destinations:

- Projects
- Accounts
- Settings

Settings secondary destinations:

- Diagnostics

Project workspace:

- Receiver launch/status panel
- Photos

Receiver controls belong inside the active project workspace. Account management and settings stay global because they are not owned by a single project. Diagnostics are reachable from Settings rather than a standalone bottom tab. Asset detail remains a child page under Photos; transfer and publish records are surfaced through receiver status and Settings diagnostics instead of standalone project tabs.

## 3. Core Capability Mapping

| Mobile surface | Core capability | Notes |
| --- | --- | --- |
| Projects | `create_project`, `set_active_project`, `rename_project`, `archive_project`, `restore_project` | Project is the required viewing and import scope |
| Project workspace | `CameraConnectorService::project_dashboard` | Single project-scoped read model for status, paths, accounts, devices, transfers, failures, publish queue, and assets |
| Receiver panel | `CameraConnectorService::set_receiver_settings` | Patch-style updates; unspecified values remain unchanged; edited inline before Start applies the current form |
| Accounts | `set_account`, `remove_account`, dashboard `accounts` | Password is write-only during setup; account is global identity |
| Start/Stop Receiver | `CameraConnectorRuntime::start_receiver`, `stop_receiver`, `status` | Mobile shell owns foreground/background lifecycle; receiver starts from the active project workspace |
| Photos | project dashboard `assets` or `project_asset_group_page_with_query` | Tap preview opens group detail; long press enters bulk selection |
| Technical gate | `technical_assessments` / provider-aware analysis drain | Local CV risk context only; not a product-facing final score |
| Model provider profiles | `loadModelProviderSettings`, `loadModelProviderSettingsList`, `saveModelProviderSettings`, `deleteModelProviderSettings` | Global app config; projects select a profile instead of storing provider secrets |
| Prompt packs | `loadGlobalPromptPacks`, `loadPromptPacks`, `createGlobalPromptPack`, `forkPromptPack`, `savePromptPack` | Gateway methods expose global prompt packs. User-editable Markdown preference is only one part of a locked model request/response contract |
| Project evaluation settings | `loadProjectEvaluationSettings`, `saveProjectEvaluationSettings` | Project-scoped enablement for automatic model evaluation and burst recommendation |
| Manual model evaluation | detail refresh action and bulk/long-press evaluation action | No separate enablement setting; still requires provider capability |
| Model recommendation | `generateProjectRecommendation`, burst-stable analysis jobs | Burst recommendation can be automatic per project; project recommendation is manual-only |
| User marks | asset-group user mark APIs through dashboard/action gateway | Favorite and mark are human choices and never overwrite model recommendation rows |
| Transfer/write status | `project_transfers`, `project_transfer_summary_with_query`, `project_recent_failed_transfers`, write-queue summary and retry/drain APIs | Receiver status shows compact health and retry affordances; Settings diagnostics shows rows and errors |
| Settings diagnostics | dashboard `accounts`, `devices`, `transfers`, receiver status | Account is identity; IP is mutable connection state; transfer/write records are operational diagnostics |

## 4. Platform Storage Contract

The core already separates storage concepts:

- Config: app settings and accounts.
- State/logs: transfer log, connected devices, receiver status, and engineering SFTP host key when that validation path is used.
- Output location: completed camera assets and in-progress temp writes only.

Mobile implementations should preserve this separation.

Android likely maps to:

- Config/state: app private storage.
- Output: Storage Access Framework document tree for MVP, with app-private fallback when no tree is selected.
- Saved location records: `document_uri` for SAF output or `local_path` for app-private fallback.
- MediaStore is deferred unless physical-device validation proves a concrete need.

iOS likely maps to:

- Config/state: app container.
- Output: app container, Files document provider, or Photos.
- Saved location records: `document_uri` or `photo_asset`.

Do not expose platform URI details through receiver protocol behavior. Receiver upload handling should keep using the storage backend contract: temporary write first, then publish final object.

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

- Background execution limits for a long-running local receiver.
- Local network permission prompt.
- Files/Photos write authorization and asset persistence.

## 6. Minimal Mobile Slice

The first implementation slice should prove these behaviors:

1. Create or select one shooting project.
2. Configure one receiver profile from the project receiver panel.
3. Configure one camera account from the global Accounts surface.
4. Start FTP receiver inside the active project.
5. Show camera-facing host, port, username, password configured state.
6. Accept one real camera JPEG.
7. Accept one real camera RAW.
8. Show grouped project photo tile and detail.
9. Show transfer log row in diagnostics when needed.
10. Show write-queue state when local output writing is pending or failed.
11. Collapse the receiver panel into a compact running status so Photos remains the main project surface.
12. Stop receiver.

SFTP stays behind engineering validation until real camera compatibility is confirmed; it is not part of the current mobile user path.

## 7. UI State Rules

Project receiver panel:

- If receiver is running, collapse to a compact status with protocol, host, port, auth mode, online accounts, transfer health, and an expand/control affordance.
- If receiver is stopped, show last status and primary start action.
- If recent failures exist, show the latest failure card.
- Pressing Start applies the current protocol/host/port/camera-facing IP values; there is no separate save action.

Photos:

- Default sort is latest received first.
- Grid tiles display previews and compact group metadata.
- Tap the preview opens group detail.
- Long press enters selection mode for bulk delete, model evaluation, burst split, and manual burst merge.
- Burst groups aggregate to one tile. The tile cover uses the model-selected
  member when available; detail can show the group as a carousel and a compact
  overview.
- Collection filters are query semantics: All, Model Selects, Favorites, Marked,
  Technical Risk, Pending Analysis.
- Model score/reason, technical risk, favorite, and mark must be visually
  distinct. Favorite/mark actions are human choices; model-selected state is
  algorithm output.
- If provider capability is missing, model actions show a provider setup state.
  Uploads and local technical gate results still appear normally.

Settings diagnostics:

- Failed transfer rows show error text.
- Retry is instruction-only for camera-side transfer failures; publish retry remains an app action.
- Final location kind is visible where available for diagnostics.

Accounts:

- Username and device name are stable identity.
- IP address is latest connection metadata.
- Password value is never shown after saving.

Model provider settings:

- Provider profiles are global app configuration, not project data.
- Each profile can store URL, provider kind, model name, send mode, batch size,
  and API key in app-private config storage.
- Projects choose a profile by id; changing a global profile list must not
  silently rewrite existing project settings.
- Deleting a profile makes dependent project model actions unconfigured until
  the user selects another profile.

Prompt settings:

- Prompt packs are global resources grouped by package folder for sharing.
- Built-in packs are read-only; editing creates a user-owned copy.
- A pack stores user-editable Markdown preference text in `Prompt.md` plus
  product metadata. The image payload contract, task instructions, JSON schema,
  safety constraints, and output parsing rules stay locked in the system
  prompt/request builder.
- Projects select a prompt pack; they do not copy provider secrets or prompt
  protocol into project data.

Project intelligence:

- Project scene is a first-level project setting.
- Automatic workflow, model provider, prompt pack selection, and technical risk
  thresholds are secondary project intelligence settings.
- Technical thresholds can use Loose, Standard, Strict, or a custom per-project
  threshold set. Portrait-specific controls appear only when the project scene
  is Portrait.

## 8. Acceptance Checklist

- Global navigation has Projects, Accounts, and Settings.
- Settings contains Diagnostics as a secondary page.
- Projects opens Project Management first; entering a project opens the photo-first project workspace.
- Receiver settings and receiver start/stop are reachable from the project receiver panel.
- Accounts are managed from the global Accounts surface.
- The app can render dashboard data without extra joins in the UI layer.
- Long filenames, paths, and transfer ids wrap cleanly on small screens.
- The app can operate with no direct filesystem path for output objects.
- Failed transfer diagnostics are visible without inspecting raw logs.
- Config/state/output locations remain separate.
- Model provider URL, model name, and API key can be created, updated, deleted,
  and selected through Settings.
- Project configuration can select a provider profile and prompt pack without
  copying API keys into project data.
- Missing provider/API key state disables model work but does not block upload,
  thumbnails, grouping, publishing, or local CV.

## 9. Remaining Validation Decisions

- Android is the active first mobile target; iOS remains a later platform decision.
- The Rust core is embedded behind the native gateway boundary for Android.
- SAF directory publishing is the Android MVP output path; MediaStore can remain deferred unless a real-device workflow proves it is needed.
- Which iOS output strategy is acceptable for MVP: app container, Files, or Photos.
- How much background receive support is required after physical-device and real-camera validation.

## 10. Deferred Specs

- Project package migration is intentionally deferred until the Android project workflow and real-camera import loop are validated. The dormant protocol spec lives at `docs/superpowers/specs/2026-05-28-project-package-migration-protocol-design.md`.

# Android App Architecture

## 1. Decision

Camera Connector Android uses **Kotlin + Jetpack Compose + Material 3** as the native shell, with the existing **Rust core** embedded behind a small gateway boundary.

This is the first mobile target. It should prove the product loop on Android before any cross-platform abstraction is introduced.

## 2. Why Native Android

Camera Connector depends on Android-native surfaces that are central to the product, not incidental implementation details:

- A long-running FTP/SFTP receiver needs an Android foreground service and a persistent notification.
- Output storage needs platform-specific handling through MediaStore or the Storage Access Framework.
- The app must expose the active host, port, protocol, account identity, transfer health, and failure diagnostics while the receiver is running.
- Network behavior depends on hotspot/LAN state, local IP selection, and permissions.

Flutter, React Native, or Tauri Mobile would still require native modules for these capabilities. For the first Android slice, that extra bridge would slow down receiver validation.

## 3. Module Shape

```text
apps/android/
  app/
    MainActivity
    Compose UI shell
    ReceiverForegroundService
    Android storage gateway
    Core gateway interface
    NativeMobileCore Kotlin adapter
    NativeCoreGateway dashboard adapter

core/
  Existing Rust service, receiver, transfer log, accounts, grouping

core-ffi/
  Mobile-facing Rust facade, cdylib/staticlib target, C ABI and JSON DTO boundary

future generated bindings/
  UniFFI or JNI Kotlin bindings over core-ffi
```

## 4. Android Responsibilities

The Android layer owns:

- Activity navigation and Compose rendering.
- Foreground service lifecycle.
- Android notification channels and notification permission guidance.
- Storage permission request and persisted document-tree URI.
- App-private config/state file locations.
- User-facing setup instructions for camera-side FTP/SFTP profiles.

The Android layer should not reimplement:

- Transfer log semantics.
- RAW/JPEG/video grouping.
- Duplicate detection.
- Account identity rules.
- Receiver protocol behavior.

Those remain core responsibilities.

## 5. Core Gateway Boundary

UI code talks to a Kotlin `CoreGateway` interface:

- `observeDashboard()`
- `startReceiver()`
- `stopReceiver()`
- `saveReceiverSettings()`
- `saveDeviceAccount()`

The first skeleton may use an in-memory gateway for layout and lifecycle wiring. Native Rust bindings should later implement the same interface.

This keeps the UI from depending directly on FFI-generated types and allows the product shell to evolve while the Rust bridge is built.

The Rust side now has a `camera-connector-ffi` crate that exposes a mobile-facing `MobileCore` facade. Its first contract is JSON-based so it can be verified in the normal Rust workspace before Android SDK/NDK builds are available.

The same crate also exports a narrow C ABI:

- `camera_connector_mobile_core_create`
- `camera_connector_mobile_core_destroy`
- `camera_connector_mobile_core_free_string`
- `camera_connector_mobile_core_project_dashboard_json`
- `camera_connector_mobile_core_project_group_assets_json`
- `camera_connector_mobile_core_save_receiver_settings_json`
- `camera_connector_mobile_core_save_device_account_json`
- `camera_connector_mobile_core_start_receiver_json`
- `camera_connector_mobile_core_stop_receiver_json`

The consumable C contract lives in:

```text
core-ffi/include/camera_connector_mobile.h
```

Every string-returning call returns a JSON envelope:

```json
{"ok":true,"value":{},"error":null}
```

or:

```json
{"ok":false,"value":null,"error":"message"}
```

Android JNI bindings keep this envelope at the native boundary, then map it into typed Kotlin state before it reaches Compose screens.

The Android source now has the Kotlin side of that bridge:

- `NativeMobileCore` owns the native handle, loads `camera_connector_ffi`, calls external functions, and unwraps the JSON envelope.
- `NativeCoreGateway` ensures an active project and adapts project dashboard JSON into the existing `CoreGateway` model used by Compose.
- `CoreGatewayFactory` chooses either `PreviewCoreGateway` or `NativeCoreGateway` through `BuildConfig.USE_NATIVE_CORE`; release-facing verification builds use the native gateway.
- `ReceiverServiceController` sends receiver start/stop commands to `ReceiverForegroundService`.
- `ReceiverForegroundService` owns the long-running native receiver lifecycle and foreground notification.
- The foreground notification deep-links back into `MainActivity` and exposes a Stop action backed by an immutable service `PendingIntent`.
- Receiver service lifecycle events are logged through the `CameraConnectorReceiver` tag so adb diagnostics can separate app receiver failures from generic Android runtime crashes.
- Connected-device smoke testing builds, installs, launches, verifies package presence, and collects adb diagnostics through `scripts\smoke_android_device.ps1`.
- Emulator FTP verification covers native account creation, receiver start, passive upload of RAW/JPEG pairs, inbox photo-grid display, photo detail navigation, transfer rows, and adb diagnostics through `scripts\verify_android_emulator_ftp_upload.ps1`.
- The same upload verifier accepts `-RealAssetDirectory`, selects a matching RAW/JPEG filename stem from the folder, and uploads the real bytes. It has been exercised with Nikon `.NEF + .JPG` files from `D:\ps\Photos\2026\5\5.4`.
- Device account setup flows through the same gateway: Compose collects device name, camera login username, and a write-only password; `NativeCoreGateway` passes that password to the Rust core so the persisted config stores the core-generated password hash rather than plaintext.
- Receiver setup also flows through the gateway: Compose can save protocol, bind host, and one unified camera-facing port while keeping Android output storage behind the platform publish boundary. The Android UI deliberately hides separate FTP/SFTP port fields; the selected port is written to both core fields.
- The native dashboard includes both runtime status and saved receiver settings. Android uses runtime status while the receiver is running, and falls back to saved settings while stopped so saved protocol/host/port changes are reflected immediately in Overview.
- Emulator UI verification now covers saving SFTP settings and confirming the Overview endpoint updates before switching the emulator-only bind host to `0.0.0.0` for start/stop verification.
- Transfer diagnostics are mapped from the native dashboard into Compose: transfer counts remain visible and recent failed transfers show the core-provided virtual display path plus error text.
- Account connection diagnostics are also mapped: Overview can show active connection count, latest remote endpoint, last seen time, and last disconnected time without treating IP address as account identity.
- Receiver runtime diagnostics are mapped as well: Overview shows phase, authentication mode, account count, and core failure message so service start failures are visible in-app.
- Android directory selection is wired at the platform boundary: `MainActivity` launches SAF document tree selection, `AndroidStorageGateway` persists the URI permission and display label, and the Output card shows the selection. Native imports stage into app-private storage first, then the Android publish worker writes to the selected SAF tree and records `document_uri`; without a selected tree it falls back to app-private file storage. The worker resolves the publish target per item, so reselecting or reauthorizing the output directory can recover failed queued publishes while the receiver keeps running.
- Android maps the native dashboard `publish_queue` summary into its UI model. The Overview receiver tags surface pending or failed publishes so storage permission loss and other recoverable publish failures are visible without inspecting logs.
- The native dashboard also exposes recent project-scoped publish failures. Android appends those failures to the Transfers list with the affected filename and last error so output permission problems are actionable from the normal diagnostics surface.
- UI actions that call the gateway are wrapped with local error handling. Native exceptions from start, stop, receiver settings, or account save operations appear as a dismissible Overview error card.
- UI actions also publish an in-flight label while native gateway calls are running. Related controls are disabled during that window to avoid duplicate start, stop, settings, or account operations.
- The Inbox UI is photo-first: it defaults to a persisted 3-column grid, allows a 2-column switch, keeps tile metadata compact, uses JPEG previews when available, and moves full path/source/RAW-JPEG detail into the photo detail screen.

The Rust side now exports JNI symbols for Kotlin `NativeMobileCore`:

- `create`
- `destroy`
- `projectDashboardJson`
- `projectGroupAssetsJson`
- `saveReceiverSettingsJson`
- `saveDeviceAccountJson`
- `startReceiverJson`
- `stopReceiverJson`

The JNI shim reuses the same `MobileCore` facade as the C ABI, so Android-specific binding code does not duplicate receiver, account, dashboard, transfer, or receiver lifecycle logic.

Android native packaging is produced by:

```text
scripts/build_android_native.ps1
```

The script builds `core-ffi` for `aarch64-linux-android` and `x86_64-linux-android`, then copies `libcamera_connector_ffi.so` into the matching `apps/android/app/src/main/jniLibs` ABI folders.

Android APK verification is handled by:

```text
scripts/verify_android_build.ps1
```

That script builds the native arm64 and x86_64 libraries, assembles the debug APK with native core enabled, and checks that the APK contains both packaged libraries.

Native gateway builds can be produced without editing source code:

```text
gradle :app:assembleDebug -PcameraConnector.useNativeCore=true
```

The native gateway now routes start/stop through `ReceiverForegroundService`, so Android owns the long-running foreground lifecycle while Rust owns receiver behavior and status. Account setup, receiver network settings, project state, SAF publishing, and publish queue visibility also cross the gateway, keeping authentication, import state, and dashboard persistence in the shared core. The remaining bridge work is native gateway device smoke testing and optional MediaStore publishing.

`NativeCoreGateway` polls the native dashboard every 2 seconds while it is open. This keeps receiver status, connected accounts, transfer failures, and newly imported assets moving into Compose without coupling the UI to service internals.

Native receiver `local_addr` values are parsed into separate host and port fields before they reach Compose, so the Overview screen can render a stable `host:port` label across FTP, SFTP, IPv4, hostnames, and bracketed IPv6 addresses.

Android 13+ notification permission is treated as a receiver start prerequisite. `AndroidPermissionGateway` checks `POST_NOTIFICATIONS`, `MainActivity` owns the permission launcher, and the Overview screen disables Start until notifications are available.

## 6. Storage Strategy

MVP strategy:

- Config/state: app-private storage.
- Current native smoke inbox: app-private `filesDir/inbox`.
- User-facing output/inbox: user-selected SAF document tree when configured, otherwise app-private fallback.
- Display path: virtual camera path from transfer log, not Android filesystem path.

The Android bootstrap only seeds `output_dir` and `state_dir` into native receiver settings. It must not reset protocol, host, or ports on app startup because those are user-configurable receiver settings.

Android URI values stay inside the storage gateway. The dashboard and inbox still use product concepts: source name, username, transfer id, original path, format, duplicate count, and final location kind.

## 7. Receiver Lifecycle

The receiver starts through `ReceiverForegroundService`.

Rules:

- Starting receiver always creates a foreground notification.
- Stopping receiver tears down the core receiver and removes the foreground notification.
- UI treats missing listener or failed service start as stopped with a visible failure.
- If notification permission or storage permission is missing, the UI surfaces setup actions instead of silently failing.

## 8. First Android Slice

The first buildable slice should include:

1. Compose shell with Overview, Inbox, and Transfers bottom tabs.
2. Secondary entry points for Receiver Settings and Device Accounts.
3. Foreground service stub with notification channel.
4. Storage gateway for SAF directory selection and persisted URI permission.
5. `CoreGateway` interface and in-memory gateway.
6. A native bridge placeholder that can later be replaced by UniFFI/JNI bindings.

After the skeleton compiles, the next milestone is connecting the foreground service to the Rust core on Android.

## 9. Current Version Targets

As of 2026-05-22, the Android skeleton targets:

- Android Gradle Plugin 9.2.0
- Kotlin 2.3.21
- Compose BOM 2026.05.00
- compileSdk 36
- minSdk 26

These versions are intentionally centralized in Gradle files so they can be revised before the first CI build if the local Android SDK differs.

Local Android build verification is available through:

```text
scripts/verify_android_build.ps1
```

The current Windows development setup uses JDK 17, Android SDK platform 36, build-tools 36.0.0, and Gradle 9.5.1.

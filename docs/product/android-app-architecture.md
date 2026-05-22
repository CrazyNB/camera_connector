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
- `camera_connector_mobile_core_dashboard_json`
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
- `NativeCoreGateway` adapts native dashboard JSON into the existing `CoreGateway` model used by Compose.
- `CoreGatewayFactory` chooses either `PreviewCoreGateway` or `NativeCoreGateway` through `BuildConfig.USE_NATIVE_CORE`.
- `ReceiverServiceController` sends receiver start/stop commands to `ReceiverForegroundService`.
- `ReceiverForegroundService` owns the long-running native receiver lifecycle and foreground notification.
- `PreviewCoreGateway` remains the default app entry until native service smoke testing is complete.
- Device account setup flows through the same gateway: Compose collects device name, FTP/SFTP username, and a write-only password; `NativeCoreGateway` passes that password to the Rust core so the persisted config stores the core-generated password hash rather than plaintext.

The Rust side now exports JNI symbols for Kotlin `NativeMobileCore`:

- `create`
- `destroy`
- `dashboardJson`
- `saveReceiverSettingsJson`
- `saveDeviceAccountJson`
- `startReceiverJson`
- `stopReceiverJson`

The JNI shim reuses the same `MobileCore` facade as the C ABI, so Android-specific binding code does not duplicate receiver, account, dashboard, transfer, or receiver lifecycle logic.

Android arm64 native packaging is produced by:

```text
scripts/build_android_native.ps1
```

The script builds `core-ffi` for `aarch64-linux-android` and copies `libcamera_connector_ffi.so` into `apps/android/app/src/main/jniLibs/arm64-v8a`.

Android APK verification is handled by:

```text
scripts/verify_android_build.ps1
```

That script builds the native arm64 library, assembles the debug APK, and checks that the APK contains `lib/arm64-v8a/libcamera_connector_ffi.so`.

Native gateway builds can be produced without editing source code:

```text
gradle :app:assembleDebug -PcameraConnector.useNativeCore=true
```

The native gateway now routes start/stop through `ReceiverForegroundService`, so Android owns the long-running foreground lifecycle while Rust owns receiver behavior and status. Account setup also crosses the gateway, keeping authentication rules in the shared core. The remaining bridge work is storage directory selection, settings editing, and native gateway device smoke testing.

`NativeCoreGateway` polls the native dashboard every 2 seconds while it is open. This keeps receiver status, connected accounts, transfer failures, and newly imported assets moving into Compose without coupling the UI to service internals.

Native receiver `local_addr` values are parsed into separate host and port fields before they reach Compose, so the Overview screen can render a stable `host:port` label across FTP, SFTP, IPv4, hostnames, and bracketed IPv6 addresses.

Android 13+ notification permission is treated as a receiver start prerequisite. `AndroidPermissionGateway` checks `POST_NOTIFICATIONS`, `MainActivity` owns the permission launcher, and the Overview screen disables Start until notifications are available.

## 6. Storage Strategy

MVP strategy:

- Config/state: app-private storage.
- Current native smoke inbox: app-private `filesDir/inbox`.
- Future user-facing output/inbox: user-selected SAF document tree.
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

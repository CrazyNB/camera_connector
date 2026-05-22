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

core/
  Existing Rust service, receiver, transfer log, accounts, grouping

core-ffi/
  Mobile-facing Rust facade, cdylib/staticlib target, JSON DTO boundary

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

## 6. Storage Strategy

MVP strategy:

- Config/state: app-private storage.
- Output/inbox: user-selected SAF document tree.
- Display path: virtual camera path from transfer log, not Android filesystem path.

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

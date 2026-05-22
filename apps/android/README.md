# Camera Connector Android

This is the native Android shell for Camera Connector.

The app is intentionally thin:

- Kotlin + Jetpack Compose renders the product shell.
- Android owns foreground service, notification, storage permission, and platform lifecycle.
- The existing Rust core remains the source of truth for receiver behavior, transfer log, account state, and inbox grouping.

## Current State

This directory is a scaffold. It establishes the project layout and Android boundaries before wiring the Rust core into the APK.

The next implementation step is a `core-ffi` bridge that exposes the current Rust service API to Kotlin through UniFFI or JNI.

The Android source now has a native gateway boundary:

- `NativeMobileCore` owns the native handle and JSON envelope parsing.
- `NativeCoreGateway` adapts native dashboard JSON into the Compose-facing `CoreGateway`.
- `CoreGatewayFactory` chooses the preview gateway or native gateway from `BuildConfig.USE_NATIVE_CORE`.
- `CoreGatewayFactory` seeds only native receiver paths with app-private `filesDir/inbox` and `filesDir/state`.
- `ReceiverForegroundService` owns native receiver start/stop while Android keeps the foreground notification alive.
- `AndroidPermissionGateway` gates receiver start on Android 13+ notification permission.
- The Overview screen can create camera accounts with device name, username, and a write-only password that is handed to the Rust core for hashed persistence.
- The Overview screen can save receiver protocol, bind host, FTP port, and SFTP port through the native gateway; output storage stays app-private until Android directory selection is wired.
- The Transfers screen reads native dashboard transfer counts and recent failure rows, including virtual display paths and core error messages.
- Account rows include current connection count, latest remote endpoint, and last seen/disconnected timestamps from the native dashboard.
- The Receiver card shows native runtime phase, authentication mode, configured account count, and receiver diagnostic message.
- The Output card launches Android's document tree picker and persists the selected inbox URI label; native smoke imports still use app-private storage until the Android storage backend writes through SAF or MediaStore.
- Compose receiver/account actions catch native gateway exceptions and show a dismissible action error card instead of failing silently.
- Rust exports matching JNI symbols from `core-ffi`, including receiver start/stop.

`PreviewCoreGateway` remains the default entry point while native service smoke testing is still in progress. Build with `-PcameraConnector.useNativeCore=true` for native gateway smoke testing.

## Local Build Prerequisites

- JDK 17
- Android SDK with API 36
- Gradle or a checked-in Gradle wrapper

This machine currently uses:

- JDK: `C:\Program Files\ojdkbuild\java-17-openjdk-17.0.3.0.6-1`
- Android SDK: `%LOCALAPPDATA%\Android\Sdk`
- Gradle: `%LOCALAPPDATA%\CameraConnectorToolchains\gradle-9.5.1`

Run the Android build check from the repository root. It builds the Rust native library for Android arm64, assembles the debug APK, and verifies the APK contains `lib/arm64-v8a/libcamera_connector_ffi.so`.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_build.ps1
```

To generate a debug APK that boots through the native gateway:

```powershell
cd apps\android
%LOCALAPPDATA%\CameraConnectorToolchains\gradle-9.5.1\bin\gradle.bat :app:assembleDebug --no-daemon -PcameraConnector.useNativeCore=true
```

To rebuild only the Rust native library and copy it into the APK source tree:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build_android_native.ps1
```

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
- `CoreGatewayFactory` seeds native receiver settings with app-private `filesDir/inbox` and `filesDir/state`.
- `ReceiverForegroundService` owns native receiver start/stop while Android keeps the foreground notification alive.
- `AndroidPermissionGateway` gates receiver start on Android 13+ notification permission.
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

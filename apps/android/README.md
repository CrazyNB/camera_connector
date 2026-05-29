# Camera Connector Android

This is the native Android shell for Camera Connector.

The app is intentionally thin:

- Kotlin + Jetpack Compose renders the product shell.
- Android owns foreground service, notification, storage permission, and platform lifecycle.
- The existing Rust core remains the source of truth for receiver behavior, transfer log, account state, and inbox grouping.

## Current State

This directory now contains the working native Android shell. The debug APK can be built with the Rust native receiver packaged for `arm64-v8a` and `x86_64`, installed on a device or emulator, and verified through the automated FTP account/login/upload smoke path.

The Android source now has a native gateway boundary:

- `NativeMobileCore` owns the native handle and JSON envelope parsing.
- `NativeCoreGateway` adapts native dashboard JSON into the Compose-facing `CoreGateway`.
- `CoreGatewayFactory` chooses the preview gateway or native gateway from `BuildConfig.USE_NATIVE_CORE`.
- `CoreGatewayFactory` seeds only native receiver paths with app-private `filesDir/inbox` and `filesDir/state`.
- `ReceiverForegroundService` owns native receiver start/stop while Android keeps the foreground notification alive.
- `ReceiverForegroundService` also runs a publish queue worker that claims staged items from Rust and publishes them through the Android storage boundary; when the user has selected a document tree it publishes to SAF and records `document_uri`, otherwise it falls back to app-private file storage. The worker resolves the target for each publish so changing or reauthorizing the output directory can recover queued failures without restarting the receiver.
- The foreground notification can reopen the app and exposes a Stop action so receiver shutdown is available outside the Compose UI.
- `AndroidPermissionGateway` gates receiver start on Android 13+ notification permission.
- The Accounts screen can create camera accounts with device name, username, and a write-only password that is handed to the Rust core for hashed persistence.
- The project receiver panel applies protocol, bind host, one unified camera-facing port, and the camera-facing setup IP when Start is pressed; the Kotlin adapter writes that port into both core protocol fields so the UI does not expose duplicate port settings.
- Diagnostics are reachable from Settings and read native dashboard transfer counts plus recent failure rows, including virtual display paths and core error messages.
- Account rows include current connection count, latest remote endpoint, and last seen/disconnected timestamps from the native dashboard.
- The Receiver card shows native runtime phase, authentication mode, configured account count, and receiver diagnostic message.
- The Output card launches Android's document tree picker and persists the selected inbox URI label; native imports use that selected SAF tree as the final publish target when it is available.
- The receiver panel and collapsed running status surface pending and failed publish queue counts from the Rust dashboard, so SAF permission loss or other retryable publish failures are visible in-app.
- Publish queue failures remain visible through the receiver status and Settings diagnostics surface, alongside completed imports and failed receiver transfers.
- Compose receiver/account actions catch native gateway exceptions and show a dismissible action error card instead of failing silently.
- Long-running receiver/account actions show a working card and disable related controls while the native gateway call is in flight.
- Rust exports matching JNI symbols from `core-ffi`, including receiver start/stop.
- The app opens on Project Management. Entering a project opens the photo-first project workspace: the receiver launch panel starts expanded while stopped, collapses into a compact running status after start, and the rest of the page is the project photo grid with compact tiles, JPEG previews from local paths or SAF document URIs, tap-to-detail, and long-press selection.

The Gradle default still keeps `USE_NATIVE_CORE=false` for lightweight IDE preview builds. Product verification and install scripts build with `-PcameraConnector.useNativeCore=true`, which is the path used for emulator and device validation.

## Local Build Prerequisites

- JDK 17
- Android SDK with API 36
- Gradle or a checked-in Gradle wrapper

This machine currently uses:

- JDK: `C:\Program Files\ojdkbuild\java-17-openjdk-17.0.3.0.6-1`
- Android SDK: `%LOCALAPPDATA%\Android\Sdk`
- Gradle: `%LOCALAPPDATA%\CameraConnectorToolchains\gradle-9.5.1`

Run the Android build check from the repository root. It builds the Rust native library for Android arm64 and x86_64, assembles the debug APK with native core enabled, and verifies the APK contains both `lib/arm64-v8a/libcamera_connector_ffi.so` and `lib/x86_64/libcamera_connector_ffi.so`.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_build.ps1
```

To generate a debug APK that boots through the native gateway:

```powershell
cd apps\android
%LOCALAPPDATA%\CameraConnectorToolchains\gradle-9.5.1\bin\gradle.bat :app:assembleDebug --no-daemon -PcameraConnector.useNativeCore=true
```

To build, install, and launch the debug APK on a connected Android device:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install_android_debug.ps1
```

If multiple adb devices are connected, pass `-Serial <serial>`. Use `-SkipBuild` for a quick reinstall of the existing APK or `-NoLaunch` when you only want to install.

To collect package, foreground service, and receiver logcat diagnostics from a connected Android device:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\collect_android_diagnostics.ps1
```

Diagnostics are written under `target\android-diagnostics\<timestamp>`. The receiver service logs under the `CameraConnectorReceiver` tag.

For a full connected-device smoke pass, use:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\smoke_android_device.ps1
```

The smoke pass builds and installs the debug APK, grants notification permission when available, verifies `pm path com.cameraconnector.app`, launches the app, and writes diagnostics to `target\android-diagnostics\smoke-latest`.

If the APK was already installed manually after approving a phone-side USB install prompt, pass `-SkipInstall` to validate the installed app launch and diagnostics without reinstalling.

For a real-camera Android validation pass, run the connected-device smoke first, then use the Android Physical Device Test Template in `docs\compatibility.md` to record camera login, foreground service behavior, SAF publish, project photo/detail visibility, transfer rows, and diagnostics path.

To generate a per-device real-camera test record with prefilled adb preflight data:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\prepare_android_real_camera_test.ps1 -Serial <serial> -RunSmoke -SkipInstall
```

To verify Android FTP import with a real RAW/JPEG folder, pass `-RealAssetDirectory` to the emulator upload script:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_emulator_ftp_upload.ps1 -Serial emulator-5554 -RealAssetDirectory "D:\ps\Photos\2026\5\5.4"
```

The script finds a matching RAW/JPEG filename stem, uploads the real bytes through the Android receiver, and verifies the transfer log, project photo grid, JPEG preview, photo detail, and transfer list. Without this parameter it still uses the small synthetic RAW/JPEG pair for quick smoke runs.

To preflight a connected Android device before installing:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\preflight_android_device.ps1
```

The preflight checks Android SDK level, packaged ABI support (`arm64-v8a` or `x86_64`), package notification permission state, and device IP interfaces, then writes `target\android-device-preflight.txt`. The full smoke script runs this automatically before install.

To inspect an already built APK and write an auditable package report:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\inspect_android_apk.ps1
```

The build verification script runs this automatically and writes `target\android-apk-report.txt` with the APK size, SHA-256 hash, manifest/classes entries, and packaged native library sizes for `arm64-v8a` and `x86_64`.

To rebuild only the Rust native library and copy it into the APK source tree:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build_android_native.ps1
```

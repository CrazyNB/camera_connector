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

`PreviewCoreGateway` remains the default entry point until an Android SDK/NDK build is available and the JNI shim is linked into the APK.

## Local Build Prerequisites

- JDK 17
- Android SDK with API 36
- Gradle or a checked-in Gradle wrapper

This workspace currently does not assume a local Android SDK is installed.

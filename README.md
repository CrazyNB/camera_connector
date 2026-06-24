# Camera Connector

Chinese version: `README.zh-CN.md`.

Camera Connector is a local camera ingest and photo triage system. Cameras push
JPEG, RAW, and video files to a local receiver; the shared Rust core records the
transfer facts, indexes the files into explicit shooting projects, groups related
assets, and exposes the same project model to desktop, Android, and CLI shells.

The current product route is FTP push. Android shows FTP as the active receiver
path and keeps future STC-style work disabled. SFTP remains an engineering
validation surface in the core and CLI, not the main Android setup path.

## Repository Map

```text
core/                  Shared Rust domain, storage, receiver, service, analysis
core-ffi/              C ABI and JNI facade for mobile shells
apps/android/          Native Android app: Kotlin, Compose, foreground service
apps/desktop/          Tauri desktop app: TypeScript workbench + Rust commands
tools/cli/             Thin command-line adapter over the shared Rust core
scripts/               Verification, Android build, device, and smoke helpers
docs/                  Product, architecture, protocol, development, validation
prototypes/            Static product prototype artifacts
```

Generated output is kept out of the product architecture: `target/`, Android
Gradle build folders, desktop `node_modules/`, IDE folders, and `.git/` are local
tooling or build state.

## Semantic Architecture

The system is organized around separated product semantics:

- Receiver facts: FTP/SFTP listener lifecycle, authenticated connections,
  transfer records, connected devices, and runtime status.
- Asset facts: stored objects, RAW/JPEG/video roles, grouping, duplicates, and
  source metadata.
- Project scope: user-created shooting projects that own imported assets,
  dashboard reads, scans, sync, and evaluation settings.
- Human decisions: favorite, marked, guest marks, manual burst edits, and delete
  actions.
- Local technical assessment: objective CV risk and gate context.
- Model evaluation: provider-backed photographic score, tier, and summary.
- Selection recommendation: model recommendation output only.
- Publishing: staged upload bytes, final platform storage, write retry state.
- Sharing and sync: LAN share sessions, guest marks, and project snapshots.
- Platform shells: desktop, Android, and CLI adapters over the same core.

The longer module guide lives in `docs/architecture.md`.

## Main Entry Points

- Shared service facade: `core/src/service.rs`
- Public core exports: `core/src/lib.rs`
- SQLite store and schema: `core/src/storage/`
- Receiver runtime: `core/src/runtime.rs`
- Mobile facade: `core-ffi/src/lib.rs`
- Android gateway boundary: `apps/android/app/src/main/java/com/cameraconnector/app/core/CoreGateway.kt`
- Android foreground receiver: `apps/android/app/src/main/java/com/cameraconnector/app/service/ReceiverForegroundService.kt`
- Desktop Tauri backend: `apps/desktop/src-tauri/src/lib.rs`
- Desktop TypeScript API: `apps/desktop/src/desktopApi.ts`
- CLI entry point: `tools/cli/src/main.rs`

## Development Setup

Windows development expects:

- Rust stable through `rustup`
- Visual Studio 2022 Build Tools with the C++ workload
- PowerShell
- JDK 17, Android SDK 36, and Gradle for Android verification
- Node.js/npm for the desktop TypeScript workbench

See `docs/development.md` for detailed commands.

## Verification

Run the full shared-core and CLI verification:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\verify.ps1
```

Useful focused checks:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_mobile_ffi_contract.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_skeleton.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\verify_android_build.ps1
```

Desktop logic tests run from `apps/desktop`:

```powershell
cmd /c npm ci
cmd /c npm run test:logic
```

## Documentation

Start with:

- `docs/README.md` for the documentation index
- `docs/architecture.md` for semantic and module boundaries
- `docs/product/PRD.md` for product requirements
- `docs/protocol.md` for receiver and storage protocol notes
- `apps/android/README.md` for Android build and device validation
- `docs/development.md` for local development and verification
- `docs/compatibility.md` for real-camera and Android device records
- `docs/troubleshooting.md` for receiver triage

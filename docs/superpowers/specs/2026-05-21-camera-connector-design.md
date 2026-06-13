# Camera Connector Design

## Goal

Replace the failed PTP/IP pull route with a push receiver. The current Android
product route is camera-to-app FTP upload. SFTP remains an engineering/core
validation path, while the Android UI reserves the secondary receiver slot for a
future STC-style mode and renders it disabled until that mode exists.

## Architecture

- `receive`: local atomic file sink and receive progress types.
- `model`: received asset metadata, import source, multi-brand RAW format detection, RAW/JPEG grouping.
- `push`: receiver protocol configuration, FTP receiver, and engineering SFTP
  support behind non-primary tooling.
- `tools/cli`: validation commands for receiver settings, local ingest, and FTP serving.

## Current Scope

- Delete PTP/IP, scanner, direct camera client, and mock PTP camera code.
- Implement passive FTP `STOR` receiver.
- Keep FTP as the only active Android user-facing receiver route for now.
- Keep the STC secondary route as a disabled product placeholder.
- Preserve local file safety through sanitized relative paths and temporary-file writing.
- Preserve duplicate uploads with numbered filenames.
- Scan completed receiver uploads into project-owned asset groups.
- Keep AP mode paused while preserving its original meaning.

## Verification

- Unit/integration tests for grouping, local sink, and passive FTP upload.
- `scripts/verify.ps1` runs format, clippy, tests, build, and CLI push smoke.

## Deferred

- STC product definition and implementation.
- Secondary push-route validation only if it becomes a product requirement.
- Real-camera AP-mode validation.
- Mobile background receiver behavior.


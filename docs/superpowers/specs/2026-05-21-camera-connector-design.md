# Camera Connector Design

## Goal

Replace the failed PTP/IP pull route with a push receiver. The camera sends files to us over FTP first, with SFTP as the second push protocol.

## Architecture

- `receive`: local atomic file sink and receive progress types.
- `model`: received asset metadata, import source, multi-brand RAW format detection, RAW/JPEG grouping.
- `push`: receiver protocol configuration and FTP receiver.
- `tools/cli`: validation commands for receiver settings, local ingest, and FTP serving.

## Current Scope

- Delete PTP/IP, scanner, direct camera client, and mock PTP camera code.
- Implement passive FTP `STOR` receiver.
- Preserve local file safety through sanitized relative paths and temporary-file publishing.
- Preserve duplicate uploads with numbered filenames.
- Scan the receiver output folder into grouped inbox assets.
- Keep AP mode paused while preserving its original meaning.

## Verification

- Unit/integration tests for grouping, local sink, and passive FTP upload.
- `scripts/verify.ps1` runs format, clippy, tests, build, and CLI push smoke.

## Deferred

- SFTP real-camera validation and compatibility matrix updates.
- Real-camera AP-mode validation.
- Mobile background receiver behavior.


# Nikon Push Importer Implementation Plan

## Phase 1: Cleanup

- Remove PTP/IP packets, sessions, datasets, scanner, direct client, and mock camera.
- Remove CLI commands that imply pull-mode import.
- Update PRD, protocol notes, compatibility table, troubleshooting, and prototype spec.

## Phase 2: Push Core

- Add `ReceivedAsset`, `ImportSource`, and `ReceivedAssetGroup`.
- Add `ReceiveProgress`, `ReceiveState`, and `LocalFileSink`.
- Add safe relative path handling and atomic publish.
- Add duplicate filename preservation.
- Add inbox scanning and grouped asset output.

## Phase 3: FTP Receiver

- Add `PushProtocol` and `PushReceiverConfig`.
- Add a minimal passive FTP receiver for Nikon upload validation.
- Support `USER`, `PASS`, `PASV`, `EPSV`, `STOR`, folder navigation commands, and basic listing commands.

## Phase 4: CLI

- Add `receiver-config`.
- Add `receive-file`.
- Add `serve-ftp`.
- Add `inbox`.

## Phase 5: Verification

- Run `cargo fmt --all -- --check`.
- Run `cargo clippy --workspace -- -D warnings`.
- Run `cargo test --workspace`.
- Run `cargo build --workspace`.
- Run `scripts/verify.ps1`.

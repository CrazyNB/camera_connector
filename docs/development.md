# Development

## Required Tools

Windows development uses:

- Rust stable through `rustup`
- Visual Studio 2022 Build Tools with the C++ workload
- PowerShell

The Rust MSVC target needs `link.exe`. If normal shells cannot find it, run commands through:

```powershell
C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat
```

## Verify Everything

Run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\verify.ps1
```

The script runs:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `cargo build --workspace`
- Push-mode CLI smoke test:
  - `receiver-config`
  - `receive-file`
  - `inbox`
  - `transfers`

Smoke outputs are written under `target/push-output`.

## Manual FTP Receiver

Start the development FTP receiver:

```powershell
target\debug\camera-connector.exe serve-ftp --bind-host 0.0.0.0 --port 2121 --output C:\Users\hxn\Pictures\CameraConnector --source-name "Studio Camera"
```

Print camera-facing settings without starting the server:

```powershell
target\debug\camera-connector.exe receiver-config --protocol ftp --port 2121 --output C:\Users\hxn\Pictures\CameraConnector --source-name "Studio Camera"
```

Validate local ingest without a camera:

```powershell
target\debug\camera-connector.exe receive-file --input C:\path\to\IMG_1234.CR3 --output C:\Users\hxn\Pictures\CameraConnector --source ftp --source-name "Studio Camera"
```

List the receiver inbox and RAW/JPEG groups:

```powershell
target\debug\camera-connector.exe inbox --path C:\Users\hxn\Pictures\CameraConnector --source ftp
```

List transfer records and filter by source name, original camera path, final filename, remote IP, or transfer id:

```powershell
target\debug\camera-connector.exe transfers --path C:\Users\hxn\Pictures\CameraConnector --source-name "Studio Camera" --original-path DCIM
```

The inbox is intentionally flat. If a camera uploads to `/DCIM/100CANON/IMG_1234.CR3`, the local completed file is `C:\Users\hxn\Pictures\CameraConnector\IMG_1234.CR3`; the original path remains in `transfer-log.jsonl` for filtering.

Duplicate uploads are preserved with numbered filenames such as `IMG_1234 (1).CR3`; existing completed files are not overwritten.


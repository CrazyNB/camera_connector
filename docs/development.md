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
  - `account`
  - `receiver-config`
  - `receive-file`
  - `inbox`
  - `transfers`

Smoke outputs are written under `target/push-output`.

## Manual FTP Receiver

Start the development FTP receiver:

```powershell
target\debug\camera-connector.exe account set --username z5 --password secret --device-name Z5_2 --ip 192.168.137.56
target\debug\camera-connector.exe serve-ftp --bind-host 0.0.0.0 --port 2121 --output C:\Users\hxn\Pictures\CameraConnector
```

Print camera-facing settings without starting the server:

```powershell
target\debug\camera-connector.exe receiver-config --protocol ftp --port 2121 --output C:\Users\hxn\Pictures\CameraConnector
```

List configured camera accounts:

```powershell
target\debug\camera-connector.exe account list
```

List current and recently connected devices from the receiver output folder:

```powershell
target\debug\camera-connector.exe devices --path C:\Users\hxn\Pictures\CameraConnector
```

The `devices` output reads the same account config, so `192.168.137.56` will display as `Z5_2` after the IP is bound to the `z5` account. A newly discovered IP can be attached later:

```powershell
target\debug\camera-connector.exe account bind-ip --username z5 --ip 192.168.137.44
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
target\debug\camera-connector.exe transfers --path C:\Users\hxn\Pictures\CameraConnector --source-name "Z5_2" --original-path DCIM
```

The inbox is intentionally flat. If a camera uploads to `/DCIM/100CANON/IMG_1234.CR3`, the local completed file is `C:\Users\hxn\Pictures\CameraConnector\IMG_1234.CR3`; the original path remains in `transfer-log.jsonl` for filtering. Receiver metadata files such as `transfer-log.jsonl` and `connected-devices.json` are not shown as inbox assets.

Transfer records also expose a virtual display path. With an account device name it looks like `Z5_2/DCIM/100CANON/IMG_1234.CR3`; without a device name the display falls back to the last IP octet, such as `IP-056/DCIM/100CANON/IMG_1234.CR3`. The full IP is still retained in the transfer log for diagnostics.

Default CLI config is stored at `%APPDATA%\CameraConnector\config.json` on Windows. Use `--config C:\path\to\config.json` on `account`, `receiver-config`, `serve-ftp`, `devices`, and `transfers` to test with an alternate config file.

Duplicate uploads are preserved with numbered filenames such as `IMG_1234 (1).CR3`; existing completed files are not overwritten.


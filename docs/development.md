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

Smoke outputs are written under `target/push-output`.

## Manual FTP Receiver

Start the development FTP receiver:

```powershell
target\debug\nikon-importer.exe serve-ftp --bind-host 0.0.0.0 --port 2121 --output C:\Users\hxn\Pictures\NikonImporter
```

Print camera-facing settings without starting the server:

```powershell
target\debug\nikon-importer.exe receiver-config --protocol ftp --port 2121 --output C:\Users\hxn\Pictures\NikonImporter
```

Validate local ingest without a camera:

```powershell
target\debug\nikon-importer.exe receive-file --input C:\path\to\DSC_1234.NEF --output C:\Users\hxn\Pictures\NikonImporter --source ftp
```

List the receiver inbox and RAW/JPEG groups:

```powershell
target\debug\nikon-importer.exe inbox --path C:\Users\hxn\Pictures\NikonImporter --source ftp
```

Duplicate uploads are preserved with numbered filenames such as `DSC_1234 (1).NEF`; existing completed files are not overwritten.

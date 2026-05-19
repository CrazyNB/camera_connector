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
- mock camera CLI smoke test:
  - `scan`
  - `info`
  - `list`
  - `thumb`
  - `pull`

Smoke outputs are written under `target/mock-output`.

## Mock Camera

Manual mock run:

```powershell
target\debug\mock-camera.exe --host 127.0.0.1 --port 15740
```

In another terminal:

```powershell
target\debug\nikon-importer.exe info --host 127.0.0.1 --port 15740
target\debug\nikon-importer.exe list --host 127.0.0.1 --port 15740
```

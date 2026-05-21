# Camera Push Import Protocol Notes

Current technical route:

1. High priority: camera pushes files to a receiver owned by this app.
2. Protocol order: FTP first, then SFTP.
3. AP mode keeps its original meaning, but is paused for now: the camera creates its own Wi-Fi AP and the phone/computer joins it. We are not implementing that path in the current slice.

## FTP Push

The app runs a local FTP receiver. The camera is configured with:

- Host: the phone/computer IP address on the current network.
- Port: default `2121` for development; production may use `21` when platform permissions allow it.
- Mode: passive FTP.
- Username/password: configured through camera accounts. Each account has a login username, optional password, and display device name. Password handling is owned by the core account model; persisted config stores an Argon2id password hash, not the plaintext camera password.
- Destination: local import folder managed by the app.

The camera may still send `CWD`, `MKD`, or `STOR` paths such as `/DCIM/100CANON/IMG_1001.CR3`. The receiver accepts those paths for protocol compatibility, but it does not mirror them locally. Completed files are published into one flat output folder as `IMG_1001.CR3`; the original remote path is kept in the transfer log for filtering and diagnostics.

Minimum FTP commands supported by the core receiver:

- `USER`
- `PASS`
- `SYST`
- `FEAT`
- `PWD`
- `CWD`
- `CDUP`
- `MKD`
- `TYPE`
- `PASV`
- `EPSV`
- `LIST`
- `NLST`
- `SIZE`
- `MDTM`
- `STOR`
- `QUIT`

`STOR` writes bytes to a temporary file first, then atomically publishes the final file. The transfer record is appended after the completed file is published.

## SFTP Push

The core receiver can run an SSH/SFTP endpoint and accept password-authenticated uploads. Real-camera SFTP compatibility still needs field validation.

Implemented behavior:

- SSH server endpoint owned by the app.
- Password authentication through the same camera account model used by FTP.
- Same flat local storage sink as FTP.
- Same asset grouping and duplicate handling.
- Same transfer log fields as FTP, with `protocol` set to `sftp`.
- Same runtime status model as FTP.

Not yet implemented:

- Public-key authentication.
- Streaming large uploads directly to temporary files; the current SFTP path is suitable for compatibility validation and will be moved to streaming storage before large-file field use.

## Storage Rules

- Never trust uploaded paths.
- Remove traversal segments such as `..`.
- Use only the final remote filename for local storage.
- Do not create local folders from camera-side upload paths.
- Replace Windows-unsafe filename characters in the final filename.
- Keep RAW/JPEG/video files intact.
- Recognize common RAW extensions across camera brands: `NEF`, `NRW`, `CR2`, `CR3`, `ARW`, `SRF`, `SR2`, `RAF`, `RW2`, `RWL`, `ORF`, `PEF`, and `DNG`.
- Group files by normalized filename stem, such as `IMG_1001.JPG` and `IMG_1001.CR3`.
- Preserve duplicates with numbered filenames instead of overwriting previous completed files.

## Transfer Log

The receiver writes `transfer-log.jsonl` in the output folder. Each completed transfer records:

- `transfer_id`: stable enough to reference a single transfer.
- `protocol`: FTP, SFTP, or manual validation source.
- `original_path`: remote path sent by the camera, such as `DCIM/100CANON/IMG_1001.CR3`.
- `final_filename` and `final_path`: flattened local result.
- `size_bytes`.
- `remote_addr`: camera IP when available.
- `source_name`: optional user-set camera/source label.
- `started_at_ms` and `completed_at_ms`.

The product uses this log for tag-style grouping and filters. Source name, original path, remote address, transfer id, and final filename are metadata only; they do not create local subfolders.

For display, the UI builds a virtual path from metadata:

- With a camera account device name: `Z5_2/BB/DSC_2552.NEF`.
- Without a source name: `IP-056/BB/DSC_2552.NEF`, using the last IPv4 octet from `192.168.137.56`.

This virtual path is not a filesystem path. It is a compact grouping label; the local file remains flat, and the full `remote_addr` remains available in the log.

Camera accounts are user configuration. They map FTP login credentials to a device name. The receiver authenticates `USER`/`PASS` against this table and applies the account device name to new connection and transfer records. Password hashing and verification live in the core receiver/account layer so CLI, UI, and future app shells share the same credential behavior. IP addresses are observed per connection and transfer; they are not persisted as account identity.

## Connected Devices

The receiver writes `connected-devices.json` in the output folder. It records current and recently seen FTP control connections:

- `remote_addr` and last remote port.
- authenticated FTP `username` when the device has logged in.
- `online` and active connection count.
- first seen, last seen, and last disconnected timestamps.
- source name resolved from the authenticated account when available.

This file powers the "connected devices" view and shows the latest IP used by each login. It is receiver metadata, not an inbox asset.

When the FTP receiver starts, it marks any previously online device records as offline. A fresh process cannot know whether old control connections still exist, so the device view only returns to online after the camera opens a new connection.

## Receiver Runtime Status

The receiver writes `receiver-status.json` in the output folder. It records:

- `phase`: stopped, starting, running, stopping, or failed.
- `protocol`: FTP or SFTP when known.
- `auth_mode`: anonymous or accounts.
- `local_addr`: the socket address that accepted camera connections.
- `output_dir`.
- `account_count`.
- `message`: failure or diagnostic text.

The status file is receiver metadata, not an inbox asset. Inbox scans must ignore it, just like `transfer-log.jsonl` and `connected-devices.json`.

Status readers should treat a stale `Running` file as stopped when the recorded listener is no longer reachable. This covers force-quit, crash, development smoke tests, and OS-level process termination where the receiver cannot run its normal shutdown path.

## Real Camera Verification

For each camera, record:

- Camera model and firmware.
- Network mode: phone hotspot, LAN, or camera AP.
- Protocol: FTP or SFTP.
- Passive mode support.
- Successful `STOR` upload.
- RAW/JPEG/video behavior.
- Any path or folder command quirks.


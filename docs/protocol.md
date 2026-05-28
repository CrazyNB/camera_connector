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
- Destination: app-managed import location. Desktop builds use a local folder; mobile builds may use a media-library, document-provider, or app-sandbox location instead of a stable filesystem path.

The camera may still send `CWD`, `MKD`, or `STOR` paths such as `/DCIM/100CANON/IMG_1001.CR3`. The receiver accepts those paths for protocol compatibility, but it does not mirror them locally. Completed files are published into one flat import location as `IMG_1001.CR3`; the original remote path is kept in the transfer log for filtering and diagnostics.

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

`STOR` writes bytes to the app state staging area first, then publishes the completed staged file into the final object store. The transfer record and SQLite asset index are updated after the completed file is published.

## SFTP Push

The core receiver can run an SSH/SFTP endpoint and accept password-authenticated uploads. The core path is implemented; real-camera SFTP compatibility still needs field validation per vendor and firmware.

Implemented behavior:

- SSH server endpoint owned by the app.
- Password authentication through the same camera account model used by FTP.
- Same flat storage backend contract as FTP.
- Same asset grouping and duplicate handling.
- Same transfer log fields as FTP, with `protocol` set to `sftp`.
- Same connected-device metadata as FTP, including latest IP, login username, account device name, online state, and disconnect state.
- Same runtime status model as FTP.
- Streaming large uploads to app state staging files before final object publish.

Not yet implemented:

- Public-key authentication.
- Real-camera SFTP compatibility matrix entries.

## Storage Rules

- System config, receiver state/logs, and uploaded assets are separate:
  - Config: receiver and product settings, stored in the app config path such as `%APPDATA%/CameraConnector/config.json`.
  - State/log directory: `camera-connector.sqlite3`, `transfer-log.jsonl`, `sftp-host-key`, and the `staging` directory.
  - Output/inbox location: completed camera files only.
- The core records final save targets as `StoredObjectLocation`, not only as local filesystem paths:
  - Desktop: `local_path`.
  - Android: `media_uri` or `document_uri` through MediaStore/SAF.
  - iOS: `document_uri` or `photo_asset` through Files/Photos APIs.
- Receiver implementations write through `LocalStagingStore` first. The current desktop final object backend is `LocalFolderObjectStore`; mobile shells should provide SAF, MediaStore, Files, or Photos object stores while preserving the same staged-write then publish contract.
- Publish workers must claim a pending `publish_queue` item before writing to final storage; claiming moves `staged` items, or `failed` items whose `next_attempt_at_ms` is due, to `publishing` so retry workers do not publish the same staged bytes twice or hammer revoked storage permissions.
- App shells may clear `next_attempt_at_ms` for failed rows in the active project after the user explicitly retries or reauthorizes storage; the next worker poll can then claim those rows immediately.
- Every completed upload is indexed under a project in SQLite. When no explicit active project is provided by the app shell, the core uses the system `Inbox` project, which remains non-archivable and non-renamable so fallback imports cannot be stranded. Dashboard and asset review views must name a project; audit-log diagnostics are separate from project views.
- Never trust uploaded paths.
- Remove traversal segments such as `..`.
- Use only the final remote filename for local storage.
- Do not create local folders from camera-side upload paths.
- Replace Windows-unsafe filename characters in the final filename.
- Keep RAW/JPEG/video files intact.
- Recognize common RAW extensions across camera brands: `NEF`, `NRW`, `CR2`, `CR3`, `ARW`, `SRF`, `SR2`, `RAF`, `RW2`, `RWL`, `ORF`, `PEF`, and `DNG`.
- Group files by normalized filename stem, such as `IMG_1001.JPG` and `IMG_1001.CR3`.
- Preserve duplicates with numbered filenames instead of overwriting previous completed files.
- Mark repeated completed imports from the same account/source and original camera path with `duplicate_index` and `duplicate_count` in asset views.

## Transfer Log

The receiver writes `transfer-log.jsonl` in the state/log directory as an audit stream. SQLite is the durable state and dashboard index. Each completed transfer records:

- `transfer_id`: stable enough to reference a single transfer.
- `protocol`: FTP, SFTP, or manual validation source.
- `original_path`: remote path sent by the camera, such as `DCIM/100CANON/IMG_1001.CR3`.
- `final_filename`: flattened result name.
- `final_location`: platform save target, such as `local_path`, `media_uri`, `document_uri`, or `photo_asset`.
- `size_bytes`.
- `username`: authenticated push-login username when available.
- `remote_addr`: camera IP when available.
- `source_name`: optional user-set camera/source label.
- `started_at_ms` and `completed_at_ms`.

The product uses the SQLite `projects`, `transfers`, `assets`, `asset_groups`, `publish_queue`, `receiver_accounts`, `connected_devices`, and `receiver_status` tables for project-scoped dashboard queries and mutable receiver state. Username, source name, original path, remote address, transfer id, and final filename are metadata only; they do not create local subfolders.

For display, the UI builds a virtual path from metadata:

- With a camera account device name resolved from username: `Z5_2/BB/DSC_2552.NEF`.
- Without a source name: `IP-056/BB/DSC_2552.NEF`, using the last IPv4 octet from `192.168.137.56`.

This virtual path is not a filesystem path. It is a compact grouping label; the saved object remains flat in the chosen storage backend, and the full `remote_addr` remains available in the log.

Camera accounts are user configuration. They map push-login credentials to a device name. FTP authenticates `USER`/`PASS`; SFTP authenticates SSH password login. Both protocols use the same account table and write the login username to new connection and transfer records. Display views resolve the username through the current account table first, then fall back to the recorded source name. Password hashing and verification live in the core receiver/account layer so CLI, UI, and future app shells share the same credential behavior. IP addresses are observed per connection and transfer; they are not persisted as account identity.

## Connected Devices

The receiver writes connected-device state to the SQLite `connected_devices` table in the state/log directory. It records current and recently seen FTP control connections and SFTP sessions:

- `remote_addr` and last remote port.
- authenticated login `username` when the device has logged in.
- `online` and active connection count.
- first seen, last seen, and last disconnected timestamps.
- source name resolved from the authenticated account when available.

This table powers the "connected devices" view and shows the latest IP used by each login. It is receiver metadata, not an inbox asset.

When the FTP or SFTP receiver starts, it marks any previously online device records as offline. A fresh process cannot know whether old sessions still exist, so the device view only returns to online after the camera opens a new connection.

## Receiver Runtime Status

The receiver writes runtime status to the SQLite `receiver_status` table in the state/log directory. It records:

- `phase`: stopped, starting, running, stopping, or failed.
- `protocol`: FTP or SFTP when known.
- `auth_mode`: anonymous or accounts.
- `local_addr`: the socket address that accepted camera connections.
- `output_dir`.
- `state_dir`.
- `account_count`.
- `message`: failure or diagnostic text.

The status table is receiver metadata, not an inbox asset. Current receivers write it outside the inbox; inbox scans still ignore known metadata filenames defensively.

Status readers should treat a stale `Running` row as stopped when the recorded listener is no longer reachable. This covers force-quit, crash, development smoke tests, and OS-level process termination where the receiver cannot run its normal shutdown path.

## Real Camera Verification

For each camera, record:

- Camera model and firmware.
- Network mode: phone hotspot, LAN, or camera AP.
- Protocol: FTP or SFTP.
- Passive mode support.
- Successful `STOR` upload.
- RAW/JPEG/video behavior.
- Any path or folder command quirks.


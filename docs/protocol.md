# Camera Push Import Protocol Notes

Current technical route:

1. High priority: camera pushes files to a receiver owned by this app.
2. Protocol order: FTP first, then SFTP, then FTPS.
3. AP mode keeps its original meaning, but is paused for now: the camera creates its own Wi-Fi AP and the phone/computer joins it. We are not implementing that path in the current slice.

## FTP Push

The app runs a local FTP receiver. The camera is configured with:

- Host: the phone/computer IP address on the current network.
- Port: default `2121` for development; production may use `21` when platform permissions allow it.
- Mode: passive FTP.
- Username/password: optional for local validation, configurable for real camera setup.
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

Planned after FTP is proven with a real camera.

Requirements:

- SSH server endpoint owned by the app or companion service.
- Configurable username/password or key material.
- Same flat local storage sink as FTP.
- Same asset grouping and duplicate handling.
- Same transfer log fields as FTP.

## FTPS Push

Planned after FTP is proven and once camera TLS behavior is validated.

Requirements:

- Explicit FTPS preferred if the camera supports it.
- Certificate and trust flow must be simple enough for camera configuration.
- Same flat storage sink as FTP.
- Same transfer log fields as FTP.

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
- `protocol`: FTP, SFTP, FTPS, or manual validation source.
- `original_path`: remote path sent by the camera, such as `DCIM/100CANON/IMG_1001.CR3`.
- `final_filename` and `final_path`: flattened local result.
- `size_bytes`.
- `remote_addr`: camera IP when available.
- `source_name`: optional user-set camera/source label.
- `started_at_ms` and `completed_at_ms`.

The product uses this log for tag-style grouping and filters. Source name, original path, remote address, transfer id, and final filename are metadata only; they do not create local subfolders.

For display, the UI builds a virtual path from metadata:

- With a source alias or configured source name: `Z5_2/BB/DSC_2552.NEF`.
- Without a source name: `IP-056/BB/DSC_2552.NEF`, using the last IPv4 octet from `192.168.137.56`.

This virtual path is not a filesystem path. It is a compact grouping label; the local file remains flat, and the full `remote_addr` remains available in the log.

## Real Camera Verification

For each camera, record:

- Camera model and firmware.
- Network mode: phone hotspot, LAN, or camera AP.
- Protocol: FTP, SFTP, or FTPS.
- Passive mode support.
- Successful `STOR` upload.
- RAW/JPEG/video behavior.
- Any path or folder command quirks.


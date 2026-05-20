# Nikon Push Import Protocol Notes

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

`STOR` writes bytes to a temporary file first, then atomically publishes the final file.

## SFTP Push

Planned after FTP is proven with a real camera.

Requirements:

- SSH server endpoint owned by the app or companion service.
- Configurable username/password or key material.
- Same local storage sink as FTP.
- Same asset grouping and duplicate handling.

## FTPS Push

Planned after FTP is proven and once camera TLS behavior is validated.

Requirements:

- Explicit FTPS preferred if the camera supports it.
- Certificate and trust flow must be simple enough for camera configuration.
- Same storage sink as FTP.

## Storage Rules

- Never trust uploaded paths.
- Remove traversal segments such as `..`.
- Replace Windows-unsafe filename characters.
- Keep RAW/JPEG/video files intact.
- Group files by normalized filename stem, such as `DSC_1234.JPG` and `DSC_1234.NEF`.
- Preserve duplicates with numbered filenames instead of overwriting previous completed files.

## Real Camera Verification

For each camera, record:

- Camera model and firmware.
- Network mode: phone hotspot, LAN, or camera AP.
- Protocol: FTP, SFTP, or FTPS.
- Passive mode support.
- Successful `STOR` upload.
- RAW/JPEG/video behavior.
- Any path or folder command quirks.

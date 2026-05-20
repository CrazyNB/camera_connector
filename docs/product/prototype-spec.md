# Nikon Wireless Importer Prototype Spec

## 1. Prototype Goal

The prototype now presents a push-mode receiver, not a PTP/IP gallery browser. The first screen should help the user start a receiver and copy settings into the Nikon camera FTP profile.

## 2. Primary Flow

```mermaid
flowchart TD
  A["Open app"] --> B["Choose protocol"]
  B --> C["Start receiver"]
  C --> D["Show host, port, user, output folder"]
  D --> E["Configure camera FTP profile"]
  E --> F["Camera uploads files"]
  F --> G["Receiver publishes completed files"]
  G --> H["Inbox groups RAW/JPEG/video assets"]
```

## 3. Screens

### Receiver Screen

Purpose: start and monitor the local receiver.

Content:

- Protocol segmented control: FTP, SFTP, FTPS.
- Host/IP and port.
- Username and password status.
- Output folder.
- Start/stop receiver action.
- Firewall/setup warning when needed.

### Camera Setup Screen

Purpose: tell the user what to enter on the Nikon camera.

Content:

- Server address.
- Port.
- Login mode.
- Destination folder behavior.
- Passive FTP note.
- Test upload checklist.

### Inbox Screen

Purpose: show files already pushed by the camera.

Content:

- Recent received files.
- RAW+JPEG grouping.
- File format, size, received time.
- Failed/interrupted transfer state.
- Duplicate uploads shown as numbered files, not silent overwrites.

### Transfer Screen

Purpose: show live receive progress.

Content:

- Current upload filename.
- Bytes received.
- Completed, receiving, failed, canceled states.
- Retry guidance: retry from the camera.

### Settings Screen

Purpose: configure local receiver behavior.

Content:

- Output folder.
- FTP port.
- Credentials.
- Duplicate policy.
- Compatibility log export.

## 4. Visual Direction

- Keep the interface utilitarian and dense.
- Use status pills for receiver state.
- Use icon buttons for copy, refresh, start, stop, and folder actions.
- Avoid marketing hero layouts.
- The first viewport must show receiver status and camera-facing settings.

## 5. Deferred AP Mode

AP mode keeps the original meaning: camera creates Wi-Fi and the device joins it. Do not show AP as the main flow in the current prototype. Keep it as a disabled or future compatibility item.

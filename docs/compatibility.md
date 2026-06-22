# Camera Push Compatibility

Record every real-camera push test here. Unknown is acceptable until a real smoke test replaces it.

After each test, update only the rows that have direct evidence. If the test
reveals a stable failure mode or setup requirement, also update
`troubleshooting.md`. If it changes receiver/storage/project semantics, update
`protocol.md` and `architecture.md`.

| Vendor | Model | Firmware | Network Mode | FTP Push | SFTP Push | Passive FTP | RAW | JPEG | Video | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Nikon | Z5_2 | 1.11 | Windows hotspot | Pass | Unknown | Pass | Pass | Pass | Pass | FTP receiver `192.168.137.1:2121`; anonymous login; flat output validated with RAW+JPEG batch and MOV upload |
| Canon | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs push test |
| Sony | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs push test |
| Fujifilm | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs push test |
| Panasonic | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs push test |

## Android Physical Device Matrix

Use this table for the current Android milestone: one real phone, one real camera, FTP push, SAF output, foreground receiver, and project-scoped visibility.

| Phone | Android | Network Mode | Camera | FTP Login | Foreground Service | SAF Publish | RAW/JPEG Pair | Project Photos | Transfers | Diagnostics | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs physical-device test |

## Test Record Template

```text
Date:
Tester:
Vendor:
Camera model:
Firmware:
Network mode:
Receiver device:
Receiver IP:
Protocol:
Port:
Authentication:
Passive mode:
Camera folder setting:
Local storage policy:
JPEG upload:
RAW upload:
Video upload:
Large file stability:
Observed FTP commands:
Notes:
```

## Android Physical Device Test Template

```text
Date:
Tester:
Phone model:
Android version:
Camera vendor/model/firmware:
Network mode: phone hotspot / LAN / camera AP
Phone receiver IP:
Camera IP:
Protocol:
Port:
Authentication:
Selected Android output: SAF tree / app-private fallback
Notification permission:
Foreground service: starts / remains visible / stops cleanly
Camera login:
JPEG upload:
RAW upload:
RAW+JPEG grouping:
SAF publish:
Project photos visibility:
Photo detail visibility:
Project photo visibility:
Transfer record visibility:
Publish queue recovery:
Diagnostics path:
Compatibility result:
Notes:
```

## 2026-05-21 Nikon Z5_2 FTP Push Smoke

```text
Date: 2026-05-21
Tester: hxn
Vendor: Nikon
Camera model: Z5_2
Firmware: 1.11
Network mode: Windows hotspot
Receiver device: Windows PC
Receiver IP: 192.168.137.1
Camera IP: 192.168.137.56
Protocol: FTP
Port: 2121
Authentication: anonymous
Passive mode: Pass
Camera folder setting: default/root
Local storage policy: flat output, original path kept in transfer log
JPEG upload: Pass, DSC_2463.JPG, DSC_2465.JPG, DSC_2466.JPG, DSC_2467.JPG
RAW upload: Pass, DSC_2463.NEF, DSC_2465.NEF, DSC_2466.NEF, DSC_2467.NEF
Video upload: Not tested
Large file stability: Not tested beyond four RAW+JPEG pairs
Observed FTP commands: passive FTP upload via receiver implementation
Notes: Files landed in C:\Users\hxn\Pictures\CameraConnector while control connection remained established.
```

## 2026-05-21 Nikon Z5_2 FTP Push Batch And Flat Output

```text
Date: 2026-05-21
Tester: hxn
Vendor: Nikon
Camera model: Z5_2
Firmware: 1.11
Network mode: Windows hotspot
Receiver device: Windows PC
Receiver IP: 192.168.137.1
Camera IP: 192.168.137.56
Protocol: FTP
Port: 2121
Authentication: anonymous
Passive mode: Pass
Camera folder setting: root plus BB
Local storage policy: Pass; 0 local subdirectories created; BB/* original paths recorded in transfer-log.jsonl
JPEG upload: Pass, 86 JPG files in receiver output
RAW upload: Pass, 86 NEF files in receiver output
Video upload: Pass, DSC_2553.MOV, 207,961,609 bytes
Large file stability: Pass for 173 completed transfer records totaling 2,824,918,361 bytes
Observed FTP commands: passive FTP upload via receiver implementation
Notes: Duplicate DSC_2552 RAW+JPEG was preserved as DSC_2552 (1).NEF and DSC_2552 (1).JPG; no .tmp files remained.
```

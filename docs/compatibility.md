# Camera Push Compatibility

Record every real-camera push test here. Unknown is acceptable until a real smoke test replaces it.

| Vendor | Model | Firmware | Network Mode | FTP Push | SFTP Push | FTPS Push | Passive FTP | RAW | JPEG | Video | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Nikon | Z5_2 | 1.11 | Windows hotspot | Pass | Unknown | Unknown | Pass | Pass | Pass | Pass | FTP receiver `192.168.137.1:2121`; anonymous login; flat inbox validated with RAW+JPEG batch and MOV upload |
| Canon | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs push test |
| Sony | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs push test |
| Fujifilm | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs push test |
| Panasonic | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs push test |

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
Local storage policy: flat inbox, original path kept in transfer log
JPEG upload: Pass, DSC_2463.JPG, DSC_2465.JPG, DSC_2466.JPG, DSC_2467.JPG
RAW upload: Pass, DSC_2463.NEF, DSC_2465.NEF, DSC_2466.NEF, DSC_2467.NEF
Video upload: Not tested
Large file stability: Not tested beyond four RAW+JPEG pairs
Observed FTP commands: passive FTP upload via receiver implementation
Notes: Files landed in C:\Users\hxn\Pictures\CameraConnector while control connection remained established.
```

## 2026-05-21 Nikon Z5_2 FTP Push Batch And Flat Inbox

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
JPEG upload: Pass, 86 JPG files in receiver inbox
RAW upload: Pass, 86 NEF files in receiver inbox
Video upload: Pass, DSC_2553.MOV, 207,961,609 bytes
Large file stability: Pass for 173 completed transfer records totaling 2,824,918,361 bytes
Observed FTP commands: passive FTP upload via receiver implementation
Notes: Duplicate DSC_2552 RAW+JPEG was preserved as DSC_2552 (1).NEF and DSC_2552 (1).JPG; no .tmp files remained.
```

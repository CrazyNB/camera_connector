# Nikon Push Compatibility

Record every real-camera push test here. Unknown is acceptable until a real smoke test replaces it.

| Model | Firmware | Network Mode | FTP Push | SFTP Push | FTPS Push | Passive FTP | RAW | JPEG | Video | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Z5_2 | 1.11 | Windows hotspot | Pass | Unknown | Unknown | Pass | Pass | Pass | Unknown | FTP receiver `192.168.137.1:2121`; anonymous login; received four RAW+JPEG pairs |
| Zf | Unknown | Phone hotspot | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs push test |
| Z6II | Unknown | Camera AP | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | AP mode paused |
| D780 | Unknown | LAN | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Unknown | Needs push test |

## Test Record Template

```text
Date:
Tester:
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
JPEG upload:
RAW upload:
Video upload:
Large file stability:
Observed FTP commands:
Notes:
```

## 2026-05-21 Z5_2 FTP Push Smoke

```text
Date: 2026-05-21
Tester: hxn
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
JPEG upload: Pass, DSC_2463.JPG, DSC_2465.JPG, DSC_2466.JPG, DSC_2467.JPG
RAW upload: Pass, DSC_2463.NEF, DSC_2465.NEF, DSC_2466.NEF, DSC_2467.NEF
Video upload: Not tested
Large file stability: Not tested beyond four RAW+JPEG pairs
Observed FTP commands: passive FTP upload via receiver implementation
Notes: Files landed in C:\Users\hxn\Pictures\NikonImporter while control connection remained established.
```

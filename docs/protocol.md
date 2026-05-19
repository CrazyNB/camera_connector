# Nikon PTP/IP Protocol Notes

## Transport

Nikon wireless import uses PTP/IP over TCP. The default port observed in the product plan is:

```text
15740
```

PTP/IP uses two connections:

- Command/Data connection: sends PTP commands and receives data and responses.
- Event connection: receives camera events.

The minimum initialization flow is:

```text
TCP connect camera_host:15740
Send InitCommandRequest
Receive InitCommandAck
Open second TCP connection
Send InitEventRequest
Receive InitEventAck
OpenSession
GetDeviceInfo
```

## Minimum Operations

V0.0 and V0.1 only need:

| Name | Code |
| --- | --- |
| GetDeviceInfo | `0x1001` |
| OpenSession | `0x1002` |
| CloseSession | `0x1003` |
| GetStorageIDs | `0x1004` |
| GetStorageInfo | `0x1005` |
| GetObjectHandles | `0x1007` |
| GetObjectInfo | `0x1008` |
| GetObject | `0x1009` |
| GetThumb | `0x100A` |

## Download Rules

- Original download concurrency defaults to `1`.
- Thumbnail concurrency defaults to `2..4`.
- Downloads write to a temporary file first.
- The final file is published only after the transfer completes and the size is verified when possible.

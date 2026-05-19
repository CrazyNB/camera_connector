# Troubleshooting

## No Camera Found

User-facing copy:

> 没有发现相机。请确认相机已经连接到手机热点或同一个 Wi-Fi，然后重新扫描。

Checks:

- Confirm the camera is awake.
- Confirm Wi-Fi or Connect to PC mode is enabled on the camera.
- Try the last successful IP if available.
- Try manual IP entry.
- Try camera AP mode with `192.168.1.1:15740`.

## Connection Timeout

User-facing copy:

> 连接超时。相机可能已经休眠，或当前网络无法访问相机。

Checks:

- Wake the camera.
- Move the phone closer to the camera.
- Confirm both devices are on the same network.
- Reduce scan scope to the current subnet.

## Local Network Permission Denied

User-facing copy:

> 需要允许访问本地网络，App 才能发现和连接相机。

Checks:

- On iOS, ensure `NSLocalNetworkUsageDescription` is configured.
- Ask the user to enable local network access in system settings.

## Thumbnail Unavailable

User-facing copy:

> 这张照片无法读取缩略图，但仍然可以尝试下载原文件。

Checks:

- Do not block the gallery.
- Show a fallback file tile.
- Allow original file download.

## Download Interrupted

User-facing copy:

> 下载中断。请保持相机开启并靠近手机，然后重试。

Checks:

- Keep original download concurrency at `1`.
- Delete or keep temporary partial files away from the published destination.
- Retry from the beginning until resume support is proven.

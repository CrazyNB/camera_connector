# Troubleshooting

## Camera Cannot Connect To Receiver

User-facing copy:

> 相机没有连上接收服务。请确认本机和相机在同一个网络，并且接收服务正在运行。

Checks:

- Confirm the receiver shows the correct local IP and port.
- Confirm Windows Firewall allows inbound TCP on the chosen FTP port.
- Use port `2121` for development if port `21` is blocked or requires administrator privileges.
- Confirm the camera is using FTP push/upload mode, not PTP/IP Connect to PC.
- Confirm passive FTP is enabled on the camera when available.

## Login Failed

User-facing copy:

> 相机登录失败。请检查相机里保存的 FTP 用户名和密码。

Checks:

- If no password is configured, use anonymous login.
- If credentials are configured, update the camera profile to match the receiver.
- Avoid special characters in the first real-camera test password.

## Upload Starts But File Does Not Appear

User-facing copy:

> 已收到连接，但文件没有完成写入。请保持相机开启，并检查保存目录权限。

Checks:

- Ensure the output folder exists or can be created.
- Ensure the app can write to the output folder.
- Check whether the camera sends nested folders and unusual filenames.
- Confirm the final file is not left as `.tmp`.

## Transfer Interrupted

User-facing copy:

> 传输中断。失败任务不会发布为最终文件，请在相机上重新发送。

Checks:

- Keep receiver running in foreground for early validation.
- Keep phone/computer close to the camera.
- Prefer single-file or small-batch tests before large RAW batches.
- Retry from the beginning until resume support is proven.

## AP Mode

AP mode keeps the original meaning: the camera creates Wi-Fi and the phone/computer joins it. This path is currently paused. Do not mix AP-mode validation with the FTP push receiver milestone unless we explicitly resume it.


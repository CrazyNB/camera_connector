package com.cameraconnector.app.ui

import com.cameraconnector.app.core.DEFAULT_CAMERA_CONNECT_HOST
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.PublishQueueState
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter

private val displayTimeFormatter: DateTimeFormatter =
    DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss")

internal fun formatEpochMillisForDisplay(
    epochMillis: Long,
    zoneId: ZoneId = ZoneId.systemDefault(),
): String = Instant.ofEpochMilli(epochMillis)
    .atZone(zoneId)
    .format(displayTimeFormatter)

internal fun formatEpochMillisTextForDisplay(value: String): String =
    value.toLongOrNull()?.let { formatEpochMillisForDisplay(it) } ?: value

internal fun publishQueueAttentionLabel(state: PublishQueueState): String? = when {
    state.failedCount > 0 -> "\u5199\u5165\u5931\u8d25 ${state.failedCount}"
    state.pendingCount > 0 -> "\u5f85\u5199\u5165 ${state.pendingCount}"
    else -> null
}

internal fun publishQueueRetryActionVisible(state: PublishQueueState): Boolean =
    state.failedCount > 0

internal fun receiverPhaseLabel(value: String): String = when (value) {
    "Starting" -> "\u542f\u52a8\u4e2d"
    "Running" -> "\u8fd0\u884c\u4e2d"
    "Stopping" -> "\u505c\u6b62\u4e2d"
    "Stopped" -> "\u5df2\u505c\u6b62"
    "Failed" -> "\u542f\u52a8\u5931\u8d25"
    "Unknown" -> "未知"
    else -> value
}

internal fun authModeLabel(value: String): String = when (value) {
    "Accounts" -> "账号认证"
    "Open" -> "\u5f00\u653e"
    "Unknown" -> "未知"
    else -> value
}

internal fun transferStatusLabel(value: String): String = when (value) {
    "Completed" -> "\u5df2\u5b8c\u6210"
    "Failed" -> "失败"
    "Pending" -> "\u7b49\u5f85\u4e2d"
    else -> value
}

internal fun normalizeCameraConnectHost(value: String?): String {
    val trimmed = value?.trim().orEmpty()
    return if (
        trimmed.isBlank() ||
        trimmed.equals("null", ignoreCase = true) ||
        trimmed == "0.0.0.0" ||
        trimmed.startsWith("127.")
    ) {
        DEFAULT_CAMERA_CONNECT_HOST
    } else {
        trimmed
    }
}

internal fun formatEndpoint(account: DeviceAccount): String {
    val host = account.latestIp?.takeIf { it.isNotBlank() } ?: "暂无来源"
    val port = account.latestPort?.let { ":$it" } ?: ""
    return "$host$port"
}

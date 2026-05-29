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
    state.failedCount > 0 -> "发布失败 ${state.failedCount}"
    state.pendingCount > 0 -> "待发布 ${state.pendingCount}"
    else -> null
}

internal fun publishQueueRetryActionVisible(state: PublishQueueState): Boolean =
    state.failedCount > 0

internal fun receiverPhaseLabel(value: String): String = when (value) {
    "Running" -> "运行中"
    "Stopped" -> "已停止"
    "Unknown" -> "未知"
    else -> value
}

internal fun authModeLabel(value: String): String = when (value) {
    "Accounts" -> "账号认证"
    "Open" -> "开放"
    "Unknown" -> "未知"
    else -> value
}

internal fun transferStatusLabel(value: String): String = when (value) {
    "Completed" -> "已完成"
    "Failed" -> "失败"
    "Pending" -> "等待中"
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

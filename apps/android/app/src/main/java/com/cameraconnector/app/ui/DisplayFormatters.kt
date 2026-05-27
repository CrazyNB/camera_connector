package com.cameraconnector.app.ui

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

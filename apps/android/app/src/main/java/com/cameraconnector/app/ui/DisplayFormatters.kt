package com.cameraconnector.app.ui

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

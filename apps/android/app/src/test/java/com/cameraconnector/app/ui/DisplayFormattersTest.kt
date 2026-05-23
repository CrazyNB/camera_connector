package com.cameraconnector.app.ui

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.Instant
import java.time.ZoneId

class DisplayFormattersTest {
    @Test
    fun formatsEpochMillisAsLocalChineseDisplayTime() {
        val timestamp = Instant.parse("2026-05-23T01:53:55Z").toEpochMilli()

        val label = formatEpochMillisForDisplay(timestamp, ZoneId.of("Asia/Shanghai"))

        assertEquals("2026-05-23 09:53:55", label)
    }
}

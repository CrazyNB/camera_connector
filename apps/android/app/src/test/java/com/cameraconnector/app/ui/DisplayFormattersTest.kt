package com.cameraconnector.app.ui

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.Instant
import java.time.ZoneId
import com.cameraconnector.app.core.PublishQueueState

class DisplayFormattersTest {
    @Test
    fun formatsEpochMillisAsLocalChineseDisplayTime() {
        val timestamp = Instant.parse("2026-05-23T01:53:55Z").toEpochMilli()

        val label = formatEpochMillisForDisplay(timestamp, ZoneId.of("Asia/Shanghai"))

        assertEquals("2026-05-23 09:53:55", label)
    }

    @Test
    fun publishQueueAttentionLabelPrioritizesFailedPublishes() {
        val label = publishQueueAttentionLabel(
            PublishQueueState(
                pendingCount = 3,
                failedCount = 2,
            ),
        )

        assertEquals("发布失败 2", label)
    }

    @Test
    fun publishQueueAttentionLabelShowsPendingPublishes() {
        val label = publishQueueAttentionLabel(PublishQueueState(pendingCount = 2))

        assertEquals("待发布 2", label)
    }

    @Test
    fun publishQueueAttentionLabelIsEmptyWhenQueueIsSettled() {
        val label = publishQueueAttentionLabel(PublishQueueState(completedCount = 4))

        assertEquals(null, label)
    }

    @Test
    fun publishQueueRetryActionOnlyShowsForFailedPublishes() {
        assertEquals(false, publishQueueRetryActionVisible(PublishQueueState(pendingCount = 2)))
        assertEquals(false, publishQueueRetryActionVisible(PublishQueueState(completedCount = 4)))
        assertEquals(true, publishQueueRetryActionVisible(PublishQueueState(failedCount = 1)))
    }
}

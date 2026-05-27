package com.cameraconnector.app.core

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Test

class NativeDashboardMappingTest {
    @Test
    fun publishQueueStateMapsNativeDashboardCounts() {
        val state = mapPublishQueueState(
            JSONObject()
                .put("total_count", 5)
                .put("pending_count", 3)
                .put("staged_count", 1)
                .put("publishing_count", 1)
                .put("completed_count", 2)
                .put("failed_count", 1),
        )

        assertEquals(5, state.totalCount)
        assertEquals(3, state.pendingCount)
        assertEquals(1, state.stagedCount)
        assertEquals(1, state.publishingCount)
        assertEquals(2, state.completedCount)
        assertEquals(1, state.failedCount)
    }

    @Test
    fun publishQueueStateDefaultsMissingCountsToZero() {
        val state = mapPublishQueueState(null)

        assertEquals(PublishQueueState(), state)
    }
}

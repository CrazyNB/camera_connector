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

    @Test
    fun recentPublishFailuresMapToTransferRowsWithErrors() {
        val rows = mapPublishFailureTransfers(
            org.json.JSONArray()
                .put(
                    JSONObject()
                        .put("queue_id", "publish-1")
                        .put("transfer_id", "ftp:1:IMG_1100.JPG")
                        .put("final_filename", "IMG_1100.JPG")
                        .put("attempt_count", 2)
                        .put("last_error", "SAF permission revoked"),
                ),
        )

        assertEquals(1, rows.size)
        assertEquals("publish-1", rows[0].id)
        assertEquals("Failed", rows[0].status)
        assertEquals("IMG_1100.JPG", rows[0].displayPath)
        assertEquals("SAF permission revoked", rows[0].message)
    }
}

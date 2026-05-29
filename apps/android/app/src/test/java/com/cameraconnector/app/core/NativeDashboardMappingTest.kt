package com.cameraconnector.app.core

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class NativeDashboardMappingTest {
    @Test
    fun projectSummaryMapsCoreLifecycleCapabilities() {
        val project = mapProjectSummary(
            JSONObject()
                .put("project_id", "project-client")
                .put("name", "Client Shoot")
                .put("slug", "client-shoot")
                .put("status", "Active")
                .put("kind", "User")
                .put(
                    "capabilities",
                    JSONObject()
                        .put("can_be_active_project", true)
                        .put("can_archive", true)
                        .put("can_rename", true)
                        .put("can_restore", false)
                        .put("can_accept_moved_groups", true),
                ),
        )

        assertEquals("project-client", project.id)
        assertTrue(project.canBeActiveProject)
        assertTrue(project.canArchive)
        assertTrue(project.canRename)
        assertFalse(project.canRestore)
        assertTrue(project.canAcceptMovedGroups)
    }

    @Test
    fun inboxQueryJsonIncludesOnlyCoreFilters() {
        val json = assetGroupQueryJson(
            InboxAssetQuery(
                username = "camera01",
                sourceName = "Studio Z5",
                originalPath = "DCIM/100",
                role = InboxAssetRole.Raw,
            ),
        )

        assertEquals("camera01", json.getString("username"))
        assertEquals("Studio Z5", json.getString("source_name"))
        assertEquals("DCIM/100", json.getString("original_path"))
        assertEquals("raw", json.getString("role"))
        assertFalse(json.has("remote_addr"))
    }

    @Test
    fun inboxAssetsMapGroupPresenceFromNativeDashboard() {
        val assets = mapInboxAssets(
            JSONObject()
                .put(
                    "groups",
                    org.json.JSONArray()
                        .put(
                            JSONObject()
                                .put("group_id", "group-1")
                                .put("group_key", "IMG_1001")
                                .put(
                                    "primary",
                                    JSONObject()
                                        .put("id", "asset-jpg")
                                        .put("filename", "IMG_1001.JPG")
                                        .put("format", "Jpeg")
                                        .put("received_time_ms", 10)
                                        .put("size_bytes", 42),
                                )
                                .put(
                                    "raw",
                                    JSONObject()
                                        .put("id", "asset-raw")
                                        .put("filename", "IMG_1001.NEF")
                                        .put("format", "Nef"),
                                ),
                        ),
                ),
        )

        assertEquals(1, assets.size)
        assertEquals("group-1", assets[0].id)
        assertTrue(assets[0].hasJpeg)
        assertTrue(assets[0].hasRaw)
        assertFalse(assets[0].hasVideo)
    }

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
    fun dashboardOutputLabelFallsBackToSavedReceiverOutputDirectory() {
        val label = dashboardOutputLabel(
            paths = JSONObject().put("output_dir", JSONObject.NULL),
            receiverSettings = JSONObject().put(
                "output_dir",
                "/data/user/0/com.cameraconnector.app/files/inbox",
            ),
        )

        assertEquals("/data/user/0/com.cameraconnector.app/files/inbox", label)
    }

    @Test
    fun dashboardOutputLabelDoesNotRenderJsonNullLiteral() {
        val label = dashboardOutputLabel(
            paths = JSONObject().put("output_dir", JSONObject.NULL),
            receiverSettings = JSONObject().put("output_dir", JSONObject.NULL),
        )

        assertEquals("应用私有目录", label)
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

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
                reviewQueue = "unconfirmed_best",
                strategyProfileId = "portrait",
            ),
        )

        assertEquals("camera01", json.getString("username"))
        assertEquals("Studio Z5", json.getString("source_name"))
        assertEquals("DCIM/100", json.getString("original_path"))
        assertEquals("raw", json.getString("role"))
        assertEquals("unconfirmed_best", json.getString("review_queue"))
        assertEquals("portrait", json.getString("strategy_profile_id"))
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
    fun inboxAssetsMapBurstBestScoreFromNativeDashboard() {
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
                                        .put("received_time_ms", 10),
                                )
                                .put(
                                    "burst",
                                    JSONObject()
                                        .put("burst_group_id", "burst-1")
                                        .put("member_count", 3)
                                        .put("member_rank", 2)
                                        .put("recommendation_status", "ready")
                                        .put("best_asset_group_id", "group-best")
                                        .put("best_score", 0.93),
                                ),
                        ),
                ),
        )

        assertEquals(0.93, assets[0].burst?.bestScore ?: 0.0, 0.0001)
    }

    @Test
    fun inboxAssetsMapQualitySignalScoresFromNativeDashboard() {
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
                                        .put("received_time_ms", 10),
                                )
                                .put(
                                    "quality",
                                    JSONObject()
                                        .put("overall", 0.82)
                                        .put("analysis_status", "ready")
                                        .put("scorer_version", "local-v1")
                                        .put("primary_reason", "balanced")
                                        .put("analyzed_at_ms", 1234)
                                        .put("sharpness", 0.73)
                                        .put("exposure", 0.66)
                                        .put("highlight_clipping_penalty", 0.08)
                                        .put("shadow_clipping_penalty", 0.12)
                                        .put("composition", 0.58)
                                        .put("composition_confidence", 0.71),
                                ),
                        ),
                ),
        )

        val quality = requireNotNull(assets[0].quality)
        assertEquals(0.82, quality.overall ?: 0.0, 0.0001)
        assertEquals(0.73, quality.sharpness ?: 0.0, 0.0001)
        assertEquals(0.66, quality.exposure ?: 0.0, 0.0001)
        assertEquals(0.08, quality.highlightClippingPenalty ?: 0.0, 0.0001)
        assertEquals(0.12, quality.shadowClippingPenalty ?: 0.0, 0.0001)
        assertEquals(0.58, quality.composition ?: 0.0, 0.0001)
        assertEquals(0.71, quality.compositionConfidence ?: 0.0, 0.0001)
    }

    @Test
    fun strategyProfilesMapWeightsAndThresholds() {
        val profiles = mapStrategyProfiles(
            org.json.JSONArray()
                .put(
                    JSONObject()
                        .put("profile_id", "general")
                        .put("name", "General")
                        .put("built_in", true)
                        .put("strategy_version", "strategy-v1")
                        .put("burst_window_ms", 1200)
                        .put("min_group_size", 2)
                        .put(
                            "weights",
                            JSONObject()
                                .put("sharpness", 0.4)
                                .put("exposure", 0.22)
                                .put("composition", 0.12)
                                .put("highlight_clipping_penalty", -0.14)
                                .put("shadow_clipping_penalty", -0.08)
                                .put("diversity", 0.04),
                        )
                        .put("reject_if_sharpness_below", 0.25)
                        .put("flag_if_overall_below", 0.4)
                        .put("near_duplicate_similarity_above", 0.92)
                        .put("auto_hide_low_score", false)
                        .put("llm_enabled", false),
                ),
        )

        assertEquals(1, profiles.size)
        assertEquals("general", profiles[0].profileId)
        assertTrue(profiles[0].builtIn)
        assertEquals(0.4, profiles[0].weights.sharpness, 0.0001)
        assertEquals(-0.14, profiles[0].weights.highlightClippingPenalty, 0.0001)
        assertEquals(0.25, profiles[0].rejectIfSharpnessBelow, 0.0001)
    }

    @Test
    fun strategyProfileSerializesForCustomSave() {
        val json = StrategyProfileUi(
            profileId = "custom-balanced",
            name = "Custom Balanced",
            builtIn = false,
            strategyVersion = "strategy-v1",
            burstWindowMs = 1200,
            minGroupSize = 2,
            weights = StrategyWeightsUi(
                sharpness = 0.42,
                exposure = 0.22,
                composition = 0.10,
                highlightClippingPenalty = -0.14,
                shadowClippingPenalty = -0.08,
                diversity = 0.04,
            ),
            rejectIfSharpnessBelow = 0.25,
            flagIfOverallBelow = 0.4,
            nearDuplicateSimilarityAbove = 0.92,
            autoHideLowScore = true,
            llmEnabled = false,
        ).toStrategyProfileJson()

        assertEquals("custom-balanced", json.getString("profile_id"))
        assertFalse(json.getBoolean("built_in"))
        assertEquals(0.10, json.getJSONObject("weights").getDouble("composition"), 0.0001)
        assertTrue(json.getBoolean("auto_hide_low_score"))
    }

    @Test
    fun reviewQueueSummaryMapsQueueCounts() {
        val summary = mapReviewQueueSummary(
            JSONObject()
                .put("project_id", "project-client")
                .put("strategy_profile_id", "general")
                .put("total_units", 12)
                .put("pending_count", 2)
                .put("unconfirmed_best_count", 5)
                .put("needs_review_count", 3)
                .put("low_score_candidate_count", 4)
                .put("near_duplicate_count", 1)
                .put("unsupported_count", 2)
                .put("user_overridden_count", 1)
                .put(
                    "queues",
                    org.json.JSONArray()
                        .put(JSONObject().put("queue", "unconfirmed_best").put("count", 5))
                        .put(JSONObject().put("queue", "unsupported").put("count", 2)),
                ),
        )

        assertEquals("project-client", summary.projectId)
        assertEquals(12, summary.totalUnits)
        assertEquals(5, summary.unconfirmedBestCount)
        assertEquals(2, summary.queueCount("unsupported"))
        assertEquals(0, summary.queueCount("missing"))
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

package com.cameraconnector.app.core

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
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
                favorite = true,
                marked = false,
                reviewQueue = "unconfirmed_best",
                strategyProfileId = "portrait",
            ),
        )

        assertEquals("camera01", json.getString("username"))
        assertEquals("Studio Z5", json.getString("source_name"))
        assertEquals("DCIM/100", json.getString("original_path"))
        assertEquals("raw", json.getString("role"))
        assertEquals(true, json.getBoolean("favorite"))
        assertEquals(false, json.getBoolean("marked"))
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
    fun inboxAssetsMapUserMarksFromNativeDashboard() {
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
                                    "user_marks",
                                    JSONObject()
                                        .put("favorite", true)
                                        .put("marked", true),
                                )
                                .put(
                                    "primary",
                                    JSONObject()
                                        .put("id", "asset-jpg")
                                        .put("filename", "IMG_1001.JPG")
                                        .put("format", "Jpeg")
                                        .put("received_time_ms", 10),
                                ),
                        ),
                ),
        )

        assertTrue(assets[0].userMarks.favorite)
        assertTrue(assets[0].userMarks.marked)
    }

    @Test
    fun inboxAssetsMapModelEvaluationAndTechnicalGateFromNativeDashboard() {
        val assets = mapInboxAssets(
            JSONObject()
                .put(
                    "groups",
                    org.json.JSONArray()
                        .put(
                            JSONObject()
                                .put("group_id", "group-1")
                                .put("group_key", "IMG_1001")
                                .put("technical_gate_status", "warn")
                                .put(
                                    "technical_defects",
                                    org.json.JSONArray()
                                        .put(
                                            JSONObject()
                                                .put("defect_type", "blur")
                                                .put("severity", "high")
                                                .put("confidence", 0.72)
                                                .put("reason", "soft detail risk"),
                                        ),
                                )
                                .put("model_status", "ready")
                                .put("model_score", 74)
                                .put("model_tier", "good")
                                .put("model_evaluator_kind", "local_stub")
                                .put("model_summary", "strong subject moment")
                                .put("is_model_select", true)
                                .put("is_favorite", true)
                                .put("is_flagged", true)
                                .put(
                                    "primary",
                                    JSONObject()
                                        .put("id", "asset-jpg")
                                        .put("filename", "IMG_1001.JPG")
                                        .put("format", "Jpeg")
                                        .put("received_time_ms", 10),
                                ),
                        ),
                ),
        )

        val asset = assets.single()
        assertEquals("warn", asset.technicalGateStatus)
        assertEquals("blur", asset.technicalDefects.single().defectType)
        assertEquals(74, asset.modelScore)
        assertEquals("good", asset.modelTier)
        assertEquals("local_stub", asset.modelEvaluatorKind)
        assertEquals("strong subject moment", asset.modelSummary)
        assertTrue(asset.isModelSelect)
        assertTrue(asset.userMarks.favorite)
        assertTrue(asset.userMarks.marked)
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
    fun modelProviderSettingsMapWithoutSecretFields() {
        val settings = mapModelProviderSettings(
            JSONObject()
                .put("provider_kind", "openai")
                .put("provider_label", "OpenAI")
                .put("base_url", "https://api.openai.com/v1")
                .put("default_model", "gpt-4.1-mini")
                .put("default_max_image_side", 1536)
                .put("default_send_mode", "preview_only")
                .put("default_batch_size", 4)
                .put("configured", true)
                .put("api_key_configured", true)
                .put("key_alias", "android-keystore:camera-model")
                .put("api_key", "must-not-map"),
        )

        assertEquals("openai", settings.providerKind)
        assertEquals("OpenAI", settings.providerLabel)
        assertEquals("https://api.openai.com/v1", settings.baseUrl)
        assertEquals("gpt-4.1-mini", settings.defaultModel)
        assertEquals(1536, settings.defaultMaxImageSide)
        assertEquals("preview_only", settings.defaultSendMode)
        assertEquals(4, settings.defaultBatchSize)
        assertTrue(settings.configured)
        assertTrue(settings.apiKeyConfigured)
        assertNull(settings.apiKey)
        assertEquals("android-keystore:camera-model", settings.keyAlias)
    }

    @Test
    fun projectEvaluationSettingsMapDefaultsModelEvaluationOff() {
        val settings = mapProjectEvaluationSettings(JSONObject().put("project_id", "project-client"))

        assertEquals("project-client", settings.projectId)
        assertFalse(settings.modelEvaluationEnabled)
        assertFalse(settings.autoEvaluateOnUpload)
        assertTrue(settings.autoBurstRecommendationEnabled)
        assertEquals("manual", settings.projectRecommendationMode)
        assertEquals("general", settings.sceneProfile)
        assertEquals("standard", settings.cvPolicy)
        assertFalse(settings.allowRiskyModelSelects)
    }

    @Test
    fun projectEvaluationSettingsSerializeManualRecommendationMode() {
        val json = ProjectEvaluationSettingsUi(
            projectId = "project-client",
            modelEvaluationEnabled = true,
            autoEvaluateOnUpload = true,
            autoBurstRecommendationEnabled = false,
            projectRecommendationMode = "automatic",
            promptProfileId = "prompt-portrait",
            sceneProfile = "portrait",
            cvPolicy = "strict",
            allowRiskyModelSelects = true,
            maxImageSide = 1024,
            batchSize = 2,
        ).toProjectEvaluationSettingsJson()

        assertEquals("manual", json.getString("project_recommendation_mode"))
        assertEquals("prompt-portrait", json.getString("prompt_profile_id"))
        assertEquals(1024, json.getInt("max_image_side"))
        assertEquals(2, json.getInt("batch_size"))
    }

    @Test
    fun providerBatchSizeOneRoundTripsWithoutPromotion() {
        val settings = ModelProviderSettingsUi(
            providerKind = "openai",
            baseUrl = "https://api.openai.com/v1",
            defaultBatchSize = 1,
            configured = true,
            apiKey = "sk-test",
        )

        val json = settings.toModelProviderSettingsJson()
        val mapped = mapModelProviderSettings(json)

        assertEquals("https://api.openai.com/v1", json.getString("base_url"))
        assertEquals("sk-test", json.getString("api_key"))
        assertEquals(1, json.getInt("default_batch_size"))
        assertEquals(1, mapped.defaultBatchSize)
        assertNull(mapped.apiKey)
    }

    @Test
    fun nullableStringsTreatJsonNullAndLiteralNullAsMissing() {
        val assets = mapInboxAssets(
            JSONObject()
                .put(
                    "groups",
                    org.json.JSONArray()
                        .put(
                            JSONObject()
                                .put("group_id", "group-1")
                                .put("group_key", "IMG_1001")
                                .put("technical_gate_status", JSONObject.NULL)
                                .put("model_summary", " null ")
                                .put(
                                    "primary",
                                    JSONObject()
                                        .put("id", "asset-jpg")
                                        .put("filename", "IMG_1001.JPG")
                                        .put("format", "Jpeg")
                                        .put("received_time_ms", 10),
                                ),
                        ),
                ),
        )

        assertNull(assets.single().technicalGateStatus)
        assertNull(assets.single().modelSummary)
    }

    @Test
    fun promptProfilesMapStyleTags() {
        val profiles = mapPromptProfiles(
            org.json.JSONArray()
                .put(
                    JSONObject()
                        .put("prompt_profile_id", "portrait-conservative")
                        .put("scope", "global")
                        .put("project_id", JSONObject.NULL)
                        .put("name", "Portrait Conservative")
                        .put("style_tags", org.json.JSONArray().put("portrait").put("conservative"))
                        .put("scene_profile", "portrait")
                        .put("active_version_id", "version-1")
                        .put("built_in", true)
                        .put("enabled", true),
                ),
        )

        assertEquals(listOf("portrait", "conservative"), profiles.single().styleTags)
    }

    @Test
    fun evaluationRunMapsLatestProjectRecommendationStatus() {
        val run = mapEvaluationRun(
            JSONObject()
                .put("run_id", "run-1")
                .put("project_id", "project-client")
                .put("run_type", "project_recommendation")
                .put("trigger", "manual")
                .put("status", "ready")
                .put("provider_kind", "openai")
                .put("provider_model", "gpt-4.1-mini")
                .put("created_at_ms", 10),
        )

        assertEquals("run-1", run.runId)
        assertEquals("project_recommendation", run.runType)
        assertEquals("manual", run.trigger)
        assertEquals("ready", run.status)
    }

    @Test
    fun generateProjectRecommendationReturnsLatestRunInsteadOfRecommendationBody() {
        var generateCalls = 0
        val run = projectRecommendationRunAfterGenerate(
            generateRecommendation = {
                generateCalls += 1
                JSONObject()
                    .put("recommendation_id", "recommendation-1")
                    .put("scope", "Project")
                    .put("project_id", "project-client")
                    .put("run_id", "run-project-1")
            },
            latestRun = {
                JSONObject()
                    .put("run_id", "run-project-1")
                    .put("project_id", "project-client")
                    .put("run_type", "project_recommendation")
                    .put("trigger", "manual")
                    .put("status", "ready")
                    .put("provider_kind", "openai")
                    .put("provider_model", "gpt-5.5")
                    .put("created_at_ms", 12_345)
            },
        )

        assertEquals(1, generateCalls)
        assertEquals("project_recommendation", run.runType)
        assertEquals("manual", run.trigger)
        assertEquals("openai", run.providerKind)
        assertEquals("gpt-5.5", run.providerModel)
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

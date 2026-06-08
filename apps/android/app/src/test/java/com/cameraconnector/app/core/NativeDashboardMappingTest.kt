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
    fun assetQueryJsonIncludesOnlyCoreFilters() {
        val json = assetGroupQueryJson(
            ProjectAssetQuery(
                role = ProjectAssetRole.Raw,
                favorite = true,
                marked = false,
                collection = "model_selects",
            ),
        )

        assertEquals("raw", json.getString("role"))
        assertEquals(true, json.getBoolean("favorite"))
        assertEquals(false, json.getBoolean("marked"))
        assertEquals("model_selects", json.getString("collection"))
        assertFalse(json.has("username"))
        assertFalse(json.has("source_name"))
        assertFalse(json.has("original_path"))
        assertFalse(json.has("remote_addr"))
    }

    @Test
    fun projectAssetsMapGroupPresenceFromNativeDashboard() {
        val assets = mapProjectAssets(
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
    fun projectAssetsMapUserMarksFromNativeDashboard() {
        val assets = mapProjectAssets(
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
    fun projectAssetsMapModelEvaluationAndTechnicalGateFromNativeDashboard() {
        val assets = mapProjectAssets(
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
    fun projectAssetsMapBurstBestScoreFromNativeDashboard() {
        val assets = mapProjectAssets(
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
    fun projectEvaluationSettingsMapDefaultsConcreteActionsOff() {
        val settings = mapProjectEvaluationSettings(JSONObject().put("project_id", "project-client"))

        assertEquals("project-client", settings.projectId)
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
    fun projectEvaluationSettingsMapAndSerializeCvPolicyOverrides() {
        val overridesJson = JSONObject()
            .put("blur_severe_edge_threshold", 0.04)
            .put("blur_severe_frequency_threshold", 0.04)
            .put("blur_high_edge_threshold", 0.12)
            .put("blur_high_frequency_threshold", 0.12)
            .put("highlight_clip_threshold", 245)
            .put("shadow_clip_threshold", 10)
            .put("clipping_high_ratio", 0.08)
            .put("clipping_high_connected_ratio", 0.08)
            .put("clipping_severe_ratio", 0.50)
            .put("clipping_severe_connected_ratio", 0.50)
            .put("color_cast_high_threshold", 0.42)
            .put("color_cast_severe_threshold", 0.70)
            .put("face_eye_open_warn_threshold", 0.35)
            .put("face_exposure_warn_ratio", 0.25)
            .put("face_color_cast_warn_threshold", 0.42)
        val settings = mapProjectEvaluationSettings(
            JSONObject()
                .put("project_id", "project-client")
                .put("cv_policy_overrides", overridesJson),
        )

        assertEquals(0.08, settings.cvPolicyOverrides?.clippingHighRatio)
        assertEquals(245, settings.cvPolicyOverrides?.highlightClipThreshold)
        assertEquals(0.42, settings.cvPolicyOverrides?.colorCastHighThreshold)
        assertEquals(0.35, settings.cvPolicyOverrides?.faceEyeOpenWarnThreshold)
        assertEquals(0.25, settings.cvPolicyOverrides?.faceExposureWarnRatio)
        assertEquals(0.42, settings.cvPolicyOverrides?.faceColorCastWarnThreshold)

        val serialized = settings.toProjectEvaluationSettingsJson()
            .getJSONObject("cv_policy_overrides")
        assertEquals(0.08, serialized.getDouble("clipping_high_ratio"), 0.0001)
        assertEquals(245, serialized.getInt("highlight_clip_threshold"))
        assertEquals(0.42, serialized.getDouble("color_cast_high_threshold"), 0.0001)
        assertEquals(0.35, serialized.getDouble("face_eye_open_warn_threshold"), 0.0001)
        assertEquals(0.25, serialized.getDouble("face_exposure_warn_ratio"), 0.0001)
        assertEquals(0.42, serialized.getDouble("face_color_cast_warn_threshold"), 0.0001)
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
        val assets = mapProjectAssets(
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
    fun promptProfilesMapStructuredPromptContent() {
        val profiles = mapPromptProfiles(
            org.json.JSONArray()
                .put(
                    JSONObject()
                        .put("prompt_profile_id", "documentary-custom")
                        .put("scope", "global")
                        .put("project_id", JSONObject.NULL)
                        .put("name", "Documentary Custom")
                        .put("style_tags", org.json.JSONArray().put("documentary"))
                        .put("scene_profile", "general")
                        .put("active_version_id", "version-2")
                        .put("built_in", false)
                        .put("enabled", true)
                        .put(
                            "active_prompt_text",
                            JSONObject()
                                .put("shared_preference", "Prefer quiet documentary emotion.")
                                .put("evaluation_instruction", "Evaluate technical and story value.")
                                .put("burst_selection_instruction", "Pick the decisive frame.")
                                .put("project_selection_instruction", "Build a coherent set.")
                                .toString(),
                        ),
                ),
        )

        val profile = profiles.single()
        assertEquals("Prefer quiet documentary emotion.", profile.activePromptText)
        assertEquals("Prefer quiet documentary emotion.", profile.sharedPreference)
        assertEquals("Evaluate technical and story value.", profile.evaluationInstruction)
        assertEquals("Pick the decisive frame.", profile.burstSelectionInstruction)
        assertEquals("Build a coherent set.", profile.projectSelectionInstruction)
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
                "/data/user/0/com.cameraconnector.app/files/output",
            ),
        )

        assertEquals("/data/user/0/com.cameraconnector.app/files/output", label)
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

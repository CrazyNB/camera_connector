package com.cameraconnector.app.service

import com.cameraconnector.app.core.ProjectAssetQuery
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Test

class SmartSelectionAnalysisWorkerTest {
    @Test
    fun providerMissingDoesNotBlockLocalTechnicalAssessment() {
        val core = FakeSmartSelectionCore(providerConfigured = false)
        val worker = SmartSelectionAnalysisWorker(
            context = null,
            core = core,
            previewSampleLoader = { _, _ -> "{}" },
        )

        val result = worker.drainOnce(maxAssessments = 1)

        assertEquals(1, result.assessedCount)
        assertEquals(listOf("group-1:false"), core.assessedGroups)
        assertEquals(listOf(false), core.drainProviderFlags)
        assertEquals(0, core.projectRecommendationCalls)
    }

    @Test
    fun uploadDrainDoesNotCreateProjectRecommendation() {
        val core = FakeSmartSelectionCore(providerConfigured = true)
        val worker = SmartSelectionAnalysisWorker(
            context = null,
            core = core,
            previewSampleLoader = { _, _ -> "{}" },
        )

        worker.drainOnce(maxAssessments = 1)

        assertEquals(1, core.drainProviderFlags.size)
        assertEquals(0, core.projectRecommendationCalls)
    }

    @Test
    fun pendingAnalysisQueueDrainsEvenWhenNoNewAssessmentRuns() {
        val core = FakeSmartSelectionCore(
            providerConfigured = true,
            assetModelStatus = "ready",
            selectedProviderId = "global",
            providerOptions = readyProviderOptions("global"),
        )
        val worker = SmartSelectionAnalysisWorker(
            context = null,
            core = core,
            previewSampleLoader = { _, _ -> "{}" },
        )

        val result = worker.drainOnce(maxAssessments = 1)

        assertEquals(0, result.assessedCount)
        assertEquals(1, result.recommendedCount)
        assertEquals(emptyList<String>(), core.assessedGroups)
        assertEquals(listOf(true), core.drainProviderFlags)
    }

    @Test
    fun projectSelectedProviderControlsModelWorkFlag() {
        val core = FakeSmartSelectionCore(
            providerConfigured = false,
            selectedProviderId = "photo-eval-model",
            providerOptions = JSONArray()
                .put(
                    JSONObject()
                        .put("settings_id", "fast-model")
                        .put("provider_kind", "openai")
                        .put("configured", true)
                        .put("api_key_configured", false),
                )
                .put(
                    JSONObject()
                        .put("settings_id", "photo-eval-model")
                        .put("provider_kind", "openai")
                        .put("configured", true)
                        .put("api_key_configured", true),
                ),
        )
        val worker = SmartSelectionAnalysisWorker(
            context = null,
            core = core,
            previewSampleLoader = { _, _ -> "{}" },
        )

        worker.drainOnce(maxAssessments = 1)

        assertEquals(listOf("group-1:true"), core.assessedGroups)
        assertEquals(listOf(true), core.drainProviderFlags)
    }

    @Test
    fun assessedBurstGroupsTriggerVisualRecommendation() {
        val core = FakeSmartSelectionCore(
            providerConfigured = true,
            assetModelStatus = "pending",
            selectedProviderId = "global",
            providerOptions = readyProviderOptions("global"),
            burstAssetCount = 2,
        )
        val worker = SmartSelectionAnalysisWorker(
            context = null,
            core = core,
            previewSampleLoader = { _, location ->
                JSONObject()
                    .put("width", 1)
                    .put("height", 1)
                    .put("luma", JSONArray().put(128))
                    .put("image_data_url", "data:image/jpeg;base64,$location")
                    .toString()
            },
        )

        val result = worker.drainOnce(maxAssessments = 2)

        assertEquals(2, result.assessedCount)
        assertEquals(2, core.assessedGroups.size)
        assertEquals(1, core.burstVisualRecommendationCalls.size)
        val call = core.burstVisualRecommendationCalls.single()
        assertEquals("burst-1", call.burstGroupId)
        assertEquals(2, call.candidateVisuals.length())
        assertEquals("group-1", call.candidateVisuals.getJSONObject(0).getString("asset_group_id"))
        assertEquals("group-2", call.candidateVisuals.getJSONObject(1).getString("asset_group_id"))
    }

    @Test
    fun burstAutoSelectionStillPreselectsWhenUploadAutoEvaluationIsDisabled() {
        val core = FakeSmartSelectionCore(
            providerConfigured = true,
            assetModelStatus = "pending",
            selectedProviderId = "global",
            providerOptions = readyProviderOptions("global"),
            autoEvaluateOnUpload = false,
            burstAssetCount = 2,
        )
        val worker = SmartSelectionAnalysisWorker(
            context = null,
            core = core,
            previewSampleLoader = { _, location ->
                JSONObject()
                    .put("width", 1)
                    .put("height", 1)
                    .put("luma", JSONArray().put(128))
                    .put("image_data_url", "data:image/jpeg;base64,$location")
                    .toString()
            },
        )

        val result = worker.drainOnce(maxAssessments = 2)

        assertEquals(2, result.assessedCount)
        assertEquals(1, core.burstVisualRecommendationCalls.size)
        assertEquals("burst-1", core.burstVisualRecommendationCalls.single().burstGroupId)
    }

    @Test
    fun burstAutoSelectionEvaluatesOnlyPreselectedCandidatesWhenUploadAutoEvaluationIsEnabled() {
        val core = FakeSmartSelectionCore(
            providerConfigured = true,
            assetModelStatus = "pending",
            selectedProviderId = "global",
            providerOptions = readyProviderOptions("global"),
            burstAssetCount = 4,
            firstBurstRecommendationCandidateIds = listOf("group-2", "group-4"),
        )
        val worker = SmartSelectionAnalysisWorker(
            context = null,
            core = core,
            previewSampleLoader = { _, location ->
                JSONObject()
                    .put("width", 1)
                    .put("height", 1)
                    .put("luma", JSONArray().put(128))
                    .put("image_data_url", "data:image/jpeg;base64,$location")
                    .toString()
            },
        )

        worker.drainOnce(maxAssessments = 4)

        assertEquals(
            listOf("group-1:false", "group-2:false", "group-3:false", "group-4:false"),
            core.assessedGroups,
        )
        assertEquals(listOf(listOf("group-2", "group-4")), core.modelEvaluationInputCalls)
        assertEquals(2, core.burstVisualRecommendationCalls.size)
    }

    private class FakeSmartSelectionCore(
        private val providerConfigured: Boolean,
        private val assetModelStatus: String? = "pending",
        private val selectedProviderId: String? = null,
        private val providerOptions: JSONArray = JSONArray(),
        private val autoEvaluateOnUpload: Boolean = true,
        private val burstAssetCount: Int = 1,
        private val firstBurstRecommendationCandidateIds: List<String> = emptyList(),
    ) : SmartSelectionCore {
        val assessedGroups = mutableListOf<String>()
        val drainProviderFlags = mutableListOf<Boolean>()
        val burstVisualRecommendationCalls = mutableListOf<BurstVisualRecommendationCall>()
        val modelEvaluationInputCalls = mutableListOf<List<String>>()
        var projectRecommendationCalls = 0

        override fun activeProject(): JSONObject =
            JSONObject().put("project_id", "project-1")

        override fun modelProviderSettingsList(): JSONArray =
            if (providerOptions.length() > 0) {
                providerOptions
            } else if (providerConfigured) {
                readyProviderOptions("global")
            } else {
                JSONArray()
            }

        override fun projectEvaluationSettings(projectId: String): JSONObject =
            JSONObject()
                .put("project_id", projectId)
                .put("model_evaluation_enabled", true)
                .put("auto_evaluate_on_upload", autoEvaluateOnUpload)
                .put("auto_burst_recommendation_enabled", true)
                .put("project_recommendation_mode", "manual")
                .put("model_provider_settings_id", selectedProviderId ?: JSONObject.NULL)

        override fun projectAssetGroupPageJson(
            projectId: String,
            query: ProjectAssetQuery,
            offset: Int,
            limit: Int,
        ): JSONObject {
            val groups = JSONArray()
            for (index in 1..burstAssetCount) {
                val group = JSONObject()
                    .put("group_id", "group-$index")
                    .put("group_key", "group-$index")
                    .put(
                        "primary",
                        JSONObject()
                            .put("id", "asset-$index")
                            .put("filename", "IMG_000$index.JPG")
                            .put("format", "Jpeg")
                            .put("received_time_ms", index)
                            .put(
                                "storage_location",
                                JSONObject().put("path", "preview-$index.jpg"),
                            ),
                    )
                if (burstAssetCount > 1) {
                    group.put(
                        "burst",
                        JSONObject()
                            .put("burst_group_id", "burst-1")
                            .put("member_count", burstAssetCount)
                            .put("recommendation_status", "pending"),
                    )
                }
                if (assetModelStatus != null) {
                    group.put("model_status", assetModelStatus)
                }
                groups.put(group)
            }
            return JSONObject().put("groups", groups)
        }

        override fun assessAssetGroupPreviewWithProviderConfigured(
            assetGroupId: String,
            sampleJson: String,
            assessorVersion: String,
            providerConfigured: Boolean,
        ): JSONObject {
            assessedGroups += "$assetGroupId:$providerConfigured"
            return JSONObject()
        }

        override fun drainAnalysisJobsWithProviderConfigured(
            limit: Int,
            providerConfigured: Boolean,
        ): JSONObject {
            drainProviderFlags += providerConfigured
            return JSONObject().put("completed_count", 1)
        }

        override fun generateProjectRecommendation(projectId: String): JSONObject {
            projectRecommendationCalls += 1
            return JSONObject()
        }

        override fun recommendBurstGroupWithCandidateVisuals(
            burstGroupId: String,
            candidateVisuals: JSONArray,
        ): JSONObject {
            burstVisualRecommendationCalls += BurstVisualRecommendationCall(
                burstGroupId = burstGroupId,
                candidateVisuals = candidateVisuals,
            )
            return if (
                firstBurstRecommendationCandidateIds.isNotEmpty() &&
                modelEvaluationInputCalls.isEmpty()
            ) {
                JSONObject()
                    .put("status", "pending")
                    .put("candidate_asset_group_ids", JSONArray(firstBurstRecommendationCandidateIds))
                    .put("selected_asset_group_ids", JSONArray())
            } else {
                JSONObject()
                    .put("status", "ready")
                    .put(
                        "selected_asset_group_ids",
                        JSONArray().put(firstBurstRecommendationCandidateIds.firstOrNull() ?: "group-1"),
                    )
                    .put("candidate_asset_group_ids", JSONArray())
            }
        }

        override fun evaluateAssetGroupsWithModelInputs(
            projectId: String,
            inputs: JSONArray,
        ): JSONObject {
            modelEvaluationInputCalls += (0 until inputs.length()).map { index ->
                inputs.getJSONObject(index).getString("asset_group_id")
            }
            return JSONObject().put("saved_count", inputs.length())
        }
    }

    private data class BurstVisualRecommendationCall(
        val burstGroupId: String,
        val candidateVisuals: JSONArray,
    )

    private companion object {
        fun readyProviderOptions(settingsId: String): JSONArray =
            JSONArray().put(
                JSONObject()
                    .put("settings_id", settingsId)
                    .put("provider_kind", "openai")
                    .put("configured", true)
                    .put("api_key_configured", true),
            )
    }
}

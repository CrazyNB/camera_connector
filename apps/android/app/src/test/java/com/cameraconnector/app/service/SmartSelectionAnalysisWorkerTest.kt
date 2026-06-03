package com.cameraconnector.app.service

import com.cameraconnector.app.core.InboxAssetQuery
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

        val result = worker.drainOnce(maxScores = 1)

        assertEquals(1, result.scoredCount)
        assertEquals(listOf("group-1:false"), core.scoredGroups)
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

        worker.drainOnce(maxScores = 1)

        assertEquals(1, core.drainProviderFlags.size)
        assertEquals(0, core.projectRecommendationCalls)
    }

    private class FakeSmartSelectionCore(
        private val providerConfigured: Boolean,
    ) : SmartSelectionCore {
        val scoredGroups = mutableListOf<String>()
        val drainProviderFlags = mutableListOf<Boolean>()
        var projectRecommendationCalls = 0

        override fun activeProject(): JSONObject =
            JSONObject().put("project_id", "project-1")

        override fun modelProviderSettings(): JSONObject =
            JSONObject()
                .put("provider_kind", if (providerConfigured) "openai" else "none")
                .put("configured", providerConfigured)

        override fun projectEvaluationSettings(projectId: String): JSONObject =
            JSONObject()
                .put("project_id", projectId)
                .put("model_evaluation_enabled", true)
                .put("auto_evaluate_on_upload", true)
                .put("auto_burst_recommendation_enabled", true)
                .put("project_recommendation_mode", "manual")

        override fun projectAssetGroupPageJson(
            projectId: String,
            query: InboxAssetQuery,
            offset: Int,
            limit: Int,
        ): JSONObject =
            JSONObject().put(
                "groups",
                JSONArray().put(
                    JSONObject()
                        .put("group_id", "group-1")
                        .put("group_key", "group-1")
                        .put(
                            "primary",
                            JSONObject()
                                .put("id", "asset-1")
                                .put("filename", "IMG_0001.JPG")
                                .put("format", "Jpeg")
                                .put("received_time_ms", 1)
                                .put(
                                    "storage_location",
                                    JSONObject().put("path", "preview.jpg"),
                                ),
                        )
                        .put("model_status", "pending"),
                ),
            )

        override fun scoreAssetGroupPreviewWithProviderConfigured(
            assetGroupId: String,
            sampleJson: String,
            scorerVersion: String,
            providerConfigured: Boolean,
        ): JSONObject {
            scoredGroups += "$assetGroupId:$providerConfigured"
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
    }
}

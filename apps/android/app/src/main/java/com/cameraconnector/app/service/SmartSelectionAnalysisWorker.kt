package com.cameraconnector.app.service

import android.content.Context
import android.util.Log
import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetQuery
import com.cameraconnector.app.core.NativeMobileCore
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.mapInboxAssets
import com.cameraconnector.app.media.loadPreviewSampleJson
import org.json.JSONObject

data class SmartSelectionDrainResult(
    val scoredCount: Int,
    val recommendedCount: Int,
    val failedCount: Int,
)

interface SmartSelectionCore {
    fun activeProject(): JSONObject?
    fun modelProviderSettings(): JSONObject
    fun projectEvaluationSettings(projectId: String): JSONObject
    fun projectAssetGroupPageJson(
        projectId: String,
        query: InboxAssetQuery,
        offset: Int,
        limit: Int,
    ): JSONObject
    fun scoreAssetGroupPreviewWithProviderConfigured(
        assetGroupId: String,
        sampleJson: String,
        scorerVersion: String = "local-v1",
        providerConfigured: Boolean,
    ): JSONObject
    fun drainAnalysisJobsWithProviderConfigured(
        limit: Int = 32,
        providerConfigured: Boolean,
    ): JSONObject
    fun generateProjectRecommendation(projectId: String): JSONObject
}

class NativeSmartSelectionCore(
    private val core: NativeMobileCore,
) : SmartSelectionCore {
    override fun activeProject(): JSONObject? = core.activeProject()
    override fun modelProviderSettings(): JSONObject = core.modelProviderSettings()
    override fun projectEvaluationSettings(projectId: String): JSONObject = core.projectEvaluationSettings(projectId)
    override fun projectAssetGroupPageJson(
        projectId: String,
        query: InboxAssetQuery,
        offset: Int,
        limit: Int,
    ): JSONObject = core.projectAssetGroupPageJson(projectId, query, offset, limit)

    override fun scoreAssetGroupPreviewWithProviderConfigured(
        assetGroupId: String,
        sampleJson: String,
        scorerVersion: String,
        providerConfigured: Boolean,
    ): JSONObject = core.scoreAssetGroupPreviewWithProviderConfigured(
        assetGroupId = assetGroupId,
        sampleJson = sampleJson,
        scorerVersion = scorerVersion,
        providerConfigured = providerConfigured,
    )

    override fun drainAnalysisJobsWithProviderConfigured(
        limit: Int,
        providerConfigured: Boolean,
    ): JSONObject = core.drainAnalysisJobsWithProviderConfigured(limit, providerConfigured)

    override fun generateProjectRecommendation(projectId: String): JSONObject =
        core.generateProjectRecommendation(projectId)
}

class SmartSelectionAnalysisWorker(
    private val context: Context?,
    private val core: SmartSelectionCore,
    private val previewSampleLoader: (Context?, String?) -> String = { loadContext, previewLocation ->
        loadPreviewSampleJson(requireNotNull(loadContext), previewLocation)
    },
) {
    constructor(
        context: Context,
        core: NativeMobileCore,
    ) : this(context, NativeSmartSelectionCore(core))

    fun drainOnce(maxScores: Int = DEFAULT_MAX_SCORES): SmartSelectionDrainResult {
        val projectId = core.activeProject()
            ?.optString("project_id")
            ?.takeIf { it.isNotBlank() }
            ?: return SmartSelectionDrainResult(scoredCount = 0, recommendedCount = 0, failedCount = 0)
        core.projectEvaluationSettings(projectId)
        val providerConfigured = core.modelProviderSettings().optBoolean("configured", false)
        val assets = mapInboxAssets(
            core.projectAssetGroupPageJson(
                projectId = projectId,
                query = InboxAssetQuery(sort = PhotoSortMode.LatestReceived),
                offset = 0,
                limit = QUERY_LIMIT,
            ),
        )
        var scoredCount = 0
        var failedCount = 0

        for (asset in assets) {
            if (scoredCount >= maxScores) {
                break
            }
            if (!asset.needsLocalScore()) {
                continue
            }
            runCatching {
                val sampleJson = previewSampleLoader(context, asset.previewLocation)
                core.scoreAssetGroupPreviewWithProviderConfigured(
                    assetGroupId = asset.id,
                    sampleJson = sampleJson,
                    providerConfigured = providerConfigured,
                )
                scoredCount += 1
            }.onFailure { error ->
                failedCount += 1
                Log.w(LOG_TAG, "smart selection scoring failed group=${asset.id}", error)
            }
        }

        var recommendedCount = 0
        if (scoredCount > 0) {
            runCatching {
                recommendedCount = core.drainAnalysisJobsWithProviderConfigured(
                    providerConfigured = providerConfigured,
                ).optInt("completed_count")
            }.onFailure { error ->
                failedCount += 1
                Log.w(LOG_TAG, "smart selection analysis queue drain failed", error)
            }
        }

        return SmartSelectionDrainResult(
            scoredCount = scoredCount,
            recommendedCount = recommendedCount,
            failedCount = failedCount,
        )
    }

    private companion object {
        const val DEFAULT_MAX_SCORES = 12
        const val QUERY_LIMIT = 128
        const val LOG_TAG = "SmartSelectionAnalysis"
    }
}

private fun InboxAsset.needsLocalScore(): Boolean {
    val currentModelStatus = this.modelStatus?.lowercase()
    val technicalStatus = this.technicalGateStatus?.lowercase()
    val legacyStatus = quality?.analysisStatus?.lowercase()
    if (currentModelStatus == "ready" || currentModelStatus == "skipped") {
        return false
    }
    if (
        technicalStatus in listOf("pass", "warn", "needs_review", "reject", "unsupported") &&
        currentModelStatus != null
    ) {
        return currentModelStatus == "pending" ||
            currentModelStatus == "running" ||
            currentModelStatus == "failed"
    }
    return legacyStatus == null ||
        legacyStatus == "pending" ||
        legacyStatus == "analyzing" ||
        legacyStatus == "stale" ||
        legacyStatus == "failed" ||
        currentModelStatus == null ||
        currentModelStatus == "pending" ||
        currentModelStatus == "running" ||
        currentModelStatus == "failed"
}

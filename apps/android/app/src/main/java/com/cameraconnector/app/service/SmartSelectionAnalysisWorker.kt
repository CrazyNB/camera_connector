package com.cameraconnector.app.service

import android.content.Context
import android.util.Log
import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetQuery
import com.cameraconnector.app.core.NativeMobileCore
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.mapInboxAssets
import com.cameraconnector.app.media.loadPreviewSampleJson

data class SmartSelectionDrainResult(
    val scoredCount: Int,
    val recommendedCount: Int,
    val failedCount: Int,
)

class SmartSelectionAnalysisWorker(
    private val context: Context,
    private val core: NativeMobileCore,
    private val strategyProfileIdProvider: () -> String? = { null },
) {
    fun drainOnce(maxScores: Int = DEFAULT_MAX_SCORES): SmartSelectionDrainResult {
        val projectId = core.activeProject()
            ?.optString("project_id")
            ?.takeIf { it.isNotBlank() }
            ?: return SmartSelectionDrainResult(scoredCount = 0, recommendedCount = 0, failedCount = 0)
        val assets = mapInboxAssets(
            core.projectAssetGroupPageJson(
                projectId = projectId,
                query = InboxAssetQuery(sort = PhotoSortMode.LatestReceived),
                offset = 0,
                limit = QUERY_LIMIT,
            ),
        )
        val affectedBurstGroupIds = linkedSetOf<String>()
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
                val sampleJson = loadPreviewSampleJson(context, asset.previewLocation)
                core.scoreAssetGroupPreview(asset.id, sampleJson)
                asset.burst?.burstGroupId
                    ?.takeIf { it.isNotBlank() }
                    ?.let(affectedBurstGroupIds::add)
                scoredCount += 1
            }.onFailure { error ->
                failedCount += 1
                Log.w(LOG_TAG, "smart selection scoring failed group=${asset.id}", error)
            }
        }

        var recommendedCount = 0
        val strategyProfileId = strategyProfileIdProvider()
            ?.trim()
            ?.takeIf { it.isNotBlank() }
        for (burstGroupId in affectedBurstGroupIds) {
            runCatching {
                core.recommendBurstGroup(burstGroupId, strategyProfileId)
                recommendedCount += 1
            }.onFailure { error ->
                failedCount += 1
                Log.w(LOG_TAG, "smart selection recommendation failed burst=$burstGroupId", error)
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
    val status = quality?.analysisStatus?.lowercase()
    return status == null || status == "pending" || status == "analyzing" || status == "stale" || status == "failed"
}

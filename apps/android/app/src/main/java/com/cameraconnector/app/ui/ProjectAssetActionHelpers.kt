package com.cameraconnector.app.ui

import android.content.Context
import android.graphics.Bitmap
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import com.cameraconnector.app.core.SelectionCandidateVisualInput
import com.cameraconnector.app.media.loadPreviewSampleJson
import java.io.ByteArrayOutputStream
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
internal suspend fun loadProjectSyncSnapshotAssets(
    coreGateway: CoreGateway,
    pageSize: Int = 2_000,
): List<ProjectAsset> {
    val cleanPageSize = pageSize.coerceAtLeast(1)
    val assets = mutableListOf<ProjectAsset>()
    var offset = 0
    while (true) {
        val page = coreGateway.loadProjectAssets(ProjectAssetQuery(), offset = offset, limit = cleanPageSize)
        assets += page
        if (page.size < cleanPageSize) {
            return assets
        }
        offset += page.size
    }
}

internal suspend fun burstRecommendationCandidateVisuals(
    context: Context,
    members: List<ProjectAsset>,
): List<SelectionCandidateVisualInput> =
    withContext(Dispatchers.IO) {
        members
            .distinctBy { it.assetSelectionId() }
            .mapNotNull { asset ->
                val imageDataUrl = runCatching {
                    JSONObject(loadPreviewSampleJson(context, asset.previewLocation))
                        .optString("image_data_url")
                        .takeIf { it.isNotBlank() && it != "null" }
                }.getOrNull()
                imageDataUrl?.let {
                    SelectionCandidateVisualInput(
                        assetGroupId = asset.id,
                        imageDataUrl = it,
                    )
                }
            }
    }

internal fun projectEvaluationFeedback(
    evaluatedCount: Int,
    recommendedBurstCount: Int,
): String =
    when {
        evaluatedCount > 0 && recommendedBurstCount > 0 ->
            "\u5df2\u5b8c\u6210\u5355\u5f20\u8bc4\u4ef7 $evaluatedCount \u00b7 \u8fde\u62cd\u8bc4\u4ef7 $recommendedBurstCount"
        recommendedBurstCount > 0 ->
            "\u5df2\u5b8c\u6210\u8fde\u62cd\u8bc4\u4ef7 $recommendedBurstCount"
        evaluatedCount > 0 ->
            "\u5df2\u5b8c\u6210\u5355\u5f20\u8bc4\u4ef7 $evaluatedCount"
        else -> "\u6ca1\u6709\u53ef\u8bc4\u4ef7\u9879"
    }


internal fun Bitmap.toJpegBytes(quality: Int = 82): ByteArray {
    val output = ByteArrayOutputStream()
    compress(Bitmap.CompressFormat.JPEG, quality, output)
    return output.toByteArray()
}

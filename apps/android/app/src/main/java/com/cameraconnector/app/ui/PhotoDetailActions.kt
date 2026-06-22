package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.AutoAwesome
import androidx.compose.material.icons.outlined.BookmarkAdd
import androidx.compose.material.icons.outlined.BookmarkAdded
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.DeleteSweep
import androidx.compose.material.icons.outlined.Star
import androidx.compose.material.icons.outlined.StarBorder
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.ProjectAsset

internal fun photoMetadataDisplayAspectRatio(
    dimensions: String?,
    orientation: String?,
): Float? {
    val values = Regex("""\d+""")
        .findAll(dimensions.orEmpty())
        .mapNotNull { it.value.toFloatOrNull() }
        .take(2)
        .toList()
    val width = values.getOrNull(0)?.takeIf { it > 0f } ?: return null
    val height = values.getOrNull(1)?.takeIf { it > 0f } ?: return null
    val swapsAxes = orientation
        .orEmpty()
        .contains(Regex("""90|270|杞疆"""))
    return if (swapsAxes) height / width else width / height
}

internal fun photoDetailSourceLines(asset: ProjectAsset): List<Pair<String, String>> =
    listOf(
        "鏉ユ簮" to asset.sourceLabel(),
        "璐﹀彿" to (asset.username ?: "鏈煡"),
        "鍘熷璺緞" to (asset.originalPath ?: asset.displayPath),
        "鎺ユ敹鏃堕棿" to formatEpochMillisTextForDisplay(asset.receivedAt),
        "鏂囦欢澶у皬" to (asset.sizeBytes?.let { "$it bytes" } ?: "鏈煡"),
    )

internal fun photoDetailFileLines(asset: ProjectAsset): List<Pair<String, String>> =
    listOfNotNull(
        "浣嶇疆" to asset.displayPath,
        asset.rawPath?.takeIf { it.isNotBlank() }?.let { "RAW" to it },
        asset.jpegPath?.takeIf { it.isNotBlank() }?.let { "JPEG" to it },
        asset.videoPath?.takeIf { it.isNotBlank() }?.let { "瑙嗛" to it },
    )

@Composable
internal fun PhotoDetailDecisionActions(
    asset: ProjectAsset,
    decision: PhotoDetailDecisionUi,
    onSplitBurstMember: ((String, String) -> Unit)?,
    markedSelected: Boolean,
    markedEnabled: Boolean,
    onToggleMarked: ((String) -> Unit)?,
    favoriteSelected: Boolean,
    favoriteEnabled: Boolean,
    onToggleFavorite: ((String) -> Unit)?,
    evaluateEnabled: Boolean,
    evaluationInFlight: Boolean,
    onEvaluateModel: (() -> Unit)?,
    onDeleteAsset: ((String) -> Unit)?,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        PhotoDetailIconAction(
            icon = if (markedSelected) Icons.Outlined.BookmarkAdded else Icons.Outlined.BookmarkAdd,
            contentDescription = if (markedSelected) "\u5df2\u6807\u8bb0" else "\u6807\u8bb0",
            tint = ElementBlue,
            enabled = markedEnabled,
            onClick = {
                onToggleMarked?.invoke(asset.assetSelectionId())
            },
        )
        PhotoDetailIconAction(
            icon = if (favoriteSelected) Icons.Outlined.Star else Icons.Outlined.StarBorder,
            contentDescription = if (favoriteSelected) "\u5df2\u6536\u85cf" else "\u6536\u85cf",
            tint = ElementSuccess,
            enabled = favoriteEnabled,
            onClick = {
                onToggleFavorite?.invoke(asset.assetSelectionId())
            },
        )
        PhotoDetailIconAction(
            icon = Icons.Outlined.AutoAwesome,
            contentDescription = "\u63d0\u4ea4\u6a21\u578b\u8bc4\u4ef7",
            tint = ElementBlue,
            enabled = evaluateEnabled && onEvaluateModel != null && !evaluationInFlight,
            loading = evaluationInFlight,
            stateDescription = when {
                evaluationInFlight -> "\u5df2\u63d0\u4ea4\uff0c\u7b49\u5f85\u7ed3\u679c"
                evaluateEnabled -> "\u53ef\u7528"
                else -> "\u4e0d\u53ef\u7528"
            },
            onClick = {
                onEvaluateModel?.invoke()
            },
        )
        PhotoDetailIconAction(
            icon = Icons.Outlined.DeleteSweep,
            contentDescription = "\u79fb\u51fa\u8fde\u62cd\u7ec4\uff0c\u4e0d\u5220\u9664\u7167\u7247",
            tint = ElementWarning,
            enabled = decision.splitBurstEnabled && onSplitBurstMember != null,
            onClick = {
                decision.splitBurstTarget?.let { target ->
                    onSplitBurstMember?.invoke(target.burstGroupId, target.memberGroupId)
                }
            },
        )
        PhotoDetailIconAction(
            icon = Icons.Outlined.Delete,
            contentDescription = "鍒犻櫎鐓х墖",
            tint = ElementDanger,
            enabled = onDeleteAsset != null,
            onClick = {
                onDeleteAsset?.invoke(asset.assetSelectionId())
            },
        )
    }
}

@Composable
private fun PhotoDetailIconAction(
    icon: ImageVector,
    contentDescription: String,
    tint: Color,
    enabled: Boolean,
    loading: Boolean = false,
    stateDescription: String? = null,
    onClick: () -> Unit,
) {
    Surface(
        modifier = Modifier
            .size(42.dp)
            .clickable(enabled = enabled, onClick = onClick),
        color = if (enabled) tint.copy(alpha = 0.08f) else ElementControlSurface,
        contentColor = if (enabled) tint else tint.copy(alpha = 0.34f),
        shape = CircleShape,
        border = BorderStroke(1.dp, if (enabled) tint.copy(alpha = 0.42f) else ElementBorder),
    ) {
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier.semantics {
                this.contentDescription = contentDescription
                stateDescription?.let { this.stateDescription = it }
            },
        ) {
            if (loading) {
                CircularProgressIndicator(
                    modifier = Modifier.size(21.dp),
                    strokeWidth = 2.dp,
                    color = tint,
                )
            } else {
                Icon(
                    imageVector = icon,
                    contentDescription = null,
                    modifier = Modifier.size(22.dp),
                )
            }
        }
    }
}

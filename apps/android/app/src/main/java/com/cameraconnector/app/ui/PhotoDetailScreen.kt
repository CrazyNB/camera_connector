package com.cameraconnector.app.ui

import android.widget.Toast
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.sp
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.media.PhotoMetadata
import com.cameraconnector.app.media.isDecodablePreviewLocation
import com.cameraconnector.app.media.loadPhotoMetadata
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlin.math.abs
import kotlin.math.roundToInt

@Composable
internal fun PhotoDetailScreen(
    asset: ProjectAsset,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
    actionsEnabled: Boolean = true,
    onSplitBurstMember: ((String, String) -> Unit)? = null,
    burstMembers: List<BurstMemberFilmstripItemUi> = emptyList(),
    onOpenBurstMember: ((ProjectAsset) -> Unit)? = null,
    previousGroupAsset: ProjectAsset? = null,
    nextGroupAsset: ProjectAsset? = null,
    onNavigatePreviousGroup: (() -> Unit)? = null,
    onNavigateNextGroup: (() -> Unit)? = null,
    onToggleMarked: ((String) -> Unit)? = null,
    onToggleFavorite: ((String) -> Unit)? = null,
    onEvaluateModel: ((ProjectAsset) -> Unit)? = null,
    onDeleteAsset: ((String) -> Unit)? = null,
) {
    var fullScreenPreview by remember { mutableStateOf(false) }
    val assetId = asset.assetSelectionId()
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val exportUi = remember(assetId, asset.previewLocation, asset.displayPath, asset.originalPath) {
        photoDetailExportUi(asset)
    }
    var exportActionsVisible by remember(assetId) { mutableStateOf(false) }
    val showExportActions = {
        if (exportUi.enabled) {
            exportActionsVisible = true
        } else {
            Toast.makeText(
                context,
                exportUi.unavailableReason ?: "\u6ca1\u6709\u53ef\u5bfc\u51fa\u7684\u7167\u7247",
                Toast.LENGTH_SHORT,
            ).show()
        }
    }
    if (fullScreenPreview) {
        FullScreenPhotoPreview(
            asset = asset,
            onDismiss = { fullScreenPreview = false },
            onLongPress = showExportActions,
        )
    }
    if (exportActionsVisible) {
        PhotoDetailExportDialog(
            exportUi = exportUi,
            onDismiss = { exportActionsVisible = false },
            onSave = {
                exportActionsVisible = false
                scope.launch {
                    val saved = savePhotoDetailExportToGallery(context, asset, exportUi)
                    Toast.makeText(
                        context,
                        if (saved) {
                            "\u5df2\u4fdd\u5b58\u5230\u76f8\u518c"
                        } else {
                            "\u4fdd\u5b58\u5931\u8d25"
                        },
                        Toast.LENGTH_SHORT,
                    ).show()
                }
            },
            onShare = {
                exportActionsVisible = false
                scope.launch {
                    val shared = sharePhotoDetailExport(context, asset, exportUi)
                    if (!shared) {
                        Toast.makeText(context, "\u5206\u4eab\u5931\u8d25", Toast.LENGTH_SHORT).show()
                    }
                }
            },
        )
    }
    var dragOffsetX by remember(assetId) { mutableFloatStateOf(0f) }
    var settleOffsetX by remember(assetId) { mutableFloatStateOf(0f) }
    var settling by remember(assetId) { mutableStateOf(false) }
    BoxWithConstraints(
        modifier = modifier
            .fillMaxSize()
            .clipToBounds(),
    ) {
        val density = LocalDensity.current
        val pageWidthPx = with(density) { maxWidth.toPx() }.coerceAtLeast(1f)
        val pageStridePx = pageWidthPx * 0.96f
        val thresholdPx = pageWidthPx * 0.16f
        val pageOffsetX = if (settling) settleOffsetX else dragOffsetX
        val dragDirection = if (pageOffsetX >= 0f) {
            DetailNavigationDirection.Previous
        } else {
            DetailNavigationDirection.Next
        }
        val shadowAlpha = (abs(pageOffsetX) / thresholdPx).coerceIn(0f, 1f)
        val groupSwipeCallbacks = remember(
            assetId,
            previousGroupAsset?.assetSelectionId(),
            nextGroupAsset?.assetSelectionId(),
            pageStridePx,
            thresholdPx,
        ) {
            DetailGroupSwipeCallbacks(
                onDragStart = {
                    settling = false
                    settleOffsetX = 0f
                },
                onDrag = { dragAmount ->
                    dragOffsetX = (dragOffsetX + dragAmount).coerceIn(
                        minimumValue = if (nextGroupAsset != null && onNavigateNextGroup != null) {
                            -pageStridePx
                        } else {
                            0f
                        },
                        maximumValue = if (previousGroupAsset != null && onNavigatePreviousGroup != null) {
                            pageStridePx
                        } else {
                            0f
                        },
                    )
                },
                onDragEnd = {
                    val releaseOffset = dragOffsetX
                    val target = when {
                        releaseOffset > thresholdPx && previousGroupAsset != null && onNavigatePreviousGroup != null ->
                            DetailNavigationDirection.Previous
                        releaseOffset < -thresholdPx && nextGroupAsset != null && onNavigateNextGroup != null ->
                            DetailNavigationDirection.Next
                        else -> null
                    }
                    scope.launch {
                        settling = true
                        settleOffsetX = releaseOffset
                        val animation = Animatable(releaseOffset)
                        if (target == null) {
                            animation.animateTo(0f, tween(durationMillis = 110)) {
                                settleOffsetX = value
                            }
                            dragOffsetX = 0f
                            settleOffsetX = 0f
                            settling = false
                        } else {
                            val targetOffset = if (target == DetailNavigationDirection.Previous) {
                                pageStridePx
                            } else {
                                -pageStridePx
                            }
                            animation.animateTo(targetOffset, tween(durationMillis = 130)) {
                                settleOffsetX = value
                            }
                            if (target == DetailNavigationDirection.Previous) {
                                onNavigatePreviousGroup?.invoke()
                            } else {
                                onNavigateNextGroup?.invoke()
                            }
                            dragOffsetX = 0f
                            settleOffsetX = 0f
                            settling = false
                        }
                    }
                },
                onDragCancel = {
                    val releaseOffset = dragOffsetX
                    scope.launch {
                        settling = true
                        settleOffsetX = releaseOffset
                        Animatable(releaseOffset).animateTo(0f, tween(durationMillis = 110)) {
                            settleOffsetX = value
                        }
                        dragOffsetX = 0f
                        settleOffsetX = 0f
                        settling = false
                    }
                },
            )
        }
        val showPreviousPage = previousGroupAsset != null && pageOffsetX > 0.5f
        val showNextPage = nextGroupAsset != null && pageOffsetX < -0.5f
        Box(Modifier.fillMaxSize()) {
            previousGroupAsset?.takeIf { showPreviousPage }?.let { previous ->
                PhotoDetailContent(
                    asset = previous,
                    onBack = onBack,
                    modifier = Modifier
                        .fillMaxSize()
                        .offset { IntOffset((pageOffsetX - pageStridePx).roundToInt(), 0) },
                    actionsEnabled = false,
                    onSplitBurstMember = null,
                    burstMembers = emptyList(),
                    onOpenBurstMember = null,
                    groupSwipeCallbacks = null,
                    onToggleMarked = null,
                    onToggleFavorite = null,
                    onEvaluateModel = null,
                    onDeleteAsset = null,
                    onPreviewClick = {},
                    onPreviewLongPress = {},
                )
            }
            nextGroupAsset?.takeIf { showNextPage }?.let { next ->
                PhotoDetailContent(
                    asset = next,
                    onBack = onBack,
                    modifier = Modifier
                        .fillMaxSize()
                        .offset { IntOffset((pageOffsetX + pageStridePx).roundToInt(), 0) },
                    actionsEnabled = false,
                    onSplitBurstMember = null,
                    burstMembers = emptyList(),
                    onOpenBurstMember = null,
                    groupSwipeCallbacks = null,
                    onToggleMarked = null,
                    onToggleFavorite = null,
                    onEvaluateModel = null,
                    onDeleteAsset = null,
                    onPreviewClick = {},
                    onPreviewLongPress = {},
                )
            }
            PhotoDetailContent(
                asset = asset,
                onBack = onBack,
                modifier = Modifier
                    .fillMaxSize()
                    .offset { IntOffset(pageOffsetX.roundToInt(), 0) },
                actionsEnabled = actionsEnabled,
                onSplitBurstMember = onSplitBurstMember,
                burstMembers = burstMembers,
                onOpenBurstMember = onOpenBurstMember,
                groupSwipeCallbacks = groupSwipeCallbacks,
                onToggleMarked = onToggleMarked,
                onToggleFavorite = onToggleFavorite,
                onEvaluateModel = onEvaluateModel,
                onDeleteAsset = onDeleteAsset,
                onPreviewClick = { fullScreenPreview = true },
                onPreviewLongPress = showExportActions,
            )
            DetailPageTurnShadow(
                direction = dragDirection,
                alpha = shadowAlpha,
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

@Composable
private fun DetailPageTurnShadow(
    direction: DetailNavigationDirection,
    alpha: Float,
    modifier: Modifier = Modifier,
) {
    if (alpha <= 0.01f) {
        return
    }
    val edgeAlignment = if (direction == DetailNavigationDirection.Next) {
        Alignment.CenterEnd
    } else {
        Alignment.CenterStart
    }
    val shadowColors = if (direction == DetailNavigationDirection.Next) {
        listOf(
            Color.Transparent,
            ElementBackground.copy(alpha = 0.18f),
            Color.Black.copy(alpha = 0.34f),
        )
    } else {
        listOf(
            Color.Black.copy(alpha = 0.34f),
            ElementBackground.copy(alpha = 0.18f),
            Color.Transparent,
        )
    }
    Box(
        modifier = modifier.graphicsLayer { this.alpha = alpha },
    ) {
        Box(
            modifier = Modifier
                .align(edgeAlignment)
                .fillMaxHeight()
                .width(54.dp)
                .background(Brush.horizontalGradient(shadowColors)),
        )
    }
}

private class DetailGroupSwipeCallbacks(
    val onDragStart: () -> Unit,
    val onDrag: (Float) -> Unit,
    val onDragEnd: () -> Unit,
    val onDragCancel: () -> Unit,
)

@Composable
private fun PhotoDetailContent(
    asset: ProjectAsset,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
    actionsEnabled: Boolean,
    onSplitBurstMember: ((String, String) -> Unit)?,
    burstMembers: List<BurstMemberFilmstripItemUi>,
    onOpenBurstMember: ((ProjectAsset) -> Unit)?,
    groupSwipeCallbacks: DetailGroupSwipeCallbacks?,
    onToggleMarked: ((String) -> Unit)?,
    onToggleFavorite: ((String) -> Unit)?,
    onEvaluateModel: ((ProjectAsset) -> Unit)?,
    onDeleteAsset: ((String) -> Unit)?,
    onPreviewClick: () -> Unit,
    onPreviewLongPress: () -> Unit,
) {
    val context = LocalContext.current
    val metadataLocation = asset.previewLocation.takeIf(::isDecodablePreviewLocation)
    val photoMetadata by produceState<PhotoMetadata?>(initialValue = null, metadataLocation) {
        value = if (metadataLocation == null) {
            null
        } else {
            withContext(Dispatchers.IO) {
                loadPhotoMetadata(context, metadataLocation)
            }
        }
    }
    val previousBurstMember = remember(asset, burstMembers) {
        adjacentBurstMemberAsset(
            currentAsset = asset,
            allProjectAssets = burstMembers.map { it.asset },
            direction = DetailNavigationDirection.Previous,
        )
    }
    val nextBurstMember = remember(asset, burstMembers) {
        adjacentBurstMemberAsset(
            currentAsset = asset,
            allProjectAssets = burstMembers.map { it.asset },
            direction = DetailNavigationDirection.Next,
        )
    }
    val detailBurstPositionText = remember(asset, burstMembers) {
        photoDetailBurstPositionText(
            asset = asset,
            burstMembers = burstMembers.map { it.asset },
        )
    }
    val decision = photoDetailDecisionUi(asset, actionsEnabled)
    val hasActionCallbacks = onToggleMarked != null ||
        onSplitBurstMember != null ||
        onToggleFavorite != null ||
        onDeleteAsset != null ||
        onEvaluateModel != null
    var evaluationSubmitted by remember(asset.id) { mutableStateOf(false) }
    val evaluationInFlight = evaluationSubmitted || asset.modelEvaluationInFlight()
    LaunchedEffect(asset.id, asset.modelStatus, asset.modelScore, asset.modelSummary) {
        if (!asset.modelEvaluationInFlight()) {
            evaluationSubmitted = false
        }
    }
    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(start = 14.dp, top = 14.dp, end = 14.dp, bottom = 104.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        HeaderWithBack(
            title = asset.groupTitle(),
            subtitle = "鐓х墖璇︽儏",
            onBack = onBack,
        )
        DetailPhotoCarousel(
            asset = asset,
            previousAsset = previousBurstMember,
            nextAsset = nextBurstMember,
            positionText = detailBurstPositionText,
            imageAspectRatio = photoMetadata?.let {
                photoMetadataDisplayAspectRatio(
                    dimensions = it.dimensions,
                    orientation = it.orientation,
                )
            },
            onPrevious = previousBurstMember?.let { member -> { onOpenBurstMember?.invoke(member) } },
            onNext = nextBurstMember?.let { member -> { onOpenBurstMember?.invoke(member) } },
            onClick = onPreviewClick,
            onLongPress = onPreviewLongPress,
            modifier = Modifier.fillMaxWidth(),
        )
        if (photoDetailActionBarVisible(decision, hasActionCallbacks)) {
            decision.disabledReason?.let { reason ->
                Text(
                    reason,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                    modifier = Modifier.padding(horizontal = 22.dp),
                )
            }
            PhotoDetailDecisionActions(
                asset = asset,
                decision = decision,
                onSplitBurstMember = onSplitBurstMember,
                markedSelected = photoDetailMarkedSelected(asset),
                markedEnabled = actionsEnabled && onToggleMarked != null,
                onToggleMarked = onToggleMarked,
                favoriteSelected = photoDetailFavoriteSelected(asset),
                favoriteEnabled = actionsEnabled && onToggleFavorite != null,
                onToggleFavorite = onToggleFavorite,
                evaluateEnabled = actionsEnabled && onEvaluateModel != null,
                evaluationInFlight = evaluationInFlight,
                onEvaluateModel = onEvaluateModel?.let { evaluate ->
                    {
                        evaluationSubmitted = true
                        evaluate(asset)
                    }
                },
                onDeleteAsset = onDeleteAsset,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 42.dp),
            )
        }
        if (
            onEvaluateModel != null ||
            asset.modelScore != null ||
            asset.modelStatus != null ||
            asset.technicalGateStatus != null ||
            asset.burst != null
        ) {
            SmartSelectionDetailCard(
                asset = asset,
                modifier = Modifier
                    .fillMaxWidth()
                    .detailHorizontalSwipe(groupSwipeCallbacks),
            )
        }
        photoMetadata?.lines()?.takeIf { it.isNotEmpty() }?.let { metadataLines ->
            ElementCard(
                modifier = Modifier
                    .fillMaxWidth()
                    .detailHorizontalSwipe(groupSwipeCallbacks),
            ) {
                Column(Modifier.padding(horizontal = 14.dp, vertical = 12.dp)) {
                    Text("鎷嶆憚鍙傛暟", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    CompactDetailGrid(metadataLines)
                }
            }
        }
        ElementCard(
            modifier = Modifier
                .fillMaxWidth()
                .detailHorizontalSwipe(groupSwipeCallbacks),
        ) {
            Column(Modifier.padding(16.dp)) {
                Text("鏉ユ簮淇℃伅", style = MaterialTheme.typography.titleMedium)
                Spacer(Modifier.height(8.dp))
                CompactDetailGrid(photoDetailSourceLines(asset))
            }
        }
        ElementCard(
            modifier = Modifier
                .fillMaxWidth()
                .detailHorizontalSwipe(groupSwipeCallbacks),
        ) {
            Column(Modifier.padding(16.dp)) {
                Text("鏂囦欢", style = MaterialTheme.typography.titleMedium)
                Spacer(Modifier.height(8.dp))
                CompactDetailGrid(photoDetailFileLines(asset))
            }
        }
    }
}

private fun Modifier.detailHorizontalSwipe(
    callbacks: DetailGroupSwipeCallbacks?,
): Modifier {
    if (callbacks == null) {
        return this
    }
    return pointerInput(callbacks) {
        detectHorizontalDragGestures(
            onDragStart = {
                callbacks.onDragStart()
            },
            onDragEnd = {
                callbacks.onDragEnd()
            },
            onDragCancel = {
                callbacks.onDragCancel()
            },
        ) { _, dragAmount ->
            callbacks.onDrag(dragAmount)
        }
    }
}
@Composable
private fun CompactDetailGrid(lines: List<Pair<String, String>>) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        lines.chunked(2).forEach { row ->
            Row(
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalAlignment = Alignment.Top,
            ) {
                row.forEach { (label, value) ->
                    CompactDetailCell(
                        label = label,
                        value = value,
                        modifier = Modifier.weight(1f),
                    )
                }
                if (row.size == 1) {
                    Spacer(Modifier.weight(1f))
                }
            }
        }
    }
}

@Composable
private fun CompactDetailCell(
    label: String,
    value: String,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier) {
        Text(
            label,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            fontSize = 11.sp,
            lineHeight = 12.sp,
        )
        Spacer(Modifier.height(1.dp))
        Text(
            value,
            color = MaterialTheme.colorScheme.onSurface,
            fontSize = 13.sp,
            lineHeight = 16.sp,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

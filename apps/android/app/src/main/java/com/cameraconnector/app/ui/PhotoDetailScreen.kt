package com.cameraconnector.app.ui

import android.app.Activity
import android.content.ContentValues
import android.content.Context
import android.content.ContextWrapper
import android.content.Intent
import android.graphics.Bitmap
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.rememberTransformableState
import androidx.compose.foundation.gestures.transformable
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.AutoAwesome
import androidx.compose.material.icons.outlined.BookmarkAdd
import androidx.compose.material.icons.outlined.BookmarkAdded
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.DeleteSweep
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material.icons.outlined.KeyboardArrowUp
import androidx.compose.material.icons.outlined.PhotoLibrary
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.Share
import androidx.compose.material.icons.outlined.Star
import androidx.compose.material.icons.outlined.StarBorder
import androidx.compose.material.icons.outlined.SyncAlt
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Shapes
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.core.content.FileProvider
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import com.cameraconnector.app.core.ProjectAssetRole
import com.cameraconnector.app.core.ProjectAssetTechnicalDefect
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.PublishQueueState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.media.PREVIEW_DETAIL_FALLBACK_ASPECT_RATIO
import com.cameraconnector.app.media.PhotoMetadata
import com.cameraconnector.app.media.PreviewQuality
import com.cameraconnector.app.media.cachedPreviewBitmap
import com.cameraconnector.app.media.isDecodablePreviewLocation
import com.cameraconnector.app.media.loadCachedPreviewBitmap
import com.cameraconnector.app.media.loadPhotoMetadata
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
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

@Composable
private fun PhotoDetailExportDialog(
    exportUi: PhotoDetailExportUi,
    onDismiss: () -> Unit,
    onSave: () -> Unit,
    onShare: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("\u7167\u7247\u64cd\u4f5c") },
        text = {
            Text(
                text = exportUi.fileName,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        },
        confirmButton = {
            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                TextButton(
                    enabled = exportUi.enabled,
                    onClick = onSave,
                ) {
                    Icon(
                        imageVector = Icons.Outlined.PhotoLibrary,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                    )
                    Spacer(Modifier.width(4.dp))
                    Text("\u4fdd\u5b58")
                }
                TextButton(
                    enabled = exportUi.enabled,
                    onClick = onShare,
                ) {
                    Icon(
                        imageVector = Icons.Outlined.Share,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                    )
                    Spacer(Modifier.width(4.dp))
                    Text("\u5206\u4eab")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("\u53d6\u6d88")
            }
        },
    )
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
            subtitle = "照片详情",
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
                burstPositionText = detailBurstPositionText,
                evaluateEnabled = false,
                onEvaluateModel = null,
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
                    Text("拍摄参数", style = MaterialTheme.typography.titleMedium)
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
                Text("来源信息", style = MaterialTheme.typography.titleMedium)
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
                Text("文件", style = MaterialTheme.typography.titleMedium)
                Spacer(Modifier.height(8.dp))
                CompactDetailGrid(photoDetailFileLines(asset))
            }
        }
    }
}

@Composable
private fun SmartSelectionDetailCard(
    asset: ProjectAsset,
    burstPositionText: String?,
    evaluateEnabled: Boolean,
    onEvaluateModel: ((ProjectAsset) -> Unit)?,
    modifier: Modifier = Modifier,
) {
    CompactSmartSelectionDetailCard(
        asset = asset,
        burstPositionText = burstPositionText,
        evaluateEnabled = evaluateEnabled,
        onEvaluateModel = onEvaluateModel,
        modifier = modifier,
    )
}

@Composable
private fun CompactSmartSelectionDetailCard(
    asset: ProjectAsset,
    burstPositionText: String?,
    evaluateEnabled: Boolean,
    onEvaluateModel: ((ProjectAsset) -> Unit)?,
    modifier: Modifier = Modifier,
) {
    val score = asset.modelScoreText()
    val summary = asset.modelSummaryDisplayText()
    val technicalRisk = asset.compactTechnicalRiskText()
    val summaryExpandable = summary.length > 90
    var summaryExpanded by remember(asset.id, summary) { mutableStateOf(false) }
    ElementCard(modifier = modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(Modifier.weight(1f)) {
                    Text("\u667a\u80fd\u4f18\u9009", style = MaterialTheme.typography.titleMedium)
                }
                Row(
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    score?.let { SmartScorePill(it, asset.modelScoreColor()) }
                }
            }
            Text(
                summary,
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable(enabled = summaryExpandable) { summaryExpanded = !summaryExpanded },
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = if (summaryExpanded) Int.MAX_VALUE else 2,
                overflow = if (summaryExpanded) TextOverflow.Clip else TextOverflow.Ellipsis,
            )
            if (summaryExpandable) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { summaryExpanded = !summaryExpanded },
                    horizontalArrangement = Arrangement.End,
                ) {
                    Icon(
                        imageVector = if (summaryExpanded) {
                            Icons.Outlined.KeyboardArrowUp
                        } else {
                            Icons.Outlined.KeyboardArrowDown
                        },
                        contentDescription = if (summaryExpanded) "收起评价摘要" else "展开评价摘要",
                        tint = ElementBlue,
                        modifier = Modifier.size(18.dp),
                    )
                }
            }
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .horizontalScroll(rememberScrollState()),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                asset.compactModelStatusTag()?.let { ElementTag(it, smartBadgeColor(asset)) }
                asset.recommendationBadgeText()?.let {
                    ElementTag(it, if (asset.isBestRecommendedAsset()) ElementSuccess else ElementInfo)
                }
                asset.compactTechnicalGateTag()?.let { ElementTag(it, ElementDanger) }
            }
            technicalRisk?.let { SmartInsightLine("\u98ce\u9669", it, ElementDanger) }
        }
    }
}

@Composable
private fun SmartScorePill(
    score: String,
    color: Color,
) {
    Surface(
        color = color.copy(alpha = 0.14f),
        contentColor = color,
        shape = RoundedCornerShape(999.dp),
        border = BorderStroke(1.dp, color.copy(alpha = 0.38f)),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 9.dp, vertical = 5.dp),
            horizontalArrangement = Arrangement.spacedBy(3.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(score, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
            Text("\u5206", style = MaterialTheme.typography.labelSmall)
        }
    }
}

@Composable
private fun SmartInsightLine(
    label: String,
    value: String,
    color: Color,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            label,
            modifier = Modifier.width(34.dp),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.labelSmall,
            maxLines = 1,
        )
        Text(
            value,
            modifier = Modifier.weight(1f),
            color = color,
            style = MaterialTheme.typography.bodySmall,
            fontWeight = FontWeight.SemiBold,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

private fun ProjectAsset.compactModelStatusTag(): String? {
    val status = modelStatus?.trim()?.lowercase()
    return when {
        status in setOf("running", "processing", "analyzing", "pending", "queued", "failed", "error", "skipped") ->
            modelEvaluationStatusLabel(modelStatus)
        else -> modelTier
            ?.takeIf { it.equals("reject", ignoreCase = true) || it.equals("weak", ignoreCase = true) }
            ?.let(::modelEvaluationTierLabel)
    }
}

private fun ProjectAsset.modelSummaryDisplayText(): String =
    modelSummary
        ?.takeIf { it.isNotBlank() }
        ?.let(::smartReasonText)
        ?: when (modelStatus?.trim()?.lowercase()) {
            "running", "processing", "analyzing" -> "\u6b63\u5728\u751f\u6210\u6a21\u578b\u8bc4\u4ef7"
            "pending", "queued" -> "\u7b49\u5f85\u6a21\u578b\u8bc4\u4ef7"
            "failed", "error" -> "\u6a21\u578b\u8bc4\u4ef7\u5931\u8d25\uff0c\u53ef\u91cd\u65b0\u8bc4\u4ef7"
            "ready", "done", "completed" -> "\u6a21\u578b\u8bc4\u4ef7\u5df2\u5b8c\u6210\uff0c\u6682\u65e0\u6587\u5b57\u6458\u8981"
            else -> "\u7b49\u5f85\u6a21\u578b\u8bc4\u4ef7"
        }

private fun ProjectAsset.modelEvaluationInFlight(): Boolean =
    modelStatus?.trim()?.lowercase() in setOf(
        "pending",
        "queued",
        "running",
        "processing",
        "analyzing",
    )

private fun ProjectAsset.compactTechnicalGateTag(): String? {
    if (technicalDefects.isNotEmpty()) {
        return null
    }
    val gate = technicalRiskStatus() ?: return null
    if (!hasTechnicalRisk()) {
        return null
    }
    return technicalGateStatusLabel(gate)
}

private fun ProjectAsset.compactTechnicalRiskText(): String? {
    if (technicalDefects.isEmpty()) {
        return null
    }
    return technicalDefects
        .take(2)
        .joinToString(" / ") { defect -> defect.userFacingRiskText() }
}

private fun ProjectAssetTechnicalDefect.userFacingRiskText(): String {
    val type = defectType.trim().lowercase()
    val level = severity.trim().lowercase()
    return when (type) {
        "blur" -> when (level) {
            "severe" -> "\u4e25\u91cd\u5931\u7126"
            "high" -> "\u5931\u7126"
            "medium" -> "\u6e05\u6670\u5ea6\u504f\u8f6f"
            "low" -> "\u7ec6\u8282\u7565\u8f6f"
            else -> "\u753b\u9762\u4e0d\u591f\u6e05\u6670"
        }
        "highlight_clip" -> when (level) {
            "severe" -> "\u5927\u9762\u79ef\u8fc7\u66dd"
            "high" -> "\u8fc7\u66dd"
            "medium" -> "\u5c40\u90e8\u8fc7\u66dd"
            "low" -> "\u9ad8\u5149\u7565\u6709\u6ea2\u51fa"
            else -> "\u9ad8\u5149\u8fc7\u66dd"
        }
        "shadow_clip" -> when (level) {
            "severe" -> "\u5927\u9762\u79ef\u6b7b\u9ed1"
            "high" -> "\u6697\u90e8\u6b7b\u9ed1"
            "medium" -> "\u6697\u90e8\u7565\u6709\u6b7b\u9ed1"
            "low" -> "\u6697\u90e8\u7565\u6697"
            else -> "\u6697\u90e8\u6b7b\u9ed1"
        }
        "noise" -> when (level) {
            "severe" -> "\u9ad8\u566a\u70b9\u660e\u663e"
            "high" -> "\u566a\u70b9\u504f\u9ad8"
            "medium" -> "\u7ec6\u8282\u7565\u810f"
            "low" -> "\u8f7b\u5fae\u566a\u70b9"
            else -> "\u566a\u70b9\u504f\u9ad8"
        }
        "color_cast" -> when (level) {
            "severe" -> "\u4e25\u91cd\u504f\u8272"
            "high" -> "\u504f\u8272\u660e\u663e"
            "medium" -> "\u8272\u5f69\u504f\u8272"
            "low" -> "\u8f7b\u5fae\u504f\u8272"
            else -> "\u8272\u5f69\u504f\u8272"
        }
        "unsupported" -> "\u9700\u4eba\u5de5\u786e\u8ba4"
        else -> reason
            ?.takeIf { it.isNotBlank() }
            ?.let(::smartReasonText)
            ?: technicalDefectTypeLabel(defectType)
    }
}
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
        .contains(Regex("""90|270|转置"""))
    return if (swapsAxes) height / width else width / height
}

internal fun photoDetailSourceLines(asset: ProjectAsset): List<Pair<String, String>> =
    listOf(
        "来源" to asset.sourceLabel(),
        "账号" to (asset.username ?: "未知"),
        "原始路径" to (asset.originalPath ?: asset.displayPath),
        "接收时间" to formatEpochMillisTextForDisplay(asset.receivedAt),
        "文件大小" to (asset.sizeBytes?.let { "$it bytes" } ?: "未知"),
    )

internal fun photoDetailFileLines(asset: ProjectAsset): List<Pair<String, String>> =
    listOfNotNull(
        "位置" to asset.displayPath,
        asset.rawPath?.takeIf { it.isNotBlank() }?.let { "RAW" to it },
        asset.jpegPath?.takeIf { it.isNotBlank() }?.let { "JPEG" to it },
        asset.videoPath?.takeIf { it.isNotBlank() }?.let { "视频" to it },
    )

internal fun detailCarouselHeight(imageAspectRatio: Float?): Dp =
    when {
        imageAspectRatio == null -> 340.dp
        imageAspectRatio >= 1.2f -> 304.dp
        imageAspectRatio <= 0.82f -> 520.dp
        else -> 420.dp
    }

@Composable
private fun DetailPhotoCarousel(
    asset: ProjectAsset,
    previousAsset: ProjectAsset?,
    nextAsset: ProjectAsset?,
    positionText: String?,
    imageAspectRatio: Float?,
    onPrevious: (() -> Unit)?,
    onNext: (() -> Unit)?,
    onClick: () -> Unit,
    onLongPress: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val assetId = asset.assetSelectionId()
    var dragOffsetX by remember(assetId) { mutableFloatStateOf(0f) }
    var settleOffsetX by remember(assetId) { mutableFloatStateOf(0f) }
    var settling by remember(assetId) { mutableStateOf(false) }
    var pendingNavigationAssetId by remember(assetId) { mutableStateOf<String?>(null) }
    fun resetMotion() {
        pendingNavigationAssetId = null
        settling = false
        dragOffsetX = 0f
        settleOffsetX = 0f
    }
    LaunchedEffect(previousAsset?.previewLocation, nextAsset?.previewLocation) {
        withContext(Dispatchers.IO) {
            listOf(previousAsset, nextAsset)
                .mapNotNull { it?.previewLocation?.takeIf(::isDecodablePreviewLocation) }
                .distinct()
                .forEach { location ->
                    loadCachedPreviewBitmap(context, location, PreviewQuality.Thumbnail)
                }
        }
    }
    var previewAspectRatio by remember(assetId) {
        mutableStateOf(cachedPreviewAspectRatio(asset.previewLocation))
    }
    LaunchedEffect(assetId, imageAspectRatio) {
        imageAspectRatio?.let { previewAspectRatio = it }
    }
    val displayAspectRatio = imageAspectRatio ?: previewAspectRatio ?: PHOTO_DETAIL_LOADING_ASPECT_RATIO
    BoxWithConstraints(
        modifier = modifier
            .height(detailCarouselHeight(displayAspectRatio))
            .clip(RoundedCornerShape(18.dp)),
    ) {
        val density = LocalDensity.current
        val pageWidthPx = with(density) { maxWidth.toPx() }
        val sidePageScale = 0.9f
        val sidePeekPx = with(density) { 24.dp.toPx() }
        val sideGapPx = with(density) { 16.dp.toPx() }
        val sidePaddingPx = with(density) { 48.dp.toPx() }
        val pageStridePx = (pageWidthPx - sidePeekPx - sidePaddingPx)
            .coerceAtLeast(pageWidthPx * 0.72f + sideGapPx)
        val thresholdPx = pageStridePx * 0.18f
        val pageOffsetX = if (settling) settleOffsetX else dragOffsetX
        val mainHorizontalPadding = 24.dp
        val sideHorizontalPadding = 48.dp
        Box(
            modifier = Modifier
                .fillMaxSize()
                .pointerInput(assetId, previousAsset?.assetSelectionId(), nextAsset?.assetSelectionId()) {
                    detectHorizontalDragGestures(
                        onDragStart = {
                            pendingNavigationAssetId = null
                            settling = false
                            settleOffsetX = 0f
                        },
                        onDragEnd = {
                            val target = when {
                                dragOffsetX > thresholdPx && onPrevious != null -> DetailNavigationDirection.Previous
                                dragOffsetX < -thresholdPx && onNext != null -> DetailNavigationDirection.Next
                                else -> null
                            }
                            val releaseOffset = dragOffsetX
                            scope.launch {
                                settling = true
                                val animation = Animatable(releaseOffset)
                                when (target) {
                                    DetailNavigationDirection.Previous -> {
                                        animation.animateTo(pageStridePx, tween(durationMillis = 90)) {
                                            settleOffsetX = value
                                        }
                                        pendingNavigationAssetId = assetId
                                        onPrevious?.invoke()
                                    }
                                    DetailNavigationDirection.Next -> {
                                        animation.animateTo(-pageStridePx, tween(durationMillis = 90)) {
                                            settleOffsetX = value
                                        }
                                        pendingNavigationAssetId = assetId
                                        onNext?.invoke()
                                    }
                                    null -> animation.animateTo(0f, tween(durationMillis = 90)) {
                                        settleOffsetX = value
                                    }
                                }
                                if (target == null) {
                                    resetMotion()
                                } else {
                                    delay(240)
                                    if (pendingNavigationAssetId == assetId) {
                                        resetMotion()
                                    }
                                }
                            }
                        },
                        onDragCancel = {
                            val releaseOffset = dragOffsetX
                            scope.launch {
                                settling = true
                                Animatable(releaseOffset).animateTo(0f, tween(durationMillis = 90)) {
                                    settleOffsetX = value
                                }
                                resetMotion()
                            }
                        },
                    ) { _, dragAmount ->
                        dragOffsetX = (dragOffsetX + dragAmount).coerceIn(
                            minimumValue = if (onNext != null) -pageStridePx else 0f,
                            maximumValue = if (onPrevious != null) pageStridePx else 0f,
                        )
                    }
                },
        ) {
            previousAsset?.let { previous ->
                DetailCarouselPhotoPage(
                    asset = previous,
                    previewQuality = PreviewQuality.Thumbnail,
                    horizontalPadding = sideHorizontalPadding,
                    pageScale = sidePageScale,
                    onClick = onPrevious,
                    modifier = Modifier
                        .fillMaxSize()
                        .offset { IntOffset((pageOffsetX - pageStridePx).roundToInt(), 0) },
                )
            }
            nextAsset?.let { next ->
                DetailCarouselPhotoPage(
                    asset = next,
                    previewQuality = PreviewQuality.Thumbnail,
                    horizontalPadding = sideHorizontalPadding,
                    pageScale = sidePageScale,
                    onClick = onNext,
                    modifier = Modifier
                        .fillMaxSize()
                        .offset { IntOffset((pageOffsetX + pageStridePx).roundToInt(), 0) },
                )
            }
            DetailCarouselPhotoPage(
                asset = asset,
                previewQuality = PreviewQuality.Detail,
                horizontalPadding = mainHorizontalPadding,
                preferredAspectRatio = displayAspectRatio,
                onPreviewAspectRatio = { aspect ->
                    if (imageAspectRatio == null) {
                        previewAspectRatio = aspect
                    }
                },
                onClick = onClick,
                onLongPress = onLongPress,
                modifier = Modifier
                    .fillMaxSize()
                    .offset { IntOffset(pageOffsetX.roundToInt(), 0) },
            )
            positionText?.let {
                Surface(
                    modifier = Modifier
                        .align(Alignment.TopStart)
                        .padding(start = 12.dp, top = 8.dp),
                    color = ElementBackground.copy(alpha = 0.82f),
                    contentColor = ElementPurple,
                    shape = RoundedCornerShape(999.dp),
                    border = BorderStroke(1.dp, ElementPurple.copy(alpha = 0.46f)),
                ) {
                    Text(
                        text = it,
                        modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp),
                        fontSize = 11.sp,
                        lineHeight = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            }
        }
    }
}

@Composable
private fun DetailCarouselPhotoPage(
    asset: ProjectAsset,
    previewQuality: PreviewQuality,
    horizontalPadding: Dp,
    preferredAspectRatio: Float? = null,
    pageScale: Float = 1f,
    onClick: (() -> Unit)?,
    onLongPress: (() -> Unit)? = null,
    onPreviewAspectRatio: ((Float) -> Unit)? = null,
    onPreviewReady: (() -> Unit)? = null,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier
            .graphicsLayer {
                scaleX = pageScale
                scaleY = pageScale
            }
            .padding(horizontal = horizontalPadding),
        contentAlignment = Alignment.Center,
    ) {
        PhotoPreview(
            asset = asset,
            previewQuality = previewQuality,
            fitToImageAspect = true,
            preferredAspectRatio = preferredAspectRatio,
            contentScale = ContentScale.Fit,
            backgroundColor = ElementSurface,
            onClick = onClick,
            onLongClick = onLongPress,
            onPreviewAspectRatio = onPreviewAspectRatio,
            onPreviewReady = onPreviewReady,
            showFallbackText = false,
            modifier = Modifier.fillMaxHeight(),
        )
    }
}

@Composable
private fun PhotoDetailDecisionActions(
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
            contentDescription = "删除照片",
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
internal fun FullScreenPhotoPreview(
    asset: ProjectAsset,
    onDismiss: () -> Unit,
    onLongPress: () -> Unit = {},
) {
    BackHandler(onBack = onDismiss)
    ImmersiveSystemBars()
    var scale by remember { mutableStateOf(1f) }
    var offsetX by remember { mutableStateOf(0f) }
    var offsetY by remember { mutableStateOf(0f) }
    val transformState = rememberTransformableState { _, zoomChange, panChange, _ ->
        val nextScale = (scale * zoomChange).coerceIn(FULLSCREEN_MIN_SCALE, FULLSCREEN_MAX_SCALE)
        if (nextScale <= FULLSCREEN_MIN_SCALE) {
            scale = FULLSCREEN_MIN_SCALE
            offsetX = 0f
            offsetY = 0f
        } else {
            scale = nextScale
            offsetX += panChange.x
            offsetY += panChange.y
        }
    }
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(
            usePlatformDefaultWidth = false,
            decorFitsSystemWindows = false,
        ),
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black)
                .pointerInput(Unit) {
                    detectTapGestures(
                        onTap = { onDismiss() },
                        onLongPress = { onLongPress() },
                        onDoubleTap = {
                            if (scale > FULLSCREEN_MIN_SCALE) {
                                scale = FULLSCREEN_MIN_SCALE
                                offsetX = 0f
                                offsetY = 0f
                            } else {
                                scale = FULLSCREEN_DOUBLE_TAP_SCALE
                            }
                        },
                    )
                }
                .transformable(transformState),
            contentAlignment = Alignment.Center,
        ) {
            PhotoPreview(
                asset = asset,
                previewQuality = PreviewQuality.FullScreen,
                contentScale = ContentScale.Fit,
                backgroundColor = Color.Black,
                clipPreview = false,
                showFallbackText = false,
                modifier = Modifier
                    .fillMaxSize()
                    .graphicsLayer {
                        scaleX = scale
                        scaleY = scale
                        translationX = offsetX
                        translationY = offsetY
                    },
            )
        }
    }
}

@Composable
internal fun ImmersiveSystemBars() {
    val view = LocalView.current
    DisposableEffect(view) {
        val window = view.context.findActivity()?.window
        if (window == null) {
            onDispose { }
        } else {
            val controller = WindowCompat.getInsetsController(window, view)
            val previousBehavior = controller.systemBarsBehavior
            WindowCompat.setDecorFitsSystemWindows(window, false)
            controller.systemBarsBehavior =
                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            controller.hide(WindowInsetsCompat.Type.systemBars())
            onDispose {
                controller.show(WindowInsetsCompat.Type.systemBars())
                controller.systemBarsBehavior = previousBehavior
                WindowCompat.setDecorFitsSystemWindows(window, true)
            }
        }
    }
}

private suspend fun savePhotoDetailExportToGallery(
    context: Context,
    asset: ProjectAsset,
    exportUi: PhotoDetailExportUi,
): Boolean =
    withContext(Dispatchers.IO) {
        val bitmap = photoDetailExportBitmap(context, asset) ?: return@withContext false
        val resolver = context.contentResolver
        val values = ContentValues().apply {
            put(MediaStore.Images.Media.DISPLAY_NAME, exportUi.fileName)
            put(MediaStore.Images.Media.MIME_TYPE, PHOTO_DETAIL_EXPORT_MIME_TYPE)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                put(
                    MediaStore.Images.Media.RELATIVE_PATH,
                    "${Environment.DIRECTORY_PICTURES}/CameraConnector",
                )
                put(MediaStore.Images.Media.IS_PENDING, 1)
            }
        }
        val uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values)
            ?: return@withContext false
        runCatching {
            val saved = resolver.openOutputStream(uri)?.use { output ->
                bitmap.compress(Bitmap.CompressFormat.JPEG, PHOTO_DETAIL_EXPORT_JPEG_QUALITY, output)
            } == true
            if (!saved) {
                resolver.delete(uri, null, null)
                return@withContext false
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val publishValues = ContentValues().apply {
                    put(MediaStore.Images.Media.IS_PENDING, 0)
                }
                resolver.update(uri, publishValues, null, null)
            }
            true
        }.getOrElse {
            resolver.delete(uri, null, null)
            false
        }
    }

private suspend fun sharePhotoDetailExport(
    context: Context,
    asset: ProjectAsset,
    exportUi: PhotoDetailExportUi,
): Boolean {
    val exportFile = writePhotoDetailExportCacheFile(context, asset, exportUi)
        ?: return false
    val uri = FileProvider.getUriForFile(
        context,
        "${context.packageName}.fileprovider",
        exportFile,
    )
    val shareIntent = Intent(Intent.ACTION_SEND).apply {
        type = PHOTO_DETAIL_EXPORT_MIME_TYPE
        putExtra(Intent.EXTRA_STREAM, uri)
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    }
    val chooser = Intent.createChooser(shareIntent, "\u5206\u4eab\u7167\u7247").apply {
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        if (context.findActivity() == null) {
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
    }
    return runCatching {
        context.startActivity(chooser)
        true
    }.getOrDefault(false)
}

private suspend fun writePhotoDetailExportCacheFile(
    context: Context,
    asset: ProjectAsset,
    exportUi: PhotoDetailExportUi,
): File? =
    withContext(Dispatchers.IO) {
        val bitmap = photoDetailExportBitmap(context, asset) ?: return@withContext null
        val exportDirectory = File(context.cacheDir, PHOTO_DETAIL_EXPORT_CACHE_DIRECTORY)
        runCatching {
            exportDirectory.mkdirs()
            val exportFile = File(exportDirectory, exportUi.fileName)
            exportFile.outputStream().use { output ->
                check(bitmap.compress(Bitmap.CompressFormat.JPEG, PHOTO_DETAIL_EXPORT_JPEG_QUALITY, output))
            }
            exportFile
        }.getOrNull()
    }

private suspend fun photoDetailExportBitmap(
    context: Context,
    asset: ProjectAsset,
): Bitmap? =
    withContext(Dispatchers.IO) {
        val previewLocation = asset.previewLocation
            ?.takeIf(::isDecodablePreviewLocation)
            ?: return@withContext null
        loadCachedPreviewBitmap(context, previewLocation, PreviewQuality.FullScreen)
            ?: loadCachedPreviewBitmap(context, previewLocation, PreviewQuality.Detail)
            ?: loadCachedPreviewBitmap(context, previewLocation, PreviewQuality.Thumbnail)
    }

private fun cachedPreviewAspectRatio(previewLocation: String?): Float? =
    cachedPreviewBitmap(
        location = previewLocation.takeIf(::isDecodablePreviewLocation),
        quality = PreviewQuality.Detail,
        allowLowerQualityFallback = true,
    )?.let(::bitmapDisplayAspectRatio)

private fun bitmapDisplayAspectRatio(bitmap: Bitmap?): Float? =
    bitmap
        ?.takeIf { it.width > 0 && it.height > 0 }
        ?.let { it.width.toFloat() / it.height.toFloat() }

@OptIn(ExperimentalFoundationApi::class)
@Composable
internal fun PhotoPreview(
    asset: ProjectAsset,
    modifier: Modifier = Modifier,
    compactFallback: Boolean = false,
    previewQuality: PreviewQuality = PreviewQuality.Thumbnail,
    fitToImageAspect: Boolean = false,
    preferredAspectRatio: Float? = null,
    contentScale: ContentScale = ContentScale.Crop,
    backgroundColor: Color = ElementPanel,
    clipPreview: Boolean = true,
    trimLetterbox: Boolean = false,
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    onPreviewAspectRatio: ((Float) -> Unit)? = null,
    onPreviewReady: (() -> Unit)? = null,
    showFallbackText: Boolean = true,
) {
    val context = LocalContext.current
    val previewLocation = asset.previewLocation.takeIf(::isDecodablePreviewLocation)
    val initialBitmap = remember(previewLocation, previewQuality) {
        cachedPreviewBitmap(
            location = previewLocation,
            quality = previewQuality,
            allowLowerQualityFallback = true,
        )
    }
    var bitmap by remember(previewLocation, previewQuality) { mutableStateOf(initialBitmap) }
    LaunchedEffect(previewLocation, previewQuality) {
        if (previewLocation == null) {
            bitmap = null
            return@LaunchedEffect
        }
        val exactCached = cachedPreviewBitmap(previewLocation, previewQuality)
        if (exactCached != null) {
            bitmap = exactCached
            bitmapDisplayAspectRatio(exactCached)?.let { onPreviewAspectRatio?.invoke(it) }
            onPreviewReady?.invoke()
            return@LaunchedEffect
        }
        cachedPreviewBitmap(
            location = previewLocation,
            quality = previewQuality,
            allowLowerQualityFallback = true,
        )?.let { fallbackBitmap ->
            bitmap = fallbackBitmap
            bitmapDisplayAspectRatio(fallbackBitmap)?.let { onPreviewAspectRatio?.invoke(it) }
        }
        if (previewQuality != PreviewQuality.Thumbnail && bitmap == null) {
            withContext(Dispatchers.IO) {
                loadCachedPreviewBitmap(context, previewLocation, PreviewQuality.Thumbnail)
            }?.let { thumbnailBitmap ->
                bitmap = thumbnailBitmap
                bitmapDisplayAspectRatio(thumbnailBitmap)?.let { onPreviewAspectRatio?.invoke(it) }
            }
        }
        withContext(Dispatchers.IO) {
            loadCachedPreviewBitmap(context, previewLocation, previewQuality)
        }?.let { loadedBitmap ->
            bitmap = loadedBitmap
            bitmapDisplayAspectRatio(loadedBitmap)?.let { onPreviewAspectRatio?.invoke(it) }
            onPreviewReady?.invoke()
        }
    }

    val loadedBitmap = bitmap
    val displayBitmap = remember(loadedBitmap, trimLetterbox) {
        if (trimLetterbox && loadedBitmap != null) {
            loadedBitmap.trimNearBlackLetterbox() ?: loadedBitmap
        } else {
            loadedBitmap
        }
    }
    val showTextLoadingFallback = loadedBitmap == null &&
        (previewQuality == PreviewQuality.Thumbnail || compactFallback)
    val aspectModifier = if (fitToImageAspect) {
        val imageAspectRatio = preferredAspectRatio
            ?: bitmapDisplayAspectRatio(loadedBitmap)
            ?: PREVIEW_DETAIL_FALLBACK_ASPECT_RATIO
        modifier.aspectRatio(imageAspectRatio)
    } else {
        modifier
    }
    val previewModifier = if (clipPreview) {
        aspectModifier.clip(elementShape)
    } else {
        aspectModifier
    }
    val clickableModifier = if (onClick == null && onLongClick == null) {
        previewModifier
    } else {
        previewModifier.combinedClickable(
            onClick = onClick ?: {},
            onLongClick = onLongClick,
        )
    }
    Box(
        modifier = clickableModifier.background(backgroundColor),
        contentAlignment = Alignment.Center,
    ) {
        if (displayBitmap != null) {
            Image(
                bitmap = displayBitmap.asImageBitmap(),
                contentDescription = asset.groupTitle(),
                modifier = Modifier.fillMaxSize(),
                contentScale = contentScale,
                filterQuality = FilterQuality.High,
            )
        } else if (showFallbackText) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                if (compactFallback || !showTextLoadingFallback) {
                    Text(
                        asset.formatBadges(),
                        color = ElementInfo,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                } else {
                    Text(
                        "\u52a0\u8f7d\u4e2d",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(asset.formatBadges(), color = ElementInfo)
                }
            }
        }
    }
}

private fun Bitmap.trimNearBlackLetterbox(): Bitmap? {
    val width = width
    val height = height
    if (width < 12 || height < 12) {
        return null
    }
    fun isMostlyBlackRow(y: Int): Boolean {
        var black = 0
        for (x in 0 until width) {
            if (pixelLuma(getPixel(x, y)) <= 14) black += 1
        }
        return black >= width * 0.9f
    }
    fun isMostlyBlackColumn(x: Int): Boolean {
        var black = 0
        for (y in 0 until height) {
            if (pixelLuma(getPixel(x, y)) <= 14) black += 1
        }
        return black >= height * 0.9f
    }
    var left = 0
    var right = width - 1
    var top = 0
    var bottom = height - 1
    while (left < right && isMostlyBlackColumn(left)) left += 1
    while (right > left && isMostlyBlackColumn(right)) right -= 1
    while (top < bottom && isMostlyBlackRow(top)) top += 1
    while (bottom > top && isMostlyBlackRow(bottom)) bottom -= 1
    val cropWidth = right - left + 1
    val cropHeight = bottom - top + 1
    if (cropWidth >= width - 2 && cropHeight >= height - 2) {
        return null
    }
    if (cropWidth < width * 0.72f || cropHeight < height * 0.72f) {
        return null
    }
    return runCatching {
        Bitmap.createBitmap(this, left, top, cropWidth, cropHeight)
    }.getOrNull()
}

private fun pixelLuma(pixel: Int): Int {
    val red = pixel shr 16 and 0xff
    val green = pixel shr 8 and 0xff
    val blue = pixel and 0xff
    return ((red * 299) + (green * 587) + (blue * 114)) / 1000
}

internal tailrec fun Context.findActivity(): Activity? {
    return when (this) {
        is Activity -> this
        is ContextWrapper -> baseContext.findActivity()
        else -> null
    }
}

internal const val FULLSCREEN_MIN_SCALE = 1f
internal const val FULLSCREEN_DOUBLE_TAP_SCALE = 2.5f
internal const val FULLSCREEN_MAX_SCALE = 5f
private const val PHOTO_DETAIL_LOADING_ASPECT_RATIO = 0.67f
private const val PHOTO_DETAIL_EXPORT_MIME_TYPE = "image/jpeg"
private const val PHOTO_DETAIL_EXPORT_JPEG_QUALITY = 95
private const val PHOTO_DETAIL_EXPORT_CACHE_DIRECTORY = "photo_exports"

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

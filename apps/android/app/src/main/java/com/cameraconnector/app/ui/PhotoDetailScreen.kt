package com.cameraconnector.app.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.graphics.Bitmap
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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.BookmarkAdd
import androidx.compose.material.icons.outlined.BookmarkAdded
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.DeleteSweep
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.PhotoLibrary
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.Star
import androidx.compose.material.icons.outlined.StarBorder
import androidx.compose.material.icons.outlined.SyncAlt
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Shapes
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
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
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetQuery
import com.cameraconnector.app.core.InboxAssetRole
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
import kotlin.math.abs
import kotlin.math.roundToInt

@Composable
internal fun PhotoDetailScreen(
    asset: InboxAsset,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
    actionsEnabled: Boolean = true,
    onSplitBurstMember: ((String, String) -> Unit)? = null,
    burstMembers: List<BurstMemberFilmstripItemUi> = emptyList(),
    onOpenBurstMember: ((InboxAsset) -> Unit)? = null,
    onNavigatePreviousGroup: (() -> Unit)? = null,
    onNavigateNextGroup: (() -> Unit)? = null,
    onToggleMarked: ((String) -> Unit)? = null,
    onToggleFavorite: ((String) -> Unit)? = null,
    onDeleteAsset: ((String) -> Unit)? = null,
) {
    var fullScreenPreview by remember { mutableStateOf(false) }
    if (fullScreenPreview) {
        FullScreenPhotoPreview(
            asset = asset,
            onDismiss = { fullScreenPreview = false },
        )
    }
    PhotoDetailContent(
        asset = asset,
        onBack = onBack,
        modifier = modifier,
        actionsEnabled = actionsEnabled,
        onSplitBurstMember = onSplitBurstMember,
        burstMembers = burstMembers,
        onOpenBurstMember = onOpenBurstMember,
        onNavigatePreviousGroup = onNavigatePreviousGroup,
        onNavigateNextGroup = onNavigateNextGroup,
        onToggleMarked = onToggleMarked,
        onToggleFavorite = onToggleFavorite,
        onDeleteAsset = onDeleteAsset,
        onPreviewClick = { fullScreenPreview = true },
    )
}

@Composable
private fun PhotoDetailContent(
    asset: InboxAsset,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
    actionsEnabled: Boolean,
    onSplitBurstMember: ((String, String) -> Unit)?,
    burstMembers: List<BurstMemberFilmstripItemUi>,
    onOpenBurstMember: ((InboxAsset) -> Unit)?,
    onNavigatePreviousGroup: (() -> Unit)?,
    onNavigateNextGroup: (() -> Unit)?,
    onToggleMarked: ((String) -> Unit)?,
    onToggleFavorite: ((String) -> Unit)?,
    onDeleteAsset: ((String) -> Unit)?,
    onPreviewClick: () -> Unit,
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
        onDeleteAsset != null
    Column(
        modifier = modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
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
            onPrevious = previousBurstMember?.let { member -> { onOpenBurstMember?.invoke(member) } },
            onNext = nextBurstMember?.let { member -> { onOpenBurstMember?.invoke(member) } },
            onClick = onPreviewClick,
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
                onDeleteAsset = onDeleteAsset,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 42.dp),
            )
        }
        if (asset.quality != null || asset.burst != null) {
            SmartSelectionDetailCard(
                asset = asset,
                burstPositionText = detailBurstPositionText,
                modifier = Modifier
                    .fillMaxWidth()
                    .detailHorizontalSwipe(
                        onSwipePrevious = onNavigatePreviousGroup,
                        onSwipeNext = onNavigateNextGroup,
                    ),
            )
        }
        photoMetadata?.lines()?.takeIf { it.isNotEmpty() }?.let { metadataLines ->
            ElementCard(
                modifier = Modifier
                    .fillMaxWidth()
                    .detailHorizontalSwipe(
                        onSwipePrevious = onNavigatePreviousGroup,
                        onSwipeNext = onNavigateNextGroup,
                    ),
            ) {
                Column(Modifier.padding(16.dp)) {
                    Text("拍摄参数", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    metadataLines.forEach { (label, value) ->
                        DetailLine(label, value)
                    }
                }
            }
        }
        ElementCard(
            modifier = Modifier
                .fillMaxWidth()
                .detailHorizontalSwipe(
                    onSwipePrevious = onNavigatePreviousGroup,
                    onSwipeNext = onNavigateNextGroup,
                ),
        ) {
            Column(Modifier.padding(16.dp)) {
                Text("来源信息", style = MaterialTheme.typography.titleMedium)
                Spacer(Modifier.height(8.dp))
                DetailLine("来源", asset.sourceLabel())
                DetailLine("账号", asset.username ?: "未记录")
                DetailLine("原始路径", asset.originalPath ?: asset.displayPath)
                DetailLine("接收时间", formatEpochMillisTextForDisplay(asset.receivedAt))
                DetailLine("文件大小", asset.sizeBytes?.let { "$it bytes" } ?: "未记录")
            }
        }
        ElementCard(
            modifier = Modifier
                .fillMaxWidth()
                .detailHorizontalSwipe(
                    onSwipePrevious = onNavigatePreviousGroup,
                    onSwipeNext = onNavigateNextGroup,
                ),
        ) {
            Column(Modifier.padding(16.dp)) {
                Text("文件组", style = MaterialTheme.typography.titleMedium)
                Spacer(Modifier.height(8.dp))
                DetailLine("主文件", asset.displayPath)
                DetailLine("RAW", asset.rawPath ?: "无")
                DetailLine("JPEG", asset.jpegPath ?: "无")
                DetailLine("视频", asset.videoPath ?: "无")
            }
        }
    }
}

@Composable
private fun SmartSelectionDetailCard(
    asset: InboxAsset,
    burstPositionText: String?,
    modifier: Modifier = Modifier,
) {
    val quality = asset.quality
    val burst = asset.burst
    val score = asset.qualityScoreText()
    ElementCard(modifier = modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.Top,
            ) {
                Column(Modifier.weight(1f)) {
                    Text("智能优选", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(4.dp))
                    Text(
                        asset.qualityReasonText() ?: qualityStatusLabel(quality?.analysisStatus),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                score?.let {
                    Surface(
                        color = smartBadgeColor(asset).copy(alpha = 0.14f),
                        contentColor = smartBadgeColor(asset),
                        shape = RoundedCornerShape(12.dp),
                        border = BorderStroke(1.dp, smartBadgeColor(asset).copy(alpha = 0.38f)),
                    ) {
                        Column(
                            modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp),
                            horizontalAlignment = Alignment.CenterHorizontally,
                        ) {
                            Text(it, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
                            Text("模型分", style = MaterialTheme.typography.labelSmall)
                        }
                    }
                }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                asset.qualityBadgeText()?.let { ElementTag(it, smartBadgeColor(asset)) }
                asset.groupBestBadgeText()?.let { ElementTag(it, ElementWarning) }
                burstPositionText?.let { ElementTag(it, ElementPurple) }
                asset.recommendationBadgeText()?.let {
                    ElementTag(it, if (asset.isBestRecommendedAsset()) ElementSuccess else ElementInfo)
                }
            }
            asset.modelStatus?.let { DetailLine("模型状态", it) }
            asset.modelTier?.let { DetailLine("模型档位", it) }
            asset.modelEvaluatorKind?.let { DetailLine("评价来源", modelEvaluationSourceLabel(it)) }
            asset.technicalGateStatus?.let { DetailLine("技术门禁", it) }
            asset.technicalDefects.takeIf { it.isNotEmpty() }?.let { defects ->
                DetailLine(
                    "技术风险",
                    defects.joinToString(" / ") { defect ->
                        listOfNotNull(defect.defectType, defect.severity, defect.reason)
                            .filter { value -> value.isNotBlank() }
                            .joinToString(":")
                    },
                )
            }
            quality?.let {
                DetailLine("评价状态", qualityStatusLabel(it.analysisStatus))
                asset.qualitySignalRows().takeIf { rows -> rows.isNotEmpty() }?.let { rows ->
                    QualitySignalStrip(rows = rows)
                }
                it.scorerVersion?.takeIf { version -> version.isNotBlank() }?.let { version ->
                    DetailLine("评价版本", version)
                }
                it.analyzedAtMs?.let { analyzedAtMs ->
                    DetailLine("分析时间", formatEpochMillisForDisplay(analyzedAtMs))
                }
            }
            burst?.let {
                DetailLine(
                    "连拍分组",
                    burstPositionText ?: "1/${it.memberCount}",
                )
                asset.groupBestScoreText()?.let { bestScore ->
                    DetailLine("组内参考分", bestScore)
                }
                DetailLine("推荐状态", recommendationStatusLabel(it.recommendationStatus))
                it.bestAssetGroupId?.takeIf { bestId -> bestId.isNotBlank() }?.let { bestId ->
                    DetailLine("当前优选", if (asset.isBestRecommendedAsset()) "当前照片" else bestId)
                }
            }
        }
    }
}

@Composable
private fun DetailPhotoCarousel(
    asset: InboxAsset,
    previousAsset: InboxAsset?,
    nextAsset: InboxAsset?,
    onPrevious: (() -> Unit)?,
    onNext: (() -> Unit)?,
    onClick: () -> Unit,
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
    BoxWithConstraints(
        modifier = modifier
            .height(420.dp)
            .clip(RoundedCornerShape(18.dp)),
    ) {
        val density = LocalDensity.current
        val pageWidthPx = with(density) { maxWidth.toPx() }
        val sidePeekPx = with(density) { 24.dp.toPx() }
        val sideGapPx = with(density) { 16.dp.toPx() }
        val sidePaddingPx = with(density) { 48.dp.toPx() }
        val pageStridePx = (pageWidthPx - sidePeekPx - sidePaddingPx)
            .coerceAtLeast(pageWidthPx * 0.72f + sideGapPx)
        val thresholdPx = pageStridePx * 0.18f
        val pageOffsetX = if (settling) settleOffsetX else dragOffsetX
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
                    horizontalPadding = 48.dp,
                    pageScale = 0.9f,
                    onClick = onPrevious,
                    modifier = Modifier
                        .fillMaxSize()
                        .offset { IntOffset((pageOffsetX - pageStridePx).roundToInt(), 0) },
                )
            }
            DetailCarouselPhotoPage(
                asset = asset,
                previewQuality = PreviewQuality.Detail,
                horizontalPadding = 40.dp,
                onClick = onClick,
                modifier = Modifier
                    .fillMaxSize()
                    .offset { IntOffset(pageOffsetX.roundToInt(), 0) },
            )
            nextAsset?.let { next ->
                DetailCarouselPhotoPage(
                    asset = next,
                    previewQuality = PreviewQuality.Thumbnail,
                    horizontalPadding = 48.dp,
                    pageScale = 0.9f,
                    onClick = onNext,
                    modifier = Modifier
                        .fillMaxSize()
                        .offset { IntOffset((pageOffsetX + pageStridePx).roundToInt(), 0) },
                )
            }
        }
    }
}

@Composable
private fun DetailCarouselPhotoPage(
    asset: InboxAsset,
    previewQuality: PreviewQuality,
    horizontalPadding: Dp,
    pageScale: Float = 1f,
    onClick: (() -> Unit)?,
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
            contentScale = ContentScale.Fit,
            backgroundColor = ElementSurface,
            onClick = onClick,
            onPreviewReady = onPreviewReady,
            showFallbackText = false,
            modifier = Modifier.fillMaxHeight(),
        )
    }
}

@Composable
private fun PhotoDetailDecisionActions(
    asset: InboxAsset,
    decision: PhotoDetailDecisionUi,
    onSplitBurstMember: ((String, String) -> Unit)?,
    markedSelected: Boolean,
    markedEnabled: Boolean,
    onToggleMarked: ((String) -> Unit)?,
    favoriteSelected: Boolean,
    favoriteEnabled: Boolean,
    onToggleFavorite: ((String) -> Unit)?,
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
            contentDescription = if (markedSelected) "已标记" else "标记",
            tint = ElementBlue,
            enabled = markedEnabled,
            onClick = {
                onToggleMarked?.invoke(asset.assetSelectionId())
            },
        )
        PhotoDetailIconAction(
            icon = if (favoriteSelected) Icons.Outlined.Star else Icons.Outlined.StarBorder,
            contentDescription = if (favoriteSelected) "已收藏" else "收藏",
            tint = ElementSuccess,
            enabled = favoriteEnabled,
            onClick = {
                onToggleFavorite?.invoke(asset.assetSelectionId())
            },
        )
        PhotoDetailIconAction(
            icon = Icons.Outlined.DeleteSweep,
            contentDescription = "移出连拍组，不删除文件",
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
        Box(contentAlignment = Alignment.Center) {
            Icon(
                imageVector = icon,
                contentDescription = contentDescription,
                modifier = Modifier.size(22.dp),
            )
        }
    }
}

private fun Modifier.detailHorizontalSwipe(
    onSwipePrevious: (() -> Unit)?,
    onSwipeNext: (() -> Unit)?,
): Modifier {
    if (onSwipePrevious == null && onSwipeNext == null) {
        return this
    }
    return pointerInput(onSwipePrevious, onSwipeNext) {
        var totalX = 0f
        detectHorizontalDragGestures(
            onDragStart = {
                totalX = 0f
            },
            onDragEnd = {
                if (kotlin.math.abs(totalX) > 90f) {
                    if (totalX > 0f) {
                        onSwipePrevious?.invoke()
                    } else {
                        onSwipeNext?.invoke()
                    }
                }
                totalX = 0f
            },
            onDragCancel = {
                totalX = 0f
            },
        ) { _, dragAmount ->
            totalX += dragAmount
        }
    }
}

@Composable
private fun DetailBurstMemberFilmstrip(
    memberItems: List<BurstMemberFilmstripItemUi>,
    currentAssetId: String,
    onOpenBurstMember: ((InboxAsset) -> Unit)?,
) {
    LazyRow(
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        contentPadding = PaddingValues(vertical = 2.dp),
    ) {
        items(memberItems, key = { it.asset.assetSelectionId() }) { item ->
            val isCurrent = item.asset.assetSelectionId() == currentAssetId
            Surface(
                modifier = Modifier.width(112.dp),
                color = ElementControlSurface,
                contentColor = MaterialTheme.colorScheme.onSurface,
                shape = RoundedCornerShape(12.dp),
                border = BorderStroke(1.dp, if (isCurrent) ElementBlue else ElementBorder),
            ) {
                Column(
                    modifier = Modifier.padding(6.dp),
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    Box {
                        PhotoPreview(
                            asset = item.asset,
                            compactFallback = true,
                            modifier = Modifier
                                .fillMaxWidth()
                                .aspectRatio(1f),
                            onClick = onOpenBurstMember?.let { openMember ->
                                { openMember(item.asset) }
                            },
                        )
                    }
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        ElementTag(item.badgeText, burstMemberDetailBadgeColor(item.badgeText))
                        item.scoreText?.let { score ->
                            Text(
                                score,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                style = MaterialTheme.typography.labelSmall,
                                fontWeight = FontWeight.Bold,
                            )
                        }
                    }
                    Text(
                        item.asset.groupTitle(),
                        style = MaterialTheme.typography.labelSmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }
    }
}

private fun burstMemberDetailBadgeColor(text: String): Color =
    when (text) {
        "最佳" -> ElementSuccess
        "当前" -> ElementBlue
        "低分" -> ElementWarning
        "需复核" -> ElementWarning
        else -> ElementInfo
    }

@Composable
internal fun BurstComparisonDialog(
    items: List<BurstMemberFilmstripItemUi>,
    onDismiss: () -> Unit,
) {
    Dialog(onDismissRequest = onDismiss) {
        ElementCard(modifier = Modifier.fillMaxWidth()) {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Column(Modifier.weight(1f)) {
                        Text("组内对比", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(2.dp))
                        Text(
                            "当前 / 优选 / 高分备选",
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            style = MaterialTheme.typography.bodySmall,
                        )
                    }
                    OutlinedButton(
                        onClick = onDismiss,
                        shape = RoundedCornerShape(12.dp),
                        border = BorderStroke(1.dp, ElementBorder),
                        contentPadding = PaddingValues(horizontal = 12.dp, vertical = 0.dp),
                    ) {
                        Text("关闭")
                    }
                }
                LazyRow(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    items(items, key = { it.asset.assetSelectionId() }) { item ->
                        BurstComparisonCandidateCard(item)
                    }
                }
            }
        }
    }
}

@Composable
private fun BurstComparisonCandidateCard(item: BurstMemberFilmstripItemUi) {
    Surface(
        modifier = Modifier.width(180.dp),
        color = ElementControlSurface,
        contentColor = MaterialTheme.colorScheme.onSurface,
        shape = RoundedCornerShape(12.dp),
        border = BorderStroke(1.dp, ElementBorder),
    ) {
        Column(
            modifier = Modifier.padding(8.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            PhotoPreview(
                asset = item.asset,
                previewQuality = PreviewQuality.Detail,
                contentScale = ContentScale.Fit,
                compactFallback = true,
                backgroundColor = Color.Black,
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(1f),
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                ElementTag(item.badgeText, burstMemberDetailBadgeColor(item.badgeText))
                item.scoreText?.let { score ->
                    Text(
                        score,
                        color = smartBadgeColor(item.asset),
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                    )
                }
            }
            Text(
                item.asset.groupTitle(),
                style = MaterialTheme.typography.labelMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            item.asset.qualitySignalRows().take(3).forEach { row ->
                DetailLine(row.label, row.value)
            }
        }
    }
}

@Composable
private fun QualitySignalStrip(rows: List<QualitySignalRow>) {
    LazyRow(
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        contentPadding = PaddingValues(vertical = 2.dp),
    ) {
        items(rows) { row ->
            Surface(
                color = ElementControlSurface,
                contentColor = MaterialTheme.colorScheme.onSurface,
                shape = RoundedCornerShape(10.dp),
                border = BorderStroke(1.dp, ElementBorder),
            ) {
                Column(
                    modifier = Modifier.padding(horizontal = 10.dp, vertical = 7.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Text(
                        row.value,
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                    )
                    Text(
                        row.label,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.labelSmall,
                        maxLines = 1,
                    )
                }
            }
        }
    }
}

@Composable
internal fun FullScreenPhotoPreview(
    asset: InboxAsset,
    onDismiss: () -> Unit,
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

@Composable
internal fun PhotoPreview(
    asset: InboxAsset,
    modifier: Modifier = Modifier,
    compactFallback: Boolean = false,
    previewQuality: PreviewQuality = PreviewQuality.Thumbnail,
    fitToImageAspect: Boolean = false,
    contentScale: ContentScale = ContentScale.Crop,
    backgroundColor: Color = ElementPanel,
    clipPreview: Boolean = true,
    onClick: (() -> Unit)? = null,
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
            onPreviewReady?.invoke()
            return@LaunchedEffect
        }
        cachedPreviewBitmap(
            location = previewLocation,
            quality = previewQuality,
            allowLowerQualityFallback = true,
        )?.let { fallbackBitmap ->
            bitmap = fallbackBitmap
        }
        if (previewQuality != PreviewQuality.Thumbnail && bitmap == null) {
            withContext(Dispatchers.IO) {
                loadCachedPreviewBitmap(context, previewLocation, PreviewQuality.Thumbnail)
            }?.let { thumbnailBitmap ->
                bitmap = thumbnailBitmap
            }
        }
        withContext(Dispatchers.IO) {
            loadCachedPreviewBitmap(context, previewLocation, previewQuality)
        }?.let { loadedBitmap ->
            bitmap = loadedBitmap
            onPreviewReady?.invoke()
        }
    }

    val loadedBitmap = bitmap
    val showTextLoadingFallback = loadedBitmap == null &&
        (previewQuality == PreviewQuality.Thumbnail || compactFallback)
    val aspectModifier = if (fitToImageAspect) {
        val imageAspectRatio = loadedBitmap
            ?.takeIf { it.width > 0 && it.height > 0 }
            ?.let { it.width.toFloat() / it.height.toFloat() }
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
    val clickableModifier = if (onClick == null) {
        previewModifier
    } else {
        previewModifier.clickable(onClick = onClick)
    }
    Box(
        modifier = clickableModifier.background(backgroundColor),
        contentAlignment = Alignment.Center,
    ) {
        if (loadedBitmap != null) {
            Image(
                bitmap = loadedBitmap.asImageBitmap(),
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
                        "加载中",
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

@Composable
internal fun DetailLine(label: String, value: String) {
    Column(Modifier.padding(vertical = 4.dp)) {
        Text(label, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(value, style = MaterialTheme.typography.bodyLarge)
    }
}

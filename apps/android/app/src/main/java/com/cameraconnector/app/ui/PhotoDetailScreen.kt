package com.cameraconnector.app.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.graphics.Bitmap
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.rememberTransformableState
import androidx.compose.foundation.gestures.transformable
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
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
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.PhotoLibrary
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material.icons.outlined.Settings
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
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
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
import com.cameraconnector.app.media.cacheThumbnailPreview
import com.cameraconnector.app.media.cachedThumbnailPreview
import com.cameraconnector.app.media.isDecodablePreviewLocation
import com.cameraconnector.app.media.loadPhotoMetadata
import com.cameraconnector.app.media.loadPreviewBitmap
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
internal fun PhotoDetailScreen(
    asset: InboxAsset,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
    actionsEnabled: Boolean = true,
    onAcceptRecommendedBest: ((String) -> Unit)? = null,
    onOverrideRecommendedBest: ((String, String) -> Unit)? = null,
    onMarkBurstNeedsReview: ((String) -> Unit)? = null,
    onRestoreAutomaticRecommendation: ((String) -> Unit)? = null,
    onSplitBurstMember: ((String, String) -> Unit)? = null,
    burstMembers: List<BurstMemberFilmstripItemUi> = emptyList(),
    comparisonItems: List<BurstMemberFilmstripItemUi> = emptyList(),
    onOpenBurstMember: ((InboxAsset) -> Unit)? = null,
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
    var fullScreenPreview by remember { mutableStateOf(false) }
    var burstComparisonOpen by remember(asset.assetSelectionId()) { mutableStateOf(false) }
    if (fullScreenPreview) {
        FullScreenPhotoPreview(
            asset = asset,
            onDismiss = { fullScreenPreview = false },
        )
    }
    if (burstComparisonOpen && comparisonItems.size > 1) {
        BurstComparisonDialog(
            items = comparisonItems,
            onDismiss = { burstComparisonOpen = false },
        )
    }

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
        PhotoPreview(
            asset = asset,
            previewQuality = PreviewQuality.Detail,
            fitToImageAspect = true,
            contentScale = ContentScale.Fit,
            backgroundColor = Color.Black,
            onClick = { fullScreenPreview = true },
            modifier = Modifier.fillMaxWidth(),
        )
        if (asset.quality != null || asset.burst != null) {
            SmartSelectionDetailCard(
                asset = asset,
                actionsEnabled = actionsEnabled,
                onAcceptRecommendedBest = onAcceptRecommendedBest,
                onOverrideRecommendedBest = onOverrideRecommendedBest,
                onMarkBurstNeedsReview = onMarkBurstNeedsReview,
                onRestoreAutomaticRecommendation = onRestoreAutomaticRecommendation,
                onSplitBurstMember = onSplitBurstMember,
                burstMembers = burstMembers,
                comparisonItems = comparisonItems,
                onOpenComparison = { burstComparisonOpen = true },
                onOpenBurstMember = onOpenBurstMember,
            )
        }
        photoMetadata?.lines()?.takeIf { it.isNotEmpty() }?.let { metadataLines ->
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("拍摄参数", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    metadataLines.forEach { (label, value) ->
                        DetailLine(label, value)
                    }
                }
            }
        }
        ElementCard(modifier = Modifier.fillMaxWidth()) {
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
        ElementCard(modifier = Modifier.fillMaxWidth()) {
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
    actionsEnabled: Boolean,
    onAcceptRecommendedBest: ((String) -> Unit)?,
    onOverrideRecommendedBest: ((String, String) -> Unit)?,
    onMarkBurstNeedsReview: ((String) -> Unit)?,
    onRestoreAutomaticRecommendation: ((String) -> Unit)?,
    onSplitBurstMember: ((String, String) -> Unit)?,
    burstMembers: List<BurstMemberFilmstripItemUi>,
    comparisonItems: List<BurstMemberFilmstripItemUi>,
    onOpenComparison: () -> Unit,
    onOpenBurstMember: ((InboxAsset) -> Unit)?,
) {
    val quality = asset.quality
    val burst = asset.burst
    val score = asset.qualityScoreText()
    val decision = photoDetailDecisionUi(asset, actionsEnabled)
    ElementCard(modifier = Modifier.fillMaxWidth()) {
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
                            Text("评分", style = MaterialTheme.typography.labelSmall)
                        }
                    }
                }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                asset.qualityBadgeText()?.let { ElementTag(it, smartBadgeColor(asset)) }
                asset.groupBestBadgeText()?.let { ElementTag(it, ElementWarning) }
                asset.burstBadgeText()?.let { ElementTag(it, ElementPurple) }
                asset.recommendationBadgeText()?.let {
                    ElementTag(it, if (asset.isBestRecommendedAsset()) ElementSuccess else ElementInfo)
                }
            }
            quality?.let {
                DetailLine("评分状态", qualityStatusLabel(it.analysisStatus))
                asset.qualitySignalRows().takeIf { rows -> rows.isNotEmpty() }?.let { rows ->
                    QualitySignalStrip(rows = rows)
                }
                it.scorerVersion?.takeIf { version -> version.isNotBlank() }?.let { version ->
                    DetailLine("评分版本", version)
                }
                it.analyzedAtMs?.let { analyzedAtMs ->
                    DetailLine("分析时间", formatEpochMillisForDisplay(analyzedAtMs))
                }
            }
            burst?.let {
                DetailLine(
                    "连拍分组",
                    asset.burstPositionBadgeText() ?: it.memberCount.toString(),
                )
                asset.groupBestScoreText()?.let { bestScore ->
                    DetailLine("组内最高评分", bestScore)
                }
                DetailLine("推荐状态", recommendationStatusLabel(it.recommendationStatus))
                it.bestAssetGroupId?.takeIf { bestId -> bestId.isNotBlank() }?.let { bestId ->
                    DetailLine("当前优选", if (asset.isBestRecommendedAsset()) "当前照片" else bestId)
                }
            }
            if (burstMembers.isNotEmpty()) {
                Text("组内照片", style = MaterialTheme.typography.titleSmall)
                DetailBurstMemberFilmstrip(
                    memberItems = burstMembers,
                    currentAssetId = asset.assetSelectionId(),
                    onOpenBurstMember = onOpenBurstMember,
                )
                if (comparisonItems.size > 1) {
                    OutlinedButton(
                        onClick = onOpenComparison,
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(12.dp),
                        border = BorderStroke(1.dp, ElementBlue.copy(alpha = 0.45f)),
                        colors = ButtonDefaults.outlinedButtonColors(
                            containerColor = ElementBlueSoft.copy(alpha = 0.5f),
                            contentColor = ElementBlue,
                        ),
                        contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                    ) {
                        Text("对比组内照片", maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                }
            }
            if (
                decision.hasAnyAction &&
                (
                    onAcceptRecommendedBest != null ||
                        onOverrideRecommendedBest != null ||
                        onMarkBurstNeedsReview != null ||
                        onRestoreAutomaticRecommendation != null ||
                        onSplitBurstMember != null
                    )
            ) {
                decision.disabledReason?.let { reason ->
                    Text(
                        reason,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    decision.acceptRecommendedBestBurstGroupId
                        ?.takeIf { onAcceptRecommendedBest != null }
                        ?.let { burstGroupId ->
                            Button(
                                onClick = { onAcceptRecommendedBest?.invoke(burstGroupId) },
                                enabled = decision.acceptRecommendedBestEnabled,
                                modifier = Modifier.fillMaxWidth(),
                                shape = RoundedCornerShape(12.dp),
                                colors = ButtonDefaults.buttonColors(
                                    containerColor = ElementSuccess,
                                    contentColor = ElementOnAccent,
                                ),
                                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                            ) {
                                Text("接受推荐", maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                        }
                    decision.overrideRecommendedBestTarget
                        ?.takeIf { onOverrideRecommendedBest != null }
                        ?.let { target ->
                            Button(
                                onClick = {
                                    onOverrideRecommendedBest?.invoke(
                                        target.burstGroupId,
                                        target.bestAssetGroupId,
                                    )
                                },
                                enabled = decision.overrideRecommendedBestEnabled,
                                modifier = Modifier.fillMaxWidth(),
                                shape = RoundedCornerShape(12.dp),
                                colors = ButtonDefaults.buttonColors(
                                    containerColor = ElementSuccess,
                                    contentColor = ElementOnAccent,
                                ),
                                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                            ) {
                                Text("设为优选", maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                        }
                    decision.markNeedsReviewBurstGroupId
                        ?.takeIf { onMarkBurstNeedsReview != null }
                        ?.let { burstGroupId ->
                            OutlinedButton(
                                onClick = { onMarkBurstNeedsReview?.invoke(burstGroupId) },
                                enabled = decision.markNeedsReviewEnabled,
                                modifier = Modifier.fillMaxWidth(),
                                shape = RoundedCornerShape(12.dp),
                                border = BorderStroke(1.dp, ElementBorder),
                                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                            ) {
                                Text("标记复核", maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                        }
                    decision.splitBurstTarget
                        ?.takeIf { onSplitBurstMember != null }
                        ?.let { target ->
                            OutlinedButton(
                                onClick = {
                                    onSplitBurstMember?.invoke(
                                        target.burstGroupId,
                                        target.memberGroupId,
                                    )
                                },
                                enabled = decision.splitBurstEnabled,
                                modifier = Modifier.fillMaxWidth(),
                                shape = RoundedCornerShape(12.dp),
                                border = BorderStroke(1.dp, ElementWarning.copy(alpha = 0.45f)),
                                colors = ButtonDefaults.outlinedButtonColors(
                                    containerColor = ElementControlSurface,
                                    contentColor = ElementWarning,
                                ),
                                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                            ) {
                                Text("\u79fb\u51fa\u8fde\u62cd\u7ec4", maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                        }
                    val restoreAutomaticBurstGroupId = decision.restoreAutomaticBurstGroupId
                    if (restoreAutomaticBurstGroupId != null && onRestoreAutomaticRecommendation != null) {
                        OutlinedButton(
                            onClick = {
                                onRestoreAutomaticRecommendation.invoke(restoreAutomaticBurstGroupId)
                            },
                            enabled = decision.restoreAutomaticEnabled,
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(12.dp),
                            border = BorderStroke(1.dp, ElementBorder),
                            contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                        ) {
                            Text("恢复自动推荐", maxLines = 1, overflow = TextOverflow.Ellipsis)
                        }
                    }
                }
            }
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
) {
    val context = LocalContext.current
    val previewLocation = asset.previewLocation.takeIf(::isDecodablePreviewLocation)
    val initialBitmap = remember(previewLocation, previewQuality) {
        if (previewQuality == PreviewQuality.Thumbnail) {
            cachedThumbnailPreview(previewLocation)
        } else {
            null
        }
    }
    val bitmap by produceState<Bitmap?>(initialValue = initialBitmap, previewLocation, previewQuality) {
        value = if (previewLocation == null) {
            null
        } else {
            cachedThumbnailPreview(previewLocation)?.takeIf { previewQuality == PreviewQuality.Thumbnail }
                ?: withContext(Dispatchers.IO) {
                    loadPreviewBitmap(context, previewLocation, previewQuality)
                }?.also { loadedBitmap ->
                    if (previewQuality == PreviewQuality.Thumbnail) {
                        cacheThumbnailPreview(previewLocation, loadedBitmap)
                    }
                }
        }
    }

    val loadedBitmap = bitmap
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
        } else {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                if (compactFallback) {
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

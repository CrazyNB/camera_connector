package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.media.PreviewQuality

@Composable
internal fun ReviewModeScreen(
    assets: List<InboxAsset>,
    currentIndex: Int,
    queueEntry: ReviewQueueEntryUi,
    onCurrentIndexChange: (Int) -> Unit,
    onOpenAsset: (InboxAsset) -> Unit,
    onAcceptRecommendedBest: (InboxAsset) -> Unit,
    onOverrideRecommendedBest: (InboxAsset) -> Unit,
    onMarkNeedsReview: (InboxAsset) -> Unit,
    onRestoreAutomaticRecommendation: (InboxAsset) -> Unit,
    onClearRecommendation: (InboxAsset) -> Unit,
    onKeepAllCandidates: (InboxAsset) -> Unit,
    onHideLowScoreCandidates: (InboxAsset) -> Unit,
    onSplitBurstMember: (ManualBurstSplitTarget) -> Unit,
    onSkipCurrent: () -> Unit,
    onUndoLatestDecision: () -> Unit,
    onExit: () -> Unit,
    modifier: Modifier = Modifier,
    actionsEnabled: Boolean = true,
    session: ReviewModeSessionUi = ReviewModeSessionUi(),
    burstMembers: List<BurstMemberFilmstripItemUi> = emptyList(),
    comparisonItems: List<BurstMemberFilmstripItemUi> = emptyList(),
) {
    val progress = reviewModeProgress(currentIndex, assets.size)
    val asset = assets.getOrNull(progress.currentIndex)
    var burstComparisonOpen by remember(asset?.assetSelectionId()) { mutableStateOf(false) }
    val hasBurstDecision = asset?.burst?.burstGroupId?.isNotBlank() == true
    val splitBurstTarget = reviewModeManualSplitTarget(asset, actionsEnabled)
    val canRestoreAutomaticRecommendation =
        asset?.let { reviewDecisionBurstGroupId(it, ReviewDecisionAction.RestoreAutomaticRecommendation) } != null
    fun handleDragAction(action: ReviewModeDragAction) {
        val currentAsset = asset ?: return
        when (action) {
            ReviewModeDragAction.Previous -> {
                if (progress.currentIndex > 0) {
                    onCurrentIndexChange(previousReviewIndex(progress.currentIndex))
                }
            }
            ReviewModeDragAction.Next -> {
                if (progress.currentIndex < progress.totalCount - 1) {
                    onCurrentIndexChange(nextReviewIndex(progress.currentIndex, progress.totalCount))
                }
            }
            ReviewModeDragAction.AcceptRecommendedBest -> {
                if (actionsEnabled && currentAsset.isBestRecommendedAsset()) {
                    onAcceptRecommendedBest(currentAsset)
                }
            }
            ReviewModeDragAction.MarkNeedsReview -> {
                if (actionsEnabled && hasBurstDecision) {
                    onMarkNeedsReview(currentAsset)
                }
            }
        }
    }
    if (burstComparisonOpen && comparisonItems.size > 1) {
        BurstComparisonDialog(
            items = comparisonItems,
            onDismiss = { burstComparisonOpen = false },
        )
    }
    var secondaryActionsOpen by remember(asset?.assetSelectionId()) { mutableStateOf(false) }
    val primaryAction = reviewModePrimaryAction(asset, actionsEnabled)
    fun performPrimaryAction(action: ReviewModePrimaryAction) {
        val currentAsset = asset ?: return
        when (action) {
            ReviewModePrimaryAction.AcceptRecommendedBest -> onAcceptRecommendedBest(currentAsset)
            ReviewModePrimaryAction.OverrideRecommendedBest -> onOverrideRecommendedBest(currentAsset)
            ReviewModePrimaryAction.MarkNeedsReview -> onMarkNeedsReview(currentAsset)
        }
    }

    Column(
        modifier = modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        if (asset == null) {
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center,
            ) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Text(
                        "当前队列没有照片",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    OutlinedButton(
                        onClick = onExit,
                        shape = elementShape,
                        border = BorderStroke(1.dp, ElementBorder),
                    ) {
                        Text("退出刷图")
                    }
                }
            }
        } else {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .weight(1f)
                    .background(Color.Black, RoundedCornerShape(18.dp))
                    .pointerInput(asset.assetSelectionId(), actionsEnabled, progress.currentIndex, progress.totalCount) {
                        var totalX = 0f
                        var totalY = 0f
                        detectDragGestures(
                            onDragStart = {
                                totalX = 0f
                                totalY = 0f
                            },
                            onDragEnd = {
                                reviewModeDragAction(totalX, totalY)?.let(::handleDragAction)
                                totalX = 0f
                                totalY = 0f
                            },
                            onDragCancel = {
                                totalX = 0f
                                totalY = 0f
                            },
                        ) { change, dragAmount ->
                            change.consume()
                            totalX += dragAmount.x
                            totalY += dragAmount.y
                        }
                    },
            ) {
                PhotoPreview(
                    asset = asset,
                    previewQuality = PreviewQuality.Detail,
                    contentScale = ContentScale.Fit,
                    backgroundColor = Color.Black,
                    modifier = Modifier.matchParentSize(),
                )
                Row(
                    modifier = Modifier
                        .align(Alignment.TopCenter)
                        .fillMaxWidth()
                        .padding(10.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    ReviewModeFloatingTag(queueEntry.primaryLabel, ElementBlue)
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        ReviewModeFloatingTag(progress.text, ElementBlue)
                        Surface(
                            modifier = Modifier.clickable(onClick = onExit),
                            color = ElementBackground.copy(alpha = 0.78f),
                            contentColor = ElementText,
                            shape = RoundedCornerShape(999.dp),
                            border = BorderStroke(1.dp, ElementBorder.copy(alpha = 0.72f)),
                        ) {
                            Text(
                                text = "退出",
                                modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp),
                                style = MaterialTheme.typography.labelMedium,
                                fontWeight = FontWeight.SemiBold,
                            )
                        }
                    }
                }
                Surface(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .padding(10.dp),
                    color = ElementBackground.copy(alpha = 0.82f),
                    contentColor = MaterialTheme.colorScheme.onSurface,
                    shape = RoundedCornerShape(16.dp),
                    border = BorderStroke(1.dp, ElementBorder.copy(alpha = 0.7f)),
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(12.dp),
                        verticalArrangement = Arrangement.spacedBy(7.dp),
                    ) {
                        ReviewModeBadgeRow(
                            asset = asset,
                        )
                        Text(
                            asset.filename(),
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            listOf(asset.sourceLabel(), asset.formatBadges(), asset.tileSmartMeta())
                                .filterNotNull()
                                .joinToString(" / "),
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            style = MaterialTheme.typography.bodySmall,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                        ReviewModeSignalStrip(rows = asset.reviewModeSignalRows())
                        session.compactText?.let { text ->
                            Text(
                                text,
                                color = ElementSuccess,
                                style = MaterialTheme.typography.bodySmall,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                }
            }

            primaryAction?.let { action ->
                Button(
                    onClick = { performPrimaryAction(action.action) },
                    enabled = action.enabled,
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(14.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = reviewModePrimaryActionColor(action.action),
                        contentColor = ElementOnAccent,
                    ),
                ) {
                    Text(action.label, fontWeight = FontWeight.Bold)
                }
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                OutlinedButton(
                    onClick = { onCurrentIndexChange(previousReviewIndex(progress.currentIndex)) },
                    enabled = progress.currentIndex > 0,
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(12.dp),
                    border = BorderStroke(1.dp, ElementBorder),
                    contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                ) {
                    Text("上一张")
                }
                OutlinedButton(
                    onClick = { secondaryActionsOpen = !secondaryActionsOpen },
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(12.dp),
                    border = BorderStroke(1.dp, ElementBorder),
                    colors = ButtonDefaults.outlinedButtonColors(
                        containerColor = if (secondaryActionsOpen) ElementBlueSoft else ElementControlSurface,
                        contentColor = if (secondaryActionsOpen) ElementBlue else MaterialTheme.colorScheme.onSurface,
                    ),
                    contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                ) {
                    Text(if (secondaryActionsOpen) "收起" else "更多")
                }
                OutlinedButton(
                    onClick = { onCurrentIndexChange(nextReviewIndex(progress.currentIndex, progress.totalCount)) },
                    enabled = progress.currentIndex < progress.totalCount - 1,
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(12.dp),
                    border = BorderStroke(1.dp, ElementBorder),
                    contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                ) {
                    Text("下一张")
                }
            }
            if (secondaryActionsOpen) {
                Surface(
                    modifier = Modifier.fillMaxWidth(),
                    color = ElementControlSurface,
                    contentColor = MaterialTheme.colorScheme.onSurface,
                    shape = RoundedCornerShape(14.dp),
                    border = BorderStroke(1.dp, ElementBorder),
                ) {
                    Column(
                        modifier = Modifier.padding(10.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            OutlinedButton(
                                onClick = { onOpenAsset(asset) },
                                modifier = Modifier.weight(1f),
                                shape = RoundedCornerShape(10.dp),
                                border = BorderStroke(1.dp, ElementBorder),
                                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                            ) {
                                Text("详情", maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                            OutlinedButton(
                                onClick = { burstComparisonOpen = true },
                                enabled = comparisonItems.size > 1,
                                modifier = Modifier.weight(1f),
                                shape = RoundedCornerShape(10.dp),
                                border = BorderStroke(1.dp, ElementBorder),
                                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                            ) {
                                Text("对比", maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                            OutlinedButton(
                                onClick = onSkipCurrent,
                                enabled = actionsEnabled,
                                modifier = Modifier.weight(1f),
                                shape = RoundedCornerShape(10.dp),
                                border = BorderStroke(1.dp, ElementBorder),
                                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                            ) {
                                Text("跳过", maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                        }
                        if (hasBurstDecision) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(8.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                OutlinedButton(
                                    onClick = { onMarkNeedsReview(asset) },
                                    enabled = actionsEnabled,
                                    modifier = Modifier.weight(1f),
                                    shape = RoundedCornerShape(10.dp),
                                    border = BorderStroke(1.dp, ElementBorder),
                                    contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                                ) {
                                    Text("复核", maxLines = 1, overflow = TextOverflow.Ellipsis)
                                }
                                OutlinedButton(
                                    onClick = { onKeepAllCandidates(asset) },
                                    enabled = actionsEnabled,
                                    modifier = Modifier.weight(1f),
                                    shape = RoundedCornerShape(10.dp),
                                    border = BorderStroke(1.dp, ElementBorder),
                                    contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                                ) {
                                    Text("保留全部", maxLines = 1, overflow = TextOverflow.Ellipsis)
                                }
                                OutlinedButton(
                                    onClick = { onHideLowScoreCandidates(asset) },
                                    enabled = actionsEnabled,
                                    modifier = Modifier.weight(1f),
                                    shape = RoundedCornerShape(10.dp),
                                    border = BorderStroke(1.dp, ElementBorder),
                                    contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                                ) {
                                    Text("隐藏低分", maxLines = 1, overflow = TextOverflow.Ellipsis)
                                }
                            }
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(8.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                OutlinedButton(
                                    onClick = { onClearRecommendation(asset) },
                                    enabled = actionsEnabled,
                                    modifier = Modifier.weight(1f),
                                    shape = RoundedCornerShape(10.dp),
                                    border = BorderStroke(1.dp, ElementWarning.copy(alpha = 0.45f)),
                                    contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                                ) {
                                    Text("清除推荐", maxLines = 1, overflow = TextOverflow.Ellipsis)
                                }
                                OutlinedButton(
                                    onClick = { onRestoreAutomaticRecommendation(asset) },
                                    enabled = actionsEnabled && canRestoreAutomaticRecommendation,
                                    modifier = Modifier.weight(1f),
                                    shape = RoundedCornerShape(10.dp),
                                    border = BorderStroke(1.dp, ElementBlue.copy(alpha = 0.45f)),
                                    colors = ButtonDefaults.outlinedButtonColors(
                                        containerColor = ElementBlueSoft.copy(alpha = 0.45f),
                                        contentColor = ElementBlue,
                                    ),
                                    contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                                ) {
                                    Text("恢复自动", maxLines = 1, overflow = TextOverflow.Ellipsis)
                                }
                            }
                        }
                        session.undoLabel?.let { undoLabel ->
                            OutlinedButton(
                                onClick = onUndoLatestDecision,
                                enabled = actionsEnabled && session.undoBurstGroupId != null,
                                modifier = Modifier.fillMaxWidth(),
                                shape = RoundedCornerShape(10.dp),
                                border = BorderStroke(1.dp, ElementBorder),
                                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                            ) {
                                Text(undoLabel, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                        }
                        if (splitBurstTarget != null) {
                            OutlinedButton(
                                onClick = { onSplitBurstMember(splitBurstTarget) },
                                enabled = actionsEnabled,
                                modifier = Modifier.fillMaxWidth(),
                                shape = RoundedCornerShape(10.dp),
                                border = BorderStroke(1.dp, ElementWarning.copy(alpha = 0.45f)),
                                colors = ButtonDefaults.outlinedButtonColors(
                                    containerColor = ElementWarning.copy(alpha = 0.10f),
                                    contentColor = ElementWarning,
                                ),
                                contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
                            ) {
                                Text("\u79fb\u51fa\u8fde\u62cd\u7ec4", maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                        }
                    }
                }
            }
        }
    }
}

private fun reviewModePrimaryActionColor(action: ReviewModePrimaryAction): Color =
    when (action) {
        ReviewModePrimaryAction.AcceptRecommendedBest -> ElementSuccess
        ReviewModePrimaryAction.OverrideRecommendedBest -> ElementBlue
        ReviewModePrimaryAction.MarkNeedsReview -> ElementWarning
    }

@Composable
private fun ReviewModeShortcutHintRow(
    hints: List<ReviewModeShortcutHintUi>,
    modifier: Modifier = Modifier,
) {
    if (hints.isEmpty()) {
        return
    }
    LazyRow(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterHorizontally),
    ) {
        items(hints, key = { "${it.gestureLabel}-${it.actionLabel}" }) { hint ->
            Surface(
                color = ElementBackground.copy(alpha = if (hint.enabled) 0.82f else 0.56f),
                contentColor = if (hint.enabled) ElementText else ElementTextMuted,
                shape = RoundedCornerShape(999.dp),
                border = BorderStroke(
                    1.dp,
                    if (hint.enabled) ElementBlue.copy(alpha = 0.36f) else ElementBorder.copy(alpha = 0.58f),
                ),
            ) {
                Text(
                    text = "${hint.gestureLabel} ${hint.actionLabel}",
                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                    style = MaterialTheme.typography.labelSmall,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

@Composable
private fun ReviewModeBurstFilmstrip(
    members: List<BurstMemberFilmstripItemUi>,
    onOpenAsset: (InboxAsset) -> Unit,
    onOverrideRecommendedBest: (InboxAsset) -> Unit,
    actionsEnabled: Boolean,
) {
    if (members.isEmpty()) {
        return
    }
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(
            "组内对比",
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            fontWeight = FontWeight.SemiBold,
        )
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            items(members, key = { it.asset.assetSelectionId() }) { item ->
                ReviewModeBurstMemberTile(
                    item = item,
                    onOpenAsset = onOpenAsset,
                    onOverrideRecommendedBest = onOverrideRecommendedBest,
                    actionsEnabled = actionsEnabled,
                )
            }
        }
    }
}

@Composable
private fun ReviewModeBurstMemberTile(
    item: BurstMemberFilmstripItemUi,
    onOpenAsset: (InboxAsset) -> Unit,
    onOverrideRecommendedBest: (InboxAsset) -> Unit,
    actionsEnabled: Boolean,
) {
    Surface(
        modifier = Modifier
            .width(88.dp)
            .clickable { onOpenAsset(item.asset) },
        color = ElementControlSurface,
        contentColor = MaterialTheme.colorScheme.onSurface,
        shape = RoundedCornerShape(12.dp),
        border = BorderStroke(1.dp, ElementBorder),
    ) {
        Column(
            modifier = Modifier.padding(6.dp),
            verticalArrangement = Arrangement.spacedBy(5.dp),
        ) {
            PhotoPreview(
                asset = item.asset,
                compactFallback = true,
                backgroundColor = item.asset.previewAccentColor().copy(alpha = 0.16f),
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(1f),
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    item.badgeText,
                    style = MaterialTheme.typography.labelSmall,
                    color = if (item.badgeText == "最佳") ElementSuccess else ElementInfo,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                item.scoreText?.let { score ->
                    Text(
                        score,
                        style = MaterialTheme.typography.labelSmall,
                        color = smartBadgeColor(item.asset),
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            }
            if (!item.asset.isBestRecommendedAsset()) {
                OutlinedButton(
                    onClick = { onOverrideRecommendedBest(item.asset) },
                    enabled = actionsEnabled,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(26.dp),
                    shape = RoundedCornerShape(8.dp),
                    border = BorderStroke(1.dp, ElementBlue.copy(alpha = 0.45f)),
                    colors = ButtonDefaults.outlinedButtonColors(
                        containerColor = ElementBlueSoft.copy(alpha = 0.58f),
                        contentColor = ElementBlue,
                    ),
                    contentPadding = PaddingValues(horizontal = 4.dp, vertical = 0.dp),
                ) {
                    Text(
                        "设最佳",
                        style = MaterialTheme.typography.labelSmall,
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }
    }
}

@Composable
private fun ReviewModeSignalStrip(rows: List<QualitySignalRow>) {
    if (rows.isEmpty()) {
        return
    }
    LazyRow(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        items(rows, key = { it.label }) { row ->
            ReviewModeFloatingTag(
                text = "${row.label} ${row.value}",
                color = reviewModeSignalColor(row.label),
            )
        }
    }
}

private fun reviewModeSignalColor(label: String): Color =
    when (label) {
        "锐度" -> ElementSuccess
        "曝光" -> ElementBlue
        "构图" -> ElementPurple
        else -> ElementInfo
    }

@Composable
private fun ReviewModeBadgeRow(
    asset: InboxAsset,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        asset.qualityBadgeText()?.let { ReviewModeFloatingTag(it, smartBadgeColor(asset)) }
        asset.groupBestBadgeText()?.let { ReviewModeFloatingTag(it, ElementWarning) }
        asset.burstBadgeText()?.let { ReviewModeFloatingTag(it, ElementPurple) }
        asset.recommendationBadgeText()?.let {
            ReviewModeFloatingTag(it, if (asset.isBestRecommendedAsset()) ElementSuccess else ElementInfo)
        }
    }
}

@Composable
private fun ReviewModeFloatingTag(
    text: String,
    color: Color,
) {
    Surface(
        color = ElementBackground.copy(alpha = 0.78f),
        contentColor = color,
        shape = RoundedCornerShape(999.dp),
        border = BorderStroke(1.dp, color.copy(alpha = 0.42f)),
    ) {
        Text(
            text = text,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

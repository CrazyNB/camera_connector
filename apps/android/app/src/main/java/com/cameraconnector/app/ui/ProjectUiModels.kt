package com.cameraconnector.app.ui

import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetQuery
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ReviewQueueSummary
import com.cameraconnector.app.core.StrategyProfileUi
import com.cameraconnector.app.core.StrategyWeightsUi
import kotlin.math.abs
import kotlin.math.roundToInt

internal enum class GlobalDestination(val label: String) {
    Projects("项目"),
    Accounts("账号"),
    Settings("设置"),
}

internal enum class ProjectDestination(val label: String) {
    Photos("照片"),
}

internal fun defaultProjectDestination(): ProjectDestination =
    ProjectDestination.Photos

internal fun ProjectDestination.assetScreenTitle(): String =
    "项目照片"

internal fun ProjectDestination.assetScreenSubtitle(): String =
    "项目内 Assets / Asset Groups"

internal data class ProjectLifecycleUi(
    val statusLabel: String,
    val canSelect: Boolean,
    val canArchive: Boolean,
    val canRename: Boolean,
    val canRestore: Boolean,
)

internal data class ReviewQueueEntryUi(
    val primaryLabel: String,
    val primaryCount: Int,
    val primaryText: String,
    val subtitle: String,
    val queue: String,
    val recommendationState: String?,
    val analysisStatus: String?,
)

internal data class ReviewModeProgressUi(
    val currentIndex: Int,
    val totalCount: Int,
    val text: String,
)

internal enum class ReviewSessionDecision {
    AcceptRecommendedBest,
    OverrideRecommendedBest,
    MarkNeedsReview,
    RestoreAutomaticRecommendation,
    ClearRecommendation,
    KeepAllCandidates,
    HideLowScoreCandidates,
    SkipCurrent,
}

private fun ReviewSessionDecision.undoLabel(): String? = when (this) {
    ReviewSessionDecision.AcceptRecommendedBest -> "撤销接受推荐"
    ReviewSessionDecision.OverrideRecommendedBest -> "撤销手动优选"
    ReviewSessionDecision.MarkNeedsReview -> "撤销标记复核"
    ReviewSessionDecision.RestoreAutomaticRecommendation -> null
    ReviewSessionDecision.ClearRecommendation -> null
    ReviewSessionDecision.KeepAllCandidates -> null
    ReviewSessionDecision.HideLowScoreCandidates -> null
    ReviewSessionDecision.SkipCurrent -> null
}

internal data class ReviewModeSessionUi(
    val processedGroupCount: Int = 0,
    val acceptedRecommendationCount: Int = 0,
    val manualOverrideCount: Int = 0,
    val markedNeedsReviewCount: Int = 0,
    val restoredAutomaticCount: Int = 0,
    val clearedRecommendationCount: Int = 0,
    val keptAllCandidatesCount: Int = 0,
    val hiddenLowScoreCount: Int = 0,
    val skippedCount: Int = 0,
    val undoBurstGroupId: String? = null,
    val undoDecision: ReviewSessionDecision? = null,
) {
    val hasActivity: Boolean
        get() = processedGroupCount > 0

    val undoLabel: String?
        get() = undoDecision?.undoLabel()

    val compactText: String?
        get() {
            if (!hasActivity) {
                return null
            }
            return buildList {
                add("本轮 $processedGroupCount 组")
                if (acceptedRecommendationCount > 0) add("接受 $acceptedRecommendationCount")
                if (manualOverrideCount > 0) add("手动 $manualOverrideCount")
                if (markedNeedsReviewCount > 0) add("复核 $markedNeedsReviewCount")
                if (restoredAutomaticCount > 0) add("恢复 $restoredAutomaticCount")
                if (clearedRecommendationCount > 0) add("清除 $clearedRecommendationCount")
                if (keptAllCandidatesCount > 0) add("保留 $keptAllCandidatesCount")
                if (hiddenLowScoreCount > 0) add("隐藏低分 $hiddenLowScoreCount")
                if (skippedCount > 0) add("跳过 $skippedCount")
            }.joinToString(" · ")
        }
}

internal data class PhotoDetailDecisionUi(
    val acceptRecommendedBestBurstGroupId: String?,
    val markNeedsReviewBurstGroupId: String?,
    val restoreAutomaticBurstGroupId: String?,
    val overrideRecommendedBestTarget: ManualBestOverrideTarget?,
    val splitBurstTarget: ManualBurstSplitTarget?,
    val acceptRecommendedBestEnabled: Boolean,
    val markNeedsReviewEnabled: Boolean,
    val restoreAutomaticEnabled: Boolean,
    val overrideRecommendedBestEnabled: Boolean,
    val splitBurstEnabled: Boolean,
    val disabledReason: String?,
) {
    val hasAnyAction: Boolean
        get() = acceptRecommendedBestBurstGroupId != null ||
            markNeedsReviewBurstGroupId != null ||
            restoreAutomaticBurstGroupId != null ||
            overrideRecommendedBestTarget != null ||
            splitBurstTarget != null
}

internal data class BurstMemberFilmstripItemUi(
    val asset: InboxAsset,
    val badgeText: String,
    val scoreText: String?,
)

internal data class ProjectPhotoGridItemUi(
    val key: String,
    val coverAsset: InboxAsset,
    val members: List<InboxAsset>,
) {
    val isBurstGroup: Boolean
        get() = key.startsWith("burst:") && members.size > 1
}

internal enum class DetailNavigationDirection {
    Previous,
    Next,
}

internal enum class ScoreFilter(
    val label: String,
    val scoreMin: Double?,
) {
    All("全部评分", null),
    Excellent("80+", 80.0),
    Usable("60+", 60.0),
}

internal enum class ProjectPhotoCollection(val label: String) {
    All("全部"),
    Selects("精选"),
}

internal enum class ReceiverStartBlockReason {
    Busy,
    MissingAccount,
    MissingNotificationPermission,
}

internal fun receiverStartBlockReason(
    running: Boolean,
    actionsEnabled: Boolean,
    notificationPermissionGranted: Boolean,
    accountCount: Int,
): ReceiverStartBlockReason? = when {
    running -> null
    !actionsEnabled -> ReceiverStartBlockReason.Busy
    accountCount <= 0 -> ReceiverStartBlockReason.MissingAccount
    !notificationPermissionGranted -> ReceiverStartBlockReason.MissingNotificationPermission
    else -> null
}

@Suppress("UNUSED_PARAMETER")
internal fun projectPhotoContentVisible(
    receiverRunning: Boolean,
    reviewModeActive: Boolean,
): Boolean = true

internal fun ProjectState.activeProjectSummary(): ProjectSummary? =
    activeProjectId?.let { id -> projects.firstOrNull { it.id == id } }

internal fun ProjectState.groupMoveTargets(sourceProjectId: String?): List<ProjectSummary> {
    val sourceId = sourceProjectId?.takeIf { it.isNotBlank() } ?: return emptyList()
    return projects.filter { project ->
        project.id != sourceId && project.canAcceptMovedGroups
    }
}

internal fun InboxAsset.assetSelectionId(): String =
    id.ifBlank { displayPath }

internal fun toggleAssetSelection(
    selectedIds: List<String>,
    asset: InboxAsset,
): List<String> {
    val id = asset.assetSelectionId()
    if (id.isBlank()) {
        return selectedIds
    }
    return if (id in selectedIds) {
        selectedIds.filterNot { it == id }
    } else {
        selectedIds + id
    }
}

internal fun selectedAssetsFromIds(
    assets: List<InboxAsset>,
    selectedIds: List<String>,
): List<InboxAsset> {
    if (selectedIds.isEmpty()) {
        return emptyList()
    }
    val selected = selectedIds.toSet()
    return assets.filter { it.assetSelectionId() in selected }
}

internal fun refreshedSelectedPhoto(
    selectedPhoto: InboxAsset?,
    visibleAssets: List<InboxAsset>,
): InboxAsset? {
    val selectedId = selectedPhoto?.assetSelectionId()?.takeIf { it.isNotBlank() } ?: return selectedPhoto
    return visibleAssets.firstOrNull { it.assetSelectionId() == selectedId }
}

internal fun isAssetSelectionMode(selectedIds: List<String>): Boolean =
    selectedIds.isNotEmpty()

internal fun projectLifecycleUi(
    project: ProjectSummary,
    selected: Boolean,
    actionsEnabled: Boolean,
): ProjectLifecycleUi {
    return ProjectLifecycleUi(
        statusLabel = when {
            selected -> "当前项目"
            project.canRestore -> "已归档"
            else -> "活跃"
        },
        canSelect = actionsEnabled && !selected && project.canBeActiveProject,
        canArchive = actionsEnabled && project.canArchive,
        canRename = actionsEnabled && project.canRename,
        canRestore = actionsEnabled && project.canRestore,
    )
}

internal enum class StrategyWeightField(
    val label: String,
    val range: ClosedFloatingPointRange<Double>,
    val value: (StrategyWeightsUi) -> Double,
) {
    Sharpness("锐度", 0.0..0.7, { it.sharpness }),
    Exposure("曝光", 0.0..0.5, { it.exposure }),
    Composition("构图", 0.0..0.12, { it.composition }),
    HighlightClippingPenalty("高光惩罚", -0.3..0.0, { it.highlightClippingPenalty }),
    ShadowClippingPenalty("暗部惩罚", -0.2..0.0, { it.shadowClippingPenalty }),
    Diversity("多样性", 0.0..0.12, { it.diversity }),
}

internal fun selectedStrategyProfile(
    profiles: List<StrategyProfileUi>,
    selectedProfileId: String?,
): StrategyProfileUi? =
    profiles.firstOrNull { it.profileId == selectedProfileId }
        ?: profiles.firstOrNull { it.profileId == "general" }
        ?: profiles.firstOrNull()

internal fun StrategyProfileUi.weightValue(field: StrategyWeightField): Double =
    field.value(weights)

internal fun StrategyProfileUi.withStrategyWeight(
    field: StrategyWeightField,
    rawValue: Double,
): StrategyProfileUi {
    val value = rawValue.coerceIn(field.range.start, field.range.endInclusive)
    val nextWeights = when (field) {
        StrategyWeightField.Sharpness -> weights.copy(sharpness = value)
        StrategyWeightField.Exposure -> weights.copy(exposure = value)
        StrategyWeightField.Composition -> weights.copy(composition = value)
        StrategyWeightField.HighlightClippingPenalty -> weights.copy(highlightClippingPenalty = value)
        StrategyWeightField.ShadowClippingPenalty -> weights.copy(shadowClippingPenalty = value)
        StrategyWeightField.Diversity -> weights.copy(diversity = value)
    }
    return copy(weights = nextWeights)
}

internal fun StrategyProfileUi.asSavableCustomStrategyProfile(nowMs: Long): StrategyProfileUi {
    val customId = if (builtIn) {
        "custom-${profileId.trim().ifBlank { "strategy" }}"
    } else {
        profileId.trim().ifBlank { "custom-strategy" }
    }
    val customName = if (builtIn && !name.startsWith("自定义 ")) {
        "自定义 $name"
    } else {
        name
    }
    return copy(
        profileId = customId,
        name = customName,
        builtIn = false,
        weights = weights.copy(composition = weights.composition.coerceIn(0.0, 0.12)),
        updatedAtMs = nowMs,
    )
}

internal fun strategyWeightDisplayText(value: Double): String =
    "${(value * 100).roundToInt()}%"

internal fun ReviewQueueSummary.reviewQueueEntryUi(): ReviewQueueEntryUi? =
    reviewQueueEntriesUi().firstOrNull()

internal fun ReviewQueueSummary.reviewQueueEntriesUi(): List<ReviewQueueEntryUi> {
    if (totalUnits <= 0) {
        return emptyList()
    }
    return buildList {
        if (unconfirmedBestCount > 0) {
            add(
                ReviewQueuePrimary(
                    label = "待确认优选",
                    count = unconfirmedBestCount,
                    unit = "组待确认",
                    queue = "unconfirmed_best",
                    recommendationState = "ready",
                ),
            )
        }
        if (unsupportedCount > 0) {
            add(
                ReviewQueuePrimary(
                    label = "不支持评分",
                    count = unsupportedCount,
                    unit = "组需复核",
                    queue = "unsupported",
                ),
            )
        }
        if (needsReviewCount > 0) {
            add(
                ReviewQueuePrimary(
                    label = "需要复核",
                    count = needsReviewCount,
                    unit = "组需复核",
                    queue = "needs_review",
                    recommendationState = "needs_review",
                ),
            )
        }
        if (lowScoreCandidateCount > 0) {
            add(
                ReviewQueuePrimary(
                    label = "低分候选",
                    count = lowScoreCandidateCount,
                    unit = "组低分",
                    queue = "low_score_candidates",
                ),
            )
        }
        if (nearDuplicateCount > 0) {
            add(
                ReviewQueuePrimary(
                    label = "近重复",
                    count = nearDuplicateCount,
                    unit = "组近重复",
                    queue = "near_duplicates",
                ),
            )
        }
        if (userOverriddenCount > 0) {
            add(
                ReviewQueuePrimary(
                    label = "手动调整",
                    count = userOverriddenCount,
                    unit = "组已调整",
                    queue = "user_overridden",
                ),
            )
        }
        if (pendingCount > 0) {
            add(
                ReviewQueuePrimary(
                    label = "等待分析",
                    count = pendingCount,
                    unit = "组待评分",
                    queue = "pending",
                    analysisStatus = "pending",
                ),
            )
        }
    }.map { primary ->
        ReviewQueueEntryUi(
            primaryLabel = primary.label,
            primaryCount = primary.count,
            primaryText = "${primary.count} ${primary.unit}",
            subtitle = reviewQueueSubtitle(),
            queue = primary.queue,
            recommendationState = primary.recommendationState,
            analysisStatus = primary.analysisStatus,
        )
    }
}

internal fun List<ReviewQueueEntryUi>.selectedReviewQueueEntry(selectedQueue: String?): ReviewQueueEntryUi? =
    firstOrNull { it.queue == selectedQueue } ?: firstOrNull()

internal fun ReviewQueueEntryUi.assetQuery(
    selectedAccount: String?,
    strategyProfileId: String,
): InboxAssetQuery =
    InboxAssetQuery(
        username = selectedAccount,
        sort = PhotoSortMode.GroupBestScore,
        recommendationState = recommendationState,
        analysisStatus = analysisStatus,
        reviewQueue = queue,
        strategyProfileId = strategyProfileId,
    )

internal fun assetListQuery(
    selectedAccount: String?,
    selectedFilter: InboxFilter,
    selectedSort: PhotoSortMode,
    selectedScoreFilter: ScoreFilter,
): InboxAssetQuery {
    val scoreMin = selectedScoreFilter.scoreMin
    return InboxAssetQuery(
        username = selectedAccount,
        role = selectedFilter.assetRole(),
        sort = if (scoreMin != null && selectedSort == PhotoSortMode.LatestReceived) {
            PhotoSortMode.GroupBestScore
        } else {
            selectedSort
        },
        scoreMin = scoreMin,
    )
}

internal fun projectPhotoCollectionAssets(
    assets: List<InboxAsset>,
    selectedAccount: String?,
    selectedFilter: InboxFilter,
    selectedSort: PhotoSortMode,
    selectedScoreFilter: ScoreFilter,
): List<InboxAsset> {
    val scoreMin = selectedScoreFilter.scoreMin?.let(::normalizedProjectQueryScore)
    val filtered = assets
        .asSequence()
        .filter { asset -> selectedAccount == null || asset.username == selectedAccount }
        .filter { asset -> selectedFilter.matches(asset) }
        .filter { asset ->
            scoreMin == null ||
                asset.projectQueryBestScore()
                    ?.let(::normalizedProjectQueryScore)
                    ?.let { it >= scoreMin }
                    ?: false
        }
        .toList()

    return when (selectedSort) {
        PhotoSortMode.LatestReceived -> filtered.sortedByDescending { it.receivedAt.toLongOrNull() ?: 0L }
        PhotoSortMode.Filename -> filtered.sortedBy { it.groupKey.ifBlank { it.displayPath } }
        PhotoSortMode.GroupBestScore -> filtered.sortedWith(
            compareByDescending<InboxAsset> {
                it.projectQueryBestScore()?.let(::normalizedProjectQueryScore) ?: -1.0
            }.thenByDescending { it.receivedAt.toLongOrNull() ?: 0L },
        )
    }
}

private fun InboxAsset.projectQueryBestScore(): Double? =
    burst?.bestScore ?: quality?.overall

private fun normalizedProjectQueryScore(value: Double): Double =
    if (value > 1.0) value / 100.0 else value

private data class ReviewQueuePrimary(
    val label: String,
    val count: Int,
    val unit: String,
    val queue: String,
    val recommendationState: String? = null,
    val analysisStatus: String? = null,
)

private fun ReviewQueueSummary.reviewQueueSubtitle(): String =
    buildList {
        add("连拍/单张 $totalUnits 组")
        if (pendingCount > 0) add("待评分 $pendingCount")
        if (needsReviewCount > 0) add("需复核 $needsReviewCount")
        if (lowScoreCandidateCount > 0) add("低分 $lowScoreCandidateCount")
        if (nearDuplicateCount > 0) add("近重复 $nearDuplicateCount")
        if (unsupportedCount > 0) add("不支持 $unsupportedCount")
        if (userOverriddenCount > 0) add("已调整 $userOverriddenCount")
    }.joinToString(" · ")

internal fun reviewModeProgress(
    currentIndex: Int,
    totalCount: Int,
): ReviewModeProgressUi {
    val safeTotal = totalCount.coerceAtLeast(0)
    if (safeTotal == 0) {
        return ReviewModeProgressUi(currentIndex = 0, totalCount = 0, text = "0/0")
    }
    val safeIndex = currentIndex.coerceIn(0, safeTotal - 1)
    return ReviewModeProgressUi(
        currentIndex = safeIndex,
        totalCount = safeTotal,
        text = "${safeIndex + 1}/$safeTotal",
    )
}

internal fun previousReviewIndex(currentIndex: Int): Int =
    (currentIndex - 1).coerceAtLeast(0)

internal fun nextReviewIndex(currentIndex: Int, totalCount: Int): Int {
    val lastIndex = (totalCount - 1).coerceAtLeast(0)
    return (currentIndex + 1).coerceAtMost(lastIndex)
}

internal fun reviewModeShouldSummarizeAfterAction(
    currentIndex: Int,
    totalCount: Int,
): Boolean =
    totalCount <= 1 || currentIndex >= totalCount - 1

internal fun InboxAsset.reviewModeSignalRows(): List<QualitySignalRow> =
    qualitySignalRows()
        .filter { row -> row.label in setOf("锐度", "曝光", "构图") }
        .take(3)

internal enum class ReviewModeDragAction {
    Previous,
    Next,
    AcceptRecommendedBest,
    MarkNeedsReview,
}

internal data class ReviewModeShortcutHintUi(
    val gestureLabel: String,
    val actionLabel: String,
    val enabled: Boolean,
)

internal enum class ReviewModePrimaryAction {
    AcceptRecommendedBest,
    OverrideRecommendedBest,
    MarkNeedsReview,
}

internal data class ReviewModePrimaryActionUi(
    val action: ReviewModePrimaryAction,
    val label: String,
    val enabled: Boolean,
)

internal fun reviewModePrimaryAction(
    asset: InboxAsset?,
    actionsEnabled: Boolean,
): ReviewModePrimaryActionUi? {
    if (asset == null) {
        return null
    }
    val action = when {
        reviewDecisionBurstGroupId(asset, ReviewDecisionAction.AcceptRecommendedBest) != null ->
            ReviewModePrimaryAction.AcceptRecommendedBest to "接受推荐"
        manualBestOverrideTarget(asset) != null ->
            ReviewModePrimaryAction.OverrideRecommendedBest to "设为优选"
        reviewDecisionBurstGroupId(asset, ReviewDecisionAction.MarkNeedsReview) != null ->
            ReviewModePrimaryAction.MarkNeedsReview to "标记复核"
        else -> return null
    }
    return ReviewModePrimaryActionUi(
        action = action.first,
        label = action.second,
        enabled = actionsEnabled,
    )
}

internal fun reviewModeDragAction(
    deltaX: Float,
    deltaY: Float,
    threshold: Float = 96f,
): ReviewModeDragAction? {
    val absX = abs(deltaX)
    val absY = abs(deltaY)
    if (maxOf(absX, absY) < threshold) {
        return null
    }
    return if (absX >= absY) {
        if (deltaX < 0f) ReviewModeDragAction.Next else ReviewModeDragAction.Previous
    } else {
        if (deltaY < 0f) ReviewModeDragAction.AcceptRecommendedBest else ReviewModeDragAction.MarkNeedsReview
    }
}

internal fun reviewModeShortcutHints(
    asset: InboxAsset?,
    currentIndex: Int,
    totalCount: Int,
    actionsEnabled: Boolean,
): List<ReviewModeShortcutHintUi> {
    if (asset == null) {
        return emptyList()
    }
    val hasPrevious = currentIndex > 0
    val hasNext = totalCount > 0 && currentIndex < totalCount - 1
    val hasBurstDecision = asset.burst?.burstGroupId?.isNotBlank() == true
    return listOf(
        ReviewModeShortcutHintUi(
            gestureLabel = "右滑",
            actionLabel = "上一张",
            enabled = hasPrevious,
        ),
        ReviewModeShortcutHintUi(
            gestureLabel = "左滑",
            actionLabel = "下一张",
            enabled = hasNext,
        ),
        ReviewModeShortcutHintUi(
            gestureLabel = "上滑",
            actionLabel = "接受推荐",
            enabled = actionsEnabled && asset.isBestRecommendedAsset(),
        ),
        ReviewModeShortcutHintUi(
            gestureLabel = "下滑",
            actionLabel = "标记复核",
            enabled = actionsEnabled && hasBurstDecision,
        ),
    )
}

internal fun ReviewModeSessionUi.record(
    decision: ReviewSessionDecision,
    burstGroupId: String? = null,
): ReviewModeSessionUi =
    when (decision) {
        ReviewSessionDecision.AcceptRecommendedBest -> copy(
            processedGroupCount = processedGroupCount + 1,
            acceptedRecommendationCount = acceptedRecommendationCount + 1,
            undoBurstGroupId = burstGroupId?.takeIf { it.isNotBlank() },
            undoDecision = decision.takeIf { !burstGroupId.isNullOrBlank() },
        )
        ReviewSessionDecision.OverrideRecommendedBest -> copy(
            processedGroupCount = processedGroupCount + 1,
            manualOverrideCount = manualOverrideCount + 1,
            undoBurstGroupId = burstGroupId?.takeIf { it.isNotBlank() },
            undoDecision = decision.takeIf { !burstGroupId.isNullOrBlank() },
        )
        ReviewSessionDecision.MarkNeedsReview -> copy(
            processedGroupCount = processedGroupCount + 1,
            markedNeedsReviewCount = markedNeedsReviewCount + 1,
            undoBurstGroupId = burstGroupId?.takeIf { it.isNotBlank() },
            undoDecision = decision.takeIf { !burstGroupId.isNullOrBlank() },
        )
        ReviewSessionDecision.RestoreAutomaticRecommendation -> copy(
            processedGroupCount = processedGroupCount + 1,
            restoredAutomaticCount = restoredAutomaticCount + 1,
            undoBurstGroupId = null,
            undoDecision = null,
        )
        ReviewSessionDecision.ClearRecommendation -> copy(
            processedGroupCount = processedGroupCount + 1,
            clearedRecommendationCount = clearedRecommendationCount + 1,
            undoBurstGroupId = null,
            undoDecision = null,
        )
        ReviewSessionDecision.KeepAllCandidates -> copy(
            processedGroupCount = processedGroupCount + 1,
            keptAllCandidatesCount = keptAllCandidatesCount + 1,
            undoBurstGroupId = null,
            undoDecision = null,
        )
        ReviewSessionDecision.HideLowScoreCandidates -> copy(
            processedGroupCount = processedGroupCount + 1,
            hiddenLowScoreCount = hiddenLowScoreCount + 1,
            undoBurstGroupId = null,
            undoDecision = null,
        )
        ReviewSessionDecision.SkipCurrent -> copy(
            processedGroupCount = processedGroupCount + 1,
            skippedCount = skippedCount + 1,
            undoBurstGroupId = null,
            undoDecision = null,
        )
    }

internal fun ReviewModeSessionUi.undoLatestDecision(): ReviewModeSessionUi =
    when (undoDecision) {
        ReviewSessionDecision.AcceptRecommendedBest -> copy(
            processedGroupCount = (processedGroupCount - 1).coerceAtLeast(0),
            acceptedRecommendationCount = (acceptedRecommendationCount - 1).coerceAtLeast(0),
            undoBurstGroupId = null,
            undoDecision = null,
        )
        ReviewSessionDecision.OverrideRecommendedBest -> copy(
            processedGroupCount = (processedGroupCount - 1).coerceAtLeast(0),
            manualOverrideCount = (manualOverrideCount - 1).coerceAtLeast(0),
            undoBurstGroupId = null,
            undoDecision = null,
        )
        ReviewSessionDecision.MarkNeedsReview -> copy(
            processedGroupCount = (processedGroupCount - 1).coerceAtLeast(0),
            markedNeedsReviewCount = (markedNeedsReviewCount - 1).coerceAtLeast(0),
            undoBurstGroupId = null,
            undoDecision = null,
        )
        ReviewSessionDecision.RestoreAutomaticRecommendation -> this
        ReviewSessionDecision.ClearRecommendation -> this
        ReviewSessionDecision.KeepAllCandidates -> this
        ReviewSessionDecision.HideLowScoreCandidates -> this
        ReviewSessionDecision.SkipCurrent -> this
        null -> this
    }

internal fun reviewModeSessionExitSummaryText(
    session: ReviewModeSessionUi,
    remainingReviewGroupCount: Int,
    lowScoreCandidateCount: Int,
): String =
    buildList {
        add("已处理 ${session.processedGroupCount} 组")
        add("接受推荐 ${session.acceptedRecommendationCount}")
        if (session.manualOverrideCount > 0) add("手动调整 ${session.manualOverrideCount}")
        add("标记复核 ${session.markedNeedsReviewCount}")
        if (session.restoredAutomaticCount > 0) add("恢复自动 ${session.restoredAutomaticCount}")
        if (session.clearedRecommendationCount > 0) add("清除推荐 ${session.clearedRecommendationCount}")
        if (session.keptAllCandidatesCount > 0) add("保留全部 ${session.keptAllCandidatesCount}")
        if (session.hiddenLowScoreCount > 0) add("隐藏低分 ${session.hiddenLowScoreCount}")
        if (session.skippedCount > 0) add("跳过 ${session.skippedCount}")
        add("当前队列剩余 ${remainingReviewGroupCount.coerceAtLeast(0)}")
        add("低分候选 ${lowScoreCandidateCount.coerceAtLeast(0)}")
    }.joinToString(" · ")

internal enum class ReviewDecisionAction {
    AcceptRecommendedBest,
    MarkNeedsReview,
    RestoreAutomaticRecommendation,
}

internal data class ManualBestOverrideTarget(
    val burstGroupId: String,
    val bestAssetGroupId: String,
)

internal data class ManualBurstSplitTarget(
    val burstGroupId: String,
    val memberGroupId: String,
)

internal data class ManualBurstMergeTarget(
    val targetBurstGroupId: String,
    val memberGroupId: String,
)

internal fun reviewDecisionBurstGroupId(
    asset: InboxAsset,
    action: ReviewDecisionAction,
): String? {
    val burstGroupId = asset.burst?.burstGroupId?.takeIf { it.isNotBlank() } ?: return null
    return when (action) {
        ReviewDecisionAction.AcceptRecommendedBest ->
            burstGroupId.takeIf { asset.isBestRecommendedAsset() }
        ReviewDecisionAction.MarkNeedsReview -> burstGroupId
        ReviewDecisionAction.RestoreAutomaticRecommendation ->
            burstGroupId.takeIf {
                asset.burst.recommendationStatus.equals("user_overridden", ignoreCase = true)
            }
    }
}

internal fun manualBestOverrideTarget(asset: InboxAsset): ManualBestOverrideTarget? {
    if (asset.isBestRecommendedAsset()) {
        return null
    }
    val burstGroupId = asset.burst?.burstGroupId?.takeIf { it.isNotBlank() } ?: return null
    val groupId = asset.groupMoveId() ?: return null
    return ManualBestOverrideTarget(
        burstGroupId = burstGroupId,
        bestAssetGroupId = groupId,
    )
}

internal fun manualBurstSplitTarget(asset: InboxAsset): ManualBurstSplitTarget? {
    val burst = asset.burst ?: return null
    if (burst.memberCount <= 1) {
        return null
    }
    val burstGroupId = burst.burstGroupId.takeIf { it.isNotBlank() } ?: return null
    val memberGroupId = asset.groupMoveId() ?: return null
    return ManualBurstSplitTarget(
        burstGroupId = burstGroupId,
        memberGroupId = memberGroupId,
    )
}

internal fun reviewModeManualSplitTarget(
    asset: InboxAsset?,
    actionsEnabled: Boolean,
): ManualBurstSplitTarget? {
    if (!actionsEnabled || asset == null) {
        return null
    }
    return manualBurstSplitTarget(asset)
}

internal fun projectPhotoGridItems(assets: List<InboxAsset>): List<ProjectPhotoGridItemUi> {
    val orderedKeys = mutableListOf<String>()
    val grouped = linkedMapOf<String, MutableList<InboxAsset>>()
    assets.forEach { asset ->
        val burst = asset.burst
        val burstGroupId = burst?.burstGroupId?.takeIf { it.isNotBlank() }
        val key = if (burstGroupId != null && burst.memberCount > 1) {
            "burst:$burstGroupId"
        } else {
            "asset:${asset.assetSelectionId()}"
        }
        val members = grouped.getOrPut(key) {
            orderedKeys += key
            mutableListOf()
        }
        if (members.none { it.assetSelectionId() == asset.assetSelectionId() }) {
            members += asset
        }
    }
    return orderedKeys.mapNotNull { key ->
        val members = grouped[key].orEmpty()
        val sortedMembers = if (key.startsWith("burst:")) {
            members.sortedWith(
                compareBy<InboxAsset> { it.burst?.memberRank ?: Int.MAX_VALUE }
                    .thenByDescending { it.quality?.overall ?: -1.0 },
            )
        } else {
            members
        }
        val cover = if (key.startsWith("burst:")) {
            sortedMembers.burstCoverAsset()
        } else {
            sortedMembers.firstOrNull()
        } ?: return@mapNotNull null
        ProjectPhotoGridItemUi(
            key = key,
            coverAsset = cover,
            members = sortedMembers,
        )
    }
}

internal fun adjacentBurstMemberAsset(
    currentAsset: InboxAsset,
    allProjectAssets: List<InboxAsset>,
    direction: DetailNavigationDirection,
): InboxAsset? {
    val members = burstMemberFilmstrip(currentAsset, allProjectAssets).map { it.asset }
    val currentIndex = members.indexOfFirst { it.assetSelectionId() == currentAsset.assetSelectionId() }
    if (currentIndex < 0) {
        return null
    }
    val targetIndex = when (direction) {
        DetailNavigationDirection.Previous -> currentIndex - 1
        DetailNavigationDirection.Next -> currentIndex + 1
    }
    return members.getOrNull(targetIndex)
}

internal fun adjacentProjectGridAsset(
    currentAsset: InboxAsset,
    visibleAssets: List<InboxAsset>,
    direction: DetailNavigationDirection,
): InboxAsset? {
    val gridItems = projectPhotoGridItems(visibleAssets)
    val currentId = currentAsset.assetSelectionId()
    val currentIndex = gridItems.indexOfFirst { item ->
        item.coverAsset.assetSelectionId() == currentId ||
            item.members.any { member -> member.assetSelectionId() == currentId }
    }
    if (currentIndex < 0) {
        return null
    }
    val targetIndex = when (direction) {
        DetailNavigationDirection.Previous -> currentIndex - 1
        DetailNavigationDirection.Next -> currentIndex + 1
    }
    return gridItems.getOrNull(targetIndex)?.coverAsset
}

private fun List<InboxAsset>.burstCoverAsset(): InboxAsset? {
    if (isEmpty()) {
        return null
    }
    val bestGroupId = firstNotNullOfOrNull { asset ->
        asset.burst?.bestAssetGroupId?.takeIf { it.isNotBlank() }
    }
    return if (bestGroupId != null) {
        firstOrNull { asset ->
            asset.id == bestGroupId ||
                asset.groupMoveId() == bestGroupId ||
                asset.assetSelectionId() == bestGroupId
        }
    } else {
        null
    } ?: maxByOrNull { it.quality?.overall ?: -1.0 }
        ?: firstOrNull()
}

internal fun manualBurstMergeTarget(selectedAssets: List<InboxAsset>): ManualBurstMergeTarget? {
    val target = selectedAssets.firstOrNull() ?: return null
    val targetBurstGroupId = target.burst?.burstGroupId?.takeIf { it.isNotBlank() } ?: return null
    val source = selectedAssets
        .drop(1)
        .firstOrNull { asset ->
            asset.groupMoveId() != null && asset.burst?.burstGroupId != targetBurstGroupId
        } ?: return null
    return ManualBurstMergeTarget(
        targetBurstGroupId = targetBurstGroupId,
        memberGroupId = source.groupMoveId() ?: return null,
    )
}

internal fun photoDetailDecisionUi(
    asset: InboxAsset,
    actionsEnabled: Boolean,
): PhotoDetailDecisionUi {
    val acceptRecommendedBestBurstGroupId =
        reviewDecisionBurstGroupId(asset, ReviewDecisionAction.AcceptRecommendedBest)
    val markNeedsReviewBurstGroupId =
        reviewDecisionBurstGroupId(asset, ReviewDecisionAction.MarkNeedsReview)
    val restoreAutomaticBurstGroupId =
        reviewDecisionBurstGroupId(asset, ReviewDecisionAction.RestoreAutomaticRecommendation)
    val overrideRecommendedBestTarget = manualBestOverrideTarget(asset)
    val splitBurstTarget = manualBurstSplitTarget(asset)
    val hasAction = acceptRecommendedBestBurstGroupId != null ||
        markNeedsReviewBurstGroupId != null ||
        restoreAutomaticBurstGroupId != null ||
        overrideRecommendedBestTarget != null ||
        splitBurstTarget != null
    val disabledReason = when {
        !actionsEnabled && hasAction -> "正在处理上一项操作"
        acceptRecommendedBestBurstGroupId == null && markNeedsReviewBurstGroupId != null ->
            "当前照片不是推荐优选"
        else -> null
    }
    return PhotoDetailDecisionUi(
        acceptRecommendedBestBurstGroupId = acceptRecommendedBestBurstGroupId,
        markNeedsReviewBurstGroupId = markNeedsReviewBurstGroupId,
        restoreAutomaticBurstGroupId = restoreAutomaticBurstGroupId,
        overrideRecommendedBestTarget = overrideRecommendedBestTarget,
        splitBurstTarget = splitBurstTarget,
        acceptRecommendedBestEnabled = actionsEnabled && acceptRecommendedBestBurstGroupId != null,
        markNeedsReviewEnabled = actionsEnabled && markNeedsReviewBurstGroupId != null,
        restoreAutomaticEnabled = actionsEnabled && restoreAutomaticBurstGroupId != null,
        overrideRecommendedBestEnabled = actionsEnabled && overrideRecommendedBestTarget != null,
        splitBurstEnabled = actionsEnabled && splitBurstTarget != null,
        disabledReason = disabledReason,
    )
}

internal fun burstMemberFilmstrip(
    currentAsset: InboxAsset,
    allProjectAssets: List<InboxAsset>,
): List<BurstMemberFilmstripItemUi> {
    val burstGroupId = currentAsset.burst?.burstGroupId?.takeIf { it.isNotBlank() } ?: return emptyList()
    val currentId = currentAsset.assetSelectionId()
    val members = allProjectAssets
        .asSequence()
        .filter { asset -> asset.burst?.burstGroupId == burstGroupId }
        .distinctBy { it.assetSelectionId() }
        .sortedWith(
            compareBy<InboxAsset> { it.burst?.memberRank ?: Int.MAX_VALUE }
                .thenByDescending { it.quality?.overall ?: -1.0 },
        )
        .toList()
    if (members.size <= 1) {
        return emptyList()
    }
    return members.map { asset ->
        BurstMemberFilmstripItemUi(
            asset = asset,
            badgeText = burstMemberBadgeText(asset, currentId),
            scoreText = asset.qualityScoreText(),
        )
    }
}

internal fun burstComparisonItems(
    currentAsset: InboxAsset,
    allProjectAssets: List<InboxAsset>,
    maxItems: Int = 3,
): List<BurstMemberFilmstripItemUi> {
    val filmstrip = burstMemberFilmstrip(currentAsset, allProjectAssets)
    if (filmstrip.size <= 1 || maxItems <= 1) {
        return emptyList()
    }
    val currentId = currentAsset.assetSelectionId()
    val selected = mutableListOf<BurstMemberFilmstripItemUi>()

    fun addIfMissing(item: BurstMemberFilmstripItemUi?) {
        if (item != null && selected.none { it.asset.assetSelectionId() == item.asset.assetSelectionId() }) {
            selected += item
        }
    }

    addIfMissing(filmstrip.firstOrNull { it.asset.assetSelectionId() == currentId })
    addIfMissing(filmstrip.firstOrNull { it.asset.isBestRecommendedAsset() })
    filmstrip
        .asSequence()
        .filter { item -> selected.none { it.asset.assetSelectionId() == item.asset.assetSelectionId() } }
        .sortedWith(
            compareByDescending<BurstMemberFilmstripItemUi> { it.asset.quality?.overall ?: -1.0 }
                .thenBy { it.asset.burst?.memberRank ?: Int.MAX_VALUE },
        )
        .take(maxItems - selected.size)
        .forEach(::addIfMissing)

    return selected.take(maxItems).takeIf { it.size > 1 }.orEmpty()
}

private fun burstMemberBadgeText(asset: InboxAsset, currentId: String): String =
    when {
        asset.isBestRecommendedAsset() -> "最佳"
        asset.assetSelectionId() == currentId -> "当前"
        (asset.quality?.overall ?: 1.0) < 0.4 -> "低分"
        asset.burst?.recommendationStatus.equals("needs_review", ignoreCase = true) -> "需复核"
        else -> "备选"
    }

package com.cameraconnector.app.ui

import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetQuery
import com.cameraconnector.app.core.EvaluationRunUi
import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.PromptProfileUi
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.ProjectState
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
internal data class PhotoDetailDecisionUi(
    val splitBurstTarget: ManualBurstSplitTarget?,
    val splitBurstEnabled: Boolean,
    val disabledReason: String?,
) {
    val hasAnyAction: Boolean
        get() = splitBurstTarget != null
}

internal data class BurstMemberFilmstripItemUi(
    val asset: InboxAsset,
    val badgeText: String,
    val scoreText: String?,
)

internal data class ProjectIntelligenceSettingsUi(
    val modelEvaluationEnabled: Boolean,
    val autoEvaluateOnUpload: Boolean,
    val autoBurstRecommendationEnabled: Boolean,
    val projectRecommendationMode: String,
    val sceneProfile: String,
    val promptProfileId: String?,
    val cvPolicy: String,
    val allowRiskyModelSelects: Boolean,
    val providerConfigured: Boolean,
) {
    val modelEvaluationToggleEnabled: Boolean
        get() = providerConfigured
}

internal data class ManualProjectRecommendationActionUi(
    val enabled: Boolean,
    val ctaLabel: String,
    val disabledReason: String?,
)

internal interface ProjectRecommendationGateway {
    suspend fun generateProjectRecommendation(projectId: String): EvaluationRunUi
}

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
    All("全部评价", null),
    Excellent("80+", 80.0),
    Usable("60+", 60.0),
}

internal enum class ProjectPhotoCollection(val label: String) {
    All("全部"),
    ModelSelects("模型优选"),
    Favorites("收藏"),
    Marked("标记"),
    QualityRisk("质量风险"),
    PendingAnalysis("待分析"),
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
): Boolean = true

internal fun ProjectState.activeProjectSummary(): ProjectSummary? =
    activeProjectId?.let { id -> projects.firstOrNull { it.id == id } }

internal fun ProjectState.groupMoveTargets(sourceProjectId: String?): List<ProjectSummary> {
    val sourceId = sourceProjectId?.takeIf { it.isNotBlank() } ?: return emptyList()
    return projects.filter { project ->
        project.id != sourceId && project.canAcceptMovedGroups
    }
}

internal fun projectIntelligenceSettingsUi(
    settings: ProjectEvaluationSettingsUi,
    providerConfigured: Boolean,
): ProjectIntelligenceSettingsUi =
    ProjectIntelligenceSettingsUi(
        modelEvaluationEnabled = settings.modelEvaluationEnabled && providerConfigured,
        autoEvaluateOnUpload = settings.autoEvaluateOnUpload,
        autoBurstRecommendationEnabled = settings.autoBurstRecommendationEnabled,
        projectRecommendationMode = "manual",
        sceneProfile = settings.sceneProfile.ifBlank { "general" },
        promptProfileId = settings.promptProfileId,
        cvPolicy = settings.cvPolicy.ifBlank { "standard" },
        allowRiskyModelSelects = settings.allowRiskyModelSelects,
        providerConfigured = providerConfigured,
    )

internal fun promptStyleTagsText(profile: PromptProfileUi): String =
    profile.styleTags
        .filter { it.isNotBlank() }
        .map(::promptStyleTagLabel)
        .joinToString(" / ")

internal fun promptProfileDisplayName(profile: PromptProfileUi): String {
    val name = profile.name.trim()
    val normalized = name.lowercase()
    return when {
        normalized == "general" ||
            normalized == "general balanced" ||
            normalized == "general default" -> "通用评价"
        normalized.contains("portrait") && normalized.contains("conservative") -> "人像稳健"
        normalized.contains("portrait") -> "人像评价"
        normalized.contains("landscape") -> "风光评价"
        normalized.contains("action") -> "动作评价"
        normalized.contains("custom") -> name.replace("Custom", "自定义")
        else -> name.ifBlank { "未命名 Prompt" }
    }
}

internal fun promptStyleTagLabel(value: String): String =
    when (value.trim().lowercase()) {
        "general" -> "通用"
        "balanced" -> "均衡"
        "portrait" -> "人像"
        "action" -> "动作"
        "landscape" -> "风光"
        "conservative" -> "稳健"
        "editorial" -> "编辑向"
        "technical" -> "技术"
        "creative" -> "创意"
        else -> value
    }

internal fun sceneProfileLabel(value: String): String =
    when (value.trim().lowercase()) {
        "general" -> "通用"
        "portrait" -> "人像"
        "action" -> "动作"
        "landscape" -> "风光"
        "custom" -> "自定义"
        else -> value
    }

internal fun cvPolicyLabel(value: String): String =
    when (value.trim().lowercase()) {
        "loose" -> "宽松"
        "standard" -> "标准"
        "strict" -> "严格"
        else -> value
    }

internal fun manualProjectRecommendationActionUi(
    provider: ModelProviderSettingsUi,
    settings: ProjectEvaluationSettingsUi,
    actionInFlight: Boolean,
): ManualProjectRecommendationActionUi {
    if (!provider.configured) {
        return ManualProjectRecommendationActionUi(
            enabled = false,
            ctaLabel = "生成项目优选",
            disabledReason = "模型服务未配置",
        )
    }
    if (settings.projectId.isBlank()) {
        return ManualProjectRecommendationActionUi(
            enabled = false,
            ctaLabel = "生成项目优选",
            disabledReason = "请先进入项目",
        )
    }
    return ManualProjectRecommendationActionUi(
        enabled = !actionInFlight && settings.projectRecommendationMode.equals("manual", ignoreCase = true),
        ctaLabel = "生成项目优选",
        disabledReason = if (actionInFlight) "项目优选生成中" else null,
    )
}

internal suspend fun runManualProjectRecommendationAction(
    projectId: String?,
    provider: ModelProviderSettingsUi,
    gateway: ProjectRecommendationGateway,
    onFeedback: (String) -> Unit,
): EvaluationRunUi? {
    if (!provider.configured) {
        onFeedback("请先配置模型服务")
        return null
    }
    val activeProjectId = projectId?.takeIf { it.isNotBlank() }
    if (activeProjectId == null) {
        onFeedback("请先进入项目")
        return null
    }
    val run = gateway.generateProjectRecommendation(activeProjectId)
    onFeedback(projectRecommendationRunFeedback(run))
    return run
}

internal fun projectRecommendationRunFeedback(run: EvaluationRunUi): String =
    "项目优选：${evaluationRunStatusLabel(run.status.ifBlank { "updated" })}"

internal fun evaluationRunStatusLabel(value: String?): String =
    when (value?.trim()?.lowercase()) {
        "ready", "done", "completed", "updated" -> "已更新"
        "running", "processing" -> "生成中"
        "queued", "pending" -> "等待中"
        "failed", "error" -> "失败"
        null, "" -> "未知"
        else -> value
    }

internal fun projectRecommendationFeedbackForActiveProject(
    run: EvaluationRunUi,
    activeProjectId: String?,
): String? {
    val projectId = activeProjectId?.takeIf { it.isNotBlank() } ?: return null
    return if (run.projectId == projectId) projectRecommendationRunFeedback(run) else null
}

internal fun scopedProjectRecommendationRun(
    run: EvaluationRunUi?,
    activeProjectId: String?,
): EvaluationRunUi? {
    val projectId = activeProjectId?.takeIf { it.isNotBlank() } ?: return null
    return run?.takeIf { it.projectId == projectId }
}

internal fun modelEvaluationSourceLabel(evaluatorKind: String?): String =
    when (evaluatorKind?.trim()?.lowercase()) {
        "local_stub" -> "本地占位结果"
        "imported" -> "导入结果"
        "llm_vlm" -> "模型评价"
        else -> evaluatorKind?.takeIf { it.isNotBlank() } ?: "未知"
    }

internal fun providerBatchSizeValue(value: Int): Int =
    value.coerceIn(1, 8)

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

internal fun assetListQuery(
    selectedCollection: ProjectPhotoCollection,
    selectedAccount: String?,
    selectedFilter: InboxFilter,
    selectedSort: PhotoSortMode,
    selectedScoreFilter: ScoreFilter,
): InboxAssetQuery {
    val scoreMin = selectedScoreFilter.scoreMin
    val favorite = if (selectedCollection == ProjectPhotoCollection.Favorites) true else null
    val marked = if (selectedCollection == ProjectPhotoCollection.Marked) true else null
    val reviewQueue = when (selectedCollection) {
        ProjectPhotoCollection.ModelSelects -> "model_selects"
        ProjectPhotoCollection.QualityRisk -> "quality_risk"
        ProjectPhotoCollection.PendingAnalysis -> "pending_analysis"
        else -> null
    }
    return InboxAssetQuery(
        username = selectedAccount,
        role = selectedFilter.assetRole(),
        sort = if (scoreMin != null && selectedSort == PhotoSortMode.LatestReceived) {
            PhotoSortMode.GroupBestScore
        } else {
            selectedSort
        },
        scoreMin = scoreMin,
        reviewQueue = reviewQueue,
        favorite = favorite,
        marked = marked,
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
    burst?.bestScore ?: modelScore?.toDouble() ?: quality?.overall

private fun normalizedProjectQueryScore(value: Double): Double =
    if (value > 1.0) value / 100.0 else value

internal data class ManualBurstSplitTarget(
    val burstGroupId: String,
    val memberGroupId: String,
)

internal data class ManualBurstMergeTarget(
    val targetBurstGroupId: String,
    val memberGroupId: String,
)

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
            members.sortedWith(burstMemberOrderComparator())
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

internal fun photoDetailBurstPositionText(
    asset: InboxAsset,
    burstMembers: List<InboxAsset>,
): String? {
    val burst = asset.burst ?: return null
    if (burst.memberCount <= 1) {
        return null
    }
    val stableAssetId = asset.assetSelectionId()
    val orderedMembers = burstMembers
        .distinctBy { it.assetSelectionId() }
        .sortedWith(burstMemberOrderComparator())
    val rank = orderedMembers
        .indexOfFirst { it.assetSelectionId() == stableAssetId }
        .takeIf { it >= 0 }
        ?.plus(1)
        ?: 1
    return "$rank/${burst.memberCount}"
}

internal fun detailBurstMemberIndex(
    asset: InboxAsset,
    burstMembers: List<BurstMemberFilmstripItemUi>,
): Int =
    burstMembers.indexOfFirst { item ->
        item.asset.assetSelectionId() == asset.assetSelectionId()
    }.takeIf { it >= 0 } ?: Int.MAX_VALUE

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
    val splitBurstTarget = manualBurstSplitTarget(asset)
    return PhotoDetailDecisionUi(
        splitBurstTarget = splitBurstTarget,
        splitBurstEnabled = actionsEnabled && splitBurstTarget != null,
        disabledReason = null,
    )
}

internal fun photoDetailActionBarVisible(
    decision: PhotoDetailDecisionUi,
    hasActionCallbacks: Boolean,
): Boolean = hasActionCallbacks

internal fun photoDetailFavoriteSelected(
    asset: InboxAsset,
): Boolean =
    asset.userMarks.favorite

internal fun photoDetailMarkedSelected(
    asset: InboxAsset,
): Boolean =
    asset.userMarks.marked

internal fun detailPageSlideOffset(
    fullWidth: Int,
    direction: DetailNavigationDirection?,
    entering: Boolean,
): Int {
    val directionMultiplier = if (direction == DetailNavigationDirection.Previous) -1 else 1
    return if (entering) {
        fullWidth * directionMultiplier
    } else {
        -fullWidth * directionMultiplier
    }
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
        .sortedWith(burstMemberOrderComparator())
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
                .thenBy { it.asset.receivedAt.toLongOrNull() ?: Long.MAX_VALUE }
                .thenBy { burstMemberOrderKey(it.asset) },
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

private fun burstMemberOrderComparator(): Comparator<InboxAsset> =
    compareBy<InboxAsset> { it.receivedAt.toLongOrNull() ?: Long.MAX_VALUE }
        .thenBy(::burstMemberOrderKey)
        .thenBy { it.assetSelectionId() }

private fun burstMemberOrderKey(asset: InboxAsset): String =
    asset.groupKey.ifBlank { asset.displayPath }

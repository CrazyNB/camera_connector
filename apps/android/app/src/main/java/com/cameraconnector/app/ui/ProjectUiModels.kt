package com.cameraconnector.app.ui

import com.cameraconnector.app.core.EvaluationRunUi
import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.PromptPackUi
import kotlin.math.abs

internal enum class GlobalDestination(val label: String) {
    Projects("\u9879\u76ee"),
    Accounts("\u8d26\u53f7"),
    Settings("\u8bbe\u7f6e"),
}

internal enum class ProjectDestination(val label: String) {
    Photos("\u7167\u7247"),
}

internal data class ProjectWorkspaceNavigationState(
    val workspaceOpen: Boolean,
)

internal fun defaultProjectDestination(): ProjectDestination =
    ProjectDestination.Photos

internal fun projectWorkspaceStateAfterBottomDestinationClick(
    current: ProjectWorkspaceNavigationState,
    destination: GlobalDestination,
    collapseCurrentProjectWorkspace: Boolean = false,
): ProjectWorkspaceNavigationState =
    when {
        destination == GlobalDestination.Projects && collapseCurrentProjectWorkspace ->
            current.copy(workspaceOpen = false)

        else -> current
    }

internal fun projectWorkspaceStateAfterOpenProjects(
    current: ProjectWorkspaceNavigationState,
): ProjectWorkspaceNavigationState =
    current.copy(workspaceOpen = false)

internal fun projectWorkspaceVisible(
    workspaceOpen: Boolean,
    activeProjectId: String?,
): Boolean =
    workspaceOpen && !activeProjectId.isNullOrBlank()

internal fun ProjectDestination.assetScreenTitle(): String =
    "\u9879\u76ee\u7167\u7247"

internal fun ProjectDestination.assetScreenSubtitle(): String =
    "\u7167\u7247\u5206\u7ec4\u4e0e\u539f\u59cb\u6587\u4ef6"

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
    val asset: ProjectAsset,
    val badgeText: String,
    val scoreText: String?,
)

internal data class BurstPreviewTileUi(
    val positionText: String,
    val scoreText: String?,
    val modelSelected: Boolean,
    val auxiliaryBadges: List<String>,
)

internal data class ManualProjectRecommendationActionUi(
    val enabled: Boolean,
    val ctaLabel: String,
    val disabledReason: String?,
)

internal data class ProjectPhotoGridItemUi(
    val key: String,
    val coverAsset: ProjectAsset,
    val members: List<ProjectAsset>,
) {
    val isBurstGroup: Boolean
        get() = key.startsWith("burst:") && members.size > 1
}

internal data class SelectedBurstEvaluationTargetUi(
    val burstGroupId: String,
    val members: List<ProjectAsset>,
)

internal data class ProjectPhotoEvaluationTargetsUi(
    val assetGroups: List<ProjectAsset>,
    val burstGroups: List<SelectedBurstEvaluationTargetUi>,
)

internal enum class DetailNavigationDirection {
    Previous,
    Next,
}

internal enum class ProjectPhotoCollection(val label: String) {
    All("\u5168\u90e8"),
    ModelSelects("\u6a21\u578b\u4f18\u9009"),
    Favorites("\u6536\u85cf"),
    Marked("\u6807\u8bb0"),
    TechnicalRisk("\u6280\u672f\u98ce\u9669"),
    PendingAnalysis("\u5f85\u5206\u6790"),
}

internal enum class ReceiverStartBlockReason {
    Busy,
    MissingAccount,
    MissingNotificationPermission,
}

internal fun receiverStartBlockReason(
    running: Boolean,
    busy: Boolean,
    actionsEnabled: Boolean,
    notificationPermissionGranted: Boolean,
    accountCount: Int,
): ReceiverStartBlockReason? = when {
    running -> null
    busy -> ReceiverStartBlockReason.Busy
    !actionsEnabled -> ReceiverStartBlockReason.Busy
    accountCount <= 0 -> ReceiverStartBlockReason.MissingAccount
    !notificationPermissionGranted -> ReceiverStartBlockReason.MissingNotificationPermission
    else -> null
}

internal fun receiverPhaseBusy(value: String): Boolean =
    value == "Starting" || value == "Stopping"

@Suppress("UNUSED_PARAMETER")
internal fun projectPhotoContentVisible(
    receiverRunning: Boolean,
): Boolean = true

internal fun ProjectState.activeProjectSummary(): ProjectSummary? =
    activeProjectId?.let { id -> projects.firstOrNull { it.id == id } }

internal fun modelProviderReadyForProject(
    settings: ProjectEvaluationSettingsUi,
    providerOptions: List<ModelProviderSettingsUi>,
): Boolean {
    val selectedProviderId = settings.modelProviderSettingsId?.takeIf { it.isNotBlank() } ?: return false
    return providerOptions.firstOrNull { it.settingsId == selectedProviderId }?.isReadyForModelWork() == true
}

internal fun projectSettingsAfterModelProviderSelection(
    settings: ProjectEvaluationSettingsUi,
    providerSettingsId: String,
): ProjectEvaluationSettingsUi =
    settings.copy(modelProviderSettingsId = providerSettingsId)

internal fun ModelProviderSettingsUi.isReadyForModelWork(): Boolean {
    if (!configured) {
        return false
    }
    return when (providerKind.lowercase()) {
        "openai", "custom" -> apiKeyConfigured
        "imported" -> true
        else -> false
    }
}

internal fun promptStyleTagsText(profile: PromptPackUi): String =
    profile.styleTags
        .filter { it.isNotBlank() }
        .map(::promptStyleTagLabel)
        .joinToString(" / ")

internal fun promptPackDisplayName(profile: PromptPackUi): String {
    val name = profile.name.trim()
    val normalized = name.lowercase()
    return when {
        normalized == "general" ||
            normalized == "general balanced" ||
            normalized == "general default" -> "\u901a\u7528\u8bc4\u4ef7"
        normalized.contains("portrait") && normalized.contains("conservative") -> "\u4eba\u50cf\u7a33\u5065"
        normalized.contains("portrait") -> "\u4eba\u50cf\u8bc4\u4ef7"
        normalized.contains("landscape") -> "\u98ce\u5149\u8bc4\u4ef7"
        normalized.contains("action") -> "\u52a8\u4f5c\u8bc4\u4ef7"
        normalized.contains("custom") -> name.replace("Custom", "\u81ea\u5b9a\u4e49")
        else -> name.ifBlank { "\u672a\u547d\u540d\u63d0\u793a\u8bcd" }
    }
}

internal fun promptPackageFolder(profile: PromptPackUi): String =
    profile.distributionFolder.trim().ifBlank { "user" }

internal fun promptPackageLabel(folder: String): String =
    when (folder.trim().ifBlank { "user" }) {
        "user" -> "\u6211\u7684\u63d0\u793a\u8bcd\u5305"
        "builtin" -> "\u5185\u7f6e\u63d0\u793a\u8bcd\u5305"
        else -> folder
    }

internal fun promptStyleTagLabel(value: String): String =
    when (value.trim().lowercase()) {
        "general" -> "\u901a\u7528"
        "balanced" -> "\u5747\u8861"
        "portrait" -> "\u4eba\u50cf"
        "action" -> "\u52a8\u4f5c"
        "landscape" -> "\u98ce\u5149"
        "conservative" -> "\u7a33\u5065"
        "editorial" -> "\u7f16\u8f91"
        "technical" -> "\u6280\u672f"
        "creative" -> "\u521b\u610f"
        else -> value
    }

internal fun sceneProfileLabel(value: String): String =
    when (value.trim().lowercase()) {
        "general" -> "\u901a\u7528"
        "portrait" -> "\u4eba\u50cf"
        "action" -> "\u52a8\u4f5c"
        "landscape" -> "\u98ce\u5149"
        "custom" -> "\u81ea\u5b9a\u4e49"
        else -> value
    }

internal fun cvPolicyLabel(value: String): String =
    when (value.trim().lowercase()) {
        "loose" -> "\u5bbd\u677e"
        "standard" -> "\u6807\u51c6"
        "strict" -> "\u4e25\u683c"
        else -> value
    }

internal fun manualProjectRecommendationActionUi(
    providerConfigured: Boolean,
    settings: ProjectEvaluationSettingsUi,
    actionInFlight: Boolean,
): ManualProjectRecommendationActionUi {
    if (!providerConfigured) {
        return ManualProjectRecommendationActionUi(
            enabled = false,
            ctaLabel = "\u751f\u6210\u9879\u76ee\u4f18\u9009",
            disabledReason = "\u6a21\u578b\u670d\u52a1\u672a\u914d\u7f6e",
        )
    }
    if (settings.projectId.isBlank()) {
        return ManualProjectRecommendationActionUi(
            enabled = false,
            ctaLabel = "\u751f\u6210\u9879\u76ee\u4f18\u9009",
            disabledReason = "\u8bf7\u5148\u8fdb\u5165\u9879\u76ee",
        )
    }
    return ManualProjectRecommendationActionUi(
        enabled = !actionInFlight && settings.projectRecommendationMode.equals("manual", ignoreCase = true),
        ctaLabel = "\u751f\u6210\u9879\u76ee\u4f18\u9009",
        disabledReason = if (actionInFlight) "\u9879\u76ee\u4f18\u9009\u751f\u6210\u4e2d" else null,
    )
}


internal fun projectRecommendationRunFeedback(run: EvaluationRunUi): String =
    "\u9879\u76ee\u4f18\u9009\uff1a${evaluationRunStatusLabel(run.status.ifBlank { "updated" })}"

internal fun evaluationRunStatusLabel(value: String?): String =
    when (value?.trim()?.lowercase()) {
        "ready", "done", "completed", "updated" -> "\u5df2\u66f4\u65b0"
        "running", "processing" -> "\u751f\u6210\u4e2d"
        "queued", "pending" -> "\u7b49\u5f85\u4e2d"
        "failed", "error" -> "\u5931\u8d25"
        null, "" -> "\u672a\u77e5"
        else -> value
    }

internal fun projectRecommendationFeedbackForActiveProject(
    run: EvaluationRunUi,
    activeProjectId: String?,
): String? {
    val projectId = activeProjectId?.takeIf { it.isNotBlank() } ?: return null
    return if (run.projectId == projectId) projectRecommendationRunFeedback(run) else null
}

internal fun activeProjectRecommendationRun(
    run: EvaluationRunUi?,
    activeProjectId: String?,
): EvaluationRunUi? {
    val projectId = activeProjectId?.takeIf { it.isNotBlank() } ?: return null
    return run?.takeIf { it.projectId == projectId }
}

internal fun modelEvaluationSourceLabel(evaluatorKind: String?): String =
    when (evaluatorKind?.trim()?.lowercase()) {
        "local_stub" -> "\u672c\u5730\u5206\u6790\u7ed3\u679c"
        "imported" -> "\u5bfc\u5165\u7ed3\u679c"
        "llm_vlm" -> "\u6a21\u578b\u8bc4\u4ef7"
        else -> evaluatorKind?.takeIf { it.isNotBlank() } ?: "\u672a\u77e5"
    }

internal fun providerBatchSizeValue(value: Int): Int =
    value.coerceIn(1, 8)

internal fun ProjectAsset.assetSelectionId(): String =
    id.ifBlank { displayPath }

internal fun toggleAssetSelection(
    selectedIds: List<String>,
    asset: ProjectAsset,
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
    assets: List<ProjectAsset>,
    selectedIds: List<String>,
): List<ProjectAsset> {
    if (selectedIds.isEmpty()) {
        return emptyList()
    }
    val selected = selectedIds.toSet()
    return assets.filter { it.assetSelectionId() in selected }
}

internal fun togglePhotoGridItemSelection(
    selectedIds: List<String>,
    item: ProjectPhotoGridItemUi,
): List<String> {
    val id = item.key.takeIf { it.isNotBlank() } ?: return selectedIds
    return if (id in selectedIds) {
        selectedIds.filterNot { it == id }
    } else {
        selectedIds + id
    }
}

internal fun selectedPhotoGridItemsFromIds(
    gridItems: List<ProjectPhotoGridItemUi>,
    selectedIds: List<String>,
): List<ProjectPhotoGridItemUi> {
    if (selectedIds.isEmpty()) {
        return emptyList()
    }
    val selected = selectedIds.toSet()
    return gridItems.filter { it.key in selected }
}

internal fun projectPhotoEvaluationTargets(
    selectedItems: List<ProjectPhotoGridItemUi>,
): ProjectPhotoEvaluationTargetsUi {
    val burstGroups = selectedItems
        .filter { it.isBurstGroup }
        .mapNotNull { item ->
            val burstGroupId = item.coverAsset.burst?.burstGroupId?.takeIf { it.isNotBlank() }
                ?: item.key.removePrefix("burst:").takeIf { it.isNotBlank() }
                ?: return@mapNotNull null
            SelectedBurstEvaluationTargetUi(
                burstGroupId = burstGroupId,
                members = item.members.distinctBy { it.assetSelectionId() },
            )
        }
    val burstMemberIds = burstGroups
        .flatMap { target -> target.members.map { it.assetSelectionId() } }
        .toSet()
    val assetGroups = selectedItems
        .filterNot { it.isBurstGroup }
        .map { it.coverAsset }
        .filterNot { it.assetSelectionId() in burstMemberIds }
        .distinctBy { it.assetSelectionId() }
    return ProjectPhotoEvaluationTargetsUi(
        assetGroups = assetGroups,
        burstGroups = burstGroups,
    )
}

internal fun refreshedSelectedPhoto(
    selectedPhoto: ProjectAsset?,
    visibleAssets: List<ProjectAsset>,
): ProjectAsset? {
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
            selected -> "\u5f53\u524d\u9879\u76ee"
            project.canRestore -> "\u5df2\u5f52\u6863"
            else -> "\u6d3b\u8dc3"
        },
        canSelect = actionsEnabled && !selected && project.canBeActiveProject,
        canArchive = actionsEnabled && project.canArchive,
        canRename = actionsEnabled && project.canRename,
        canRestore = actionsEnabled && project.canRestore,
    )
}

internal fun assetListQuery(
    selectedCollection: ProjectPhotoCollection,
    selectedFilter: AssetFormatFilter,
    selectedSort: PhotoSortMode,
): ProjectAssetQuery {
    val favorite = if (selectedCollection == ProjectPhotoCollection.Favorites) true else null
    val marked = if (selectedCollection == ProjectPhotoCollection.Marked) true else null
    val collection = when (selectedCollection) {
        ProjectPhotoCollection.ModelSelects -> "model_selects"
        ProjectPhotoCollection.TechnicalRisk -> "technical_risk"
        ProjectPhotoCollection.PendingAnalysis -> "pending_analysis"
        else -> null
    }
    return ProjectAssetQuery(
        role = selectedFilter.assetRole(),
        sort = selectedSort,
        collection = collection,
        favorite = favorite,
        marked = marked,
    )
}

internal fun photoListFilterSummary(
    selectedFilter: AssetFormatFilter,
    selectedSort: PhotoSortMode,
): String? =
    buildList {
        if (selectedFilter != AssetFormatFilter.All) {
            add(selectedFilter.label)
        }
        if (selectedSort != PhotoSortMode.LatestReceived) {
            add(selectedSort.label)
        }
    }.takeIf { it.isNotEmpty() }?.joinToString(" / ")

internal fun projectPhotoCollectionAssets(
    assets: List<ProjectAsset>,
    selectedFilter: AssetFormatFilter,
    selectedSort: PhotoSortMode,
): List<ProjectAsset> {
    val filtered = assets
        .asSequence()
        .filter { asset -> selectedFilter.matches(asset) }
        .toList()

    return when (selectedSort) {
        PhotoSortMode.LatestReceived -> filtered.sortedByDescending { it.receivedAt.toLongOrNull() ?: 0L }
        PhotoSortMode.Filename -> filtered.sortedBy { it.groupKey.ifBlank { it.displayPath } }
        PhotoSortMode.ModelScore -> filtered.sortedWith(
            compareByDescending<ProjectAsset> {
                it.projectQueryBestScore()?.let(::normalizedProjectQueryScore) ?: -1.0
            }.thenByDescending { it.receivedAt.toLongOrNull() ?: 0L },
        )
    }
}

private fun ProjectAsset.projectQueryBestScore(): Double? =
    burst?.bestScore ?: modelScore?.toDouble()

private fun normalizedProjectQueryScore(value: Double): Double =
    if (value > 1.0) value / 100.0 else value

internal data class ManualBurstSplitTarget(
    val burstGroupId: String,
    val memberGroupId: String,
)

internal data class ManualBurstMergeTarget(
    val memberGroupIds: List<String>,
)

internal fun manualBurstSplitTargets(selectedItems: List<ProjectPhotoGridItemUi>): List<ManualBurstSplitTarget> =
    selectedItems
        .flatMap { item ->
            if (item.isBurstGroup) {
                item.members
            } else {
                listOf(item.coverAsset)
            }
        }
        .mapNotNull(::manualBurstSplitTarget)
        .distinctBy { "${it.burstGroupId}\t${it.memberGroupId}" }

internal fun manualBurstSplitTarget(asset: ProjectAsset): ManualBurstSplitTarget? {
    val burst = asset.burst ?: return null
    if (burst.memberCount <= 1) {
        return null
    }
    val burstGroupId = burst.burstGroupId.takeIf { it.isNotBlank() } ?: return null
    val memberGroupId = asset.assetGroupId() ?: return null
    return ManualBurstSplitTarget(
        burstGroupId = burstGroupId,
        memberGroupId = memberGroupId,
    )
}

internal fun projectPhotoGridItems(assets: List<ProjectAsset>): List<ProjectPhotoGridItemUi> {
    val orderedKeys = mutableListOf<String>()
    val grouped = linkedMapOf<String, MutableList<ProjectAsset>>()
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
    currentAsset: ProjectAsset,
    allProjectAssets: List<ProjectAsset>,
    direction: DetailNavigationDirection,
): ProjectAsset? {
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
    asset: ProjectAsset,
    burstMembers: List<ProjectAsset>,
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
    asset: ProjectAsset,
    burstMembers: List<BurstMemberFilmstripItemUi>,
): Int =
    burstMembers.indexOfFirst { item ->
        item.asset.assetSelectionId() == asset.assetSelectionId()
    }.takeIf { it >= 0 } ?: Int.MAX_VALUE

internal fun photoDetailSelectionAfterDelete(
    deletedAsset: ProjectAsset,
    burstMembers: List<BurstMemberFilmstripItemUi>,
): ProjectAsset? = null

internal fun photoDetailSelectionAfterSplit(
    splitAsset: ProjectAsset,
    burstMembers: List<BurstMemberFilmstripItemUi>,
): ProjectAsset? =
    burstMembers
        .map { it.asset }
        .firstOrNull { it.assetSelectionId() != splitAsset.assetSelectionId() }

internal fun adjacentProjectGridAsset(
    currentAsset: ProjectAsset,
    visibleAssets: List<ProjectAsset>,
    direction: DetailNavigationDirection,
): ProjectAsset? {
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

private fun List<ProjectAsset>.burstCoverAsset(): ProjectAsset? {
    if (isEmpty()) {
        return null
    }
    val bestGroupId = firstNotNullOfOrNull { asset ->
        asset.burst?.bestAssetGroupId?.takeIf { it.isNotBlank() }
    }
    return if (bestGroupId != null) {
        firstOrNull { asset ->
            asset.id == bestGroupId ||
                asset.assetGroupId() == bestGroupId ||
                asset.assetSelectionId() == bestGroupId
        }
    } else {
        null
    } ?: maxByOrNull { it.modelScore ?: -1 }
        ?: firstOrNull()
}

internal fun manualBurstMergeTarget(selectedItems: List<ProjectPhotoGridItemUi>): ManualBurstMergeTarget? {
    val mergeContainerIds = selectedItems
        .mapNotNull { item -> item.mergeContainerId() }
        .distinct()
    if (mergeContainerIds.size < 2) {
        return null
    }
    val memberGroupIds = selectedItems
        .flatMap { item ->
            if (item.isBurstGroup) {
                item.members.mapNotNull { member -> member.assetGroupId() }
            } else {
                listOfNotNull(item.coverAsset.assetGroupId())
            }
        }
        .distinct()
    if (memberGroupIds.size < 2) {
        return null
    }
    return ManualBurstMergeTarget(
        memberGroupIds = memberGroupIds,
    )
}

private fun ProjectPhotoGridItemUi.mergeContainerId(): String? =
    coverAsset.burst
        ?.burstGroupId
        ?.takeIf { isBurstGroup && it.isNotBlank() }
        ?: coverAsset.assetGroupId()

internal fun photoDetailDecisionUi(
    asset: ProjectAsset,
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

internal fun photoDetailFavoriteSelected(asset: ProjectAsset): Boolean =
    asset.userMarks.favorite

internal fun photoDetailMarkedSelected(asset: ProjectAsset): Boolean =
    asset.userMarks.marked

internal fun burstPreviewTileUi(
    item: BurstMemberFilmstripItemUi,
    index: Int,
    total: Int,
): BurstPreviewTileUi =
    BurstPreviewTileUi(
        positionText = "${index + 1}/$total",
        scoreText = (item.scoreText ?: item.asset.modelScoreText())?.let { "\u8bc4\u5206 $it" },
        modelSelected = item.asset.isBestRecommendedAsset(),
        auxiliaryBadges = item.asset.tileAuxiliaryBadges(),
    )

internal fun burstMemberFilmstrip(
    currentAsset: ProjectAsset,
    allProjectAssets: List<ProjectAsset>,
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
            scoreText = asset.modelScoreText(),
        )
    }
}

internal fun burstComparisonItems(
    currentAsset: ProjectAsset,
    allProjectAssets: List<ProjectAsset>,
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
            compareByDescending<BurstMemberFilmstripItemUi> { it.asset.modelScore ?: -1 }
                .thenBy { it.asset.receivedAt.toLongOrNull() ?: Long.MAX_VALUE }
                .thenBy { burstMemberOrderKey(it.asset) },
        )
        .take(maxItems - selected.size)
        .forEach(::addIfMissing)

    return selected.take(maxItems).takeIf { it.size > 1 }.orEmpty()
}

private fun burstMemberBadgeText(asset: ProjectAsset, currentId: String): String =
    when {
        asset.isBestRecommendedAsset() -> "\u6700\u4f73"
        asset.assetSelectionId() == currentId -> "\u5f53\u524d"
        (asset.modelScore ?: 100) < 40 -> "\u4f4e\u5206"
        asset.hasTechnicalRisk() -> "\u98ce\u9669"
        else -> "\u5907\u9009"
    }
private fun burstMemberOrderComparator(): Comparator<ProjectAsset> =
    compareBy<ProjectAsset> { it.receivedAt.toLongOrNull() ?: Long.MAX_VALUE }
        .thenBy(::burstMemberOrderKey)
        .thenBy { it.assetSelectionId() }

private fun burstMemberOrderKey(asset: ProjectAsset): String =
    asset.groupKey.ifBlank { asset.displayPath }

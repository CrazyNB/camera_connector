package com.cameraconnector.app.ui

import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.PromptPackUi

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
)

internal data class PhotoDetailExportUi(
    val enabled: Boolean,
    val fileName: String,
    val unavailableReason: String?,
)

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

internal data class LanShareActionUi(
    val enabled: Boolean,
    val label: String,
    val disabledReason: String?,
)

internal enum class LanShareMenuAction {
    GuestSelection,
    ProjectSync,
}

internal data class LanShareMenuItemUi(
    val action: LanShareMenuAction,
    val label: String,
)

internal fun lanShareMenuItems(): List<LanShareMenuItemUi> =
    listOf(
        LanShareMenuItemUi(
            action = LanShareMenuAction.GuestSelection,
            label = "\u591a\u65b9\u7b5b\u9009",
        ),
        LanShareMenuItemUi(
            action = LanShareMenuAction.ProjectSync,
            label = "\u5c40\u57df\u7f51\u9879\u76ee\u5171\u4eab",
        ),
    )

internal fun lanShareDialogShowsUserVisibleLink(
    mode: LanShareMenuAction,
    sharingActive: Boolean,
): Boolean = mode == LanShareMenuAction.GuestSelection && sharingActive

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

internal enum class GuestMarkFilter(val label: String, val queryValue: String?) {
    All("全部访客", null),
    Favorite("访客收藏", "favorite"),
    Marked("访客标记", "marked"),
    Reject("访客删除", "reject"),
    None("未标记", "none"),
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

internal fun lanShareActionUi(
    activeProjectId: String?,
    assetCount: Int,
    running: Boolean,
): LanShareActionUi =
    when {
        running -> LanShareActionUi(
            enabled = false,
            label = "启动中",
            disabledReason = null,
        )
        activeProjectId.isNullOrBlank() -> LanShareActionUi(
            enabled = false,
            label = "局域网选片",
            disabledReason = "请先进入项目",
        )
        assetCount <= 0 -> LanShareActionUi(
            enabled = false,
            label = "局域网选片",
            disabledReason = "当前没有照片",
        )
        else -> LanShareActionUi(
            enabled = true,
            label = "局域网选片",
            disabledReason = null,
        )
    }

internal fun providerBatchSizeValue(value: Int): Int =
    value.coerceIn(1, 8)

internal fun ProjectAsset.assetSelectionId(): String =
    id.ifBlank { displayPath }

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
    selectedGuestMarkFilter: GuestMarkFilter = GuestMarkFilter.All,
    selectedMinModelScore: Int? = null,
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
        guestMark = selectedGuestMarkFilter.queryValue,
        minModelScore = selectedMinModelScore,
    )
}

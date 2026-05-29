package com.cameraconnector.app.ui

import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.ProjectState

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

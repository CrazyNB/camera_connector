package com.cameraconnector.app.ui

import com.cameraconnector.app.core.ProjectSummary

private const val SYSTEM_INBOX_PROJECT_ID = "project-inbox"

internal data class ProjectLifecycleUi(
    val statusLabel: String,
    val canSelect: Boolean,
    val canArchive: Boolean,
    val canRestore: Boolean,
)

internal fun projectLifecycleUi(
    project: ProjectSummary,
    selected: Boolean,
    actionsEnabled: Boolean,
): ProjectLifecycleUi {
    val archived = project.status.equals("archived", ignoreCase = true)
    val systemInbox = project.id == SYSTEM_INBOX_PROJECT_ID
    return ProjectLifecycleUi(
        statusLabel = when {
            selected -> "当前项目"
            archived -> "已归档"
            systemInbox -> "系统收件箱"
            else -> "活跃"
        },
        canSelect = actionsEnabled && !selected && !archived,
        canArchive = actionsEnabled && !archived && !systemInbox,
        canRestore = actionsEnabled && archived,
    )
}

package com.cameraconnector.app.ui

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
@Composable
internal fun DeleteSelectedProjectAssetsDialog(
    deleteSelectionCandidates: List<ProjectAsset>,
    deleteSelectionInFlight: Boolean,
    actionsEnabled: Boolean,
    sourceProjectId: String?,
    assetQuery: ProjectAssetQuery,
    coreGateway: CoreGateway,
    coroutineScope: CoroutineScope,
    onDeleteSelectionInFlightChange: (Boolean) -> Unit,
    onClearDeleteCandidates: () -> Unit,
    onClearSelection: () -> Unit,
    onShowProjectFeedback: (String) -> Unit,
) {
if (deleteSelectionCandidates.isNotEmpty()) {
        AlertDialog(
            onDismissRequest = {
                if (!deleteSelectionInFlight) {
                    onClearDeleteCandidates()
                }
            },
            title = { Text("删除选中照片？") },
            text = {
                Text(
                    "将彻底删除选中的 ${deleteSelectionCandidates.size} 张照片及其本地文件、评价、推荐和分组信息。此操作不可撤销。",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            },
            confirmButton = {
                TextButton(
                    enabled = actionsEnabled && !deleteSelectionInFlight,
                    onClick = {
                        val projectId = sourceProjectId ?: return@TextButton
                        val deleteTargets = deleteSelectionCandidates
                            .mapNotNull { it.assetGroupId() }
                            .distinct()
                        if (deleteTargets.isEmpty() || deleteSelectionInFlight) {
                            return@TextButton
                        }
                        onDeleteSelectionInFlightChange(true)
                        onShowProjectFeedback("正在删除")
                        coroutineScope.launch {
                            runCatching {
                                deleteTargets.forEach { groupId ->
                                    coreGateway.deleteProjectGroup(projectId, groupId)
                                }
                                withContext(Dispatchers.IO) {
                                    coreGateway.loadProjectAssets(assetQuery)
                                }
                                deleteTargets.size
                            }.onSuccess { deletedCount ->
                                onClearSelection()
                                onClearDeleteCandidates()
                                onShowProjectFeedback("已删除 $deletedCount 张照片")
                            }.onFailure {
                                onShowProjectFeedback("删除失败")
                            }
                            onDeleteSelectionInFlightChange(false)
                        }
                    },
                ) {
                    Text("删除", color = ElementDanger)
                }
            },
            dismissButton = {
                TextButton(
                    enabled = !deleteSelectionInFlight,
                    onClick = { onClearDeleteCandidates() },
                ) {
                    Text("取消")
                }
            },
        )
    }
}

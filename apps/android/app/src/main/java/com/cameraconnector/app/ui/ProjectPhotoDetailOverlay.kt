package com.cameraconnector.app.ui

import android.content.Context
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.ModelEvaluationPreviewInput
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import com.cameraconnector.app.media.loadPreviewSampleJson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
@Composable
internal fun ProjectPhotoDetailOverlay(
    photo: ProjectAsset,
    dashboardAssets: List<ProjectAsset>,
    visibleAssets: List<ProjectAsset>,
    activeProjectId: String?,
    assetQuery: ProjectAssetQuery,
    coreGateway: CoreGateway,
    context: Context,
    coroutineScope: CoroutineScope,
    actionsEnabled: Boolean,
    feedbackMessage: String?,
    onSelectPhoto: (ProjectAsset?) -> Unit,
    onShowProjectFeedback: (String) -> Unit,
    onSplitBurstMember: (String, String) -> Unit,
    onSetAssetGroupUserMarks: (String, String, Boolean?, Boolean?) -> Unit,
    modifier: Modifier = Modifier,
) {
    var deleteCandidate by remember { mutableStateOf<ProjectAsset?>(null) }
    var deleteInFlight by remember { mutableStateOf(false) }
        val detailBurstMembers = burstMemberFilmstrip(photo, dashboardAssets)
        val previousGroupAsset = adjacentProjectGridAsset(
            currentAsset = photo,
            visibleAssets = visibleAssets,
            direction = DetailNavigationDirection.Previous,
        )
        val nextGroupAsset = adjacentProjectGridAsset(
            currentAsset = photo,
            visibleAssets = visibleAssets,
            direction = DetailNavigationDirection.Next,
        )
        BackHandler {
            onSelectPhoto(null)
        }
        Box(modifier = modifier.fillMaxSize()) {
            PhotoDetailScreen(
                asset = photo,
                onBack = {
                    onSelectPhoto(null)
                },
                actionsEnabled = actionsEnabled && !deleteInFlight,
                onSplitBurstMember = { burstGroupId, memberGroupId ->
                    val nextMember = photoDetailSelectionAfterSplit(photo, detailBurstMembers)
                    onSplitBurstMember(burstGroupId, memberGroupId)
                    if (nextMember != null) {
                        onSelectPhoto(nextMember)
                    } else {
                        onSelectPhoto(null)
                    }
                },
                burstMembers = detailBurstMembers,
                onOpenBurstMember = { asset ->
                    onSelectPhoto(asset)
                },
                previousGroupAsset = previousGroupAsset,
                nextGroupAsset = nextGroupAsset,
                onNavigatePreviousGroup = {
                    previousGroupAsset?.let {
                        onSelectPhoto(it)
                    }
                },
                onNavigateNextGroup = {
                    nextGroupAsset?.let {
                        onSelectPhoto(it)
                    }
                },
                onToggleMarked = { groupId ->
                    val projectId = activeProjectId ?: return@PhotoDetailScreen
                    val nextMarked = !photo.userMarks.marked
                    onSelectPhoto(photo.copy(userMarks = photo.userMarks.copy(marked = nextMarked)))
                    onShowProjectFeedback(if (nextMarked) "已标记" else "已取消标记")
                    onSetAssetGroupUserMarks(projectId, groupId, null, nextMarked)
                },
                onToggleFavorite = { assetId ->
                    val projectId = activeProjectId ?: return@PhotoDetailScreen
                    val nextFavorite = !photo.userMarks.favorite
                    onSelectPhoto(photo.copy(userMarks = photo.userMarks.copy(favorite = nextFavorite)))
                    onShowProjectFeedback(if (nextFavorite) "已收藏" else "已取消收藏")
                    onSetAssetGroupUserMarks(projectId, assetId, nextFavorite, null)
                },
                onEvaluateModel = { asset ->
                    val projectId = activeProjectId ?: return@PhotoDetailScreen
                    if (asset.modelEvaluationInFlight()) {
                        onShowProjectFeedback("\u8bc4\u4ef7\u5df2\u63d0\u4ea4")
                        return@PhotoDetailScreen
                    }
                    onSelectPhoto(asset.copy(modelStatus = "pending"))
                    onShowProjectFeedback("\u5df2\u63d0\u4ea4\u8bc4\u4ef7")
                    coroutineScope.launch {
                        runCatching {
                            val input = withContext(Dispatchers.IO) {
                                ModelEvaluationPreviewInput(
                                    assetGroupId = asset.id,
                                    sampleJson = loadPreviewSampleJson(
                                        context,
                                        asset.previewLocation,
                                    ),
                                )
                            }
                            val count = coreGateway.evaluateAssetGroupsWithModelInputs(projectId, listOf(input))
                            val refreshedAsset = withContext(Dispatchers.IO) {
                                coreGateway.loadProjectAssets(assetQuery)
                                    .firstOrNull { it.assetSelectionId() == asset.assetSelectionId() }
                            }
                            count to refreshedAsset
                        }.onSuccess { count ->
                            count.second?.let { refreshed -> onSelectPhoto(refreshed) }
                                ?: run { onSelectPhoto(asset.copy(modelStatus = "ready")) }
                            onShowProjectFeedback("\u5df2\u5b8c\u6210\u8bc4\u4ef7 ${count.first}")
                        }.onFailure {
                            onSelectPhoto(asset.copy(modelStatus = "failed"))
                            onShowProjectFeedback("\u8bc4\u4ef7\u5931\u8d25")
                        }
                    }
                },
                onDeleteAsset = {
                    deleteCandidate = photo
                },
                modifier = Modifier.fillMaxSize(),
            )
            deleteCandidate?.let { candidate ->
                AlertDialog(
                    onDismissRequest = {
                        if (!deleteInFlight) {
                            deleteCandidate = null
                        }
                    },
                    title = { Text("\u5220\u9664\u7167\u7247\uff1f") },
                    text = {
                        Text(
                            "\u5c06\u5220\u9664\u8be5\u7167\u7247\u7684\u6240\u6709\u683c\u5f0f\u6587\u4ef6\u3001\u8bc4\u4ef7\u3001\u63a8\u8350\u548c\u5206\u7ec4\u4fe1\u606f\u3002",
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    },
                    confirmButton = {
                        TextButton(
                            enabled = actionsEnabled && !deleteInFlight,
                            onClick = {
                                val projectId = activeProjectId ?: return@TextButton
                                if (deleteInFlight) {
                                    return@TextButton
                                }
                                val deletedSelectionId = candidate.assetSelectionId()
                                deleteCandidate = null
                                deleteInFlight = true
                                onShowProjectFeedback("\u6b63\u5728\u5220\u9664")
                                coroutineScope.launch {
                                    runCatching {
                                        coreGateway.deleteProjectGroup(projectId, deletedSelectionId)
                                        withContext(Dispatchers.IO) {
                                            coreGateway.loadProjectAssets(assetQuery)
                                        }
                                        photoDetailSelectionAfterDelete(candidate, detailBurstMembers)
                                    }.onSuccess {
                                        onSelectPhoto(null)
                                        onShowProjectFeedback("\u5df2\u5220\u9664")
                                    }.onFailure {
                                        onShowProjectFeedback("\u5220\u9664\u5931\u8d25")
                                    }
                                    deleteInFlight = false
                                }
                            },
                        ) {
                            Text("\u5220\u9664", color = ElementDanger)
                        }
                    },
                    dismissButton = {
                        TextButton(
                            enabled = !deleteInFlight,
                            onClick = { deleteCandidate = null },
                        ) {
                            Text("\u53d6\u6d88")
                        }
                    },
                )
            }
            feedbackMessage?.let { message ->
                ProjectFeedbackToast(
                    message = message,
                    modifier = Modifier
                        .align(Alignment.TopCenter)
                        .padding(top = 18.dp),
                )
            }
        }
}

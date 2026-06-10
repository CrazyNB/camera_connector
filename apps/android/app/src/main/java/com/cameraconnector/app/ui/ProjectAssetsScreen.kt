package com.cameraconnector.app.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.graphics.Bitmap
import androidx.activity.compose.BackHandler
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.horizontalScroll
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
import androidx.compose.material3.AlertDialog
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.AutoAwesome
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material.icons.outlined.FilterList
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material.icons.outlined.KeyboardArrowUp
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
import androidx.compose.material3.TextButton
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
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
import com.cameraconnector.app.core.DEFAULT_LISTEN_HOST
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.ModelEvaluationPreviewInput
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import com.cameraconnector.app.core.ProjectAssetRole
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.PublishQueueState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.core.SelectionCandidateVisualInput
import com.cameraconnector.app.media.PREVIEW_DETAIL_FALLBACK_ASPECT_RATIO
import com.cameraconnector.app.media.PhotoMetadata
import com.cameraconnector.app.media.PreviewQuality
import com.cameraconnector.app.media.cacheThumbnailPreview
import com.cameraconnector.app.media.cachedThumbnailPreview
import com.cameraconnector.app.media.isDecodablePreviewLocation
import com.cameraconnector.app.media.loadPhotoMetadata
import com.cameraconnector.app.media.loadPreviewBitmap
import com.cameraconnector.app.media.loadPreviewSampleJson
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject

@Composable
internal fun ProjectAssetsScreen(
    coreGateway: CoreGateway,
    dashboard: DashboardState,
    projectState: ProjectState,
    notificationPermissionGranted: Boolean,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onOpenProjects: () -> Unit,
    onOpenProjectIntelligence: () -> Unit,
    onConfigureAccount: () -> Unit,
    onRequestNotificationPermission: () -> Unit,
    actionsEnabled: Boolean,
    onStartReceiver: (ReceiverSettings, String) -> Unit,
    onStopReceiver: () -> Unit,
    cameraConnectHost: String,
    onRetryFailedPublishes: () -> Unit,
    onMoveProjectGroup: (String, String, String) -> Unit,
    onSplitBurstMember: (String, String) -> Unit,
    onMergeBurstMember: (String, String) -> Unit,
    onSetAssetGroupUserMarks: (String, String, Boolean?, Boolean?) -> Unit,
    gridColumnCount: Int,
    modifier: Modifier = Modifier,
) {
    var selectedFilter by remember { mutableStateOf(AssetFormatFilter.All) }
    var selectedSort by remember { mutableStateOf(PhotoSortMode.LatestReceived) }
    var selectedPhotoCollection by rememberSaveable { mutableStateOf(ProjectPhotoCollection.All) }
    var selectedPhoto by remember { mutableStateOf<ProjectAsset?>(null) }
    var selectedBurstPreview by remember { mutableStateOf<ProjectPhotoGridItemUi?>(null) }
    var detailNavigationDirection by remember { mutableStateOf<DetailNavigationDirection?>(null) }
    var deleteCandidate by remember { mutableStateOf<ProjectAsset?>(null) }
    var deleteInFlight by remember { mutableStateOf(false) }
    var selectedAssetIds by rememberSaveable { mutableStateOf(emptyList<String>()) }
    var movePickerOpen by remember { mutableStateOf(false) }
    var filterExpanded by remember { mutableStateOf(false) }
    var projectFeedbackMessage by remember { mutableStateOf<String?>(null) }
    var projectFeedbackToken by remember { mutableStateOf(0) }
    val assetQuery = remember(
        selectedPhotoCollection,
        selectedFilter,
        selectedSort,
    ) {
        assetListQuery(
            selectedCollection = selectedPhotoCollection,
            selectedFilter = selectedFilter,
            selectedSort = selectedSort,
        )
    }
    val filteredAssets by produceState<List<ProjectAsset>>(
        initialValue = dashboard.assets,
        projectState.activeProjectId,
        assetQuery,
        selectedPhotoCollection,
        selectedFilter,
        selectedSort,
        dashboard.assets,
    ) {
        value = withContext(Dispatchers.IO) {
            coreGateway.loadProjectAssets(assetQuery)
        }
    }
    val gridItems = remember(filteredAssets) {
        projectPhotoGridItems(filteredAssets)
    }
    val selectionMode = isAssetSelectionMode(selectedAssetIds)
    val selectedGridItems = remember(gridItems, selectedAssetIds) {
        selectedPhotoGridItemsFromIds(gridItems, selectedAssetIds)
    }
    val selectedEvaluationTargets = remember(selectedGridItems) {
        projectPhotoEvaluationTargets(selectedGridItems)
    }
    val selectedAssets = remember(selectedGridItems) {
        selectedGridItems
            .filterNot { it.isBurstGroup }
            .map { it.coverAsset }
    }
    val selectedBurstMergeTarget = remember(selectedAssets) {
        manualBurstMergeTarget(selectedAssets)
    }
    val sourceProjectId = projectState.activeProjectId
    val moveTargets = remember(projectState.projects, sourceProjectId) {
        projectState.groupMoveTargets(sourceProjectId)
    }
    val context = LocalContext.current
    val coroutineScope = rememberCoroutineScope()
    var receiverPanelExpanded by remember { mutableStateOf(!dashboard.receiver.running) }
    val receiverConnectHost = normalizeCameraConnectHost(cameraConnectHost)
    LaunchedEffect(projectFeedbackToken) {
        if (projectFeedbackMessage != null) {
            delay(1_400)
            projectFeedbackMessage = null
        }
    }
    fun showProjectFeedback(message: String) {
        projectFeedbackMessage = message
        projectFeedbackToken += 1
    }

    LaunchedEffect(dashboard.receiver.running) {
        receiverPanelExpanded = !dashboard.receiver.running
    }

    LaunchedEffect(projectState.activeProjectId, assetQuery, selectedPhotoCollection) {
        selectedAssetIds = emptyList()
        selectedBurstPreview = null
        movePickerOpen = false
        detailNavigationDirection = null
    }
    LaunchedEffect(filteredAssets, selectedPhoto?.assetSelectionId()) {
        val currentPhoto = selectedPhoto
        val refreshedPhoto = refreshedSelectedPhoto(currentPhoto, filteredAssets)
        if (refreshedPhoto != currentPhoto) {
            detailNavigationDirection = null
            selectedPhoto = refreshedPhoto
        }
    }

    selectedPhoto?.let { photo ->
        val detailBurstMembers = burstMemberFilmstrip(photo, dashboard.assets)
        BackHandler {
            detailNavigationDirection = null
            selectedPhoto = null
        }
        Box(modifier = modifier.fillMaxSize()) {
            PhotoDetailScreen(
                asset = photo,
                onBack = {
                    detailNavigationDirection = null
                    selectedPhoto = null
                },
                actionsEnabled = actionsEnabled && !deleteInFlight,
                onSplitBurstMember = { burstGroupId, memberGroupId ->
                    val nextMember = photoDetailSelectionAfterSplit(photo, detailBurstMembers)
                    onSplitBurstMember(burstGroupId, memberGroupId)
                    if (nextMember != null) {
                        detailNavigationDirection = if (
                            detailBurstMemberIndex(nextMember, detailBurstMembers) <
                            detailBurstMemberIndex(photo, detailBurstMembers)
                        ) {
                            DetailNavigationDirection.Previous
                        } else {
                            DetailNavigationDirection.Next
                        }
                        selectedPhoto = nextMember
                    } else {
                        detailNavigationDirection = null
                        selectedPhoto = null
                    }
                },
                burstMembers = detailBurstMembers,
                onOpenBurstMember = { asset ->
                    detailNavigationDirection = if (
                        detailBurstMemberIndex(asset, detailBurstMembers) <
                        detailBurstMemberIndex(photo, detailBurstMembers)
                    ) {
                        DetailNavigationDirection.Previous
                    } else {
                        DetailNavigationDirection.Next
                    }
                    selectedPhoto = asset
                },
                onNavigatePreviousGroup = {
                    adjacentProjectGridAsset(
                        currentAsset = photo,
                        visibleAssets = filteredAssets,
                        direction = DetailNavigationDirection.Previous,
                    )?.let {
                        detailNavigationDirection = DetailNavigationDirection.Previous
                        selectedPhoto = it
                    }
                },
                onNavigateNextGroup = {
                    adjacentProjectGridAsset(
                        currentAsset = photo,
                        visibleAssets = filteredAssets,
                        direction = DetailNavigationDirection.Next,
                    )?.let {
                        detailNavigationDirection = DetailNavigationDirection.Next
                        selectedPhoto = it
                    }
                },
                onToggleMarked = { groupId ->
                    val projectId = projectState.activeProjectId ?: return@PhotoDetailScreen
                    val nextMarked = !photo.userMarks.marked
                    selectedPhoto = photo.copy(userMarks = photo.userMarks.copy(marked = nextMarked))
                    showProjectFeedback(if (nextMarked) "已标记" else "已取消标记")
                    onSetAssetGroupUserMarks(projectId, groupId, null, nextMarked)
                },
                onToggleFavorite = { assetId ->
                    val projectId = projectState.activeProjectId ?: return@PhotoDetailScreen
                    val nextFavorite = !photo.userMarks.favorite
                    selectedPhoto = photo.copy(userMarks = photo.userMarks.copy(favorite = nextFavorite))
                    showProjectFeedback(if (nextFavorite) "已收藏" else "已取消收藏")
                    onSetAssetGroupUserMarks(projectId, assetId, nextFavorite, null)
                },
                onEvaluateModel = { asset ->
                    val projectId = projectState.activeProjectId ?: return@PhotoDetailScreen
                    if (asset.modelEvaluationInFlight()) {
                        showProjectFeedback("\u8bc4\u4ef7\u5df2\u63d0\u4ea4")
                        return@PhotoDetailScreen
                    }
                    selectedPhoto = asset.copy(modelStatus = "pending")
                    showProjectFeedback("\u5df2\u63d0\u4ea4\u8bc4\u4ef7")
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
                            count.second?.let { refreshed -> selectedPhoto = refreshed }
                                ?: run { selectedPhoto = asset.copy(modelStatus = "ready") }
                            showProjectFeedback("\u5df2\u5b8c\u6210\u8bc4\u4ef7 ${count.first}")
                        }.onFailure {
                            selectedPhoto = asset.copy(modelStatus = "failed")
                            showProjectFeedback("\u8bc4\u4ef7\u5931\u8d25")
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
                                val projectId = projectState.activeProjectId ?: return@TextButton
                                if (deleteInFlight) {
                                    return@TextButton
                                }
                                val deletedSelectionId = candidate.assetSelectionId()
                                deleteCandidate = null
                                deleteInFlight = true
                                showProjectFeedback("\u6b63\u5728\u5220\u9664")
                                coroutineScope.launch {
                                    runCatching {
                                        coreGateway.deleteProjectGroup(projectId, deletedSelectionId)
                                        withContext(Dispatchers.IO) {
                                            coreGateway.loadProjectAssets(assetQuery)
                                        }
                                        photoDetailSelectionAfterDelete(candidate, detailBurstMembers)
                                    }.onSuccess {
                                        selectedPhoto = null
                                        detailNavigationDirection = null
                                        showProjectFeedback("\u5df2\u5220\u9664")
                                    }.onFailure {
                                        showProjectFeedback("\u5220\u9664\u5931\u8d25")
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
            projectFeedbackMessage?.let { message ->
                ProjectFeedbackToast(
                    message = message,
                    modifier = Modifier
                        .align(Alignment.TopCenter)
                        .padding(top = 18.dp),
                )
            }
        }
        return
    }
    BackHandler(enabled = selectionMode) {
        selectedAssetIds = emptyList()
        movePickerOpen = false
    }
    if (movePickerOpen) {
        MoveSelectedGroupsDialog(
            selectedCount = selectedAssets.size,
            targets = moveTargets,
            actionsEnabled = actionsEnabled && sourceProjectId != null,
            onDismiss = { movePickerOpen = false },
            onMoveToProject = { targetProjectId ->
                val currentProjectId = sourceProjectId
                if (currentProjectId != null) {
                    selectedAssets
                        .mapNotNull { it.groupMoveId() }
                        .distinct()
                        .forEach { groupId ->
                            onMoveProjectGroup(currentProjectId, groupId, targetProjectId)
                        }
                    selectedAssetIds = emptyList()
                    movePickerOpen = false
                }
            },
        )
    }

    Box(modifier = modifier.fillMaxSize()) {
        Column(
            modifier = Modifier
            .fillMaxSize()
            .padding(16.dp)
            .then(if (receiverPanelExpanded) Modifier.blur(6.dp) else Modifier)
            .animateContentSize(),
        ) {
        actionError?.let { message ->
            ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError)
            Spacer(Modifier.height(10.dp))
        }

        if (!receiverPanelExpanded) {
            ProjectReceiverStatusStrip(
                dashboard = dashboard,
                projectState = projectState,
                onExpand = { receiverPanelExpanded = true },
                onOpenProjectIntelligence = onOpenProjectIntelligence,
                connectHost = receiverConnectHost,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(10.dp))
        }

        if (projectPhotoContentVisible(dashboard.receiver.running)) {
            PhotoListControlRow(
                selectedCollection = selectedPhotoCollection,
                onCollectionChange = { collection ->
                    selectedPhotoCollection = collection
                },
                selectedFilter = selectedFilter,
                selectedSort = selectedSort,
                expanded = filterExpanded,
                onToggle = { filterExpanded = !filterExpanded },
            )
            Spacer(Modifier.height(8.dp))
            if (filterExpanded) {
                Spacer(Modifier.height(8.dp))
                AssetFormatFilterBar(
                    selectedFilter = selectedFilter,
                    onFilterChange = { selectedFilter = it },
                    assets = dashboard.assets,
                )
                Spacer(Modifier.height(8.dp))
                PhotoSortBar(
                    selectedSort = selectedSort,
                    onSortChange = { selectedSort = it },
                )
            }
            Spacer(Modifier.height(10.dp))
            if (filteredAssets.isEmpty()) {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        when {
                            selectedPhotoCollection == ProjectPhotoCollection.Favorites -> "\u8fd8\u6ca1\u6709\u6536\u85cf\u7167\u7247"
                            selectedPhotoCollection == ProjectPhotoCollection.Marked -> "\u8fd8\u6ca1\u6709\u6807\u8bb0\u7167\u7247"
                            dashboard.assets.isEmpty() -> "\u8fd8\u6ca1\u6709\u5bfc\u5165\u6587\u4ef6"
                            else -> "当前筛选下没有文件"
                        },
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                selectedBurstPreview?.let { previewItem ->
                    BurstGroupPreviewDialog(
                        item = previewItem,
                        allProjectAssets = dashboard.assets,
                        onDismiss = { selectedBurstPreview = null },
                        onOpenAsset = { memberAsset ->
                            selectedBurstPreview = null
                            detailNavigationDirection = null
                            selectedPhoto = memberAsset
                        },
                    )
                }
                Box(modifier = Modifier.fillMaxSize()) {
                    LazyVerticalGrid(
                        columns = GridCells.Fixed(gridColumnCount),
                        modifier = Modifier.fillMaxSize(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(10.dp),
                        contentPadding = PaddingValues(bottom = if (selectionMode) 104.dp else 8.dp),
                    ) {
                        items(
                            count = gridItems.size,
                            key = { index -> gridItems[index].key },
                        ) { index ->
                            val item = gridItems[index]
                            val asset = item.coverAsset
                            val selected = item.key in selectedAssetIds
                            CompactPhotoTile(
                                asset = asset,
                                selected = selected,
                                selectionMode = selectionMode,
                                onClick = {
                                    if (selectionMode) {
                                        selectedAssetIds = togglePhotoGridItemSelection(selectedAssetIds, item)
                                    } else if (item.isBurstGroup) {
                                        selectedBurstPreview = item
                                    } else {
                                        detailNavigationDirection = null
                                        selectedPhoto = asset
                                    }
                                },
                                onLongClick = {
                                    selectedAssetIds = togglePhotoGridItemSelection(selectedAssetIds, item)
                                },
                            )
                        }
                    }
                    if (selectionMode) {
                        SelectedAssetsActionBar(
                            selectedCount = selectedGridItems.size,
                            totalCount = gridItems.size,
                            canOpen = selectedGridItems.size == 1,
                            canEvaluate = actionsEnabled &&
                                sourceProjectId != null &&
                                (
                                    selectedEvaluationTargets.assetGroups.isNotEmpty() ||
                                        selectedEvaluationTargets.burstGroups.isNotEmpty()
                                ),
                            canMove = actionsEnabled &&
                                sourceProjectId != null &&
                                selectedAssets.any { it.groupMoveId() != null } &&
                                moveTargets.isNotEmpty(),
                            canMerge = actionsEnabled && selectedBurstMergeTarget != null,
                            onOpen = {
                                selectedGridItems.firstOrNull()?.let { item ->
                                    selectedAssetIds = emptyList()
                                    if (item.isBurstGroup) {
                                        selectedBurstPreview = item
                                    } else {
                                        detailNavigationDirection = null
                                        selectedPhoto = item.coverAsset
                                    }
                                }
                            },
                            onEvaluate = {
                                val currentProjectId = sourceProjectId
                                val targets = selectedEvaluationTargets
                                if (
                                    currentProjectId != null &&
                                    (
                                        targets.assetGroups.isNotEmpty() ||
                                            targets.burstGroups.isNotEmpty()
                                    )
                                ) {
                                    selectedAssetIds = emptyList()
                                    coroutineScope.launch {
                                        runCatching {
                                            var evaluatedCount = 0
                                            var recommendedBurstCount = 0
                                            val assetsForEvaluation = targets.assetGroups.distinctBy { it.id }
                                            if (assetsForEvaluation.isNotEmpty()) {
                                                val inputs = withContext(Dispatchers.IO) {
                                                    assetsForEvaluation.map { asset ->
                                                        ModelEvaluationPreviewInput(
                                                            assetGroupId = asset.id,
                                                            sampleJson = loadPreviewSampleJson(
                                                                context,
                                                                asset.previewLocation,
                                                            ),
                                                        )
                                                    }
                                                }
                                                evaluatedCount += coreGateway.evaluateAssetGroupsWithModelInputs(
                                                    currentProjectId,
                                                    inputs,
                                                )
                                            }
                                            targets.burstGroups.forEach { burstTarget ->
                                                val candidateVisuals = burstRecommendationCandidateVisuals(
                                                    context = context,
                                                    members = burstTarget.members,
                                                )
                                                if (candidateVisuals.size < 2) {
                                                    error("burst candidate visuals are incomplete")
                                                }
                                                if (
                                                    coreGateway.recommendBurstGroupWithCandidateVisuals(
                                                        burstGroupId = burstTarget.burstGroupId,
                                                        candidateVisuals = candidateVisuals,
                                                    )
                                                ) {
                                                    recommendedBurstCount += 1
                                                }
                                            }
                                            evaluatedCount to recommendedBurstCount
                                        }.onSuccess { (evaluatedCount, recommendedBurstCount) ->
                                            showProjectFeedback(
                                                projectEvaluationFeedback(
                                                    evaluatedCount = evaluatedCount,
                                                    recommendedBurstCount = recommendedBurstCount,
                                                ),
                                            )
                                        }.onFailure {
                                            showProjectFeedback("\u8bc4\u4ef7\u5931\u8d25")
                                        }
                                    }
                                }
                            },
                            onMove = { movePickerOpen = true },
                            onMerge = {
                                selectedBurstMergeTarget?.let { target ->
                                    onMergeBurstMember(target.targetBurstGroupId, target.memberGroupId)
                                    selectedAssetIds = emptyList()
                                }
                            },
                            onSelectAll = {
                                selectedAssetIds = gridItems.map { it.key }
                            },
                            onCancel = { selectedAssetIds = emptyList() },
                            modifier = Modifier
                                .align(Alignment.BottomCenter)
                                .fillMaxWidth(),
                        )
                    }
                }
            }
        }
        }
        if (receiverPanelExpanded) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(ElementBackground.copy(alpha = 0.68f)),
            )
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(16.dp),
            ) {
                ProjectReceiverLaunchPanel(
                    dashboard = dashboard,
                    projectState = projectState,
                    notificationPermissionGranted = notificationPermissionGranted,
                    actionsEnabled = actionsEnabled,
                    onOpenProjects = onOpenProjects,
                    onOpenProjectIntelligence = onOpenProjectIntelligence,
                    onConfigureAccount = onConfigureAccount,
                    onRequestNotificationPermission = onRequestNotificationPermission,
                    onStartReceiver = onStartReceiver,
                    onStopReceiver = onStopReceiver,
                    onRetryFailedPublishes = onRetryFailedPublishes,
                    onCollapse = { receiverPanelExpanded = false },
                    connectHost = receiverConnectHost,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
        actionInFlight?.let { action ->
            ActionLoadingOverlay(action = action)
        }
        projectFeedbackMessage?.let { message ->
            ProjectFeedbackToast(
                message = message,
                modifier = Modifier
                    .align(Alignment.TopCenter)
                    .padding(top = 18.dp),
            )
        }
    }
}


@Composable
private fun ProjectFeedbackToast(
    message: String,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier,
        color = ElementSurface.copy(alpha = 0.96f),
        contentColor = ElementText,
        shape = RoundedCornerShape(999.dp),
        border = BorderStroke(1.dp, ElementCardBorder),
    ) {
        Text(
            text = message,
            modifier = Modifier.padding(horizontal = 18.dp, vertical = 10.dp),
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.SemiBold,
        )
    }
}

private fun ProjectAsset.modelEvaluationInFlight(): Boolean =
    modelStatus?.trim()?.lowercase() in setOf(
        "pending",
        "queued",
        "running",
        "processing",
        "analyzing",
    )


@Composable
internal fun BurstGroupPreviewDialog(
    item: ProjectPhotoGridItemUi,
    allProjectAssets: List<ProjectAsset>,
    onDismiss: () -> Unit,
    onOpenAsset: (ProjectAsset) -> Unit,
) {
    val previewItems = remember(item, allProjectAssets) {
        val filmstrip = burstMemberFilmstrip(item.coverAsset, allProjectAssets)
        if (filmstrip.isNotEmpty()) {
            filmstrip
        } else {
            item.members.map { asset ->
                BurstMemberFilmstripItemUi(
                    asset = asset,
                    badgeText = if (asset.assetSelectionId() == item.coverAsset.assetSelectionId()) "优选" else "备选",
                    scoreText = null,
                )
            }
        }
    }
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(18.dp),
            contentAlignment = Alignment.Center,
        ) {
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
                            Text(
                                "\u8fde\u62cd\u7ec4",
                                style = MaterialTheme.typography.titleLarge,
                                fontWeight = FontWeight.Bold,
                            )
                            Spacer(Modifier.height(4.dp))
                            Text(
                                "${previewItems.size} 张 · 优选 ${item.coverAsset.filename()}",
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                style = MaterialTheme.typography.bodySmall,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                        TextButton(onClick = onDismiss) {
                            Text("关闭")
                        }
                    }
                    LazyVerticalGrid(
                        columns = GridCells.Fixed(3),
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(360.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        items(
                            count = previewItems.size,
                            key = { index -> previewItems[index].asset.assetSelectionId() },
                        ) { index ->
                            BurstGroupPreviewTile(
                                item = previewItems[index],
                                tileUi = burstPreviewTileUi(
                                    item = previewItems[index],
                                    index = index,
                                    total = previewItems.size,
                                ),
                                onClick = { onOpenAsset(previewItems[index].asset) },
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun BurstGroupPreviewTile(
    item: BurstMemberFilmstripItemUi,
    tileUi: BurstPreviewTileUi,
    onClick: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(ElementSurface)
            .border(1.dp, ElementCardBorder, RoundedCornerShape(14.dp))
            .clickable(onClick = onClick)
            .padding(7.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(1f),
        ) {
            PhotoPreview(
                asset = item.asset,
                compactFallback = true,
                backgroundColor = item.asset.previewAccentColor().copy(alpha = 0.16f),
                modifier = Modifier.matchParentSize(),
            )
            PhotoEdgeBadge(
                text = tileUi.positionText,
                color = ElementPurple,
                modifier = Modifier
                    .align(Alignment.TopStart)
                    .padding(5.dp),
            )
            tileUi.scoreText?.let { scoreText ->
                PhotoEdgeBadge(
                    text = scoreText,
                    color = ElementWarning,
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .padding(5.dp),
                )
            }
            if (tileUi.modelSelected) {
                PhotoEdgeBadge(
                    text = "\u4f18\u9009",
                    color = ElementSuccess,
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(5.dp),
                )
            }
        }
        Text(
            item.asset.filename(),
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        if (tileUi.auxiliaryBadges.isNotEmpty()) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                tileUi.auxiliaryBadges.forEach { badge ->
                    PhotoInlineBadge(
                        text = badge,
                        color = auxiliaryBadgeColor(badge),
                    )
                }
            }
        }
    }
}

@Composable
internal fun ProjectReceiverStatusStrip(
    dashboard: DashboardState,
    projectState: ProjectState,
    onExpand: () -> Unit,
    onOpenProjectIntelligence: () -> Unit,
    connectHost: String?,
    modifier: Modifier = Modifier,
) {
    val project = projectState.activeProjectSummary()
    Surface(
        modifier = modifier,
        color = ElementControlSurface.copy(alpha = 0.86f),
        contentColor = MaterialTheme.colorScheme.onSurface,
        shape = RoundedCornerShape(14.dp),
        border = BorderStroke(1.dp, ElementBorder),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = 12.dp, top = 9.dp, end = 4.dp, bottom = 9.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Row(
                modifier = Modifier.weight(1f),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(
                    modifier = Modifier
                        .size(10.dp)
                        .background(
                            if (dashboard.receiver.running) ElementSuccess else ElementInfo,
                            CircleShape,
                        ),
                )
                Spacer(Modifier.width(9.dp))
                Column(Modifier.weight(1f)) {
                    Text(
                        project?.name ?: "当前项目",
                        style = MaterialTheme.typography.bodyMedium,
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        receiverEndpointLabel(dashboard.receiver, connectHost),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
            Spacer(Modifier.width(4.dp))
            IconButton(
                onClick = onOpenProjectIntelligence,
                enabled = project != null,
                modifier = Modifier.size(34.dp),
            ) {
                Icon(
                    imageVector = Icons.Outlined.AutoAwesome,
                    contentDescription = "项目智能",
                    tint = if (project != null) ElementBlue else MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(18.dp),
                )
            }
            IconButton(
                onClick = onExpand,
                modifier = Modifier.size(34.dp),
            ) {
                Icon(
                    imageVector = Icons.Outlined.KeyboardArrowDown,
                    contentDescription = "展开接收抽屉",
                    tint = ElementBlue,
                )
            }
        }
    }
}

@Composable
internal fun ProjectLaunchHeader(
    projectState: ProjectState,
    actionsEnabled: Boolean,
    onOpenProjects: () -> Unit,
    onOpenProjectIntelligence: () -> Unit,
    onCollapse: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val project = projectState.activeProjectSummary()
    Row(
        modifier = modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(
            modifier = Modifier.weight(1f),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconButton(
                onClick = onOpenProjects,
                enabled = actionsEnabled,
                modifier = Modifier.size(34.dp),
            ) {
                Icon(
                    imageVector = Icons.AutoMirrored.Outlined.ArrowBack,
                    contentDescription = "返回项目管理",
                    tint = ElementBlue,
                )
            }
            Spacer(Modifier.width(6.dp))
            Text(
                project?.name ?: "项目",
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        IconButton(
            onClick = onOpenProjectIntelligence,
            enabled = project != null,
            modifier = Modifier.size(34.dp),
        ) {
            Icon(
                imageVector = Icons.Outlined.AutoAwesome,
                contentDescription = "项目智能",
                tint = if (project != null) ElementBlue else MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        IconButton(
            onClick = onCollapse,
            modifier = Modifier.size(34.dp),
        ) {
            Icon(
                imageVector = Icons.Outlined.KeyboardArrowUp,
                contentDescription = "\u6536\u8d77\u542f\u52a8\u9875",
                tint = ElementBlue,
            )
        }
    }
}

@Composable
internal fun ProjectReceiverLaunchPanel(
    dashboard: DashboardState,
    projectState: ProjectState,
    notificationPermissionGranted: Boolean,
    actionsEnabled: Boolean,
    onOpenProjects: () -> Unit,
    onOpenProjectIntelligence: () -> Unit,
    onConfigureAccount: () -> Unit,
    onRequestNotificationPermission: () -> Unit,
    onStartReceiver: (ReceiverSettings, String) -> Unit,
    onStopReceiver: () -> Unit,
    onRetryFailedPublishes: () -> Unit,
    onCollapse: () -> Unit,
    connectHost: String?,
    modifier: Modifier = Modifier,
) {
    var protocol by remember(dashboard.receiver.protocol) {
        mutableStateOf(dashboard.receiver.protocol.ifBlank { "FTP" })
    }
    var portInput by remember(dashboard.receiver.port) {
        mutableStateOf(dashboard.receiver.port.takeIf { it in 1..65_535 }?.toString() ?: "2121")
    }
    var connectHostInput by rememberSaveable(connectHost) {
        mutableStateOf(normalizeCameraConnectHost(connectHost))
    }
    val port = portInput.toIntOrNull()
    val cleanConnectHost = normalizeCameraConnectHost(connectHostInput)
    val receiverSettingsValid = port in 1..65_535
    val receiverSettings = ReceiverSettings(
        protocol = protocol,
        host = DEFAULT_LISTEN_HOST,
        ftpPort = port ?: dashboard.receiver.port,
        sftpPort = port ?: dashboard.receiver.port,
        outputLabel = dashboard.receiver.outputLabel,
    )
    val onlineConnections = dashboard.accounts.sumOf { it.activeConnections }
    val receiverBusy = receiverPhaseBusy(dashboard.receiver.phase)
    val startBlockReason = receiverStartBlockReason(
        running = dashboard.receiver.running,
        busy = receiverBusy,
        actionsEnabled = actionsEnabled,
        notificationPermissionGranted = notificationPermissionGranted,
        accountCount = dashboard.accounts.size,
    )
    var visibleStartBlockReason by remember { mutableStateOf<ReceiverStartBlockReason?>(null) }

    visibleStartBlockReason?.let { reason ->
        ReceiverStartBlockedDialog(
            reason = reason,
            onDismiss = { visibleStartBlockReason = null },
            onConfigureAccount = {
                visibleStartBlockReason = null
                onConfigureAccount()
            },
            onRequestNotificationPermission = {
                visibleStartBlockReason = null
                onRequestNotificationPermission()
            },
        )
    }

    ElementCard(modifier = modifier.fillMaxWidth()) {
        Column(Modifier.padding(14.dp)) {
            ProjectLaunchHeader(
                projectState = projectState,
                actionsEnabled = actionsEnabled,
                onOpenProjects = onOpenProjects,
                onOpenProjectIntelligence = onOpenProjectIntelligence,
                onCollapse = onCollapse,
            )
            Spacer(Modifier.height(10.dp))
            val collapseArrowOffset = rememberInfiniteTransition(label = "collapseArrow")
                .animateFloat(
                    initialValue = 0f,
                    targetValue = -7f,
                    animationSpec = infiniteRepeatable(
                        animation = tween(durationMillis = 680),
                        repeatMode = RepeatMode.Reverse,
                    ),
                    label = "collapseArrowOffset",
                )
            ReceiverHeroControl(
                running = dashboard.receiver.running,
                phase = dashboard.receiver.phase,
                onlineConnections = onlineConnections,
                accountCount = dashboard.accounts.size,
                publishQueue = dashboard.publishQueue,
                message = dashboard.receiver.message,
                enabled = actionsEnabled &&
                    !receiverBusy &&
                    (dashboard.receiver.running || receiverSettingsValid),
                retryEnabled = actionsEnabled,
                onToggleReceiver = {
                    if (dashboard.receiver.running) {
                        onStopReceiver()
                    } else if (receiverBusy) {
                        visibleStartBlockReason = ReceiverStartBlockReason.Busy
                    } else if (startBlockReason == null) {
                        onStartReceiver(receiverSettings, cleanConnectHost)
                    } else {
                        visibleStartBlockReason = startBlockReason
                    }
                },
                onRetryFailedPublishes = onRetryFailedPublishes,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(12.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                ProtocolSegment(
                    label = "FTP",
                    selected = protocol == "FTP",
                    enabled = actionsEnabled && !dashboard.receiver.running && !receiverBusy,
                    onClick = { protocol = "FTP" },
                    modifier = Modifier.weight(1f),
                )
                ProtocolSegment(
                    label = "SFTP",
                    selected = protocol == "SFTP",
                    enabled = actionsEnabled && !dashboard.receiver.running && !receiverBusy,
                    onClick = { protocol = "SFTP" },
                    modifier = Modifier.weight(1f),
                )
            }
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = connectHostInput,
                onValueChange = { connectHostInput = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("相机连接 IP") },
                singleLine = true,
                enabled = actionsEnabled && !receiverBusy,
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = portInput,
                onValueChange = { portInput = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("端口") },
                singleLine = true,
                enabled = actionsEnabled && !dashboard.receiver.running && !receiverBusy,
            )
            Spacer(Modifier.height(8.dp))
            Text(
                "输出目录：${dashboard.receiver.outputLabel}",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            if (dashboard.receiver.running || receiverBusy) {
                Spacer(Modifier.height(8.dp))
                Text(
                    "\u4fee\u6539\u914d\u7f6e\u524d\u9700\u8981\u5148\u505c\u6b62\u63a5\u6536\u3002\u6536\u8d77\u540e\u7167\u7247\u5217\u8868\u4f1a\u5360\u6ee1\u4e3b\u8981\u7a7a\u95f4\u3002",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                )
            }
            Spacer(Modifier.height(10.dp))
            OutlinedButton(
                onClick = onCollapse,
                modifier = Modifier.fillMaxWidth(),
                shape = elementShape,
                border = BorderStroke(1.dp, ElementBorder),
            ) {
                Icon(
                    imageVector = Icons.Outlined.KeyboardArrowUp,
                    contentDescription = null,
                    modifier = Modifier
                        .size(22.dp)
                        .graphicsLayer { translationY = collapseArrowOffset.value },
                )
            }
        }
    }
}

internal fun receiverEndpointLabel(receiver: ReceiverState, connectHost: String? = null): String =
    "${receiver.protocol} ${normalizeCameraConnectHost(connectHost)}:${receiver.port}"

private suspend fun burstRecommendationCandidateVisuals(
    context: Context,
    members: List<ProjectAsset>,
): List<SelectionCandidateVisualInput> =
    withContext(Dispatchers.IO) {
        members
            .distinctBy { it.assetSelectionId() }
            .mapNotNull { asset ->
                val imageDataUrl = runCatching {
                    JSONObject(loadPreviewSampleJson(context, asset.previewLocation))
                        .optString("image_data_url")
                        .takeIf { it.isNotBlank() && it != "null" }
                }.getOrNull()
                imageDataUrl?.let {
                    SelectionCandidateVisualInput(
                        assetGroupId = asset.id,
                        imageDataUrl = it,
                    )
                }
            }
    }

internal fun projectEvaluationFeedback(
    evaluatedCount: Int,
    recommendedBurstCount: Int,
): String =
    when {
        evaluatedCount > 0 && recommendedBurstCount > 0 ->
            "\u5df2\u5b8c\u6210\u5355\u5f20\u8bc4\u4ef7 $evaluatedCount \u00b7 \u8fde\u62cd\u8bc4\u4ef7 $recommendedBurstCount"
        recommendedBurstCount > 0 ->
            "\u5df2\u5b8c\u6210\u8fde\u62cd\u8bc4\u4ef7 $recommendedBurstCount"
        evaluatedCount > 0 ->
            "\u5df2\u5b8c\u6210\u5355\u5f20\u8bc4\u4ef7 $evaluatedCount"
        else -> "\u6ca1\u6709\u53ef\u8bc4\u4ef7\u9879"
    }

@Composable
internal fun ReceiverStartBlockedDialog(
    reason: ReceiverStartBlockReason,
    onDismiss: () -> Unit,
    onConfigureAccount: () -> Unit,
    onRequestNotificationPermission: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(
                when (reason) {
                    ReceiverStartBlockReason.MissingAccount -> "需要先配置账号"
                    ReceiverStartBlockReason.MissingNotificationPermission -> "需要通知权限"
                    ReceiverStartBlockReason.Busy -> "正在处理"
                },
            )
        },
        text = {
            Text(
                when (reason) {
                    ReceiverStartBlockReason.MissingAccount ->
                        "\u63a5\u6536\u670d\u52a1\u4f7f\u7528\u8d26\u53f7\u8ba4\u8bc1\u3002\u8bf7\u5148\u521b\u5efa\u76f8\u673a\u8d26\u53f7\uff0c\u518d\u542f\u52a8\u63a5\u6536\u3002"
                    ReceiverStartBlockReason.MissingNotificationPermission ->
                        "\u63a5\u6536\u670d\u52a1\u4f1a\u4ee5\u524d\u53f0\u670d\u52a1\u8fd0\u884c\uff0c\u9700\u8981\u5148\u5141\u8bb8\u901a\u77e5\u6743\u9650\u3002"
                    ReceiverStartBlockReason.Busy ->
                        "\u5f53\u524d\u8fd8\u6709\u64cd\u4f5c\u672a\u5b8c\u6210\uff0c\u8bf7\u7a0d\u540e\u518d\u542f\u52a8\u63a5\u6536\u3002"
                },
            )
        },
        confirmButton = {
            TextButton(
                onClick = when (reason) {
                    ReceiverStartBlockReason.MissingAccount -> onConfigureAccount
                    ReceiverStartBlockReason.MissingNotificationPermission -> onRequestNotificationPermission
                    ReceiverStartBlockReason.Busy -> onDismiss
                },
            ) {
                Text(
                    when (reason) {
                        ReceiverStartBlockReason.MissingAccount -> "\u53bb\u914d\u7f6e\u8d26\u53f7"
                        ReceiverStartBlockReason.MissingNotificationPermission -> "\u5f00\u542f\u6743\u9650"
                        ReceiverStartBlockReason.Busy -> "\u77e5\u9053\u4e86"
                    },
                )
            }
        },
        dismissButton = {
            if (reason != ReceiverStartBlockReason.Busy) {
                TextButton(onClick = onDismiss) {
                    Text("取消")
                }
            }
        },
    )
}

@Composable
internal fun SelectedAssetsActionBar(
    selectedCount: Int,
    totalCount: Int,
    canOpen: Boolean,
    canEvaluate: Boolean,
    canMove: Boolean,
    canMerge: Boolean,
    onOpen: () -> Unit,
    onEvaluate: () -> Unit,
    onMove: () -> Unit,
    onMerge: () -> Unit,
    onSelectAll: () -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier.animateContentSize(),
        color = ElementPanel.copy(alpha = 0.96f),
        contentColor = MaterialTheme.colorScheme.onSurface,
        shape = RoundedCornerShape(18.dp),
        border = BorderStroke(1.dp, ElementBlue.copy(alpha = 0.35f)),
    ) {
        Column(
            modifier = Modifier.padding(horizontal = 14.dp, vertical = 10.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Text(
                    "\u5df2\u9009 $selectedCount \u9879",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    modifier = Modifier.weight(1f),
                )
                TextButton(
                    onClick = if (selectedCount < totalCount) onSelectAll else onCancel,
                    enabled = totalCount > 0,
                ) {
                    Text(if (selectedCount < totalCount) "\u5168\u9009" else "\u6e05\u7a7a")
                }
                TextButton(onClick = onCancel) {
                    Text("\u5b8c\u6210")
                }
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                SelectionActionButton(
                    icon = Icons.Outlined.PhotoLibrary,
                    text = "\u6253\u5f00",
                    enabled = canOpen,
                    primary = true,
                    modifier = Modifier.weight(1f),
                    onClick = onOpen,
                )
                SelectionActionButton(
                    icon = Icons.Outlined.AutoAwesome,
                    text = "\u8bc4\u4ef7",
                    enabled = canEvaluate,
                    modifier = Modifier.weight(1f),
                    accent = ElementBlue,
                    onClick = onEvaluate,
                )
                SelectionActionButton(
                    icon = Icons.Outlined.SyncAlt,
                    text = "\u79fb\u52a8",
                    enabled = canMove,
                    modifier = Modifier.weight(1f),
                    onClick = onMove,
                )
                SelectionActionButton(
                    icon = Icons.Outlined.PhotoLibrary,
                    text = "\u5408\u5e76",
                    enabled = canMerge,
                    modifier = Modifier.weight(1f),
                    accent = ElementPurple,
                    onClick = onMerge,
                )
            }
        }
    }
}

@Composable
private fun SelectionActionButton(
    icon: ImageVector,
    text: String,
    enabled: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    primary: Boolean = false,
    accent: Color = MaterialTheme.colorScheme.onSurface,
) {
    val buttonModifier = modifier.height(44.dp)
    if (primary) {
        Button(
            onClick = onClick,
            enabled = enabled,
            modifier = buttonModifier,
            shape = RoundedCornerShape(10.dp),
            colors = ButtonDefaults.buttonColors(
                containerColor = ElementBlue,
                contentColor = ElementOnAccent,
            ),
            contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
        ) {
            SelectionActionButtonContent(icon = icon, text = text)
        }
    } else {
        OutlinedButton(
            onClick = onClick,
            enabled = enabled,
            modifier = buttonModifier,
            shape = RoundedCornerShape(10.dp),
            border = BorderStroke(1.dp, if (enabled) accent.copy(alpha = 0.45f) else ElementBorder),
            colors = ButtonDefaults.outlinedButtonColors(
                containerColor = ElementControlSurface,
                contentColor = accent,
            ),
            contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
        ) {
            SelectionActionButtonContent(icon = icon, text = text)
        }
    }
}

@Composable
private fun SelectionActionButtonContent(
    icon: ImageVector,
    text: String,
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            modifier = Modifier.size(17.dp),
        )
        Text(
            text,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            fontSize = 11.sp,
            lineHeight = 12.sp,
            fontWeight = FontWeight.SemiBold,
        )
    }
}

@Composable
internal fun MoveSelectedGroupsDialog(
    selectedCount: Int,
    targets: List<ProjectSummary>,
    actionsEnabled: Boolean,
    onDismiss: () -> Unit,
    onMoveToProject: (String) -> Unit,
) {
    Dialog(onDismissRequest = onDismiss) {
        ElementCard(modifier = Modifier.fillMaxWidth()) {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Text(
                    "\u79fb\u52a8 $selectedCount \u4e2a\u5206\u7ec4",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold,
                )
                if (targets.isEmpty()) {
                    Text(
                        "\u5f53\u524d\u6ca1\u6709\u53ef\u79fb\u52a8\u5230\u7684\u9879\u76ee",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                } else {
                    targets.forEach { target ->
                        Button(
                            onClick = { onMoveToProject(target.id) },
                            enabled = actionsEnabled,
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(10.dp),
                            colors = ButtonDefaults.buttonColors(
                                containerColor = ElementBlue,
                                contentColor = ElementOnAccent,
                            ),
                        ) {
                            Text(
                                target.name,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                }
                OutlinedButton(
                    onClick = onDismiss,
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(10.dp),
                    border = BorderStroke(1.dp, ElementBorder),
                    colors = ButtonDefaults.outlinedButtonColors(
                        containerColor = ElementControlSurface,
                        contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    ),
                ) {
                    Text("取消")
                }
            }
        }
    }
}

@Composable
internal fun PhotoListControlRow(
    selectedCollection: ProjectPhotoCollection,
    onCollectionChange: (ProjectPhotoCollection) -> Unit,
    selectedFilter: AssetFormatFilter,
    selectedSort: PhotoSortMode,
    expanded: Boolean,
    onToggle: () -> Unit,
) {
    val activeSummary = photoListFilterSummary(selectedFilter, selectedSort)
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            LazyRow(
                modifier = Modifier.weight(1f),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(ProjectPhotoCollection.entries) { collection ->
                    FilterChipButton(
                        label = collection.label,
                        selected = selectedCollection == collection,
                        onClick = { onCollectionChange(collection) },
                    )
                }
            }
            Spacer(Modifier.width(8.dp))
            Surface(
                modifier = Modifier
                    .size(42.dp)
                    .clickable(onClick = onToggle),
                color = if (expanded) ElementBlue else ElementControlSurface,
                contentColor = if (expanded) ElementOnAccent else ElementBlue,
                shape = RoundedCornerShape(14.dp),
                border = BorderStroke(1.dp, if (expanded) ElementBlue else ElementBorder),
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        imageVector = Icons.Outlined.FilterList,
                        contentDescription = if (expanded) "收起筛选" else "展开筛选",
                        modifier = Modifier.size(20.dp),
                    )
                }
            }
        }
        activeSummary?.let { summary ->
            Text(
                summary,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
internal fun PhotoSortBar(
    selectedSort: PhotoSortMode,
    onSortChange: (PhotoSortMode) -> Unit,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(PhotoSortMode.entries) { sortMode ->
            FilterChipButton(
                label = sortMode.label,
                selected = selectedSort == sortMode,
                onClick = { onSortChange(sortMode) },
            )
        }
    }
}

@Composable
internal fun AssetFormatFilterBar(
    selectedFilter: AssetFormatFilter,
    onFilterChange: (AssetFormatFilter) -> Unit,
    assets: List<ProjectAsset>,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(AssetFormatFilter.entries) { filter ->
            val count = assets.count { filter.matches(it) }
            FilterChipButton(
                label = "${filter.label} $count",
                selected = selectedFilter == filter,
                onClick = { onFilterChange(filter) },
            )
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
internal fun CompactPhotoTile(
    asset: ProjectAsset,
    selected: Boolean,
    selectionMode: Boolean,
    onClick: () -> Unit,
    onLongClick: () -> Unit,
) {
    val burstBadge = asset.burstCountBadgeText()
    val primaryBadge = asset.tilePrimaryBadgeText()
    val recommendationBadge = asset.recommendationBadgeText()
    val auxiliaryBadges = asset.tileAuxiliaryBadges()
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .background(ElementSurface)
            .border(
                width = 1.dp,
                color = if (selected) ElementBlue else ElementCardBorder,
                shape = RoundedCornerShape(16.dp),
            )
            .semantics {
                contentDescription = listOf(
                    "照片 ${asset.filename()}",
                    primaryBadge,
                    recommendationBadge?.takeIf { asset.isBestRecommendedAsset() },
                    auxiliaryBadges.joinToString(" ").takeIf { it.isNotBlank() },
                ).filterNotNull().joinToString(" ")
                stateDescription = when {
                    selected -> "已选择"
                    selectionMode -> "未选择"
                    else -> "可打开"
                }
            }
            .combinedClickable(
                onClick = onClick,
                onLongClick = onLongClick,
            )
            .padding(8.dp),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(1.22f),
        ) {
            PhotoPreview(
                asset = asset,
                compactFallback = true,
                backgroundColor = asset.previewAccentColor().copy(alpha = 0.16f),
                modifier = Modifier.matchParentSize(),
            )
            burstBadge?.let {
                PhotoEdgeBadge(
                    text = it,
                    color = ElementPurple,
                    modifier = Modifier
                        .align(Alignment.TopStart)
                        .padding(6.dp),
                )
            }
            primaryBadge?.let {
                PhotoEdgeBadge(
                    text = it,
                    color = asset.modelScoreText()?.let { asset.modelScoreColor() }
                        ?: asset.tilePrimaryBadgeColor(),
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .padding(6.dp),
                )
            }
            recommendationBadge?.takeIf { asset.isBestRecommendedAsset() }?.let {
                PhotoEdgeBadge(
                    text = it,
                    color = ElementSuccess,
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(6.dp),
                )
            }
        }
        Spacer(Modifier.height(8.dp))
        Text(
            asset.filename(),
            fontSize = 12.sp,
            lineHeight = 14.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        if (auxiliaryBadges.isNotEmpty()) {
            Spacer(Modifier.height(6.dp))
            Row(
                horizontalArrangement = Arrangement.spacedBy(5.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                auxiliaryBadges.forEach { badge ->
                    PhotoInlineBadge(
                        text = badge,
                        color = auxiliaryBadgeColor(badge),
                    )
                }
            }
        }
    }
}

@Composable
private fun PhotoEdgeBadge(
    text: String,
    color: Color,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier,
        color = ElementBackground.copy(alpha = 0.78f),
        contentColor = color,
        shape = RoundedCornerShape(999.dp),
        border = BorderStroke(1.dp, color.copy(alpha = 0.46f)),
    ) {
        Text(
            text = text,
            modifier = Modifier.padding(horizontal = 7.dp, vertical = 3.dp),
            fontSize = 10.sp,
            lineHeight = 11.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun PhotoInlineBadge(
    text: String,
    color: Color,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier,
        color = color.copy(alpha = 0.12f),
        contentColor = color,
        shape = RoundedCornerShape(999.dp),
        border = BorderStroke(1.dp, color.copy(alpha = 0.36f)),
    ) {
        Text(
            text = text,
            modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
            fontSize = 9.sp,
            lineHeight = 10.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

private fun auxiliaryBadgeColor(text: String): Color =
    when (text) {
        "收藏" -> ElementSuccess
        "标记" -> ElementBlue
        "风险", "不支持预览" -> ElementDanger
        "RAW", "JPG+RAW" -> ElementPurple
        "视频" -> ElementInfo
        else -> ElementInfo
    }

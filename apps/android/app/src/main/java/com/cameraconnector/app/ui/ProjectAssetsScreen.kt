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
import androidx.compose.runtime.DisposableEffect
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
import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetQuery
import com.cameraconnector.app.core.InboxAssetRole
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.PublishQueueState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.core.ReviewQueueSummary
import com.cameraconnector.app.media.PREVIEW_DETAIL_FALLBACK_ASPECT_RATIO
import com.cameraconnector.app.media.PhotoMetadata
import com.cameraconnector.app.media.PreviewQuality
import com.cameraconnector.app.media.cacheThumbnailPreview
import com.cameraconnector.app.media.cachedThumbnailPreview
import com.cameraconnector.app.media.isDecodablePreviewLocation
import com.cameraconnector.app.media.loadPhotoMetadata
import com.cameraconnector.app.media.loadPreviewBitmap
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

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
    onConfigureAccount: () -> Unit,
    onRequestNotificationPermission: () -> Unit,
    actionsEnabled: Boolean,
    onStartReceiver: (ReceiverSettings, String) -> Unit,
    onStopReceiver: () -> Unit,
    cameraConnectHost: String,
    onRetryFailedPublishes: () -> Unit,
    onMoveProjectGroup: (String, String, String) -> Unit,
    onAcceptRecommendedBest: (String) -> Unit,
    onOverrideRecommendedBest: (String, String) -> Unit,
    onMarkBurstNeedsReview: (String) -> Unit,
    onRestoreAutomaticRecommendation: (String) -> Unit,
    onClearRecommendation: (String) -> Unit,
    onKeepAllCandidates: (String) -> Unit,
    onHideLowScoreCandidates: (String) -> Unit,
    onSplitBurstMember: (String, String) -> Unit,
    onMergeBurstMember: (String, String) -> Unit,
    gridColumnCount: Int,
    selectedStrategyProfileId: String,
    onImmersiveModeChange: (Boolean) -> Unit = {},
    modifier: Modifier = Modifier,
) {
    var selectedAccount by remember { mutableStateOf<String?>(null) }
    var selectedFilter by remember { mutableStateOf(InboxFilter.All) }
    var selectedSort by remember { mutableStateOf(PhotoSortMode.LatestReceived) }
    var selectedScoreFilter by remember { mutableStateOf(ScoreFilter.All) }
    var selectedPhotoCollection by rememberSaveable { mutableStateOf(ProjectPhotoCollection.All) }
    var selectedPhoto by remember { mutableStateOf<InboxAsset?>(null) }
    var selectedBurstGridItem by remember { mutableStateOf<ProjectPhotoGridItemUi?>(null) }
    var selectedAssetIds by rememberSaveable { mutableStateOf(emptyList<String>()) }
    var movePickerOpen by remember { mutableStateOf(false) }
    var filterExpanded by remember { mutableStateOf(false) }
    var reviewModeActive by rememberSaveable { mutableStateOf(false) }
    var reviewModeIndex by rememberSaveable { mutableStateOf(0) }
    var reviewSessionAcceptedCount by rememberSaveable { mutableStateOf(0) }
    var reviewSessionManualOverrideCount by rememberSaveable { mutableStateOf(0) }
    var reviewSessionNeedsReviewCount by rememberSaveable { mutableStateOf(0) }
    var reviewSessionRestoredAutomaticCount by rememberSaveable { mutableStateOf(0) }
    var reviewSessionClearedRecommendationCount by rememberSaveable { mutableStateOf(0) }
    var reviewSessionKeptAllCandidatesCount by rememberSaveable { mutableStateOf(0) }
    var reviewSessionHiddenLowScoreCount by rememberSaveable { mutableStateOf(0) }
    var reviewSessionSkippedCount by rememberSaveable { mutableStateOf(0) }
    var reviewSessionUndoBurstGroupId by rememberSaveable { mutableStateOf<String?>(null) }
    var reviewSessionUndoDecisionName by rememberSaveable { mutableStateOf<String?>(null) }
    var reviewExitSummaryOpen by rememberSaveable { mutableStateOf(false) }
    var selectedReviewQueue by rememberSaveable { mutableStateOf<String?>(null) }
    val reviewSessionUndoDecision = remember(reviewSessionUndoDecisionName) {
        reviewSessionUndoDecisionName?.let { name ->
            runCatching { ReviewSessionDecision.valueOf(name) }.getOrNull()
        }
    }
    val reviewSession = remember(
        reviewSessionAcceptedCount,
        reviewSessionManualOverrideCount,
        reviewSessionNeedsReviewCount,
        reviewSessionRestoredAutomaticCount,
        reviewSessionClearedRecommendationCount,
        reviewSessionKeptAllCandidatesCount,
        reviewSessionHiddenLowScoreCount,
        reviewSessionSkippedCount,
        reviewSessionUndoBurstGroupId,
        reviewSessionUndoDecision,
    ) {
        ReviewModeSessionUi(
            processedGroupCount = reviewSessionAcceptedCount +
                reviewSessionManualOverrideCount +
                reviewSessionNeedsReviewCount +
                reviewSessionRestoredAutomaticCount +
                reviewSessionClearedRecommendationCount +
                reviewSessionKeptAllCandidatesCount +
                reviewSessionHiddenLowScoreCount +
                reviewSessionSkippedCount,
            acceptedRecommendationCount = reviewSessionAcceptedCount,
            manualOverrideCount = reviewSessionManualOverrideCount,
            markedNeedsReviewCount = reviewSessionNeedsReviewCount,
            restoredAutomaticCount = reviewSessionRestoredAutomaticCount,
            clearedRecommendationCount = reviewSessionClearedRecommendationCount,
            keptAllCandidatesCount = reviewSessionKeptAllCandidatesCount,
            hiddenLowScoreCount = reviewSessionHiddenLowScoreCount,
            skippedCount = reviewSessionSkippedCount,
            undoBurstGroupId = reviewSessionUndoBurstGroupId,
            undoDecision = reviewSessionUndoDecision,
        )
    }
    val reviewQueueSummary by produceState<ReviewQueueSummary?>(
        initialValue = null,
        projectState.activeProjectId,
        selectedStrategyProfileId,
        dashboard.inbox,
    ) {
        val projectId = projectState.activeProjectId
        value = if (projectId == null) {
            null
        } else {
            withContext(Dispatchers.IO) {
                coreGateway.loadReviewQueueSummary(projectId, selectedStrategyProfileId)
            }
        }
    }
    val reviewQueueEntries = remember(reviewQueueSummary) {
        reviewQueueSummary?.reviewQueueEntriesUi().orEmpty()
    }
    val reviewQueueEntry = remember(reviewQueueEntries, selectedReviewQueue) {
        reviewQueueEntries.selectedReviewQueueEntry(selectedReviewQueue)
    }
    val inboxQuery = remember(
        selectedAccount,
        selectedFilter,
        selectedSort,
        selectedScoreFilter,
        reviewModeActive,
        reviewQueueEntry,
        selectedStrategyProfileId,
    ) {
        val reviewQuery = reviewQueueEntry?.assetQuery(selectedAccount, selectedStrategyProfileId)
            ?.takeIf { reviewModeActive }
        if (reviewQuery != null) {
            reviewQuery.copy(role = selectedFilter.assetRole())
        } else {
            assetListQuery(
                selectedAccount = selectedAccount,
                selectedFilter = selectedFilter,
                selectedSort = selectedSort,
                selectedScoreFilter = selectedScoreFilter,
            )
        }
    }
    val filteredAssets by produceState<List<InboxAsset>>(
        initialValue = dashboard.inbox,
        projectState.activeProjectId,
        inboxQuery,
        selectedPhotoCollection,
        selectedAccount,
        selectedFilter,
        selectedSort,
        selectedScoreFilter,
        selectedStrategyProfileId,
        dashboard.inbox,
    ) {
        value = withContext(Dispatchers.IO) {
            if (reviewModeActive) {
                coreGateway.loadInbox(inboxQuery)
            } else if (selectedPhotoCollection == ProjectPhotoCollection.Selects) {
                val projectId = projectState.activeProjectId
                if (projectId == null) {
                    emptyList()
                } else {
                    projectPhotoCollectionAssets(
                        assets = coreGateway.loadSelects(
                            projectId = projectId,
                            strategyProfileId = selectedStrategyProfileId,
                        ),
                        selectedAccount = selectedAccount,
                        selectedFilter = selectedFilter,
                        selectedSort = selectedSort,
                        selectedScoreFilter = selectedScoreFilter,
                    )
                }
            } else {
                coreGateway.loadInbox(inboxQuery)
            }
        }
    }
    val selectionMode = isAssetSelectionMode(selectedAssetIds)
    val selectedAssets = remember(filteredAssets, selectedAssetIds) {
        selectedAssetsFromIds(filteredAssets, selectedAssetIds)
    }
    val selectedBurstMergeTarget = remember(selectedAssets) {
        manualBurstMergeTarget(selectedAssets)
    }
    val sourceProjectId = projectState.activeProjectId
    val moveTargets = remember(projectState.projects, sourceProjectId) {
        projectState.groupMoveTargets(sourceProjectId)
    }
    var receiverPanelExpanded by remember { mutableStateOf(!dashboard.receiver.running) }
    val receiverConnectHost = normalizeCameraConnectHost(cameraConnectHost)
    LaunchedEffect(reviewModeActive) {
        onImmersiveModeChange(reviewModeActive)
    }
    DisposableEffect(Unit) {
        onDispose { onImmersiveModeChange(false) }
    }
    fun resetReviewSession() {
        reviewSessionAcceptedCount = 0
        reviewSessionManualOverrideCount = 0
        reviewSessionNeedsReviewCount = 0
        reviewSessionRestoredAutomaticCount = 0
        reviewSessionClearedRecommendationCount = 0
        reviewSessionKeptAllCandidatesCount = 0
        reviewSessionHiddenLowScoreCount = 0
        reviewSessionSkippedCount = 0
        reviewSessionUndoBurstGroupId = null
        reviewSessionUndoDecisionName = null
        reviewExitSummaryOpen = false
    }
    fun applyReviewSession(nextSession: ReviewModeSessionUi) {
        reviewSessionAcceptedCount = nextSession.acceptedRecommendationCount
        reviewSessionManualOverrideCount = nextSession.manualOverrideCount
        reviewSessionNeedsReviewCount = nextSession.markedNeedsReviewCount
        reviewSessionRestoredAutomaticCount = nextSession.restoredAutomaticCount
        reviewSessionClearedRecommendationCount = nextSession.clearedRecommendationCount
        reviewSessionKeptAllCandidatesCount = nextSession.keptAllCandidatesCount
        reviewSessionHiddenLowScoreCount = nextSession.hiddenLowScoreCount
        reviewSessionSkippedCount = nextSession.skippedCount
        reviewSessionUndoBurstGroupId = nextSession.undoBurstGroupId
        reviewSessionUndoDecisionName = nextSession.undoDecision?.name
    }
    fun recordReviewDecision(decision: ReviewSessionDecision, burstGroupId: String) {
        applyReviewSession(reviewSession.record(decision, burstGroupId))
    }
    fun advanceReviewModeAfterAction() {
        if (reviewModeShouldSummarizeAfterAction(reviewModeIndex, filteredAssets.size)) {
            reviewExitSummaryOpen = true
        } else {
            reviewModeIndex = nextReviewIndex(reviewModeIndex, filteredAssets.size)
        }
    }
    fun undoLatestReviewDecision() {
        val burstGroupId = reviewSession.undoBurstGroupId ?: return
        onRestoreAutomaticRecommendation(burstGroupId)
        applyReviewSession(reviewSession.undoLatestDecision())
    }
    fun leaveReviewMode() {
        reviewModeActive = false
        reviewModeIndex = 0
        resetReviewSession()
    }
    fun requestReviewModeExit() {
        if (reviewSession.hasActivity) {
            reviewExitSummaryOpen = true
        } else {
            leaveReviewMode()
        }
    }

    LaunchedEffect(dashboard.receiver.running) {
        receiverPanelExpanded = !dashboard.receiver.running
    }

    LaunchedEffect(projectState.activeProjectId, inboxQuery, selectedPhotoCollection) {
        selectedAssetIds = emptyList()
        movePickerOpen = false
        selectedBurstGridItem = null
    }
    LaunchedEffect(reviewQueueEntries, selectedReviewQueue) {
        if (selectedReviewQueue != null && reviewQueueEntries.none { it.queue == selectedReviewQueue }) {
            selectedReviewQueue = reviewQueueEntries.firstOrNull()?.queue
            reviewModeIndex = 0
        }
    }
    LaunchedEffect(
        projectState.activeProjectId,
        reviewQueueEntry,
        reviewModeActive,
        reviewSession.processedGroupCount,
    ) {
        if (reviewQueueEntry == null) {
            if (reviewModeActive && reviewSession.hasActivity) {
                reviewExitSummaryOpen = true
            }
            reviewModeActive = false
            reviewModeIndex = 0
        }
    }
    LaunchedEffect(reviewModeActive, filteredAssets.size) {
        if (!reviewModeActive) {
            reviewModeIndex = 0
        } else {
            reviewModeIndex = reviewModeProgress(reviewModeIndex, filteredAssets.size).currentIndex
        }
    }
    LaunchedEffect(reviewModeActive, filteredAssets.size, reviewSession.processedGroupCount) {
        if (reviewModeActive && filteredAssets.isEmpty() && reviewSession.hasActivity) {
            reviewExitSummaryOpen = true
        }
    }
    LaunchedEffect(filteredAssets, selectedPhoto?.assetSelectionId()) {
        val currentPhoto = selectedPhoto
        val refreshedPhoto = refreshedSelectedPhoto(currentPhoto, filteredAssets)
        if (refreshedPhoto != currentPhoto) {
            selectedPhoto = refreshedPhoto
        }
    }

    if (reviewExitSummaryOpen) {
        ReviewSessionSummaryDialog(
            session = reviewSession,
            remainingReviewGroupCount = filteredAssets.size,
            lowScoreCandidateCount = reviewQueueSummary?.lowScoreCandidateCount ?: 0,
            onContinue = { reviewExitSummaryOpen = false },
            onExit = { leaveReviewMode() },
            canContinue = reviewModeActive && filteredAssets.isNotEmpty(),
        )
    }
    selectedBurstGridItem?.let { item ->
        BurstGroupPreviewDialog(
            item = item,
            allProjectAssets = dashboard.inbox,
            onDismiss = { selectedBurstGridItem = null },
            onOpenAsset = { asset ->
                selectedBurstGridItem = null
                selectedPhoto = asset
            },
        )
    }
    selectedPhoto?.let { photo ->
        BackHandler {
            selectedPhoto = null
        }
        PhotoDetailScreen(
            asset = photo,
            onBack = { selectedPhoto = null },
            actionsEnabled = actionsEnabled,
            onAcceptRecommendedBest = { burstGroupId ->
                onAcceptRecommendedBest(burstGroupId)
                if (reviewModeActive) {
                    recordReviewDecision(ReviewSessionDecision.AcceptRecommendedBest, burstGroupId)
                }
            },
            onOverrideRecommendedBest = { burstGroupId, bestAssetGroupId ->
                onOverrideRecommendedBest(burstGroupId, bestAssetGroupId)
                if (reviewModeActive) {
                    recordReviewDecision(
                        ReviewSessionDecision.OverrideRecommendedBest,
                        burstGroupId,
                    )
                }
            },
            onMarkBurstNeedsReview = { burstGroupId ->
                onMarkBurstNeedsReview(burstGroupId)
                if (reviewModeActive) {
                    recordReviewDecision(ReviewSessionDecision.MarkNeedsReview, burstGroupId)
                }
            },
            onRestoreAutomaticRecommendation = { burstGroupId ->
                onRestoreAutomaticRecommendation(burstGroupId)
                if (reviewModeActive) {
                    recordReviewDecision(
                        ReviewSessionDecision.RestoreAutomaticRecommendation,
                        burstGroupId,
                    )
                }
            },
            onSplitBurstMember = { burstGroupId, memberGroupId ->
                onSplitBurstMember(burstGroupId, memberGroupId)
            },
            burstMembers = burstMemberFilmstrip(photo, dashboard.inbox),
            comparisonItems = burstComparisonItems(photo, dashboard.inbox),
            onOpenBurstMember = { asset -> selectedPhoto = asset },
            modifier = modifier,
        )
        return
    }
    BackHandler(enabled = selectionMode) {
        selectedAssetIds = emptyList()
        movePickerOpen = false
    }
    BackHandler(enabled = reviewModeActive) {
        requestReviewModeExit()
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
            .then(if (receiverPanelExpanded && !reviewModeActive) Modifier.blur(6.dp) else Modifier)
            .animateContentSize(),
        ) {
        actionError?.let { message ->
            ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError)
            Spacer(Modifier.height(10.dp))
        }

        if (!reviewModeActive) {
            if (!receiverPanelExpanded) {
                ProjectReceiverStatusStrip(
                    dashboard = dashboard,
                    projectState = projectState,
                    onExpand = { receiverPanelExpanded = true },
                    connectHost = receiverConnectHost,
                    modifier = Modifier.fillMaxWidth(),
                )
                Spacer(Modifier.height(10.dp))
            }
        }

        if (projectPhotoContentVisible(dashboard.receiver.running, reviewModeActive)) {
            if (!reviewModeActive) {
                PhotoListControlRow(
                    selectedCollection = selectedPhotoCollection,
                    onCollectionChange = { collection ->
                        selectedPhotoCollection = collection
                        reviewModeActive = false
                    },
                    selectedAccount = selectedAccount,
                    selectedFilter = selectedFilter,
                    selectedSort = selectedSort,
                    selectedScoreFilter = selectedScoreFilter,
                    expanded = filterExpanded,
                    onToggle = { filterExpanded = !filterExpanded },
                )
                if (reviewQueueEntries.isNotEmpty()) {
                    Spacer(Modifier.height(8.dp))
                    ReviewQueueSwitchBar(
                        entries = reviewQueueEntries,
                        selectedQueue = reviewQueueEntry?.queue.orEmpty(),
                        onQueueChange = { queue ->
                            selectedReviewQueue = queue
                            reviewModeIndex = 0
                        },
                        onOpenBrushMode = { entry ->
                            selectedPhotoCollection = ProjectPhotoCollection.All
                            selectedReviewQueue = entry.queue
                            reviewModeActive = true
                            reviewModeIndex = 0
                            resetReviewSession()
                            selectedSort = PhotoSortMode.GroupBestScore
                            filterExpanded = false
                        },
                    )
                }
                if (filterExpanded) {
                    Spacer(Modifier.height(8.dp))
                    AccountFilterBar(
                        selectedAccount = selectedAccount,
                        onAccountChange = { selectedAccount = it },
                        assets = dashboard.inbox,
                    )
                    Spacer(Modifier.height(8.dp))
                    InboxFilterBar(
                        selectedFilter = selectedFilter,
                        onFilterChange = { selectedFilter = it },
                        assets = dashboard.inbox.filter { selectedAccount == null || it.username == selectedAccount },
                    )
                    Spacer(Modifier.height(8.dp))
                    ScoreFilterBar(
                        selectedScoreFilter = selectedScoreFilter,
                        onScoreFilterChange = { selectedScoreFilter = it },
                    )
                    Spacer(Modifier.height(8.dp))
                    PhotoSortBar(
                        selectedSort = selectedSort,
                        onSortChange = { selectedSort = it },
                    )
                }
                Spacer(Modifier.height(10.dp))
            }
            if (reviewModeActive && reviewQueueEntry != null) {
                val reviewAsset = filteredAssets.getOrNull(
                    reviewModeProgress(reviewModeIndex, filteredAssets.size).currentIndex,
                )
                ReviewModeScreen(
                    assets = filteredAssets,
                    currentIndex = reviewModeIndex,
                    queueEntry = reviewQueueEntry,
                    onCurrentIndexChange = { reviewModeIndex = it },
                    onOpenAsset = { selectedPhoto = it },
                    onAcceptRecommendedBest = { asset ->
                        reviewDecisionBurstGroupId(
                            asset,
                            ReviewDecisionAction.AcceptRecommendedBest,
                        )?.let { burstGroupId ->
                            onAcceptRecommendedBest(burstGroupId)
                            recordReviewDecision(ReviewSessionDecision.AcceptRecommendedBest, burstGroupId)
                            advanceReviewModeAfterAction()
                        }
                    },
                    onOverrideRecommendedBest = { asset ->
                        manualBestOverrideTarget(asset)?.let { target ->
                            onOverrideRecommendedBest(target.burstGroupId, target.bestAssetGroupId)
                            recordReviewDecision(
                                ReviewSessionDecision.OverrideRecommendedBest,
                                target.burstGroupId,
                            )
                            advanceReviewModeAfterAction()
                        }
                    },
                    onMarkNeedsReview = { asset ->
                        reviewDecisionBurstGroupId(
                            asset,
                            ReviewDecisionAction.MarkNeedsReview,
                        )?.let { burstGroupId ->
                            onMarkBurstNeedsReview(burstGroupId)
                            recordReviewDecision(ReviewSessionDecision.MarkNeedsReview, burstGroupId)
                            advanceReviewModeAfterAction()
                        }
                    },
                    onRestoreAutomaticRecommendation = { asset ->
                        reviewDecisionBurstGroupId(
                            asset,
                            ReviewDecisionAction.RestoreAutomaticRecommendation,
                        )?.let { burstGroupId ->
                            onRestoreAutomaticRecommendation(burstGroupId)
                            recordReviewDecision(
                                ReviewSessionDecision.RestoreAutomaticRecommendation,
                                burstGroupId,
                            )
                            advanceReviewModeAfterAction()
                        }
                    },
                    onClearRecommendation = { asset ->
                        asset.burst?.burstGroupId?.takeIf { it.isNotBlank() }?.let { burstGroupId ->
                            onClearRecommendation(burstGroupId)
                            recordReviewDecision(ReviewSessionDecision.ClearRecommendation, burstGroupId)
                            advanceReviewModeAfterAction()
                        }
                    },
                    onKeepAllCandidates = { asset ->
                        asset.burst?.burstGroupId?.takeIf { it.isNotBlank() }?.let { burstGroupId ->
                            onKeepAllCandidates(burstGroupId)
                            recordReviewDecision(ReviewSessionDecision.KeepAllCandidates, burstGroupId)
                            advanceReviewModeAfterAction()
                        }
                    },
                    onHideLowScoreCandidates = { asset ->
                        asset.burst?.burstGroupId?.takeIf { it.isNotBlank() }?.let { burstGroupId ->
                            onHideLowScoreCandidates(burstGroupId)
                            recordReviewDecision(ReviewSessionDecision.HideLowScoreCandidates, burstGroupId)
                            advanceReviewModeAfterAction()
                        }
                    },
                    onSplitBurstMember = { target ->
                        onSplitBurstMember(target.burstGroupId, target.memberGroupId)
                    },
                    onSkipCurrent = {
                        recordReviewDecision(ReviewSessionDecision.SkipCurrent, "")
                        advanceReviewModeAfterAction()
                    },
                    onUndoLatestDecision = { undoLatestReviewDecision() },
                    onExit = { requestReviewModeExit() },
                    actionsEnabled = actionsEnabled,
                    session = reviewSession,
                    burstMembers = reviewAsset
                        ?.let { burstMemberFilmstrip(it, dashboard.inbox) }
                        .orEmpty(),
                    comparisonItems = reviewAsset
                        ?.let { burstComparisonItems(it, dashboard.inbox) }
                        .orEmpty(),
                    modifier = Modifier.fillMaxSize(),
                )
            } else if (filteredAssets.isEmpty()) {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        when {
                            selectedPhotoCollection == ProjectPhotoCollection.Selects -> "还没有精选照片。"
                            dashboard.inbox.isEmpty() -> "还没有导入文件。"
                            else -> "当前筛选下没有文件。"
                        },
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                val gridItems = remember(filteredAssets) {
                    projectPhotoGridItems(filteredAssets)
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
                            val selected = asset.assetSelectionId() in selectedAssetIds
                            CompactPhotoTile(
                                asset = asset,
                                selected = selected,
                                selectionMode = selectionMode,
                                onClick = {
                                    if (selectionMode) {
                                        selectedAssetIds = toggleAssetSelection(selectedAssetIds, asset)
                                    } else if (item.isBurstGroup) {
                                        selectedBurstGridItem = item
                                    } else {
                                        selectedPhoto = asset
                                    }
                                },
                                onLongClick = {
                                    selectedAssetIds = toggleAssetSelection(selectedAssetIds, asset)
                                },
                            )
                        }
                    }
                    if (selectionMode) {
                        SelectedAssetsActionBar(
                            selectedCount = selectedAssets.size,
                            canOpen = selectedAssets.size == 1,
                            canMove = actionsEnabled &&
                                sourceProjectId != null &&
                                selectedAssets.any { it.groupMoveId() != null } &&
                                moveTargets.isNotEmpty(),
                            canMerge = actionsEnabled && selectedBurstMergeTarget != null,
                            onOpen = {
                                selectedAssets.firstOrNull()?.let { asset ->
                                    selectedAssetIds = emptyList()
                                    selectedPhoto = asset
                                }
                            },
                            onMove = { movePickerOpen = true },
                            onMerge = {
                                selectedBurstMergeTarget?.let { target ->
                                    onMergeBurstMember(target.targetBurstGroupId, target.memberGroupId)
                                    selectedAssetIds = emptyList()
                                }
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
        if (receiverPanelExpanded && !reviewModeActive) {
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
    }
}


@Composable
internal fun ReviewSessionSummaryDialog(
    session: ReviewModeSessionUi,
    remainingReviewGroupCount: Int,
    lowScoreCandidateCount: Int,
    onContinue: () -> Unit,
    onExit: () -> Unit,
    canContinue: Boolean,
) {
    AlertDialog(
        onDismissRequest = onContinue,
        title = { Text("本轮刷图小结") },
        text = {
            Text(
                reviewModeSessionExitSummaryText(
                    session = session,
                    remainingReviewGroupCount = remainingReviewGroupCount,
                    lowScoreCandidateCount = lowScoreCandidateCount,
                ),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        },
        confirmButton = {
            TextButton(onClick = onExit) {
                Text("退出刷图")
            }
        },
        dismissButton = if (canContinue) {
            {
                TextButton(onClick = onContinue) {
                    Text("继续刷图")
                }
            }
        } else {
            null
        },
    )
}

@Composable
internal fun BurstGroupPreviewDialog(
    item: ProjectPhotoGridItemUi,
    allProjectAssets: List<InboxAsset>,
    onDismiss: () -> Unit,
    onOpenAsset: (InboxAsset) -> Unit,
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
                    scoreText = asset.qualityScoreText(),
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
                                "连拍组",
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
                                positionText = "${index + 1}/${previewItems.size}",
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
    positionText: String,
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
                text = positionText,
                color = ElementPurple,
                modifier = Modifier
                    .align(Alignment.TopStart)
                    .padding(5.dp),
            )
            if (item.asset.isBestRecommendedAsset()) {
                PhotoEdgeBadge(
                    text = "优选",
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
        item.scoreText?.let { score ->
            Text(
                "评分 $score",
                style = MaterialTheme.typography.labelSmall,
                color = smartBadgeColor(item.asset),
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
internal fun ProjectReceiverStatusStrip(
    dashboard: DashboardState,
    projectState: ProjectState,
    onExpand: () -> Unit,
    connectHost: String?,
    modifier: Modifier = Modifier,
) {
    val project = projectState.activeProjectSummary()
    Surface(
        modifier = modifier.clickable(onClick = onExpand),
        color = ElementControlSurface.copy(alpha = 0.86f),
        contentColor = MaterialTheme.colorScheme.onSurface,
        shape = RoundedCornerShape(14.dp),
        border = BorderStroke(1.dp, ElementBorder),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 12.dp, vertical = 9.dp),
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
            Spacer(Modifier.width(10.dp))
            ElementTag(
                text = if (dashboard.receiver.running) "接收中" else receiverPhaseLabel(dashboard.receiver.phase),
                color = if (dashboard.receiver.running) ElementSuccess else ElementInfo,
            )
            Spacer(Modifier.width(8.dp))
            Icon(
                imageVector = Icons.Outlined.KeyboardArrowDown,
                contentDescription = "展开接收抽屉",
                tint = ElementBlue,
            )
        }
    }
}

@Composable
internal fun ProjectLaunchHeader(
    projectState: ProjectState,
    actionsEnabled: Boolean,
    onOpenProjects: () -> Unit,
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
            onClick = onCollapse,
            modifier = Modifier.size(34.dp),
        ) {
            Icon(
                imageVector = Icons.Outlined.KeyboardArrowUp,
                contentDescription = "收起启动页",
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
    val startBlockReason = receiverStartBlockReason(
        running = dashboard.receiver.running,
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
                enabled = actionsEnabled && (dashboard.receiver.running || receiverSettingsValid),
                retryEnabled = actionsEnabled,
                onToggleReceiver = {
                    if (dashboard.receiver.running) {
                        onStopReceiver()
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
                    enabled = actionsEnabled && !dashboard.receiver.running,
                    onClick = { protocol = "FTP" },
                    modifier = Modifier.weight(1f),
                )
                ProtocolSegment(
                    label = "SFTP",
                    selected = protocol == "SFTP",
                    enabled = actionsEnabled && !dashboard.receiver.running,
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
                enabled = actionsEnabled,
            )
            Spacer(Modifier.height(8.dp))
            OutlinedTextField(
                value = portInput,
                onValueChange = { portInput = it },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("端口") },
                singleLine = true,
                enabled = actionsEnabled && !dashboard.receiver.running,
            )
            Spacer(Modifier.height(8.dp))
            Text(
                "输出目录：${dashboard.receiver.outputLabel}",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            if (dashboard.receiver.running) {
                Spacer(Modifier.height(8.dp))
                Text(
                    "修改配置前需要先停止接收。收起后照片列表会占满主要空间。",
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
                        "接收服务使用账号认证。请先创建相机账号，再启动接收。"
                    ReceiverStartBlockReason.MissingNotificationPermission ->
                        "接收服务会以前台服务运行，需要先允许通知权限。"
                    ReceiverStartBlockReason.Busy ->
                        "当前还有操作未完成，请稍后再启动接收。"
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
                        ReceiverStartBlockReason.MissingAccount -> "去配置账号"
                        ReceiverStartBlockReason.MissingNotificationPermission -> "开启权限"
                        ReceiverStartBlockReason.Busy -> "知道了"
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
    canOpen: Boolean,
    canMove: Boolean,
    canMerge: Boolean,
    onOpen: () -> Unit,
    onMove: () -> Unit,
    onMerge: () -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier,
        color = ElementPanel.copy(alpha = 0.96f),
        contentColor = MaterialTheme.colorScheme.onSurface,
        shape = RoundedCornerShape(18.dp),
        border = BorderStroke(1.dp, ElementBlue.copy(alpha = 0.35f)),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                "已选择 $selectedCount 个分组",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
            )
            Row(
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Button(
                    onClick = onOpen,
                    enabled = canOpen,
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(10.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = ElementBlue,
                        contentColor = ElementOnAccent,
                    ),
                    contentPadding = PaddingValues(horizontal = 18.dp, vertical = 0.dp),
                ) {
                    Text("打开")
                }
                OutlinedButton(
                    onClick = onMove,
                    enabled = canMove,
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(10.dp),
                    border = BorderStroke(1.dp, ElementBorder),
                    colors = ButtonDefaults.outlinedButtonColors(
                        containerColor = ElementControlSurface,
                        contentColor = MaterialTheme.colorScheme.onSurface,
                    ),
                    contentPadding = PaddingValues(horizontal = 18.dp, vertical = 0.dp),
                ) {
                    Text("移动")
                }
                OutlinedButton(
                    onClick = onMerge,
                    enabled = canMerge,
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(10.dp),
                    border = BorderStroke(1.dp, ElementPurple.copy(alpha = 0.45f)),
                    colors = ButtonDefaults.outlinedButtonColors(
                        containerColor = ElementControlSurface,
                        contentColor = ElementPurple,
                    ),
                    contentPadding = PaddingValues(horizontal = 18.dp, vertical = 0.dp),
                ) {
                    Text("\u5408\u5e76\u8fde\u62cd", maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                OutlinedButton(
                    onClick = onCancel,
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(10.dp),
                    border = BorderStroke(1.dp, ElementBorder),
                    colors = ButtonDefaults.outlinedButtonColors(
                        containerColor = ElementControlSurface,
                        contentColor = MaterialTheme.colorScheme.onSurfaceVariant,
                    ),
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 0.dp),
                ) {
                    Text("取消")
                }
            }
        }
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
                    "移动 $selectedCount 个分组",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.Bold,
                )
                if (targets.isEmpty()) {
                    Text(
                        "当前没有可移动到的项目",
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
    selectedAccount: String?,
    selectedFilter: InboxFilter,
    selectedSort: PhotoSortMode,
    selectedScoreFilter: ScoreFilter,
    expanded: Boolean,
    onToggle: () -> Unit,
) {
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
        Text(
            listOf(
                selectedAccount?.let { "账号：$it" } ?: "全部账号",
                selectedFilter.label,
                selectedScoreFilter.label,
                selectedSort.label,
            ).joinToString(" / "),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.bodySmall,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
internal fun FilterToggleRow(
    selectedAccount: String?,
    selectedFilter: InboxFilter,
    selectedSort: PhotoSortMode,
    selectedScoreFilter: ScoreFilter,
    expanded: Boolean,
    onToggle: () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onToggle)
            .padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column {
            Text("筛选", style = MaterialTheme.typography.titleSmall)
            Spacer(Modifier.height(2.dp))
            Text(
                listOf(
                    selectedAccount?.let { "账号：$it" } ?: "全部账号",
                    selectedFilter.label,
                    selectedScoreFilter.label,
                    selectedSort.label,
                ).joinToString(" / "),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Text(if (expanded) "收起 ▲" else "展开 ▼", color = ElementBlue, fontWeight = FontWeight.SemiBold)
    }
}

@Composable
internal fun ScoreFilterBar(
    selectedScoreFilter: ScoreFilter,
    onScoreFilterChange: (ScoreFilter) -> Unit,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(ScoreFilter.entries) { filter ->
            FilterChipButton(
                label = filter.label,
                selected = selectedScoreFilter == filter,
                onClick = { onScoreFilterChange(filter) },
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
internal fun ProjectPhotoCollectionBar(
    selectedCollection: ProjectPhotoCollection,
    onCollectionChange: (ProjectPhotoCollection) -> Unit,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(ProjectPhotoCollection.entries) { collection ->
            FilterChipButton(
                label = collection.label,
                selected = selectedCollection == collection,
                onClick = { onCollectionChange(collection) },
            )
        }
    }
}

@Composable
internal fun AccountFilterBar(
    selectedAccount: String?,
    onAccountChange: (String?) -> Unit,
    assets: List<InboxAsset>,
) {
    val accounts = remember(assets) {
        assets.mapNotNull { it.username?.takeIf { username -> username.isNotBlank() } }
            .distinct()
            .sorted()
    }
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        item {
            FilterChipButton(
                label = "全部账号 ${assets.size}",
                selected = selectedAccount == null,
                onClick = { onAccountChange(null) },
            )
        }
        items(accounts) { account ->
            val count = assets.count { it.username == account }
            FilterChipButton(
                label = "账号：$account $count",
                selected = selectedAccount == account,
                onClick = { onAccountChange(account) },
            )
        }
    }
}

@Composable
internal fun InboxFilterBar(
    selectedFilter: InboxFilter,
    onFilterChange: (InboxFilter) -> Unit,
    assets: List<InboxAsset>,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(InboxFilter.entries) { filter ->
            val count = assets.count { filter.matches(it) }
            FilterChipButton(
                label = "${filter.label} $count",
                selected = selectedFilter == filter,
                onClick = { onFilterChange(filter) },
            )
        }
    }
}

@Composable
internal fun ReviewQueueEntryStrip(
    entry: ReviewQueueEntryUi,
    active: Boolean,
    onOpenBrushMode: () -> Unit,
    onClearBrushMode: () -> Unit,
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = if (active) onClearBrushMode else onOpenBrushMode),
        color = if (active) ElementBlueSoft else ElementControlSurface.copy(alpha = 0.9f),
        contentColor = MaterialTheme.colorScheme.onSurface,
        shape = RoundedCornerShape(14.dp),
        border = BorderStroke(1.dp, if (active) ElementBlue.copy(alpha = 0.5f) else ElementBorder),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        if (active) "刷图模式" else entry.primaryLabel,
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                    )
                    Spacer(Modifier.width(8.dp))
                    ElementTag(
                        text = entry.primaryText,
                        color = if (active) ElementBlue else ElementSuccess,
                    )
                }
                Spacer(Modifier.height(4.dp))
                Text(
                    entry.subtitle,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Spacer(Modifier.width(10.dp))
            Text(
                if (active) "退出" else "开始",
                color = ElementBlue,
                fontWeight = FontWeight.SemiBold,
            )
        }
    }
}

@Composable
internal fun ReviewQueueSwitchBar(
    entries: List<ReviewQueueEntryUi>,
    selectedQueue: String,
    onQueueChange: (String) -> Unit,
    onOpenBrushMode: (ReviewQueueEntryUi) -> Unit,
) {
    val targetEntry = entries.firstOrNull { it.queue == selectedQueue } ?: entries.firstOrNull()
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        LazyRow(
            modifier = Modifier.weight(1f),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
        items(entries, key = { it.queue }) { entry ->
            FilterChipButton(
                label = "${entry.primaryLabel} ${entry.primaryCount}",
                selected = entry.queue == selectedQueue,
                onClick = { onQueueChange(entry.queue) },
            )
        }
        }
        targetEntry?.let { entry ->
                FilterChipButton(
                    label = "筛图",
                    selected = false,
                    onClick = { onOpenBrushMode(entry) },
                )
            }
        }
    }
@OptIn(ExperimentalFoundationApi::class)
@Composable
internal fun CompactPhotoTile(
    asset: InboxAsset,
    selected: Boolean,
    selectionMode: Boolean,
    onClick: () -> Unit,
    onLongClick: () -> Unit,
) {
    val burstBadge = asset.burstCountBadgeText()
    val qualityBadge = asset.qualityBadgeText()
    val recommendationBadge = asset.recommendationBadgeText()
    val smartMeta = asset.tileSmartMeta()
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
                contentDescription = "照片 ${asset.filename()} ${asset.sourceLabel()} ${asset.formatBadges()}"
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
            qualityBadge?.let {
                PhotoEdgeBadge(
                    text = it,
                    color = smartBadgeColor(asset),
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
        Spacer(Modifier.height(4.dp))
        Text(
            listOf(asset.sourceLabel(), asset.formatBadges()).joinToString(" · "),
            fontSize = 10.sp,
            lineHeight = 12.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        smartMeta?.let {
            Spacer(Modifier.height(3.dp))
            Text(
                it,
                fontSize = 10.sp,
                lineHeight = 12.sp,
                color = smartBadgeColor(asset),
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
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

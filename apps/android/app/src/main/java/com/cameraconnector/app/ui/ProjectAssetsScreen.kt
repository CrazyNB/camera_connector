package com.cameraconnector.app.ui
import androidx.activity.compose.BackHandler
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
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
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.LanShareSessionUi
import com.cameraconnector.app.core.ModelEvaluationPreviewInput
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.media.PreviewQuality
import com.cameraconnector.app.media.loadPreviewBitmap
import com.cameraconnector.app.media.loadPreviewSampleJson
import com.cameraconnector.app.share.CoreLanShareGateway
import com.cameraconnector.app.share.LAN_PROJECT_SYNC_DISCOVERY_PORT
import com.cameraconnector.app.share.LanShareDiscoveryInfo
import com.cameraconnector.app.share.LanShareHttpServer
import com.cameraconnector.app.share.LanShareRouter
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
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
    onOpenProjectIntelligence: () -> Unit,
    onConfigureAccount: () -> Unit,
    onRequestNotificationPermission: () -> Unit,
    actionsEnabled: Boolean,
    onStartReceiver: (ReceiverSettings) -> Unit,
    onStopReceiver: () -> Unit,
    receiverPanelExpanded: Boolean,
    onReceiverPanelExpandedChange: (Boolean) -> Unit,
    onRetryFailedPublishes: () -> Unit,
    onSplitBurstMember: (String, String) -> Unit,
    onSplitBurstMembers: (List<ManualBurstSplitTarget>) -> Unit,
    onCreateManualBurstGroup: (String, List<String>) -> Unit,
    onSetAssetGroupUserMarks: (String, String, Boolean?, Boolean?) -> Unit,
    gridColumnCount: Int,
    modifier: Modifier = Modifier,
) {
    var selectedFilter by remember { mutableStateOf(AssetFormatFilter.All) }
    var selectedSort by remember { mutableStateOf(PhotoSortMode.LatestReceived) }
    var selectedGuestMarkFilter by remember { mutableStateOf(GuestMarkFilter.All) }
    var selectedMinModelScore by remember { mutableStateOf<Int?>(null) }
    var selectedPhotoCollection by rememberSaveable { mutableStateOf(ProjectPhotoCollection.All) }
    var selectedPhoto by remember { mutableStateOf<ProjectAsset?>(null) }
    var selectedBurstPreview by remember { mutableStateOf<ProjectPhotoGridItemUi?>(null) }
    var deleteSelectionCandidates by remember { mutableStateOf<List<ProjectAsset>>(emptyList()) }
    var deleteSelectionInFlight by remember { mutableStateOf(false) }
    var selectedAssetIds by rememberSaveable { mutableStateOf(emptyList<String>()) }
    var filterExpanded by remember { mutableStateOf(false) }
    var projectFeedbackMessage by remember { mutableStateOf<String?>(null) }
    var projectFeedbackToken by remember { mutableStateOf(0) }
    var lanShareSession by remember { mutableStateOf<LanShareSessionUi?>(null) }
    var lanSharePort by remember { mutableStateOf<Int?>(null) }
    var lanShareServer by remember { mutableStateOf<LanShareHttpServer?>(null) }
    var lanShareStarting by remember { mutableStateOf(false) }
    var lanShareError by remember { mutableStateOf<String?>(null) }
    var lanShareConfigVisible by remember { mutableStateOf(false) }
    var lanShareDialogAction by remember { mutableStateOf(LanShareMenuAction.GuestSelection) }
    var lanShareScopeSnapshot by remember { mutableStateOf<LanShareScopeUi?>(null) }
    var lanShareFavoriteOnly by rememberSaveable { mutableStateOf(false) }
    var lanShareMarkedOnly by rememberSaveable { mutableStateOf(false) }
    var lanShareMinModelScore by rememberSaveable { mutableStateOf<Int?>(null) }
    val assetQuery = remember(
        selectedPhotoCollection,
        selectedFilter,
        selectedSort,
        selectedGuestMarkFilter,
        selectedMinModelScore,
    ) {
        assetListQuery(
            selectedCollection = selectedPhotoCollection,
            selectedFilter = selectedFilter,
            selectedSort = selectedSort,
            selectedGuestMarkFilter = selectedGuestMarkFilter,
            selectedMinModelScore = selectedMinModelScore,
        )
    }
    val filteredAssets by produceState<List<ProjectAsset>>(
        initialValue = dashboard.assets,
        projectState.activeProjectId,
        assetQuery,
        selectedPhotoCollection,
        selectedFilter,
        selectedSort,
        selectedGuestMarkFilter,
        selectedMinModelScore,
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
    val selectedDeleteAssets = remember(selectedGridItems) {
        selectedGridItems
            .flatMap { it.members }
            .distinctBy { it.assetSelectionId() }
    }
    val selectedBurstMergeTarget = remember(selectedGridItems) {
        manualBurstMergeTarget(selectedGridItems)
    }
    val selectedBurstSplitTargets = remember(selectedGridItems) {
        manualBurstSplitTargets(selectedGridItems)
    }
    val sourceProjectId = projectState.activeProjectId
    val activeProject = projectState.activeProjectSummary()
    val context = LocalContext.current
    val clipboardManager = LocalClipboardManager.current
    val coroutineScope = rememberCoroutineScope()
    val localReceiverEndpointCandidates by produceState<List<ReceiverLanEndpointCandidate>>(initialValue = emptyList()) {
        value = withContext(Dispatchers.IO) { localReceiverLanEndpointCandidates(context) }
    }
    val localReceiverIpv4Addresses = remember(localReceiverEndpointCandidates) {
        localReceiverEndpointCandidates.map { it.host }
    }
    val advertisedReceiverHost = receiverAdvertisedHost(localReceiverIpv4Addresses)
    val lanShareActive = lanShareSession != null && lanSharePort != null
    val lanShareUrl = remember(lanShareSession, lanSharePort, advertisedReceiverHost) {
        val session = lanShareSession
        val port = lanSharePort
        val host = advertisedReceiverHost
        if (session != null && port != null && host != null) {
            "http://$host:$port/s/${session.token}"
        } else {
            null
        }
    }
    val lanShareAction = lanShareActionUi(
        activeProjectId = sourceProjectId,
        assetCount = 0,
        running = lanShareStarting,
    )
    val lanShareQuery = remember(
        assetQuery,
        lanShareFavoriteOnly,
        lanShareMarkedOnly,
        lanShareMinModelScore,
    ) {
        lanShareAssetQuery(
            baseQuery = assetQuery,
            favoriteOnly = lanShareFavoriteOnly,
            markedOnly = lanShareMarkedOnly,
            minModelScore = lanShareMinModelScore,
        )
    }
    val lanSharePreviewAssets by produceState<List<ProjectAsset>>(
        initialValue = emptyList(),
        projectState.activeProjectId,
        lanShareQuery,
        lanShareConfigVisible,
    ) {
        value = if (sourceProjectId == null) {
            emptyList()
        } else {
            withContext(Dispatchers.IO) {
                coreGateway.loadProjectAssets(lanShareQuery)
            }
        }
    }
    val projectSyncShareAssets by produceState<List<ProjectAsset>>(
        initialValue = emptyList(),
        projectState.activeProjectId,
        lanShareConfigVisible,
    ) {
        value = if (sourceProjectId == null) {
            emptyList()
        } else {
            withContext(Dispatchers.IO) {
                coreGateway.loadProjectAssets(ProjectAssetQuery())
            }
        }
    }
    val configuredLanShareAction = lanShareActionUi(
        activeProjectId = sourceProjectId,
        assetCount = when (lanShareDialogAction) {
            LanShareMenuAction.GuestSelection -> lanSharePreviewAssets.size
            LanShareMenuAction.ProjectSync -> projectSyncShareAssets.size
        },
        running = lanShareStarting,
    )
    val currentLanShareScope = LanShareScopeUi(
        projectName = activeProject?.name ?: "当前项目",
        favoriteOnly = lanShareFavoriteOnly,
        markedOnly = lanShareMarkedOnly,
        minModelScore = lanShareMinModelScore,
        assetCount = when (lanShareDialogAction) {
            LanShareMenuAction.GuestSelection -> lanSharePreviewAssets.size
            LanShareMenuAction.ProjectSync -> projectSyncShareAssets.size
        },
    )
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
    fun showLanShareDialog(action: LanShareMenuAction) {
        lanShareDialogAction = action
        lanShareConfigVisible = true
    }
    DisposableEffect(Unit) {
        onDispose {
            lanShareServer?.close()
        }
    }
    fun stopLanShare(showFeedback: Boolean = true) {
        val session = lanShareSession
        val server = lanShareServer
        lanShareSession = null
        lanSharePort = null
        lanShareServer = null
        lanShareError = null
        lanShareScopeSnapshot = null
        server?.close()
        if (session != null) {
            coroutineScope.launch {
                runCatching { coreGateway.stopLanShareSession(session.shareId) }
                if (showFeedback) {
                    showProjectFeedback("已停止共享")
                }
            }
        } else if (showFeedback) {
            showProjectFeedback("已停止共享")
        }
    }
    fun startLanShare() {
        val projectId = sourceProjectId ?: return
        if (!configuredLanShareAction.enabled) {
            return
        }
        val shareScope = currentLanShareScope
        lanShareStarting = true
        lanShareError = null
        coroutineScope.launch {
            var createdSession: LanShareSessionUi? = null
            var createdServer: LanShareHttpServer? = null
            runCatching {
                val session = coreGateway.createLanShareSession(
                    projectId = projectId,
                    query = when (lanShareDialogAction) {
                        LanShareMenuAction.GuestSelection -> lanShareQuery
                        LanShareMenuAction.ProjectSync -> ProjectAssetQuery()
                    },
                    title = when (lanShareDialogAction) {
                        LanShareMenuAction.GuestSelection -> "LAN photo selection"
                        LanShareMenuAction.ProjectSync -> "LAN project sync"
                    },
                )
                createdSession = session
                val router = LanShareRouter(
                    gateway = CoreLanShareGateway(coreGateway),
                    previewLoader = previewLoader@{ token, groupId, fullQuality ->
                        val asset = coreGateway.loadLanShareAssets(token)
                            .firstOrNull { candidate -> candidate.assetSelectionId() == groupId }
                            ?: return@previewLoader null
                        val quality = if (fullQuality) PreviewQuality.FullScreen else PreviewQuality.Thumbnail
                        val jpegQuality = if (fullQuality) 92 else 82
                        withContext(Dispatchers.IO) {
                            loadPreviewBitmap(context, asset.previewLocation, quality)
                                ?.toJpegBytes(jpegQuality)
                        }
                    },
                    discoveryInfo = LanShareDiscoveryInfo(
                        token = session.token,
                        projectName = activeProject?.name ?: "Android LAN Share",
                    ),
                    projectSnapshotLoader = {
                        loadProjectSyncSnapshotAssets(coreGateway)
                    },
                )
                val server = LanShareHttpServer(router)
                createdServer = server
                val port = runCatching { server.start(LAN_PROJECT_SYNC_DISCOVERY_PORT) }
                    .getOrElse { server.start(0) }
                Triple(session, server, port)
            }.onSuccess { (session, server, port) ->
                lanShareServer?.close()
                lanShareSession = session
                lanShareServer = server
                lanSharePort = port
                lanShareScopeSnapshot = shareScope
                showProjectFeedback(
                    when (lanShareDialogAction) {
                        LanShareMenuAction.GuestSelection -> "多方筛选已开启"
                        LanShareMenuAction.ProjectSync -> "局域网项目共享已开启"
                    },
                )
            }.onFailure { error ->
                createdServer?.close()
                createdSession?.let { session ->
                    runCatching { coreGateway.stopLanShareSession(session.shareId) }
                }
                lanShareError = error.message ?: "共享启动失败"
                showProjectFeedback("共享启动失败")
            }
            lanShareStarting = false
        }
    }
    LaunchedEffect(dashboard.receiver.running) {
        if (dashboard.receiver.running && receiverPanelExpanded) {
            onReceiverPanelExpandedChange(false)
        }
    }
    LaunchedEffect(projectState.activeProjectId, assetQuery, selectedPhotoCollection) {
        selectedAssetIds = emptyList()
        selectedBurstPreview = null
    }
    LaunchedEffect(filteredAssets, selectedPhoto?.assetSelectionId()) {
        val currentPhoto = selectedPhoto
        val refreshedPhoto = refreshedSelectedPhoto(currentPhoto, filteredAssets)
        if (refreshedPhoto != currentPhoto) {
            selectedPhoto = refreshedPhoto
        }
    }
    selectedPhoto?.let { photo ->
        ProjectPhotoDetailOverlay(
            photo = photo,
            dashboardAssets = dashboard.assets,
            visibleAssets = filteredAssets,
            activeProjectId = projectState.activeProjectId,
            assetQuery = assetQuery,
            coreGateway = coreGateway,
            context = context,
            coroutineScope = coroutineScope,
            actionsEnabled = actionsEnabled,
            feedbackMessage = projectFeedbackMessage,
            onSelectPhoto = { selectedPhoto = it },
            onShowProjectFeedback = ::showProjectFeedback,
            onSplitBurstMember = onSplitBurstMember,
            onSetAssetGroupUserMarks = onSetAssetGroupUserMarks,
            modifier = modifier,
        )
        return
    }
    BackHandler(enabled = selectionMode) {
        selectedAssetIds = emptyList()
    }
    DeleteSelectedProjectAssetsDialog(
        deleteSelectionCandidates = deleteSelectionCandidates,
        deleteSelectionInFlight = deleteSelectionInFlight,
        actionsEnabled = actionsEnabled,
        sourceProjectId = sourceProjectId,
        assetQuery = assetQuery,
        coreGateway = coreGateway,
        coroutineScope = coroutineScope,
        onDeleteSelectionInFlightChange = { deleteSelectionInFlight = it },
        onClearDeleteCandidates = { deleteSelectionCandidates = emptyList() },
        onClearSelection = { selectedAssetIds = emptyList() },
        onShowProjectFeedback = ::showProjectFeedback,
    )
    if (lanShareConfigVisible) {
        LanShareModeDialog(
            mode = lanShareDialogAction,
            scope = lanShareScopeSnapshot ?: currentLanShareScope,
            action = configuredLanShareAction,
            sharingActive = lanShareActive,
            activeUrl = lanShareUrl.takeIf { lanShareDialogAction == LanShareMenuAction.GuestSelection },
            error = lanShareError,
            editable = !lanShareActive && lanShareDialogAction == LanShareMenuAction.GuestSelection,
            onFavoriteOnlyChange = { lanShareFavoriteOnly = it },
            onMarkedOnlyChange = { lanShareMarkedOnly = it },
            onMinModelScoreChange = { lanShareMinModelScore = it },
            onDismiss = { lanShareConfigVisible = false },
            onStart = ::startLanShare,
            onStop = { stopLanShare() },
            onCopyLink = { url ->
                clipboardManager.setText(AnnotatedString(url))
                showProjectFeedback("\u591a\u65b9\u7b5b\u9009\u94fe\u63a5\u5df2\u590d\u5236")
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
                lanShareAction = lanShareAction,
                lanShareUrl = lanShareUrl,
                onOpenProjects = onOpenProjects,
                onExpand = { onReceiverPanelExpandedChange(true) },
                onOpenProjectIntelligence = onOpenProjectIntelligence,
                onConfigureGuestSelection = { showLanShareDialog(LanShareMenuAction.GuestSelection) },
                onConfigureProjectSync = { showLanShareDialog(LanShareMenuAction.ProjectSync) },
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(10.dp))
        }
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
                Spacer(Modifier.height(8.dp))
                GuestMarkFilterBar(
                    selectedFilter = selectedGuestMarkFilter,
                    onFilterChange = { selectedGuestMarkFilter = it },
                )
                Spacer(Modifier.height(8.dp))
                ModelScoreThresholdBar(
                    selectedScore = selectedMinModelScore,
                    onScoreChange = { selectedMinModelScore = it },
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
                            canSplitOut = actionsEnabled &&
                                selectedBurstSplitTargets.isNotEmpty(),
                            canMerge = actionsEnabled && selectedBurstMergeTarget != null,
                            canDelete = actionsEnabled &&
                                sourceProjectId != null &&
                                selectedDeleteAssets.isNotEmpty() &&
                                !deleteSelectionInFlight,
                            onOpen = {
                                selectedGridItems.firstOrNull()?.let { item ->
                                    selectedAssetIds = emptyList()
                                    if (item.isBurstGroup) {
                                        selectedBurstPreview = item
                                    } else {
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
                            onSplitOut = {
                                if (selectedBurstSplitTargets.isNotEmpty()) {
                                    onSplitBurstMembers(selectedBurstSplitTargets)
                                    selectedAssetIds = emptyList()
                                }
                            },
                            onDelete = {
                                deleteSelectionCandidates = selectedDeleteAssets
                            },
                            onMerge = {
                                val projectId = sourceProjectId
                                selectedBurstMergeTarget?.let { target ->
                                    if (projectId != null) {
                                        onCreateManualBurstGroup(projectId, target.memberGroupIds)
                                    }
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
                    lanShareAction = lanShareAction,
                    lanShareUrl = lanShareUrl,
                    onOpenProjects = onOpenProjects,
                    onOpenProjectIntelligence = onOpenProjectIntelligence,
                    onConfigureGuestSelection = { showLanShareDialog(LanShareMenuAction.GuestSelection) },
                    onConfigureProjectSync = { showLanShareDialog(LanShareMenuAction.ProjectSync) },
                    onConfigureAccount = onConfigureAccount,
                    onRequestNotificationPermission = onRequestNotificationPermission,
                    onStartReceiver = onStartReceiver,
                    onStopReceiver = onStopReceiver,
                    onRetryFailedPublishes = onRetryFailedPublishes,
                    onCollapse = { onReceiverPanelExpandedChange(false) },
                    endpointCandidates = localReceiverEndpointCandidates,
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

package com.cameraconnector.app.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.graphics.Bitmap
import androidx.activity.compose.BackHandler
import androidx.compose.animation.animateContentSize
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
import androidx.compose.material.icons.outlined.Home
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
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.PublishQueueState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState
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
    gridColumnCount: Int,
    modifier: Modifier = Modifier,
) {
    var selectedAccount by remember { mutableStateOf<String?>(null) }
    var selectedFilter by remember { mutableStateOf(InboxFilter.All) }
    var selectedPhoto by remember { mutableStateOf<InboxAsset?>(null) }
    var selectedAssetIds by rememberSaveable { mutableStateOf(emptyList<String>()) }
    var movePickerOpen by remember { mutableStateOf(false) }
    var filterExpanded by remember { mutableStateOf(false) }
    val inboxQuery = remember(selectedAccount, selectedFilter) {
        InboxAssetQuery(
            username = selectedAccount,
            role = selectedFilter.assetRole(),
        )
    }
    val filteredAssets by produceState<List<InboxAsset>>(
        initialValue = dashboard.inbox,
        projectState.activeProjectId,
        inboxQuery,
        dashboard.inbox,
    ) {
        value = withContext(Dispatchers.IO) {
            coreGateway.loadInbox(inboxQuery)
        }
    }
    val selectionMode = isAssetSelectionMode(selectedAssetIds)
    val selectedAssets = remember(filteredAssets, selectedAssetIds) {
        selectedAssetsFromIds(filteredAssets, selectedAssetIds)
    }
    val sourceProjectId = projectState.activeProjectId
    val moveTargets = remember(projectState.projects, sourceProjectId) {
        projectState.groupMoveTargets(sourceProjectId)
    }
    var receiverPanelExpanded by remember { mutableStateOf(!dashboard.receiver.running) }
    val receiverConnectHost = normalizeCameraConnectHost(cameraConnectHost)

    LaunchedEffect(dashboard.receiver.running) {
        receiverPanelExpanded = !dashboard.receiver.running
    }

    LaunchedEffect(projectState.activeProjectId, inboxQuery) {
        selectedAssetIds = emptyList()
        movePickerOpen = false
    }

    selectedPhoto?.let { photo ->
        BackHandler {
            selectedPhoto = null
        }
        PhotoDetailScreen(
            asset = photo,
            onBack = { selectedPhoto = null },
            modifier = modifier,
        )
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

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(16.dp)
            .animateContentSize(),
    ) {
        actionError?.let { message ->
            ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError)
            Spacer(Modifier.height(10.dp))
        }
        actionInFlight?.let { action ->
            ProcessingCard(action)
            Spacer(Modifier.height(10.dp))
        }

        if (receiverPanelExpanded) {
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
        } else {
            ProjectReceiverStatusStrip(
                dashboard = dashboard,
                projectState = projectState,
                onExpand = { receiverPanelExpanded = true },
                connectHost = receiverConnectHost,
                modifier = Modifier.fillMaxWidth(),
            )
        }

        Spacer(Modifier.height(10.dp))
        if (dashboard.receiver.running) {
            FilterToggleRow(
                selectedAccount = selectedAccount,
                selectedFilter = selectedFilter,
                expanded = filterExpanded,
                onToggle = { filterExpanded = !filterExpanded },
            )
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
            }
            Spacer(Modifier.height(10.dp))
            if (filteredAssets.isEmpty()) {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        if (dashboard.inbox.isEmpty()) "还没有导入文件。" else "当前筛选下没有文件。",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            } else {
                Box(modifier = Modifier.fillMaxSize()) {
                    LazyVerticalGrid(
                        columns = GridCells.Fixed(gridColumnCount),
                        modifier = Modifier.fillMaxSize(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(10.dp),
                        contentPadding = PaddingValues(bottom = if (selectionMode) 104.dp else 8.dp),
                    ) {
                        items(
                            count = filteredAssets.size,
                            key = { index -> filteredAssets[index].assetSelectionId() },
                        ) { index ->
                            val asset = filteredAssets[index]
                            val selected = asset.assetSelectionId() in selectedAssetIds
                            CompactPhotoTile(
                                asset = asset,
                                selected = selected,
                                selectionMode = selectionMode,
                                onClick = {
                                    if (selectionMode) {
                                        selectedAssetIds = toggleAssetSelection(selectedAssetIds, asset)
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
                            onOpen = {
                                selectedAssets.firstOrNull()?.let { asset ->
                                    selectedAssetIds = emptyList()
                                    selectedPhoto = asset
                                }
                            },
                            onMove = { movePickerOpen = true },
                            onCancel = { selectedAssetIds = emptyList() },
                            modifier = Modifier
                                .align(Alignment.BottomCenter)
                                .fillMaxWidth(),
                        )
                    }
                }
            }
        } else {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Text(
                    "启动接收后，照片分组会自动出现在这里。",
                    modifier = Modifier.padding(16.dp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
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
            Text("控制", color = ElementBlue, fontWeight = FontWeight.SemiBold)
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

    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(10.dp)) {
        ProjectScopeCard(
            projectState = projectState,
            actionsEnabled = actionsEnabled,
            onOpenProjects = onOpenProjects,
            modifier = Modifier.fillMaxWidth(),
        )
        ElementCard(modifier = Modifier.fillMaxWidth()) {
            Column(Modifier.padding(16.dp)) {
                ReceiverHeroControl(
                    running = dashboard.receiver.running,
                    phase = dashboard.receiver.phase,
                    endpoint = receiverEndpointLabel(dashboard.receiver, cleanConnectHost),
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
                Spacer(Modifier.height(4.dp))
                Text("接收配置", style = MaterialTheme.typography.titleMedium)
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
                Spacer(Modifier.height(12.dp))
                Text(
                    "相机连接：${receiverEndpointLabel(dashboard.receiver, cleanConnectHost)}",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
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
                    Spacer(Modifier.height(10.dp))
                    OutlinedButton(
                        onClick = onCollapse,
                        modifier = Modifier.fillMaxWidth(),
                        shape = elementShape,
                        border = BorderStroke(1.dp, ElementBorder),
                    ) {
                        Text("收起到顶部状态")
                    }
                }
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
    onOpen: () -> Unit,
    onMove: () -> Unit,
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
                    onClick = onCancel,
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
internal fun FilterToggleRow(
    selectedAccount: String?,
    selectedFilter: InboxFilter,
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


@OptIn(ExperimentalFoundationApi::class)
@Composable
internal fun CompactPhotoTile(
    asset: InboxAsset,
    selected: Boolean,
    selectionMode: Boolean,
    onClick: () -> Unit,
    onLongClick: () -> Unit,
) {
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
        PhotoPreview(
            asset = asset,
            compactFallback = true,
            backgroundColor = asset.previewAccentColor().copy(alpha = 0.16f),
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(1.22f),
        )
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
    }
}

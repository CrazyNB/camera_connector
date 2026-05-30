package com.cameraconnector.app.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Shapes
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.DEFAULT_LISTEN_HOST
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.core.StrategyProfileUi
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch

@Composable
fun CameraConnectorApp(
    coreGateway: CoreGateway,
    storageGateway: AndroidStorageGateway,
    notificationPermissionRequired: Boolean,
    notificationPermissionGranted: Flow<Boolean>,
    selectedInboxLabel: Flow<String?>,
    onRequestNotificationPermission: () -> Unit,
    onChooseInboxDirectory: () -> Unit,
) {
    val dashboard by coreGateway.observeDashboard().collectAsState(
        initial = DashboardState(
            receiver = ReceiverState(
                running = false,
                phase = "Unknown",
                protocol = "FTP",
                authMode = "Unknown",
                accountCount = 0,
                host = DEFAULT_LISTEN_HOST,
                port = 2121,
                outputLabel = "应用私有目录",
                message = null,
            ),
            accounts = emptyList(),
            inbox = emptyList(),
            transfers = emptyList(),
        ),
    )
    val projectState by coreGateway.observeProjects().collectAsState(
        initial = ProjectState(projects = emptyList(), activeProjectId = null),
    )
    val notificationsGranted by notificationPermissionGranted.collectAsState(initial = true)
    val selectedInbox by selectedInboxLabel.collectAsState(initial = null)
    val scope = rememberCoroutineScope()
    var destination by remember { mutableStateOf(GlobalDestination.Projects) }
    var projectWorkspaceOpen by remember { mutableStateOf(false) }
    var projectImmersiveMode by remember { mutableStateOf(false) }
    var settingsDiagnosticsOpen by remember { mutableStateOf(false) }
    var accountDetail by remember { mutableStateOf<DeviceAccount?>(null) }
    var addingAccount by remember { mutableStateOf(false) }
    var actionError by remember { mutableStateOf<String?>(null) }
    var actionInFlight by remember { mutableStateOf<String?>(null) }
    var projectPhotoGridColumnCount by rememberSaveable {
        mutableStateOf(storageGateway.projectPhotoGridColumnCount())
    }
    var cameraConnectHost by rememberSaveable {
        mutableStateOf(normalizeCameraConnectHost(storageGateway.cameraConnectHost()))
    }
    var strategyProfiles by remember { mutableStateOf<List<StrategyProfileUi>>(emptyList()) }
    var selectedStrategyProfileId by rememberSaveable {
        mutableStateOf(storageGateway.smartSelectionStrategyProfileId())
    }

    fun saveCameraConnectHost(host: String) {
        val normalized = normalizeCameraConnectHost(host)
        cameraConnectHost = normalized
        storageGateway.persistCameraConnectHost(normalized)
    }

    fun runAction(actionName: String, action: suspend () -> Unit) {
        scope.launch {
            actionInFlight = actionName
            try {
                action()
                actionError = null
            } catch (error: Throwable) {
                actionError = error.message ?: error::class.java.simpleName
            } finally {
                actionInFlight = null
            }
        }
    }

    fun openProject(projectId: String) {
        destination = GlobalDestination.Projects
        projectWorkspaceOpen = true
        settingsDiagnosticsOpen = false
        accountDetail = null
        addingAccount = false
        runAction("正在进入项目") {
            coreGateway.setActiveProject(projectId)
        }
    }

    LaunchedEffect(coreGateway) {
        runCatching {
            coreGateway.loadStrategyProfiles()
        }.onSuccess { profiles ->
            strategyProfiles = profiles
            val selected = selectedStrategyProfile(profiles, selectedStrategyProfileId)
            if (selected != null && selected.profileId != selectedStrategyProfileId) {
                selectedStrategyProfileId = selected.profileId
                storageGateway.persistSmartSelectionStrategyProfileId(selected.profileId)
            }
        }.onFailure { error ->
            actionError = error.message ?: error::class.java.simpleName
        }
    }

    BackHandler(enabled = destination == GlobalDestination.Projects && projectWorkspaceOpen) {
        projectWorkspaceOpen = false
        projectImmersiveMode = false
    }
    BackHandler(enabled = destination == GlobalDestination.Settings && settingsDiagnosticsOpen) {
        settingsDiagnosticsOpen = false
    }

    MaterialTheme(
        colorScheme = elementColorScheme,
        shapes = Shapes(small = elementShape, medium = elementShape, large = elementShape),
    ) {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.background,
        ) {
            Scaffold(
                containerColor = MaterialTheme.colorScheme.background,
                bottomBar = {
                    if (!projectImmersiveMode) {
                        NavigationBar(containerColor = MaterialTheme.colorScheme.surface) {
                            GlobalDestination.entries.forEach { item ->
                                NavigationBarItem(
                                    selected = destination == item,
                                    onClick = {
                                        destination = item
                                        projectWorkspaceOpen = false
                                        projectImmersiveMode = false
                                        settingsDiagnosticsOpen = false
                                        accountDetail = null
                                        addingAccount = false
                                    },
                                    label = { Text(item.label) },
                                    icon = { Icon(item.icon(), contentDescription = item.label) },
                                    colors = NavigationBarItemDefaults.colors(
                                        selectedIconColor = MaterialTheme.colorScheme.primary,
                                        selectedTextColor = MaterialTheme.colorScheme.primary,
                                        indicatorColor = ElementBlueSoft,
                                    ),
                                )
                            }
                        }
                    }
                },
            ) { padding ->
                when (destination) {
                    GlobalDestination.Projects -> if (!projectWorkspaceOpen) {
                        ProjectManagementScreen(
                            dashboard = dashboard,
                            projectState = projectState,
                            cameraConnectHost = cameraConnectHost,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onEnterProject = ::openProject,
                            onCreateAndEnterProject = { name ->
                                projectWorkspaceOpen = true
                                runAction("正在创建项目") {
                                    coreGateway.createProject(name)
                                }
                            },
                            modifier = Modifier.padding(padding),
                        )
                    } else {
                        ProjectAssetsScreen(
                            coreGateway = coreGateway,
                            dashboard = dashboard,
                            projectState = projectState,
                            notificationPermissionGranted = notificationsGranted,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onOpenProjects = {
                                projectWorkspaceOpen = false
                                projectImmersiveMode = false
                            },
                            onConfigureAccount = {
                                destination = GlobalDestination.Accounts
                                projectWorkspaceOpen = false
                                settingsDiagnosticsOpen = false
                                accountDetail = null
                                addingAccount = true
                            },
                            onRequestNotificationPermission = onRequestNotificationPermission,
                            actionsEnabled = actionInFlight == null,
                            onStartReceiver = { settings, cameraHost ->
                                saveCameraConnectHost(cameraHost)
                                runAction("正在启动接收服务") {
                                    coreGateway.saveReceiverSettings(settings)
                                    coreGateway.startReceiver()
                                }
                            },
                            onStopReceiver = {
                                runAction("正在停止接收服务") {
                                    coreGateway.stopReceiver()
                                }
                            },
                            cameraConnectHost = cameraConnectHost,
                            onRetryFailedPublishes = {
                                runAction("正在重试发布") {
                                    coreGateway.retryFailedPublishes()
                                }
                            },
                            onMoveProjectGroup = { sourceProjectId, groupId, targetProjectId ->
                                runAction("正在移动文件组") {
                                    coreGateway.moveProjectGroup(sourceProjectId, groupId, targetProjectId)
                                }
                            },
                            onAcceptRecommendedBest = { burstGroupId ->
                                runAction("正在接受推荐") {
                                    coreGateway.acceptRecommendedBest(burstGroupId, selectedStrategyProfileId)
                                }
                            },
                            onOverrideRecommendedBest = { burstGroupId, bestAssetGroupId ->
                                runAction("正在设为最佳") {
                                    coreGateway.overrideRecommendedBest(
                                        burstGroupId,
                                        bestAssetGroupId,
                                        selectedStrategyProfileId,
                                    )
                                }
                            },
                            onMarkBurstNeedsReview = { burstGroupId ->
                                runAction("正在标记复核") {
                                    coreGateway.markBurstNeedsReview(burstGroupId, selectedStrategyProfileId)
                                }
                            },
                            onRestoreAutomaticRecommendation = { burstGroupId ->
                                runAction("正在撤销上一步") {
                                    coreGateway.restoreAutomaticRecommendation(burstGroupId, selectedStrategyProfileId)
                                }
                            },
                            onClearRecommendation = { burstGroupId ->
                                runAction("正在清除推荐") {
                                    coreGateway.clearRecommendation(burstGroupId, selectedStrategyProfileId)
                                }
                            },
                            onKeepAllCandidates = { burstGroupId ->
                                runAction("正在保留全部") {
                                    coreGateway.keepAllCandidates(burstGroupId, selectedStrategyProfileId)
                                }
                            },
                            onHideLowScoreCandidates = { burstGroupId ->
                                runAction("正在隐藏低分") {
                                    coreGateway.hideLowScoreCandidates(burstGroupId, selectedStrategyProfileId)
                                }
                            },
                            onSplitBurstMember = { burstGroupId, memberGroupId ->
                                runAction("\u6b63\u5728\u79fb\u51fa\u8fde\u62cd\u7ec4") {
                                    coreGateway.splitBurstMember(burstGroupId, memberGroupId)
                                }
                            },
                            onMergeBurstMember = { targetBurstGroupId, memberGroupId ->
                                runAction("\u6b63\u5728\u5408\u5e76\u8fde\u62cd\u7ec4") {
                                    coreGateway.mergeBurstMember(targetBurstGroupId, memberGroupId)
                                }
                            },
                            gridColumnCount = projectPhotoGridColumnCount,
                            selectedStrategyProfileId = selectedStrategyProfileId,
                            onImmersiveModeChange = { projectImmersiveMode = it },
                            modifier = Modifier.padding(padding),
                        )
                    }

                    GlobalDestination.Accounts -> {
                        val selectedAccount = accountDetail
                        if (selectedAccount != null || addingAccount) {
                            AccountDetailScreen(
                                account = selectedAccount,
                                actionError = actionError,
                                actionInFlight = actionInFlight,
                                onClearActionError = { actionError = null },
                                onBack = {
                                    accountDetail = null
                                    addingAccount = false
                                },
                                onSaveDeviceAccount = { account, password ->
                                    runAction("正在保存账号") {
                                        coreGateway.saveDeviceAccount(account, password)
                                    }
                                },
                                onDeleteDeviceAccount = { username ->
                                    runAction("正在删除账号") {
                                        coreGateway.removeDeviceAccount(username)
                                    }
                                    accountDetail = null
                                    addingAccount = false
                                },
                                modifier = Modifier.padding(padding),
                            )
                        } else {
                            AccountsScreen(
                                dashboard = dashboard,
                                actionError = actionError,
                                actionInFlight = actionInFlight,
                                onClearActionError = { actionError = null },
                                onOpenAccount = {
                                    accountDetail = it
                                    addingAccount = false
                                },
                                onAddAccount = {
                                    accountDetail = null
                                    addingAccount = true
                                },
                                modifier = Modifier.padding(padding),
                            )
                        }
                    }

                    GlobalDestination.Settings -> if (settingsDiagnosticsOpen) {
                        DiagnosticsScreen(
                            dashboard = dashboard,
                            onBack = { settingsDiagnosticsOpen = false },
                            modifier = Modifier.padding(padding),
                        )
                    } else {
                        SettingsScreen(
                            dashboard = dashboard,
                            notificationPermissionRequired = notificationPermissionRequired,
                            notificationPermissionGranted = notificationsGranted,
                            onRequestNotificationPermission = onRequestNotificationPermission,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            selectedInboxLabel = selectedInbox,
                            onChooseInboxDirectory = onChooseInboxDirectory,
                            onOpenDiagnostics = { settingsDiagnosticsOpen = true },
                            projectPhotoGridColumnCount = projectPhotoGridColumnCount,
                            onProjectPhotoGridColumnCountChange = { count ->
                                projectPhotoGridColumnCount = count
                                storageGateway.persistProjectPhotoGridColumnCount(count)
                            },
                            strategyProfiles = strategyProfiles,
                            selectedStrategyProfileId = selectedStrategyProfileId,
                            onSelectedStrategyProfileChange = { profileId ->
                                selectedStrategyProfileId = profileId
                                storageGateway.persistSmartSelectionStrategyProfileId(profileId)
                            },
                            onSaveStrategyProfile = { profile ->
                                runAction("正在保存优选策略") {
                                    val saved = coreGateway.saveStrategyProfile(profile)
                                    val loadedProfiles = coreGateway.loadStrategyProfiles()
                                    strategyProfiles = if (loadedProfiles.any { it.profileId == saved.profileId }) {
                                        loadedProfiles
                                    } else {
                                        loadedProfiles + saved
                                    }
                                    selectedStrategyProfileId = saved.profileId
                                    storageGateway.persistSmartSelectionStrategyProfileId(saved.profileId)
                                }
                            },
                            modifier = Modifier.padding(padding),
                        )
                    }
                }
            }
        }
    }
}

private fun GlobalDestination.icon(): ImageVector = when (this) {
    GlobalDestination.Projects -> Icons.Outlined.Home
    GlobalDestination.Accounts -> Icons.Outlined.Person
    GlobalDestination.Settings -> Icons.Outlined.Settings
}

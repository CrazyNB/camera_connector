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
import com.cameraconnector.app.core.EvaluationRunUi
import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PromptProfileUi
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.core.StrategyProfileUi
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.CancellationException
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
    var projectConfigId by rememberSaveable { mutableStateOf<String?>(null) }
    var settingsDiagnosticsOpen by remember { mutableStateOf(false) }
    var settingsPromptProfilesOpen by remember { mutableStateOf(false) }
    var editingPromptProfileId by rememberSaveable { mutableStateOf<String?>(null) }
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
    var modelProviderSettings by remember { mutableStateOf(ModelProviderSettingsUi()) }
    var globalPromptProfiles by remember { mutableStateOf<List<PromptProfileUi>>(emptyList()) }
    var projectConfigEvaluationSettings by remember { mutableStateOf<ProjectEvaluationSettingsUi?>(null) }
    var projectConfigPromptProfiles by remember { mutableStateOf<List<PromptProfileUi>>(emptyList()) }
    var projectConfigLatestRecommendationRun by remember { mutableStateOf<EvaluationRunUi?>(null) }

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
            } catch (error: CancellationException) {
                throw error
            } catch (error: Throwable) {
                if (!error.isUiCancellationNoise()) {
                    actionError = error.message ?: error::class.java.simpleName
                }
            } finally {
                actionInFlight = null
            }
        }
    }

    fun runLightAction(action: suspend () -> Unit) {
        scope.launch {
            try {
                action()
                actionError = null
            } catch (error: CancellationException) {
                throw error
            } catch (error: Throwable) {
                if (!error.isUiCancellationNoise()) {
                    actionError = error.message ?: error::class.java.simpleName
                }
            }
        }
    }

    fun openProject(projectId: String) {
        destination = GlobalDestination.Projects
        projectWorkspaceOpen = true
        projectConfigId = null
        settingsDiagnosticsOpen = false
        settingsPromptProfilesOpen = false
        editingPromptProfileId = null
        accountDetail = null
        addingAccount = false
        runAction("正在进入项目") {
            coreGateway.setActiveProject(projectId)
        }
    }

    fun openProjectConfig(projectId: String) {
        destination = GlobalDestination.Projects
        projectWorkspaceOpen = false
        projectConfigId = projectId
        settingsDiagnosticsOpen = false
        settingsPromptProfilesOpen = false
        editingPromptProfileId = null
        accountDetail = null
        addingAccount = false
    }

    LaunchedEffect(coreGateway) {
        runCatching {
            Triple(
                coreGateway.loadModelProviderSettings(),
                coreGateway.loadStrategyProfiles(),
                coreGateway.loadGlobalPromptProfiles(),
            )
        }.onSuccess { (providerSettings, profiles, prompts) ->
            modelProviderSettings = providerSettings
            strategyProfiles = profiles
            globalPromptProfiles = prompts
            val selected = selectedStrategyProfile(strategyProfiles, selectedStrategyProfileId)
            if (selected != null && selected.profileId != selectedStrategyProfileId) {
                selectedStrategyProfileId = selected.profileId
                storageGateway.persistSmartSelectionStrategyProfileId(selected.profileId)
            }
        }.onFailure { error ->
            if (!error.isUiCancellationNoise()) {
                actionError = error.message ?: error::class.java.simpleName
            }
        }
    }

    LaunchedEffect(coreGateway, projectConfigId) {
        val projectId = projectConfigId
        projectConfigLatestRecommendationRun = null
        if (projectId.isNullOrBlank()) {
            projectConfigEvaluationSettings = null
            projectConfigPromptProfiles = emptyList()
            return@LaunchedEffect
        }

        runCatching {
            Triple(
                coreGateway.loadProjectEvaluationSettings(projectId),
                coreGateway.loadPromptProfiles(projectId),
                coreGateway.latestProjectRecommendationRunStatus(projectId),
            )
        }.onSuccess { (settings, profiles, latestRun) ->
            projectConfigEvaluationSettings = settings
            projectConfigPromptProfiles = profiles
            projectConfigLatestRecommendationRun = scopedProjectRecommendationRun(latestRun, projectId)
        }.onFailure { error ->
            if (!error.isUiCancellationNoise()) {
                actionError = error.message ?: error::class.java.simpleName
            }
        }
    }

    BackHandler(enabled = destination == GlobalDestination.Projects && projectConfigId != null) {
        projectConfigId = null
    }
    BackHandler(enabled = destination == GlobalDestination.Projects && projectWorkspaceOpen) {
        projectWorkspaceOpen = false
    }
    BackHandler(enabled = destination == GlobalDestination.Settings && settingsDiagnosticsOpen) {
        settingsDiagnosticsOpen = false
    }
    BackHandler(enabled = destination == GlobalDestination.Settings && settingsPromptProfilesOpen) {
        settingsPromptProfilesOpen = false
    }
    BackHandler(enabled = destination == GlobalDestination.Settings && editingPromptProfileId != null) {
        editingPromptProfileId = null
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
                    NavigationBar(containerColor = MaterialTheme.colorScheme.surface) {
                        GlobalDestination.entries.forEach { item ->
                            NavigationBarItem(
                                selected = destination == item,
                                onClick = {
                                    destination = item
                                    projectWorkspaceOpen = false
                                    projectConfigId = null
                                    settingsDiagnosticsOpen = false
                                    settingsPromptProfilesOpen = false
                                    editingPromptProfileId = null
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
                },
            ) { padding ->
                when (destination) {
                    GlobalDestination.Projects -> if (projectConfigId != null) {
                        val targetProjectId = projectConfigId
                        ProjectSettingsScreen(
                            project = targetProjectId?.let { id -> projectState.projects.firstOrNull { it.id == id } },
                            provider = modelProviderSettings,
                            settings = projectConfigEvaluationSettings,
                            promptProfiles = globalPromptProfiles.ifEmpty { projectConfigPromptProfiles },
                            latestRun = projectConfigLatestRecommendationRun,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onBack = { projectConfigId = null },
                            onSaveSettings = { settings ->
                                runAction("正在保存项目配置") {
                                    val saved = coreGateway.saveProjectEvaluationSettings(settings)
                                    if (projectConfigId == saved.projectId) {
                                        projectConfigEvaluationSettings = saved
                                    }
                                }
                            },
                            onGenerateProjectRecommendation = {
                                val projectId = projectConfigId
                                if (!projectId.isNullOrBlank()) {
                                    runAction("正在生成项目优选") {
                                        val run = coreGateway.generateProjectRecommendation(projectId)
                                        if (projectConfigId == projectId) {
                                            projectConfigLatestRecommendationRun = run
                                        }
                                    }
                                }
                            },
                            onConfigureModelProvider = {
                                destination = GlobalDestination.Settings
                                projectConfigId = null
                                projectWorkspaceOpen = false
                                settingsDiagnosticsOpen = false
                            },
                            modifier = Modifier.padding(padding),
                        )
                    } else if (!projectWorkspaceOpen) {
                        ProjectManagementScreen(
                            dashboard = dashboard,
                            projectState = projectState,
                            cameraConnectHost = cameraConnectHost,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onEnterProject = ::openProject,
                            onConfigureProject = ::openProjectConfig,
                            onCreateAndEnterProject = { name ->
                                projectWorkspaceOpen = true
                                projectConfigId = null
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
                            onSetAssetGroupUserMarks = { projectId, groupId, favorite, marked ->
                                runLightAction {
                                    coreGateway.setAssetGroupUserMarks(
                                        projectId = projectId,
                                        groupId = groupId,
                                        favorite = favorite,
                                        marked = marked,
                                    )
                                }
                            },
                            gridColumnCount = projectPhotoGridColumnCount,
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
                    } else if (editingPromptProfileId != null) {
                        val editingProfile = globalPromptProfiles.firstOrNull {
                            it.promptProfileId == editingPromptProfileId
                        }
                        PromptProfileEditorScreen(
                            profile = editingProfile,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onBack = { editingPromptProfileId = null },
                            onSave = { profile, name, promptText ->
                                runAction(if (profile.builtIn) "正在复制提示词" else "正在保存提示词") {
                                    val saved = if (profile.builtIn) {
                                        val forked = coreGateway.forkGlobalPromptProfile(
                                            profile.promptProfileId,
                                            name,
                                        )
                                        coreGateway.saveGlobalPromptProfileVersion(
                                            forked.promptProfileId,
                                            promptText,
                                        )
                                    } else {
                                        coreGateway.saveGlobalPromptProfileVersion(
                                            profile.promptProfileId,
                                            promptText,
                                        )
                                    }
                                    val loaded = coreGateway.loadGlobalPromptProfiles()
                                    globalPromptProfiles = if (loaded.any { it.promptProfileId == saved.promptProfileId }) {
                                        loaded
                                    } else {
                                        loaded + saved
                                    }
                                    editingPromptProfileId = saved.promptProfileId
                                }
                            },
                            modifier = Modifier.padding(padding),
                        )
                    } else if (settingsPromptProfilesOpen) {
                        PromptProfilesScreen(
                            promptProfiles = globalPromptProfiles,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onBack = { settingsPromptProfilesOpen = false },
                            onOpenPromptProfile = { promptId -> editingPromptProfileId = promptId },
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
                            onOpenPromptProfiles = { settingsPromptProfilesOpen = true },
                            projectPhotoGridColumnCount = projectPhotoGridColumnCount,
                            onProjectPhotoGridColumnCountChange = { count ->
                                projectPhotoGridColumnCount = count
                                storageGateway.persistProjectPhotoGridColumnCount(count)
                            },
                            modelProviderSettings = modelProviderSettings,
                            onSaveModelProviderSettings = { settings ->
                                runAction("正在保存模型服务设置") {
                                    val saved = coreGateway.saveModelProviderSettings(settings)
                                    modelProviderSettings = saved
                                    storageGateway.persistModelProviderConfigured(
                                        configured = saved.configured,
                                        keyAlias = saved.keyAlias,
                                    )
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

private fun Throwable.isUiCancellationNoise(): Boolean =
    this is CancellationException ||
        message?.contains("coroutine scope left the composition", ignoreCase = true) == true

private fun GlobalDestination.icon(): ImageVector = when (this) {
    GlobalDestination.Projects -> Icons.Outlined.Home
    GlobalDestination.Accounts -> Icons.Outlined.Person
    GlobalDestination.Settings -> Icons.Outlined.Settings
}

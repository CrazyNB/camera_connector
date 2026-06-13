package com.cameraconnector.app.ui

import android.content.Context
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Shapes
import androidx.compose.material3.Surface
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.DEFAULT_LISTEN_HOST
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PromptPackUi
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.core.SelectionCandidateVisualInput
import com.cameraconnector.app.media.loadPreviewSampleJson
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject

@Composable
fun CameraConnectorApp(
    coreGateway: CoreGateway,
    storageGateway: AndroidStorageGateway,
    notificationPermissionRequired: Boolean,
    notificationPermissionGranted: Flow<Boolean>,
    selectedOutputLabel: Flow<String?>,
    onRequestNotificationPermission: () -> Unit,
    onChooseOutputDirectory: () -> Unit,
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
            assets = emptyList(),
            transfers = emptyList(),
        ),
    )
    val projectState by coreGateway.observeProjects().collectAsState(
        initial = ProjectState(projects = emptyList(), activeProjectId = null),
    )
    val notificationsGranted by notificationPermissionGranted.collectAsState(initial = true)
    val selectedOutput by selectedOutputLabel.collectAsState(initial = null)
    val scope = rememberCoroutineScope()
    var destination by remember { mutableStateOf(GlobalDestination.Projects) }
    var projectWorkspaceOpen by remember { mutableStateOf(false) }
    val receiverPanelExpandedByProject = remember { mutableStateMapOf<String, Boolean>() }
    var projectConfigId by rememberSaveable { mutableStateOf<String?>(null) }
    var projectConfigPanel by rememberSaveable { mutableStateOf<ProjectIntelligencePanel?>(null) }
    var settingsDiagnosticsOpen by remember { mutableStateOf(false) }
    var settingsModelProvidersOpen by remember { mutableStateOf(false) }
    var settingsModelProvidersReturnProjectId by rememberSaveable { mutableStateOf<String?>(null) }
    var settingsPromptPacksOpen by remember { mutableStateOf(false) }
    var editingPromptPackId by rememberSaveable { mutableStateOf<String?>(null) }
    var creatingPromptPack by rememberSaveable { mutableStateOf(false) }
    var creatingPromptPackPackage by rememberSaveable { mutableStateOf("user") }
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
    var modelProviderSettings by remember { mutableStateOf(ModelProviderSettingsUi()) }
    var modelProviderSettingsList by remember { mutableStateOf<List<ModelProviderSettingsUi>>(emptyList()) }
    var globalPromptPacks by remember { mutableStateOf<List<PromptPackUi>>(emptyList()) }
    var projectConfigEvaluationSettings by remember { mutableStateOf<ProjectEvaluationSettingsUi?>(null) }
    var projectConfigPromptPacks by remember { mutableStateOf<List<PromptPackUi>>(emptyList()) }
    val projectWorkspaceIsVisible = projectWorkspaceVisible(
        workspaceOpen = projectWorkspaceOpen,
        activeProjectId = projectState.activeProjectId,
    )
    val context = LocalContext.current

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
        projectConfigPanel = null
        settingsDiagnosticsOpen = false
        settingsModelProvidersOpen = false
        settingsModelProvidersReturnProjectId = null
        settingsPromptPacksOpen = false
        editingPromptPackId = null
        creatingPromptPack = false
        accountDetail = null
        addingAccount = false
        runAction("正在进入项目") {
            coreGateway.setActiveProject(projectId)
        }
    }

    fun openProjectConfig(projectId: String, keepProjectWorkspace: Boolean = false) {
        destination = GlobalDestination.Projects
        projectWorkspaceOpen = keepProjectWorkspace && projectWorkspaceOpen
        projectConfigId = projectId
        projectConfigPanel = null
        settingsDiagnosticsOpen = false
        settingsModelProvidersOpen = false
        settingsModelProvidersReturnProjectId = null
        settingsPromptPacksOpen = false
        editingPromptPackId = null
        creatingPromptPack = false
        accountDetail = null
        addingAccount = false
    }

    LaunchedEffect(coreGateway) {
        runCatching {
            Triple(
                coreGateway.loadModelProviderSettings(),
                coreGateway.loadModelProviderSettingsList(),
                coreGateway.loadGlobalPromptPacks(),
            )
        }.onSuccess { (providerSettings, providerSettingsList, prompts) ->
            modelProviderSettings = providerSettings
            modelProviderSettingsList = providerSettingsList
            globalPromptPacks = prompts
        }.onFailure { error ->
            if (!error.isUiCancellationNoise()) {
                actionError = error.message ?: error::class.java.simpleName
            }
        }
    }

    LaunchedEffect(coreGateway, projectConfigId) {
        val projectId = projectConfigId
        if (projectId.isNullOrBlank()) {
            projectConfigEvaluationSettings = null
            projectConfigPromptPacks = emptyList()
            return@LaunchedEffect
        }

        runCatching {
            coreGateway.loadProjectEvaluationSettings(projectId) to coreGateway.loadPromptPacks(projectId)
        }.onSuccess { (settings, profiles) ->
            projectConfigEvaluationSettings = settings
            projectConfigPromptPacks = profiles
        }.onFailure { error ->
            if (!error.isUiCancellationNoise()) {
                actionError = error.message ?: error::class.java.simpleName
            }
        }
    }

    fun closeModelProviderSettings() {
        val returnProjectId = settingsModelProvidersReturnProjectId
        settingsModelProvidersOpen = false
        settingsModelProvidersReturnProjectId = null
        if (!returnProjectId.isNullOrBlank()) {
            destination = GlobalDestination.Projects
            projectConfigId = returnProjectId
            projectConfigPanel = ProjectIntelligencePanel.Model
            settingsDiagnosticsOpen = false
            settingsPromptPacksOpen = false
            editingPromptPackId = null
            creatingPromptPack = false
        }
    }

    BackHandler(enabled = destination == GlobalDestination.Projects && projectConfigId != null) {
        projectConfigPanel = null
        projectConfigId = null
    }
    BackHandler(
        enabled = destination == GlobalDestination.Projects &&
            projectConfigId == null &&
            projectWorkspaceOpen,
    ) {
        projectWorkspaceOpen = false
    }
    BackHandler(enabled = destination == GlobalDestination.Settings && settingsDiagnosticsOpen) {
        settingsDiagnosticsOpen = false
    }
    BackHandler(enabled = destination == GlobalDestination.Settings && settingsModelProvidersOpen) {
        closeModelProviderSettings()
    }
    BackHandler(enabled = destination == GlobalDestination.Settings && settingsPromptPacksOpen) {
        settingsPromptPacksOpen = false
    }
    BackHandler(
        enabled = destination == GlobalDestination.Settings &&
            (editingPromptPackId != null || creatingPromptPack),
    ) {
        editingPromptPackId = null
        creatingPromptPack = false
    }

    MaterialTheme(
        colorScheme = elementColorScheme,
        shapes = Shapes(small = elementShape, medium = elementShape, large = elementShape),
    ) {
        fun selectGlobalDestination(item: GlobalDestination) {
            val collapseCurrentProjectWorkspace =
                item == GlobalDestination.Projects &&
                    destination == GlobalDestination.Projects &&
                    projectConfigId == null &&
                    projectWorkspaceIsVisible
            destination = item
            projectConfigId = null
            projectConfigPanel = null
            projectWorkspaceOpen = projectWorkspaceStateAfterBottomDestinationClick(
                current = ProjectWorkspaceNavigationState(projectWorkspaceOpen),
                destination = item,
                collapseCurrentProjectWorkspace = collapseCurrentProjectWorkspace,
            ).workspaceOpen
            settingsDiagnosticsOpen = false
            settingsModelProvidersOpen = false
            settingsModelProvidersReturnProjectId = null
            settingsPromptPacksOpen = false
            editingPromptPackId = null
            creatingPromptPack = false
            accountDetail = null
            addingAccount = false
        }

        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.background,
        ) {
            Scaffold(
                containerColor = MaterialTheme.colorScheme.background,
                bottomBar = {
                    CameraConnectorBottomBar(
                        selected = destination,
                        onSelect = ::selectGlobalDestination,
                    )
                },
            ) { padding ->
                when (destination) {
                    GlobalDestination.Projects -> if (projectConfigId != null) {
                        val targetProjectId = projectConfigId
                        ProjectSettingsScreen(
                            project = targetProjectId?.let { id -> projectState.projects.firstOrNull { it.id == id } },
                            providerOptions = modelProviderSettingsList.ifEmpty {
                                listOf(modelProviderSettings).filter { it.configured }
                            },
                            settings = projectConfigEvaluationSettings,
                            promptPacks = globalPromptPacks.ifEmpty { projectConfigPromptPacks },
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            selectedPanel = projectConfigPanel,
                            onSelectedPanelChange = { projectConfigPanel = it },
                            onClearActionError = { actionError = null },
                            onBack = {
                                projectConfigPanel = null
                                projectConfigId = null
                            },
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
                                    runAction("\u6b63\u5728\u751f\u6210\u9879\u76ee\u4f18\u9009") {
                                        val candidateVisuals = if (projectState.activeProjectId == projectId) {
                                            projectRecommendationCandidateVisuals(context, dashboard.assets)
                                        } else {
                                            emptyList()
                                        }
                                        if (candidateVisuals.isNotEmpty()) {
                                            coreGateway.generateProjectRecommendationWithCandidateVisuals(
                                                projectId,
                                                candidateVisuals,
                                            )
                                        } else {
                                            coreGateway.generateProjectRecommendation(projectId)
                                        }
                                        if (projectConfigId == projectId) {
                                            projectConfigEvaluationSettings = coreGateway.loadProjectEvaluationSettings(projectId)
                                        }
                                    }
                                }
                            },
                            onConfigureModelProvider = {
                                val sourceProjectId = projectConfigId
                                destination = GlobalDestination.Settings
                                projectConfigPanel = ProjectIntelligencePanel.Model
                                settingsModelProvidersReturnProjectId = sourceProjectId
                                settingsDiagnosticsOpen = false
                                settingsModelProvidersOpen = true
                                settingsPromptPacksOpen = false
                                editingPromptPackId = null
                                creatingPromptPack = false
                            },
                            modifier = Modifier.padding(padding),
                        )
                    } else if (!projectWorkspaceIsVisible) {
                        ProjectManagementScreen(
                            dashboard = dashboard,
                            projectState = projectState,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onEnterProject = ::openProject,
                            onConfigureProject = ::openProjectConfig,
                            onDeleteProject = { projectId ->
                                if (projectState.activeProjectId == projectId) {
                                    projectWorkspaceOpen = false
                                    projectConfigId = null
                                }
                                runAction("正在删除项目") {
                                    coreGateway.deleteProject(projectId)
                                }
                            },
                            onCreateProject = { name ->
                                projectWorkspaceOpen = false
                                runAction("正在创建项目") {
                                    val project = coreGateway.createProject(name)
                                    projectConfigId = project.id
                                    projectConfigPanel = null
                                }
                            },
                            modifier = Modifier.padding(padding),
                        )
                    } else {
                        val activeProjectId = projectState.activeProjectId
                        val receiverPanelExpanded = activeProjectId
                            ?.let { receiverPanelExpandedByProject[it] }
                            ?: !dashboard.receiver.running
                        ProjectAssetsScreen(
                            coreGateway = coreGateway,
                            dashboard = dashboard,
                            projectState = projectState,
                            notificationPermissionGranted = notificationsGranted,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onOpenProjects = {
                                projectWorkspaceOpen = projectWorkspaceStateAfterOpenProjects(
                                    ProjectWorkspaceNavigationState(projectWorkspaceOpen),
                                ).workspaceOpen
                            },
                            onOpenProjectIntelligence = {
                                projectState.activeProjectId?.let { projectId ->
                                    openProjectConfig(projectId, keepProjectWorkspace = true)
                                }
                            },
                            onConfigureAccount = {
                                destination = GlobalDestination.Accounts
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
                            receiverPanelExpanded = receiverPanelExpanded,
                            onReceiverPanelExpandedChange = { expanded ->
                                activeProjectId?.let { receiverPanelExpandedByProject[it] = expanded }
                            },
                            onRetryFailedPublishes = {
                                runAction("\u6b63\u5728\u91cd\u8bd5\u5199\u5165") {
                                    coreGateway.retryFailedPublishes()
                                }
                            },
                            onSplitBurstMember = { burstGroupId, memberGroupId ->
                                runAction("\u6b63\u5728\u79fb\u51fa\u8fde\u62cd\u7ec4") {
                                    coreGateway.splitBurstMember(burstGroupId, memberGroupId)
                                }
                            },
                            onSplitBurstMembers = { targets ->
                                runAction("\u6b63\u5728\u79fb\u51fa\u8fde\u62cd\u7ec4") {
                                    targets.forEach { target ->
                                        coreGateway.splitBurstMember(target.burstGroupId, target.memberGroupId)
                                    }
                                }
                            },
                            onCreateManualBurstGroup = { projectId, memberGroupIds ->
                                runAction("\u6b63\u5728\u5408\u5e76\u8fde\u62cd\u7ec4") {
                                    coreGateway.createManualBurstGroup(projectId, memberGroupIds)
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
                        val accountForDetail = accountDetail
                        if (accountForDetail != null || addingAccount) {
                            AccountDetailScreen(
                                account = accountForDetail,
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
                    } else if (settingsModelProvidersOpen) {
                        ModelProviderProfilesScreen(
                            modelProviderSettings = modelProviderSettings,
                            modelProviderSettingsList = modelProviderSettingsList,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onBack = { closeModelProviderSettings() },
                            onSaveModelProviderSettings = { settings ->
                                runAction("正在保存模型服务设置") {
                                    val saved = coreGateway.saveModelProviderSettings(settings)
                                    modelProviderSettings = saved
                                    modelProviderSettingsList = coreGateway.loadModelProviderSettingsList()
                                }
                            },
                            onDeleteModelProviderSettings = { settingsId ->
                                runAction("正在删除模型服务设置") {
                                    coreGateway.deleteModelProviderSettings(settingsId)
                                    modelProviderSettingsList = coreGateway.loadModelProviderSettingsList()
                                    modelProviderSettings = coreGateway.loadModelProviderSettings()
                                }
                            },
                            modifier = Modifier.padding(padding),
                        )
                    } else if (editingPromptPackId != null || creatingPromptPack) {
                        val editingProfile = editingPromptPackId?.let { promptId ->
                            globalPromptPacks.firstOrNull { it.promptPackId == promptId }
                        }
                        PromptPackEditorScreen(
                            profile = editingProfile,
                            initialDistributionFolder = creatingPromptPackPackage,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onBack = {
                                editingPromptPackId = null
                                creatingPromptPack = false
                                creatingPromptPackPackage = "user"
                            },
                            onSave = { profile, name, styleTags, sceneProfile, distributionFolder, promptText ->
                                runAction(if (profile.builtIn) "正在复制提示词" else "正在保存提示词") {
                                    val saved = if (profile.builtIn) {
                                        val forked = coreGateway.forkGlobalPromptPack(
                                            profile.promptPackId,
                                            name,
                                            distributionFolder,
                                        )
                                        coreGateway.saveGlobalPromptPack(
                                            forked.promptPackId,
                                            name,
                                            styleTags,
                                            sceneProfile,
                                            promptText,
                                        )
                                    } else {
                                        coreGateway.saveGlobalPromptPack(
                                            profile.promptPackId,
                                            name,
                                            styleTags,
                                            sceneProfile,
                                            promptText,
                                        )
                                    }
                                    val loaded = coreGateway.loadGlobalPromptPacks()
                                    globalPromptPacks = if (loaded.any { it.promptPackId == saved.promptPackId }) {
                                        loaded
                                    } else {
                                        loaded + saved
                                    }
                                    editingPromptPackId = saved.promptPackId
                                }
                            },
                            onCreate = { name, styleTags, sceneProfile, distributionFolder, promptText ->
                                runAction("正在创建提示词") {
                                    val saved = coreGateway.createGlobalPromptPack(
                                        name = name,
                                        styleTags = styleTags,
                                        sceneProfile = sceneProfile,
                                        distributionFolder = distributionFolder,
                                        promptText = promptText,
                                    )
                                    val loaded = coreGateway.loadGlobalPromptPacks()
                                    globalPromptPacks = if (loaded.any { it.promptPackId == saved.promptPackId }) {
                                        loaded
                                    } else {
                                        loaded + saved
                                    }
                                    creatingPromptPack = false
                                    creatingPromptPackPackage = "user"
                                    editingPromptPackId = saved.promptPackId
                                    settingsPromptPacksOpen = true
                                }
                            },
                            modifier = Modifier.padding(padding),
                        )
                    } else if (settingsPromptPacksOpen) {
                        PromptPacksScreen(
                            promptPacks = globalPromptPacks,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onBack = { settingsPromptPacksOpen = false },
                            onCreatePromptPackage = {
                                editingPromptPackId = null
                                creatingPromptPackPackage = ""
                                creatingPromptPack = true
                            },
                            onCreatePromptPackInPackage = { packageFolder ->
                                editingPromptPackId = null
                                creatingPromptPackPackage = packageFolder
                                creatingPromptPack = true
                            },
                            onOpenPromptPack = { promptId -> editingPromptPackId = promptId },
                            onDeletePromptPack = { promptId ->
                                runAction("正在删除提示词") {
                                    coreGateway.deleteGlobalPromptPack(promptId)
                                    globalPromptPacks = coreGateway.loadGlobalPromptPacks()
                                    if (editingPromptPackId == promptId) {
                                        editingPromptPackId = null
                                    }
                                }
                            },
                            onDeletePromptPackage = { packageFolder ->
                                runAction("正在删除提示词包") {
                                    coreGateway.deleteGlobalPromptPackage(packageFolder)
                                    globalPromptPacks = coreGateway.loadGlobalPromptPacks()
                                    if (editingPromptPackId?.let { promptId ->
                                            globalPromptPacks.none { it.promptPackId == promptId }
                                        } == true
                                    ) {
                                        editingPromptPackId = null
                                    }
                                }
                            },
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
                            selectedOutputLabel = selectedOutput,
                            onChooseOutputDirectory = onChooseOutputDirectory,
                            onOpenDiagnostics = { settingsDiagnosticsOpen = true },
                            onOpenPromptPacks = { settingsPromptPacksOpen = true },
                            onOpenModelProviders = {
                                settingsModelProvidersReturnProjectId = null
                                settingsModelProvidersOpen = true
                            },
                            projectPhotoGridColumnCount = projectPhotoGridColumnCount,
                            onProjectPhotoGridColumnCountChange = { count ->
                                projectPhotoGridColumnCount = count
                                storageGateway.persistProjectPhotoGridColumnCount(count)
                            },
                            modelProviderSettingsList = modelProviderSettingsList,
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

private suspend fun projectRecommendationCandidateVisuals(
    context: Context,
    assets: List<ProjectAsset>,
): List<SelectionCandidateVisualInput> =
    withContext(Dispatchers.IO) {
        projectRecommendationVisualCandidates(assets)
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

private fun projectRecommendationVisualCandidates(assets: List<ProjectAsset>): List<ProjectAsset> =
    assets
        .asSequence()
        .filter { asset ->
            asset.isModelSelect ||
                asset.modelScore != null ||
                asset.modelStatus?.equals("ready", ignoreCase = true) == true
        }
        .distinctBy { it.id }
        .sortedWith(
            compareByDescending<ProjectAsset> { it.isModelSelect }
                .thenByDescending { it.modelScore ?: Int.MIN_VALUE }
                .thenByDescending { it.receivedAt.toLongOrNull() ?: Long.MIN_VALUE },
        )
        .take(PROJECT_RECOMMENDATION_VISUAL_LIMIT)
        .toList()

private const val PROJECT_RECOMMENDATION_VISUAL_LIMIT = 48

@Composable
private fun CameraConnectorBottomBar(
    selected: GlobalDestination,
    onSelect: (GlobalDestination) -> Unit,
) {
    Box(
        modifier = Modifier.fillMaxWidth(),
        contentAlignment = Alignment.BottomCenter,
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(78.dp),
        ) {
            Canvas(modifier = Modifier.fillMaxSize()) {
                val centerX = size.width / 2f
                val top = 10.dp.toPx()
                val dip = 20.dp.toPx()
                val shoulder = 66.dp.toPx()
                val neck = 38.dp.toPx()
                val dock = Path().apply {
                    moveTo(0f, top)
                    lineTo(centerX - shoulder, top)
                    cubicTo(
                        centerX - 54.dp.toPx(),
                        top,
                        centerX - 50.dp.toPx(),
                        top + dip,
                        centerX - neck,
                        top + dip,
                    )
                    lineTo(centerX + neck, top + dip)
                    cubicTo(
                        centerX + 50.dp.toPx(),
                        top + dip,
                        centerX + 54.dp.toPx(),
                        top,
                        centerX + shoulder,
                        top,
                    )
                    lineTo(size.width, top)
                    lineTo(size.width, size.height)
                    lineTo(0f, size.height)
                    close()
                }
                drawPath(
                    path = dock,
                    color = ElementSurface,
                )
                drawPath(
                    path = dock,
                    color = ElementCardBorder.copy(alpha = 0.72f),
                    style = Stroke(width = 1.dp.toPx(), cap = StrokeCap.Round),
                )
            }
            SecondaryBottomDestination(
                destination = GlobalDestination.Settings,
                selected = selected == GlobalDestination.Settings,
                onClick = { onSelect(GlobalDestination.Settings) },
                modifier = Modifier
                    .align(Alignment.TopStart)
                    .padding(start = 42.dp, top = 24.dp),
            )
            PrimaryBottomDestination(
                destination = GlobalDestination.Projects,
                selected = selected == GlobalDestination.Projects,
                onClick = { onSelect(GlobalDestination.Projects) },
                modifier = Modifier
                    .align(Alignment.TopCenter)
                    .offset(y = 2.dp),
            )
            SecondaryBottomDestination(
                destination = GlobalDestination.Accounts,
                selected = selected == GlobalDestination.Accounts,
                onClick = { onSelect(GlobalDestination.Accounts) },
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(end = 42.dp, top = 24.dp),
            )
        }
    }
}

@Composable
private fun PrimaryBottomDestination(
    destination: GlobalDestination,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier
            .size(52.dp)
            .clip(CircleShape)
            .clickable(onClick = onClick),
        color = if (selected) ElementBlue else ElementControlSurface,
        contentColor = if (selected) ElementOnAccent else MaterialTheme.colorScheme.onSurfaceVariant,
        shape = CircleShape,
        border = BorderStroke(1.dp, if (selected) ElementBlue else ElementCardBorder),
        shadowElevation = if (selected) 10.dp else 2.dp,
    ) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = destination.icon(),
                contentDescription = destination.label,
                modifier = Modifier.size(24.dp),
            )
        }
    }
}

@Composable
private fun SecondaryBottomDestination(
    destination: GlobalDestination,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier
            .size(40.dp)
            .clip(CircleShape)
            .clickable(onClick = onClick),
        color = if (selected) ElementBlueSoft else Color.Transparent,
        contentColor = if (selected) ElementBlue else MaterialTheme.colorScheme.onSurfaceVariant,
        shape = CircleShape,
        border = BorderStroke(
            1.dp,
            if (selected) ElementBlue.copy(alpha = 0.52f) else Color.Transparent,
        ),
    ) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = destination.icon(),
                contentDescription = destination.label,
                modifier = Modifier.size(21.dp),
            )
        }
    }
}

private fun GlobalDestination.icon(): ImageVector = when (this) {
    GlobalDestination.Projects -> Icons.Outlined.Home
    GlobalDestination.Accounts -> Icons.Outlined.Person
    GlobalDestination.Settings -> Icons.Outlined.Settings
}

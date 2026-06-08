package com.cameraconnector.app.ui

import android.content.Context
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
import androidx.compose.ui.platform.LocalContext
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.DEFAULT_LISTEN_HOST
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PromptProfileUi
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
    var projectConfigId by rememberSaveable { mutableStateOf<String?>(null) }
    var settingsDiagnosticsOpen by remember { mutableStateOf(false) }
    var settingsModelProvidersOpen by remember { mutableStateOf(false) }
    var settingsPromptProfilesOpen by remember { mutableStateOf(false) }
    var editingPromptProfileId by rememberSaveable { mutableStateOf<String?>(null) }
    var creatingPromptProfile by rememberSaveable { mutableStateOf(false) }
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
    var globalPromptProfiles by remember { mutableStateOf<List<PromptProfileUi>>(emptyList()) }
    var projectConfigEvaluationSettings by remember { mutableStateOf<ProjectEvaluationSettingsUi?>(null) }
    var projectConfigPromptProfiles by remember { mutableStateOf<List<PromptProfileUi>>(emptyList()) }
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
        settingsDiagnosticsOpen = false
        settingsModelProvidersOpen = false
        settingsPromptProfilesOpen = false
        editingPromptProfileId = null
        creatingPromptProfile = false
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
        settingsDiagnosticsOpen = false
        settingsModelProvidersOpen = false
        settingsPromptProfilesOpen = false
        editingPromptProfileId = null
        creatingPromptProfile = false
        accountDetail = null
        addingAccount = false
    }

    LaunchedEffect(coreGateway) {
        runCatching {
            Triple(
                coreGateway.loadModelProviderSettings(),
                coreGateway.loadModelProviderSettingsList(),
                coreGateway.loadGlobalPromptProfiles(),
            )
        }.onSuccess { (providerSettings, providerSettingsList, prompts) ->
            modelProviderSettings = providerSettings
            modelProviderSettingsList = providerSettingsList
            globalPromptProfiles = prompts
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
            projectConfigPromptProfiles = emptyList()
            return@LaunchedEffect
        }

        runCatching {
            coreGateway.loadProjectEvaluationSettings(projectId) to coreGateway.loadPromptProfiles(projectId)
        }.onSuccess { (settings, profiles) ->
            projectConfigEvaluationSettings = settings
            projectConfigPromptProfiles = profiles
        }.onFailure { error ->
            if (!error.isUiCancellationNoise()) {
                actionError = error.message ?: error::class.java.simpleName
            }
        }
    }

    BackHandler(enabled = destination == GlobalDestination.Projects && projectConfigId != null) {
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
        settingsModelProvidersOpen = false
    }
    BackHandler(enabled = destination == GlobalDestination.Settings && settingsPromptProfilesOpen) {
        settingsPromptProfilesOpen = false
    }
    BackHandler(
        enabled = destination == GlobalDestination.Settings &&
            (editingPromptProfileId != null || creatingPromptProfile),
    ) {
        editingPromptProfileId = null
        creatingPromptProfile = false
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
                                    projectConfigId = null
                                    projectWorkspaceOpen = projectWorkspaceStateAfterBottomDestinationClick(
                                        current = ProjectWorkspaceNavigationState(projectWorkspaceOpen),
                                        destination = item,
                                    ).workspaceOpen
                                    settingsDiagnosticsOpen = false
                                    settingsModelProvidersOpen = false
                                    settingsPromptProfilesOpen = false
                                    editingPromptProfileId = null
                                    creatingPromptProfile = false
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
                            providerOptions = modelProviderSettingsList.ifEmpty {
                                listOf(modelProviderSettings).filter { it.configured }
                            },
                            settings = projectConfigEvaluationSettings,
                            promptProfiles = globalPromptProfiles.ifEmpty { projectConfigPromptProfiles },
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
                                destination = GlobalDestination.Settings
                                projectConfigId = null
                                settingsDiagnosticsOpen = false
                                settingsModelProvidersOpen = true
                                settingsPromptProfilesOpen = false
                            },
                            modifier = Modifier.padding(padding),
                        )
                    } else if (!projectWorkspaceIsVisible) {
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
                            onRetryFailedPublishes = {
                                runAction("正在重试发布") {
                                    coreGateway.retryFailedPublishes()
                                }
                            },
                            onMoveProjectGroup = { sourceProjectId, groupId, targetProjectId ->
                                runAction("正在移动文件") {
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
                            onBack = { settingsModelProvidersOpen = false },
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
                    } else if (editingPromptProfileId != null || creatingPromptProfile) {
                        val editingProfile = editingPromptProfileId?.let { promptId ->
                            globalPromptProfiles.firstOrNull { it.promptProfileId == promptId }
                        }
                        PromptProfileEditorScreen(
                            profile = editingProfile,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onBack = {
                                editingPromptProfileId = null
                                creatingPromptProfile = false
                            },
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
                            onCreate = { name, styleTags, sceneProfile, promptText ->
                                runAction("正在创建提示词") {
                                    coreGateway.createGlobalPromptProfile(
                                        name = name,
                                        styleTags = styleTags,
                                        sceneProfile = sceneProfile,
                                        promptText = promptText,
                                    )
                                    globalPromptProfiles = coreGateway.loadGlobalPromptProfiles()
                                    creatingPromptProfile = false
                                    editingPromptProfileId = null
                                    settingsPromptProfilesOpen = true
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
                            onCreatePromptProfile = {
                                editingPromptProfileId = null
                                creatingPromptProfile = true
                            },
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
                            selectedOutputLabel = selectedOutput,
                            onChooseOutputDirectory = onChooseOutputDirectory,
                            onOpenDiagnostics = { settingsDiagnosticsOpen = true },
                            onOpenPromptProfiles = { settingsPromptProfilesOpen = true },
                            onOpenModelProviders = { settingsModelProvidersOpen = true },
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

private fun GlobalDestination.icon(): ImageVector = when (this) {
    GlobalDestination.Projects -> Icons.Outlined.Home
    GlobalDestination.Accounts -> Icons.Outlined.Person
    GlobalDestination.Settings -> Icons.Outlined.Settings
}

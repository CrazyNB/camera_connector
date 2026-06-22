package com.cameraconnector.app.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PromptPackUi
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.ProjectSummary

@Composable
internal fun SettingsScreen(
    dashboard: DashboardState,
    notificationPermissionRequired: Boolean,
    notificationPermissionGranted: Boolean,
    onRequestNotificationPermission: () -> Unit,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    selectedOutputLabel: String?,
    onChooseOutputDirectory: () -> Unit,
    onOpenDiagnostics: () -> Unit,
    onOpenPromptPacks: () -> Unit,
    onOpenModelProviders: () -> Unit,
    projectPhotoGridColumnCount: Int,
    onProjectPhotoGridColumnCountChange: (Int) -> Unit,
    modelProviderSettingsList: List<ModelProviderSettingsUi> = emptyList(),
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            CompactPageHeader(
                title = "系统设置",
                subtitle = "接收、存储与通知权限",
            )
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        item {
            Text("工具", style = MaterialTheme.typography.titleMedium)
        }
        item {
            SettingsMenuRow(
                title = "诊断日志",
                subtitle = "\u8fde\u63a5\u3001\u4f20\u8f93\u548c\u5199\u5165\u72b6\u6001",
                trailing = ">",
                onClick = onOpenDiagnostics,
            )
        }

        item {
            Text("照片视图", style = MaterialTheme.typography.titleMedium)
        }
        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Row(
                    modifier = Modifier.padding(16.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Column(Modifier.weight(1f)) {
                        Text("项目照片网格", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(4.dp))
                        Text(
                            "${projectPhotoGridColumnCount}\u5217",
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    Spacer(Modifier.width(12.dp))
                    GridColumnToggle(
                        columnCount = projectPhotoGridColumnCount,
                        onColumnCountChange = onProjectPhotoGridColumnCountChange,
                    )
                }
            }
        }

        item {
            Text("\u667a\u80fd\u4f18\u9009", style = MaterialTheme.typography.titleMedium)
        }
        item {
            val configuredCount = modelProviderSettingsList.count { it.configured && it.providerKind != "none" }
            SettingsMenuRow(
                title = "模型服务",
                subtitle = if (configuredCount > 0) {
                    "已配置 ${configuredCount} 个"
                } else {
                    "未配置"
                },
                trailing = ">",
                onClick = onOpenModelProviders,
            )
        }
        item {
            SettingsMenuRow(
                title = "\u63d0\u793a\u8bcd\u914d\u7f6e",
                subtitle = "评价偏好与风格标签",
                trailing = ">",
                onClick = onOpenPromptPacks,
            )
        }
        if (notificationPermissionRequired && !notificationPermissionGranted) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text("通知权限", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(8.dp))
                        Text("启动接收服务前需要允许通知权限")
                        Spacer(Modifier.height(12.dp))
                        Button(onClick = onRequestNotificationPermission) {
                            Text("允许通知")
                        }
                    }
                }
            }
        }

        item {
            Text("导入位置", style = MaterialTheme.typography.titleMedium)
        }
        item {
            SettingsMenuRow(
                title = "\u5916\u90e8\u6587\u4ef6\u5939\u6388\u6743",
                subtitle = selectedOutputLabel ?: "\u672a\u6388\u6743",
                trailing = ">",
                onClick = onChooseOutputDirectory,
            )
        }
        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("当前接收目录", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    Text(dashboard.receiver.outputLabel, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}

@Composable
internal fun ModelProviderProfilesScreen(
    modelProviderSettings: ModelProviderSettingsUi,
    modelProviderSettingsList: List<ModelProviderSettingsUi>,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onSaveModelProviderSettings: (ModelProviderSettingsUi) -> Unit,
    onDeleteModelProviderSettings: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var editingSettingsId by rememberSaveable { mutableStateOf<String?>(null) }
    var creatingProvider by rememberSaveable { mutableStateOf(false) }
    val editingProvider = modelProviderSettingsList.firstOrNull { it.settingsId == editingSettingsId }
    val editorOpen = creatingProvider || editingProvider != null

    fun closeEditorOrScreen() {
        if (editorOpen) {
            creatingProvider = false
            editingSettingsId = null
        } else {
            onBack()
        }
    }

    BackHandler(enabled = editorOpen) {
        creatingProvider = false
        editingSettingsId = null
    }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            CompactBackHeader(
                title = when {
                    creatingProvider -> "新建模型服务"
                    editingProvider != null -> "编辑模型服务"
                    else -> "模型服务"
                },
                onBack = ::closeEditorOrScreen,
                trailing = {
                    if (!editorOpen) {
                    OutlinedButton(
                        onClick = { creatingProvider = true },
                        enabled = actionInFlight == null,
                        shape = elementShape,
                    ) {
                        Text("新建")
                    }
                    }
                },
            )
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        if (editorOpen) {
            item {
                ModelProviderSettingsCard(
                    settings = editingProvider ?: modelProviderSettings.copy(settingsId = ""),
                    settingsList = editingProvider?.let(::listOf).orEmpty(),
                    actionsEnabled = actionInFlight == null,
                    onSaveSettings = { settings ->
                        onSaveModelProviderSettings(settings)
                        creatingProvider = false
                        editingSettingsId = null
                    },
                    onDeleteSettings = { settingsId ->
                        onDeleteModelProviderSettings(settingsId)
                        creatingProvider = false
                        editingSettingsId = null
                    },
                )
            }
        } else if (modelProviderSettingsList.isEmpty()) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(
                        modifier = Modifier.padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Text("还没有模型服务", style = MaterialTheme.typography.titleMedium)
                        Button(
                            onClick = { creatingProvider = true },
                            enabled = actionInFlight == null,
                            shape = elementShape,
                            modifier = Modifier.fillMaxWidth(),
                        ) {
                            Text("新建模型服务")
                        }
                    }
                }
            }
        } else {
            items(modelProviderSettingsList, key = { it.settingsId }) { option ->
                SettingsMenuRow(
                    title = modelProviderOptionLabel(option),
                    subtitle = option.baseUrl.ifBlank { option.providerLabel },
                    trailing = ">",
                    onClick = {
                        creatingProvider = false
                        editingSettingsId = option.settingsId
                    },
                )
            }
        }
    }
}

@Composable
internal fun ProjectSettingsScreen(
    project: ProjectSummary?,
    providerOptions: List<ModelProviderSettingsUi>,
    settings: ProjectEvaluationSettingsUi?,
    promptPacks: List<PromptPackUi>,
    actionError: String?,
    actionInFlight: String?,
    selectedPanel: ProjectIntelligencePanel?,
    onSelectedPanelChange: (ProjectIntelligencePanel?) -> Unit,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
    onGenerateProjectRecommendation: () -> Unit,
    onConfigureModelProvider: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val projectSettings = settings ?: project?.let { ProjectEvaluationSettingsUi(projectId = it.id) }

    if (projectSettings != null && selectedPanel != null) {
        ProjectIntelligencePanelPage(
            panel = selectedPanel,
            projectSettings = projectSettings,
            providerOptions = providerOptions,
            promptPacks = promptPacks,
            actionError = actionError,
            actionInFlight = actionInFlight,
            actionsEnabled = actionInFlight == null,
            onClearActionError = onClearActionError,
            onBack = { onSelectedPanelChange(null) },
            onSaveSettings = onSaveSettings,
            onConfigureModelProvider = onConfigureModelProvider,
            modifier = modifier,
        )
        return
    }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item(key = "project-settings-header") {
            CompactBackHeader(
                title = project?.name ?: "项目配置",
                subtitle = "\u9879\u76ee\u667a\u80fd\u3001\u573a\u666f\u548c\u4f18\u9009\u7b56\u7565",
                onBack = onBack,
            )
        }

        actionError?.let { message ->
            item(key = "project-settings-error") {
                ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError)
            }
        }

        if (projectSettings == null) {
            item(key = "project-settings-missing") {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "项目不存在或已被移除",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            item(key = "project-scene-settings") {
                ProjectSceneQuickSettings(
                    projectSettings = projectSettings,
                    actionsEnabled = actionInFlight == null,
                    onSaveSettings = onSaveSettings,
                )
            }
            item(key = "project-intelligence-settings") {
                ProjectIntelligenceSettingsCard(
                    providerOptions = providerOptions,
                    settings = projectSettings,
                    promptPacks = promptPacks,
                    actionsEnabled = actionInFlight == null,
                    onGenerateProjectRecommendation = onGenerateProjectRecommendation,
                    onOpenPanel = onSelectedPanelChange,
                )
            }
        }
    }
}

@Composable
internal fun OptionRow(
    title: String,
    values: List<String>,
    selected: String,
    enabled: Boolean,
    labelForValue: (String) -> String = { it },
    onSelected: (String) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(title, style = MaterialTheme.typography.labelLarge)
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            items(values) { value ->
                FilterChipButton(
                    label = labelForValue(value),
                    selected = value == selected,
                    onClick = { if (enabled) onSelected(value) },
                )
            }
        }
    }
}

@Composable
internal fun GridColumnToggle(
    columnCount: Int,
    onColumnCountChange: (Int) -> Unit,
) {
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        listOf(2 to "2列", 3 to "3列").forEach { (count, label) ->
            OutlinedButton(
                onClick = { onColumnCountChange(count) },
                modifier = Modifier
                    .height(30.dp)
                    .defaultMinSize(minWidth = 1.dp, minHeight = 1.dp)
                    .semantics {
                        contentDescription = "照片${label}视图"
                        stateDescription = if (columnCount == count) "已选中" else "未选中"
                    },
                border = BorderStroke(1.dp, if (columnCount == count) ElementBlue else ElementBorder),
                colors = ButtonDefaults.outlinedButtonColors(
                    containerColor = if (columnCount == count) ElementBlue else ElementControlSurface,
                    contentColor = if (columnCount == count) ElementOnAccent else MaterialTheme.colorScheme.onSurfaceVariant,
                ),
                shape = elementShape,
                contentPadding = PaddingValues(horizontal = 9.dp, vertical = 0.dp),
            ) {
                Text(label, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            }
        }
    }
}

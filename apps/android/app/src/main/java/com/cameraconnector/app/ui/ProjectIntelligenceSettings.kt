package com.cameraconnector.app.ui

import android.widget.Toast
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.KeyboardArrowRight
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PromptPackUi
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi


@Composable
internal fun ProjectIntelligencePanelPage(
    panel: ProjectIntelligencePanel,
    projectSettings: ProjectEvaluationSettingsUi,
    providerOptions: List<ModelProviderSettingsUi>,
    promptPacks: List<PromptPackUi>,
    actionError: String?,
    actionInFlight: String?,
    actionsEnabled: Boolean,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
    onConfigureModelProvider: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val modelOptions = providerOptions
        .filter { it.configured && it.providerKind != "none" }
        .distinctBy { it.settingsId }
    val selectedProviderReady = modelProviderReadyForProject(projectSettings, modelOptions)
    val selectablePromptPacks = promptPacks
        .filter { it.enabled && (it.scope.equals("global", ignoreCase = true) || it.projectId == null) }
        .ifEmpty { promptPacks.filter { it.enabled } }
    val selectedPrompt = selectablePromptPacks.firstOrNull { it.promptPackId == projectSettings.promptPackId }
        ?: selectablePromptPacks.firstOrNull()

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item(key = "project-intelligence-panel-header-${panel.name}") {
            CompactBackHeader(
                title = projectIntelligencePanelTitle(panel),
                subtitle = projectIntelligencePanelSubtitle(panel),
                onBack = onBack,
            )
        }
        actionError?.let { message ->
            item(key = "project-intelligence-panel-error-${panel.name}") {
                ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError)
            }
        }
        item(key = "project-intelligence-panel-content-${panel.name}") {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    ProjectIntelligencePanelContent(
                        panel = panel,
                        projectSettings = projectSettings,
                        modelOptions = modelOptions,
                        selectedProviderReady = selectedProviderReady,
                        selectedPrompt = selectedPrompt,
                        promptPacks = selectablePromptPacks,
                        actionsEnabled = actionsEnabled,
                        onSaveSettings = onSaveSettings,
                        onConfigureModelProvider = onConfigureModelProvider,
                    )
                }
            }
        }
    }
}

@Composable
internal fun ProjectIntelligenceSettingsCard(
    providerOptions: List<ModelProviderSettingsUi>,
    settings: ProjectEvaluationSettingsUi?,
    promptPacks: List<PromptPackUi>,
    actionsEnabled: Boolean,
    onGenerateProjectRecommendation: () -> Unit,
    onOpenPanel: (ProjectIntelligencePanel) -> Unit,
) {
    val projectSettings = settings ?: ProjectEvaluationSettingsUi(projectId = "")
    val modelOptions = providerOptions
        .filter { it.configured && it.providerKind != "none" }
        .distinctBy { it.settingsId }
    val selectedProviderReady = modelProviderReadyForProject(projectSettings, modelOptions)
    val selectablePromptPacks = promptPacks
        .filter { it.enabled && (it.scope.equals("global", ignoreCase = true) || it.projectId == null) }
        .ifEmpty { promptPacks.filter { it.enabled } }
    val selectedPrompt = selectablePromptPacks.firstOrNull { it.promptPackId == projectSettings.promptPackId }
        ?: selectablePromptPacks.firstOrNull()
    val recommendationAction = manualProjectRecommendationActionUi(
        providerConfigured = selectedProviderReady,
        settings = projectSettings,
        actionInFlight = !actionsEnabled,
    )

    ProjectIntelligenceOverviewCard(
        projectSettings = projectSettings,
        modelOptions = modelOptions,
        selectedProviderReady = selectedProviderReady,
        selectedPrompt = selectedPrompt,
        recommendationAction = recommendationAction,
        actionsEnabled = actionsEnabled,
        onOpenPanel = onOpenPanel,
        onGenerateProjectRecommendation = onGenerateProjectRecommendation,
    )
}

internal enum class ProjectIntelligencePanel {
    Model,
    Workflow,
    Scene,
    Prompt,
}

@Composable
private fun ProjectIntelligenceOverviewCard(
    projectSettings: ProjectEvaluationSettingsUi,
    modelOptions: List<ModelProviderSettingsUi>,
    selectedProviderReady: Boolean,
    selectedPrompt: PromptPackUi?,
    recommendationAction: ManualProjectRecommendationActionUi,
    actionsEnabled: Boolean,
    onOpenPanel: (ProjectIntelligencePanel) -> Unit,
    onGenerateProjectRecommendation: () -> Unit,
) {
    val selectedModel = modelOptions.firstOrNull { it.settingsId == projectSettings.modelProviderSettingsId }
    ElementCard(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(vertical = 4.dp),
            verticalArrangement = Arrangement.spacedBy(2.dp),
        ) {
            ProjectIntelligenceMenuRow(
                title = "模型服务",
                subtitle = selectedModel?.let(::modelProviderOptionLabel)
                    ?: if (selectedProviderReady) "已配置" else "未配置",
                warning = !selectedProviderReady,
                onClick = { onOpenPanel(ProjectIntelligencePanel.Model) },
            )
            ProjectIntelligenceMenuRow(
                title = "自动流程",
                subtitle = projectWorkflowSummary(projectSettings),
                onClick = { onOpenPanel(ProjectIntelligencePanel.Workflow) },
            )
            ProjectIntelligenceMenuRow(
                title = "技术风险阈值",
                subtitle = technicalRiskSummary(projectSettings),
                onClick = { onOpenPanel(ProjectIntelligencePanel.Scene) },
            )
            ProjectIntelligenceMenuRow(
                title = "评价提示词",
                subtitle = selectedPrompt?.let(::promptPackDisplayName) ?: "未选择",
                warning = selectedPrompt == null,
                onClick = { onOpenPanel(ProjectIntelligencePanel.Prompt) },
            )
            Button(
                onClick = onGenerateProjectRecommendation,
                enabled = actionsEnabled && recommendationAction.enabled,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 12.dp),
                shape = elementShape,
            ) {
                Text(recommendationAction.ctaLabel)
            }
        }
    }
}

@Composable
private fun ProjectIntelligenceMenuRow(
    title: String,
    subtitle: String,
    warning: Boolean = false,
    onClick: () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 13.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Text(title, style = MaterialTheme.typography.titleMedium)
            Text(
                subtitle,
                color = if (warning) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Icon(
            imageVector = Icons.AutoMirrored.Outlined.KeyboardArrowRight,
            contentDescription = "进入$title",
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
internal fun ProjectSceneQuickSettings(
    projectSettings: ProjectEvaluationSettingsUi,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
) {
    ElementCard(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            OptionRow(
                title = "项目场景",
                values = projectSceneProfileOptions(),
                selected = projectSettings.sceneProfile,
                enabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
                labelForValue = ::sceneProfileLabel,
                onSelected = { onSaveSettings(projectSettings.copy(sceneProfile = it)) },
            )
            Text(
                sceneProfileHint(projectSettings.sceneProfile),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}

@Composable
private fun ProjectIntelligencePanelContent(
    panel: ProjectIntelligencePanel,
    projectSettings: ProjectEvaluationSettingsUi,
    modelOptions: List<ModelProviderSettingsUi>,
    selectedProviderReady: Boolean,
    selectedPrompt: PromptPackUi?,
    promptPacks: List<PromptPackUi>,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
    onConfigureModelProvider: () -> Unit,
) {
    when (panel) {
        ProjectIntelligencePanel.Model -> ProjectModelPanel(
            projectSettings = projectSettings,
            modelOptions = modelOptions,
            selectedProviderReady = selectedProviderReady,
            actionsEnabled = actionsEnabled,
            onSaveSettings = onSaveSettings,
            onConfigureModelProvider = onConfigureModelProvider,
        )
        ProjectIntelligencePanel.Workflow -> ProjectWorkflowPanel(
            projectSettings = projectSettings,
            selectedProviderReady = selectedProviderReady,
            actionsEnabled = actionsEnabled,
            onSaveSettings = onSaveSettings,
            onConfigureModelProvider = onConfigureModelProvider,
        )
        ProjectIntelligencePanel.Scene -> ProjectScenePanel(
            projectSettings = projectSettings,
            actionsEnabled = actionsEnabled,
            onSaveSettings = onSaveSettings,
        )
        ProjectIntelligencePanel.Prompt -> ProjectPromptPanel(
            projectSettings = projectSettings,
            selectedPrompt = selectedPrompt,
            promptPacks = promptPacks,
            actionsEnabled = actionsEnabled,
            onSaveSettings = onSaveSettings,
        )
    }
}

@Composable
private fun ProjectModelPanel(
    projectSettings: ProjectEvaluationSettingsUi,
    modelOptions: List<ModelProviderSettingsUi>,
    selectedProviderReady: Boolean,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
    onConfigureModelProvider: () -> Unit,
) {
    if (!selectedProviderReady) {
        Text(
            if (modelOptions.isEmpty()) "还没有可用模型服务。" else "当前项目尚未选择可用模型服务。",
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        modelOptions.forEach { option ->
            ProjectSelectableRow(
                title = modelProviderOptionLabel(option),
                subtitle = option.baseUrl.ifBlank { option.providerLabel },
                selected = projectSettings.modelProviderSettingsId == option.settingsId,
                enabled = actionsEnabled,
                onClick = {
                    onSaveSettings(projectSettingsAfterModelProviderSelection(projectSettings, option.settingsId))
                },
            )
        }
    }
    OutlinedButton(
        onClick = onConfigureModelProvider,
        enabled = actionsEnabled,
        modifier = Modifier.fillMaxWidth(),
        shape = elementShape,
    ) {
        Text("管理模型服务")
    }
}

@Composable
private fun ProjectWorkflowPanel(
    projectSettings: ProjectEvaluationSettingsUi,
    selectedProviderReady: Boolean,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
    onConfigureModelProvider: () -> Unit,
) {
    val context = LocalContext.current
    val showModelMissingMessage = {
        Toast.makeText(context, "请先配置模型服务", Toast.LENGTH_SHORT).show()
    }
    if (!selectedProviderReady) {
        MissingModelProviderNotice(
            enabled = actionsEnabled,
            onConfigureModelProvider = onConfigureModelProvider,
        )
    }
    SettingsSwitchRow(
        title = "上传后自动评价",
        checked = projectSettings.autoEvaluateOnUpload,
        enabled = actionsEnabled && selectedProviderReady,
        onDisabledClick = if (actionsEnabled && !selectedProviderReady) showModelMissingMessage else null,
        onCheckedChange = { onSaveSettings(projectSettings.copy(autoEvaluateOnUpload = it)) },
    )
    SettingsSwitchRow(
        title = "连拍组自动优选",
        checked = projectSettings.autoBurstRecommendationEnabled,
        enabled = actionsEnabled && selectedProviderReady && projectSettings.projectId.isNotBlank(),
        onDisabledClick = if (actionsEnabled && !selectedProviderReady) showModelMissingMessage else null,
        onCheckedChange = { onSaveSettings(projectSettings.copy(autoBurstRecommendationEnabled = it)) },
    )
    SettingsSwitchRow(
        title = "允许风险照片参与优选",
        checked = projectSettings.allowRiskyModelSelects,
        enabled = actionsEnabled && selectedProviderReady && projectSettings.projectId.isNotBlank(),
        onDisabledClick = if (actionsEnabled && !selectedProviderReady) showModelMissingMessage else null,
        onCheckedChange = { onSaveSettings(projectSettings.copy(allowRiskyModelSelects = it)) },
    )
}

@Composable
private fun MissingModelProviderNotice(
    enabled: Boolean,
    onConfigureModelProvider: () -> Unit,
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = ElementBlue.copy(alpha = 0.08f),
        shape = elementShape,
        border = BorderStroke(1.dp, ElementBlue.copy(alpha = 0.35f)),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(2.dp),
            ) {
                Text(
                    "需要先配置模型服务",
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    "自动评价和模型优选会使用当前项目选择的模型。",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                )
            }
            OutlinedButton(
                onClick = onConfigureModelProvider,
                enabled = enabled,
                shape = elementShape,
                contentPadding = PaddingValues(horizontal = 12.dp, vertical = 0.dp),
            ) {
                Text("去配置", fontSize = 13.sp)
            }
        }
    }
}

@Composable
private fun ProjectScenePanel(
    projectSettings: ProjectEvaluationSettingsUi,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
) {
    val selectedThresholdMode = selectedCvThresholdMode(projectSettings)
    OptionRow(
        title = "风险阈值",
        values = listOf("loose", "standard", "strict", "custom"),
        selected = selectedThresholdMode,
        enabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
        labelForValue = ::cvThresholdModeLabel,
        onSelected = { onSaveSettings(projectSettingsAfterCvThresholdModeSelection(projectSettings, it)) },
    )
    Text(
        if (selectedThresholdMode == "custom") {
            "使用自定义阈值；切回预设会停用自定义值。"
        } else {
            cvPolicyHint(projectSettings.cvPolicy)
        },
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        style = MaterialTheme.typography.bodySmall,
    )
    CvPolicyAdvancedControls(
        projectSettings = projectSettings,
        actionsEnabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
        onSaveSettings = onSaveSettings,
    )
}

@Composable
private fun ProjectPromptPanel(
    projectSettings: ProjectEvaluationSettingsUi,
    selectedPrompt: PromptPackUi?,
    promptPacks: List<PromptPackUi>,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
) {
    if (promptPacks.isEmpty()) {
        Text("还没有可用提示词。", color = MaterialTheme.colorScheme.onSurfaceVariant)
        return
    }
    var collapsedPackages by rememberSaveable(
        projectSettings.projectId,
        promptPacks.joinToString("|") { it.promptPackId },
    ) {
        mutableStateOf(emptyList<String>())
    }
    val packages = promptPacks
        .groupBy { promptPackageFolder(it) }
        .toList()
        .sortedWith(
            compareBy<Pair<String, List<PromptPackUi>>> {
                when (it.first) {
                    "user" -> 0
                    "builtin" -> 2
                    else -> 1
                }
            }.thenBy { promptPackageLabel(it.first) },
        )
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        packages.forEach { (packageFolder, packsInPackage) ->
            val expanded = packageFolder !in collapsedPackages
            PromptSelectionPackageSection(
                packageFolder = packageFolder,
                profiles = packsInPackage.sortedBy { promptPackDisplayName(it) },
                selectedPrompt = selectedPrompt,
                expanded = expanded,
                enabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
                onToggle = {
                    collapsedPackages = if (expanded) {
                        collapsedPackages + packageFolder
                    } else {
                        collapsedPackages - packageFolder
                    }
                },
                onSelected = { profile ->
                    onSaveSettings(projectSettings.copy(promptPackId = profile.promptPackId))
                },
            )
        }
    }
}

@Composable
private fun PromptSelectionPackageSection(
    packageFolder: String,
    profiles: List<PromptPackUi>,
    selectedPrompt: PromptPackUi?,
    expanded: Boolean,
    enabled: Boolean,
    onToggle: () -> Unit,
    onSelected: (PromptPackUi) -> Unit,
) {
    Surface(
        color = ElementControlSurface,
        shape = elementShape,
        border = BorderStroke(1.dp, ElementBorder),
        modifier = Modifier.fillMaxWidth(),
    ) {
        Column(Modifier.fillMaxWidth()) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable(onClick = onToggle)
                    .padding(horizontal = 14.dp, vertical = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    imageVector = if (expanded) Icons.Outlined.KeyboardArrowDown else Icons.AutoMirrored.Outlined.KeyboardArrowRight,
                    contentDescription = if (expanded) "收起提示词包" else "展开提示词包",
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(20.dp),
                )
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(
                        promptPackageLabel(packageFolder),
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        "${profiles.size} 个提示词",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
            if (expanded) {
                HorizontalDivider(
                    color = MaterialTheme.colorScheme.outline.copy(alpha = 0.35f),
                    thickness = 1.dp,
                )
                profiles.forEachIndexed { index, profile ->
                    PromptSelectionRow(
                        profile = profile,
                        selected = profile.promptPackId == selectedPrompt?.promptPackId,
                        enabled = enabled,
                        onClick = { onSelected(profile) },
                    )
                    if (index != profiles.lastIndex) {
                        HorizontalDivider(
                            modifier = Modifier.padding(start = 14.dp),
                            color = MaterialTheme.colorScheme.outline.copy(alpha = 0.2f),
                            thickness = 1.dp,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun PromptSelectionRow(
    profile: PromptPackUi,
    selected: Boolean,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = enabled, onClick = onClick)
            .background(if (selected) ElementBlue.copy(alpha = 0.14f) else Color.Transparent)
            .padding(horizontal = 14.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Text(
                promptPackDisplayName(profile),
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                promptPackMetaText(profile),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        if (selected) {
            Surface(color = ElementBlue, shape = CircleShape) {
                Text(
                    "当前",
                    modifier = Modifier.padding(horizontal = 9.dp, vertical = 4.dp),
                    color = ElementOnAccent,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.SemiBold,
                )
            }
        }
    }
}

@Composable
private fun ProjectSelectableRow(
    title: String,
    subtitle: String,
    selected: Boolean,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Surface(
        color = if (selected) ElementBlue.copy(alpha = 0.14f) else ElementControlSurface,
        shape = elementShape,
        border = BorderStroke(1.dp, if (selected) ElementBlue else ElementBorder),
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = enabled, onClick = onClick),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 14.dp, vertical = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                Text(title, style = MaterialTheme.typography.titleMedium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(
                    subtitle,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            if (selected) {
                Surface(color = ElementBlue, shape = CircleShape) {
                    Text(
                        "当前",
                        modifier = Modifier.padding(horizontal = 9.dp, vertical = 4.dp),
                        color = ElementOnAccent,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            }
        }
    }
}

private fun projectIntelligencePanelTitle(panel: ProjectIntelligencePanel): String =
    when (panel) {
        ProjectIntelligencePanel.Model -> "模型服务"
        ProjectIntelligencePanel.Workflow -> "自动流程"
        ProjectIntelligencePanel.Scene -> "技术风险阈值"
        ProjectIntelligencePanel.Prompt -> "评价提示词"
    }

private fun projectIntelligencePanelSubtitle(panel: ProjectIntelligencePanel): String =
    when (panel) {
        ProjectIntelligencePanel.Model -> "选择当前项目使用的模型配置"
        ProjectIntelligencePanel.Workflow -> "控制上传后评价与连拍优选"
        ProjectIntelligencePanel.Scene -> "设置本项目的本地风险检测灵敏度"
        ProjectIntelligencePanel.Prompt -> "选择模型评价使用的摄影偏好"
    }

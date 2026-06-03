package com.cameraconnector.app.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.graphics.Bitmap
import androidx.activity.compose.BackHandler
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
import androidx.compose.foundation.layout.defaultMinSize
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
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
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
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.EvaluationRunUi
import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetQuery
import com.cameraconnector.app.core.InboxAssetRole
import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PromptProfileUi
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
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
internal fun SettingsScreen(
    dashboard: DashboardState,
    notificationPermissionRequired: Boolean,
    notificationPermissionGranted: Boolean,
    onRequestNotificationPermission: () -> Unit,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    selectedInboxLabel: String?,
    onChooseInboxDirectory: () -> Unit,
    onOpenDiagnostics: () -> Unit,
    onOpenPromptProfiles: () -> Unit,
    projectPhotoGridColumnCount: Int,
    onProjectPhotoGridColumnCountChange: (Int) -> Unit,
    modelProviderSettings: ModelProviderSettingsUi = ModelProviderSettingsUi(),
    onSaveModelProviderSettings: (ModelProviderSettingsUi) -> Unit = {},
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Text("系统设置", style = MaterialTheme.typography.headlineMedium)
            Spacer(Modifier.height(4.dp))
            Text("接收、存储与通知权限", color = MaterialTheme.colorScheme.onSurfaceVariant)
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        actionInFlight?.let { action ->
            item { ProcessingCard(action) }
        }

        item {
            Text("工具", style = MaterialTheme.typography.titleMedium)
        }
        item {
            SettingsMenuRow(
                title = "诊断日志",
                subtitle = "连接、传输和发布状态",
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
                            "${projectPhotoGridColumnCount}列",
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
            Text("智能优选", style = MaterialTheme.typography.titleMedium)
        }
        item {
            SettingsMenuRow(
                title = "提示词配置",
                subtitle = "全局评价偏好，协议由系统锁定",
                trailing = ">",
                onClick = onOpenPromptProfiles,
            )
        }
        item {
            ModelProviderSettingsCard(
                settings = modelProviderSettings,
                actionsEnabled = actionInFlight == null,
                onSaveSettings = onSaveModelProviderSettings,
            )
        }
        if (notificationPermissionRequired && !notificationPermissionGranted) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text("通知权限", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(8.dp))
                        Text("启动接收服务前需要允许通知。")
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
                title = "外部文件夹授权",
                subtitle = selectedInboxLabel ?: "未授权",
                trailing = ">",
                onClick = onChooseInboxDirectory,
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
internal fun PromptProfilesScreen(
    promptProfiles: List<PromptProfileUi>,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onOpenPromptProfile: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconButton(onClick = onBack) {
                    Icon(Icons.AutoMirrored.Outlined.ArrowBack, contentDescription = "返回设置")
                }
                Column(Modifier.weight(1f)) {
                    Text("提示词配置", style = MaterialTheme.typography.headlineSmall)
                    Spacer(Modifier.height(4.dp))
                    Text("只编辑评价偏好，输入输出协议由系统锁定", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        actionInFlight?.let { action ->
            item { ProcessingCard(action) }
        }

        if (promptProfiles.isEmpty()) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "暂无提示词配置。",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            items(promptProfiles, key = { it.promptProfileId }) { profile ->
                PromptProfileRow(
                    profile = profile,
                    onClick = { onOpenPromptProfile(profile.promptProfileId) },
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
    }
}

@Composable
private fun PromptProfileRow(
    profile: PromptProfileUi,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ElementCard(modifier = modifier.clickable(onClick = onClick)) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text(
                    promptProfileDisplayName(profile),
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Spacer(Modifier.height(4.dp))
                Text(
                    listOf(
                        promptStyleTagsText(profile),
                        if (profile.builtIn) "内置偏好，只能复制后编辑" else "自定义偏好，可编辑",
                    ).filter { it.isNotBlank() }.joinToString(" · "),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Spacer(Modifier.width(12.dp))
            Text("编辑", color = ElementBlue, fontWeight = FontWeight.SemiBold)
        }
    }
}

@Composable
internal fun PromptProfileEditorScreen(
    profile: PromptProfileUi?,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onSave: (PromptProfileUi, String, String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var name by remember(profile?.promptProfileId) {
        mutableStateOf(profile?.let(::promptProfileDisplayName)?.let { "自定义 $it" } ?: "")
    }
    var promptText by remember(profile?.promptProfileId, profile?.activePromptText) {
        mutableStateOf(profile?.activePromptText.orEmpty())
    }
    val cleanName = name.trim()
    val cleanPrompt = promptText.trim()

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconButton(onClick = onBack) {
                    Icon(Icons.AutoMirrored.Outlined.ArrowBack, contentDescription = "返回提示词列表")
                }
                Column(Modifier.weight(1f)) {
                    Text(profile?.let(::promptProfileDisplayName) ?: "编辑提示词", style = MaterialTheme.typography.headlineSmall)
                    Spacer(Modifier.height(4.dp))
                    Text(
                        if (profile?.builtIn == true) "内置评价偏好会复制为全局自定义偏好" else "保存后作为新的全局评价偏好版本",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        actionInFlight?.let { action ->
            item { ProcessingCard(action) }
        }

        if (profile == null) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "提示词不存在或已被移除。",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            if (profile.builtIn) {
                item {
                    OutlinedTextField(
                        value = name,
                        onValueChange = { name = it },
                        label = { Text("复制后的名称") },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                    )
                }
            }
            item {
                OutlinedTextField(
                    value = promptText,
                    onValueChange = { promptText = it },
                    label = { Text("用户评价偏好") },
                    modifier = Modifier.fillMaxWidth(),
                    minLines = 10,
                    enabled = actionInFlight == null,
                )
            }
            item {
                Button(
                    onClick = { onSave(profile, cleanName, cleanPrompt) },
                    enabled = actionInFlight == null && cleanPrompt.isNotBlank() &&
                        (!profile.builtIn || cleanName.isNotBlank()),
                    modifier = Modifier.fillMaxWidth(),
                    shape = elementShape,
                    colors = ButtonDefaults.buttonColors(
                        containerColor = ElementBlue,
                        contentColor = ElementOnAccent,
                    ),
                ) {
                    Text(if (profile.builtIn) "复制并保存偏好" else "保存偏好版本")
                }
            }
        }
    }
}

@Composable
internal fun ProjectSettingsScreen(
    project: ProjectSummary?,
    provider: ModelProviderSettingsUi,
    settings: ProjectEvaluationSettingsUi?,
    promptProfiles: List<PromptProfileUi>,
    latestRun: EvaluationRunUi?,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
    onGenerateProjectRecommendation: () -> Unit,
    onConfigureModelProvider: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val projectSettings = settings ?: project?.let { ProjectEvaluationSettingsUi(projectId = it.id) }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconButton(onClick = onBack) {
                    Icon(Icons.AutoMirrored.Outlined.ArrowBack, contentDescription = "返回项目管理")
                }
                Column(Modifier.weight(1f)) {
                    Text(project?.name ?: "项目配置", style = MaterialTheme.typography.headlineSmall)
                    Spacer(Modifier.height(4.dp))
                    Text("项目智能、场景和优选策略", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        actionInFlight?.let { action ->
            item { ProcessingCard(action) }
        }

        if (projectSettings == null) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "项目不存在或已被移除。",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            item {
                ProjectIntelligenceSettingsCard(
                    provider = provider,
                    settings = projectSettings,
                    promptProfiles = promptProfiles,
                    latestRun = latestRun,
                    actionsEnabled = actionInFlight == null,
                    onSaveSettings = onSaveSettings,
                    onGenerateProjectRecommendation = onGenerateProjectRecommendation,
                    onConfigureModelProvider = onConfigureModelProvider,
                )
            }
        }
    }
}

@Composable
private fun ModelProviderSettingsCard(
    settings: ModelProviderSettingsUi,
    actionsEnabled: Boolean,
    onSaveSettings: (ModelProviderSettingsUi) -> Unit,
) {
    var providerKind by remember(settings.providerKind) { mutableStateOf(settings.providerKind.ifBlank { "none" }) }
    var baseUrl by remember(settings.baseUrl) { mutableStateOf(settings.baseUrl) }
    var model by remember(settings.defaultModel) { mutableStateOf(settings.defaultModel) }
    var apiKey by remember(settings.apiKeyConfigured) { mutableStateOf("") }
    var sendMode by remember(settings.defaultSendMode) {
        mutableStateOf(settings.defaultSendMode.ifBlank { "preview_only" })
    }
    var batchSize by remember(settings.defaultBatchSize) {
        mutableStateOf(providerBatchSizeValue(settings.defaultBatchSize))
    }
    val configuredText = if (settings.configured) "已配置" else "未配置"
    val normalizedProviderKind = providerKind.trim().lowercase().ifBlank { "none" }
    val canSaveProvider = normalizedProviderKind == "none" ||
        (baseUrl.trim().isNotBlank() &&
            model.trim().isNotBlank() &&
            (apiKey.trim().isNotBlank() || settings.apiKeyConfigured || settings.keyAlias != null))

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
                    Text("全局模型服务", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(4.dp))
                    Text(
                        "${modelProviderKindLabel(settings.providerKind, settings.providerLabel)} · $configuredText",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                ElementTag(
                    text = modelProviderKindLabel(providerKind, settings.providerLabel),
                    color = if (settings.configured) ElementBlue else ElementBorder,
                )
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf("none", "openai", "custom").forEach { kind ->
                    FilterChipButton(
                        label = modelProviderKindLabel(kind, settings.providerLabel),
                        selected = normalizedProviderKind == kind,
                        onClick = {
                            providerKind = kind
                            if (kind == "openai" && baseUrl.isBlank()) {
                                baseUrl = "https://api.openai.com/v1"
                            }
                        },
                    )
                }
            }
            OutlinedTextField(
                value = baseUrl,
                onValueChange = { baseUrl = it },
                label = { Text("Base URL") },
                modifier = Modifier.fillMaxWidth(),
                enabled = normalizedProviderKind != "none",
                singleLine = true,
            )
            OutlinedTextField(
                value = model,
                onValueChange = { model = it },
                label = { Text("模型名称") },
                modifier = Modifier.fillMaxWidth(),
                enabled = normalizedProviderKind != "none",
                singleLine = true,
            )
            OutlinedTextField(
                value = apiKey,
                onValueChange = { apiKey = it },
                label = { Text("API Key") },
                supportingText = {
                    Text(if (settings.apiKeyConfigured) "已保存密钥，留空不会覆盖" else "仅保存到本机配置，不回显明文")
                },
                modifier = Modifier.fillMaxWidth(),
                enabled = normalizedProviderKind != "none",
                singleLine = true,
                visualTransformation = PasswordVisualTransformation(),
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf("preview_only", "review_image").forEach { mode ->
                    FilterChipButton(
                        label = modelSendModeLabel(mode),
                        selected = sendMode == mode,
                        onClick = { sendMode = mode },
                    )
                }
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("批处理数量", color = MaterialTheme.colorScheme.onSurfaceVariant)
                BatchSizeToggle(
                    batchSize = batchSize,
                    onBatchSizeChange = { batchSize = it },
                )
            }
            Button(
                onClick = {
                    val providerEnabled = normalizedProviderKind != "none"
                    onSaveSettings(
                        settings.copy(
                            providerKind = normalizedProviderKind,
                            providerLabel = modelProviderKindLabel(normalizedProviderKind, settings.providerLabel),
                            baseUrl = if (providerEnabled) baseUrl.trim() else "",
                            defaultModel = if (providerEnabled) model.trim() else "",
                            defaultSendMode = sendMode,
                            defaultBatchSize = batchSize,
                            configured = providerEnabled && canSaveProvider,
                            apiKey = apiKey.trim().takeIf { it.isNotBlank() },
                        ),
                    )
                },
                enabled = actionsEnabled && canSaveProvider,
                modifier = Modifier.fillMaxWidth(),
                shape = elementShape,
            ) {
                Text("保存模型服务配置")
            }
        }
    }
}

private fun modelProviderKindLabel(kind: String, fallback: String): String =
    when (kind.trim().lowercase()) {
        "", "none" -> "未配置"
        "openai" -> "OpenAI"
        "custom" -> "自定义"
        "mock", "local_stub" -> "本地占位"
        else -> fallback
            .takeUnless { it.equals("Model provider", ignoreCase = true) }
            ?.ifBlank { kind }
            ?: "模型服务"
    }

private fun modelSendModeLabel(mode: String): String =
    when (mode.trim().lowercase()) {
        "preview_only" -> "仅发送预览"
        "review_image" -> "发送审阅图"
        else -> mode
    }

@Composable
private fun ProjectIntelligenceSettingsCard(
    provider: ModelProviderSettingsUi,
    settings: ProjectEvaluationSettingsUi?,
    promptProfiles: List<PromptProfileUi>,
    latestRun: EvaluationRunUi?,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
    onGenerateProjectRecommendation: () -> Unit,
    onConfigureModelProvider: () -> Unit,
) {
    val projectSettings = settings ?: ProjectEvaluationSettingsUi(projectId = "")
    val intelligenceUi = projectIntelligenceSettingsUi(projectSettings, providerConfigured = provider.configured)
    val selectablePromptProfiles = promptProfiles
        .filter { it.enabled && (it.scope.equals("global", ignoreCase = true) || it.projectId == null) }
        .ifEmpty { promptProfiles.filter { it.enabled } }
    val selectedPrompt = selectablePromptProfiles.firstOrNull { it.promptProfileId == projectSettings.promptProfileId }
        ?: selectablePromptProfiles.firstOrNull()
    val recommendationAction = manualProjectRecommendationActionUi(
        provider = provider,
        settings = projectSettings,
        actionInFlight = !actionsEnabled,
    )

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
                    Text("项目智能设置", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(4.dp))
                    Text(
                        latestRun?.let { "最近项目优选：${evaluationRunStatusLabel(it.status)}" }
                            ?: "项目优选：手动触发",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                Switch(
                    checked = intelligenceUi.modelEvaluationEnabled,
                    enabled = actionsEnabled && intelligenceUi.modelEvaluationToggleEnabled &&
                        projectSettings.projectId.isNotBlank(),
                    onCheckedChange = { enabled ->
                        onSaveSettings(projectSettings.copy(modelEvaluationEnabled = enabled))
                    },
                )
            }

            if (!provider.configured) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text("模型服务未配置", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    OutlinedButton(
                        onClick = onConfigureModelProvider,
                        enabled = actionsEnabled,
                        shape = elementShape,
                    ) {
                        Text("去配置")
                    }
                }
            }

            SettingsSwitchRow(
                title = "上传后自动模型评价",
                checked = projectSettings.autoEvaluateOnUpload,
                enabled = actionsEnabled && provider.configured && projectSettings.modelEvaluationEnabled,
                onCheckedChange = { onSaveSettings(projectSettings.copy(autoEvaluateOnUpload = it)) },
            )
            SettingsSwitchRow(
                title = "连拍组自动优选",
                checked = projectSettings.autoBurstRecommendationEnabled,
                enabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
                onCheckedChange = { onSaveSettings(projectSettings.copy(autoBurstRecommendationEnabled = it)) },
            )
            SettingsSwitchRow(
                title = "允许风险照片参与优选",
                checked = projectSettings.allowRiskyModelSelects,
                enabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
                onCheckedChange = { onSaveSettings(projectSettings.copy(allowRiskyModelSelects = it)) },
            )

            OptionRow(
                title = "项目场景",
                values = listOf("general", "portrait", "action", "landscape", "custom"),
                selected = projectSettings.sceneProfile,
                enabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
                labelForValue = ::sceneProfileLabel,
                onSelected = { onSaveSettings(projectSettings.copy(sceneProfile = it)) },
            )
            OptionRow(
                title = "CV 门控强度",
                values = listOf("loose", "standard", "strict"),
                selected = projectSettings.cvPolicy,
                enabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
                labelForValue = ::cvPolicyLabel,
                onSelected = { onSaveSettings(projectSettings.copy(cvPolicy = it)) },
            )

            if (selectablePromptProfiles.isNotEmpty()) {
                Text("评价提示词", style = MaterialTheme.typography.labelLarge)
                LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    items(selectablePromptProfiles, key = { it.promptProfileId }) { profile ->
                        FilterChipButton(
                            label = listOf(promptProfileDisplayName(profile), promptStyleTagsText(profile))
                                .filter { it.isNotBlank() }
                                .joinToString(" · "),
                            selected = profile.promptProfileId == selectedPrompt?.promptProfileId,
                            onClick = {
                                onSaveSettings(projectSettings.copy(promptProfileId = profile.promptProfileId))
                            },
                        )
                    }
                }
            }

            Button(
                onClick = onGenerateProjectRecommendation,
                enabled = actionsEnabled && recommendationAction.enabled,
                modifier = Modifier.fillMaxWidth(),
                shape = elementShape,
            ) {
                Text(recommendationAction.ctaLabel)
            }
            recommendationAction.disabledReason?.let {
                Text(it, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

@Composable
private fun BatchSizeToggle(
    batchSize: Int,
    onBatchSizeChange: (Int) -> Unit,
) {
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        listOf(1, 2, 4, 8).forEach { value ->
            OutlinedButton(
                onClick = { onBatchSizeChange(value) },
                modifier = Modifier
                    .height(30.dp)
                    .defaultMinSize(minWidth = 1.dp, minHeight = 1.dp),
                border = BorderStroke(1.dp, if (batchSize == value) ElementBlue else ElementBorder),
                colors = ButtonDefaults.outlinedButtonColors(
                    containerColor = if (batchSize == value) ElementBlue else ElementControlSurface,
                    contentColor = if (batchSize == value) ElementOnAccent else MaterialTheme.colorScheme.onSurfaceVariant,
                ),
                shape = elementShape,
                contentPadding = PaddingValues(horizontal = 9.dp, vertical = 0.dp),
            ) {
                Text(value.toString(), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            }
        }
    }
}

@Composable
private fun SettingsSwitchRow(
    title: String,
    checked: Boolean,
    enabled: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(title)
        Switch(checked = checked, enabled = enabled, onCheckedChange = onCheckedChange)
    }
}

@Composable
private fun OptionRow(
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

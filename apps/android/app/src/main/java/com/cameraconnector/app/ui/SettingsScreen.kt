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
import androidx.compose.material3.Slider
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
import androidx.compose.runtime.saveable.rememberSaveable
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
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import com.cameraconnector.app.core.ProjectAssetRole
import com.cameraconnector.app.core.ModelProviderSettingsUi
import com.cameraconnector.app.core.PromptProfileUi
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.PublishQueueState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.core.TechnicalAssessmentPolicyUi
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
import java.util.Locale
import kotlin.math.roundToInt

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
    onOpenPromptProfiles: () -> Unit,
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
                subtitle = "\u8fde\u63a5\u3001\u4f20\u8f93\u548c\u53d1\u5e03\u72b6\u6001",
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
                onClick = onOpenPromptProfiles,
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
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconButton(onClick = ::closeEditorOrScreen) {
                    Icon(
                        Icons.AutoMirrored.Outlined.ArrowBack,
                        contentDescription = if (editorOpen) "返回模型服务列表" else "返回设置",
                    )
                }
                Column(Modifier.weight(1f)) {
                    Text(
                        when {
                            creatingProvider -> "新建模型服务"
                            editingProvider != null -> "编辑模型服务"
                            else -> "模型服务"
                        },
                        style = MaterialTheme.typography.headlineSmall,
                    )
                }
                if (!editorOpen) {
                    OutlinedButton(
                        onClick = { creatingProvider = true },
                        enabled = actionInFlight == null,
                        shape = elementShape,
                    ) {
                        Text("新建")
                    }
                }
            }
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        actionInFlight?.let { action ->
            item { ProcessingCard(action) }
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
internal fun PromptProfilesScreen(
    promptProfiles: List<PromptProfileUi>,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onCreatePromptProfile: () -> Unit,
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
                    Text("\u63d0\u793a\u8bcd\u914d\u7f6e", style = MaterialTheme.typography.headlineSmall)
                    Spacer(Modifier.height(4.dp))
                    Text("管理摄影评价偏好", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                OutlinedButton(
                    onClick = onCreatePromptProfile,
                    enabled = actionInFlight == null,
                    shape = elementShape,
                ) {
                    Text("新建")
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
                        "\u6682\u65e0\u63d0\u793a\u8bcd\u914d\u7f6e",
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
                        if (profile.builtIn) "\u5185\u7f6e\u504f\u597d\uff0c\u53ea\u80fd\u590d\u5236\u540e\u7f16\u8f91" else "\u81ea\u5b9a\u4e49\u504f\u597d\uff0c\u53ef\u7f16\u8f91",
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
    onCreate: (String, List<String>, String, String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val createMode = profile == null
    val builtInProfile = profile?.builtIn == true
    var name by remember(profile?.promptProfileId) {
        mutableStateOf(
            when {
                createMode -> ""
                builtInProfile -> profile.let(::promptProfileDisplayName).let { "自定义 $it" }
                else -> profile.let(::promptProfileDisplayName)
            },
        )
    }
    var styleTagsText by remember(profile?.promptProfileId) {
        mutableStateOf(
            profile?.styleTags
                ?.filter { it.isNotBlank() }
                ?.joinToString(" ") { promptStyleTagLabel(it) }
                ?: "通用 均衡",
        )
    }
    var sceneProfile by remember(profile?.promptProfileId) {
        mutableStateOf(profile?.sceneProfile?.ifBlank { "general" } ?: "general")
    }
    var promptText by remember(profile?.promptProfileId, profile?.sharedPreference, profile?.activePromptText) {
        mutableStateOf(profile?.sharedPreference ?: profile?.activePromptText.orEmpty())
    }
    val cleanName = name.trim()
    val cleanPrompt = promptText.trim()
    val cleanStyleTags = parsePromptStyleTags(styleTagsText)
    val actionsEnabled = actionInFlight == null
    val canSubmit = actionsEnabled && cleanPrompt.isNotBlank() &&
        when {
            createMode -> cleanName.isNotBlank()
            builtInProfile -> cleanName.isNotBlank()
            else -> true
        }

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
                    Icon(Icons.AutoMirrored.Outlined.ArrowBack, contentDescription = "\u8fd4\u56de\u63d0\u793a\u8bcd\u5217\u8868")
                }
                Column(Modifier.weight(1f)) {
                    Text(
                        when {
                            createMode -> "新建提示词"
                            else -> profile.let(::promptProfileDisplayName)
                        },
                        style = MaterialTheme.typography.headlineSmall,
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(
                        when {
                            createMode -> "创建一套全局摄影偏好，项目内按需选择"
                            builtInProfile -> "内置偏好会复制为全局自定义偏好"
                            else -> "保存后成为这套全局偏好的新版本"
                        },
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

        if (createMode || builtInProfile) {
            item {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text(if (createMode) "提示词名称" else "复制后的名称") },
                    modifier = Modifier.fillMaxWidth(),
                    enabled = actionsEnabled,
                    singleLine = true,
                )
            }
        }

        if (createMode) {
            item {
                OutlinedTextField(
                    value = styleTagsText,
                    onValueChange = { styleTagsText = it },
                    label = { Text("风格标签") },
                    supportingText = { Text("用空格或逗号分隔，例如：通用 均衡 人像") },
                    modifier = Modifier.fillMaxWidth(),
                    enabled = actionsEnabled,
                    singleLine = true,
                )
            }
            item {
                OptionRow(
                    title = "适用场景",
                    values = listOf("general", "portrait", "action", "landscape", "custom"),
                    selected = sceneProfile,
                    enabled = actionsEnabled,
                    labelForValue = ::sceneProfileLabel,
                    onSelected = { sceneProfile = it },
                )
            }
        } else {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(
                        modifier = Modifier.padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        Text("提示词信息", style = MaterialTheme.typography.titleMedium)
                        Text(
                            listOf(
                                profile.let(::promptStyleTagsText),
                                sceneProfileLabel(profile.sceneProfile),
                                if (builtInProfile) "内置" else "自定义",
                            ).filter { it.isNotBlank() }.joinToString(" · "),
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
        }

        item {
            OutlinedTextField(
                value = promptText,
                onValueChange = { promptText = it },
                label = { Text("我的摄影评价偏好") },
                supportingText = {
                    Text("这里会同时影响单张评价、连拍组内优选和项目优选的审美偏好")
                },
                modifier = Modifier.fillMaxWidth(),
                minLines = 10,
                enabled = actionsEnabled,
            )
        }
        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text("应用会保持输出格式稳定", style = MaterialTheme.typography.titleMedium)
                    Text(
                        "这里只编辑你的摄影偏好；评分字段、优选结果和 JSON 输出格式由应用自动生成。",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
        item {
            Button(
                onClick = {
                    if (createMode) {
                        onCreate(
                            cleanName,
                            cleanStyleTags.ifEmpty { listOf("general", "balanced") },
                            sceneProfile,
                            cleanPrompt,
                        )
                    } else {
                        onSave(profile, cleanName, cleanPrompt)
                    }
                },
                enabled = canSubmit,
                modifier = Modifier.fillMaxWidth(),
                shape = elementShape,
                colors = ButtonDefaults.buttonColors(
                    containerColor = ElementBlue,
                    contentColor = ElementOnAccent,
                ),
            ) {
                Text(
                    when {
                        createMode -> "创建提示词"
                        builtInProfile -> "复制并保存偏好"
                        else -> "保存偏好版本"
                    },
                )
            }
        }
    }
}

private fun parsePromptStyleTags(value: String): List<String> =
    value
        .split(' ', '\n', '\t', ',', '，', '/', '、')
        .map { it.trim() }
        .filter { it.isNotBlank() }
        .distinct()

@Composable
internal fun ProjectSettingsScreen(
    project: ProjectSummary?,
    providerOptions: List<ModelProviderSettingsUi>,
    settings: ProjectEvaluationSettingsUi?,
    promptProfiles: List<PromptProfileUi>,
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
                    Text("\u9879\u76ee\u667a\u80fd\u3001\u573a\u666f\u548c\u4f18\u9009\u7b56\u7565", color = MaterialTheme.colorScheme.onSurfaceVariant)
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
                        "项目不存在或已被移除",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            item {
                ProjectIntelligenceSettingsCard(
                    providerOptions = providerOptions,
                    settings = projectSettings,
                    promptProfiles = promptProfiles,
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
    settingsList: List<ModelProviderSettingsUi>,
    actionsEnabled: Boolean,
    onSaveSettings: (ModelProviderSettingsUi) -> Unit,
    onDeleteSettings: (String) -> Unit,
) {
    val providerOptions = settingsList
        .filter { it.configured || it.settingsId.isNotBlank() || it.defaultModel.isNotBlank() }
        .distinctBy { it.settingsId }
    var selectedSettingsId by remember(providerOptions.map { it.settingsId }.joinToString("|")) {
        mutableStateOf(providerOptions.firstOrNull()?.settingsId ?: settings.settingsId)
    }
    var editingNewProfile by remember { mutableStateOf(providerOptions.isEmpty()) }
    val selectedSettings = if (editingNewProfile) {
        ModelProviderSettingsUi(
            settingsId = "",
            providerKind = "custom",
            providerLabel = "\u81ea\u5b9a\u4e49",
            defaultSendMode = settings.defaultSendMode.ifBlank { "preview_only" },
            defaultBatchSize = providerBatchSizeValue(settings.defaultBatchSize),
        )
    } else {
        providerOptions.firstOrNull { it.settingsId == selectedSettingsId } ?: settings
    }
    var settingsName by remember(selectedSettings.settingsId, editingNewProfile) {
        mutableStateOf(selectedSettings.settingsId.takeUnless { editingNewProfile }.orEmpty())
    }
    var providerKind by remember(selectedSettings.settingsId, selectedSettings.providerKind, editingNewProfile) {
        mutableStateOf(selectedSettings.providerKind.ifBlank { "none" })
    }
    var baseUrl by remember(selectedSettings.settingsId, selectedSettings.baseUrl, editingNewProfile) {
        mutableStateOf(selectedSettings.baseUrl)
    }
    var model by remember(selectedSettings.settingsId, selectedSettings.defaultModel, editingNewProfile) {
        mutableStateOf(selectedSettings.defaultModel)
    }
    var apiKey by remember(selectedSettings.settingsId, selectedSettings.apiKeyConfigured, editingNewProfile) {
        mutableStateOf("")
    }
    var sendMode by remember(selectedSettings.settingsId, selectedSettings.defaultSendMode, editingNewProfile) {
        mutableStateOf(selectedSettings.defaultSendMode.ifBlank { "preview_only" })
    }
    var batchSize by remember(selectedSettings.settingsId, selectedSettings.defaultBatchSize, editingNewProfile) {
        mutableStateOf(providerBatchSizeValue(selectedSettings.defaultBatchSize))
    }
    val normalizedProviderKind = providerKind.trim().lowercase().ifBlank { "none" }
    val cleanSettingsName = settingsName.trim()
    val canSaveProvider = cleanSettingsName.isNotBlank() &&
        normalizedProviderKind != "none" &&
        (baseUrl.trim().isNotBlank() &&
            model.trim().isNotBlank() &&
            (apiKey.trim().isNotBlank() || selectedSettings.apiKeyConfigured || selectedSettings.keyAlias != null))
    val canDeleteProvider = !editingNewProfile &&
        selectedSettings.settingsId.isNotBlank() &&
        providerOptions.any { it.settingsId == selectedSettings.settingsId }

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
                    Text("模型配置", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(4.dp))
                    Text(
                        if (editingNewProfile) "填写服务地址、模型和密钥" else "编辑当前模型服务",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (providerOptions.size > 1) {
                    OutlinedButton(
                        onClick = { editingNewProfile = true },
                        enabled = actionsEnabled,
                        shape = elementShape,
                    ) {
                        Text("新建")
                    }
                }
            }
            if (providerOptions.size > 1) {
                LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    items(providerOptions, key = { it.settingsId }) { option ->
                        FilterChipButton(
                            label = modelProviderOptionLabel(option),
                            selected = !editingNewProfile && option.settingsId == selectedSettingsId,
                            onClick = {
                                editingNewProfile = false
                                selectedSettingsId = option.settingsId
                            },
                        )
                    }
                }
            }
            OutlinedTextField(
                value = settingsName,
                onValueChange = { settingsName = it },
                label = { Text("配置名称") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf("openai", "custom").forEach { kind ->
                    FilterChipButton(
                        label = modelProviderKindLabel(kind, selectedSettings.providerLabel),
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
                label = { Text("服务地址") },
                modifier = Modifier.fillMaxWidth(),
                enabled = true,
                singleLine = true,
            )
            OutlinedTextField(
                value = model,
                onValueChange = { model = it },
                label = { Text("模型名称") },
                modifier = Modifier.fillMaxWidth(),
                enabled = true,
                singleLine = true,
            )
            OutlinedTextField(
                value = apiKey,
                onValueChange = { apiKey = it },
                label = { Text("API 密钥") },
                supportingText = {
                    Text(if (selectedSettings.apiKeyConfigured) "已保存密钥，留空不会覆盖" else "仅保存到本机配置，不回显明文")
                },
                modifier = Modifier.fillMaxWidth(),
                enabled = true,
                singleLine = true,
                visualTransformation = PasswordVisualTransformation(),
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf("preview_only", "detail_image").forEach { mode ->
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
                Text("\u6279\u5904\u7406\u6570\u91cf", color = MaterialTheme.colorScheme.onSurfaceVariant)
                BatchSizeToggle(
                    batchSize = batchSize,
                    onBatchSizeChange = { batchSize = it },
                )
            }
            Button(
                onClick = {
                    onSaveSettings(
                        selectedSettings.copy(
                            settingsId = cleanSettingsName,
                            providerKind = normalizedProviderKind,
                            providerLabel = modelProviderKindLabel(normalizedProviderKind, selectedSettings.providerLabel),
                            baseUrl = baseUrl.trim(),
                            defaultModel = model.trim(),
                            defaultSendMode = sendMode,
                            defaultBatchSize = batchSize,
                            configured = true,
                            apiKey = apiKey.trim().takeIf { it.isNotBlank() },
                        ),
                    )
                    editingNewProfile = false
                    selectedSettingsId = cleanSettingsName
                },
                enabled = actionsEnabled && canSaveProvider,
                modifier = Modifier.fillMaxWidth(),
                shape = elementShape,
            ) {
                Text(if (editingNewProfile) "创建模型配置" else "保存模型配置")
            }
            if (canDeleteProvider) {
                OutlinedButton(
                    onClick = { onDeleteSettings(selectedSettings.settingsId) },
                    enabled = actionsEnabled,
                    modifier = Modifier.fillMaxWidth(),
                    shape = elementShape,
                ) {
                    Text("删除模型配置")
                }
            }
        }
    }
}

private fun modelProviderKindLabel(kind: String, fallback: String): String =
    when (kind.trim().lowercase()) {
        "", "none" -> "\u672a\u914d\u7f6e"
        "openai" -> "OpenAI"
        "custom" -> "\u81ea\u5b9a\u4e49"
        "mock", "local_stub" -> "本地占位"
        else -> fallback
            .takeUnless { it.equals("Model provider", ignoreCase = true) }
            ?.ifBlank { kind }
            ?: "模型服务"
    }

private fun modelSendModeLabel(mode: String): String =
    when (mode.trim().lowercase()) {
        "preview_only" -> "\u4ec5\u53d1\u9001\u9884\u89c8"
        "detail_image" -> "\u53d1\u9001\u5927\u56fe"
        else -> mode
    }

private fun modelProviderOptionLabel(settings: ModelProviderSettingsUi): String =
    listOf(settings.providerLabel, settings.defaultModel)
        .map { it.trim() }
        .filter { it.isNotBlank() && !it.equals("Model provider", ignoreCase = true) }
        .joinToString(" · ")
        .ifBlank { settings.settingsId }

@Composable
private fun ProjectIntelligenceSettingsCard(
    providerOptions: List<ModelProviderSettingsUi>,
    settings: ProjectEvaluationSettingsUi?,
    promptProfiles: List<PromptProfileUi>,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
    onGenerateProjectRecommendation: () -> Unit,
    onConfigureModelProvider: () -> Unit,
) {
    val projectSettings = settings ?: ProjectEvaluationSettingsUi(projectId = "")
    val selectedProviderId = projectSettings.modelProviderSettingsId
    val modelOptions = providerOptions
        .filter { it.configured && it.providerKind != "none" }
        .distinctBy { it.settingsId }
    val selectedProviderReady = modelProviderReadyForProject(projectSettings, modelOptions)
    val selectablePromptProfiles = promptProfiles
        .filter { it.enabled && (it.scope.equals("global", ignoreCase = true) || it.projectId == null) }
        .ifEmpty { promptProfiles.filter { it.enabled } }
    val selectedPrompt = selectablePromptProfiles.firstOrNull { it.promptProfileId == projectSettings.promptProfileId }
        ?: selectablePromptProfiles.firstOrNull()
    val recommendationAction = manualProjectRecommendationActionUi(
        providerConfigured = selectedProviderReady,
        settings = projectSettings,
        actionInFlight = !actionsEnabled,
    )

    ElementCard(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            if (!selectedProviderReady) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        if (modelOptions.isEmpty()) "模型服务未配置" else "请选择模型服务",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    OutlinedButton(
                        onClick = onConfigureModelProvider,
                        enabled = actionsEnabled,
                        shape = elementShape,
                    ) {
                        Text("\u53bb\u914d\u7f6e")
                    }
                }
            }

            if (modelOptions.isNotEmpty()) {
                Text("使用模型", style = MaterialTheme.typography.labelLarge)
                LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    items(modelOptions, key = { it.settingsId }) { option ->
                        FilterChipButton(
                            label = modelProviderOptionLabel(option),
                            selected = selectedProviderId == option.settingsId,
                            onClick = {
                                onSaveSettings(
                                    projectSettingsAfterModelProviderSelection(projectSettings, option.settingsId),
                                )
                            },
                        )
                    }
                }
            }

            SettingsSwitchRow(
                title = "\u4e0a\u4f20\u540e\u81ea\u52a8\u8bc4\u4ef7",
                checked = projectSettings.autoEvaluateOnUpload,
                enabled = actionsEnabled && selectedProviderReady,
                onCheckedChange = {
                    onSaveSettings(
                        projectSettings.copy(
                            autoEvaluateOnUpload = it,
                        ),
                    )
                },
            )
            SettingsSwitchRow(
                title = "\u8fde\u62cd\u7ec4\u81ea\u52a8\u4f18\u9009",
                checked = projectSettings.autoBurstRecommendationEnabled,
                enabled = actionsEnabled && selectedProviderReady && projectSettings.projectId.isNotBlank(),
                onCheckedChange = {
                    onSaveSettings(
                        projectSettings.copy(
                            autoBurstRecommendationEnabled = it,
                        ),
                    )
                },
            )
            SettingsSwitchRow(
                title = "\u5141\u8bb8\u98ce\u9669\u7167\u7247\u53c2\u4e0e\u4f18\u9009",
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

            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("高级配置", style = MaterialTheme.typography.labelLarge)
                OptionRow(
                    title = "技术风险阈值",
                    values = listOf("loose", "standard", "strict"),
                    selected = projectSettings.cvPolicy,
                    enabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
                    labelForValue = ::cvPolicyLabel,
                    onSelected = { onSaveSettings(projectSettings.copy(cvPolicy = it)) },
                )
                Text(
                    cvPolicyHint(projectSettings.cvPolicy),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                )
                CvPolicyAdvancedControls(
                    projectSettings = projectSettings,
                    actionsEnabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
                    onSaveSettings = onSaveSettings,
                )
            }

            if (selectablePromptProfiles.isNotEmpty()) {
                Text("\u8bc4\u4ef7\u63d0\u793a\u8bcd", style = MaterialTheme.typography.labelLarge)
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
private fun CvPolicyAdvancedControls(
    projectSettings: ProjectEvaluationSettingsUi,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
) {
    val basePolicy = technicalPolicyForCvPolicy(projectSettings.cvPolicy)
    val customPolicy = projectSettings.cvPolicyOverrides
    var draftPolicy by remember(projectSettings.projectId, projectSettings.cvPolicy, customPolicy) {
        mutableStateOf(customPolicy ?: basePolicy)
    }
    SettingsSwitchRow(
        title = "自定义技术阈值",
        checked = customPolicy != null,
        enabled = actionsEnabled,
        onCheckedChange = { enabled ->
            val nextPolicy = if (enabled) draftPolicy else null
            onSaveSettings(projectSettings.copy(cvPolicyOverrides = nextPolicy))
        },
    )
    if (customPolicy == null) {
        return
    }
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            cvThresholdControlSpecs(draftPolicy, sceneProfile = projectSettings.sceneProfile).forEach { control ->
                ThresholdSlider(
                    title = control.title,
                    value = control.sliderValue,
                    displayPercent = control.displayPercent,
                    description = control.description,
                    enabled = actionsEnabled,
                    onValueChange = {
                        draftPolicy = updateCvThresholdControl(draftPolicy, control.key, it)
                    },
                )
            }
            CvPolicyCapabilityRows(sceneProfile = projectSettings.sceneProfile)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedButton(
                    onClick = {
                    draftPolicy = basePolicy
                    onSaveSettings(projectSettings.copy(cvPolicyOverrides = basePolicy))
                },
                enabled = actionsEnabled,
                shape = elementShape,
            ) {
                Text("重置")
            }
            Button(
                onClick = { onSaveSettings(projectSettings.copy(cvPolicyOverrides = draftPolicy)) },
                enabled = actionsEnabled && draftPolicy != customPolicy,
                shape = elementShape,
            ) {
                Text("应用阈值")
            }
        }
    }
}

internal enum class CvThresholdControlKey {
    BlurHigh,
    Clipping,
    ColorCast,
    FaceEyes,
    FaceExposure,
    FaceColorCast,
}

internal data class CvThresholdControlSpec(
    val key: CvThresholdControlKey,
    val title: String,
    val sliderValue: Double,
    val displayPercent: Int,
    val description: String,
)

internal fun cvThresholdControlSpecs(
    policy: TechnicalAssessmentPolicyUi,
    sceneProfile: String = "general",
): List<CvThresholdControlSpec> {
    val controls = mutableListOf(
        CvThresholdControlSpec(
            key = CvThresholdControlKey.BlurHigh,
            title = "失焦灵敏度",
            sliderValue = blurSensitivity(policy),
            displayPercent = percentLabel(blurSensitivity(policy)),
            description = blurThresholdDescription(policy),
        ),
        CvThresholdControlSpec(
            key = CvThresholdControlKey.Clipping,
            title = "死黑/死白灵敏度",
            sliderValue = clippingSensitivity(policy),
            displayPercent = percentLabel(clippingSensitivity(policy)),
            description = clippingThresholdDescription(policy),
        ),
        CvThresholdControlSpec(
            key = CvThresholdControlKey.ColorCast,
            title = "偏色灵敏度",
            sliderValue = colorCastSensitivity(policy),
            displayPercent = percentLabel(colorCastSensitivity(policy)),
            description = colorCastThresholdDescription(policy),
        ),
    )
    if (sceneProfile.trim().equals("portrait", ignoreCase = true)) {
        controls += CvThresholdControlSpec(
            key = CvThresholdControlKey.FaceEyes,
            title = "闭眼灵敏度",
            sliderValue = faceEyesSensitivity(policy),
            displayPercent = percentLabel(faceEyesSensitivity(policy)),
            description = faceEyesThresholdDescription(policy),
        )
        controls += CvThresholdControlSpec(
            key = CvThresholdControlKey.FaceExposure,
            title = "面部死黑/死白灵敏度",
            sliderValue = faceExposureSensitivity(policy),
            displayPercent = percentLabel(faceExposureSensitivity(policy)),
            description = faceExposureThresholdDescription(policy),
        )
        controls += CvThresholdControlSpec(
            key = CvThresholdControlKey.FaceColorCast,
            title = "面部偏色灵敏度",
            sliderValue = faceColorCastSensitivity(policy),
            displayPercent = percentLabel(faceColorCastSensitivity(policy)),
            description = faceColorCastThresholdDescription(policy),
        )
    }
    return controls
}

internal fun updateCvThresholdControl(
    policy: TechnicalAssessmentPolicyUi,
    key: CvThresholdControlKey,
    value: Double,
): TechnicalAssessmentPolicyUi =
    when (key) {
        CvThresholdControlKey.BlurHigh -> {
            val next = denormalize(value, BLUR_HIGH_MIN, BLUR_HIGH_MAX)
            policy.copy(
                blurHighEdgeThreshold = next,
                blurHighFrequencyThreshold = next,
                blurSevereEdgeThreshold = policy.blurSevereEdgeThreshold.coerceAtMost(next),
                blurSevereFrequencyThreshold = policy.blurSevereFrequencyThreshold.coerceAtMost(next),
            )
        }
        CvThresholdControlKey.Clipping -> {
            val sensitivity = value.coerceIn(0.0, 1.0)
            policy.copy(
                clippingHighRatio = inverseDenormalize(sensitivity, CLIPPING_HIGH_MIN, CLIPPING_HIGH_MAX),
                clippingHighConnectedRatio = inverseDenormalize(
                    sensitivity,
                    CLIPPING_HIGH_CONNECTED_MIN,
                    CLIPPING_HIGH_CONNECTED_MAX,
                ),
                clippingSevereRatio = inverseDenormalize(
                    sensitivity,
                    CLIPPING_SEVERE_MIN,
                    CLIPPING_SEVERE_MAX,
                ),
                clippingSevereConnectedRatio = inverseDenormalize(
                    sensitivity,
                    CLIPPING_SEVERE_MIN,
                    CLIPPING_SEVERE_MAX,
                ),
            )
        }
        CvThresholdControlKey.ColorCast -> {
            val sensitivity = value.coerceIn(0.0, 1.0)
            policy.copy(
                colorCastHighThreshold = inverseDenormalize(
                    sensitivity,
                    COLOR_CAST_HIGH_MIN,
                    COLOR_CAST_HIGH_MAX,
                ),
                colorCastSevereThreshold = inverseDenormalize(
                    sensitivity,
                    COLOR_CAST_SEVERE_MIN,
                    COLOR_CAST_SEVERE_MAX,
                ),
            )
        }
        CvThresholdControlKey.FaceEyes -> {
            val next = denormalize(value, FACE_EYE_OPEN_WARN_MIN, FACE_EYE_OPEN_WARN_MAX)
            policy.copy(faceEyeOpenWarnThreshold = next)
        }
        CvThresholdControlKey.FaceExposure -> {
            val next = inverseDenormalize(value, FACE_EXPOSURE_WARN_MIN, FACE_EXPOSURE_WARN_MAX)
            policy.copy(faceExposureWarnRatio = next)
        }
        CvThresholdControlKey.FaceColorCast -> {
            val next = inverseDenormalize(value, FACE_COLOR_CAST_WARN_MIN, FACE_COLOR_CAST_WARN_MAX)
            policy.copy(faceColorCastWarnThreshold = next)
        }
    }

@Composable
private fun ThresholdSlider(
    title: String,
    value: Double,
    displayPercent: Int,
    description: String,
    enabled: Boolean,
    onValueChange: (Double) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(title, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Text("$displayPercent%", fontWeight = FontWeight.SemiBold)
        }
        Slider(
            value = value.toFloat(),
            onValueChange = { onValueChange(it.toDouble()) },
            enabled = enabled,
            valueRange = 0f..1f,
        )
        Text(description, color = MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
    }
}

@Composable
private fun CvPolicyCapabilityRows(sceneProfile: String) {
    if (sceneProfile.trim().lowercase() != "portrait") {
        return
    }
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        CapabilityStatusRow(title = "人像专项", status = "已启用人脸/闭眼")
    }
}

@Composable
private fun CapabilityStatusRow(title: String, status: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(title, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Surface(
            color = ElementControlSurface,
            shape = CircleShape,
            border = BorderStroke(1.dp, ElementBorder),
        ) {
            Text(
                status,
                modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontSize = 12.sp,
                fontWeight = FontWeight.SemiBold,
            )
        }
    }
}

private const val BLUR_HIGH_MIN = 0.06
private const val BLUR_HIGH_MAX = 0.22
private const val CLIPPING_HIGH_MIN = 0.04
private const val CLIPPING_HIGH_MAX = 0.30
private const val CLIPPING_HIGH_CONNECTED_MIN = 0.04
private const val CLIPPING_HIGH_CONNECTED_MAX = 0.30
private const val CLIPPING_SEVERE_MIN = 0.35
private const val CLIPPING_SEVERE_MAX = 0.75
private const val COLOR_CAST_HIGH_MIN = 0.28
private const val COLOR_CAST_HIGH_MAX = 0.65
private const val COLOR_CAST_SEVERE_MIN = 0.50
private const val COLOR_CAST_SEVERE_MAX = 0.90
private const val FACE_EYE_OPEN_WARN_MIN = 0.20
private const val FACE_EYE_OPEN_WARN_MAX = 0.55
private const val FACE_EXPOSURE_WARN_MIN = 0.12
private const val FACE_EXPOSURE_WARN_MAX = 0.40
private const val FACE_COLOR_CAST_WARN_MIN = 0.28
private const val FACE_COLOR_CAST_WARN_MAX = 0.65

private fun blurSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    normalize(policy.blurHighEdgeThreshold, BLUR_HIGH_MIN, BLUR_HIGH_MAX)

private fun clippingSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    listOf(
        inverseNormalize(policy.clippingHighRatio, CLIPPING_HIGH_MIN, CLIPPING_HIGH_MAX),
        inverseNormalize(
            policy.clippingHighConnectedRatio,
            CLIPPING_HIGH_CONNECTED_MIN,
            CLIPPING_HIGH_CONNECTED_MAX,
        ),
        inverseNormalize(policy.clippingSevereRatio, CLIPPING_SEVERE_MIN, CLIPPING_SEVERE_MAX),
    ).average().coerceIn(0.0, 1.0)

private fun colorCastSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    listOf(
        inverseNormalize(policy.colorCastHighThreshold, COLOR_CAST_HIGH_MIN, COLOR_CAST_HIGH_MAX),
        inverseNormalize(policy.colorCastSevereThreshold, COLOR_CAST_SEVERE_MIN, COLOR_CAST_SEVERE_MAX),
    ).average().coerceIn(0.0, 1.0)

private fun faceEyesSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    normalize(policy.faceEyeOpenWarnThreshold, FACE_EYE_OPEN_WARN_MIN, FACE_EYE_OPEN_WARN_MAX)

private fun faceExposureSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    inverseNormalize(policy.faceExposureWarnRatio, FACE_EXPOSURE_WARN_MIN, FACE_EXPOSURE_WARN_MAX)

private fun faceColorCastSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    inverseNormalize(policy.faceColorCastWarnThreshold, FACE_COLOR_CAST_WARN_MIN, FACE_COLOR_CAST_WARN_MAX)

private fun percentLabel(value: Double): Int =
    (value.coerceIn(0.0, 1.0) * 100).roundToInt()

private fun blurThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：边缘和高频细节都低于 ${formatRatioPercent(policy.blurHighEdgeThreshold)} 时标记失焦；" +
        "低于 ${formatRatioPercent(policy.blurSevereEdgeThreshold)} 视为严重。"

private fun clippingThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：近黑 <=${policy.shadowClipThreshold} / 近白 >=${policy.highlightClipThreshold}，" +
        "占比超过 ${formatRatioPercent(policy.clippingHighRatio)} 或连片超过 ${formatRatioPercent(policy.clippingHighConnectedRatio)} 时标记；" +
        "${formatRatioPercent(policy.clippingSevereRatio)} 以上视为严重。"

private fun colorCastThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：RGB 通道相对亮度差异超过 ${formatDecimal(policy.colorCastHighThreshold, 2)} 时标记偏色；" +
        "超过 ${formatDecimal(policy.colorCastSevereThreshold, 2)} 视为严重。"

private fun faceEyesThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：检测到人脸时，任一眼睁开概率低于 ${formatDecimal(policy.faceEyeOpenWarnThreshold, 2)} 标记闭眼风险。"

private fun faceExposureThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：人脸区域近黑/近白像素占比超过 ${formatRatioPercent(policy.faceExposureWarnRatio)} 标记面部曝光风险。"

private fun faceColorCastThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：人脸区域 RGB 相对亮度差异超过 ${formatDecimal(policy.faceColorCastWarnThreshold, 2)} 标记面部偏色。"

private fun formatRatioPercent(value: Double): String =
    "${percentLabel(value)}%"

private fun formatDecimal(value: Double, digits: Int): String =
    "%.${digits}f".format(Locale.US, value)

private fun normalize(value: Double, min: Double, max: Double): Double =
    ((value - min) / (max - min)).coerceIn(0.0, 1.0)

private fun inverseNormalize(value: Double, min: Double, max: Double): Double =
    ((max - value) / (max - min)).coerceIn(0.0, 1.0)

private fun denormalize(value: Double, min: Double, max: Double): Double =
    min + (max - min) * value.coerceIn(0.0, 1.0)

private fun inverseDenormalize(value: Double, min: Double, max: Double): Double =
    max - (max - min) * value.coerceIn(0.0, 1.0)

private fun technicalPolicyForCvPolicy(value: String): TechnicalAssessmentPolicyUi =
    when (value.trim().lowercase()) {
        "loose" -> TechnicalAssessmentPolicyUi(
            blurSevereEdgeThreshold = 0.025,
            blurSevereFrequencyThreshold = 0.025,
            blurHighEdgeThreshold = 0.09,
            blurHighFrequencyThreshold = 0.09,
            highlightClipThreshold = 250,
            shadowClipThreshold = 5,
            clippingHighRatio = 0.18,
            clippingHighConnectedRatio = 0.25,
            clippingSevereRatio = 0.65,
            clippingSevereConnectedRatio = 0.65,
            colorCastHighThreshold = 0.55,
            colorCastSevereThreshold = 0.85,
            faceEyeOpenWarnThreshold = 0.25,
            faceExposureWarnRatio = 0.35,
            faceColorCastWarnThreshold = 0.55,
        )
        "strict" -> TechnicalAssessmentPolicyUi(
            blurSevereEdgeThreshold = 0.06,
            blurSevereFrequencyThreshold = 0.06,
            blurHighEdgeThreshold = 0.16,
            blurHighFrequencyThreshold = 0.16,
            highlightClipThreshold = 242,
            shadowClipThreshold = 13,
            clippingHighRatio = 0.08,
            clippingHighConnectedRatio = 0.12,
            clippingSevereRatio = 0.40,
            clippingSevereConnectedRatio = 0.40,
            colorCastHighThreshold = 0.32,
            colorCastSevereThreshold = 0.55,
            faceEyeOpenWarnThreshold = 0.45,
            faceExposureWarnRatio = 0.16,
            faceColorCastWarnThreshold = 0.32,
        )
        else -> TechnicalAssessmentPolicyUi(
            blurSevereEdgeThreshold = 0.04,
            blurSevereFrequencyThreshold = 0.04,
            blurHighEdgeThreshold = 0.12,
            blurHighFrequencyThreshold = 0.12,
            highlightClipThreshold = 245,
            shadowClipThreshold = 10,
            clippingHighRatio = 0.12,
            clippingHighConnectedRatio = 0.18,
            clippingSevereRatio = 0.50,
            clippingSevereConnectedRatio = 0.50,
            colorCastHighThreshold = 0.42,
            colorCastSevereThreshold = 0.70,
            faceEyeOpenWarnThreshold = 0.35,
            faceExposureWarnRatio = 0.25,
            faceColorCastWarnThreshold = 0.42,
        )
    }

private fun cvPolicyHint(value: String): String =
    when (value.trim().lowercase()) {
        "loose" -> "减少误报，只标记明显失焦、死黑和过曝。"
        "strict" -> "更早提示风险，适合需要严格筛片的项目。"
        else -> "平衡误报和漏报，适合大多数项目。"
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

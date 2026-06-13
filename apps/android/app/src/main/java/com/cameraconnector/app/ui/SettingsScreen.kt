package com.cameraconnector.app.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.graphics.Bitmap
import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.gestures.detectDragGestures
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
import androidx.compose.material.icons.automirrored.outlined.KeyboardArrowRight
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material.icons.outlined.PhotoLibrary
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.SyncAlt
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
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
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onSizeChanged
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
import com.cameraconnector.app.core.PromptPackUi
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
internal fun PromptPacksScreen(
    promptPacks: List<PromptPackUi>,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onCreatePromptPackage: () -> Unit,
    onCreatePromptPackInPackage: (String) -> Unit,
    onOpenPromptPack: (String) -> Unit,
    onDeletePromptPackage: (String) -> Unit,
    onDeletePromptPack: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var collapsedPackages by rememberSaveable { mutableStateOf(emptyList<String>()) }
    val promptPackages = promptPacks
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

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            CompactBackHeader(
                title = "\u63d0\u793a\u8bcd\u914d\u7f6e",
                subtitle = "按提示词包管理摄影评价偏好",
                onBack = onBack,
                trailing = {
                OutlinedButton(
                    onClick = onCreatePromptPackage,
                    enabled = actionInFlight == null,
                    shape = elementShape,
                ) {
                    Icon(Icons.Outlined.Add, contentDescription = null, modifier = Modifier.size(18.dp))
                    Spacer(Modifier.width(6.dp))
                    Text("新建提示词包")
                }
                },
            )
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        if (promptPacks.isEmpty()) {
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
            promptPackages.forEach { (packageFolder, packsInPackage) ->
                val expanded = packageFolder !in collapsedPackages
                item(key = "package-$packageFolder") {
                    PromptPackageSection(
                        packageFolder = packageFolder,
                        profiles = packsInPackage.sortedBy { promptPackDisplayName(it) },
                        expanded = expanded,
                        actionInFlight = actionInFlight,
                        onToggle = {
                            collapsedPackages = if (expanded) {
                                collapsedPackages + packageFolder
                            } else {
                                collapsedPackages - packageFolder
                            }
                        },
                        onCreate = {
                            onCreatePromptPackInPackage(packageFolder)
                            collapsedPackages = collapsedPackages - packageFolder
                        },
                        onOpenPromptPack = onOpenPromptPack,
                        onDeletePackage = { onDeletePromptPackage(packageFolder) },
                        onDeletePromptPack = onDeletePromptPack,
                    )
                }
            }
        }
    }
}

@Composable
private fun PromptPackageSection(
    packageFolder: String,
    profiles: List<PromptPackUi>,
    expanded: Boolean,
    actionInFlight: String?,
    onToggle: () -> Unit,
    onCreate: () -> Unit,
    onOpenPromptPack: (String) -> Unit,
    onDeletePackage: () -> Unit,
    onDeletePromptPack: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val isBuiltinPackage = packageFolder == "builtin"
    ElementCard(modifier = modifier.fillMaxWidth()) {
        Column(Modifier.fillMaxWidth()) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable(onClick = onToggle)
                    .padding(horizontal = 14.dp, vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                Icon(
                    imageVector = if (expanded) Icons.Outlined.KeyboardArrowDown else Icons.AutoMirrored.Outlined.KeyboardArrowRight,
                    contentDescription = if (expanded) "收起提示词包" else "展开提示词包",
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(20.dp),
                )
                Column(Modifier.weight(1f)) {
                    Text(
                        promptPackageLabel(packageFolder),
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        "${profiles.size} 个提示词",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
                if (!isBuiltinPackage) {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedButton(
                            onClick = onCreate,
                            enabled = actionInFlight == null,
                            shape = elementShape,
                            contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp),
                        ) {
                            Icon(Icons.Outlined.Add, contentDescription = null, modifier = Modifier.size(16.dp))
                            Spacer(Modifier.width(4.dp))
                            Text("新建")
                        }
                        OutlinedButton(
                            onClick = onDeletePackage,
                            enabled = actionInFlight == null,
                            shape = elementShape,
                            colors = ButtonDefaults.outlinedButtonColors(contentColor = ElementDanger),
                            border = BorderStroke(1.dp, ElementDanger),
                            contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp),
                        ) {
                            Text("删除包")
                        }
                    }
                }
            }
            if (expanded) {
                HorizontalDivider(
                    color = MaterialTheme.colorScheme.outline.copy(alpha = 0.45f),
                    thickness = 1.dp,
                )
                Column(Modifier.fillMaxWidth()) {
                    profiles.forEachIndexed { index, profile ->
                        PromptPackRow(
                            profile = profile,
                            onClick = { onOpenPromptPack(profile.promptPackId) },
                            onDelete = { onDeletePromptPack(profile.promptPackId) },
                            modifier = Modifier.fillMaxWidth(),
                        )
                        if (index != profiles.lastIndex) {
                            HorizontalDivider(
                                modifier = Modifier.padding(start = 14.dp),
                                color = MaterialTheme.colorScheme.outline.copy(alpha = 0.25f),
                                thickness = 1.dp,
                            )
                        }
                    }
                }
            } else if (!isBuiltinPackage && profiles.isEmpty()) {
                Text(
                    "展开后新建提示词",
                    modifier = Modifier.padding(start = 44.dp, end = 14.dp, bottom = 12.dp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun PromptPackRow(
    profile: PromptPackUi,
    onClick: () -> Unit,
    onDelete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 13.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
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
            profile.activePromptText
                ?.takeIf { it.isNotBlank() }
                ?.let { promptMarkdown ->
                    MarkdownText(
                        markdown = promptMarkdown,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.82f),
                        compact = true,
                        maxLines = 1,
                    )
                }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            if (!profile.builtIn) {
                Text(
                    "删除",
                    modifier = Modifier.clickable(onClick = onDelete),
                    color = ElementDanger,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                )
            }
            Text(
                if (profile.builtIn) "复制" else "编辑",
                color = ElementBlue,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
            )
        }
    }
}

private fun promptPackMetaText(profile: PromptPackUi): String =
    listOf(
        promptStyleTagsText(profile),
        sceneProfileLabel(profile.sceneProfile),
        if (profile.builtIn) "内置" else "自定义",
    ).filter { it.isNotBlank() }.distinct().joinToString(" / ")

@Composable
internal fun PromptPackEditorScreen(
    profile: PromptPackUi?,
    initialDistributionFolder: String,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onSave: (PromptPackUi, String, List<String>, String, String, String) -> Unit,
    onCreate: (String, List<String>, String, String, String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val createMode = profile == null
    val builtInProfile = profile?.builtIn == true
    val editableExistingProfile = !createMode && !builtInProfile
    var name by remember(profile?.promptPackId) {
        mutableStateOf(
            when {
                createMode -> ""
                builtInProfile -> profile.let(::promptPackDisplayName).let { "自定义 $it" }
                else -> profile.let(::promptPackDisplayName)
            },
        )
    }
    var styleTagsText by remember(profile?.promptPackId) {
        mutableStateOf(
            profile?.styleTags
                ?.filter { it.isNotBlank() }
                ?.joinToString(" ") { promptStyleTagLabel(it) }
                ?: "通用 均衡",
        )
    }
    var sceneProfile by remember(profile?.promptPackId) {
        mutableStateOf(profile?.sceneProfile?.ifBlank { "general" } ?: "general")
    }
    var distributionFolder by remember(profile?.promptPackId, initialDistributionFolder) {
        mutableStateOf(
            when {
                createMode -> initialDistributionFolder
                builtInProfile -> "user"
                else -> profile.distributionFolder.takeIf { it.isNotBlank() } ?: "user"
            },
        )
    }
    var promptText by remember(profile?.promptPackId, profile?.sharedPreference, profile?.activePromptText) {
        mutableStateOf(profile?.sharedPreference ?: profile?.activePromptText.orEmpty())
    }
    var promptTab by rememberSaveable(profile?.promptPackId) { mutableStateOf("edit") }
    val cleanName = name.trim()
    val cleanPrompt = promptText.trim()
    val cleanPackage = distributionFolder.trim()
    val cleanStyleTags = parsePromptStyleTags(styleTagsText)
    val actionsEnabled = actionInFlight == null
    val canSubmit = actionsEnabled && cleanPrompt.isNotBlank() &&
        when {
            createMode -> cleanName.isNotBlank() && cleanPackage.isNotBlank()
            builtInProfile -> cleanName.isNotBlank() && cleanPackage.isNotBlank()
            else -> cleanName.isNotBlank()
        }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            CompactBackHeader(
                title = when {
                    createMode -> "新建提示词"
                    else -> profile.let(::promptPackDisplayName)
                },
                subtitle = when {
                    createMode -> "选择提示词包，保存后进入编辑"
                    builtInProfile -> "内置偏好会复制为全局自定义偏好"
                    else -> "保存后成为这套全局偏好的新版本"
                },
                onBack = onBack,
            )
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        if (createMode || builtInProfile || editableExistingProfile) {
            item {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text(if (builtInProfile) "复制后的名称" else "提示词名称") },
                    modifier = Modifier.fillMaxWidth(),
                    enabled = actionsEnabled,
                    singleLine = true,
                )
            }
        }

        if (createMode || builtInProfile) {
            item {
                OutlinedTextField(
                    value = distributionFolder,
                    onValueChange = { distributionFolder = it },
                    label = { Text("提示词包") },
                    supportingText = { Text("同一个提示词包可以整体导入、导出和共享，例如 user、portrait") },
                    modifier = Modifier.fillMaxWidth(),
                    enabled = actionsEnabled,
                    singleLine = true,
                )
            }
        } else {
            item {
                Surface(
                    modifier = Modifier.fillMaxWidth(),
                    color = ElementControlSurface,
                    shape = elementShape,
                    border = BorderStroke(1.dp, ElementBorder),
                ) {
                    Row(
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                            Text("提示词包", style = MaterialTheme.typography.labelLarge)
                            Text(
                                promptPackageLabel(promptPackageFolder(profile)),
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        Text("不可更改", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }

        if (createMode || editableExistingProfile) {
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
                    values = projectSceneProfileOptions(),
                    selected = sceneProfile,
                    enabled = actionsEnabled,
                    labelForValue = ::sceneProfileLabel,
                    onSelected = { sceneProfile = it },
                )
            }
        }

        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            "我的摄影评价偏好",
                            modifier = Modifier.weight(1f),
                            style = MaterialTheme.typography.titleMedium,
                        )
                        PromptEditorTab(
                            text = "编辑",
                            selected = promptTab == "edit",
                            enabled = actionsEnabled,
                            onClick = { promptTab = "edit" },
                        )
                        PromptEditorTab(
                            text = "预览",
                            selected = promptTab == "preview",
                            enabled = actionsEnabled,
                            onClick = { promptTab = "preview" },
                        )
                    }
                    if (promptTab == "preview") {
                        if (cleanPrompt.isBlank()) {
                            Text(
                                "输入 Markdown 后会在这里预览。",
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        } else {
                            MarkdownText(markdown = cleanPrompt)
                        }
                    } else {
                        OutlinedTextField(
                            value = promptText,
                            onValueChange = { promptText = it },
                            modifier = Modifier.fillMaxWidth(),
                            minLines = 10,
                            enabled = actionsEnabled,
                            placeholder = { Text("例如：\n# 人像偏好\n- 优先自然表情\n- 避免过度磨皮\n- 保留现场氛围") },
                            supportingText = {
                                Text(
                                    "支持 Markdown：# 标题、- 列表、> 引用、`代码`、**加粗**、*斜体*",
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            },
                        )
                    }
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
                            cleanPackage,
                            cleanPrompt,
                        )
                    } else {
                        onSave(
                            profile,
                            cleanName,
                            cleanStyleTags.ifEmpty { listOf("general", "balanced") },
                            sceneProfile,
                            cleanPackage.ifBlank { "user" },
                            cleanPrompt,
                        )
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
                        builtInProfile -> "保存偏好"
                        else -> "保存提示词"
                    },
                )
            }
        }
    }
}

@Composable
private fun PromptEditorTab(
    text: String,
    selected: Boolean,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    if (selected) {
        Button(
            onClick = onClick,
            enabled = enabled,
            shape = elementShape,
            colors = ButtonDefaults.buttonColors(
                containerColor = ElementBlue,
                contentColor = ElementOnAccent,
            ),
            contentPadding = PaddingValues(horizontal = 14.dp, vertical = 8.dp),
        ) {
            Text(text)
        }
    } else {
        OutlinedButton(
            onClick = onClick,
            enabled = enabled,
            shape = elementShape,
            contentPadding = PaddingValues(horizontal = 14.dp, vertical = 8.dp),
        ) {
            Text(text)
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
        "mock", "local_stub" -> "\u672c\u5730\u5206\u6790"
        else -> fallback
            .takeUnless(::isPlaceholderModelProviderLabel)
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
        .filter { it.isNotBlank() && !isPlaceholderModelProviderLabel(it) }
        .joinToString(" · ")
        .ifBlank { settings.settingsId }

private fun isPlaceholderModelProviderLabel(value: String): Boolean {
    val normalized = value.trim()
    return normalized.equals("Model provider", ignoreCase = true) || normalized == "模型服务"
}

@Composable
private fun ProjectIntelligencePanelPage(
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
private fun ProjectIntelligenceSettingsCard(
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
private fun ProjectSceneQuickSettings(
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

private fun projectWorkflowSummary(settings: ProjectEvaluationSettingsUi): String =
    listOf(
        "自动评价${if (settings.autoEvaluateOnUpload) "开" else "关"}",
        "连拍优选${if (settings.autoBurstRecommendationEnabled) "开" else "关"}",
        "风险参与${if (settings.allowRiskyModelSelects) "开" else "关"}",
    ).joinToString(" · ")

private fun technicalRiskSummary(settings: ProjectEvaluationSettingsUi): String =
    listOfNotNull(
        cvPolicyLabel(settings.cvPolicy),
        if (settings.cvPolicyOverrides != null) "自定义阈值" else null,
    ).joinToString(" · ")

private fun projectSceneProfileOptions(): List<String> =
    listOf("general", "portrait", "action", "landscape")

private fun sceneProfileHint(value: String): String =
    when (value.trim().lowercase()) {
        "portrait" -> "优先启用人像相关风险判断，并影响模型评价语境。"
        "action" -> "适合运动、抓拍和动态主体。"
        "landscape" -> "适合风光、建筑和环境类作品。"
        else -> "通用摄影场景，适合大多数项目。"
    }

internal fun selectedCvThresholdMode(settings: ProjectEvaluationSettingsUi): String =
    if (settings.cvPolicyOverrides != null) {
        "custom"
    } else {
        settings.cvPolicy.ifBlank { "standard" }
    }

internal fun projectSettingsAfterCvThresholdModeSelection(
    settings: ProjectEvaluationSettingsUi,
    selectedMode: String,
): ProjectEvaluationSettingsUi {
    val mode = selectedMode.trim().lowercase()
    if (mode == "custom") {
        val baseMode = settings.cvPolicy.ifBlank { "standard" }
        return settings.copy(
            cvPolicy = baseMode,
            cvPolicyOverrides = settings.cvPolicyOverrides ?: technicalPolicyForCvPolicy(baseMode),
        )
    }
    val preset = when (mode) {
        "loose", "standard", "strict" -> mode
        else -> "standard"
    }
    return settings.copy(cvPolicy = preset, cvPolicyOverrides = null)
}

private fun cvThresholdModeLabel(value: String): String =
    when (value.trim().lowercase()) {
        "custom" -> "自定义"
        else -> cvPolicyLabel(value)
    }

@Composable
private fun PromptPackSelector(
    selectedPrompt: PromptPackUi?,
    promptPacks: List<PromptPackUi>,
    expanded: Boolean,
    enabled: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    onSelected: (PromptPackUi) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text("评价提示词", style = MaterialTheme.typography.labelLarge)
        Surface(
            color = ElementControlSurface,
            shape = elementShape,
            border = BorderStroke(1.dp, ElementBorder),
            modifier = Modifier
                .fillMaxWidth()
                .clickable(enabled = enabled) { onExpandedChange(!expanded) },
        ) {
            Column(Modifier.fillMaxWidth()) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 14.dp, vertical = 12.dp),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                        Text(
                            selectedPrompt?.let(::promptPackDisplayName) ?: "未选择提示词",
                            style = MaterialTheme.typography.titleMedium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            selectedPrompt?.let(::promptPackMetaText) ?: "用于模型评价、连拍优选和项目优选",
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                    Icon(
                        imageVector = if (expanded) Icons.Outlined.KeyboardArrowDown else Icons.AutoMirrored.Outlined.KeyboardArrowRight,
                        contentDescription = if (expanded) "收起提示词列表" else "展开提示词列表",
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (expanded) {
                    HorizontalDivider(
                        color = MaterialTheme.colorScheme.outline.copy(alpha = 0.35f),
                        thickness = 1.dp,
                    )
                    promptPacks.forEachIndexed { index, profile ->
                        PromptPackOptionRow(
                            profile = profile,
                            selected = profile.promptPackId == selectedPrompt?.promptPackId,
                            enabled = enabled,
                            onClick = { onSelected(profile) },
                        )
                        if (index != promptPacks.lastIndex) {
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
}

@Composable
private fun PromptPackOptionRow(
    profile: PromptPackUi,
    selected: Boolean,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 11.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Text(
                promptPackDisplayName(profile),
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.SemiBold,
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
            Surface(
                color = ElementBlue,
                shape = CircleShape,
            ) {
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
    val controls = cvThresholdControlSpecs(draftPolicy, sceneProfile = projectSettings.sceneProfile)
    if (customPolicy == null) {
        return
    }
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = ElementControlSurface.copy(alpha = 0.78f),
        shape = elementShape,
        border = BorderStroke(1.dp, ElementBorder.copy(alpha = 0.85f)),
    ) {
        Column(
            modifier = Modifier.padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text("风险触发灵敏度", style = MaterialTheme.typography.titleMedium)
                    Text(
                        if (projectSettings.sceneProfile.trim().equals("portrait", ignoreCase = true)) {
                            "包含人像闭眼、面部曝光和面部偏色"
                        } else {
                            "包含失焦、死黑死白和偏色"
                        },
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
                Surface(
                    color = ElementBlue.copy(alpha = 0.14f),
                    shape = CircleShape,
                    border = BorderStroke(1.dp, ElementBlue.copy(alpha = 0.5f)),
                ) {
                    Text(
                        sceneProfileLabel(projectSettings.sceneProfile),
                        modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp),
                        color = ElementBlue,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            }
            controls.forEachIndexed { index, control ->
                ThresholdSlider(
                    title = control.title,
                    value = control.sliderValue,
                    displayLabel = control.displayLabel,
                    description = control.description,
                    enabled = actionsEnabled,
                    onValueChange = {
                        draftPolicy = updateCvThresholdControl(draftPolicy, control.key, it)
                    },
                )
                if (index != controls.lastIndex) {
                    HorizontalDivider(color = ElementBorder.copy(alpha = 0.6f))
                }
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                OutlinedButton(
                    onClick = {
                        draftPolicy = basePolicy
                        onSaveSettings(projectSettings.copy(cvPolicyOverrides = basePolicy))
                    },
                    enabled = actionsEnabled,
                    modifier = Modifier.weight(1f),
                    shape = elementShape,
                ) {
                    Text("重置预设")
                }
                Button(
                    onClick = { onSaveSettings(projectSettings.copy(cvPolicyOverrides = draftPolicy)) },
                    enabled = actionsEnabled && draftPolicy != customPolicy,
                    modifier = Modifier.weight(1f),
                    shape = elementShape,
                ) {
                    Text("应用阈值")
                }
            }
        }
    }
}

internal enum class CvThresholdControlKey {
    BlurHigh,
    Clipping,
    ShadowClipThreshold,
    HighlightClipThreshold,
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
    val displayLabel: String,
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
            displayLabel = "${percentLabel(blurSensitivity(policy))}%",
            description = blurThresholdDescription(policy),
        ),
        CvThresholdControlSpec(
            key = CvThresholdControlKey.Clipping,
            title = "死黑/死白灵敏度",
            sliderValue = clippingSensitivity(policy),
            displayPercent = percentLabel(clippingSensitivity(policy)),
            displayLabel = "${percentLabel(clippingSensitivity(policy))}%",
            description = clippingThresholdDescription(policy),
        ),
        CvThresholdControlSpec(
            key = CvThresholdControlKey.ShadowClipThreshold,
            title = "\u8fd1\u9ed1\u8fb9\u754c",
            sliderValue = shadowClipThresholdValue(policy),
            displayPercent = policy.shadowClipThreshold,
            displayLabel = "<=${policy.shadowClipThreshold}",
            description = "\u4eae\u5ea6\u5c0f\u4e8e\u7b49\u4e8e ${policy.shadowClipThreshold} \u7684\u50cf\u7d20\u8ba1\u5165\u6697\u90e8\u6b7b\u9ed1\u3002\u6570\u503c\u8d8a\u4f4e\uff0c\u8bef\u62a5\u8d8a\u5c11\u3002",
        ),
        CvThresholdControlSpec(
            key = CvThresholdControlKey.HighlightClipThreshold,
            title = "\u8fd1\u767d\u8fb9\u754c",
            sliderValue = highlightClipThresholdValue(policy),
            displayPercent = policy.highlightClipThreshold,
            displayLabel = ">=${policy.highlightClipThreshold}",
            description = "\u4eae\u5ea6\u5927\u4e8e\u7b49\u4e8e ${policy.highlightClipThreshold} \u7684\u50cf\u7d20\u8ba1\u5165\u9ad8\u5149\u6ea2\u51fa\u3002\u6570\u503c\u8d8a\u9ad8\uff0c\u5224\u5b9a\u8d8a\u4fdd\u5b88\u3002",
        ),
        CvThresholdControlSpec(
            key = CvThresholdControlKey.ColorCast,
            title = "偏色灵敏度",
            sliderValue = colorCastSensitivity(policy),
            displayPercent = percentLabel(colorCastSensitivity(policy)),
            displayLabel = "${percentLabel(colorCastSensitivity(policy))}%",
            description = colorCastThresholdDescription(policy),
        ),
    )
    if (sceneProfile.trim().equals("portrait", ignoreCase = true)) {
        controls += CvThresholdControlSpec(
            key = CvThresholdControlKey.FaceEyes,
            title = "闭眼灵敏度",
            sliderValue = faceEyesSensitivity(policy),
            displayPercent = percentLabel(faceEyesSensitivity(policy)),
            displayLabel = "${percentLabel(faceEyesSensitivity(policy))}%",
            description = faceEyesThresholdDescription(policy),
        )
        controls += CvThresholdControlSpec(
            key = CvThresholdControlKey.FaceExposure,
            title = "面部死黑/死白灵敏度",
            sliderValue = faceExposureSensitivity(policy),
            displayPercent = percentLabel(faceExposureSensitivity(policy)),
            displayLabel = "${percentLabel(faceExposureSensitivity(policy))}%",
            description = faceExposureThresholdDescription(policy),
        )
        controls += CvThresholdControlSpec(
            key = CvThresholdControlKey.FaceColorCast,
            title = "面部偏色灵敏度",
            sliderValue = faceColorCastSensitivity(policy),
            displayPercent = percentLabel(faceColorCastSensitivity(policy)),
            displayLabel = "${percentLabel(faceColorCastSensitivity(policy))}%",
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
        CvThresholdControlKey.ShadowClipThreshold -> {
            policy.copy(
                shadowClipThreshold = denormalize(
                    value,
                    SHADOW_CLIP_THRESHOLD_MIN.toDouble(),
                    SHADOW_CLIP_THRESHOLD_MAX.toDouble(),
                ).roundToInt().coerceIn(SHADOW_CLIP_THRESHOLD_MIN, SHADOW_CLIP_THRESHOLD_MAX),
            )
        }
        CvThresholdControlKey.HighlightClipThreshold -> {
            policy.copy(
                highlightClipThreshold = denormalize(
                    value,
                    HIGHLIGHT_CLIP_THRESHOLD_MIN.toDouble(),
                    HIGHLIGHT_CLIP_THRESHOLD_MAX.toDouble(),
                ).roundToInt().coerceIn(HIGHLIGHT_CLIP_THRESHOLD_MIN, HIGHLIGHT_CLIP_THRESHOLD_MAX),
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
    displayLabel: String,
    description: String,
    enabled: Boolean,
    onValueChange: (Double) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(5.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(title, style = MaterialTheme.typography.titleSmall)
            Surface(
                color = ElementBlue.copy(alpha = 0.12f),
                shape = CircleShape,
            ) {
                Text(
                    displayLabel,
                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp),
                    color = ElementBlue,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.SemiBold,
                )
            }
        }
        CompactThresholdSlider(
            value = value,
            enabled = enabled,
            onValueChange = onValueChange,
            modifier = Modifier.fillMaxWidth(),
        )
        Text(
            description,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.bodySmall,
            lineHeight = 16.sp,
        )
    }
}

@Composable
private fun CompactThresholdSlider(
    value: Double,
    enabled: Boolean,
    onValueChange: (Double) -> Unit,
    modifier: Modifier = Modifier,
) {
    var widthPx by remember { mutableStateOf(0) }
    val activeColor = if (enabled) ElementBlue else ElementBlue.copy(alpha = 0.35f)
    val inactiveColor = ElementBorder.copy(alpha = if (enabled) 0.58f else 0.28f)
    val thumbColor = if (enabled) ElementBlue else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)
    val normalized = value.coerceIn(0.0, 1.0).toFloat()
    val updateFromX: (Float) -> Unit = { x ->
        if (enabled && widthPx > 0) {
            onValueChange((x / widthPx.toFloat()).coerceIn(0f, 1f).toDouble())
        }
    }

    Canvas(
        modifier = modifier
            .height(24.dp)
            .onSizeChanged { widthPx = it.width }
            .pointerInput(enabled, widthPx) {
                detectTapGestures { offset -> updateFromX(offset.x) }
            }
            .pointerInput(enabled, widthPx) {
                detectDragGestures { change, _ ->
                    change.consume()
                    updateFromX(change.position.x)
                }
            },
    ) {
        val horizontalPadding = 4.dp.toPx()
        val startX = horizontalPadding
        val endX = size.width - horizontalPadding
        val centerY = size.height / 2f
        val usableWidth = (endX - startX).coerceAtLeast(1f)
        val activeEndX = startX + usableWidth * normalized
        val trackStroke = 7.dp.toPx()
        val thumbStroke = 3.dp.toPx()

        drawLine(
            color = inactiveColor,
            start = Offset(startX, centerY),
            end = Offset(endX, centerY),
            strokeWidth = trackStroke,
            cap = StrokeCap.Round,
        )
        drawLine(
            color = activeColor,
            start = Offset(startX, centerY),
            end = Offset(activeEndX, centerY),
            strokeWidth = trackStroke,
            cap = StrokeCap.Round,
        )
        drawLine(
            color = thumbColor,
            start = Offset(activeEndX, centerY - 9.dp.toPx()),
            end = Offset(activeEndX, centerY + 9.dp.toPx()),
            strokeWidth = thumbStroke,
            cap = StrokeCap.Round,
        )
        drawCircle(
            color = activeColor,
            radius = 2.dp.toPx(),
            center = Offset(endX, centerY),
        )
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
private const val SHADOW_CLIP_THRESHOLD_MIN = 0
private const val SHADOW_CLIP_THRESHOLD_MAX = 15
private const val HIGHLIGHT_CLIP_THRESHOLD_MIN = 235
private const val HIGHLIGHT_CLIP_THRESHOLD_MAX = 255
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

private fun shadowClipThresholdValue(policy: TechnicalAssessmentPolicyUi): Double =
    normalize(
        policy.shadowClipThreshold.toDouble(),
        SHADOW_CLIP_THRESHOLD_MIN.toDouble(),
        SHADOW_CLIP_THRESHOLD_MAX.toDouble(),
    )

private fun highlightClipThresholdValue(policy: TechnicalAssessmentPolicyUi): Double =
    normalize(
        policy.highlightClipThreshold.toDouble(),
        HIGHLIGHT_CLIP_THRESHOLD_MIN.toDouble(),
        HIGHLIGHT_CLIP_THRESHOLD_MAX.toDouble(),
    )

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
            shadowClipThreshold = 2,
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
            shadowClipThreshold = 8,
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
            shadowClipThreshold = 5,
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
    onDisabledClick: (() -> Unit)? = null,
    onCheckedChange: (Boolean) -> Unit,
) {
    val rowClickEnabled = enabled || onDisabledClick != null
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = rowClickEnabled) {
                if (enabled) {
                    onCheckedChange(!checked)
                } else {
                    onDisabledClick?.invoke()
                }
            },
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            title,
            color = if (enabled) {
                MaterialTheme.colorScheme.onSurface
            } else {
                MaterialTheme.colorScheme.onSurfaceVariant
            },
        )
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

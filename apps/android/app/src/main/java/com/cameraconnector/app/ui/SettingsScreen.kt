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
    onCreatePromptPackage: () -> Unit,
    onCreatePromptProfileInPackage: (String) -> Unit,
    onOpenPromptProfile: (String) -> Unit,
    onDeletePromptPackage: (String) -> Unit,
    onDeletePromptProfile: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var collapsedPackages by rememberSaveable { mutableStateOf(emptyList<String>()) }
    val promptPackages = promptProfiles
        .groupBy { promptPackageFolder(it) }
        .toList()
        .sortedWith(
            compareBy<Pair<String, List<PromptProfileUi>>> {
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
                    Text("按提示词包管理摄影评价偏好", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                OutlinedButton(
                    onClick = onCreatePromptPackage,
                    enabled = actionInFlight == null,
                    shape = elementShape,
                ) {
                    Icon(Icons.Outlined.Add, contentDescription = null, modifier = Modifier.size(18.dp))
                    Spacer(Modifier.width(6.dp))
                    Text("新建提示词包")
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
            promptPackages.forEach { (packageFolder, profilesInPackage) ->
                val expanded = packageFolder !in collapsedPackages
                item(key = "package-$packageFolder") {
                    PromptPackageSection(
                        packageFolder = packageFolder,
                        profiles = profilesInPackage.sortedBy { promptProfileDisplayName(it) },
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
                            onCreatePromptProfileInPackage(packageFolder)
                            collapsedPackages = collapsedPackages - packageFolder
                        },
                        onOpenPromptProfile = onOpenPromptProfile,
                        onDeletePackage = { onDeletePromptPackage(packageFolder) },
                        onDeletePromptProfile = onDeletePromptProfile,
                    )
                }
            }
        }
    }
}

@Composable
private fun PromptPackageSection(
    packageFolder: String,
    profiles: List<PromptProfileUi>,
    expanded: Boolean,
    actionInFlight: String?,
    onToggle: () -> Unit,
    onCreate: () -> Unit,
    onOpenPromptProfile: (String) -> Unit,
    onDeletePackage: () -> Unit,
    onDeletePromptProfile: (String) -> Unit,
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
                        PromptProfileRow(
                            profile = profile,
                            onClick = { onOpenPromptProfile(profile.promptProfileId) },
                            onDelete = { onDeletePromptProfile(profile.promptProfileId) },
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
private fun PromptProfileRow(
    profile: PromptProfileUi,
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
                promptProfileDisplayName(profile),
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                promptProfileMetaText(profile),
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

private fun promptProfileMetaText(profile: PromptProfileUi): String =
    listOf(
        promptStyleTagsText(profile),
        sceneProfileLabel(profile.sceneProfile),
        if (profile.builtIn) "内置" else "自定义",
    ).filter { it.isNotBlank() }.distinct().joinToString(" / ")

@Composable
internal fun PromptProfileEditorScreen(
    profile: PromptProfileUi?,
    initialDistributionFolder: String,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onSave: (PromptProfileUi, String, List<String>, String, String, String) -> Unit,
    onCreate: (String, List<String>, String, String, String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val createMode = profile == null
    val builtInProfile = profile?.builtIn == true
    val editableExistingProfile = !createMode && !builtInProfile
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
    var distributionFolder by remember(profile?.promptProfileId, initialDistributionFolder) {
        mutableStateOf(
            when {
                createMode -> initialDistributionFolder
                builtInProfile -> "user"
                else -> profile.distributionFolder.takeIf { it.isNotBlank() } ?: "user"
            },
        )
    }
    var promptText by remember(profile?.promptProfileId, profile?.sharedPreference, profile?.activePromptText) {
        mutableStateOf(profile?.sharedPreference ?: profile?.activePromptText.orEmpty())
    }
    var promptTab by rememberSaveable(profile?.promptProfileId) { mutableStateOf("edit") }
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
                            createMode -> "选择提示词包，保存后进入编辑"
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
                    values = listOf("general", "portrait", "action", "landscape", "custom"),
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
    var selectedPanel by rememberSaveable(projectSettings.projectId) {
        mutableStateOf<ProjectIntelligencePanel?>(null)
    }

    selectedPanel?.let { panel ->
        ProjectIntelligencePanelCard(
            panel = panel,
            projectSettings = projectSettings,
            modelOptions = modelOptions,
            selectedProviderReady = selectedProviderReady,
            selectedPrompt = selectedPrompt,
            promptProfiles = selectablePromptProfiles,
            actionsEnabled = actionsEnabled,
            onBack = { selectedPanel = null },
            onSaveSettings = onSaveSettings,
            onConfigureModelProvider = onConfigureModelProvider,
        )
        return
    }

    ProjectIntelligenceOverviewCard(
        projectSettings = projectSettings,
        modelOptions = modelOptions,
        selectedProviderReady = selectedProviderReady,
        selectedPrompt = selectedPrompt,
        recommendationAction = recommendationAction,
        actionsEnabled = actionsEnabled,
        onOpenPanel = { selectedPanel = it },
        onGenerateProjectRecommendation = onGenerateProjectRecommendation,
    )
}

private enum class ProjectIntelligencePanel {
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
    selectedPrompt: PromptProfileUi?,
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
                title = "场景与技术风险",
                subtitle = "${sceneProfileLabel(projectSettings.sceneProfile)} · ${cvPolicyLabel(projectSettings.cvPolicy)}",
                onClick = { onOpenPanel(ProjectIntelligencePanel.Scene) },
            )
            ProjectIntelligenceMenuRow(
                title = "评价提示词",
                subtitle = selectedPrompt?.let(::promptProfileDisplayName) ?: "未选择",
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
            recommendationAction.disabledReason?.let {
                Text(
                    it,
                    modifier = Modifier.padding(start = 16.dp, end = 16.dp, bottom = 12.dp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
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
private fun ProjectIntelligencePanelCard(
    panel: ProjectIntelligencePanel,
    projectSettings: ProjectEvaluationSettingsUi,
    modelOptions: List<ModelProviderSettingsUi>,
    selectedProviderReady: Boolean,
    selectedPrompt: PromptProfileUi?,
    promptProfiles: List<PromptProfileUi>,
    actionsEnabled: Boolean,
    onBack: () -> Unit,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
    onConfigureModelProvider: () -> Unit,
) {
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
                IconButton(onClick = onBack) {
                    Icon(Icons.AutoMirrored.Outlined.ArrowBack, contentDescription = "返回项目智能")
                }
                Column(Modifier.weight(1f)) {
                    Text(projectIntelligencePanelTitle(panel), style = MaterialTheme.typography.titleLarge)
                    Text(
                        projectIntelligencePanelSubtitle(panel),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
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
                )
                ProjectIntelligencePanel.Scene -> ProjectScenePanel(
                    projectSettings = projectSettings,
                    actionsEnabled = actionsEnabled,
                    onSaveSettings = onSaveSettings,
                )
                ProjectIntelligencePanel.Prompt -> ProjectPromptPanel(
                    projectSettings = projectSettings,
                    selectedPrompt = selectedPrompt,
                    promptProfiles = promptProfiles,
                    actionsEnabled = actionsEnabled,
                    onSaveSettings = onSaveSettings,
                )
            }
        }
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
) {
    SettingsSwitchRow(
        title = "上传后自动评价",
        checked = projectSettings.autoEvaluateOnUpload,
        enabled = actionsEnabled && selectedProviderReady,
        onCheckedChange = { onSaveSettings(projectSettings.copy(autoEvaluateOnUpload = it)) },
    )
    SettingsSwitchRow(
        title = "连拍组自动优选",
        checked = projectSettings.autoBurstRecommendationEnabled,
        enabled = actionsEnabled && selectedProviderReady && projectSettings.projectId.isNotBlank(),
        onCheckedChange = { onSaveSettings(projectSettings.copy(autoBurstRecommendationEnabled = it)) },
    )
    SettingsSwitchRow(
        title = "允许风险照片参与优选",
        checked = projectSettings.allowRiskyModelSelects,
        enabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
        onCheckedChange = { onSaveSettings(projectSettings.copy(allowRiskyModelSelects = it)) },
    )
}

@Composable
private fun ProjectScenePanel(
    projectSettings: ProjectEvaluationSettingsUi,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
) {
    OptionRow(
        title = "项目场景",
        values = listOf("general", "portrait", "action", "landscape", "custom"),
        selected = projectSettings.sceneProfile,
        enabled = actionsEnabled && projectSettings.projectId.isNotBlank(),
        labelForValue = ::sceneProfileLabel,
        onSelected = { onSaveSettings(projectSettings.copy(sceneProfile = it)) },
    )
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

@Composable
private fun ProjectPromptPanel(
    projectSettings: ProjectEvaluationSettingsUi,
    selectedPrompt: PromptProfileUi?,
    promptProfiles: List<PromptProfileUi>,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
) {
    if (promptProfiles.isEmpty()) {
        Text("还没有可用提示词。", color = MaterialTheme.colorScheme.onSurfaceVariant)
        return
    }
    var collapsedPackages by rememberSaveable(
        projectSettings.projectId,
        promptProfiles.joinToString("|") { it.promptProfileId },
    ) {
        mutableStateOf(emptyList<String>())
    }
    val packages = promptProfiles
        .groupBy { promptPackageFolder(it) }
        .toList()
        .sortedWith(
            compareBy<Pair<String, List<PromptProfileUi>>> {
                when (it.first) {
                    "user" -> 0
                    "builtin" -> 2
                    else -> 1
                }
            }.thenBy { promptPackageLabel(it.first) },
        )
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        packages.forEach { (packageFolder, profilesInPackage) ->
            val expanded = packageFolder !in collapsedPackages
            PromptSelectionPackageSection(
                packageFolder = packageFolder,
                profiles = profilesInPackage.sortedBy { promptProfileDisplayName(it) },
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
                    onSaveSettings(projectSettings.copy(promptProfileId = profile.promptProfileId))
                },
            )
        }
    }
}

@Composable
private fun PromptSelectionPackageSection(
    packageFolder: String,
    profiles: List<PromptProfileUi>,
    selectedPrompt: PromptProfileUi?,
    expanded: Boolean,
    enabled: Boolean,
    onToggle: () -> Unit,
    onSelected: (PromptProfileUi) -> Unit,
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
                        selected = profile.promptProfileId == selectedPrompt?.promptProfileId,
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
    profile: PromptProfileUi,
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
                promptProfileDisplayName(profile),
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                promptProfileMetaText(profile),
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
        ProjectIntelligencePanel.Scene -> "场景与技术风险"
        ProjectIntelligencePanel.Prompt -> "评价提示词"
    }

private fun projectIntelligencePanelSubtitle(panel: ProjectIntelligencePanel): String =
    when (panel) {
        ProjectIntelligencePanel.Model -> "选择当前项目使用的模型配置"
        ProjectIntelligencePanel.Workflow -> "控制上传后评价与连拍优选"
        ProjectIntelligencePanel.Scene -> "设置项目场景和技术风险阈值"
        ProjectIntelligencePanel.Prompt -> "选择模型评价使用的摄影偏好"
    }

private fun projectWorkflowSummary(settings: ProjectEvaluationSettingsUi): String =
    listOf(
        "自动评价${if (settings.autoEvaluateOnUpload) "开" else "关"}",
        "连拍优选${if (settings.autoBurstRecommendationEnabled) "开" else "关"}",
        "风险参与${if (settings.allowRiskyModelSelects) "开" else "关"}",
    ).joinToString(" · ")

@Composable
private fun PromptProfileSelector(
    selectedPrompt: PromptProfileUi?,
    promptProfiles: List<PromptProfileUi>,
    expanded: Boolean,
    enabled: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    onSelected: (PromptProfileUi) -> Unit,
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
                            selectedPrompt?.let(::promptProfileDisplayName) ?: "未选择提示词",
                            style = MaterialTheme.typography.titleMedium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            selectedPrompt?.let(::promptProfileMetaText) ?: "用于模型评价、连拍优选和项目优选",
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
                    promptProfiles.forEachIndexed { index, profile ->
                        PromptProfileOptionRow(
                            profile = profile,
                            selected = profile.promptProfileId == selectedPrompt?.promptProfileId,
                            enabled = enabled,
                            onClick = { onSelected(profile) },
                        )
                        if (index != promptProfiles.lastIndex) {
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
private fun PromptProfileOptionRow(
    profile: PromptProfileUi,
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
                promptProfileDisplayName(profile),
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                promptProfileMetaText(profile),
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

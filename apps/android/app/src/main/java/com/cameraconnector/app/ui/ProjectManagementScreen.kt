package com.cameraconnector.app.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.graphics.Bitmap
import android.os.StatFs
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.MoreVert
import androidx.compose.material.icons.outlined.PhotoLibrary
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.SyncAlt
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
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
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
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
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import com.cameraconnector.app.core.ProjectAssetRole
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
internal fun ProjectManagementScreen(
    dashboard: DashboardState,
    projectState: ProjectState,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onEnterProject: (String) -> Unit,
    onConfigureProject: (String) -> Unit,
    onDeleteProject: (String) -> Unit,
    onCreateProject: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var createExpanded by rememberSaveable { mutableStateOf(false) }
    var projectNameInput by rememberSaveable { mutableStateOf("") }
    var pendingDeleteProject by remember { mutableStateOf<ProjectSummary?>(null) }
    val cleanProjectName = projectNameInput.trim()
    val actionsEnabled = actionInFlight == null
    val context = LocalContext.current
    val deviceStorage by produceState(initialValue = DeviceStorageSnapshot.empty(), key1 = context) {
        value = withContext(Dispatchers.IO) {
            loadDeviceStorageSnapshot(context)
        }
    }

    Box(modifier = modifier.fillMaxSize()) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {

        actionError?.let { message ->
            item { ActionMessageCard(title = "\u64cd\u4f5c\u5931\u8d25", message = message, onClose = onClearActionError) }
        }

        item {
            ProjectGlobalStatusCard(
                dashboard = dashboard,
                projectState = projectState,
                storage = deviceStorage,
                modifier = Modifier.fillMaxWidth(),
            )
        }

        if (projectState.projects.isEmpty()) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "\u8fd8\u6ca1\u6709\u62cd\u6444\u9879\u76ee\uff0c\u8bf7\u5148\u65b0\u5efa\u9879\u76ee\u3002",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            items(projectState.projects, key = { it.id }) { project ->
                ProjectManagementRow(
                    project = project,
                    selected = project.id == projectState.activeProjectId,
                    enterEnabled = actionsEnabled && project.canBeActiveProject,
                    actionsEnabled = actionsEnabled,
                    onEnter = { onEnterProject(project.id) },
                    onConfigure = { onConfigureProject(project.id) },
                    onDelete = { pendingDeleteProject = project },
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }

        item {
            Button(
                onClick = { createExpanded = true },
                enabled = actionsEnabled && !createExpanded,
                modifier = Modifier.fillMaxWidth(),
                shape = elementShape,
                colors = ButtonDefaults.buttonColors(
                    containerColor = ElementBlue,
                    contentColor = ElementOnAccent,
                ),
            ) {
                Text("\u65b0\u5efa\u9879\u76ee")
            }
        }

        if (createExpanded) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(
                        modifier = Modifier.padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Text("\u521b\u5efa\u62cd\u6444\u9879\u76ee", style = MaterialTheme.typography.titleMedium)
                        OutlinedTextField(
                            value = projectNameInput,
                            onValueChange = { projectNameInput = it },
                            modifier = Modifier.fillMaxWidth(),
                            label = { Text("\u9879\u76ee\u540d\u79f0") },
                            singleLine = true,
                            enabled = actionsEnabled,
                        )
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End),
                        ) {
                            OutlinedButton(
                                onClick = {
                                    createExpanded = false
                                    projectNameInput = ""
                                },
                                enabled = actionsEnabled,
                                shape = elementShape,
                            ) {
                                Text("\u53d6\u6d88")
                            }
                             Button(
                                 onClick = {
                                    onCreateProject(cleanProjectName)
                                     createExpanded = false
                                     projectNameInput = ""
                                 },
                                 enabled = actionsEnabled && cleanProjectName.isNotBlank(),
                                 shape = elementShape,
                             ) {
                                Text("\u521b\u5efa\u5e76\u914d\u7f6e")
                             }
                         }
                     }
                }
            }
        }
        }
        actionInFlight?.let { action ->
            ActionLoadingOverlay(action)
        }
    }
    pendingDeleteProject?.let { project ->
        AlertDialog(
            onDismissRequest = { pendingDeleteProject = null },
            title = { Text("删除项目") },
            text = {
                Text("将删除「${project.name}」下的照片、分组、评价、推荐和本地文件。此操作不可撤销。")
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        pendingDeleteProject = null
                        onDeleteProject(project.id)
                    },
                ) {
                    Text("删除", color = ElementDanger)
                }
            },
            dismissButton = {
                TextButton(onClick = { pendingDeleteProject = null }) {
                    Text("取消")
                }
            },
            containerColor = ElementSurface,
            titleContentColor = MaterialTheme.colorScheme.onSurface,
            textContentColor = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
internal fun ProjectGlobalStatusCard(
    dashboard: DashboardState,
    projectState: ProjectState,
    storage: DeviceStorageSnapshot,
    modifier: Modifier = Modifier,
) {
    val onlineConnections = dashboard.accounts.sumOf { it.activeConnections }
    val globalAssets = dashboard.globalAssets

    ElementCard(modifier = modifier) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text("\u5168\u5c40\u72b6\u6001", style = MaterialTheme.typography.titleMedium)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                ProjectInlineMetric(
                    value = projectState.projects.size.toString(),
                    label = "\u9879\u76ee",
                    accentColor = ElementBlue,
                    modifier = Modifier.weight(1f),
                )
                ProjectInlineMetric(
                    value = onlineConnections.toString(),
                    label = "\u5728\u7ebf\u8fde\u63a5",
                    accentColor = if (onlineConnections > 0) ElementSuccess else ElementInfo,
                    modifier = Modifier.weight(1f),
                )
                ProjectInlineMetric(
                    value = globalAssets.photoCount.toString(),
                    label = "\u7167\u7247",
                    accentColor = ElementPurple,
                    modifier = Modifier.weight(1f),
                )
            }
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(1.dp)
                    .background(ElementCardBorder.copy(alpha = 0.48f)),
            )
            DeviceStorageContent(
                storage = storage,
                projectBytes = globalAssets.storageBytes,
            )
        }
    }
}


@Composable
private fun DeviceStorageContent(
    storage: DeviceStorageSnapshot,
    projectBytes: Long,
    modifier: Modifier = Modifier,
) {
    val segments = storageBarSegments(storage, projectBytes)
    val usedStorageColor = storage.accentColor()
    val projectStorageColor = ElementBlue.copy(alpha = 0.64f)
    val projectStorageTextColor = ElementBlue.copy(alpha = 0.72f)
    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text("\u624b\u673a\u5b58\u50a8", style = MaterialTheme.typography.titleMedium)
                Text(
                    "\u5185\u90e8\u5b58\u50a8\u4f7f\u7528\u60c5\u51b5",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodySmall,
                )
            }
            Text(
                storage.usedPercentLabel,
                color = storage.accentColor(),
                fontWeight = FontWeight.Bold,
                style = MaterialTheme.typography.titleMedium,
            )
        }
        val usedRatio = (segments.projectRatio + segments.otherUsedRatio).coerceIn(0f, 1f)
        val projectShareOfUsed = if (usedRatio > 0f) {
            (segments.projectRatio / usedRatio).coerceIn(0f, 1f)
        } else {
            0f
        }
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(10.dp)
                .clip(CircleShape)
                .background(ElementControlSurface),
        ) {
            if (usedRatio > 0f) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth(usedRatio)
                        .height(10.dp)
                        .clip(CircleShape)
                        .background(usedStorageColor),
                ) {
                    if (projectShareOfUsed > 0f) {
                        Box(
                            modifier = Modifier
                                .align(Alignment.CenterEnd)
                                .fillMaxWidth(projectShareOfUsed)
                                .height(10.dp)
                                .clip(CircleShape)
                                .background(projectStorageColor),
                        )
                    }
                }
            }
        }
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            StorageValueColumn(
                "\u5df2\u7528",
                formatBytes(storage.usedBytes),
                valueColor = storage.accentColor(),
                indicatorColor = storage.accentColor().copy(alpha = 0.7f),
            )
            StorageValueColumn(
                "\u9879\u76ee",
                formatBytes(projectBytes),
                valueColor = projectStorageTextColor,
                indicatorColor = projectStorageColor,
            )
            StorageValueColumn("\u53ef\u7528", formatBytes(storage.availableBytes))
            StorageValueColumn("\u603b\u91cf", formatBytes(storage.totalBytes))
        }
    }
}

@Composable
private fun StorageValueColumn(
    label: String,
    value: String,
    valueColor: Color? = null,
    indicatorColor: Color? = null,
) {
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Row(
            horizontalArrangement = Arrangement.spacedBy(5.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            indicatorColor?.let { color ->
                Box(
                    modifier = Modifier
                        .size(6.dp)
                        .clip(CircleShape)
                        .background(color),
                )
            }
            Text(
                label,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.labelSmall,
            )
        }
        Text(
            value,
            color = valueColor ?: MaterialTheme.colorScheme.onSurface,
            fontWeight = FontWeight.SemiBold,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

@Composable
internal fun ProjectInlineMetric(
    value: String,
    label: String,
    accentColor: Color,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .clip(elementShape)
            .background(accentColor.copy(alpha = 0.08f))
            .padding(10.dp),
    ) {
        Text(value, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
        Spacer(Modifier.height(4.dp))
        Text(
            label,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.labelMedium,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

internal data class DeviceStorageSnapshot(
    val totalBytes: Long,
    val availableBytes: Long,
) {
    val usedBytes: Long = (totalBytes - availableBytes).coerceAtLeast(0L)
    val usedRatio: Float = if (totalBytes <= 0L) 0f else usedBytes.toFloat() / totalBytes.toFloat()
    val usedPercentLabel: String = "${(usedRatio * 100).toInt()}%"

    fun accentColor(): Color = when {
        usedRatio >= 0.9f -> ElementDanger
        usedRatio >= 0.75f -> ElementWarning
        else -> ElementSuccess
    }

    companion object {
        fun empty(): DeviceStorageSnapshot = DeviceStorageSnapshot(totalBytes = 0L, availableBytes = 0L)
    }
}

internal data class StorageBarSegments(
    val projectRatio: Float,
    val otherUsedRatio: Float,
)

internal fun storageBarSegments(
    storage: DeviceStorageSnapshot,
    projectBytes: Long,
): StorageBarSegments {
    if (storage.totalBytes <= 0L) {
        return StorageBarSegments(projectRatio = 0f, otherUsedRatio = 0f)
    }
    val usedRatio = storage.usedRatio.coerceIn(0f, 1f)
    val projectRatio = (projectBytes.coerceAtLeast(0L).toFloat() / storage.totalBytes.toFloat())
        .coerceIn(0f, usedRatio)
    return StorageBarSegments(
        projectRatio = projectRatio,
        otherUsedRatio = (usedRatio - projectRatio).coerceAtLeast(0f),
    )
}

private fun loadDeviceStorageSnapshot(context: Context): DeviceStorageSnapshot {
    val statFs = StatFs(context.filesDir.absolutePath)
    val totalBytes = statFs.totalBytes.coerceAtLeast(0L)
    val availableBytes = statFs.availableBytes.coerceAtLeast(0L)
    return DeviceStorageSnapshot(
        totalBytes = totalBytes,
        availableBytes = availableBytes.coerceAtMost(totalBytes),
    )
}

private fun formatBytes(bytes: Long): String {
    if (bytes <= 0L) return "0 B"
    val units = listOf("B", "KB", "MB", "GB", "TB")
    var value = bytes.toDouble()
    var unitIndex = 0
    while (value >= 1024.0 && unitIndex < units.lastIndex) {
        value /= 1024.0
        unitIndex += 1
    }
    return if (value >= 10 || unitIndex == 0) {
        "${value.toInt()} ${units[unitIndex]}"
    } else {
        String.format("%.1f %s", value, units[unitIndex])
    }
}

@Composable
internal fun ProjectManagementRow(
    project: ProjectSummary,
    selected: Boolean,
    enterEnabled: Boolean,
    actionsEnabled: Boolean,
    onEnter: () -> Unit,
    onConfigure: () -> Unit,
    onDelete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val lifecycle = projectLifecycleUi(project, selected, actionsEnabled = true)
    var overflowExpanded by remember { mutableStateOf(false) }
    ElementCard(modifier = modifier) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier = Modifier
                    .size(10.dp)
                    .clip(CircleShape)
                    .background(if (selected) ElementSuccess else ElementBlue),
            )
            Spacer(Modifier.width(12.dp))
            Column(Modifier.weight(1f)) {
                Text(
                    project.name,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Spacer(Modifier.height(4.dp))
                Text(
                    lifecycle.statusLabel,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Spacer(Modifier.width(12.dp))
            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                OutlinedButton(
                    onClick = onConfigure,
                    enabled = actionsEnabled,
                    modifier = Modifier.height(40.dp),
                    shape = elementShape,
                    contentPadding = PaddingValues(horizontal = 14.dp, vertical = 0.dp),
                ) {
                    Text("配置")
                }
                Button(
                    onClick = onEnter,
                    enabled = enterEnabled,
                    modifier = Modifier.height(40.dp),
                    shape = elementShape,
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 0.dp),
                ) {
                    Text("进入")
                }
                Box {
                    IconButton(
                        onClick = { overflowExpanded = true },
                        enabled = actionsEnabled,
                        modifier = Modifier.size(30.dp),
                    ) {
                        Icon(
                            Icons.Outlined.MoreVert,
                            contentDescription = "更多项目操作",
                            tint = if (actionsEnabled) {
                                MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.68f)
                            } else {
                                MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.32f)
                            },
                            modifier = Modifier.size(16.dp),
                        )
                    }
                    DropdownMenu(
                        expanded = overflowExpanded,
                        onDismissRequest = { overflowExpanded = false },
                        containerColor = ElementSurface,
                    ) {
                        DropdownMenuItem(
                            text = { Text("删除项目", color = ElementDanger) },
                            leadingIcon = {
                                Icon(
                                    Icons.Outlined.Delete,
                                    contentDescription = null,
                                    tint = ElementDanger,
                                )
                            },
                            onClick = {
                                overflowExpanded = false
                                onDelete()
                            },
                        )
                    }
                }
            }
        }
    }
}

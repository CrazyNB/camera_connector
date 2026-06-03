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
import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetQuery
import com.cameraconnector.app.core.InboxAssetRole
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
    cameraConnectHost: String,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onEnterProject: (String) -> Unit,
    onConfigureProject: (String) -> Unit,
    onCreateAndEnterProject: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var createExpanded by rememberSaveable { mutableStateOf(false) }
    var projectNameInput by rememberSaveable { mutableStateOf("") }
    val cleanProjectName = projectNameInput.trim()
    val actionsEnabled = actionInFlight == null

    Box(modifier = modifier.fillMaxSize()) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
        item {
            Column {
                Text("项目管理", style = MaterialTheme.typography.headlineMedium)
                Spacer(Modifier.height(4.dp))
                Text(
                    "启动后默认进入，用户选择或新建项目",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        item {
            ProjectGlobalStatusCard(
                dashboard = dashboard,
                projectState = projectState,
                cameraConnectHost = cameraConnectHost,
                modifier = Modifier.fillMaxWidth(),
            )
        }

        if (projectState.projects.isEmpty()) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "还没有拍摄项目，请先新建项目。",
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
                    enabled = actionsEnabled && project.canBeActiveProject,
                    onEnter = { onEnterProject(project.id) },
                    onConfigure = { onConfigureProject(project.id) },
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
                Text("新建项目")
            }
        }

        if (createExpanded) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(
                        modifier = Modifier.padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Text("创建拍摄项目", style = MaterialTheme.typography.titleMedium)
                        OutlinedTextField(
                            value = projectNameInput,
                            onValueChange = { projectNameInput = it },
                            modifier = Modifier.fillMaxWidth(),
                            label = { Text("项目名称") },
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
                                Text("取消")
                            }
                            Button(
                                onClick = {
                                    onCreateAndEnterProject(cleanProjectName)
                                    createExpanded = false
                                    projectNameInput = ""
                                },
                                enabled = actionsEnabled && cleanProjectName.isNotBlank(),
                                shape = elementShape,
                            ) {
                                Text("创建并进入")
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
}

@Composable
internal fun ProjectGlobalStatusCard(
    dashboard: DashboardState,
    projectState: ProjectState,
    cameraConnectHost: String,
    modifier: Modifier = Modifier,
) {
    val project = projectState.activeProjectSummary()
    val onlineConnections = dashboard.accounts.sumOf { it.activeConnections }

    ElementCard(modifier = modifier) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text("全局状态", style = MaterialTheme.typography.titleMedium)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                ProjectInlineMetric(
                    value = projectState.projects.size.toString(),
                    label = "项目",
                    accentColor = ElementBlue,
                    modifier = Modifier.weight(1f),
                )
                ProjectInlineMetric(
                    value = onlineConnections.toString(),
                    label = "在线连接",
                    accentColor = if (onlineConnections > 0) ElementSuccess else ElementInfo,
                    modifier = Modifier.weight(1f),
                )
                ProjectInlineMetric(
                    value = dashboard.publishQueue.pendingCount.toString(),
                    label = "待发布",
                    accentColor = ElementWarning,
                    modifier = Modifier.weight(1f),
                )
            }
            ProjectStatusLine(
                label = "当前项目",
                value = project?.name ?: "未选择",
            )
            ProjectStatusLine(
                label = "接收端点",
                value = receiverEndpointLabel(dashboard.receiver, cameraConnectHost),
            )
            ProjectStatusLine(
                label = "接收状态",
                value = receiverPhaseLabel(dashboard.receiver.phase),
            )
        }
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

@Composable
internal fun ProjectStatusLine(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(label, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.width(12.dp))
        Text(
            value,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            fontWeight = FontWeight.SemiBold,
        )
    }
}

@Composable
internal fun ProjectManagementRow(
    project: ProjectSummary,
    selected: Boolean,
    enabled: Boolean,
    onEnter: () -> Unit,
    onConfigure: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val lifecycle = projectLifecycleUi(project, selected, actionsEnabled = true)
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
                    "${lifecycle.statusLabel} · ${project.slug}",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Spacer(Modifier.width(12.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedButton(
                    onClick = onConfigure,
                    enabled = enabled,
                    shape = elementShape,
                    contentPadding = PaddingValues(horizontal = 12.dp, vertical = 0.dp),
                ) {
                    Text("配置")
                }
                Button(
                    onClick = onEnter,
                    enabled = enabled,
                    shape = elementShape,
                    contentPadding = PaddingValues(horizontal = 12.dp, vertical = 0.dp),
                ) {
                    Text("进入")
                }
            }
        }
    }
}

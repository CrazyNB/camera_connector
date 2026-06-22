package com.cameraconnector.app.ui

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.AutoAwesome
import androidx.compose.material.icons.outlined.FilterList
import androidx.compose.material.icons.outlined.KeyboardArrowUp
import androidx.compose.material.icons.outlined.MoreVert
import androidx.compose.material.icons.outlined.Share
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.DEFAULT_FTP_RECEIVER_PORT
import com.cameraconnector.app.core.DEFAULT_LISTEN_HOST
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState

@Composable
internal fun ProjectReceiverStatusStrip(
    dashboard: DashboardState,
    projectState: ProjectState,
    lanShareAction: LanShareActionUi,
    lanShareUrl: String?,
    onOpenProjects: () -> Unit,
    onExpand: () -> Unit,
    onOpenProjectIntelligence: () -> Unit,
    onConfigureGuestSelection: () -> Unit,
    onConfigureProjectSync: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val project = projectState.activeProjectSummary()
    Surface(
        modifier = modifier.clickable(onClick = onExpand),
        color = ElementControlSurface.copy(alpha = 0.86f),
        contentColor = MaterialTheme.colorScheme.onSurface,
        shape = RoundedCornerShape(14.dp),
        border = BorderStroke(1.dp, ElementBorder),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(start = 12.dp, top = 9.dp, end = 4.dp, bottom = 9.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            ReceiverHeaderIconButton(
                imageVector = Icons.AutoMirrored.Outlined.ArrowBack,
                contentDescription = "\u8fd4\u56de\u9879\u76ee\u7ba1\u7406",
                onClick = onOpenProjects,
            )
            Spacer(Modifier.width(2.dp))
            Row(
                modifier = Modifier.weight(1f),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(
                    modifier = Modifier
                        .size(10.dp)
                        .background(
                            if (dashboard.receiver.running) ElementSuccess else ElementInfo,
                            CircleShape,
                        ),
                )
                Spacer(Modifier.width(9.dp))
                Column(Modifier.weight(1f)) {
                    Text(
                        project?.name ?: "当前项目",
                        style = MaterialTheme.typography.bodyMedium,
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        receiverCollapsedStatusLabel(dashboard.receiver),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
            Spacer(Modifier.width(4.dp))
            ReceiverHeaderIconButton(
                imageVector = Icons.Outlined.AutoAwesome,
                contentDescription = "\u9879\u76ee\u667a\u80fd",
                onClick = onOpenProjectIntelligence,
                enabled = project != null,
            )
            ProjectReceiverLanShareMenu(
                lanShareAction = lanShareAction,
                lanShareUrl = lanShareUrl,
                onConfigureGuestSelection = onConfigureGuestSelection,
                onConfigureProjectSync = onConfigureProjectSync,
            )
        }
    }
}

@Composable
private fun ReceiverHeaderIconButton(
    imageVector: ImageVector,
    contentDescription: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    tint: Color = ElementBlue,
) {
    IconButton(
        onClick = onClick,
        enabled = enabled,
        modifier = modifier.size(32.dp),
    ) {
        Icon(
            imageVector = imageVector,
            contentDescription = contentDescription,
            tint = if (enabled) tint else MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(17.dp),
        )
    }
}

@Composable
private fun ReceiverCollapseButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val transition = rememberInfiniteTransition(label = "receiver-collapse")
    val offsetY by transition.animateFloat(
        initialValue = 0f,
        targetValue = -3f,
        animationSpec = infiniteRepeatable(
            animation = tween(durationMillis = 850),
            repeatMode = RepeatMode.Reverse,
        ),
        label = "receiver-collapse-y",
    )
    ReceiverHeaderIconButton(
        imageVector = Icons.Outlined.KeyboardArrowUp,
        contentDescription = "\u6536\u8d77\u542f\u52a8\u9875",
        onClick = onClick,
        modifier = modifier.graphicsLayer {
            translationY = offsetY
            alpha = 0.94f
        },
    )
}


@Composable
private fun ProjectReceiverLanShareMenu(
    lanShareAction: LanShareActionUi,
    lanShareUrl: String?,
    onConfigureGuestSelection: () -> Unit,
    onConfigureProjectSync: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var expanded by remember { mutableStateOf(false) }
    val sharingActive = lanShareUrl != null
    val actionTint = if (sharingActive) ElementSuccess else ElementBlue
    Box(modifier = modifier) {
        ReceiverHeaderIconButton(
            imageVector = Icons.Outlined.MoreVert,
            contentDescription = "\u66f4\u591a\u9879\u76ee\u64cd\u4f5c",
            onClick = { expanded = true },
            tint = actionTint,
        )
        DropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            containerColor = ElementBackground.copy(alpha = 0.88f),
            shape = RoundedCornerShape(10.dp),
            tonalElevation = 0.dp,
            shadowElevation = 0.dp,
            border = BorderStroke(1.dp, ElementBorder.copy(alpha = 0.45f)),
        ) {
            lanShareMenuItems().forEach { item ->
                DropdownMenuItem(
                    text = {
                        Text(
                            text = item.label,
                            style = MaterialTheme.typography.labelMedium,
                            fontSize = 13.sp,
                            fontWeight = FontWeight.SemiBold,
                        )
                    },
                    modifier = Modifier.height(38.dp),
                    contentPadding = PaddingValues(horizontal = 12.dp, vertical = 2.dp),
                    enabled = lanShareUrl != null || lanShareAction.enabled || lanShareAction.disabledReason != null,
                    onClick = {
                        expanded = false
                        when (item.action) {
                            LanShareMenuAction.GuestSelection -> onConfigureGuestSelection()
                            LanShareMenuAction.ProjectSync -> onConfigureProjectSync()
                        }
                    },
                    leadingIcon = {
                        Icon(
                            imageVector = when (item.action) {
                                LanShareMenuAction.GuestSelection -> Icons.Outlined.FilterList
                                LanShareMenuAction.ProjectSync -> Icons.Outlined.Share
                            },
                            contentDescription = null,
                            tint = actionTint,
                            modifier = Modifier.size(16.dp),
                        )
                    },
                )
            }
        }
    }
}

@Composable
internal fun ProjectLaunchHeader(
    projectState: ProjectState,
    actionsEnabled: Boolean,
    lanShareAction: LanShareActionUi,
    lanShareUrl: String?,
    onOpenProjects: () -> Unit,
    onOpenProjectIntelligence: () -> Unit,
    onConfigureGuestSelection: () -> Unit,
    onConfigureProjectSync: () -> Unit,
    onCollapse: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val project = projectState.activeProjectSummary()
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(36.dp),
    ) {
        Row(
            modifier = Modifier
                .align(Alignment.CenterStart)
                .fillMaxWidth(0.58f),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            ReceiverHeaderIconButton(
                imageVector = Icons.AutoMirrored.Outlined.ArrowBack,
                contentDescription = "\u8fd4\u56de\u9879\u76ee\u7ba1\u7406",
                onClick = onOpenProjects,
                enabled = actionsEnabled,
            )
            Spacer(Modifier.width(4.dp))
            Text(
                project?.name ?: "项目",
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        ReceiverCollapseButton(
            onClick = onCollapse,
            modifier = Modifier
                .align(Alignment.Center)
        )
        Row(
            modifier = Modifier.align(Alignment.CenterEnd),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            ReceiverHeaderIconButton(
                imageVector = Icons.Outlined.AutoAwesome,
                contentDescription = "\u9879\u76ee\u667a\u80fd",
                onClick = onOpenProjectIntelligence,
                enabled = project != null,
            )
            ProjectReceiverLanShareMenu(
                lanShareAction = lanShareAction,
                lanShareUrl = lanShareUrl,
                onConfigureGuestSelection = onConfigureGuestSelection,
                onConfigureProjectSync = onConfigureProjectSync,
            )
        }
    }
}

@Composable
internal fun ProjectReceiverLaunchPanel(
    dashboard: DashboardState,
    projectState: ProjectState,
    notificationPermissionGranted: Boolean,
    actionsEnabled: Boolean,
    lanShareAction: LanShareActionUi,
    lanShareUrl: String?,
    onOpenProjects: () -> Unit,
    onOpenProjectIntelligence: () -> Unit,
    onConfigureGuestSelection: () -> Unit,
    onConfigureProjectSync: () -> Unit,
    onConfigureAccount: () -> Unit,
    onRequestNotificationPermission: () -> Unit,
    onStartReceiver: (ReceiverSettings) -> Unit,
    onStopReceiver: () -> Unit,
    onRetryFailedPublishes: () -> Unit,
    onCollapse: () -> Unit,
    endpointCandidates: List<ReceiverLanEndpointCandidate>,
    modifier: Modifier = Modifier,
) {
    var protocol by remember { mutableStateOf("FTP") }
    val endpointRows = receiverCameraEndpointRows(
        candidates = endpointCandidates,
        port = DEFAULT_FTP_RECEIVER_PORT,
    )
    val receiverSettings = ReceiverSettings(
        protocol = protocol,
        host = DEFAULT_LISTEN_HOST,
        ftpPort = DEFAULT_FTP_RECEIVER_PORT,
        sftpPort = DEFAULT_FTP_RECEIVER_PORT,
        outputLabel = dashboard.receiver.outputLabel,
    )
    val onlineConnections = dashboard.accounts.sumOf { it.activeConnections }
    val receiverBusy = receiverPhaseBusy(dashboard.receiver.phase)
    val startBlockReason = receiverStartBlockReason(
        running = dashboard.receiver.running,
        busy = receiverBusy,
        actionsEnabled = actionsEnabled,
        notificationPermissionGranted = notificationPermissionGranted,
        accountCount = dashboard.accounts.size,
    )
    var visibleStartBlockReason by remember { mutableStateOf<ReceiverStartBlockReason?>(null) }

    visibleStartBlockReason?.let { reason ->
        ReceiverStartBlockedDialog(
            reason = reason,
            onDismiss = { visibleStartBlockReason = null },
            onConfigureAccount = {
                visibleStartBlockReason = null
                onConfigureAccount()
            },
            onRequestNotificationPermission = {
                visibleStartBlockReason = null
                onRequestNotificationPermission()
            },
        )
    }

    ElementCard(modifier = modifier.fillMaxWidth()) {
        Column(Modifier.padding(14.dp)) {
            ProjectLaunchHeader(
                projectState = projectState,
                actionsEnabled = actionsEnabled,
                lanShareAction = lanShareAction,
                lanShareUrl = lanShareUrl,
                onOpenProjects = onOpenProjects,
                onOpenProjectIntelligence = onOpenProjectIntelligence,
                onConfigureGuestSelection = onConfigureGuestSelection,
                onConfigureProjectSync = onConfigureProjectSync,
                onCollapse = onCollapse,
            )
            Spacer(Modifier.height(10.dp))
            ReceiverHeroControl(
                running = dashboard.receiver.running,
                phase = dashboard.receiver.phase,
                onlineConnections = onlineConnections,
                accountCount = dashboard.accounts.size,
                publishQueue = dashboard.publishQueue,
                message = dashboard.receiver.message,
                enabled = actionsEnabled && !receiverBusy,
                retryEnabled = actionsEnabled,
                onToggleReceiver = {
                    if (dashboard.receiver.running) {
                        onStopReceiver()
                    } else if (receiverBusy) {
                        visibleStartBlockReason = ReceiverStartBlockReason.Busy
                    } else if (startBlockReason == null) {
                        onStartReceiver(receiverSettings)
                    } else {
                        visibleStartBlockReason = startBlockReason
                    }
                },
                onRetryFailedPublishes = onRetryFailedPublishes,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(12.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                ProtocolSegment(
                    label = "FTP",
                    selected = protocol == "FTP",
                    enabled = actionsEnabled && !dashboard.receiver.running && !receiverBusy,
                    onClick = { protocol = "FTP" },
                    modifier = Modifier.weight(1f),
                )
                ProtocolSegment(
                    label = "STC 开发中",
                    selected = false,
                    enabled = false,
                    onClick = {},
                    modifier = Modifier.weight(1f),
                )
            }
            Spacer(Modifier.height(8.dp))
            ReceiverCameraEndpointList(rows = endpointRows)
            Spacer(Modifier.height(8.dp))
            Text(
                "输出目录：${dashboard.receiver.outputLabel}",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun ReceiverCameraEndpointList(
    rows: List<ReceiverCameraEndpointRowUi>,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .border(BorderStroke(1.dp, ElementBorder), RoundedCornerShape(6.dp))
            .background(ElementControlSurface.copy(alpha = 0.45f), RoundedCornerShape(6.dp))
            .padding(12.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Text(
            text = "\u76f8\u673a FTP \u53ef\u586b\u5199\u5730\u5740",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            fontWeight = FontWeight.SemiBold,
        )
        if (rows.size > 1) {
            Text(
                text = "\u76f8\u673a\u8fde\u63a5\u54ea\u4e2a\u7f51\u7edc\uff0c\u5c31\u5728\u76f8\u673a FTP \u8bbe\u7f6e\u91cc\u586b\u5bf9\u5e94\u5730\u5740\u3002",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        rows.forEach { row ->
            ReceiverCameraEndpointRow(row = row)
        }
    }
}

@Composable
private fun ReceiverCameraEndpointRow(row: ReceiverCameraEndpointRowUi) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(ElementControlSurface.copy(alpha = 0.62f), RoundedCornerShape(6.dp))
            .padding(horizontal = 10.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = row.label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Spacer(Modifier.width(12.dp))
        Text(
            text = row.endpoint,
            modifier = Modifier.weight(1f),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurface,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

internal fun receiverCollapsedStatusLabel(receiver: ReceiverState): String =
    receiverPhaseLabel(receiver.phase)


@Composable
internal fun ReceiverStartBlockedDialog(
    reason: ReceiverStartBlockReason,
    onDismiss: () -> Unit,
    onConfigureAccount: () -> Unit,
    onRequestNotificationPermission: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(
                when (reason) {
                    ReceiverStartBlockReason.MissingAccount -> "需要先配置账号"
                    ReceiverStartBlockReason.MissingNotificationPermission -> "需要通知权限"
                    ReceiverStartBlockReason.Busy -> "正在处理"
                },
            )
        },
        text = {
            Text(
                when (reason) {
                    ReceiverStartBlockReason.MissingAccount ->
                        "\u63a5\u6536\u670d\u52a1\u4f7f\u7528\u8d26\u53f7\u8ba4\u8bc1\u3002\u8bf7\u5148\u521b\u5efa\u76f8\u673a\u8d26\u53f7\uff0c\u518d\u542f\u52a8\u63a5\u6536\u3002"
                    ReceiverStartBlockReason.MissingNotificationPermission ->
                        "\u63a5\u6536\u670d\u52a1\u4f1a\u4ee5\u524d\u53f0\u670d\u52a1\u8fd0\u884c\uff0c\u9700\u8981\u5148\u5141\u8bb8\u901a\u77e5\u6743\u9650\u3002"
                    ReceiverStartBlockReason.Busy ->
                        "\u5f53\u524d\u8fd8\u6709\u64cd\u4f5c\u672a\u5b8c\u6210\uff0c\u8bf7\u7a0d\u540e\u518d\u542f\u52a8\u63a5\u6536\u3002"
                },
            )
        },
        confirmButton = {
            TextButton(
                onClick = when (reason) {
                    ReceiverStartBlockReason.MissingAccount -> onConfigureAccount
                    ReceiverStartBlockReason.MissingNotificationPermission -> onRequestNotificationPermission
                    ReceiverStartBlockReason.Busy -> onDismiss
                },
            ) {
                Text(
                    when (reason) {
                        ReceiverStartBlockReason.MissingAccount -> "\u53bb\u914d\u7f6e\u8d26\u53f7"
                        ReceiverStartBlockReason.MissingNotificationPermission -> "\u5f00\u542f\u6743\u9650"
                        ReceiverStartBlockReason.Busy -> "\u77e5\u9053\u4e86"
                    },
                )
            }
        },
        dismissButton = {
            if (reason != ReceiverStartBlockReason.Busy) {
                TextButton(onClick = onDismiss) {
                    Text("取消")
                }
            }
        },
    )
}

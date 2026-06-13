package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.PublishQueueState

@Composable
internal fun ReceiverHeroControl(
    running: Boolean,
    phase: String,
    onlineConnections: Int,
    accountCount: Int,
    publishQueue: PublishQueueState,
    message: String?,
    enabled: Boolean,
    retryEnabled: Boolean,
    onToggleReceiver: () -> Unit,
    onRetryFailedPublishes: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val busy = receiverPhaseBusy(phase)
    Column(
        modifier = modifier.padding(vertical = 8.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        PowerButton(
            running = running,
            phase = phase,
            busy = busy,
            enabled = enabled && !busy,
            onClick = onToggleReceiver,
        )
        Spacer(Modifier.height(12.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            ElementTag(
                text = receiverPhaseLabel(phase),
                color = if (running) ElementSuccess else ElementInfo,
            )
            ElementTag(
                text = if (onlineConnections > 0) "\u5728\u7ebf\u8fde\u63a5 $onlineConnections" else "\u672a\u8fde\u63a5",
                color = if (onlineConnections > 0) ElementSuccess else ElementInfo,
            )
            ElementTag(text = "已配置账号 $accountCount", color = ElementBlue)
            publishQueueAttentionLabel(publishQueue)?.let { label ->
                ElementTag(
                    text = label,
                    color = if (publishQueue.failedCount > 0) ElementDanger else ElementInfo,
                )
            }
        }
        if (publishQueueRetryActionVisible(publishQueue)) {
            Spacer(Modifier.height(10.dp))
            OutlinedButton(
                onClick = onRetryFailedPublishes,
                enabled = retryEnabled,
                shape = elementShape,
                colors = ButtonDefaults.outlinedButtonColors(contentColor = ElementDanger),
            ) {
                Icon(
                    Icons.Outlined.Refresh,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp),
                )
                Spacer(Modifier.width(6.dp))
                Text("\u91cd\u8bd5\u5199\u5165")
            }
        }
        message?.takeIf { running }?.let {
            Spacer(Modifier.height(8.dp))
            Text(it, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
internal fun PowerButton(
    running: Boolean,
    phase: String,
    busy: Boolean,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Button(
        onClick = onClick,
        enabled = enabled,
        modifier = Modifier.size(136.dp),
        shape = CircleShape,
        colors = ButtonDefaults.buttonColors(
            containerColor = if (running) ElementDanger else ElementBlue,
            disabledContainerColor = ElementInfo.copy(alpha = 0.35f),
        ),
        contentPadding = PaddingValues(0.dp),
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(
                when {
                    running -> "\u505c\u6b62"
                    busy -> receiverPhaseLabel(phase)
                    else -> "\u542f\u52a8"
                },
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
            )
            Spacer(Modifier.height(4.dp))
            Text(
                when {
                    running -> "\u63a5\u6536\u670d\u52a1"
                    busy -> "\u8bf7\u7a0d\u5019"
                    else -> "\u5f00\u59cb\u63a5\u6536"
                },
            )
        }
    }
}

@Composable
internal fun ProtocolSegment(
    label: String,
    selected: Boolean,
    enabled: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    OutlinedButton(
        onClick = onClick,
        enabled = enabled,
        modifier = modifier,
        border = BorderStroke(1.dp, if (selected) ElementBlue else ElementBorder),
        colors = ButtonDefaults.outlinedButtonColors(
            containerColor = if (selected) ElementBlue else ElementControlSurface,
            contentColor = if (selected) ElementOnAccent else MaterialTheme.colorScheme.onSurface,
            disabledContainerColor = if (selected) ElementBlue.copy(alpha = 0.55f) else ElementControlSurface,
            disabledContentColor = if (selected) ElementOnAccent else ElementInfo,
        ),
        shape = elementShape,
    ) {
        Text(label)
    }
}

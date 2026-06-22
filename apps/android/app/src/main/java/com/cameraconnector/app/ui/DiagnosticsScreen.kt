package com.cameraconnector.app.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.DashboardState

@Composable
internal fun MetricCard(
    value: String,
    label: String,
    accentColor: Color,
    modifier: Modifier = Modifier,
) {
    ElementCard(modifier = modifier) {
        Column(Modifier.padding(12.dp)) {
            Text(
                value,
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
            )
            Spacer(Modifier.height(4.dp))
            Text(
                label,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.labelMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(Modifier.height(10.dp))
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(3.dp)
                    .clip(CircleShape)
                    .background(accentColor),
            )
        }
    }
}

@Composable
internal fun DiagnosticsScreen(
    dashboard: DashboardState,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    BackHandler(onBack = onBack)
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            CompactBackHeader(
                title = "诊断日志",
                subtitle = "\u8fde\u63a5\u3001\u4f20\u8f93\u548c\u5199\u5165\u72b6\u6001",
                onBack = onBack,
            )
        }
        item {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                MetricCard(
                    value = dashboard.accounts.count { it.online || it.activeConnections > 0 }.toString(),
                    label = "在线账号",
                    accentColor = ElementBlue,
                    modifier = Modifier.weight(1f),
                )
                MetricCard(
                    value = dashboard.transfers.count { it.status == "Failed" }.toString(),
                    label = "失败传输",
                    accentColor = ElementDanger,
                    modifier = Modifier.weight(1f),
                )
                MetricCard(
                    value = dashboard.publishQueue.pendingCount.toString(),
                    label = "\u961f\u5217\u5f85\u5904\u7406",
                    accentColor = ElementWarning,
                    modifier = Modifier.weight(1f),
                )
            }
        }
        val recentTransfers = dashboard.transfers.take(6)
        if (recentTransfers.isEmpty()) {
            item { Text("暂无诊断事件", color = MaterialTheme.colorScheme.onSurfaceVariant) }
        } else {
            items(recentTransfers) { transfer ->
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text(transfer.displayPath, style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(4.dp))
                        Text(
                            "${transferStatusLabel(transfer.status)} · ${transfer.id}",
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        transfer.message?.let { message ->
                            Spacer(Modifier.height(4.dp))
                            Text(message, color = ElementDanger)
                        }
                    }
                }
            }
        }
    }
}

package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
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
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.DeviceAccount

@Composable
internal fun AccountsScreen(
    dashboard: DashboardState,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onOpenAccount: (DeviceAccount) -> Unit,
    onAddAccount: () -> Unit,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            CompactPageHeader(
                title = "账号管理",
                subtitle = "\u76f8\u673a\u8d26\u53f7\u548c\u8fde\u63a5\u72b6\u6001",
            )
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        if (dashboard.accounts.isEmpty()) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "\u8fd8\u6ca1\u6709\u8d26\u53f7\u3002\u8bf7\u4e3a\u76f8\u673a\u914d\u7f6e\u767b\u5f55\u7528\u6237\u540d\u548c\u5bc6\u7801\u3002",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            items(dashboard.accounts) { account ->
                AccountMenuRow(account = account, onClick = { onOpenAccount(account) })
            }
        }
        item {
            Button(
                onClick = onAddAccount,
                modifier = Modifier.fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(
                    containerColor = ElementBlue,
                    contentColor = ElementOnAccent,
                ),
            ) {
                Text("新增账号")
            }
        }
    }
}

@Composable
internal fun AccountDetailScreen(
    account: DeviceAccount?,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onSaveDeviceAccount: (DeviceAccount, String?) -> Unit,
    onDeleteDeviceAccount: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val isNew = account == null
    var deviceName by remember(account?.username) { mutableStateOf(account?.deviceName.orEmpty()) }
    var username by remember(account?.username) { mutableStateOf(account?.username.orEmpty()) }
    var password by remember(account?.username) { mutableStateOf("") }
    val locked = account?.let { it.online || it.activeConnections > 0 } ?: false
    val actionsEnabled = actionInFlight == null
    val cleanUsername = username.trim()
    val passwordOk = account?.passwordConfigured == true || password.isNotBlank()
    val canSave = actionsEnabled && !locked && cleanUsername.isNotBlank() && passwordOk

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            HeaderWithBack(
                title = if (isNew) "新增账号" else "账号详情",
                subtitle = if (locked) "\u8fde\u63a5\u4e2d\u7684\u8d26\u53f7\u4e0d\u53ef\u7f16\u8f91\u6216\u5220\u9664" else "\u7528\u6237\u540d\u3001\u5bc6\u7801\u3001\u8bbe\u5907\u540d",
                onBack = onBack,
            )
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        account?.let {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text("\u8fde\u63a5\u72b6\u6001", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(8.dp))
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            ElementTag(
                                text = if (it.online) "\u5728\u7ebf" else "\u672a\u8fde\u63a5",
                                color = if (it.online) ElementSuccess else ElementInfo,
                            )
                            ElementTag("连接 ${it.activeConnections}", ElementBlue)
                        }
                        Spacer(Modifier.height(8.dp))
                        Text("最近来源：${formatEndpoint(it)}")
                        it.lastSeenAtMs?.let { value -> Text("最近在线：${formatEpochMillisForDisplay(value)}") }
                        it.lastDisconnectedAtMs?.let { value ->
                            Text("最近断开 ${formatEpochMillisForDisplay(value)}")
                        }
                    }
                }
            }
        }

        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    OutlinedTextField(
                        value = username,
                        onValueChange = { username = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("\u7528\u6237\u540d") },
                        singleLine = true,
                        enabled = actionsEnabled && !locked && isNew,
                    )
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = deviceName,
                        onValueChange = { deviceName = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("设备名称") },
                        singleLine = true,
                        enabled = actionsEnabled && !locked,
                    )
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = password,
                        onValueChange = { password = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text(if (account?.passwordConfigured == true) "新密码（留空不修改）" else "密码") },
                        singleLine = true,
                        visualTransformation = PasswordVisualTransformation(),
                        enabled = actionsEnabled && !locked,
                    )
                    if (locked) {
                        Spacer(Modifier.height(8.dp))
                        Text("\u8bf7\u5148\u7b49\u5f85\u76f8\u673a\u65ad\u5f00\u6216\u505c\u6b62\u63a5\u6536\u670d\u52a1\uff0c\u518d\u4fee\u6539\u8be5\u8d26\u53f7\u3002")
                    }
                    Spacer(Modifier.height(16.dp))
                    Button(
                        onClick = {
                            val nextUsername = cleanUsername
                            onSaveDeviceAccount(
                                DeviceAccount(
                                    username = nextUsername,
                                    deviceName = deviceName.trim().ifBlank { nextUsername },
                                    passwordConfigured = account?.passwordConfigured == true || password.isNotBlank(),
                                    latestIp = account?.latestIp,
                                    latestPort = account?.latestPort,
                                    activeConnections = account?.activeConnections ?: 0,
                                    lastSeenAtMs = account?.lastSeenAtMs,
                                    lastDisconnectedAtMs = account?.lastDisconnectedAtMs,
                                    online = account?.online ?: false,
                                ),
                                password.takeIf { it.isNotBlank() },
                            )
                            password = ""
                        },
                        enabled = canSave,
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        Text("保存")
                    }
                    if (!isNew) {
                        Spacer(Modifier.height(8.dp))
                        OutlinedButton(
                            onClick = { onDeleteDeviceAccount(cleanUsername) },
                            enabled = actionsEnabled && !locked,
                            modifier = Modifier.fillMaxWidth(),
                            colors = ButtonDefaults.outlinedButtonColors(contentColor = ElementDanger),
                            border = BorderStroke(1.dp, ElementDanger),
                        ) {
                            Text("删除账号")
                        }
                    }
                }
            }
        }
    }
}

@Composable
internal fun AccountMenuRow(account: DeviceAccount, onClick: () -> Unit) {
    SettingsMenuRow(
        title = account.deviceName.ifBlank { account.username },
        subtitle = "${account.username} · ${formatEndpoint(account)}",
        trailing = if (account.online || account.activeConnections > 0) "在线 >" else "未连接 >",
        onClick = onClick,
        trailingColor = if (account.online || account.activeConnections > 0) ElementSuccess else ElementInfo,
    )
}

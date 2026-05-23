package com.cameraconnector.app.ui

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
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch

@Composable
fun CameraConnectorApp(
    coreGateway: CoreGateway,
    storageGateway: AndroidStorageGateway,
    notificationPermissionRequired: Boolean,
    notificationPermissionGranted: Flow<Boolean>,
    selectedInboxLabel: Flow<String?>,
    onRequestNotificationPermission: () -> Unit,
    onChooseInboxDirectory: () -> Unit,
) {
    val dashboard by coreGateway.observeDashboard().collectAsState(
        initial = DashboardState(
            receiver = com.cameraconnector.app.core.ReceiverState(
                running = false,
                phase = "Unknown",
                protocol = "FTP",
                authMode = "Unknown",
                accountCount = 0,
                host = "0.0.0.0",
                port = 2121,
                outputLabel = "未配置",
                message = null,
            ),
            accounts = emptyList(),
            inbox = emptyList(),
            transfers = emptyList(),
        ),
    )
    val notificationsGranted by notificationPermissionGranted.collectAsState(initial = true)
    val selectedInbox by selectedInboxLabel.collectAsState(initial = null)
    val scope = rememberCoroutineScope()
    var tab by remember { mutableStateOf(MainTab.Overview) }
    var actionError by remember { mutableStateOf<String?>(null) }
    var actionInFlight by remember { mutableStateOf<String?>(null) }
    fun runAction(actionName: String, action: suspend () -> Unit) {
        scope.launch {
            actionInFlight = actionName
            try {
                action()
                actionError = null
            } catch (error: Throwable) {
                actionError = error.message ?: error::class.java.simpleName
            } finally {
                actionInFlight = null
            }
        }
    }

    MaterialTheme {
        Surface(modifier = Modifier.fillMaxSize()) {
            Scaffold(
                bottomBar = {
                    NavigationBar {
                        MainTab.entries.forEach { item ->
                            NavigationBarItem(
                                selected = tab == item,
                                onClick = { tab = item },
                                label = { Text(item.label) },
                                icon = { Text(item.icon) },
                            )
                        }
                    }
                },
            ) { padding ->
                when (tab) {
                    MainTab.Overview -> OverviewScreen(
                        dashboard = dashboard,
                        notificationPermissionRequired = notificationPermissionRequired,
                        notificationPermissionGranted = notificationsGranted,
                        onRequestNotificationPermission = onRequestNotificationPermission,
                        actionError = actionError,
                        actionInFlight = actionInFlight,
                        onClearActionError = { actionError = null },
                        selectedInboxLabel = selectedInbox,
                        onChooseInboxDirectory = onChooseInboxDirectory,
                        onStart = { runAction("正在启动接收服务") { coreGateway.startReceiver() } },
                        onStop = { runAction("正在停止接收服务") { coreGateway.stopReceiver() } },
                        onSaveReceiverSettings = { settings ->
                            runAction("正在保存接收设置") { coreGateway.saveReceiverSettings(settings) }
                        },
                        onSaveDeviceAccount = { account, password ->
                            runAction("正在保存账号") { coreGateway.saveDeviceAccount(account, password) }
                        },
                        modifier = Modifier.padding(padding),
                    )

                    MainTab.Inbox -> InboxScreen(
                        dashboard = dashboard,
                        modifier = Modifier.padding(padding),
                    )

                    MainTab.Transfers -> TransfersScreen(
                        dashboard = dashboard,
                        modifier = Modifier.padding(padding),
                    )
                }
            }
        }
    }
}

private enum class MainTab(val label: String, val icon: String) {
    Overview("总览", "总"),
    Inbox("收件箱", "箱"),
    Transfers("传输", "传"),
}

@Composable
private fun OverviewScreen(
    dashboard: DashboardState,
    notificationPermissionRequired: Boolean,
    notificationPermissionGranted: Boolean,
    onRequestNotificationPermission: () -> Unit,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    selectedInboxLabel: String?,
    onChooseInboxDirectory: () -> Unit,
    onStart: () -> Unit,
    onStop: () -> Unit,
    onSaveReceiverSettings: (ReceiverSettings) -> Unit,
    onSaveDeviceAccount: (DeviceAccount, String?) -> Unit,
    modifier: Modifier = Modifier,
) {
    var protocol by remember(dashboard.receiver.protocol) {
        mutableStateOf(dashboard.receiver.protocol.ifBlank { "FTP" })
    }
    var hostInput by remember(dashboard.receiver.host) {
        mutableStateOf(dashboard.receiver.host)
    }
    var ftpPortInput by remember(dashboard.receiver.protocol, dashboard.receiver.port) {
        mutableStateOf(if (dashboard.receiver.protocol == "FTP") dashboard.receiver.port.toString() else "2121")
    }
    var sftpPortInput by remember(dashboard.receiver.protocol, dashboard.receiver.port) {
        mutableStateOf(if (dashboard.receiver.protocol == "SFTP") dashboard.receiver.port.toString() else "2222")
    }
    var deviceName by remember { mutableStateOf("") }
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    val ftpPort = ftpPortInput.toIntOrNull()
    val sftpPort = sftpPortInput.toIntOrNull()
    val receiverSettingsValid = hostInput.trim().isNotBlank() &&
        ftpPort in 1..65_535 &&
        sftpPort in 1..65_535
    val actionsEnabled = actionInFlight == null

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Text("相机连接器", style = MaterialTheme.typography.headlineMedium)
        }

        actionError?.let { message ->
            item {
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text("操作失败", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(8.dp))
                        Text(message)
                        Spacer(Modifier.height(12.dp))
                        Button(onClick = onClearActionError) {
                            Text("关闭")
                        }
                    }
                }
            }
        }

        actionInFlight?.let { action ->
            item {
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text("处理中", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(8.dp))
                        Text(action)
                    }
                }
            }
        }

        item {
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("接收服务", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    Text("${dashboard.receiver.protocol} ${dashboard.receiver.host}:${dashboard.receiver.port}")
                    Text(
                        "${receiverPhaseLabel(dashboard.receiver.phase)} / " +
                            "${authModeLabel(dashboard.receiver.authMode)} / " +
                            "账号=${dashboard.receiver.accountCount}",
                    )
                    dashboard.receiver.message?.let { Text(it) }
                    Spacer(Modifier.height(12.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Button(
                            onClick = onStart,
                            enabled = actionsEnabled && !dashboard.receiver.running && notificationPermissionGranted,
                        ) {
                            Text("启动")
                        }
                        Button(onClick = onStop, enabled = actionsEnabled && dashboard.receiver.running) {
                            Text("停止")
                        }
                    }
                    Spacer(Modifier.height(16.dp))
                    Text("接收设置", style = MaterialTheme.typography.titleSmall)
                    Spacer(Modifier.height(8.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Button(
                            onClick = { protocol = "FTP" },
                            enabled = actionsEnabled && protocol != "FTP" && !dashboard.receiver.running,
                        ) {
                            Text("FTP")
                        }
                        Button(
                            onClick = { protocol = "SFTP" },
                            enabled = actionsEnabled && protocol != "SFTP" && !dashboard.receiver.running,
                        ) {
                            Text("SFTP")
                        }
                    }
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = hostInput,
                        onValueChange = { hostInput = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("监听地址") },
                        singleLine = true,
                        enabled = actionsEnabled && !dashboard.receiver.running,
                    )
                    Spacer(Modifier.height(8.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(
                            value = ftpPortInput,
                            onValueChange = { ftpPortInput = it },
                            modifier = Modifier.weight(1f),
                            label = { Text("FTP 端口") },
                            singleLine = true,
                            enabled = actionsEnabled && !dashboard.receiver.running,
                        )
                        OutlinedTextField(
                            value = sftpPortInput,
                            onValueChange = { sftpPortInput = it },
                            modifier = Modifier.weight(1f),
                            label = { Text("SFTP 端口") },
                            singleLine = true,
                            enabled = actionsEnabled && !dashboard.receiver.running,
                        )
                    }
                    if (dashboard.receiver.running) {
                        Spacer(Modifier.height(8.dp))
                        Text("修改设置前请先停止接收服务。")
                    }
                    Spacer(Modifier.height(12.dp))
                    Button(
                        onClick = {
                            onSaveReceiverSettings(
                                ReceiverSettings(
                                    protocol = protocol,
                                    host = hostInput.trim(),
                                    ftpPort = ftpPort ?: 2121,
                                    sftpPort = sftpPort ?: 2222,
                                    outputLabel = dashboard.receiver.outputLabel,
                                ),
                            )
                        },
                        enabled = actionsEnabled && !dashboard.receiver.running && receiverSettingsValid,
                    ) {
                        Text("保存接收设置")
                    }
                }
            }
        }

        if (notificationPermissionRequired && !notificationPermissionGranted) {
            item {
                Card(modifier = Modifier.fillMaxWidth()) {
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
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("设备账号", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    if (dashboard.accounts.isEmpty()) {
                        Text("还没有相机账号。")
                    } else {
                        dashboard.accounts.forEach { account ->
                            Text("${account.deviceName} / ${account.username}")
                            val endpoint = listOfNotNull(
                                account.latestIp,
                                account.latestPort?.toString(),
                            ).joinToString(":")
                            Text(
                                "${if (account.online) "在线" else "离线"} / " +
                                    "连接数=${account.activeConnections}" +
                                    endpoint.ifBlank { "" }.let { if (it.isBlank()) "" else " / $it" },
                            )
                            account.lastSeenAtMs?.let { Text("最近在线：$it") }
                            account.lastDisconnectedAtMs?.let { Text("最近断开：$it") }
                        }
                    }
                    Spacer(Modifier.height(12.dp))
                    OutlinedTextField(
                        value = deviceName,
                        onValueChange = { deviceName = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("设备名称") },
                        singleLine = true,
                    )
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = username,
                        onValueChange = { username = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("FTP/SFTP 用户名") },
                        singleLine = true,
                    )
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = password,
                        onValueChange = { password = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("密码") },
                        singleLine = true,
                        visualTransformation = PasswordVisualTransformation(),
                    )
                    Spacer(Modifier.height(12.dp))
                    Button(
                        onClick = {
                            val cleanUsername = username.trim()
                            val cleanPassword = password
                            onSaveDeviceAccount(
                                DeviceAccount(
                                    username = cleanUsername,
                                    deviceName = deviceName.trim().ifBlank { cleanUsername },
                                    passwordConfigured = cleanPassword.isNotBlank(),
                                    latestIp = null,
                                    latestPort = null,
                                    activeConnections = 0,
                                    lastSeenAtMs = null,
                                    lastDisconnectedAtMs = null,
                                    online = false,
                                ),
                                cleanPassword.takeIf { it.isNotBlank() },
                            )
                            password = ""
                        },
                        enabled = actionsEnabled && username.trim().isNotBlank() && password.isNotBlank(),
                    ) {
                        Text("保存账号")
                    }
                }
            }
        }

        item {
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("导入位置", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    Text(selectedInboxLabel ?: "应用私有收件箱")
                    Spacer(Modifier.height(4.dp))
                    Text("当前原生收件箱：${dashboard.receiver.outputLabel}")
                    Spacer(Modifier.height(12.dp))
                    Button(onClick = onChooseInboxDirectory) {
                        Text("选择导入文件夹")
                    }
                }
            }
        }
    }
}

@Composable
private fun InboxScreen(
    dashboard: DashboardState,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Text("收件箱", style = MaterialTheme.typography.headlineMedium)
        }
        if (dashboard.inbox.isEmpty()) {
            item { Text("还没有导入文件。") }
        } else {
            items(dashboard.inbox) { asset ->
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text(asset.displayPath, style = MaterialTheme.typography.titleMedium)
                        Text("${asset.format} / 接收时间：${asset.receivedAt}")
                    }
                }
            }
        }
    }
}

@Composable
private fun TransfersScreen(
    dashboard: DashboardState,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Text("传输记录", style = MaterialTheme.typography.headlineMedium)
        }
        if (dashboard.transfers.isEmpty()) {
            item { Text("还没有传输记录。") }
        } else {
            items(dashboard.transfers) { transfer ->
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text(transfer.displayPath, style = MaterialTheme.typography.titleMedium)
                        Text("${transferStatusLabel(transfer.status)} / ${transfer.id}")
                        transfer.message?.let { Text(it) }
                    }
                }
            }
        }
    }
}

private fun receiverPhaseLabel(value: String): String = when (value) {
    "Running" -> "运行中"
    "Stopped" -> "已停止"
    "Unknown" -> "未知"
    else -> value
}

private fun authModeLabel(value: String): String = when (value) {
    "Accounts" -> "账号认证"
    "Open" -> "开放"
    "Unknown" -> "未知"
    else -> value
}

private fun transferStatusLabel(value: String): String = when (value) {
    "Completed" -> "已完成"
    "Failed" -> "失败"
    "Pending" -> "等待中"
    else -> value
}

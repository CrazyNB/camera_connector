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
    onRequestNotificationPermission: () -> Unit,
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
                outputLabel = "Not configured",
                message = null,
            ),
            accounts = emptyList(),
            inbox = emptyList(),
            transfers = emptyList(),
        ),
    )
    val notificationsGranted by notificationPermissionGranted.collectAsState(initial = true)
    val scope = rememberCoroutineScope()
    var tab by remember { mutableStateOf(MainTab.Overview) }

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
                        onStart = { scope.launch { coreGateway.startReceiver() } },
                        onStop = { scope.launch { coreGateway.stopReceiver() } },
                        onSaveReceiverSettings = { settings ->
                            scope.launch { coreGateway.saveReceiverSettings(settings) }
                        },
                        onSaveDeviceAccount = { account, password ->
                            scope.launch { coreGateway.saveDeviceAccount(account, password) }
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
    Overview("Overview", "O"),
    Inbox("Inbox", "I"),
    Transfers("Transfers", "T"),
}

@Composable
private fun OverviewScreen(
    dashboard: DashboardState,
    notificationPermissionRequired: Boolean,
    notificationPermissionGranted: Boolean,
    onRequestNotificationPermission: () -> Unit,
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

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Text("Camera Connector", style = MaterialTheme.typography.headlineMedium)
        }

        item {
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("Receiver", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    Text("${dashboard.receiver.protocol} ${dashboard.receiver.host}:${dashboard.receiver.port}")
                    Text("${dashboard.receiver.phase} / ${dashboard.receiver.authMode} / accounts=${dashboard.receiver.accountCount}")
                    dashboard.receiver.message?.let { Text(it) }
                    Spacer(Modifier.height(12.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Button(
                            onClick = onStart,
                            enabled = !dashboard.receiver.running && notificationPermissionGranted,
                        ) {
                            Text("Start")
                        }
                        Button(onClick = onStop, enabled = dashboard.receiver.running) {
                            Text("Stop")
                        }
                    }
                    Spacer(Modifier.height(16.dp))
                    Text("Settings", style = MaterialTheme.typography.titleSmall)
                    Spacer(Modifier.height(8.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Button(
                            onClick = { protocol = "FTP" },
                            enabled = protocol != "FTP" && !dashboard.receiver.running,
                        ) {
                            Text("FTP")
                        }
                        Button(
                            onClick = { protocol = "SFTP" },
                            enabled = protocol != "SFTP" && !dashboard.receiver.running,
                        ) {
                            Text("SFTP")
                        }
                    }
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = hostInput,
                        onValueChange = { hostInput = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("Bind host") },
                        singleLine = true,
                        enabled = !dashboard.receiver.running,
                    )
                    Spacer(Modifier.height(8.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(
                            value = ftpPortInput,
                            onValueChange = { ftpPortInput = it },
                            modifier = Modifier.weight(1f),
                            label = { Text("FTP port") },
                            singleLine = true,
                            enabled = !dashboard.receiver.running,
                        )
                        OutlinedTextField(
                            value = sftpPortInput,
                            onValueChange = { sftpPortInput = it },
                            modifier = Modifier.weight(1f),
                            label = { Text("SFTP port") },
                            singleLine = true,
                            enabled = !dashboard.receiver.running,
                        )
                    }
                    if (dashboard.receiver.running) {
                        Spacer(Modifier.height(8.dp))
                        Text("Stop receiver before changing settings.")
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
                        enabled = !dashboard.receiver.running && receiverSettingsValid,
                    ) {
                        Text("Save receiver settings")
                    }
                }
            }
        }

        if (notificationPermissionRequired && !notificationPermissionGranted) {
            item {
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text("Notifications", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(8.dp))
                        Text("Allow notifications before starting the receiver.")
                        Spacer(Modifier.height(12.dp))
                        Button(onClick = onRequestNotificationPermission) {
                            Text("Allow notifications")
                        }
                    }
                }
            }
        }

        item {
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("Accounts", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    if (dashboard.accounts.isEmpty()) {
                        Text("No camera accounts yet.")
                    } else {
                        dashboard.accounts.forEach { account ->
                            Text("${account.deviceName} / ${account.username}")
                            val endpoint = listOfNotNull(
                                account.latestIp,
                                account.latestPort?.toString(),
                            ).joinToString(":")
                            Text(
                                "${if (account.online) "Online" else "Offline"} / " +
                                    "connections=${account.activeConnections}" +
                                    endpoint.ifBlank { "" }.let { if (it.isBlank()) "" else " / $it" },
                            )
                            account.lastSeenAtMs?.let { Text("Last seen: $it") }
                            account.lastDisconnectedAtMs?.let { Text("Last disconnected: $it") }
                        }
                    }
                    Spacer(Modifier.height(12.dp))
                    OutlinedTextField(
                        value = deviceName,
                        onValueChange = { deviceName = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("Device name") },
                        singleLine = true,
                    )
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = username,
                        onValueChange = { username = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("FTP/SFTP username") },
                        singleLine = true,
                    )
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = password,
                        onValueChange = { password = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("Password") },
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
                        enabled = username.trim().isNotBlank() && password.isNotBlank(),
                    ) {
                        Text("Save account")
                    }
                }
            }
        }

        item {
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("Output", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    Text(dashboard.receiver.outputLabel)
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
            Text("Inbox", style = MaterialTheme.typography.headlineMedium)
        }
        if (dashboard.inbox.isEmpty()) {
            item { Text("No imported assets yet.") }
        } else {
            items(dashboard.inbox) { asset ->
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text(asset.displayPath, style = MaterialTheme.typography.titleMedium)
                        Text("${asset.format} / ${asset.receivedAt}")
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
            Text("Transfers", style = MaterialTheme.typography.headlineMedium)
        }
        if (dashboard.transfers.isEmpty()) {
            item { Text("No transfer records yet.") }
        } else {
            items(dashboard.transfers) { transfer ->
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text(transfer.displayPath, style = MaterialTheme.typography.titleMedium)
                        Text("${transfer.status} / ${transfer.id}")
                        transfer.message?.let { Text(it) }
                    }
                }
            }
        }
    }
}

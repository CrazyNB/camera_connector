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
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.launch

@Composable
fun CameraConnectorApp(
    coreGateway: CoreGateway,
    storageGateway: AndroidStorageGateway,
) {
    val dashboard by coreGateway.observeDashboard().collectAsState(
        initial = DashboardState(
            receiver = com.cameraconnector.app.core.ReceiverState(
                running = false,
                protocol = "FTP",
                host = "0.0.0.0",
                port = 2121,
                outputLabel = "Not configured",
            ),
            accounts = emptyList(),
            inbox = emptyList(),
            transfers = emptyList(),
        ),
    )
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
                        onStart = { scope.launch { coreGateway.startReceiver() } },
                        onStop = { scope.launch { coreGateway.stopReceiver() } },
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
    onStart: () -> Unit,
    onStop: () -> Unit,
    modifier: Modifier = Modifier,
) {
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
                    Text(if (dashboard.receiver.running) "Running" else "Stopped")
                    Spacer(Modifier.height(12.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Button(onClick = onStart, enabled = !dashboard.receiver.running) {
                            Text("Start")
                        }
                        Button(onClick = onStop, enabled = dashboard.receiver.running) {
                            Text("Stop")
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
                    dashboard.accounts.forEach { account ->
                        Text("${account.deviceName} / ${account.username}")
                        Text(if (account.online) "Online" else "Offline")
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

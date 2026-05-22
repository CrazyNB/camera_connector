package com.cameraconnector.app.core

import com.cameraconnector.app.service.ReceiverServiceController
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject

class NativeCoreGateway(
    private val nativeCore: NativeMobileCore,
    private val stateDir: String?,
    private val receiverServiceController: ReceiverServiceController,
) : CoreGateway, AutoCloseable {
    private val gatewayScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val dashboard = MutableStateFlow(emptyDashboard())

    init {
        dashboard.value = loadDashboard()
        pollDashboard()
    }

    override fun observeDashboard(): Flow<DashboardState> = dashboard.asStateFlow()

    suspend fun refresh() {
        dashboard.value = withContext(Dispatchers.IO) {
            loadDashboard()
        }
    }

    override suspend fun startReceiver() {
        receiverServiceController.startReceiver()
        refreshAfterServiceCommand()
    }

    override suspend fun stopReceiver() {
        receiverServiceController.stopReceiver()
        refreshAfterServiceCommand()
    }

    override suspend fun saveReceiverSettings(settings: ReceiverSettings) {
        withContext(Dispatchers.IO) {
            nativeCore.saveReceiverSettings(settings)
        }
        refresh()
    }

    override suspend fun saveDeviceAccount(account: DeviceAccount, password: String?) {
        withContext(Dispatchers.IO) {
            nativeCore.saveDeviceAccount(account, password = password?.takeIf { it.isNotBlank() })
        }
        refresh()
    }

    override fun close() {
        gatewayScope.cancel()
        nativeCore.close()
    }

    private fun loadDashboard(): DashboardState =
        mapDashboard(nativeCore.dashboardJson(stateDir, offset = 0, limit = 50))

    private suspend fun refreshAfterServiceCommand() {
        delay(250)
        refresh()
    }

    private fun pollDashboard() {
        gatewayScope.launch {
            while (isActive) {
                runCatching {
                    dashboard.value = loadDashboard()
                }
                delay(DASHBOARD_POLL_INTERVAL_MS)
            }
        }
    }

    private fun mapDashboard(value: JSONObject): DashboardState {
        val receiverStatus = value.optJSONObject("receiver_status")
        val paths = value.optJSONObject("paths")
        val assets = value.optJSONObject("assets")
        val transfers = value.optJSONObject("transfers")
        val protocol = receiverStatus?.optString("protocol")?.uppercase().orEmpty()
            .ifBlank { "FTP" }
        val (host, port) = splitHostAndPort(
            receiverStatus?.optString("local_addr").orEmpty(),
            defaultPort = if (protocol == "SFTP") 2222 else 2121,
        )

        return DashboardState(
            receiver = ReceiverState(
                running = receiverStatus?.optString("phase") == "Running",
                phase = receiverStatus?.optString("phase").orEmpty()
                    .ifBlank { "Unknown" },
                protocol = protocol,
                authMode = receiverStatus?.optString("auth_mode").orEmpty()
                    .ifBlank { "Unknown" },
                accountCount = receiverStatus?.optInt("account_count") ?: 0,
                host = host,
                port = port,
                outputLabel = paths?.optString("output_dir").orEmpty()
                    .ifBlank { "Choose inbox folder" },
                message = receiverStatus?.takeIf { !it.isNull("message") }
                    ?.optString("message")
                    .orEmpty()
                    .takeIf { it.isNotBlank() },
            ),
            accounts = mapAccounts(value),
            inbox = mapInbox(assets),
            transfers = mapTransfers(transfers, value.optJSONArray("recent_failures")),
        )
    }

    private fun splitHostAndPort(localAddr: String, defaultPort: Int): Pair<String, Int> {
        val trimmed = localAddr.trim()
        if (trimmed.isBlank()) {
            return "0.0.0.0" to defaultPort
        }

        if (trimmed.startsWith("[")) {
            val hostEnd = trimmed.indexOf(']')
            val host = trimmed.substring(0, hostEnd + 1).takeIf { hostEnd > 0 } ?: trimmed
            val port = trimmed.substringAfter("]:", "")
                .toIntOrNull()
                ?: defaultPort
            return host to port
        }

        val splitAt = trimmed.lastIndexOf(':')
        if (splitAt <= 0 || splitAt == trimmed.lastIndex || trimmed.indexOf(':') != splitAt) {
            return trimmed to defaultPort
        }

        return trimmed.substring(0, splitAt) to
            (trimmed.substring(splitAt + 1).toIntOrNull() ?: defaultPort)
    }

    private fun mapAccounts(value: JSONObject): List<DeviceAccount> {
        val accounts = value.optJSONArray("accounts") ?: return emptyList()
        return buildList {
            for (index in 0 until accounts.length()) {
                val item = accounts.optJSONObject(index) ?: continue
                add(
                    DeviceAccount(
                        username = item.optString("username"),
                        deviceName = item.optString("device_name"),
                        passwordConfigured = item.optBoolean("password_configured"),
                        latestIp = item.optString("last_remote_addr").takeIf { it.isNotBlank() },
                        latestPort = item.optInt("last_remote_port")
                            .takeIf { !item.isNull("last_remote_port") },
                        activeConnections = item.optInt("active_connections"),
                        lastSeenAtMs = item.optLong("last_seen_at_ms")
                            .takeIf { !item.isNull("last_seen_at_ms") },
                        lastDisconnectedAtMs = item.optLong("last_disconnected_at_ms")
                            .takeIf { !item.isNull("last_disconnected_at_ms") },
                        online = item.optBoolean("online"),
                    ),
                )
            }
        }
    }

    private fun mapInbox(assets: JSONObject?): List<InboxAsset> {
        val groups = assets?.optJSONArray("groups") ?: return emptyList()
        return buildList {
            for (index in 0 until groups.length()) {
                val group = groups.optJSONObject(index) ?: continue
                val primary = group.optJSONObject("primary") ?: continue
                add(
                    InboxAsset(
                        displayPath = primary.optString("virtual_display_path")
                            .ifBlank { primary.optString("filename") },
                        format = primary.optString("format"),
                        receivedAt = primary.optLong("received_time_ms").toString(),
                    ),
                )
            }
        }
    }

    private fun mapTransfers(transfers: JSONObject?, recentFailures: JSONArray?): List<TransferRow> {
        if (transfers == null && recentFailures == null) {
            return emptyList()
        }

        return buildList {
            transfers?.let {
                add(
                    TransferRow(
                        id = "summary",
                        status = "completed=${it.optInt("completed_count")}",
                        displayPath = "failed=${it.optInt("failed_count")}",
                        message = "total=${it.optInt("total_count")}",
                    ),
                )
            }

            if (recentFailures != null) {
                for (index in 0 until recentFailures.length()) {
                    val item = recentFailures.optJSONObject(index) ?: continue
                    val record = item.optJSONObject("record")
                    add(
                        TransferRow(
                            id = record?.optString("transfer_id").orEmpty()
                                .ifBlank { "failure-$index" },
                            status = record?.optString("status").orEmpty()
                                .ifBlank { "Failed" },
                            displayPath = item.optString("virtual_display_path")
                                .ifBlank { record?.optString("original_path").orEmpty() }
                                .ifBlank { record?.optString("final_filename").orEmpty() }
                                .ifBlank { "Failed transfer" },
                            message = record?.optString("error").orEmpty()
                                .takeIf { it.isNotBlank() },
                        ),
                    )
                }
            }
        }
    }

    private fun emptyDashboard(): DashboardState =
        DashboardState(
            receiver = ReceiverState(
                running = false,
                phase = "Unknown",
                protocol = "FTP",
                authMode = "Unknown",
                accountCount = 0,
                host = "0.0.0.0",
                port = 2121,
                outputLabel = "Choose inbox folder",
                message = null,
            ),
            accounts = emptyList(),
            inbox = emptyList(),
            transfers = emptyList(),
        )

    private companion object {
        const val DASHBOARD_POLL_INTERVAL_MS = 2_000L
    }
}

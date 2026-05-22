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

    override suspend fun saveDeviceAccount(account: DeviceAccount) {
        withContext(Dispatchers.IO) {
            nativeCore.saveDeviceAccount(account, password = null)
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

        return DashboardState(
            receiver = ReceiverState(
                running = receiverStatus?.optString("phase") == "Running",
                protocol = receiverStatus?.optString("protocol")?.uppercase().orEmpty()
                    .ifBlank { "FTP" },
                host = receiverStatus?.optString("local_addr").orEmpty()
                    .ifBlank { "0.0.0.0" },
                port = receiverStatus?.optInt("port", 2121) ?: 2121,
                outputLabel = paths?.optString("output_dir").orEmpty()
                    .ifBlank { "Choose inbox folder" },
            ),
            accounts = mapAccounts(value),
            inbox = mapInbox(assets),
            transfers = mapTransfers(transfers),
        )
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

    private fun mapTransfers(transfers: JSONObject?): List<TransferRow> {
        if (transfers == null) {
            return emptyList()
        }
        return listOf(
            TransferRow(
                id = "summary",
                status = "completed=${transfers.optInt("completed_count")}",
                displayPath = "failed=${transfers.optInt("failed_count")}",
                message = "total=${transfers.optInt("total_count")}",
            ),
        )
    }

    private fun emptyDashboard(): DashboardState =
        DashboardState(
            receiver = ReceiverState(
                running = false,
                protocol = "FTP",
                host = "0.0.0.0",
                port = 2121,
                outputLabel = "Choose inbox folder",
            ),
            accounts = emptyList(),
            inbox = emptyList(),
            transfers = emptyList(),
        )

    private companion object {
        const val DASHBOARD_POLL_INTERVAL_MS = 2_000L
    }
}

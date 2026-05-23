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

    override suspend fun removeDeviceAccount(username: String) {
        withContext(Dispatchers.IO) {
            nativeCore.removeDeviceAccount(username)
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
        val receiverSettings = value.optJSONObject("receiver_settings")
        val paths = value.optJSONObject("paths")
        val assets = value.optJSONObject("assets")
        val transfers = value.optJSONObject("transfers")
        val running = receiverStatus?.optString("phase") == "Running"
        val settingsProtocol = normalizeProtocol(receiverSettings?.optString("protocol"))
        val statusProtocol = receiverStatus?.optString("protocol")
            ?.takeIf { it.isNotBlank() && !it.equals("null", ignoreCase = true) }
            ?.let(::normalizeProtocol)
        val protocol = if (running) statusProtocol ?: settingsProtocol else settingsProtocol
        val configuredHost = receiverSettings?.optString("bind_host")
            ?.takeIf { it.isNotBlank() && !it.equals("null", ignoreCase = true) }
            ?: DEFAULT_LISTEN_HOST
        val configuredPort = when (protocol) {
            "SFTP" -> receiverSettings?.optInt("sftp_port")?.takeIf { it in 1..65_535 } ?: 2222
            else -> receiverSettings?.optInt("ftp_port")?.takeIf { it in 1..65_535 } ?: 2121
        }
        val localAddr = receiverStatus?.optString("local_addr").orEmpty()
        val (statusHost, statusPort) = splitHostAndPort(localAddr, defaultPort = configuredPort)
        val (host, port) = if (localAddr.isBlank() || localAddr.equals("null", ignoreCase = true)) {
            configuredHost to configuredPort
        } else {
            statusHost to statusPort
        }

        return DashboardState(
            receiver = ReceiverState(
                running = running,
                phase = receiverStatus?.optString("phase").orEmpty()
                    .ifBlank { "Unknown" },
                protocol = protocol,
                authMode = receiverStatus?.optString("auth_mode").orEmpty()
                    .ifBlank { "Unknown" },
                accountCount = receiverStatus?.optInt("account_count") ?: 0,
                host = host,
                port = port,
                outputLabel = paths?.optString("output_dir").orEmpty()
                    .ifBlank { "选择收件箱文件夹" },
                message = receiverStatus?.takeIf { !it.isNull("message") }
                    ?.optString("message")
                    .orEmpty()
                    .takeIf { it.isNotBlank() },
            ),
            accounts = mapAccounts(value),
            inbox = mapInbox(assets),
            transfers = mapTransfers(transfers, assets, value.optJSONArray("recent_failures")),
        )
    }

    private fun splitHostAndPort(localAddr: String, defaultPort: Int): Pair<String, Int> {
        val trimmed = localAddr.trim()
        if (trimmed.isBlank() || trimmed.equals("null", ignoreCase = true)) {
            return DEFAULT_LISTEN_HOST to defaultPort
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

    private fun normalizeProtocol(value: String?): String {
        val protocol = value.orEmpty().trim().uppercase()
        return if (protocol.isBlank() || protocol == "NULL") {
            "FTP"
        } else {
            protocol
        }
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
                val raw = group.optJSONObject("raw")
                val jpeg = group.optJSONObject("jpeg")
                val video = group.optJSONObject("video")
                add(
                    InboxAsset(
                        id = primary.optString("id"),
                        groupKey = group.optString("group_key")
                            .ifBlank { primary.optString("id") },
                        displayPath = primary.optString("virtual_display_path")
                            .ifBlank { primary.optString("filename") },
                        format = primary.optString("format"),
                        receivedAt = primary.optLong("received_time_ms").toString(),
                        username = primary.optString("username").takeIf { it.isNotBlank() },
                        displaySource = primary.optString("display_source").takeIf { it.isNotBlank() },
                        originalPath = primary.optString("original_path").takeIf { it.isNotBlank() },
                        sizeBytes = primary.optLong("size_bytes").takeIf { !primary.isNull("size_bytes") },
                        previewLocation = jpeg?.assetStorageLocation()
                            ?: primary.assetStorageLocation(),
                        rawPath = raw?.assetDisplayPath(),
                        jpegPath = jpeg?.assetDisplayPath(),
                        videoPath = video?.assetDisplayPath(),
                    ),
                )
            }
        }
    }

    private fun JSONObject.assetDisplayPath(): String =
        optString("virtual_display_path").ifBlank { optString("filename") }

    private fun JSONObject.assetStorageLocation(): String? {
        val location = optJSONObject("storage_location") ?: return null
        return location.optString("path")
            .ifBlank { location.optString("uri") }
            .ifBlank { null }
    }

    private fun mapTransfers(
        transfers: JSONObject?,
        assets: JSONObject?,
        recentFailures: JSONArray?,
    ): List<TransferRow> {
        if (transfers == null && assets == null && recentFailures == null) {
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

            val groups = assets?.optJSONArray("groups")
            if (groups != null) {
                for (index in 0 until groups.length()) {
                    val group = groups.optJSONObject(index) ?: continue
                    val seenIds = mutableSetOf<String>()
                    listOf("primary", "jpeg", "raw", "video")
                        .mapNotNull { group.optJSONObject(it) }
                        .forEach { asset ->
                            val id = asset.optString("id")
                            if (id.isBlank() || !seenIds.add(id)) {
                                return@forEach
                            }
                            val displayPath = asset.assetDisplayPath()
                            if (displayPath.isBlank()) {
                                return@forEach
                            }
                            add(
                                TransferRow(
                                    id = id,
                                    status = "Completed",
                                    displayPath = displayPath,
                                    message = asset.optString("size_bytes").takeIf { it.isNotBlank() }
                                        ?.let { "$it bytes" },
                                ),
                            )
                        }
                }
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
                host = DEFAULT_LISTEN_HOST,
                port = 2121,
                outputLabel = "选择收件箱文件夹",
                message = null,
            ),
            accounts = emptyList(),
            inbox = emptyList(),
            transfers = emptyList(),
        )

    private companion object {
        const val DEFAULT_LISTEN_HOST = "192.168.137.1"
        const val DASHBOARD_POLL_INTERVAL_MS = 2_000L
    }
}

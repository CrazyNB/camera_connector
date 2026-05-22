package com.cameraconnector.app.core

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

interface CoreGateway {
    fun observeDashboard(): Flow<DashboardState>
    suspend fun startReceiver()
    suspend fun stopReceiver()
    suspend fun saveReceiverSettings(settings: ReceiverSettings)
    suspend fun saveDeviceAccount(account: DeviceAccount, password: String?)
}

data class DashboardState(
    val receiver: ReceiverState,
    val accounts: List<DeviceAccount>,
    val inbox: List<InboxAsset>,
    val transfers: List<TransferRow>,
)

data class ReceiverState(
    val running: Boolean,
    val protocol: String,
    val host: String,
    val port: Int,
    val outputLabel: String,
)

data class ReceiverSettings(
    val protocol: String,
    val host: String,
    val ftpPort: Int,
    val sftpPort: Int,
    val outputLabel: String,
)

data class DeviceAccount(
    val username: String,
    val deviceName: String,
    val passwordConfigured: Boolean,
    val latestIp: String?,
    val online: Boolean,
)

data class InboxAsset(
    val displayPath: String,
    val format: String,
    val receivedAt: String,
)

data class TransferRow(
    val id: String,
    val status: String,
    val displayPath: String,
    val message: String?,
)

class PreviewCoreGateway : CoreGateway {
    private val dashboard = MutableStateFlow(
        DashboardState(
            receiver = ReceiverState(
                running = false,
                protocol = "FTP",
                host = "192.168.137.1",
                port = 2121,
                outputLabel = "Choose inbox folder",
            ),
            accounts = listOf(
                DeviceAccount(
                    username = "camera01",
                    deviceName = "Camera 01",
                    passwordConfigured = true,
                    latestIp = null,
                    online = false,
                ),
            ),
            inbox = emptyList(),
            transfers = emptyList(),
        ),
    )

    override fun observeDashboard(): Flow<DashboardState> = dashboard.asStateFlow()

    override suspend fun startReceiver() {
        dashboard.value = dashboard.value.copy(
            receiver = dashboard.value.receiver.copy(running = true),
        )
    }

    override suspend fun stopReceiver() {
        dashboard.value = dashboard.value.copy(
            receiver = dashboard.value.receiver.copy(running = false),
        )
    }

    override suspend fun saveReceiverSettings(settings: ReceiverSettings) {
        dashboard.value = dashboard.value.copy(
            receiver = ReceiverState(
                running = dashboard.value.receiver.running,
                protocol = settings.protocol,
                host = settings.host,
                port = if (settings.protocol == "SFTP") settings.sftpPort else settings.ftpPort,
                outputLabel = settings.outputLabel,
            ),
        )
    }

    override suspend fun saveDeviceAccount(account: DeviceAccount, password: String?) {
        val accountWithPasswordState = account.copy(
            passwordConfigured = account.passwordConfigured || !password.isNullOrBlank(),
        )
        val nextAccounts = dashboard.value.accounts
            .filterNot { it.username == accountWithPasswordState.username } + accountWithPasswordState
        dashboard.value = dashboard.value.copy(accounts = nextAccounts)
    }
}

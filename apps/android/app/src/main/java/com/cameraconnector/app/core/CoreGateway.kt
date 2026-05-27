package com.cameraconnector.app.core

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

interface CoreGateway {
    fun observeDashboard(): Flow<DashboardState>
    fun observeProjects(): Flow<ProjectState>
    suspend fun createProject(name: String): ProjectSummary
    suspend fun setActiveProject(projectId: String)
    suspend fun archiveProject(projectId: String)
    suspend fun restoreProject(projectId: String)
    suspend fun startReceiver()
    suspend fun stopReceiver()
    suspend fun saveReceiverSettings(settings: ReceiverSettings)
    suspend fun saveDeviceAccount(account: DeviceAccount, password: String?)
    suspend fun removeDeviceAccount(username: String)
}

data class DashboardState(
    val receiver: ReceiverState,
    val accounts: List<DeviceAccount>,
    val inbox: List<InboxAsset>,
    val transfers: List<TransferRow>,
    val publishQueue: PublishQueueState = PublishQueueState(),
)

data class ProjectState(
    val projects: List<ProjectSummary>,
    val activeProjectId: String?,
)

data class ProjectSummary(
    val id: String,
    val name: String,
    val slug: String,
    val status: String,
    val createdAtMs: Long,
    val updatedAtMs: Long,
)

data class ReceiverState(
    val running: Boolean,
    val phase: String,
    val protocol: String,
    val authMode: String,
    val accountCount: Int,
    val host: String,
    val port: Int,
    val outputLabel: String,
    val message: String?,
)

data class ReceiverSettings(
    val protocol: String,
    val host: String,
    val ftpPort: Int,
    val sftpPort: Int,
    val outputLabel: String,
)

data class PublishQueueState(
    val totalCount: Int = 0,
    val pendingCount: Int = 0,
    val stagedCount: Int = 0,
    val publishingCount: Int = 0,
    val completedCount: Int = 0,
    val failedCount: Int = 0,
)

data class DeviceAccount(
    val username: String,
    val deviceName: String,
    val passwordConfigured: Boolean,
    val latestIp: String?,
    val latestPort: Int?,
    val activeConnections: Int,
    val lastSeenAtMs: Long?,
    val lastDisconnectedAtMs: Long?,
    val online: Boolean,
)

data class InboxAsset(
    val id: String = "",
    val groupKey: String = "",
    val displayPath: String,
    val format: String,
    val receivedAt: String,
    val username: String? = null,
    val displaySource: String? = null,
    val originalPath: String? = null,
    val sizeBytes: Long? = null,
    val previewLocation: String? = null,
    val rawPath: String? = null,
    val jpegPath: String? = null,
    val videoPath: String? = null,
)

data class TransferRow(
    val id: String,
    val status: String,
    val displayPath: String,
    val message: String?,
)

class PreviewCoreGateway : CoreGateway {
    private val projects = MutableStateFlow(
        ProjectState(
            projects = listOf(
                ProjectSummary(
                    id = "project-preview",
                    name = "Preview Project",
                    slug = "preview-project",
                    status = "Active",
                    createdAtMs = 0,
                    updatedAtMs = 0,
                ),
            ),
            activeProjectId = "project-preview",
        ),
    )

    private val dashboard = MutableStateFlow(
        DashboardState(
            receiver = ReceiverState(
                running = false,
                phase = "Stopped",
                protocol = "FTP",
                authMode = "Accounts",
                accountCount = 1,
                host = "192.168.137.1",
                port = 2121,
                outputLabel = "选择收件箱文件夹",
                message = null,
            ),
            accounts = listOf(
                DeviceAccount(
                    username = "camera01",
                    deviceName = "相机 01",
                    passwordConfigured = true,
                    latestIp = null,
                    latestPort = null,
                    activeConnections = 0,
                    lastSeenAtMs = null,
                    lastDisconnectedAtMs = null,
                    online = false,
                ),
            ),
            inbox = emptyList(),
            transfers = emptyList(),
        ),
    )

    override fun observeDashboard(): Flow<DashboardState> = dashboard.asStateFlow()

    override fun observeProjects(): Flow<ProjectState> = projects.asStateFlow()

    override suspend fun createProject(name: String): ProjectSummary {
        val project = ProjectSummary(
            id = "project-preview-${projects.value.projects.size + 1}",
            name = name,
            slug = name.lowercase()
                .replace(Regex("[^a-z0-9]+"), "-")
                .trim('-')
                .ifBlank { "project" },
            status = "Active",
            createdAtMs = 0,
            updatedAtMs = 0,
        )
        projects.value = ProjectState(
            projects = listOf(project) + projects.value.projects,
            activeProjectId = project.id,
        )
        return project
    }

    override suspend fun setActiveProject(projectId: String) {
        projects.value = projects.value.copy(activeProjectId = projectId)
    }

    override suspend fun archiveProject(projectId: String) {
        val nextProjects = projects.value.projects.map { project ->
            if (project.id == projectId) project.copy(status = "Archived") else project
        }
        val nextActiveProjectId = projects.value.activeProjectId.takeUnless { it == projectId }
            ?: nextProjects.firstOrNull { it.status == "Active" }?.id
        projects.value = ProjectState(
            projects = nextProjects,
            activeProjectId = nextActiveProjectId,
        )
    }

    override suspend fun restoreProject(projectId: String) {
        val nextProjects = projects.value.projects.map { project ->
            if (project.id == projectId) project.copy(status = "Active") else project
        }
        projects.value = projects.value.copy(projects = nextProjects)
    }

    override suspend fun startReceiver() {
        dashboard.value = dashboard.value.copy(
            receiver = dashboard.value.receiver.copy(
                running = true,
                phase = "Running",
                message = null,
            ),
        )
    }

    override suspend fun stopReceiver() {
        dashboard.value = dashboard.value.copy(
            receiver = dashboard.value.receiver.copy(
                running = false,
                phase = "Stopped",
            ),
        )
    }

    override suspend fun saveReceiverSettings(settings: ReceiverSettings) {
        dashboard.value = dashboard.value.copy(
            receiver = ReceiverState(
                running = dashboard.value.receiver.running,
                phase = dashboard.value.receiver.phase,
                protocol = settings.protocol,
                authMode = dashboard.value.receiver.authMode,
                accountCount = dashboard.value.receiver.accountCount,
                host = settings.host,
                port = if (settings.protocol == "SFTP") settings.sftpPort else settings.ftpPort,
                outputLabel = settings.outputLabel,
                message = dashboard.value.receiver.message,
            ),
        )
    }

    override suspend fun saveDeviceAccount(account: DeviceAccount, password: String?) {
        val accountWithPasswordState = account.copy(
            passwordConfigured = account.passwordConfigured || !password.isNullOrBlank(),
        )
        val nextAccounts = dashboard.value.accounts
            .filterNot { it.username == accountWithPasswordState.username } + accountWithPasswordState
        dashboard.value = dashboard.value.copy(
            accounts = nextAccounts,
            receiver = dashboard.value.receiver.copy(accountCount = nextAccounts.size),
        )
    }

    override suspend fun removeDeviceAccount(username: String) {
        val nextAccounts = dashboard.value.accounts.filterNot { it.username == username }
        dashboard.value = dashboard.value.copy(
            accounts = nextAccounts,
            receiver = dashboard.value.receiver.copy(accountCount = nextAccounts.size),
        )
    }
}

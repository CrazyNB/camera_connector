package com.cameraconnector.app.core

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

interface CoreGateway {
    fun observeDashboard(): Flow<DashboardState>
    fun observeProjects(): Flow<ProjectState>
    suspend fun loadInbox(
        query: InboxAssetQuery = InboxAssetQuery(),
        offset: Int = 0,
        limit: Int = 2_000,
    ): List<InboxAsset>
    suspend fun loadSelects(
        projectId: String? = null,
        strategyProfileId: String? = null,
        offset: Int = 0,
        limit: Int = 2_000,
    ): List<InboxAsset>
    suspend fun createProject(name: String): ProjectSummary
    suspend fun setActiveProject(projectId: String)
    suspend fun renameProject(projectId: String, name: String)
    suspend fun archiveProject(projectId: String)
    suspend fun restoreProject(projectId: String)
    suspend fun moveProjectGroup(
        sourceProjectId: String,
        groupId: String,
        targetProjectId: String,
    )
    suspend fun startReceiver()
    suspend fun stopReceiver()
    suspend fun saveReceiverSettings(settings: ReceiverSettings)
    suspend fun saveDeviceAccount(account: DeviceAccount, password: String?)
    suspend fun removeDeviceAccount(username: String)
    suspend fun retryFailedPublishes()
    suspend fun loadStrategyProfiles(): List<StrategyProfileUi>
    suspend fun saveStrategyProfile(profile: StrategyProfileUi): StrategyProfileUi
    suspend fun loadReviewQueueSummary(
        projectId: String? = null,
        strategyProfileId: String? = null,
    ): ReviewQueueSummary
    suspend fun acceptRecommendedBest(burstGroupId: String, strategyProfileId: String? = null)
    suspend fun markBurstNeedsReview(burstGroupId: String, strategyProfileId: String? = null)
    suspend fun restoreAutomaticRecommendation(burstGroupId: String, strategyProfileId: String? = null)
    suspend fun clearRecommendation(burstGroupId: String, strategyProfileId: String? = null)
    suspend fun keepAllCandidates(burstGroupId: String, strategyProfileId: String? = null)
    suspend fun hideLowScoreCandidates(burstGroupId: String, strategyProfileId: String? = null)
    suspend fun overrideRecommendedBest(
        burstGroupId: String,
        bestAssetGroupId: String,
        strategyProfileId: String? = null,
    )
    suspend fun splitBurstMember(burstGroupId: String, memberGroupId: String)
    suspend fun mergeBurstMember(targetBurstGroupId: String, memberGroupId: String)
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
    val canBeActiveProject: Boolean = status.equals("Active", ignoreCase = true),
    val canArchive: Boolean = canBeActiveProject,
    val canRename: Boolean = true,
    val canRestore: Boolean = status.equals("Archived", ignoreCase = true),
    val canAcceptMovedGroups: Boolean = canBeActiveProject,
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
    val hasRaw: Boolean = rawPath != null,
    val hasJpeg: Boolean = jpegPath != null,
    val hasVideo: Boolean = videoPath != null,
    val burst: InboxAssetBurst? = null,
    val quality: InboxAssetQuality? = null,
)

data class InboxAssetBurst(
    val burstGroupId: String,
    val memberCount: Int,
    val memberRank: Int?,
    val recommendationStatus: String?,
    val bestAssetGroupId: String?,
    val bestScore: Double? = null,
)

data class InboxAssetQuality(
    val overall: Double?,
    val analysisStatus: String?,
    val scorerVersion: String?,
    val primaryReason: String?,
    val analyzedAtMs: Long?,
    val sharpness: Double? = null,
    val exposure: Double? = null,
    val highlightClippingPenalty: Double? = null,
    val shadowClippingPenalty: Double? = null,
    val composition: Double? = null,
    val compositionConfidence: Double? = null,
)

data class StrategyWeightsUi(
    val sharpness: Double,
    val exposure: Double,
    val composition: Double,
    val highlightClippingPenalty: Double,
    val shadowClippingPenalty: Double,
    val diversity: Double,
)

data class StrategyProfileUi(
    val profileId: String,
    val name: String,
    val builtIn: Boolean,
    val strategyVersion: String,
    val burstWindowMs: Long,
    val minGroupSize: Int,
    val weights: StrategyWeightsUi,
    val rejectIfSharpnessBelow: Double,
    val flagIfOverallBelow: Double,
    val nearDuplicateSimilarityAbove: Double,
    val maxLlmCandidatesPerGroup: Int = 5,
    val autoDelete: Boolean = false,
    val autoHideLowScore: Boolean,
    val markBest: Boolean = true,
    val keepRawPairs: Boolean = true,
    val llmEnabled: Boolean,
    val updatedAtMs: Long = 0,
)

data class ReviewQueueCount(
    val queue: String,
    val count: Int,
)

data class ReviewQueueSummary(
    val projectId: String,
    val strategyProfileId: String,
    val totalUnits: Int,
    val pendingCount: Int,
    val unconfirmedBestCount: Int,
    val needsReviewCount: Int,
    val lowScoreCandidateCount: Int,
    val nearDuplicateCount: Int,
    val unsupportedCount: Int,
    val userOverriddenCount: Int,
    val queues: List<ReviewQueueCount>,
) {
    fun queueCount(queue: String): Int =
        queues.firstOrNull { it.queue == queue }?.count ?: 0
}

data class InboxAssetQuery(
    val username: String? = null,
    val sourceName: String? = null,
    val originalPath: String? = null,
    val remoteAddr: String? = null,
    val format: String? = null,
    val role: InboxAssetRole? = null,
    val sort: PhotoSortMode = PhotoSortMode.LatestReceived,
    val recommendationState: String? = null,
    val scoreMin: Double? = null,
    val scoreMax: Double? = null,
    val analysisStatus: String? = null,
    val reviewQueue: String? = null,
    val strategyProfileId: String? = null,
)

enum class PhotoSortMode(val wireName: String, val label: String) {
    LatestReceived("latest_received", "最新接收"),
    Filename("filename", "文件名"),
    GroupBestScore("group_best_score", "组内最高评分"),
}

enum class InboxAssetRole(val wireName: String) {
    Jpeg("jpeg"),
    Raw("raw"),
    Video("video"),
}

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
                host = DEFAULT_LISTEN_HOST,
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

    override suspend fun loadInbox(query: InboxAssetQuery, offset: Int, limit: Int): List<InboxAsset> =
        dashboard.value.inbox
            .asSequence()
            .filter { asset -> query.username == null || asset.username == query.username }
            .filter { asset -> query.sourceName == null || asset.displaySource == query.sourceName }
            .filter { asset ->
                query.originalPath == null ||
                    asset.originalPath.orEmpty().contains(query.originalPath, ignoreCase = true)
            }
            .filter { asset -> query.role == null || asset.matchesRole(query.role) }
            .filter { asset ->
                query.analysisStatus == null ||
                    asset.quality?.analysisStatus.equals(query.analysisStatus, ignoreCase = true)
            }
            .filter { asset ->
                query.recommendationState == null ||
                    asset.burst?.recommendationStatus.equals(query.recommendationState, ignoreCase = true)
            }
            .filter { asset ->
                query.scoreMin == null ||
                    (asset.groupBestScore()?.let(::normalizedQueryScore) ?: -1.0) >= normalizedQueryScore(query.scoreMin)
            }
            .filter { asset ->
                query.scoreMax == null ||
                    (asset.groupBestScore()?.let(::normalizedQueryScore) ?: Double.POSITIVE_INFINITY) <= normalizedQueryScore(query.scoreMax)
            }
            .sortedWith(query.sort.previewComparator())
            .let { assets ->
                if (query.reviewQueue.isNullOrBlank()) {
                    assets
                } else {
                    assets.collapsePreviewReviewUnits().asSequence()
                }
            }
            .drop(offset.coerceAtLeast(0))
            .take(limit.coerceAtLeast(0))
            .toList()

    override suspend fun loadSelects(
        projectId: String?,
        strategyProfileId: String?,
        offset: Int,
        limit: Int,
    ): List<InboxAsset> =
        dashboard.value.inbox
            .asSequence()
            .filter { asset ->
                asset.burst?.recommendationStatus?.equals("accepted", ignoreCase = true) == true &&
                    asset.isBestPreviewRepresentative()
            }
            .sortedWith(PhotoSortMode.GroupBestScore.previewComparator())
            .drop(offset.coerceAtLeast(0))
            .take(limit.coerceAtLeast(0))
            .toList()

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

    override suspend fun renameProject(projectId: String, name: String) {
        val nextProjects = projects.value.projects.map { project ->
            if (project.id == projectId) {
                project.copy(
                    name = name,
                    slug = name.lowercase()
                        .replace(Regex("[^a-z0-9]+"), "-")
                        .trim('-')
                        .ifBlank { "project" },
                )
            } else {
                project
            }
        }
        projects.value = projects.value.copy(projects = nextProjects)
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

    override suspend fun moveProjectGroup(
        sourceProjectId: String,
        groupId: String,
        targetProjectId: String,
    ) = Unit

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

    override suspend fun retryFailedPublishes() = Unit

    override suspend fun loadStrategyProfiles(): List<StrategyProfileUi> =
        previewStrategyProfiles()

    override suspend fun saveStrategyProfile(profile: StrategyProfileUi): StrategyProfileUi =
        profile.copy(builtIn = false)

    override suspend fun loadReviewQueueSummary(
        projectId: String?,
        strategyProfileId: String?,
    ): ReviewQueueSummary =
        emptyReviewQueueSummary(
            projectId = projectId ?: projects.value.activeProjectId.orEmpty(),
            strategyProfileId = strategyProfileId ?: "general",
        )

    override suspend fun acceptRecommendedBest(burstGroupId: String, strategyProfileId: String?) = Unit

    override suspend fun markBurstNeedsReview(burstGroupId: String, strategyProfileId: String?) = Unit

    override suspend fun restoreAutomaticRecommendation(
        burstGroupId: String,
        strategyProfileId: String?,
    ) = Unit

    override suspend fun clearRecommendation(burstGroupId: String, strategyProfileId: String?) = Unit

    override suspend fun keepAllCandidates(burstGroupId: String, strategyProfileId: String?) = Unit

    override suspend fun hideLowScoreCandidates(burstGroupId: String, strategyProfileId: String?) = Unit

    override suspend fun overrideRecommendedBest(
        burstGroupId: String,
        bestAssetGroupId: String,
        strategyProfileId: String?,
    ) = Unit

    override suspend fun splitBurstMember(burstGroupId: String, memberGroupId: String) = Unit
    override suspend fun mergeBurstMember(targetBurstGroupId: String, memberGroupId: String) = Unit
}

private fun InboxAsset.matchesRole(role: InboxAssetRole): Boolean = when (role) {
    InboxAssetRole.Jpeg -> hasJpeg
    InboxAssetRole.Raw -> hasRaw
    InboxAssetRole.Video -> hasVideo
}

private fun PhotoSortMode.previewComparator(): Comparator<InboxAsset> = when (this) {
    PhotoSortMode.LatestReceived -> compareByDescending { it.receivedAt.toLongOrNull() ?: 0L }
    PhotoSortMode.Filename -> compareBy { it.groupKey.ifBlank { it.displayPath } }
    PhotoSortMode.GroupBestScore -> compareByDescending { asset ->
        asset.groupBestScore()?.let(::normalizedQueryScore) ?: -1.0
    }
}

private fun InboxAsset.groupBestScore(): Double? =
    burst?.bestScore ?: quality?.overall

private fun Sequence<InboxAsset>.collapsePreviewReviewUnits(): List<InboxAsset> =
    toList()
        .groupBy { asset -> asset.burst?.burstGroupId?.takeIf { it.isNotBlank() } ?: asset.id }
        .values
        .mapNotNull { assets ->
            assets.firstOrNull { it.isBestPreviewRepresentative() }
                ?: assets.maxByOrNull { it.groupBestScore()?.let(::normalizedQueryScore) ?: -1.0 }
                ?: assets.firstOrNull()
        }

private fun InboxAsset.isBestPreviewRepresentative(): Boolean {
    val bestId = burst?.bestAssetGroupId?.takeIf { it.isNotBlank() } ?: return false
    return bestId == id || bestId == groupKey
}

private fun normalizedQueryScore(value: Double): Double =
    if (value > 1.0) value / 100.0 else value

internal fun emptyReviewQueueSummary(
    projectId: String = "",
    strategyProfileId: String = "general",
): ReviewQueueSummary =
    ReviewQueueSummary(
        projectId = projectId,
        strategyProfileId = strategyProfileId,
        totalUnits = 0,
        pendingCount = 0,
        unconfirmedBestCount = 0,
        needsReviewCount = 0,
        lowScoreCandidateCount = 0,
        nearDuplicateCount = 0,
        unsupportedCount = 0,
        userOverriddenCount = 0,
        queues = listOf(
            ReviewQueueCount("pending", 0),
            ReviewQueueCount("unconfirmed_best", 0),
            ReviewQueueCount("needs_review", 0),
            ReviewQueueCount("low_score_candidates", 0),
            ReviewQueueCount("near_duplicates", 0),
            ReviewQueueCount("unsupported", 0),
            ReviewQueueCount("user_overridden", 0),
        ),
    )

private fun previewStrategyProfiles(): List<StrategyProfileUi> =
    listOf(
        StrategyProfileUi(
            profileId = "general",
            name = "General",
            builtIn = true,
            strategyVersion = "strategy-v1",
            burstWindowMs = 1200,
            minGroupSize = 2,
            weights = StrategyWeightsUi(
                sharpness = 0.40,
                exposure = 0.22,
                composition = 0.12,
                highlightClippingPenalty = -0.14,
                shadowClippingPenalty = -0.08,
                diversity = 0.04,
            ),
            rejectIfSharpnessBelow = 0.25,
            flagIfOverallBelow = 0.40,
            nearDuplicateSimilarityAbove = 0.92,
            autoHideLowScore = false,
            llmEnabled = false,
        ),
    )

package com.cameraconnector.app.core

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

interface CoreGateway {
    fun observeDashboard(): Flow<DashboardState>
    fun observeProjects(): Flow<ProjectState>
    suspend fun loadProjectAssets(
        query: ProjectAssetQuery = ProjectAssetQuery(),
        offset: Int = 0,
        limit: Int = 2_000,
    ): List<ProjectAsset>
    suspend fun setAssetGroupUserMarks(
        projectId: String,
        groupId: String,
        favorite: Boolean? = null,
        marked: Boolean? = null,
    ): ProjectAssetUserMarks
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
    suspend fun deleteProjectGroup(projectId: String, groupId: String)
    suspend fun startReceiver()
    suspend fun stopReceiver()
    suspend fun saveReceiverSettings(settings: ReceiverSettings)
    suspend fun saveDeviceAccount(account: DeviceAccount, password: String?)
    suspend fun removeDeviceAccount(username: String)
    suspend fun retryFailedPublishes()
    suspend fun loadModelProviderSettings(): ModelProviderSettingsUi
    suspend fun loadModelProviderSettingsList(): List<ModelProviderSettingsUi>
    suspend fun saveModelProviderSettings(settings: ModelProviderSettingsUi): ModelProviderSettingsUi
    suspend fun deleteModelProviderSettings(settingsId: String)
    suspend fun loadProjectEvaluationSettings(projectId: String): ProjectEvaluationSettingsUi
    suspend fun saveProjectEvaluationSettings(settings: ProjectEvaluationSettingsUi): ProjectEvaluationSettingsUi
    suspend fun loadGlobalPromptProfiles(): List<PromptProfileUi>
    suspend fun createGlobalPromptProfile(
        name: String,
        styleTags: List<String>,
        sceneProfile: String,
        promptText: String,
    ): PromptProfileUi
    suspend fun forkGlobalPromptProfile(sourceProfileId: String, name: String): PromptProfileUi
    suspend fun saveGlobalPromptProfileVersion(promptProfileId: String, promptText: String): PromptProfileUi
    suspend fun loadPromptProfiles(projectId: String): List<PromptProfileUi>
    suspend fun forkPromptProfile(projectId: String, sourceProfileId: String, name: String): PromptProfileUi
    suspend fun savePromptProfileVersion(
        projectId: String,
        promptProfileId: String,
        promptText: String,
    ): PromptProfileUi
    suspend fun generateProjectRecommendation(projectId: String): EvaluationRunUi
    suspend fun generateProjectRecommendationWithCandidateVisuals(
        projectId: String,
        candidateVisuals: List<SelectionCandidateVisualInput>,
    ): EvaluationRunUi
    suspend fun latestProjectRecommendationRunStatus(projectId: String): EvaluationRunUi?
    suspend fun enqueueModelEvaluation(projectId: String, assetGroupIds: List<String>): Int
    suspend fun evaluateAssetGroupsWithModelInputs(
        projectId: String,
        inputs: List<ModelEvaluationPreviewInput>,
    ): Int
    suspend fun recommendBurstGroupWithCandidateVisuals(
        burstGroupId: String,
        candidateVisuals: List<SelectionCandidateVisualInput>,
    ): Boolean
    suspend fun shouldScheduleSubjectAssessment(projectId: String): Boolean
    suspend fun saveSubjectAssessment(assessment: SubjectAssessmentUi): SubjectAssessmentUi
    suspend fun loadSubjectAssessments(projectId: String, groupIds: List<String>): List<SubjectAssessmentUi>
    suspend fun splitBurstMember(burstGroupId: String, memberGroupId: String)
    suspend fun mergeBurstMember(targetBurstGroupId: String, memberGroupId: String)
}

data class DashboardState(
    val receiver: ReceiverState,
    val accounts: List<DeviceAccount>,
    val assets: List<ProjectAsset>,
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

data class ProjectAsset(
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
    val burst: ProjectAssetBurst? = null,
    val technicalGateStatus: String? = null,
    val technicalDefects: List<ProjectAssetTechnicalDefect> = emptyList(),
    val modelStatus: String? = null,
    val modelScore: Int? = null,
    val modelTier: String? = null,
    val modelEvaluatorKind: String? = null,
    val modelSummary: String? = null,
    val isModelSelect: Boolean = false,
    val userMarks: ProjectAssetUserMarks = ProjectAssetUserMarks(),
)

data class ProjectAssetUserMarks(
    val favorite: Boolean = false,
    val marked: Boolean = false,
)

data class ProjectAssetBurst(
    val burstGroupId: String,
    val memberCount: Int,
    val recommendationStatus: String?,
    val bestAssetGroupId: String?,
    val bestScore: Double? = null,
)

data class ProjectAssetTechnicalDefect(
    val defectType: String,
    val severity: String,
    val confidence: Double,
    val reason: String?,
)

data class ModelProviderSettingsUi(
    val settingsId: String = "global",
    val providerKind: String = "none",
    val providerLabel: String = "Model provider",
    val baseUrl: String = "",
    val defaultModel: String = "",
    val defaultMaxImageSide: Int = 1536,
    val defaultSendMode: String = "preview_only",
    val defaultBatchSize: Int = 1,
    val configured: Boolean = false,
    val apiKey: String? = null,
    val apiKeyConfigured: Boolean = false,
    val keyAlias: String? = null,
    val updatedAtMs: Long = 0,
)

data class PromptProfileUi(
    val promptProfileId: String,
    val scope: String,
    val projectId: String?,
    val name: String,
    val styleTags: List<String>,
    val sceneProfile: String,
    val activeVersionId: String?,
    val builtIn: Boolean,
    val enabled: Boolean,
    val activePromptText: String? = null,
    val sharedPreference: String? = null,
    val evaluationInstruction: String? = null,
    val burstSelectionInstruction: String? = null,
    val projectSelectionInstruction: String? = null,
)

data class ProjectEvaluationSettingsUi(
    val projectId: String,
    val autoEvaluateOnUpload: Boolean = false,
    val autoBurstRecommendationEnabled: Boolean = true,
    val projectRecommendationMode: String = "manual",
    val promptProfileId: String? = null,
    val modelProviderSettingsId: String? = null,
    val sceneProfile: String = "general",
    val cvPolicy: String = "standard",
    val cvPolicyOverrides: TechnicalAssessmentPolicyUi? = null,
    val allowRiskyModelSelects: Boolean = false,
    val maxImageSide: Int? = null,
    val batchSize: Int? = null,
    val updatedAtMs: Long = 0,
)

data class TechnicalAssessmentPolicyUi(
    val blurSevereEdgeThreshold: Double,
    val blurSevereFrequencyThreshold: Double,
    val blurHighEdgeThreshold: Double,
    val blurHighFrequencyThreshold: Double,
    val highlightClipThreshold: Int,
    val shadowClipThreshold: Int,
    val clippingHighRatio: Double,
    val clippingHighConnectedRatio: Double,
    val clippingSevereRatio: Double,
    val clippingSevereConnectedRatio: Double,
    val colorCastHighThreshold: Double,
    val colorCastSevereThreshold: Double,
    val faceEyeOpenWarnThreshold: Double = 0.35,
    val faceExposureWarnRatio: Double = 0.25,
    val faceColorCastWarnThreshold: Double = 0.42,
)

data class EvaluationRunUi(
    val runId: String,
    val projectId: String,
    val runType: String,
    val trigger: String,
    val status: String,
    val providerKind: String,
    val providerModel: String,
    val promptProfileId: String? = null,
    val promptVersionId: String? = null,
    val promptHash: String? = null,
    val errorMessage: String? = null,
    val startedAtMs: Long? = null,
    val completedAtMs: Long? = null,
    val createdAtMs: Long = 0,
)

data class SubjectAssessmentUi(
    val assessmentId: String,
    val projectId: String,
    val assetGroupId: String,
    val subjectType: String,
    val detectorKind: String,
    val detectorVersion: String,
    val status: String,
    val gateStatus: String,
    val regionsJson: String,
    val signalsJson: String,
    val summary: String,
    val createdAtMs: Long = 0,
    val updatedAtMs: Long = 0,
)

data class ProjectAssetQuery(
    val role: ProjectAssetRole? = null,
    val sort: PhotoSortMode = PhotoSortMode.LatestReceived,
    val collection: String? = null,
    val favorite: Boolean? = null,
    val marked: Boolean? = null,
)

data class ModelEvaluationPreviewInput(
    val assetGroupId: String,
    val sampleJson: String,
)

data class SelectionCandidateVisualInput(
    val assetGroupId: String,
    val imageDataUrl: String,
)

enum class PhotoSortMode(val wireName: String, val label: String) {
    LatestReceived("latest_received", "\u6700\u65B0\u63A5\u6536"),
    Filename("filename", "\u6587\u4EF6\u540D"),
    ModelScore("model_score", "\u6A21\u578B\u4F18\u5148"),
}

enum class ProjectAssetRole(val wireName: String) {
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
            projects = emptyList(),
            activeProjectId = null,
        ),
    )

    private val dashboard = MutableStateFlow(
        DashboardState(
            receiver = ReceiverState(
                running = false,
                phase = "Stopped",
                protocol = "FTP",
                authMode = "Unknown",
                accountCount = 0,
                host = DEFAULT_LISTEN_HOST,
                port = 2121,
                outputLabel = "\u9009\u62E9\u8F93\u51FA\u6587\u4EF6\u5939",
                message = null,
            ),
            accounts = emptyList(),
            assets = emptyList(),
            transfers = emptyList(),
        ),
    )

    override fun observeDashboard(): Flow<DashboardState> = dashboard.asStateFlow()

    override fun observeProjects(): Flow<ProjectState> = projects.asStateFlow()

    override suspend fun loadProjectAssets(query: ProjectAssetQuery, offset: Int, limit: Int): List<ProjectAsset> =
        dashboard.value.assets
            .asSequence()
            .filter { asset -> query.role == null || asset.matchesRole(query.role) }
            .filter { asset -> query.favorite == null || asset.userMarks.favorite == query.favorite }
            .filter { asset -> query.marked == null || asset.userMarks.marked == query.marked }
            .sortedWith(query.sort.previewComparator())
            .drop(offset.coerceAtLeast(0))
            .take(limit.coerceAtLeast(0))
            .toList()

    override suspend fun setAssetGroupUserMarks(
        projectId: String,
        groupId: String,
        favorite: Boolean?,
        marked: Boolean?,
    ): ProjectAssetUserMarks {
        val nextMarks = dashboard.value.assets
            .firstOrNull { it.id == groupId }
            ?.userMarks
            ?.let {
                ProjectAssetUserMarks(
                    favorite = favorite ?: it.favorite,
                    marked = marked ?: it.marked,
                )
            }
            ?: ProjectAssetUserMarks(
                favorite = favorite ?: false,
                marked = marked ?: false,
            )
        dashboard.value = dashboard.value.copy(
            assets = dashboard.value.assets.map { asset ->
                if (asset.id == groupId) asset.copy(userMarks = nextMarks) else asset
            },
        )
        return nextMarks
    }

    override suspend fun deleteProjectGroup(projectId: String, groupId: String) {
        dashboard.value = dashboard.value.copy(
            assets = dashboard.value.assets.filterNot { it.id == groupId },
        )
    }

    override suspend fun createProject(name: String): ProjectSummary {
        val project = ProjectSummary(
            id = "preview-project-${projects.value.projects.size + 1}",
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

    override suspend fun loadModelProviderSettings(): ModelProviderSettingsUi =
        ModelProviderSettingsUi(providerKind = "none", configured = false)

    override suspend fun loadModelProviderSettingsList(): List<ModelProviderSettingsUi> =
        listOf(loadModelProviderSettings()).filter { it.configured }

    override suspend fun saveModelProviderSettings(settings: ModelProviderSettingsUi): ModelProviderSettingsUi =
        settings.copy(configured = settings.configured && settings.providerKind != "none")

    override suspend fun deleteModelProviderSettings(settingsId: String) = Unit

    override suspend fun loadProjectEvaluationSettings(projectId: String): ProjectEvaluationSettingsUi =
        ProjectEvaluationSettingsUi(projectId = projectId)

    override suspend fun saveProjectEvaluationSettings(
        settings: ProjectEvaluationSettingsUi,
    ): ProjectEvaluationSettingsUi =
        settings.copy(projectRecommendationMode = "manual")

    override suspend fun loadGlobalPromptProfiles(): List<PromptProfileUi> =
        previewPromptProfiles("")

    override suspend fun createGlobalPromptProfile(
        name: String,
        styleTags: List<String>,
        sceneProfile: String,
        promptText: String,
    ): PromptProfileUi =
        PromptProfileUi(
            promptProfileId = "global-custom-${name.ifBlank { "prompt" }}",
            scope = "global",
            projectId = null,
            name = name.ifBlank { "自定义提示词" },
            styleTags = styleTags,
            sceneProfile = sceneProfile.ifBlank { "general" },
            activeVersionId = "preview-version",
            builtIn = false,
            enabled = true,
            activePromptText = promptText,
            sharedPreference = promptText,
        )

    override suspend fun forkGlobalPromptProfile(sourceProfileId: String, name: String): PromptProfileUi =
        previewPromptProfiles("")
            .firstOrNull { it.promptProfileId == sourceProfileId }
            ?.let { source ->
                source.copy(
                    promptProfileId = "global-$sourceProfileId",
                    scope = "global",
                    projectId = null,
                    name = name.ifBlank { "自定义 ${source.name}" },
                    builtIn = false,
                )
            }
            ?: PromptProfileUi(
                promptProfileId = "global-custom",
                scope = "global",
                projectId = null,
                name = name.ifBlank { "自定义提示词" },
                styleTags = emptyList(),
                sceneProfile = "general",
                activeVersionId = null,
                builtIn = false,
                enabled = true,
                activePromptText = "",
            )

    override suspend fun saveGlobalPromptProfileVersion(
        promptProfileId: String,
        promptText: String,
    ): PromptProfileUi =
        previewPromptProfiles("")
            .firstOrNull { it.promptProfileId == promptProfileId }
            ?.copy(builtIn = false, activeVersionId = "preview-version", activePromptText = promptText)
            ?: PromptProfileUi(
                promptProfileId = promptProfileId,
                scope = "global",
                projectId = null,
                name = "自定义提示词",
                styleTags = emptyList(),
                sceneProfile = "general",
                activeVersionId = "preview-version",
                builtIn = false,
                enabled = true,
                activePromptText = promptText,
                sharedPreference = promptText,
            )

    override suspend fun loadPromptProfiles(projectId: String): List<PromptProfileUi> =
        previewPromptProfiles(projectId)

    override suspend fun forkPromptProfile(
        projectId: String,
        sourceProfileId: String,
        name: String,
    ): PromptProfileUi =
        previewPromptProfiles(projectId)
            .firstOrNull { it.promptProfileId == sourceProfileId }
            ?.copy(
                promptProfileId = "project-$sourceProfileId",
                scope = "project",
                projectId = projectId,
                name = name.ifBlank { "Custom prompt" },
                builtIn = false,
            )
            ?: PromptProfileUi(
                promptProfileId = "project-custom",
                scope = "project",
                projectId = projectId,
                name = name.ifBlank { "Custom prompt" },
                styleTags = emptyList(),
                sceneProfile = "general",
                activeVersionId = null,
                builtIn = false,
                enabled = true,
            )

    override suspend fun savePromptProfileVersion(
        projectId: String,
        promptProfileId: String,
        promptText: String,
    ): PromptProfileUi =
        previewPromptProfiles(projectId)
            .firstOrNull { it.promptProfileId == promptProfileId }
            ?.copy(builtIn = false, activeVersionId = "preview-version")
            ?: forkPromptProfile(projectId, promptProfileId, "Custom prompt")

    override suspend fun generateProjectRecommendation(projectId: String): EvaluationRunUi =
        EvaluationRunUi(
            runId = "preview-run",
            projectId = projectId,
            runType = "project_recommendation",
            trigger = "manual",
            status = "skipped",
            providerKind = "none",
            providerModel = "",
        )

    override suspend fun generateProjectRecommendationWithCandidateVisuals(
        projectId: String,
        candidateVisuals: List<SelectionCandidateVisualInput>,
    ): EvaluationRunUi =
        generateProjectRecommendation(projectId)

    override suspend fun latestProjectRecommendationRunStatus(projectId: String): EvaluationRunUi? = null

    override suspend fun enqueueModelEvaluation(projectId: String, assetGroupIds: List<String>): Int =
        assetGroupIds.distinct().size

    override suspend fun evaluateAssetGroupsWithModelInputs(
        projectId: String,
        inputs: List<ModelEvaluationPreviewInput>,
    ): Int =
        inputs.map { it.assetGroupId }.distinct().size

    override suspend fun recommendBurstGroupWithCandidateVisuals(
        burstGroupId: String,
        candidateVisuals: List<SelectionCandidateVisualInput>,
    ): Boolean =
        burstGroupId.isNotBlank() && candidateVisuals.size >= 2

    override suspend fun shouldScheduleSubjectAssessment(projectId: String): Boolean =
        loadProjectEvaluationSettings(projectId).sceneProfile == "portrait"

    override suspend fun saveSubjectAssessment(assessment: SubjectAssessmentUi): SubjectAssessmentUi =
        assessment

    override suspend fun loadSubjectAssessments(
        projectId: String,
        groupIds: List<String>,
    ): List<SubjectAssessmentUi> = emptyList()

    override suspend fun splitBurstMember(burstGroupId: String, memberGroupId: String) = Unit
    override suspend fun mergeBurstMember(targetBurstGroupId: String, memberGroupId: String) = Unit
}

private fun ProjectAsset.matchesRole(role: ProjectAssetRole): Boolean = when (role) {
    ProjectAssetRole.Jpeg -> hasJpeg
    ProjectAssetRole.Raw -> hasRaw
    ProjectAssetRole.Video -> hasVideo
}

private fun PhotoSortMode.previewComparator(): Comparator<ProjectAsset> = when (this) {
    PhotoSortMode.LatestReceived -> compareByDescending { it.receivedAt.toLongOrNull() ?: 0L }
    PhotoSortMode.Filename -> compareBy { it.groupKey.ifBlank { it.displayPath } }
    PhotoSortMode.ModelScore -> compareByDescending { asset ->
        asset.groupBestModelScore()?.let(::normalizedQueryScore) ?: -1.0
    }
}

private fun ProjectAsset.groupBestModelScore(): Double? =
    burst?.bestScore ?: modelScore?.toDouble()

private fun normalizedQueryScore(value: Double): Double =
    if (value > 1.0) value / 100.0 else value

private fun previewPromptProfiles(projectId: String): List<PromptProfileUi> =
    listOf(
        PromptProfileUi(
            promptProfileId = "general-default",
            scope = "global",
            projectId = null,
            name = "General Default",
            styleTags = listOf("general", "balanced"),
            sceneProfile = "general",
            activeVersionId = "general-default-v1",
            builtIn = true,
            enabled = true,
            activePromptText = "Evaluate photographic quality with balanced, concise reasoning.",
            sharedPreference = "Evaluate photographic quality with balanced, concise reasoning.",
        ),
        PromptProfileUi(
            promptProfileId = "portrait-conservative",
            scope = "global",
            projectId = null,
            name = "Portrait Conservative",
            styleTags = listOf("portrait", "conservative"),
            sceneProfile = "portrait",
            activeVersionId = "portrait-conservative-v1",
            builtIn = true,
            enabled = true,
            activePromptText = "Evaluate portrait photos conservatively, prioritizing expression, focus, and skin tone.",
            sharedPreference = "Evaluate portrait photos conservatively, prioritizing expression, focus, and skin tone.",
        ),
    )

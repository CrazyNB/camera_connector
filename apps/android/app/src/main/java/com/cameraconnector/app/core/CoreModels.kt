package com.cameraconnector.app.core
data class DashboardState(
    val receiver: ReceiverState,
    val accounts: List<DeviceAccount>,
    val assets: List<ProjectAsset>,
    val transfers: List<TransferRow>,
    val publishQueue: PublishQueueState = PublishQueueState(),
    val globalAssets: GlobalAssetSummaryUi = GlobalAssetSummaryUi(),
)

data class GlobalAssetSummaryUi(
    val photoCount: Int = 0,
    val fileCount: Int = 0,
    val storageBytes: Long = 0L,
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
    val guestMark: GuestMark? = null,
)

enum class GuestMark(val wireName: String) {
    Favorite("favorite"),
    Marked("marked"),
    Reject("reject"),
}

data class LanShareSessionUi(
    val shareId: String,
    val projectId: String,
    val token: String,
    val title: String? = null,
    val active: Boolean,
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
    val providerLabel: String = "模型服务",
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

data class PromptPackUi(
    val promptPackId: String,
    val distributionFolder: String = "user",
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
    val promptPackId: String? = null,
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
    val promptPackId: String? = null,
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
    val userMarkAny: List<String> = emptyList(),
    val guestMark: String? = null,
    val minModelScore: Int? = null,
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

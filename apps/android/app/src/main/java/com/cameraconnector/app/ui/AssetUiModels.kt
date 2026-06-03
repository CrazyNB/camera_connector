package com.cameraconnector.app.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.graphics.Bitmap
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.rememberTransformableState
import androidx.compose.foundation.gestures.transformable
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.PhotoLibrary
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.SyncAlt
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Shapes
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.InboxAssetQuery
import com.cameraconnector.app.core.InboxAssetRole
import com.cameraconnector.app.core.ProjectState
import com.cameraconnector.app.core.ProjectSummary
import com.cameraconnector.app.core.PublishQueueState
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.media.PREVIEW_DETAIL_FALLBACK_ASPECT_RATIO
import com.cameraconnector.app.media.PhotoMetadata
import com.cameraconnector.app.media.PreviewQuality
import com.cameraconnector.app.media.cacheThumbnailPreview
import com.cameraconnector.app.media.cachedThumbnailPreview
import com.cameraconnector.app.media.isDecodablePreviewLocation
import com.cameraconnector.app.media.loadPhotoMetadata
import com.cameraconnector.app.media.loadPreviewBitmap
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlin.math.roundToInt

internal enum class InboxFilter(val label: String) {
    All("全部文件"),
    Raw("RAW"),
    Jpeg("JPEG"),
    Video("视频"),
}

internal fun InboxFilter.matches(asset: InboxAsset): Boolean = when (this) {
    InboxFilter.All -> true
    InboxFilter.Raw -> asset.hasRaw
    InboxFilter.Jpeg -> asset.hasJpeg
    InboxFilter.Video -> asset.hasVideo
}

internal fun InboxFilter.assetRole(): InboxAssetRole? = when (this) {
    InboxFilter.All -> null
    InboxFilter.Raw -> InboxAssetRole.Raw
    InboxFilter.Jpeg -> InboxAssetRole.Jpeg
    InboxFilter.Video -> InboxAssetRole.Video
}

internal fun InboxAsset.filename(): String =
    displayPath.substringAfterLast('/').substringAfterLast('\\').ifBlank { displayPath }

internal fun InboxAsset.groupTitle(): String =
    groupKey.ifBlank { filename().substringBeforeLast('.', filename()) }

internal fun InboxAsset.groupMoveId(): String? =
    id.takeIf { it.isNotBlank() }

internal fun InboxAsset.sourceLabel(): String =
    displaySource?.takeIf { it.isNotBlank() }
        ?: username?.takeIf { it.isNotBlank() }?.let { "账号：$it" }
        ?: sourceGroupLabel(displayPath)

internal fun InboxAsset.formatBadges(): String =
    buildList {
        if (hasJpeg) add("JPG")
        if (hasRaw) add("RAW")
        if (hasVideo) add("视频")
        if (isEmpty()) add(format.ifBlank { "未知" })
    }.joinToString(" · ")

internal fun InboxAsset.previewAccentColor(): Color = when {
    hasVideo -> ElementWarning
    hasRaw -> ElementPurple
    hasJpeg -> ElementSuccess
    else -> ElementBlue
}

internal fun InboxAsset.qualityScoreText(): String? =
    normalizedScoreText(modelScore?.toDouble() ?: quality?.overall)

internal fun InboxAsset.qualityBadgeText(): String? =
    qualityScoreText()?.let { "模型分 $it" }
        ?: quality?.analysisStatus?.let { qualityStatusLabel(it) }

internal fun InboxAsset.tileQualityBadgeText(): String? =
    when {
        technicalGateStatus in setOf("warn", "reject", "needs_review", "unsupported") -> "风险"
        modelStatus.equals("running", ignoreCase = true) ||
            quality?.analysisStatus.equals("running", ignoreCase = true) -> "分析中"
        quality?.analysisStatus.equals("pending", ignoreCase = true) ||
            quality?.analysisStatus.equals("queued", ignoreCase = true) -> "待分析"
        quality?.analysisStatus.equals("failed", ignoreCase = true) ||
            quality?.analysisStatus.equals("error", ignoreCase = true) -> "分析失败"
        quality?.analysisStatus.equals("unsupported", ignoreCase = true) -> "需复核"
        else -> null
    }

internal fun InboxAsset.groupBestScoreText(): String? =
    normalizedScoreText(burst?.bestScore)

internal fun InboxAsset.groupBestBadgeText(): String? {
    val bestScore = groupBestScoreText() ?: return null
    if (bestScore == qualityScoreText()) {
        return null
    }
    return "组最高 $bestScore"
}

internal fun InboxAsset.burstBadgeText(): String? {
    return burstCountBadgeText()
}

internal fun InboxAsset.burstCountBadgeText(): String? {
    val burst = burst ?: return null
    if (burst.memberCount <= 1) return null
    return burst.memberCount.toString()
}

internal fun InboxAsset.recommendationBadgeText(): String? =
    when {
        isBestRecommendedAsset() -> "优选"
        burst?.recommendationStatus != null -> recommendationStatusLabel(burst.recommendationStatus)
        else -> null
    }

internal fun InboxAsset.tileSmartMeta(): String? =
    listOfNotNull(
        tileQualityReasonText(),
        recommendationBadgeText()?.takeUnless { isBestRecommendedAsset() },
    ).distinct().takeIf { it.isNotEmpty() }?.joinToString(" · ")

private fun InboxAsset.tileQualityReasonText(): String? {
    val status = quality?.analysisStatus?.lowercase()
    if (status in setOf("pending", "queued", "running", "processing", "analyzing")) {
        return qualityStatusLabel(status)
    }
    val hasRisk = technicalGateStatus in setOf("warn", "reject", "needs_review", "unsupported") ||
        status in setOf("failed", "error", "unsupported")
    return if (hasRisk) {
        qualityReasonText() ?: qualityStatusLabel(quality?.analysisStatus)
    } else {
        null
    }
}

internal fun InboxAsset.qualityReasonText(): String? =
    modelSummary
        ?.takeIf { it.isNotBlank() }
        ?.let(::smartReasonText)
        ?: quality?.primaryReason
        ?.takeIf { it.isNotBlank() }
        ?.let(::smartReasonText)

internal fun smartReasonText(reason: String): String {
    val trimmed = reason.trim()
    return when (trimmed.lowercase()) {
        "balanced", "balanced technical score" -> "技术表现均衡"
        "low sharpness" -> "锐度偏低"
        "highlight clipping" -> "高光溢出"
        "shadow clipping" -> "阴影过暗"
        "weak exposure" -> "曝光偏弱"
        "unsupported preview sample" -> "需要复核预览"
        "no supported scores" -> "缺少可评价预览"
        "some frames need review" -> "部分照片需要复核"
        "edge weighted detail" -> "主体靠近边缘"
        "low information area" -> "画面信息偏少"
        else -> if (trimmed.startsWith("best technical score:", ignoreCase = true)) {
            "技术分数领先"
        } else {
            trimmed
        }
    }
}

internal data class QualitySignalRow(
    val label: String,
    val value: String,
)

internal fun InboxAsset.qualitySignalRows(): List<QualitySignalRow> {
    val quality = quality ?: return emptyList()
    return listOfNotNull(
        quality.sharpness?.let { QualitySignalRow("锐度", normalizedScoreText(it).orEmpty()) },
        quality.exposure?.let { QualitySignalRow("曝光", normalizedScoreText(it).orEmpty()) },
        quality.composition?.let { QualitySignalRow("构图", normalizedScoreText(it).orEmpty()) },
        quality.highlightClippingPenalty?.let { QualitySignalRow("高光", normalizedScoreText(it).orEmpty()) },
        quality.shadowClippingPenalty?.let { QualitySignalRow("阴影", normalizedScoreText(it).orEmpty()) },
        quality.compositionConfidence?.let { QualitySignalRow("构图置信", normalizedScoreText(it).orEmpty()) },
    ).filter { it.value.isNotBlank() }
}

internal fun InboxAsset.isBestRecommendedAsset(): Boolean {
    if (isModelSelect) {
        return true
    }
    val bestId = burst?.bestAssetGroupId?.takeIf { it.isNotBlank() } ?: return false
    return bestId == id || bestId == groupKey
}

internal fun qualityStatusLabel(value: String?): String =
    when (value?.lowercase()) {
        "completed", "ready", "done" -> "已评价"
        "pending", "queued" -> "待分析"
        "running", "processing", "analyzing" -> "分析中"
        "stale" -> "更新中"
        "unsupported" -> "不支持评价"
        "failed", "error" -> "评价失败"
        null, "" -> "待分析"
        else -> value
    }

internal fun recommendationStatusLabel(value: String?): String =
    when (value?.lowercase()) {
        "recommended", "completed", "ready", "done" -> "已推荐"
        "accepted" -> "已推荐"
        "needs_review" -> "需要复核"
        "user_overridden" -> "人工变更"
        "pending", "queued" -> "待推荐"
        "running", "processing", "analyzing" -> "推荐中"
        "stale" -> "更新中"
        "unsupported" -> "不支持推荐"
        "failed", "error" -> "推荐失败"
        null, "" -> "待推荐"
        else -> value
    }

internal fun smartBadgeColor(asset: InboxAsset): Color = when {
    asset.isModelSelect -> ElementSuccess
    asset.technicalGateStatus in setOf("warn", "reject", "needs_review", "unsupported") -> ElementDanger
    asset.quality?.analysisStatus?.equals("failed", ignoreCase = true) == true -> ElementDanger
    asset.modelStatus?.equals("running", ignoreCase = true) == true ||
        asset.quality?.analysisStatus?.equals("running", ignoreCase = true) == true -> ElementBlue
    asset.qualityScoreText() != null -> ElementWarning
    asset.burst != null -> ElementPurple
    else -> ElementInfo
}

private fun normalizedScoreText(value: Double?): String? {
    val raw = value?.takeIf { it.isFinite() } ?: return null
    val normalized = if (raw <= 1.0) raw * 100.0 else raw
    return normalized.coerceIn(0.0, 100.0).roundToInt().toString()
}

internal fun sourceGroupLabel(displayPath: String): String =
    displayPath.substringBeforeLast('/', missingDelimiterValue = "未分组").ifBlank { "未分组" }

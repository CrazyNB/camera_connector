package com.cameraconnector.app.ui

import androidx.compose.ui.graphics.Color
import com.cameraconnector.app.core.GuestMark
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetRole
import kotlin.math.roundToInt

internal enum class AssetFormatFilter(val label: String) {
    All("全部文件"),
    Raw("RAW"),
    Jpeg("JPEG"),
    Video("视频"),
}

internal fun AssetFormatFilter.matches(asset: ProjectAsset): Boolean = when (this) {
    AssetFormatFilter.All -> true
    AssetFormatFilter.Raw -> asset.hasRaw
    AssetFormatFilter.Jpeg -> asset.hasJpeg
    AssetFormatFilter.Video -> asset.hasVideo
}

internal fun AssetFormatFilter.assetRole(): ProjectAssetRole? = when (this) {
    AssetFormatFilter.All -> null
    AssetFormatFilter.Raw -> ProjectAssetRole.Raw
    AssetFormatFilter.Jpeg -> ProjectAssetRole.Jpeg
    AssetFormatFilter.Video -> ProjectAssetRole.Video
}

internal fun ProjectAsset.filename(): String =
    displayPath.substringAfterLast('/').substringAfterLast('\\').ifBlank { displayPath }

internal fun ProjectAsset.groupTitle(): String =
    groupKey.ifBlank { filename().substringBeforeLast('.', filename()) }

internal fun ProjectAsset.assetGroupId(): String? =
    id.takeIf { it.isNotBlank() }

internal fun ProjectAsset.sourceLabel(): String =
    displaySource?.takeIf { it.isNotBlank() }
        ?: username?.takeIf { it.isNotBlank() }?.let { "账号 $it" }
        ?: sourceGroupLabel(displayPath)

internal fun ProjectAsset.formatBadges(): String =
    buildList {
        if (hasJpeg) add("JPG")
        if (hasRaw) add("RAW")
        if (hasVideo) add("视频")
        if (isEmpty()) add(format.ifBlank { "未知" })
    }.joinToString(" · ")

internal fun ProjectAsset.previewAccentColor(): Color = when {
    hasVideo -> ElementWarning
    hasRaw -> ElementPurple
    hasJpeg -> ElementSuccess
    else -> ElementBlue
}

internal fun ProjectAsset.modelScoreText(): String? =
    normalizedScoreText(modelScore?.toDouble())

internal fun ProjectAsset.modelBadgeText(): String? =
    modelScoreText()?.let { "模型 $it" }

internal fun ProjectAsset.modelScoreColor(): Color {
    val tier = modelTier?.trim()?.lowercase()
    val score = modelScoreText()?.toIntOrNull()
    return when {
        tier == "reject" || (score != null && score < 40) -> ElementDanger
        tier == "weak" || (score != null && score < 70) -> ElementWarning
        score != null -> ElementSuccess
        else -> ElementInfo
    }
}

internal fun ProjectAsset.tilePrimaryBadgeText(): String? {
    val score = modelScoreText()
    if (score != null) {
        return "评分 $score"
    }
    return when {
        modelStatus.equals("running", ignoreCase = true) ||
            modelStatus.equals("processing", ignoreCase = true) ||
            modelStatus.equals("analyzing", ignoreCase = true) -> "评价中"
        modelStatus.equals("failed", ignoreCase = true) ||
            modelStatus.equals("error", ignoreCase = true) -> "失败"
        modelStatus.equals("pending", ignoreCase = true) ||
            modelStatus.equals("queued", ignoreCase = true) -> "待评"
        hasTechnicalRisk() -> "风险"
        modelStatus.equals("skipped", ignoreCase = true) -> "未评"
        else -> null
    }
}

internal fun ProjectAsset.tilePrimaryBadgeColor(): Color =
    modelScoreText()?.let { modelScoreColor() } ?: when (tilePrimaryBadgeText()) {
        null -> ElementInfo
        "评价中", "待评" -> ElementBlue
        "失败", "风险" -> ElementDanger
        "未评" -> ElementInfo
        else -> ElementWarning
    }

internal fun ProjectAsset.tileAnalysisBadgeText(): String? =
    when {
        hasTechnicalRisk() -> "风险"
        modelStatus.equals("running", ignoreCase = true) -> "\u5206\u6790\u4e2d"
        modelStatus.equals("pending", ignoreCase = true) -> "\u5f85\u5206\u6790"
        modelStatus.equals("failed", ignoreCase = true) -> "分析失败"
        modelStatus.equals("skipped", ignoreCase = true) -> "\u672a\u8bc4\u4ef7"
        else -> null
    }

internal fun ProjectAsset.groupBestModelScoreText(): String? =
    normalizedScoreText(burst?.bestScore)

internal fun ProjectAsset.burstBadgeText(): String? =
    burstCountBadgeText()

internal fun ProjectAsset.burstCountBadgeText(): String? {
    val burst = burst ?: return null
    if (burst.memberCount <= 1) return null
    return burst.memberCount.toString()
}

internal fun ProjectAsset.recommendationBadgeText(): String? =
    when {
        isBestRecommendedAsset() -> "\u4f18\u9009"
        burst?.recommendationStatus?.trim()?.lowercase() in setOf(
            "pending",
            "queued",
            "running",
            "processing",
            "analyzing",
            "stale",
            "unsupported",
            "failed",
            "error",
        ) -> recommendationStatusLabel(burst?.recommendationStatus)
        else -> null
    }

internal fun ProjectAsset.compactFormatBadge(): String? =
    when {
        hasJpeg && hasRaw -> "JPG+RAW"
        hasJpeg -> "JPG"
        hasRaw -> "RAW"
        hasVideo -> "视频"
        else -> null
    }

internal fun ProjectAsset.guestMarkBadgeText(): String? =
    when (guestMark) {
        GuestMark.Favorite -> "访客 收藏"
        GuestMark.Marked -> "访客 标记"
        GuestMark.Reject -> "访客 删除"
        null -> null
    }

internal fun ProjectAsset.tileAuxiliaryBadges(): List<String> =
    buildList {
        guestMarkBadgeText()?.let(::add)
        if (userMarks.favorite) add("收藏")
        if (userMarks.marked) add("标记")
        tileRiskAuxiliaryBadge()?.let(::add)
        compactFormatBadge()?.let(::add)
    }.distinct().take(2)

private fun ProjectAsset.tileRiskAuxiliaryBadge(): String? {
    val risk = technicalRiskStatus() ?: return null
    if (risk !in TECHNICAL_RISK_STATUSES) {
        return null
    }
    return when {
        risk == "unsupported" && tilePrimaryBadgeText() != "风险" -> "不支持预览"
        tilePrimaryBadgeText() != "风险" -> "风险"
        else -> null
    }
}

internal fun ProjectAsset.tileSmartMeta(): String? =
    listOfNotNull(
        tileAnalysisReasonText(),
        recommendationBadgeText()?.takeUnless { isBestRecommendedAsset() },
    ).distinct().takeIf { it.isNotEmpty() }?.joinToString(" · ")

private fun ProjectAsset.tileAnalysisReasonText(): String? {
    val modelSummaryText = modelSummary
        ?.takeIf { it.isNotBlank() }
        ?.let(::smartReasonText)
    if (modelSummaryText != null) {
        return modelSummaryText
    }
    return if (hasTechnicalRisk()) {
        technicalRiskSummary()
    } else {
        null
    }
}

internal fun ProjectAsset.smartSummaryText(): String? =
    modelSummary
        ?.takeIf { it.isNotBlank() }
        ?.let(::smartReasonText)
        ?: technicalRiskSummary()

private fun ProjectAsset.technicalRiskSummary(): String? =
    technicalDefects
        .firstOrNull { it.reason?.isNotBlank() == true }
        ?.reason
        ?.let(::smartReasonText)
        ?: when (technicalRiskStatus()) {
            "warn" -> "\u5b58\u5728\u6280\u672f\u98ce\u9669"
            "reject" -> "\u4e25\u91cd\u6280\u672f\u98ce\u9669"
            "inconclusive" -> "无法判断"
            "unsupported" -> "\u9700\u8981\u4eba\u5de5\u9884\u89c8"
            else -> null
        }

internal fun smartReasonText(reason: String): String {
    val trimmed = reason.trim()
    return when (trimmed.lowercase()) {
        "severe defocus or blur risk" -> "\u4e25\u91cd\u5931\u7126\u6216\u6a21\u7cca"
        "soft detail risk" -> "细节偏软"
        "large highlight clipping risk" -> "\u5927\u9762\u79ef\u9ad8\u5149\u6ea2\u51fa"
        "large shadow clipping risk" -> "\u5927\u9762\u79ef\u6b7b\u9ed1"
        "unsupported preview sample" -> "\u9700\u8981\u4eba\u5de5\u9884\u89c8"
        "passes local technical gate" -> "\u901a\u8fc7\u6280\u672f\u95e8\u63a7"
        "usable with technical warnings" -> "\u53ef\u7528\u4f46\u6709\u6280\u672f\u98ce\u9669"
        "rejected by local technical gate" -> "\u6280\u672f\u95e8\u63a7\u4e0d\u5efa\u8bae\u5165\u9009"
        "unsupported image for model evaluation" -> "暂不支持模型评价"
        else -> trimmed
    }
}

internal fun modelEvaluationStatusLabel(value: String?): String =
    when (value?.trim()?.lowercase()) {
        "ready", "done", "completed" -> "\u5df2\u8bc4\u4ef7"
        "running", "processing", "analyzing" -> "\u8bc4\u4ef7\u4e2d"
        "pending", "queued" -> "\u5f85\u8bc4\u4ef7"
        "skipped" -> "\u672a\u8bc4\u4ef7"
        "failed", "error" -> "\u8bc4\u4ef7\u5931\u8d25"
        null, "" -> "\u672a\u77e5"
        else -> value
    }

internal fun modelEvaluationTierLabel(value: String?): String =
    when (value?.trim()?.lowercase()) {
        "excellent" -> "\u4f18\u79c0"
        "good" -> "\u826f\u597d"
        "normal" -> "\u666e\u901a"
        "weak" -> "\u504f\u5f31"
        "reject" -> "\u4e0d\u5efa\u8bae\u5165\u9009"
        null, "" -> "\u672a\u77e5"
        else -> value
    }

internal fun technicalGateStatusLabel(value: String?): String =
    when (value?.trim()?.lowercase()) {
        "pass" -> "\u901a\u8fc7"
        "warn" -> "\u6709\u98ce\u9669"
        "reject" -> "\u4e25\u91cd\u98ce\u9669"
        "inconclusive" -> "\u65e0\u6cd5\u5224\u65ad"
        "unsupported" -> "\u6682\u4e0d\u652f\u6301"
        null, "" -> "\u672a\u77e5"
        else -> value
    }

internal fun technicalDefectTypeLabel(value: String?): String =
    when (value?.trim()?.lowercase()) {
        "blur" -> "\u6a21\u7cca"
        "highlight_clip" -> "\u9ad8\u5149\u6ea2\u51fa"
        "shadow_clip" -> "\u6697\u90e8\u6b7b\u9ed1"
        "noise" -> "\u9ad8\u566a\u70b9"
        "color_cast" -> "\u504f\u8272"
        "unsupported" -> "\u4e0d\u652f\u6301"
        null, "" -> "\u672a\u77e5"
        else -> value
    }

internal fun ProjectAsset.isBestRecommendedAsset(): Boolean {
    if (isModelSelect) {
        return true
    }
    val bestId = burst?.bestAssetGroupId?.takeIf { it.isNotBlank() } ?: return false
    return bestId == id ||
        bestId == assetGroupId() ||
        bestId == assetSelectionId() ||
        bestId == groupKey
}

internal fun recommendationStatusLabel(value: String?): String =
    when (value?.lowercase()) {
        "recommended", "completed", "ready", "done" -> "\u5df2\u63a8\u8350"
        "pending", "queued" -> "\u5f85\u63a8\u8350"
        "running", "processing", "analyzing" -> "\u63a8\u8350\u4e2d"
        "stale" -> "\u66f4\u65b0\u4e2d"
        "no_selection" -> "\u672a\u63a8\u8350"
        "unsupported" -> "\u4e0d\u652f\u6301\u63a8\u8350"
        "failed", "error" -> "推荐失败"
        null, "" -> "\u5f85\u63a8\u8350"
        else -> value
    }

internal fun smartBadgeColor(asset: ProjectAsset): Color = when {
    asset.isModelSelect -> ElementSuccess
    asset.hasTechnicalRisk() -> ElementDanger
    asset.modelStatus?.equals("failed", ignoreCase = true) == true -> ElementDanger
    asset.modelStatus?.equals("running", ignoreCase = true) == true -> ElementBlue
    asset.modelScoreText() != null -> ElementWarning
    asset.burst != null -> ElementPurple
    else -> ElementInfo
}

private val TECHNICAL_RISK_STATUSES = setOf("warn", "reject", "inconclusive", "unsupported")

internal fun ProjectAsset.hasTechnicalRisk(): Boolean =
    technicalRiskStatus() in TECHNICAL_RISK_STATUSES

internal fun ProjectAsset.technicalRiskStatus(): String? =
    technicalGateStatus?.trim()?.lowercase()?.takeIf { it.isNotBlank() }

private fun normalizedScoreText(value: Double?): String? {
    val raw = value?.takeIf { it.isFinite() } ?: return null
    val normalized = if (raw <= 1.0) raw * 100.0 else raw
    return normalized.coerceIn(0.0, 100.0).roundToInt().toString()
}

internal fun sourceGroupLabel(displayPath: String): String =
    displayPath.substringBeforeLast('/', missingDelimiterValue = "未分组").ifBlank { "未分组" }

package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material.icons.outlined.KeyboardArrowUp
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetTechnicalDefect

@Composable
internal fun SmartSelectionDetailCard(
    asset: ProjectAsset,
    modifier: Modifier = Modifier,
) {
    CompactSmartSelectionDetailCard(
        asset = asset,
        modifier = modifier,
    )
}

@Composable
private fun CompactSmartSelectionDetailCard(
    asset: ProjectAsset,
    modifier: Modifier = Modifier,
) {
    val score = asset.modelScoreText()
    val summary = asset.modelSummaryDisplayText()
    val technicalRisk = asset.compactTechnicalRiskText()
    val summaryExpandable = summary.length > 90
    var summaryExpanded by remember(asset.id, summary) { mutableStateOf(false) }
    ElementCard(modifier = modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(Modifier.weight(1f)) {
                    Text("\u667a\u80fd\u4f18\u9009", style = MaterialTheme.typography.titleMedium)
                }
                Row(
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    score?.let { SmartScorePill(it, asset.modelScoreColor()) }
                }
            }
            Text(
                summary,
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable(enabled = summaryExpandable) { summaryExpanded = !summaryExpanded },
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
                maxLines = if (summaryExpanded) Int.MAX_VALUE else 2,
                overflow = if (summaryExpanded) TextOverflow.Clip else TextOverflow.Ellipsis,
            )
            if (summaryExpandable) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { summaryExpanded = !summaryExpanded },
                    horizontalArrangement = Arrangement.End,
                ) {
                    Icon(
                        imageVector = if (summaryExpanded) {
                            Icons.Outlined.KeyboardArrowUp
                        } else {
                            Icons.Outlined.KeyboardArrowDown
                        },
                        contentDescription = if (summaryExpanded) "\u6536\u8d77\u8bc4\u4ef7\u6458\u8981" else "\u5c55\u5f00\u8bc4\u4ef7\u6458\u8981",
                        tint = ElementBlue,
                        modifier = Modifier.size(18.dp),
                    )
                }
            }
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .horizontalScroll(rememberScrollState()),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                asset.compactModelStatusTag()?.let { ElementTag(it, smartBadgeColor(asset)) }
                asset.recommendationBadgeText()?.let {
                    ElementTag(it, if (asset.isBestRecommendedAsset()) ElementSuccess else ElementInfo)
                }
                asset.compactTechnicalGateTag()?.let { ElementTag(it, ElementDanger) }
            }
            technicalRisk?.let { SmartInsightLine("\u98ce\u9669", it, ElementDanger) }
        }
    }
}

@Composable
private fun SmartScorePill(
    score: String,
    color: Color,
) {
    Surface(
        color = color.copy(alpha = 0.14f),
        contentColor = color,
        shape = RoundedCornerShape(999.dp),
        border = BorderStroke(1.dp, color.copy(alpha = 0.38f)),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 9.dp, vertical = 5.dp),
            horizontalArrangement = Arrangement.spacedBy(3.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(score, style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
            Text("\u5206", style = MaterialTheme.typography.labelSmall)
        }
    }
}

@Composable
private fun SmartInsightLine(
    label: String,
    value: String,
    color: Color,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            label,
            modifier = Modifier.width(34.dp),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.labelSmall,
            maxLines = 1,
        )
        Text(
            value,
            modifier = Modifier.weight(1f),
            color = color,
            style = MaterialTheme.typography.bodySmall,
            fontWeight = FontWeight.SemiBold,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

private fun ProjectAsset.compactModelStatusTag(): String? {
    val status = modelStatus?.trim()?.lowercase()
    return when {
        status in setOf("running", "processing", "analyzing", "pending", "queued", "failed", "error", "skipped") ->
            modelEvaluationStatusLabel(modelStatus)
        else -> modelTier
            ?.takeIf { it.equals("reject", ignoreCase = true) || it.equals("weak", ignoreCase = true) }
            ?.let(::modelEvaluationTierLabel)
    }
}

private fun ProjectAsset.modelSummaryDisplayText(): String =
    modelSummary
        ?.takeIf { it.isNotBlank() }
        ?.let(::smartReasonText)
        ?: when (modelStatus?.trim()?.lowercase()) {
            "running", "processing", "analyzing" -> "\u6b63\u5728\u751f\u6210\u6a21\u578b\u8bc4\u4ef7"
            "pending", "queued" -> "\u7b49\u5f85\u6a21\u578b\u8bc4\u4ef7"
            "failed", "error" -> "\u6a21\u578b\u8bc4\u4ef7\u5931\u8d25\uff0c\u53ef\u91cd\u65b0\u8bc4\u4ef7"
            "ready", "done", "completed" -> "\u6a21\u578b\u8bc4\u4ef7\u5df2\u5b8c\u6210\uff0c\u6682\u65e0\u6587\u5b57\u6458\u8981"
            else -> "\u7b49\u5f85\u6a21\u578b\u8bc4\u4ef7"
        }

internal fun ProjectAsset.modelEvaluationInFlight(): Boolean =
    modelStatus?.trim()?.lowercase() in setOf(
        "pending",
        "queued",
        "running",
        "processing",
        "analyzing",
    )

private fun ProjectAsset.compactTechnicalGateTag(): String? {
    if (technicalDefects.isNotEmpty()) {
        return null
    }
    val gate = technicalRiskStatus() ?: return null
    if (!hasTechnicalRisk()) {
        return null
    }
    return technicalGateStatusLabel(gate)
}

private fun ProjectAsset.compactTechnicalRiskText(): String? {
    if (technicalDefects.isEmpty()) {
        return null
    }
    return technicalDefects
        .take(2)
        .joinToString(" / ") { defect -> defect.userFacingRiskText() }
}

private fun ProjectAssetTechnicalDefect.userFacingRiskText(): String {
    val type = defectType.trim().lowercase()
    val level = severity.trim().lowercase()
    return when (type) {
        "blur" -> when (level) {
            "severe" -> "\u4e25\u91cd\u5931\u7126"
            "high" -> "\u5931\u7126"
            "medium" -> "\u6e05\u6670\u5ea6\u504f\u8f6f"
            "low" -> "\u7ec6\u8282\u7565\u8f6f"
            else -> "\u753b\u9762\u4e0d\u591f\u6e05\u6670"
        }
        "highlight_clip" -> when (level) {
            "severe" -> "\u5927\u9762\u79ef\u8fc7\u66dd"
            "high" -> "\u8fc7\u66dd"
            "medium" -> "\u5c40\u90e8\u8fc7\u66dd"
            "low" -> "\u9ad8\u5149\u7565\u6709\u6ea2\u51fa"
            else -> "\u9ad8\u5149\u8fc7\u66dd"
        }
        "shadow_clip" -> when (level) {
            "severe" -> "\u5927\u9762\u79ef\u6b7b\u9ed1"
            "high" -> "\u6697\u90e8\u6b7b\u9ed1"
            "medium" -> "\u6697\u90e8\u7565\u6709\u6b7b\u9ed1"
            "low" -> "\u6697\u90e8\u7565\u6697"
            else -> "\u6697\u90e8\u6b7b\u9ed1"
        }
        "noise" -> when (level) {
            "severe" -> "\u9ad8\u566a\u70b9\u660e\u663e"
            "high" -> "\u566a\u70b9\u504f\u9ad8"
            "medium" -> "\u7ec6\u8282\u7565\u810f"
            "low" -> "\u8f7b\u5fae\u566a\u70b9"
            else -> "\u566a\u70b9\u504f\u9ad8"
        }
        "color_cast" -> when (level) {
            "severe" -> "\u4e25\u91cd\u504f\u8272"
            "high" -> "\u504f\u8272\u660e\u663e"
            "medium" -> "\u8272\u5f69\u504f\u8272"
            "low" -> "\u8f7b\u5fae\u504f\u8272"
            else -> "\u8272\u5f69\u504f\u8272"
        }
        "unsupported" -> "\u9700\u4eba\u5de5\u786e\u8ba4"
        else -> reason
            ?.takeIf { it.isNotBlank() }
            ?.let(::smartReasonText)
            ?: technicalDefectTypeLabel(defectType)
    }
}

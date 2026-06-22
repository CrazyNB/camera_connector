package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Button
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cameraconnector.app.core.ProjectEvaluationSettingsUi

internal fun projectWorkflowSummary(settings: ProjectEvaluationSettingsUi): String =
    listOf(
        "自动评价${if (settings.autoEvaluateOnUpload) "开" else "关"}",
        "连拍优选${if (settings.autoBurstRecommendationEnabled) "开" else "关"}",
        "风险参与${if (settings.allowRiskyModelSelects) "开" else "关"}",
    ).joinToString(" · ")

internal fun technicalRiskSummary(settings: ProjectEvaluationSettingsUi): String =
    listOfNotNull(
        cvPolicyLabel(settings.cvPolicy),
        if (settings.cvPolicyOverrides != null) "自定义阈值" else null,
    ).joinToString(" · ")

internal fun projectSceneProfileOptions(): List<String> =
    listOf("general", "portrait", "action", "landscape")

internal fun sceneProfileHint(value: String): String =
    when (value.trim().lowercase()) {
        "portrait" -> "优先启用人像相关风险判断，并影响模型评价语境。"
        "action" -> "适合运动、抓拍和动态主体。"
        "landscape" -> "适合风光、建筑和环境类作品。"
        else -> "通用摄影场景，适合大多数项目。"
    }

@Composable
internal fun CvPolicyAdvancedControls(
    projectSettings: ProjectEvaluationSettingsUi,
    actionsEnabled: Boolean,
    onSaveSettings: (ProjectEvaluationSettingsUi) -> Unit,
) {
    val basePolicy = technicalPolicyForCvPolicy(projectSettings.cvPolicy)
    val customPolicy = projectSettings.cvPolicyOverrides
    var draftPolicy by remember(projectSettings.projectId, projectSettings.cvPolicy, customPolicy) {
        mutableStateOf(customPolicy ?: basePolicy)
    }
    val controls = cvThresholdControlSpecs(draftPolicy, sceneProfile = projectSettings.sceneProfile)
    if (customPolicy == null) {
        return
    }
    Surface(
        modifier = Modifier.fillMaxWidth(),
        color = ElementControlSurface.copy(alpha = 0.78f),
        shape = elementShape,
        border = BorderStroke(1.dp, ElementBorder.copy(alpha = 0.85f)),
    ) {
        Column(
            modifier = Modifier.padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text("风险触发灵敏度", style = MaterialTheme.typography.titleMedium)
                    Text(
                        if (projectSettings.sceneProfile.trim().equals("portrait", ignoreCase = true)) {
                            "包含人像闭眼、面部曝光和面部偏色"
                        } else {
                            "包含失焦、死黑死白和偏色"
                        },
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodySmall,
                    )
                }
                Surface(
                    color = ElementBlue.copy(alpha = 0.14f),
                    shape = CircleShape,
                    border = BorderStroke(1.dp, ElementBlue.copy(alpha = 0.5f)),
                ) {
                    Text(
                        sceneProfileLabel(projectSettings.sceneProfile),
                        modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp),
                        color = ElementBlue,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            }
            controls.forEachIndexed { index, control ->
                ThresholdSlider(
                    title = control.title,
                    value = control.sliderValue,
                    displayLabel = control.displayLabel,
                    description = control.description,
                    enabled = actionsEnabled,
                    onValueChange = {
                        draftPolicy = updateCvThresholdControl(draftPolicy, control.key, it)
                    },
                )
                if (index != controls.lastIndex) {
                    HorizontalDivider(color = ElementBorder.copy(alpha = 0.6f))
                }
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                OutlinedButton(
                    onClick = {
                        draftPolicy = basePolicy
                        onSaveSettings(projectSettings.copy(cvPolicyOverrides = basePolicy))
                    },
                    enabled = actionsEnabled,
                    modifier = Modifier.weight(1f),
                    shape = elementShape,
                ) {
                    Text("重置预设")
                }
                Button(
                    onClick = { onSaveSettings(projectSettings.copy(cvPolicyOverrides = draftPolicy)) },
                    enabled = actionsEnabled && draftPolicy != customPolicy,
                    modifier = Modifier.weight(1f),
                    shape = elementShape,
                ) {
                    Text("应用阈值")
                }
            }
        }
    }
}

@Composable
private fun ThresholdSlider(
    title: String,
    value: Double,
    displayLabel: String,
    description: String,
    enabled: Boolean,
    onValueChange: (Double) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(5.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(title, style = MaterialTheme.typography.titleSmall)
            Surface(
                color = ElementBlue.copy(alpha = 0.12f),
                shape = CircleShape,
            ) {
                Text(
                    displayLabel,
                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp),
                    color = ElementBlue,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.SemiBold,
                )
            }
        }
        CompactThresholdSlider(
            value = value,
            enabled = enabled,
            onValueChange = onValueChange,
            modifier = Modifier.fillMaxWidth(),
        )
        Text(
            description,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.bodySmall,
            lineHeight = 16.sp,
        )
    }
}

@Composable
private fun CompactThresholdSlider(
    value: Double,
    enabled: Boolean,
    onValueChange: (Double) -> Unit,
    modifier: Modifier = Modifier,
) {
    var widthPx by remember { mutableStateOf(0) }
    val activeColor = if (enabled) ElementBlue else ElementBlue.copy(alpha = 0.35f)
    val inactiveColor = ElementBorder.copy(alpha = if (enabled) 0.58f else 0.28f)
    val thumbColor = if (enabled) ElementBlue else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)
    val normalized = value.coerceIn(0.0, 1.0).toFloat()
    val updateFromX: (Float) -> Unit = { x ->
        if (enabled && widthPx > 0) {
            onValueChange((x / widthPx.toFloat()).coerceIn(0f, 1f).toDouble())
        }
    }

    Canvas(
        modifier = modifier
            .height(24.dp)
            .onSizeChanged { widthPx = it.width }
            .pointerInput(enabled, widthPx) {
                detectTapGestures { offset -> updateFromX(offset.x) }
            }
            .pointerInput(enabled, widthPx) {
                detectDragGestures { change, _ ->
                    change.consume()
                    updateFromX(change.position.x)
                }
            },
    ) {
        val horizontalPadding = 4.dp.toPx()
        val startX = horizontalPadding
        val endX = size.width - horizontalPadding
        val centerY = size.height / 2f
        val usableWidth = (endX - startX).coerceAtLeast(1f)
        val activeEndX = startX + usableWidth * normalized
        val trackStroke = 7.dp.toPx()
        val thumbStroke = 3.dp.toPx()

        drawLine(
            color = inactiveColor,
            start = Offset(startX, centerY),
            end = Offset(endX, centerY),
            strokeWidth = trackStroke,
            cap = StrokeCap.Round,
        )
        drawLine(
            color = activeColor,
            start = Offset(startX, centerY),
            end = Offset(activeEndX, centerY),
            strokeWidth = trackStroke,
            cap = StrokeCap.Round,
        )
        drawLine(
            color = thumbColor,
            start = Offset(activeEndX, centerY - 9.dp.toPx()),
            end = Offset(activeEndX, centerY + 9.dp.toPx()),
            strokeWidth = thumbStroke,
            cap = StrokeCap.Round,
        )
        drawCircle(
            color = activeColor,
            radius = 2.dp.toPx(),
            center = Offset(endX, centerY),
        )
    }
}

@Composable
internal fun SettingsSwitchRow(
    title: String,
    checked: Boolean,
    enabled: Boolean,
    onDisabledClick: (() -> Unit)? = null,
    onCheckedChange: (Boolean) -> Unit,
) {
    val rowClickEnabled = enabled || onDisabledClick != null
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = rowClickEnabled) {
                if (enabled) {
                    onCheckedChange(!checked)
                } else {
                    onDisabledClick?.invoke()
                }
            },
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            title,
            color = if (enabled) {
                MaterialTheme.colorScheme.onSurface
            } else {
                MaterialTheme.colorScheme.onSurfaceVariant
            },
        )
        Switch(checked = checked, enabled = enabled, onCheckedChange = onCheckedChange)
    }
}

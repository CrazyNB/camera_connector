package com.cameraconnector.app.ui

import com.cameraconnector.app.core.ProjectEvaluationSettingsUi
import com.cameraconnector.app.core.TechnicalAssessmentPolicyUi
import java.util.Locale
import kotlin.math.roundToInt

internal fun selectedCvThresholdMode(settings: ProjectEvaluationSettingsUi): String =
    if (settings.cvPolicyOverrides != null) {
        "custom"
    } else {
        settings.cvPolicy.ifBlank { "standard" }
    }

internal fun projectSettingsAfterCvThresholdModeSelection(
    settings: ProjectEvaluationSettingsUi,
    selectedMode: String,
): ProjectEvaluationSettingsUi {
    val mode = selectedMode.trim().lowercase()
    if (mode == "custom") {
        val baseMode = settings.cvPolicy.ifBlank { "standard" }
        return settings.copy(
            cvPolicy = baseMode,
            cvPolicyOverrides = settings.cvPolicyOverrides ?: technicalPolicyForCvPolicy(baseMode),
        )
    }
    val preset = when (mode) {
        "loose", "standard", "strict" -> mode
        else -> "standard"
    }
    return settings.copy(cvPolicy = preset, cvPolicyOverrides = null)
}

internal fun cvThresholdModeLabel(value: String): String =
    when (value.trim().lowercase()) {
        "custom" -> "自定义"
        else -> cvPolicyLabel(value)
    }


internal enum class CvThresholdControlKey {
    BlurHigh,
    Clipping,
    ShadowClipThreshold,
    HighlightClipThreshold,
    ColorCast,
    FaceEyes,
    FaceExposure,
    FaceColorCast,
}

internal data class CvThresholdControlSpec(
    val key: CvThresholdControlKey,
    val title: String,
    val sliderValue: Double,
    val displayPercent: Int,
    val displayLabel: String,
    val description: String,
)

internal fun cvThresholdControlSpecs(
    policy: TechnicalAssessmentPolicyUi,
    sceneProfile: String = "general",
): List<CvThresholdControlSpec> {
    val controls = mutableListOf(
        CvThresholdControlSpec(
            key = CvThresholdControlKey.BlurHigh,
            title = "失焦灵敏度",
            sliderValue = blurSensitivity(policy),
            displayPercent = percentLabel(blurSensitivity(policy)),
            displayLabel = "${percentLabel(blurSensitivity(policy))}%",
            description = blurThresholdDescription(policy),
        ),
        CvThresholdControlSpec(
            key = CvThresholdControlKey.Clipping,
            title = "死黑/死白灵敏度",
            sliderValue = clippingSensitivity(policy),
            displayPercent = percentLabel(clippingSensitivity(policy)),
            displayLabel = "${percentLabel(clippingSensitivity(policy))}%",
            description = clippingThresholdDescription(policy),
        ),
        CvThresholdControlSpec(
            key = CvThresholdControlKey.ShadowClipThreshold,
            title = "\u8fd1\u9ed1\u8fb9\u754c",
            sliderValue = shadowClipThresholdValue(policy),
            displayPercent = policy.shadowClipThreshold,
            displayLabel = "<=${policy.shadowClipThreshold}",
            description = "\u4eae\u5ea6\u5c0f\u4e8e\u7b49\u4e8e ${policy.shadowClipThreshold} \u7684\u50cf\u7d20\u8ba1\u5165\u6697\u90e8\u6b7b\u9ed1\u3002\u6570\u503c\u8d8a\u4f4e\uff0c\u8bef\u62a5\u8d8a\u5c11\u3002",
        ),
        CvThresholdControlSpec(
            key = CvThresholdControlKey.HighlightClipThreshold,
            title = "\u8fd1\u767d\u8fb9\u754c",
            sliderValue = highlightClipThresholdValue(policy),
            displayPercent = policy.highlightClipThreshold,
            displayLabel = ">=${policy.highlightClipThreshold}",
            description = "\u4eae\u5ea6\u5927\u4e8e\u7b49\u4e8e ${policy.highlightClipThreshold} \u7684\u50cf\u7d20\u8ba1\u5165\u9ad8\u5149\u6ea2\u51fa\u3002\u6570\u503c\u8d8a\u9ad8\uff0c\u5224\u5b9a\u8d8a\u4fdd\u5b88\u3002",
        ),
        CvThresholdControlSpec(
            key = CvThresholdControlKey.ColorCast,
            title = "偏色灵敏度",
            sliderValue = colorCastSensitivity(policy),
            displayPercent = percentLabel(colorCastSensitivity(policy)),
            displayLabel = "${percentLabel(colorCastSensitivity(policy))}%",
            description = colorCastThresholdDescription(policy),
        ),
    )
    if (sceneProfile.trim().equals("portrait", ignoreCase = true)) {
        controls += CvThresholdControlSpec(
            key = CvThresholdControlKey.FaceEyes,
            title = "闭眼灵敏度",
            sliderValue = faceEyesSensitivity(policy),
            displayPercent = percentLabel(faceEyesSensitivity(policy)),
            displayLabel = "${percentLabel(faceEyesSensitivity(policy))}%",
            description = faceEyesThresholdDescription(policy),
        )
        controls += CvThresholdControlSpec(
            key = CvThresholdControlKey.FaceExposure,
            title = "面部死黑/死白灵敏度",
            sliderValue = faceExposureSensitivity(policy),
            displayPercent = percentLabel(faceExposureSensitivity(policy)),
            displayLabel = "${percentLabel(faceExposureSensitivity(policy))}%",
            description = faceExposureThresholdDescription(policy),
        )
        controls += CvThresholdControlSpec(
            key = CvThresholdControlKey.FaceColorCast,
            title = "面部偏色灵敏度",
            sliderValue = faceColorCastSensitivity(policy),
            displayPercent = percentLabel(faceColorCastSensitivity(policy)),
            displayLabel = "${percentLabel(faceColorCastSensitivity(policy))}%",
            description = faceColorCastThresholdDescription(policy),
        )
    }
    return controls
}

internal fun updateCvThresholdControl(
    policy: TechnicalAssessmentPolicyUi,
    key: CvThresholdControlKey,
    value: Double,
): TechnicalAssessmentPolicyUi =
    when (key) {
        CvThresholdControlKey.BlurHigh -> {
            val next = denormalize(value, BLUR_HIGH_MIN, BLUR_HIGH_MAX)
            policy.copy(
                blurHighEdgeThreshold = next,
                blurHighFrequencyThreshold = next,
                blurSevereEdgeThreshold = policy.blurSevereEdgeThreshold.coerceAtMost(next),
                blurSevereFrequencyThreshold = policy.blurSevereFrequencyThreshold.coerceAtMost(next),
            )
        }
        CvThresholdControlKey.Clipping -> {
            val sensitivity = value.coerceIn(0.0, 1.0)
            policy.copy(
                clippingHighRatio = inverseDenormalize(sensitivity, CLIPPING_HIGH_MIN, CLIPPING_HIGH_MAX),
                clippingHighConnectedRatio = inverseDenormalize(
                    sensitivity,
                    CLIPPING_HIGH_CONNECTED_MIN,
                    CLIPPING_HIGH_CONNECTED_MAX,
                ),
                clippingSevereRatio = inverseDenormalize(
                    sensitivity,
                    CLIPPING_SEVERE_MIN,
                    CLIPPING_SEVERE_MAX,
                ),
                clippingSevereConnectedRatio = inverseDenormalize(
                    sensitivity,
                    CLIPPING_SEVERE_MIN,
                    CLIPPING_SEVERE_MAX,
                ),
            )
        }
        CvThresholdControlKey.ShadowClipThreshold -> {
            policy.copy(
                shadowClipThreshold = denormalize(
                    value,
                    SHADOW_CLIP_THRESHOLD_MIN.toDouble(),
                    SHADOW_CLIP_THRESHOLD_MAX.toDouble(),
                ).roundToInt().coerceIn(SHADOW_CLIP_THRESHOLD_MIN, SHADOW_CLIP_THRESHOLD_MAX),
            )
        }
        CvThresholdControlKey.HighlightClipThreshold -> {
            policy.copy(
                highlightClipThreshold = denormalize(
                    value,
                    HIGHLIGHT_CLIP_THRESHOLD_MIN.toDouble(),
                    HIGHLIGHT_CLIP_THRESHOLD_MAX.toDouble(),
                ).roundToInt().coerceIn(HIGHLIGHT_CLIP_THRESHOLD_MIN, HIGHLIGHT_CLIP_THRESHOLD_MAX),
            )
        }
        CvThresholdControlKey.ColorCast -> {
            val sensitivity = value.coerceIn(0.0, 1.0)
            policy.copy(
                colorCastHighThreshold = inverseDenormalize(
                    sensitivity,
                    COLOR_CAST_HIGH_MIN,
                    COLOR_CAST_HIGH_MAX,
                ),
                colorCastSevereThreshold = inverseDenormalize(
                    sensitivity,
                    COLOR_CAST_SEVERE_MIN,
                    COLOR_CAST_SEVERE_MAX,
                ),
            )
        }
        CvThresholdControlKey.FaceEyes -> {
            val next = denormalize(value, FACE_EYE_OPEN_WARN_MIN, FACE_EYE_OPEN_WARN_MAX)
            policy.copy(faceEyeOpenWarnThreshold = next)
        }
        CvThresholdControlKey.FaceExposure -> {
            val next = inverseDenormalize(value, FACE_EXPOSURE_WARN_MIN, FACE_EXPOSURE_WARN_MAX)
            policy.copy(faceExposureWarnRatio = next)
        }
        CvThresholdControlKey.FaceColorCast -> {
            val next = inverseDenormalize(value, FACE_COLOR_CAST_WARN_MIN, FACE_COLOR_CAST_WARN_MAX)
            policy.copy(faceColorCastWarnThreshold = next)
        }
    }


private const val BLUR_HIGH_MIN = 0.06
private const val BLUR_HIGH_MAX = 0.22
private const val CLIPPING_HIGH_MIN = 0.04
private const val CLIPPING_HIGH_MAX = 0.30
private const val CLIPPING_HIGH_CONNECTED_MIN = 0.04
private const val CLIPPING_HIGH_CONNECTED_MAX = 0.30
private const val CLIPPING_SEVERE_MIN = 0.35
private const val CLIPPING_SEVERE_MAX = 0.75
private const val SHADOW_CLIP_THRESHOLD_MIN = 0
private const val SHADOW_CLIP_THRESHOLD_MAX = 15
private const val HIGHLIGHT_CLIP_THRESHOLD_MIN = 235
private const val HIGHLIGHT_CLIP_THRESHOLD_MAX = 255
private const val COLOR_CAST_HIGH_MIN = 0.28
private const val COLOR_CAST_HIGH_MAX = 0.65
private const val COLOR_CAST_SEVERE_MIN = 0.50
private const val COLOR_CAST_SEVERE_MAX = 0.90
private const val FACE_EYE_OPEN_WARN_MIN = 0.20
private const val FACE_EYE_OPEN_WARN_MAX = 0.55
private const val FACE_EXPOSURE_WARN_MIN = 0.12
private const val FACE_EXPOSURE_WARN_MAX = 0.40
private const val FACE_COLOR_CAST_WARN_MIN = 0.28
private const val FACE_COLOR_CAST_WARN_MAX = 0.65

private fun blurSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    normalize(policy.blurHighEdgeThreshold, BLUR_HIGH_MIN, BLUR_HIGH_MAX)

private fun clippingSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    listOf(
        inverseNormalize(policy.clippingHighRatio, CLIPPING_HIGH_MIN, CLIPPING_HIGH_MAX),
        inverseNormalize(
            policy.clippingHighConnectedRatio,
            CLIPPING_HIGH_CONNECTED_MIN,
            CLIPPING_HIGH_CONNECTED_MAX,
        ),
        inverseNormalize(policy.clippingSevereRatio, CLIPPING_SEVERE_MIN, CLIPPING_SEVERE_MAX),
    ).average().coerceIn(0.0, 1.0)

private fun shadowClipThresholdValue(policy: TechnicalAssessmentPolicyUi): Double =
    normalize(
        policy.shadowClipThreshold.toDouble(),
        SHADOW_CLIP_THRESHOLD_MIN.toDouble(),
        SHADOW_CLIP_THRESHOLD_MAX.toDouble(),
    )

private fun highlightClipThresholdValue(policy: TechnicalAssessmentPolicyUi): Double =
    normalize(
        policy.highlightClipThreshold.toDouble(),
        HIGHLIGHT_CLIP_THRESHOLD_MIN.toDouble(),
        HIGHLIGHT_CLIP_THRESHOLD_MAX.toDouble(),
    )

private fun colorCastSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    listOf(
        inverseNormalize(policy.colorCastHighThreshold, COLOR_CAST_HIGH_MIN, COLOR_CAST_HIGH_MAX),
        inverseNormalize(policy.colorCastSevereThreshold, COLOR_CAST_SEVERE_MIN, COLOR_CAST_SEVERE_MAX),
    ).average().coerceIn(0.0, 1.0)

private fun faceEyesSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    normalize(policy.faceEyeOpenWarnThreshold, FACE_EYE_OPEN_WARN_MIN, FACE_EYE_OPEN_WARN_MAX)

private fun faceExposureSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    inverseNormalize(policy.faceExposureWarnRatio, FACE_EXPOSURE_WARN_MIN, FACE_EXPOSURE_WARN_MAX)

private fun faceColorCastSensitivity(policy: TechnicalAssessmentPolicyUi): Double =
    inverseNormalize(policy.faceColorCastWarnThreshold, FACE_COLOR_CAST_WARN_MIN, FACE_COLOR_CAST_WARN_MAX)

private fun percentLabel(value: Double): Int =
    (value.coerceIn(0.0, 1.0) * 100).roundToInt()

private fun blurThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：边缘和高频细节都低于 ${formatRatioPercent(policy.blurHighEdgeThreshold)} 时标记失焦；" +
        "低于 ${formatRatioPercent(policy.blurSevereEdgeThreshold)} 视为严重。"

private fun clippingThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：近黑 <=${policy.shadowClipThreshold} / 近白 >=${policy.highlightClipThreshold}，" +
        "占比超过 ${formatRatioPercent(policy.clippingHighRatio)} 或连片超过 ${formatRatioPercent(policy.clippingHighConnectedRatio)} 时标记；" +
        "${formatRatioPercent(policy.clippingSevereRatio)} 以上视为严重。"

private fun colorCastThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：RGB 通道相对亮度差异超过 ${formatDecimal(policy.colorCastHighThreshold, 2)} 时标记偏色；" +
        "超过 ${formatDecimal(policy.colorCastSevereThreshold, 2)} 视为严重。"

private fun faceEyesThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：检测到人脸时，任一眼睁开概率低于 ${formatDecimal(policy.faceEyeOpenWarnThreshold, 2)} 标记闭眼风险。"

private fun faceExposureThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：人脸区域近黑/近白像素占比超过 ${formatRatioPercent(policy.faceExposureWarnRatio)} 标记面部曝光风险。"

private fun faceColorCastThresholdDescription(policy: TechnicalAssessmentPolicyUi): String =
    "当前：人脸区域 RGB 相对亮度差异超过 ${formatDecimal(policy.faceColorCastWarnThreshold, 2)} 标记面部偏色。"

private fun formatRatioPercent(value: Double): String =
    "${percentLabel(value)}%"

private fun formatDecimal(value: Double, digits: Int): String =
    "%.${digits}f".format(Locale.US, value)

private fun normalize(value: Double, min: Double, max: Double): Double =
    ((value - min) / (max - min)).coerceIn(0.0, 1.0)

private fun inverseNormalize(value: Double, min: Double, max: Double): Double =
    ((max - value) / (max - min)).coerceIn(0.0, 1.0)

private fun denormalize(value: Double, min: Double, max: Double): Double =
    min + (max - min) * value.coerceIn(0.0, 1.0)

private fun inverseDenormalize(value: Double, min: Double, max: Double): Double =
    max - (max - min) * value.coerceIn(0.0, 1.0)

internal fun technicalPolicyForCvPolicy(value: String): TechnicalAssessmentPolicyUi =
    when (value.trim().lowercase()) {
        "loose" -> TechnicalAssessmentPolicyUi(
            blurSevereEdgeThreshold = 0.025,
            blurSevereFrequencyThreshold = 0.025,
            blurHighEdgeThreshold = 0.09,
            blurHighFrequencyThreshold = 0.09,
            highlightClipThreshold = 250,
            shadowClipThreshold = 2,
            clippingHighRatio = 0.18,
            clippingHighConnectedRatio = 0.25,
            clippingSevereRatio = 0.65,
            clippingSevereConnectedRatio = 0.65,
            colorCastHighThreshold = 0.55,
            colorCastSevereThreshold = 0.85,
            faceEyeOpenWarnThreshold = 0.25,
            faceExposureWarnRatio = 0.35,
            faceColorCastWarnThreshold = 0.55,
        )
        "strict" -> TechnicalAssessmentPolicyUi(
            blurSevereEdgeThreshold = 0.06,
            blurSevereFrequencyThreshold = 0.06,
            blurHighEdgeThreshold = 0.16,
            blurHighFrequencyThreshold = 0.16,
            highlightClipThreshold = 242,
            shadowClipThreshold = 8,
            clippingHighRatio = 0.08,
            clippingHighConnectedRatio = 0.12,
            clippingSevereRatio = 0.40,
            clippingSevereConnectedRatio = 0.40,
            colorCastHighThreshold = 0.32,
            colorCastSevereThreshold = 0.55,
            faceEyeOpenWarnThreshold = 0.45,
            faceExposureWarnRatio = 0.16,
            faceColorCastWarnThreshold = 0.32,
        )
        else -> TechnicalAssessmentPolicyUi(
            blurSevereEdgeThreshold = 0.04,
            blurSevereFrequencyThreshold = 0.04,
            blurHighEdgeThreshold = 0.12,
            blurHighFrequencyThreshold = 0.12,
            highlightClipThreshold = 245,
            shadowClipThreshold = 5,
            clippingHighRatio = 0.12,
            clippingHighConnectedRatio = 0.18,
            clippingSevereRatio = 0.50,
            clippingSevereConnectedRatio = 0.50,
            colorCastHighThreshold = 0.42,
            colorCastSevereThreshold = 0.70,
            faceEyeOpenWarnThreshold = 0.35,
            faceExposureWarnRatio = 0.25,
            faceColorCastWarnThreshold = 0.42,
        )
    }

internal fun cvPolicyHint(value: String): String =
    when (value.trim().lowercase()) {
        "loose" -> "减少误报，只标记明显失焦、死黑和过曝。"
        "strict" -> "更早提示风险，适合需要严格筛片的项目。"
        else -> "平衡误报和漏报，适合大多数项目。"
    }

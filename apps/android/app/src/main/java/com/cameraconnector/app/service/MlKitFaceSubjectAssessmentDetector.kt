package com.cameraconnector.app.service

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Rect
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.media.PreviewQuality
import com.cameraconnector.app.media.loadPreviewBitmap
import com.google.android.gms.tasks.Tasks
import com.google.mlkit.vision.common.InputImage
import com.google.mlkit.vision.face.Face
import com.google.mlkit.vision.face.FaceDetection
import com.google.mlkit.vision.face.FaceDetectorOptions
import org.json.JSONArray
import org.json.JSONObject
import java.util.concurrent.TimeUnit
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToInt
import kotlin.math.sqrt

class MlKitFaceSubjectAssessmentDetector : SubjectAssessmentDetector {
    override fun assess(
        context: Context?,
        projectId: String,
        asset: ProjectAsset,
        policy: SubjectAssessmentPolicy,
    ): JSONObject? {
        val now = System.currentTimeMillis()
        if (context == null) {
            return skippedAssessment(projectId, asset, "缺少 Android 上下文，未进行人脸检测。", now)
        }
        val bitmap = loadPreviewBitmap(context, asset.previewLocation, PreviewQuality.Detail)
            ?: return skippedAssessment(projectId, asset, "无法读取预览图，未进行人脸检测。", now)
        val detector = FaceDetection.getClient(faceDetectorOptions())
        return try {
            val faces = Tasks.await(
                detector.process(InputImage.fromBitmap(bitmap, 0)),
                FACE_DETECTION_TIMEOUT_SECONDS,
                TimeUnit.SECONDS,
            )
            readyAssessment(projectId, asset, bitmap, faces, policy, now)
        } finally {
            detector.close()
        }
    }
}

private fun faceDetectorOptions(): FaceDetectorOptions =
    FaceDetectorOptions.Builder()
        .setPerformanceMode(FaceDetectorOptions.PERFORMANCE_MODE_ACCURATE)
        .setLandmarkMode(FaceDetectorOptions.LANDMARK_MODE_ALL)
        .setClassificationMode(FaceDetectorOptions.CLASSIFICATION_MODE_ALL)
        .setMinFaceSize(MIN_FACE_SIZE_RATIO)
        .build()

private fun readyAssessment(
    projectId: String,
    asset: ProjectAsset,
    bitmap: Bitmap,
    faces: List<Face>,
    policy: SubjectAssessmentPolicy,
    now: Long,
): JSONObject {
    val analyses = faces.map { face -> faceRegionAnalysis(bitmap, face.boundingBox, policy) }
    val primaryAnalysis = analyses.maxByOrNull { it.areaRatio }
    val eyeOpenMin = faces.mapNotNull(::faceEyeOpenMin).minOrNull()
    val closedEyes = eyeOpenMin?.let { it < policy.eyeOpenWarnThreshold } ?: false
    val shadowRatio = primaryAnalysis?.shadowRatio ?: 0.0
    val highlightRatio = primaryAnalysis?.highlightRatio ?: 0.0
    val colorCastStrength = primaryAnalysis?.colorCastStrength ?: 0.0
    val gateStatus = when {
        faces.isEmpty() -> "warn"
        closedEyes -> "warn"
        shadowRatio >= policy.faceExposureWarnRatio -> "warn"
        highlightRatio >= policy.faceExposureWarnRatio -> "warn"
        colorCastStrength >= policy.faceColorCastWarnThreshold -> "warn"
        else -> "pass"
    }
    return baseAssessment(projectId, asset, status = "ready", gateStatus = gateStatus, now = now)
        .put("regions", faceRegions(bitmap, faces, analyses))
        .put(
            "signals",
            JSONObject()
                .put("face_count", faces.size)
                .put("image_width", bitmap.width)
                .put("image_height", bitmap.height)
                .put("largest_face_area_ratio", primaryAnalysis?.areaRatio ?: 0.0)
                .put("eyes_open_probability_min", eyeOpenMin ?: JSONObject.NULL)
                .put("closed_eyes", closedEyes)
                .put("face_shadow_ratio", shadowRatio)
                .put("face_highlight_ratio", highlightRatio)
                .put("face_color_cast_strength", colorCastStrength),
        )
        .put("summary", faceAssessmentSummary(faces.size, closedEyes, shadowRatio, highlightRatio, colorCastStrength, policy))
}

private fun skippedAssessment(
    projectId: String,
    asset: ProjectAsset,
    summary: String,
    now: Long,
): JSONObject =
    baseAssessment(projectId, asset, status = "skipped", gateStatus = "unsupported", now = now)
        .put("regions", JSONArray())
        .put(
            "signals",
            JSONObject()
                .put("face_count", 0)
                .put("closed_eyes", JSONObject.NULL)
                .put("face_shadow_ratio", JSONObject.NULL)
                .put("face_highlight_ratio", JSONObject.NULL)
                .put("face_color_cast_strength", JSONObject.NULL),
        )
        .put("summary", summary)

private fun baseAssessment(
    projectId: String,
    asset: ProjectAsset,
    status: String,
    gateStatus: String,
    now: Long,
): JSONObject =
    JSONObject()
        .put("assessment_id", "subject:face:$projectId:${asset.id}")
        .put("project_id", projectId)
        .put("asset_group_id", asset.id)
        .put("subject_type", "face")
        .put("detector_kind", "android_mlkit")
        .put("detector_version", "mlkit-face-16.1.7")
        .put("status", status)
        .put("gate_status", gateStatus)
        .put("created_at_ms", now)
        .put("updated_at_ms", now)

private fun faceRegions(
    bitmap: Bitmap,
    faces: List<Face>,
    analyses: List<FaceRegionAnalysis>,
): JSONArray {
    val regions = JSONArray()
    faces.forEachIndexed { index, face ->
        val rect = face.boundingBox.clampedTo(bitmap)
        val analysis = analyses.getOrNull(index)
        regions.put(
            JSONObject()
                .put("kind", "face")
                .put("x", rect.left)
                .put("y", rect.top)
                .put("width", rect.width())
                .put("height", rect.height())
                .put("area_ratio", analysis?.areaRatio ?: 0.0)
                .put("tracking_id", face.trackingId ?: JSONObject.NULL)
                .put("left_eye_open_probability", probabilityOrNull(face.leftEyeOpenProbability))
                .put("right_eye_open_probability", probabilityOrNull(face.rightEyeOpenProbability)),
        )
    }
    return regions
}

private fun faceEyeOpenMin(face: Face): Double? =
    listOfNotNull(
        probabilityDouble(face.leftEyeOpenProbability),
        probabilityDouble(face.rightEyeOpenProbability),
    ).minOrNull()

private fun probabilityOrNull(value: Float?): Any =
    probabilityDouble(value) ?: JSONObject.NULL

private fun probabilityDouble(value: Float?): Double? =
    value?.takeIf { it >= 0f }?.toDouble()

private fun faceAssessmentSummary(
    faceCount: Int,
    closedEyes: Boolean,
    shadowRatio: Double,
    highlightRatio: Double,
    colorCastStrength: Double,
    policy: SubjectAssessmentPolicy,
): String =
    when {
        faceCount <= 0 -> "未检测到人脸。"
        closedEyes -> "检测到人脸，存在闭眼风险。"
        shadowRatio >= policy.faceExposureWarnRatio -> "检测到人脸，面部暗部死黑明显。"
        highlightRatio >= policy.faceExposureWarnRatio -> "检测到人脸，面部高光过曝明显。"
        colorCastStrength >= policy.faceColorCastWarnThreshold -> "检测到人脸，面部偏色明显。"
        else -> "检测到人脸，眼睛状态和面部曝光可用。"
    }

private data class FaceRegionAnalysis(
    val areaRatio: Double,
    val shadowRatio: Double,
    val highlightRatio: Double,
    val colorCastStrength: Double,
)

private fun faceRegionAnalysis(
    bitmap: Bitmap,
    bounds: Rect,
    policy: SubjectAssessmentPolicy,
): FaceRegionAnalysis {
    val rect = bounds.clampedTo(bitmap)
    if (rect.isEmpty) {
        return FaceRegionAnalysis(
            areaRatio = 0.0,
            shadowRatio = 0.0,
            highlightRatio = 0.0,
            colorCastStrength = 0.0,
        )
    }
    val sampleStride = sampleStride(rect.width(), rect.height())
    var samples = 0
    var shadowPixels = 0
    var highlightPixels = 0
    var redSum = 0.0
    var greenSum = 0.0
    var blueSum = 0.0
    var y = rect.top
    while (y < rect.bottom) {
        var x = rect.left
        while (x < rect.right) {
            val pixel = bitmap.getPixel(x, y)
            val red = pixel shr 16 and 0xff
            val green = pixel shr 8 and 0xff
            val blue = pixel and 0xff
            val luma = red * 0.2126 + green * 0.7152 + blue * 0.0722
            if (luma <= policy.shadowClipThreshold) {
                shadowPixels += 1
            }
            if (luma >= policy.highlightClipThreshold) {
                highlightPixels += 1
            }
            redSum += red
            greenSum += green
            blueSum += blue
            samples += 1
            x += sampleStride
        }
        y += sampleStride
    }
    val safeSamples = samples.coerceAtLeast(1)
    val redMean = redSum / safeSamples
    val greenMean = greenSum / safeSamples
    val blueMean = blueSum / safeSamples
    val mean = (redMean + greenMean + blueMean) / 3.0
    val colorCastStrength = if (mean <= 1.0) {
        0.0
    } else {
        max(abs(redMean - mean), max(abs(greenMean - mean), abs(blueMean - mean))) / mean
    }
    return FaceRegionAnalysis(
        areaRatio = rect.areaRatio(bitmap),
        shadowRatio = shadowPixels.toDouble() / safeSamples.toDouble(),
        highlightRatio = highlightPixels.toDouble() / safeSamples.toDouble(),
        colorCastStrength = colorCastStrength,
    )
}

private fun Rect.clampedTo(bitmap: Bitmap): Rect =
    run {
        val clampedLeft = left.coerceIn(0, bitmap.width)
        val clampedTop = top.coerceIn(0, bitmap.height)
        val clampedRight = right.coerceIn(0, bitmap.width)
        val clampedBottom = bottom.coerceIn(0, bitmap.height)
        Rect(
            min(clampedLeft, clampedRight),
            min(clampedTop, clampedBottom),
            max(clampedLeft, clampedRight),
            max(clampedTop, clampedBottom),
        )
    }

private fun Rect.areaRatio(bitmap: Bitmap): Double {
    val imageArea = bitmap.width.toDouble() * bitmap.height.toDouble()
    if (imageArea <= 0.0) {
        return 0.0
    }
    return width().coerceAtLeast(0).toDouble() * height().coerceAtLeast(0).toDouble() / imageArea
}

private fun sampleStride(width: Int, height: Int): Int {
    val area = width.coerceAtLeast(1).toDouble() * height.coerceAtLeast(1).toDouble()
    return sqrt(area / FACE_REGION_MAX_SAMPLES.toDouble()).roundToInt().coerceAtLeast(1)
}

private const val MIN_FACE_SIZE_RATIO = 0.08f
private const val FACE_DETECTION_TIMEOUT_SECONDS = 15L
private const val FACE_REGION_MAX_SAMPLES = 4_000

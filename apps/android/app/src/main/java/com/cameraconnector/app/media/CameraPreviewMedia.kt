package com.cameraconnector.app.media

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Matrix
import android.net.Uri
import android.util.LruCache
import androidx.exifinterface.media.ExifInterface
import java.io.File
import java.io.InputStream
import java.util.Locale

enum class PreviewQuality {
    Thumbnail,
    Detail,
    FullScreen,
}

data class PhotoMetadata(
    val shotTime: String? = null,
    val camera: String? = null,
    val lens: String? = null,
    val iso: String? = null,
    val aperture: String? = null,
    val shutter: String? = null,
    val focalLength: String? = null,
    val exposureBias: String? = null,
    val dimensions: String? = null,
    val whiteBalance: String? = null,
    val flash: String? = null,
    val colorSpace: String? = null,
    val orientation: String? = null,
) {
    fun lines(): List<Pair<String, String>> = listOfNotNull(
        shotTime?.let { "拍摄时间" to it },
        camera?.let { "相机" to it },
        lens?.let { "镜头" to it },
        iso?.let { "ISO" to it },
        aperture?.let { "光圈" to it },
        shutter?.let { "快门" to it },
        focalLength?.let { "焦距" to it },
        exposureBias?.let { "曝光补偿" to it },
        dimensions?.let { "像素尺寸" to it },
        whiteBalance?.let { "白平衡" to it },
        flash?.let { "闪光灯" to it },
        colorSpace?.let { "色彩空间" to it },
        orientation?.let { "方向" to it },
    )
}

internal const val PREVIEW_DETAIL_FALLBACK_ASPECT_RATIO = 3f / 2f

private object ThumbnailPreviewMemoryCache {
    private val maxSizeBytes = (Runtime.getRuntime().maxMemory() / 8)
        .coerceIn(THUMBNAIL_MEMORY_CACHE_MIN_BYTES.toLong(), THUMBNAIL_MEMORY_CACHE_MAX_BYTES.toLong())
        .toInt()
    private val cache = object : LruCache<String, Bitmap>(maxSizeBytes) {
        override fun sizeOf(key: String, value: Bitmap): Int =
            value.allocationByteCount.takeIf { it > 0 } ?: value.byteCount
    }

    fun get(key: String): Bitmap? = synchronized(cache) {
        cache.get(key)?.takeUnless { it.isRecycled }
    }

    fun put(key: String, bitmap: Bitmap) {
        if (bitmap.isRecycled) {
            return
        }
        synchronized(cache) {
            cache.put(key, bitmap)
        }
    }
}

fun cachedThumbnailPreview(location: String?): Bitmap? =
    location?.let { ThumbnailPreviewMemoryCache.get(thumbnailPreviewCacheKey(it)) }

fun cacheThumbnailPreview(location: String?, bitmap: Bitmap) {
    location ?: return
    ThumbnailPreviewMemoryCache.put(thumbnailPreviewCacheKey(location), bitmap)
}

fun loadPreviewBitmap(
    context: Context,
    location: String?,
    quality: PreviewQuality,
): Bitmap? {
    if (location.isNullOrBlank()) {
        return null
    }
    return runCatching {
        if (location.startsWith("content://")) {
            val uri = Uri.parse(location)
            loadCameraPreviewBitmap(
                isRawPreview = isRawPreviewLocation(location),
                isJpegPreview = isJpegPreviewLocation(location),
                quality = quality,
            ) { context.contentResolver.openInputStream(uri) }
        } else {
            loadCameraPreviewBitmap(
                isRawPreview = isRawPreviewLocation(location),
                isJpegPreview = isJpegPreviewLocation(location),
                quality = quality,
            ) { File(location).inputStream() }
        }
    }.getOrNull()
}

fun loadPhotoMetadata(context: Context, location: String?): PhotoMetadata? {
    if (location.isNullOrBlank()) {
        return null
    }
    return runCatching {
        readExifInterface(context, location) { exif ->
            PhotoMetadata(
                shotTime = formatExifDateTime(
                    exif.getAttribute(ExifInterface.TAG_DATETIME_ORIGINAL)
                        ?: exif.getAttribute(ExifInterface.TAG_DATETIME),
                ),
                camera = formatCameraName(
                    make = exif.getAttribute(ExifInterface.TAG_MAKE),
                    model = exif.getAttribute(ExifInterface.TAG_MODEL),
                ),
                lens = exif.getAttribute(ExifInterface.TAG_LENS_MODEL)
                    ?: exif.getAttribute(ExifInterface.TAG_LENS_MAKE),
                iso = readIso(exif)?.let { "ISO $it" },
                aperture = readDoubleAttribute(exif, ExifInterface.TAG_F_NUMBER)?.let {
                    "f/${formatDecimal(it, 1)}"
                },
                shutter = readDoubleAttribute(exif, ExifInterface.TAG_EXPOSURE_TIME)?.let(::formatShutterSpeed),
                focalLength = readDoubleAttribute(exif, ExifInterface.TAG_FOCAL_LENGTH)?.let {
                    val focal35mm = exif.getAttribute(ExifInterface.TAG_FOCAL_LENGTH_IN_35MM_FILM)
                    val focalText = "${formatDecimal(it, 1)} mm"
                    if (focal35mm.isNullOrBlank()) {
                        focalText
                    } else {
                        "$focalText（等效 ${focal35mm} mm）"
                    }
                },
                exposureBias = readSignedDoubleAttribute(exif, ExifInterface.TAG_EXPOSURE_BIAS_VALUE)?.let {
                    "${formatSignedDecimal(it, 1)} EV"
                },
                dimensions = formatPixelDimensions(exif),
                whiteBalance = formatWhiteBalance(exif.getAttributeInt(ExifInterface.TAG_WHITE_BALANCE, -1)),
                flash = formatFlash(exif.getAttributeInt(ExifInterface.TAG_FLASH, -1)),
                colorSpace = formatColorSpace(exif.getAttributeInt(ExifInterface.TAG_COLOR_SPACE, -1)),
                orientation = formatOrientation(
                    exif.getAttributeInt(
                        ExifInterface.TAG_ORIENTATION,
                        ExifInterface.ORIENTATION_UNDEFINED,
                    ),
                ),
            )
        }
    }.getOrNull()?.takeIf { it.lines().isNotEmpty() }
}

internal fun isDecodablePreviewLocation(location: String?): Boolean {
    val normalized = location.orEmpty().substringBefore('?').lowercase()
    return isMediaStoreImageUri(normalized) ||
        normalized.endsWith(".jpg") ||
        normalized.endsWith(".jpeg") ||
        normalized.endsWith(".png") ||
        normalized.endsWith(".webp") ||
        normalized.endsWith(".heic") ||
        normalized.endsWith(".heif") ||
        normalized.endsWith(".nef") ||
        normalized.endsWith(".nrw") ||
        normalized.endsWith(".cr2") ||
        normalized.endsWith(".cr3") ||
        normalized.endsWith(".arw") ||
        normalized.endsWith(".raf") ||
        normalized.endsWith(".rw2") ||
        normalized.endsWith(".orf") ||
        normalized.endsWith(".pef") ||
        normalized.endsWith(".dng")
}

private fun thumbnailPreviewCacheKey(location: String): String =
    "${PreviewQuality.Thumbnail.name}:$location"

private fun <T> readExifInterface(context: Context, location: String, block: (ExifInterface) -> T): T? {
    return if (location.startsWith("content://")) {
        val uri = Uri.parse(location)
        context.contentResolver.openInputStream(uri)?.use { stream ->
            block(ExifInterface(stream))
        }
    } else {
        block(ExifInterface(File(location).absolutePath))
    }
}

private fun readIso(exif: ExifInterface): String? =
    exif.getAttribute(ExifInterface.TAG_PHOTOGRAPHIC_SENSITIVITY)
        ?: exif.getAttribute(ExifInterface.TAG_ISO_SPEED_RATINGS)
        ?: exif.getAttribute(ExifInterface.TAG_ISO_SPEED)

private fun readDoubleAttribute(exif: ExifInterface, tag: String): Double? {
    val value = exif.getAttributeDouble(tag, Double.NaN)
    return value.takeUnless { it.isNaN() || it <= 0.0 }
}

private fun readSignedDoubleAttribute(exif: ExifInterface, tag: String): Double? {
    val value = exif.getAttributeDouble(tag, Double.NaN)
    return value.takeUnless { it.isNaN() }
}

private fun formatCameraName(make: String?, model: String?): String? {
    val cleanedMake = make?.trim().orEmpty()
    val cleanedModel = model?.trim().orEmpty()
    return when {
        cleanedMake.isBlank() && cleanedModel.isBlank() -> null
        cleanedMake.isBlank() -> cleanedModel
        cleanedModel.isBlank() -> cleanedMake
        cleanedModel.startsWith(cleanedMake, ignoreCase = true) -> cleanedModel
        else -> "$cleanedMake $cleanedModel"
    }
}

private fun formatExifDateTime(value: String?): String? {
    if (value.isNullOrBlank()) {
        return null
    }
    return value.replaceFirst(Regex("""^(\d{4}):(\d{2}):(\d{2})"""), "$1-$2-$3")
}

private fun formatShutterSpeed(seconds: Double): String =
    if (seconds >= 1.0) {
        "${formatDecimal(seconds, 1)} s"
    } else {
        val denominator = (1.0 / seconds).toInt()
        "1/$denominator s"
    }

private fun formatPixelDimensions(exif: ExifInterface): String? {
    val width = exif.getAttributeInt(ExifInterface.TAG_PIXEL_X_DIMENSION, 0)
        .takeIf { it > 0 }
        ?: exif.getAttributeInt(ExifInterface.TAG_IMAGE_WIDTH, 0).takeIf { it > 0 }
    val height = exif.getAttributeInt(ExifInterface.TAG_PIXEL_Y_DIMENSION, 0)
        .takeIf { it > 0 }
        ?: exif.getAttributeInt(ExifInterface.TAG_IMAGE_LENGTH, 0).takeIf { it > 0 }
    return if (width != null && height != null) {
        "$width × $height"
    } else {
        null
    }
}

private fun formatWhiteBalance(value: Int): String? = when (value) {
    0 -> "自动"
    1 -> "手动"
    else -> null
}

private fun formatFlash(value: Int): String? = when {
    value < 0 -> null
    value and 0x1 == 0x1 -> "已闪光"
    else -> "未闪光"
}

private fun formatColorSpace(value: Int): String? = when (value) {
    1 -> "sRGB"
    0xffff -> "未校准"
    else -> null
}

private fun formatOrientation(value: Int): String? = when (value) {
    ExifInterface.ORIENTATION_NORMAL -> "正常"
    ExifInterface.ORIENTATION_ROTATE_90 -> "旋转 90°"
    ExifInterface.ORIENTATION_ROTATE_180 -> "旋转 180°"
    ExifInterface.ORIENTATION_ROTATE_270 -> "旋转 270°"
    ExifInterface.ORIENTATION_FLIP_HORIZONTAL -> "水平翻转"
    ExifInterface.ORIENTATION_FLIP_VERTICAL -> "垂直翻转"
    ExifInterface.ORIENTATION_TRANSPOSE -> "转置"
    ExifInterface.ORIENTATION_TRANSVERSE -> "横向转置"
    else -> null
}

private fun formatDecimal(value: Double, digits: Int): String =
    String.format(Locale.US, "%.${digits}f", value).trimEnd('0').trimEnd('.')

private fun formatSignedDecimal(value: Double, digits: Int): String {
    val prefix = if (value > 0.0) "+" else ""
    return "$prefix${formatDecimal(value, digits)}"
}

private fun loadCameraPreviewBitmap(
    isRawPreview: Boolean,
    isJpegPreview: Boolean,
    quality: PreviewQuality,
    openStream: () -> InputStream?,
): Bitmap? {
    val orientation = readExifOrientation(
        isRawPreview = isRawPreview,
        openStream = openStream,
    )
    if (quality != PreviewQuality.Thumbnail) {
        val maxDimensionPx = when (quality) {
            PreviewQuality.Detail -> PREVIEW_DETAIL_MAX_DIMENSION_PX
            PreviewQuality.FullScreen -> PREVIEW_FULLSCREEN_MAX_DIMENSION_PX
            PreviewQuality.Thumbnail -> PREVIEW_MAX_DIMENSION_PX
        }
        return (if (isJpegPreview && !isRawPreview) {
            decodeFullBitmap(
                openStream = openStream,
                orientation = orientation,
            )
        } else if (isRawPreview) {
            decodeLargestEmbeddedJpeg(
                openStream = openStream,
                maxDimensionPx = maxDimensionPx,
                orientation = orientation,
            )
        } else {
            null
        })
            ?: decodeSampledBitmap(
                maxDimensionPx = maxDimensionPx,
                openStream = openStream,
                orientation = orientation,
                preferredConfig = Bitmap.Config.ARGB_8888,
            )
            ?: loadExifThumbnail(
                openStream = openStream,
                orientation = orientation,
            )
    }
    return loadExifThumbnail(
        openStream = openStream,
        orientation = orientation,
    )
        ?: decodeSampledBitmap(
            maxDimensionPx = PREVIEW_MAX_DIMENSION_PX,
            openStream = openStream,
            orientation = orientation,
            preferredConfig = Bitmap.Config.RGB_565,
        )
}

private fun readExifOrientation(isRawPreview: Boolean, openStream: () -> InputStream?): Int {
    val exifOrientation = runCatching {
        openStream()?.use { stream ->
            ExifInterface(stream).getAttributeInt(
                ExifInterface.TAG_ORIENTATION,
                ExifInterface.ORIENTATION_NORMAL,
            )
        } ?: ExifInterface.ORIENTATION_NORMAL
    }.getOrDefault(ExifInterface.ORIENTATION_NORMAL)
    if (!isRawPreview) {
        return exifOrientation
    }
    if (exifOrientation in ExifInterface.ORIENTATION_FLIP_HORIZONTAL..ExifInterface.ORIENTATION_ROTATE_270) {
        return exifOrientation
    }
    return readRawTiffOrientation(openStream) ?: ExifInterface.ORIENTATION_NORMAL
}

private fun readRawTiffOrientation(openStream: () -> InputStream?): Int? {
    return runCatching {
        val bytes = ByteArray(RAW_ORIENTATION_READ_LIMIT_BYTES)
        val size = openStream()?.use { stream ->
            var total = 0
            while (total < bytes.size) {
                val read = stream.read(bytes, total, bytes.size - total)
                if (read <= 0) {
                    break
                }
                total += read
            }
            total
        } ?: return@runCatching null
        parseTiffOrientation(bytes, size)
    }.getOrNull()
}

private fun parseTiffOrientation(bytes: ByteArray, size: Int): Int? {
    val tiffOffset = findTiffHeaderOffset(bytes, size) ?: return null
    val littleEndian = when {
        bytes[tiffOffset] == 'I'.code.toByte() && bytes[tiffOffset + 1] == 'I'.code.toByte() -> true
        bytes[tiffOffset] == 'M'.code.toByte() && bytes[tiffOffset + 1] == 'M'.code.toByte() -> false
        else -> return null
    }
    if (readUnsignedShort(bytes, tiffOffset + 2, littleEndian) != TIFF_MAGIC) {
        return null
    }
    val ifdOffset = readUnsignedInt(bytes, tiffOffset + 4, littleEndian)
    if (ifdOffset <= 0 || ifdOffset > Int.MAX_VALUE - tiffOffset) {
        return null
    }
    val ifdStart = tiffOffset + ifdOffset.toInt()
    if (ifdStart + 2 > size) {
        return null
    }
    val entryCount = readUnsignedShort(bytes, ifdStart, littleEndian)
    for (index in 0 until entryCount) {
        val entryOffset = ifdStart + 2 + index * TIFF_IFD_ENTRY_BYTES
        if (entryOffset + TIFF_IFD_ENTRY_BYTES > size) {
            return null
        }
        val tag = readUnsignedShort(bytes, entryOffset, littleEndian)
        if (tag == TIFF_ORIENTATION_TAG) {
            val type = readUnsignedShort(bytes, entryOffset + 2, littleEndian)
            val count = readUnsignedInt(bytes, entryOffset + 4, littleEndian)
            if (type != TIFF_SHORT_TYPE || count < 1) {
                return null
            }
            val orientation = readUnsignedShort(bytes, entryOffset + 8, littleEndian)
            return orientation.takeIf { it in ExifInterface.ORIENTATION_NORMAL..ExifInterface.ORIENTATION_ROTATE_270 }
        }
    }
    return null
}

private fun findTiffHeaderOffset(bytes: ByteArray, size: Int): Int? {
    if (size < TIFF_HEADER_BYTES) {
        return null
    }
    val limit = minOf(size - TIFF_HEADER_BYTES, TIFF_HEADER_SCAN_LIMIT_BYTES)
    for (offset in 0..limit) {
        val hasEndianMarker =
            (bytes[offset] == 'I'.code.toByte() && bytes[offset + 1] == 'I'.code.toByte()) ||
                (bytes[offset] == 'M'.code.toByte() && bytes[offset + 1] == 'M'.code.toByte())
        if (hasEndianMarker) {
            val littleEndian = bytes[offset] == 'I'.code.toByte()
            if (readUnsignedShort(bytes, offset + 2, littleEndian) == TIFF_MAGIC) {
                return offset
            }
        }
    }
    return null
}

private fun readUnsignedShort(bytes: ByteArray, offset: Int, littleEndian: Boolean): Int {
    val first = bytes[offset].toInt() and 0xff
    val second = bytes[offset + 1].toInt() and 0xff
    return if (littleEndian) {
        first or (second shl 8)
    } else {
        (first shl 8) or second
    }
}

private fun readUnsignedInt(bytes: ByteArray, offset: Int, littleEndian: Boolean): Long {
    val b0 = bytes[offset].toLong() and 0xff
    val b1 = bytes[offset + 1].toLong() and 0xff
    val b2 = bytes[offset + 2].toLong() and 0xff
    val b3 = bytes[offset + 3].toLong() and 0xff
    return if (littleEndian) {
        b0 or (b1 shl 8) or (b2 shl 16) or (b3 shl 24)
    } else {
        (b0 shl 24) or (b1 shl 16) or (b2 shl 8) or b3
    }
}

private fun decodeFullBitmap(
    openStream: () -> InputStream?,
    orientation: Int,
): Bitmap? {
    return runCatching {
        val decodeOptions = BitmapFactory.Options().apply {
            inPreferredConfig = Bitmap.Config.ARGB_8888
        }
        val bitmap = openStream()?.use { stream ->
            BitmapFactory.decodeStream(stream, null, decodeOptions)
        }
        applyExifOrientation(bitmap, orientation)
    }.getOrNull()
}

private fun loadExifThumbnail(openStream: () -> InputStream?, orientation: Int): Bitmap? {
    return runCatching {
        openStream()?.use { stream ->
            applyExifOrientation(
                bitmap = ExifInterface(stream).thumbnailBitmap,
                orientation = orientation,
            )
        }
    }.getOrNull()
}

private fun decodeLargestEmbeddedJpeg(
    openStream: () -> InputStream?,
    maxDimensionPx: Int,
    orientation: Int,
): Bitmap? {
    return runCatching {
        val bytes = openStream()?.use { stream -> stream.readBytes() } ?: return@runCatching null
        findEmbeddedJpegRanges(bytes)
            .sortedByDescending { range -> range.last - range.first }
            .firstNotNullOfOrNull { range ->
                decodeSampledJpegBytes(
                    bytes = bytes,
                    offset = range.first,
                    length = range.last - range.first + 1,
                    maxDimensionPx = maxDimensionPx,
                    orientation = orientation,
                )
            }
    }.getOrNull()
}

private fun findEmbeddedJpegRanges(bytes: ByteArray): List<IntRange> {
    val ranges = mutableListOf<IntRange>()
    var cursor = 0
    while (cursor < bytes.size - JPEG_SOI_BYTES) {
        val start = findJpegStart(bytes, cursor) ?: break
        val end = findJpegEnd(bytes, start + JPEG_SOI_BYTES) ?: break
        ranges += start..end
        cursor = end + 1
    }
    return ranges
}

private fun findJpegStart(bytes: ByteArray, fromIndex: Int): Int? {
    var index = fromIndex
    while (index < bytes.size - JPEG_SOI_BYTES) {
        if (
            (bytes[index].toInt() and 0xff) == 0xff &&
            (bytes[index + 1].toInt() and 0xff) == 0xd8 &&
            (bytes[index + 2].toInt() and 0xff) == 0xff
        ) {
            return index
        }
        index += 1
    }
    return null
}

private fun findJpegEnd(bytes: ByteArray, fromIndex: Int): Int? {
    var index = fromIndex
    while (index < bytes.size - 1) {
        if ((bytes[index].toInt() and 0xff) == 0xff && (bytes[index + 1].toInt() and 0xff) == 0xd9) {
            return index + 1
        }
        index += 1
    }
    return null
}

private fun decodeSampledJpegBytes(
    bytes: ByteArray,
    offset: Int,
    length: Int,
    maxDimensionPx: Int,
    orientation: Int,
): Bitmap? {
    val bounds = BitmapFactory.Options().apply {
        inJustDecodeBounds = true
    }
    BitmapFactory.decodeByteArray(bytes, offset, length, bounds)
    if (bounds.outWidth <= 0 || bounds.outHeight <= 0) {
        return null
    }
    val decodeOptions = BitmapFactory.Options().apply {
        inSampleSize = calculateBitmapSampleSize(
            width = bounds.outWidth,
            height = bounds.outHeight,
            maxDimensionPx = maxDimensionPx,
        )
        inPreferredConfig = Bitmap.Config.ARGB_8888
    }
    return applyExifOrientation(
        bitmap = BitmapFactory.decodeByteArray(bytes, offset, length, decodeOptions),
        orientation = orientation,
    )
}

private fun decodeSampledBitmap(
    maxDimensionPx: Int,
    openStream: () -> InputStream?,
    orientation: Int,
    preferredConfig: Bitmap.Config,
): Bitmap? {
    return runCatching {
        decodeSampledBitmapUnsafe(
            maxDimensionPx = maxDimensionPx,
            openStream = openStream,
            orientation = orientation,
            preferredConfig = preferredConfig,
        )
    }.getOrNull()
}

private fun decodeSampledBitmapUnsafe(
    maxDimensionPx: Int,
    openStream: () -> InputStream?,
    orientation: Int,
    preferredConfig: Bitmap.Config,
): Bitmap? {
    val bounds = BitmapFactory.Options().apply {
        inJustDecodeBounds = true
    }
    openStream()?.use { stream ->
        BitmapFactory.decodeStream(stream, null, bounds)
    } ?: return null
    if (bounds.outWidth <= 0 || bounds.outHeight <= 0) {
        return null
    }

    val sampleSize = calculateBitmapSampleSize(
        width = bounds.outWidth,
        height = bounds.outHeight,
        maxDimensionPx = maxDimensionPx,
    )
    val decodeOptions = BitmapFactory.Options().apply {
        inSampleSize = sampleSize
        inPreferredConfig = preferredConfig
    }
    val bitmap = openStream()?.use { stream ->
        BitmapFactory.decodeStream(stream, null, decodeOptions)
    }
    return applyExifOrientation(bitmap, orientation)
}

private fun applyExifOrientation(bitmap: Bitmap?, orientation: Int): Bitmap? {
    bitmap ?: return null
    val matrix = Matrix()
    when (orientation) {
        ExifInterface.ORIENTATION_FLIP_HORIZONTAL -> matrix.preScale(-1f, 1f)
        ExifInterface.ORIENTATION_ROTATE_180 -> matrix.postRotate(180f)
        ExifInterface.ORIENTATION_FLIP_VERTICAL -> matrix.preScale(1f, -1f)
        ExifInterface.ORIENTATION_TRANSPOSE -> {
            matrix.preScale(-1f, 1f)
            matrix.postRotate(90f)
        }
        ExifInterface.ORIENTATION_ROTATE_90 -> matrix.postRotate(90f)
        ExifInterface.ORIENTATION_TRANSVERSE -> {
            matrix.preScale(-1f, 1f)
            matrix.postRotate(270f)
        }
        ExifInterface.ORIENTATION_ROTATE_270 -> matrix.postRotate(270f)
        else -> return bitmap
    }

    return Bitmap.createBitmap(bitmap, 0, 0, bitmap.width, bitmap.height, matrix, true)
}

private fun calculateBitmapSampleSize(width: Int, height: Int, maxDimensionPx: Int): Int {
    var sampleSize = 1
    var sampledWidth = width
    var sampledHeight = height
    while (sampledWidth / 2 >= maxDimensionPx || sampledHeight / 2 >= maxDimensionPx) {
        sampleSize *= 2
        sampledWidth /= 2
        sampledHeight /= 2
    }
    return sampleSize
}

private fun isMediaStoreImageUri(normalizedLocation: String): Boolean =
    normalizedLocation.startsWith("content://media/") &&
        normalizedLocation.contains("/images/")

internal fun isRawPreviewLocation(location: String?): Boolean {
    val normalized = location.orEmpty().substringBefore('?').lowercase()
    return normalized.endsWith(".nef") ||
        normalized.endsWith(".nrw") ||
        normalized.endsWith(".cr2") ||
        normalized.endsWith(".cr3") ||
        normalized.endsWith(".arw") ||
        normalized.endsWith(".raf") ||
        normalized.endsWith(".rw2") ||
        normalized.endsWith(".orf") ||
        normalized.endsWith(".pef") ||
        normalized.endsWith(".dng")
}

internal fun isJpegPreviewLocation(location: String?): Boolean {
    val normalized = location.orEmpty().substringBefore('?').lowercase()
    return normalized.endsWith(".jpg") || normalized.endsWith(".jpeg")
}

private const val PREVIEW_MAX_DIMENSION_PX = 512
private const val PREVIEW_DETAIL_MAX_DIMENSION_PX = 2400
private const val PREVIEW_FULLSCREEN_MAX_DIMENSION_PX = 4096
private const val THUMBNAIL_MEMORY_CACHE_MIN_BYTES = 16 * 1024 * 1024
private const val THUMBNAIL_MEMORY_CACHE_MAX_BYTES = 64 * 1024 * 1024
private const val RAW_ORIENTATION_READ_LIMIT_BYTES = 512 * 1024
private const val TIFF_HEADER_SCAN_LIMIT_BYTES = 4096
private const val TIFF_HEADER_BYTES = 8
private const val TIFF_MAGIC = 42
private const val TIFF_IFD_ENTRY_BYTES = 12
private const val TIFF_ORIENTATION_TAG = 0x0112
private const val TIFF_SHORT_TYPE = 3
private const val JPEG_SOI_BYTES = 3

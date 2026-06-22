package com.cameraconnector.app.media

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Matrix
import androidx.exifinterface.media.ExifInterface
import java.io.InputStream
internal fun loadCameraPreviewBitmap(
    isRawPreview: Boolean,
    isJpegPreview: Boolean,
    quality: PreviewQuality,
    localPath: String?,
    openStream: () -> InputStream?,
): Bitmap? {
    val orientation = readExifOrientation(
        isRawPreview = isRawPreview,
        openStream = openStream,
    )
    if (quality != PreviewQuality.Thumbnail) {
        val maxDimensionPx = previewDecodeMaxDimensionPx(quality)
        return (if (isRawPreview) {
            decodeLargestEmbeddedJpeg(
                openStream = openStream,
                maxDimensionPx = maxDimensionPx,
                orientation = orientation,
            )
        } else {
            decodeSampledBitmapFile(
                path = localPath,
                maxDimensionPx = maxDimensionPx,
                orientation = orientation,
                preferredConfig = Bitmap.Config.ARGB_8888,
            ) ?: decodeSampledBitmap(
                maxDimensionPx = maxDimensionPx,
                openStream = openStream,
                orientation = orientation,
                preferredConfig = Bitmap.Config.ARGB_8888,
            ) ?: decodeSampledBitmapBytes(
                maxDimensionPx = maxDimensionPx,
                openStream = openStream,
                orientation = orientation,
                preferredConfig = Bitmap.Config.ARGB_8888,
            )
        })
            ?: loadExifThumbnail(
                openStream = openStream,
                orientation = orientation,
            )
    }
    if (isJpegPreview && !isRawPreview) {
        return loadExifThumbnail(
            openStream = openStream,
            orientation = orientation,
        )
            ?: decodeSampledBitmapFile(
                path = localPath,
                maxDimensionPx = PREVIEW_MAX_DIMENSION_PX,
                orientation = orientation,
                preferredConfig = Bitmap.Config.RGB_565,
            )
            ?: decodeSampledBitmap(
                maxDimensionPx = PREVIEW_MAX_DIMENSION_PX,
                openStream = openStream,
                orientation = orientation,
                preferredConfig = Bitmap.Config.RGB_565,
            )
            ?: decodeSampledBitmapBytes(
                maxDimensionPx = PREVIEW_MAX_DIMENSION_PX,
                openStream = openStream,
                orientation = orientation,
                preferredConfig = Bitmap.Config.RGB_565,
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
    preferredConfig: Bitmap.Config = Bitmap.Config.ARGB_8888,
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
        inPreferredConfig = preferredConfig
    }
    return applyExifOrientation(
        bitmap = BitmapFactory.decodeByteArray(bytes, offset, length, decodeOptions),
        orientation = orientation,
    )
}

private fun decodeSampledBitmapFile(
    path: String?,
    maxDimensionPx: Int,
    orientation: Int,
    preferredConfig: Bitmap.Config,
): Bitmap? {
    if (path.isNullOrBlank()) {
        return null
    }
    return runCatching {
        val bounds = BitmapFactory.Options().apply {
            inJustDecodeBounds = true
        }
        BitmapFactory.decodeFile(path, bounds)
        if (bounds.outWidth <= 0 || bounds.outHeight <= 0) {
            return@runCatching null
        }
        val decodeOptions = BitmapFactory.Options().apply {
            inSampleSize = calculateBitmapSampleSize(
                width = bounds.outWidth,
                height = bounds.outHeight,
                maxDimensionPx = maxDimensionPx,
            )
            inPreferredConfig = preferredConfig
        }
        applyExifOrientation(
            bitmap = BitmapFactory.decodeFile(path, decodeOptions),
            orientation = orientation,
        )
    }.getOrNull()
}

private fun decodeSampledBitmapBytes(
    maxDimensionPx: Int,
    openStream: () -> InputStream?,
    orientation: Int,
    preferredConfig: Bitmap.Config,
): Bitmap? {
    return runCatching {
        val bytes = openStream()?.use { stream -> stream.readBytes() } ?: return@runCatching null
        decodeSampledJpegBytes(
            bytes = bytes,
            offset = 0,
            length = bytes.size,
            maxDimensionPx = maxDimensionPx,
            orientation = orientation,
            preferredConfig = preferredConfig,
        )
    }.getOrNull()
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


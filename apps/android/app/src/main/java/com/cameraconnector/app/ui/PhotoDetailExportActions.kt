package com.cameraconnector.app.ui

import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.PhotoLibrary
import androidx.compose.material.icons.outlined.Share
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.core.content.FileProvider
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.media.PreviewQuality
import com.cameraconnector.app.media.isDecodablePreviewLocation
import com.cameraconnector.app.media.loadCachedPreviewBitmap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File

@Composable
internal fun PhotoDetailExportDialog(
    exportUi: PhotoDetailExportUi,
    onDismiss: () -> Unit,
    onSave: () -> Unit,
    onShare: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("\u7167\u7247\u64cd\u4f5c") },
        text = {
            Text(
                text = exportUi.fileName,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        },
        confirmButton = {
            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                TextButton(
                    enabled = exportUi.enabled,
                    onClick = onSave,
                ) {
                    Icon(
                        imageVector = Icons.Outlined.PhotoLibrary,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                    )
                    Spacer(Modifier.width(4.dp))
                    Text("\u4fdd\u5b58")
                }
                TextButton(
                    enabled = exportUi.enabled,
                    onClick = onShare,
                ) {
                    Icon(
                        imageVector = Icons.Outlined.Share,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                    )
                    Spacer(Modifier.width(4.dp))
                    Text("\u5206\u4eab")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("\u53d6\u6d88")
            }
        },
    )
}

internal suspend fun savePhotoDetailExportToGallery(
    context: Context,
    asset: ProjectAsset,
    exportUi: PhotoDetailExportUi,
): Boolean =
    withContext(Dispatchers.IO) {
        val bitmap = photoDetailExportBitmap(context, asset) ?: return@withContext false
        val resolver = context.contentResolver
        val values = ContentValues().apply {
            put(MediaStore.Images.Media.DISPLAY_NAME, exportUi.fileName)
            put(MediaStore.Images.Media.MIME_TYPE, PHOTO_DETAIL_EXPORT_MIME_TYPE)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                put(
                    MediaStore.Images.Media.RELATIVE_PATH,
                    "${Environment.DIRECTORY_PICTURES}/CameraConnector",
                )
                put(MediaStore.Images.Media.IS_PENDING, 1)
            }
        }
        val uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values)
            ?: return@withContext false
        runCatching {
            val saved = resolver.openOutputStream(uri)?.use { output ->
                bitmap.compress(Bitmap.CompressFormat.JPEG, PHOTO_DETAIL_EXPORT_JPEG_QUALITY, output)
            } == true
            if (!saved) {
                resolver.delete(uri, null, null)
                return@withContext false
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val publishValues = ContentValues().apply {
                    put(MediaStore.Images.Media.IS_PENDING, 0)
                }
                resolver.update(uri, publishValues, null, null)
            }
            true
        }.getOrElse {
            resolver.delete(uri, null, null)
            false
        }
    }

internal suspend fun sharePhotoDetailExport(
    context: Context,
    asset: ProjectAsset,
    exportUi: PhotoDetailExportUi,
): Boolean {
    val exportFile = writePhotoDetailExportCacheFile(context, asset, exportUi)
        ?: return false
    val uri = FileProvider.getUriForFile(
        context,
        "${context.packageName}.fileprovider",
        exportFile,
    )
    val shareIntent = Intent(Intent.ACTION_SEND).apply {
        type = PHOTO_DETAIL_EXPORT_MIME_TYPE
        putExtra(Intent.EXTRA_STREAM, uri)
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    }
    val chooser = Intent.createChooser(shareIntent, "\u5206\u4eab\u7167\u7247").apply {
        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        if (context.findActivity() == null) {
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
    }
    return runCatching {
        context.startActivity(chooser)
        true
    }.getOrDefault(false)
}

private suspend fun writePhotoDetailExportCacheFile(
    context: Context,
    asset: ProjectAsset,
    exportUi: PhotoDetailExportUi,
): File? =
    withContext(Dispatchers.IO) {
        val bitmap = photoDetailExportBitmap(context, asset) ?: return@withContext null
        val exportDirectory = File(context.cacheDir, PHOTO_DETAIL_EXPORT_CACHE_DIRECTORY)
        runCatching {
            exportDirectory.mkdirs()
            val exportFile = File(exportDirectory, exportUi.fileName)
            exportFile.outputStream().use { output ->
                check(bitmap.compress(Bitmap.CompressFormat.JPEG, PHOTO_DETAIL_EXPORT_JPEG_QUALITY, output))
            }
            exportFile
        }.getOrNull()
    }

private suspend fun photoDetailExportBitmap(
    context: Context,
    asset: ProjectAsset,
): Bitmap? =
    withContext(Dispatchers.IO) {
        val previewLocation = asset.previewLocation
            ?.takeIf(::isDecodablePreviewLocation)
            ?: return@withContext null
        loadCachedPreviewBitmap(context, previewLocation, PreviewQuality.FullScreen)
            ?: loadCachedPreviewBitmap(context, previewLocation, PreviewQuality.Detail)
            ?: loadCachedPreviewBitmap(context, previewLocation, PreviewQuality.Thumbnail)
    }

private const val PHOTO_DETAIL_EXPORT_MIME_TYPE = "image/jpeg"
private const val PHOTO_DETAIL_EXPORT_JPEG_QUALITY = 95
private const val PHOTO_DETAIL_EXPORT_CACHE_DIRECTORY = "photo_exports"

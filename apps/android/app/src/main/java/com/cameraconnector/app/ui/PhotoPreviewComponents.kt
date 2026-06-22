package com.cameraconnector.app.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.graphics.Bitmap
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.rememberTransformableState
import androidx.compose.foundation.gestures.transformable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.media.PREVIEW_DETAIL_FALLBACK_ASPECT_RATIO
import com.cameraconnector.app.media.PreviewQuality
import com.cameraconnector.app.media.cachedPreviewBitmap
import com.cameraconnector.app.media.isDecodablePreviewLocation
import com.cameraconnector.app.media.loadCachedPreviewBitmap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext


@Composable
internal fun FullScreenPhotoPreview(
    asset: ProjectAsset,
    onDismiss: () -> Unit,
    onLongPress: () -> Unit = {},
) {
    BackHandler(onBack = onDismiss)
    ImmersiveSystemBars()
    var scale by remember { mutableStateOf(1f) }
    var offsetX by remember { mutableStateOf(0f) }
    var offsetY by remember { mutableStateOf(0f) }
    val transformState = rememberTransformableState { _, zoomChange, panChange, _ ->
        val nextScale = (scale * zoomChange).coerceIn(FULLSCREEN_MIN_SCALE, FULLSCREEN_MAX_SCALE)
        if (nextScale <= FULLSCREEN_MIN_SCALE) {
            scale = FULLSCREEN_MIN_SCALE
            offsetX = 0f
            offsetY = 0f
        } else {
            scale = nextScale
            offsetX += panChange.x
            offsetY += panChange.y
        }
    }
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(
            usePlatformDefaultWidth = false,
            decorFitsSystemWindows = false,
        ),
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black)
                .pointerInput(Unit) {
                    detectTapGestures(
                        onTap = { onDismiss() },
                        onLongPress = { onLongPress() },
                        onDoubleTap = {
                            if (scale > FULLSCREEN_MIN_SCALE) {
                                scale = FULLSCREEN_MIN_SCALE
                                offsetX = 0f
                                offsetY = 0f
                            } else {
                                scale = FULLSCREEN_DOUBLE_TAP_SCALE
                            }
                        },
                    )
                }
                .transformable(transformState),
            contentAlignment = Alignment.Center,
        ) {
            PhotoPreview(
                asset = asset,
                previewQuality = PreviewQuality.FullScreen,
                contentScale = ContentScale.Fit,
                backgroundColor = Color.Black,
                clipPreview = false,
                showFallbackText = false,
                modifier = Modifier
                    .fillMaxSize()
                    .graphicsLayer {
                        scaleX = scale
                        scaleY = scale
                        translationX = offsetX
                        translationY = offsetY
                    },
            )
        }
    }
}

@Composable
internal fun ImmersiveSystemBars() {
    val view = LocalView.current
    DisposableEffect(view) {
        val window = view.context.findActivity()?.window
        if (window == null) {
            onDispose { }
        } else {
            val controller = WindowCompat.getInsetsController(window, view)
            val previousBehavior = controller.systemBarsBehavior
            WindowCompat.setDecorFitsSystemWindows(window, false)
            controller.systemBarsBehavior =
                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            controller.hide(WindowInsetsCompat.Type.systemBars())
            onDispose {
                controller.show(WindowInsetsCompat.Type.systemBars())
                controller.systemBarsBehavior = previousBehavior
                WindowCompat.setDecorFitsSystemWindows(window, true)
            }
        }
    }
}

internal fun cachedPreviewAspectRatio(previewLocation: String?): Float? =
    cachedPreviewBitmap(
        location = previewLocation.takeIf(::isDecodablePreviewLocation),
        quality = PreviewQuality.Detail,
        allowLowerQualityFallback = true,
    )?.let(::bitmapDisplayAspectRatio)

private fun bitmapDisplayAspectRatio(bitmap: Bitmap?): Float? =
    bitmap
        ?.takeIf { it.width > 0 && it.height > 0 }
        ?.let { it.width.toFloat() / it.height.toFloat() }

@OptIn(ExperimentalFoundationApi::class)
@Composable
internal fun PhotoPreview(
    asset: ProjectAsset,
    modifier: Modifier = Modifier,
    compactFallback: Boolean = false,
    previewQuality: PreviewQuality = PreviewQuality.Thumbnail,
    fitToImageAspect: Boolean = false,
    preferredAspectRatio: Float? = null,
    contentScale: ContentScale = ContentScale.Crop,
    backgroundColor: Color = ElementPanel,
    clipPreview: Boolean = true,
    trimLetterbox: Boolean = false,
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    onPreviewAspectRatio: ((Float) -> Unit)? = null,
    onPreviewReady: (() -> Unit)? = null,
    showFallbackText: Boolean = true,
) {
    val context = LocalContext.current
    val previewLocation = asset.previewLocation.takeIf(::isDecodablePreviewLocation)
    val initialBitmap = remember(previewLocation, previewQuality) {
        cachedPreviewBitmap(
            location = previewLocation,
            quality = previewQuality,
            allowLowerQualityFallback = true,
        )
    }
    var bitmap by remember(previewLocation, previewQuality) { mutableStateOf(initialBitmap) }
    LaunchedEffect(previewLocation, previewQuality) {
        if (previewLocation == null) {
            bitmap = null
            return@LaunchedEffect
        }
        val exactCached = cachedPreviewBitmap(previewLocation, previewQuality)
        if (exactCached != null) {
            bitmap = exactCached
            bitmapDisplayAspectRatio(exactCached)?.let { onPreviewAspectRatio?.invoke(it) }
            onPreviewReady?.invoke()
            return@LaunchedEffect
        }
        cachedPreviewBitmap(
            location = previewLocation,
            quality = previewQuality,
            allowLowerQualityFallback = true,
        )?.let { fallbackBitmap ->
            bitmap = fallbackBitmap
            bitmapDisplayAspectRatio(fallbackBitmap)?.let { onPreviewAspectRatio?.invoke(it) }
        }
        if (previewQuality != PreviewQuality.Thumbnail && bitmap == null) {
            withContext(Dispatchers.IO) {
                loadCachedPreviewBitmap(context, previewLocation, PreviewQuality.Thumbnail)
            }?.let { thumbnailBitmap ->
                bitmap = thumbnailBitmap
                bitmapDisplayAspectRatio(thumbnailBitmap)?.let { onPreviewAspectRatio?.invoke(it) }
            }
        }
        withContext(Dispatchers.IO) {
            loadCachedPreviewBitmap(context, previewLocation, previewQuality)
        }?.let { loadedBitmap ->
            bitmap = loadedBitmap
            bitmapDisplayAspectRatio(loadedBitmap)?.let { onPreviewAspectRatio?.invoke(it) }
            onPreviewReady?.invoke()
        }
    }

    val loadedBitmap = bitmap
    val displayBitmap = remember(loadedBitmap, trimLetterbox) {
        if (trimLetterbox && loadedBitmap != null) {
            loadedBitmap.trimNearBlackLetterbox() ?: loadedBitmap
        } else {
            loadedBitmap
        }
    }
    val showTextLoadingFallback = loadedBitmap == null &&
        (previewQuality == PreviewQuality.Thumbnail || compactFallback)
    val aspectModifier = if (fitToImageAspect) {
        val imageAspectRatio = preferredAspectRatio
            ?: bitmapDisplayAspectRatio(loadedBitmap)
            ?: PREVIEW_DETAIL_FALLBACK_ASPECT_RATIO
        modifier.aspectRatio(imageAspectRatio)
    } else {
        modifier
    }
    val previewModifier = if (clipPreview) {
        aspectModifier.clip(elementShape)
    } else {
        aspectModifier
    }
    val clickableModifier = if (onClick == null && onLongClick == null) {
        previewModifier
    } else {
        previewModifier.combinedClickable(
            onClick = onClick ?: {},
            onLongClick = onLongClick,
        )
    }
    Box(
        modifier = clickableModifier.background(backgroundColor),
        contentAlignment = Alignment.Center,
    ) {
        if (displayBitmap != null) {
            Image(
                bitmap = displayBitmap.asImageBitmap(),
                contentDescription = asset.groupTitle(),
                modifier = Modifier.fillMaxSize(),
                contentScale = contentScale,
                filterQuality = FilterQuality.High,
            )
        } else if (showFallbackText) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                if (compactFallback || !showTextLoadingFallback) {
                    Text(
                        asset.formatBadges(),
                        color = ElementInfo,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                } else {
                    Text(
                        "\u52a0\u8f7d\u4e2d",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(asset.formatBadges(), color = ElementInfo)
                }
            }
        }
    }
}

private fun Bitmap.trimNearBlackLetterbox(): Bitmap? {
    val width = width
    val height = height
    if (width < 12 || height < 12) {
        return null
    }
    fun isMostlyBlackRow(y: Int): Boolean {
        var black = 0
        for (x in 0 until width) {
            if (pixelLuma(getPixel(x, y)) <= 14) black += 1
        }
        return black >= width * 0.9f
    }
    fun isMostlyBlackColumn(x: Int): Boolean {
        var black = 0
        for (y in 0 until height) {
            if (pixelLuma(getPixel(x, y)) <= 14) black += 1
        }
        return black >= height * 0.9f
    }
    var left = 0
    var right = width - 1
    var top = 0
    var bottom = height - 1
    while (left < right && isMostlyBlackColumn(left)) left += 1
    while (right > left && isMostlyBlackColumn(right)) right -= 1
    while (top < bottom && isMostlyBlackRow(top)) top += 1
    while (bottom > top && isMostlyBlackRow(bottom)) bottom -= 1
    val cropWidth = right - left + 1
    val cropHeight = bottom - top + 1
    if (cropWidth >= width - 2 && cropHeight >= height - 2) {
        return null
    }
    if (cropWidth < width * 0.72f || cropHeight < height * 0.72f) {
        return null
    }
    return runCatching {
        Bitmap.createBitmap(this, left, top, cropWidth, cropHeight)
    }.getOrNull()
}

private fun pixelLuma(pixel: Int): Int {
    val red = pixel shr 16 and 0xff
    val green = pixel shr 8 and 0xff
    val blue = pixel and 0xff
    return ((red * 299) + (green * 587) + (blue * 114)) / 1000
}

internal tailrec fun Context.findActivity(): Activity? {
    return when (this) {
        is Activity -> this
        is ContextWrapper -> baseContext.findActivity()
        else -> null
    }
}

internal const val FULLSCREEN_MIN_SCALE = 1f
internal const val FULLSCREEN_DOUBLE_TAP_SCALE = 2.5f
internal const val FULLSCREEN_MAX_SCALE = 5f

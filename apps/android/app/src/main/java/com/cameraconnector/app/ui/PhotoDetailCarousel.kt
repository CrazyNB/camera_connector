package com.cameraconnector.app.ui

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.media.PreviewQuality
import com.cameraconnector.app.media.isDecodablePreviewLocation
import com.cameraconnector.app.media.loadCachedPreviewBitmap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlin.math.roundToInt

@Composable
internal fun DetailPhotoCarousel(
    asset: ProjectAsset,
    previousAsset: ProjectAsset?,
    nextAsset: ProjectAsset?,
    positionText: String?,
    imageAspectRatio: Float?,
    onPrevious: (() -> Unit)?,
    onNext: (() -> Unit)?,
    onClick: () -> Unit,
    onLongPress: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val assetId = asset.assetSelectionId()
    var dragOffsetX by remember(assetId) { mutableFloatStateOf(0f) }
    var settleOffsetX by remember(assetId) { mutableFloatStateOf(0f) }
    var settling by remember(assetId) { mutableStateOf(false) }
    var pendingNavigationAssetId by remember(assetId) { mutableStateOf<String?>(null) }
    fun resetMotion() {
        pendingNavigationAssetId = null
        settling = false
        dragOffsetX = 0f
        settleOffsetX = 0f
    }
    LaunchedEffect(previousAsset?.previewLocation, nextAsset?.previewLocation) {
        withContext(Dispatchers.IO) {
            listOf(previousAsset, nextAsset)
                .mapNotNull { it?.previewLocation?.takeIf(::isDecodablePreviewLocation) }
                .distinct()
                .forEach { location ->
                    loadCachedPreviewBitmap(context, location, PreviewQuality.Thumbnail)
                }
        }
    }
    var previewAspectRatio by remember(assetId) {
        mutableStateOf(cachedPreviewAspectRatio(asset.previewLocation))
    }
    LaunchedEffect(assetId, imageAspectRatio) {
        imageAspectRatio?.let { previewAspectRatio = it }
    }
    val displayAspectRatio = imageAspectRatio ?: previewAspectRatio ?: PHOTO_DETAIL_LOADING_ASPECT_RATIO
    BoxWithConstraints(
        modifier = modifier
            .height(detailCarouselHeight(displayAspectRatio))
            .clip(RoundedCornerShape(18.dp)),
    ) {
        val density = LocalDensity.current
        val pageWidthPx = with(density) { maxWidth.toPx() }
        val sidePageScale = 0.9f
        val sidePeekPx = with(density) { 24.dp.toPx() }
        val sideGapPx = with(density) { 16.dp.toPx() }
        val sidePaddingPx = with(density) { 48.dp.toPx() }
        val pageStridePx = (pageWidthPx - sidePeekPx - sidePaddingPx)
            .coerceAtLeast(pageWidthPx * 0.72f + sideGapPx)
        val thresholdPx = pageStridePx * 0.18f
        val pageOffsetX = if (settling) settleOffsetX else dragOffsetX
        val mainHorizontalPadding = 24.dp
        val sideHorizontalPadding = 48.dp
        Box(
            modifier = Modifier
                .fillMaxSize()
                .pointerInput(assetId, previousAsset?.assetSelectionId(), nextAsset?.assetSelectionId()) {
                    detectHorizontalDragGestures(
                        onDragStart = {
                            pendingNavigationAssetId = null
                            settling = false
                            settleOffsetX = 0f
                        },
                        onDragEnd = {
                            val target = when {
                                dragOffsetX > thresholdPx && onPrevious != null -> DetailNavigationDirection.Previous
                                dragOffsetX < -thresholdPx && onNext != null -> DetailNavigationDirection.Next
                                else -> null
                            }
                            val releaseOffset = dragOffsetX
                            scope.launch {
                                settling = true
                                val animation = Animatable(releaseOffset)
                                when (target) {
                                    DetailNavigationDirection.Previous -> {
                                        animation.animateTo(pageStridePx, tween(durationMillis = 90)) {
                                            settleOffsetX = value
                                        }
                                        pendingNavigationAssetId = assetId
                                        onPrevious?.invoke()
                                    }
                                    DetailNavigationDirection.Next -> {
                                        animation.animateTo(-pageStridePx, tween(durationMillis = 90)) {
                                            settleOffsetX = value
                                        }
                                        pendingNavigationAssetId = assetId
                                        onNext?.invoke()
                                    }
                                    null -> animation.animateTo(0f, tween(durationMillis = 90)) {
                                        settleOffsetX = value
                                    }
                                }
                                if (target == null) {
                                    resetMotion()
                                } else {
                                    delay(240)
                                    if (pendingNavigationAssetId == assetId) {
                                        resetMotion()
                                    }
                                }
                            }
                        },
                        onDragCancel = {
                            val releaseOffset = dragOffsetX
                            scope.launch {
                                settling = true
                                Animatable(releaseOffset).animateTo(0f, tween(durationMillis = 90)) {
                                    settleOffsetX = value
                                }
                                resetMotion()
                            }
                        },
                    ) { _, dragAmount ->
                        dragOffsetX = (dragOffsetX + dragAmount).coerceIn(
                            minimumValue = if (onNext != null) -pageStridePx else 0f,
                            maximumValue = if (onPrevious != null) pageStridePx else 0f,
                        )
                    }
                },
        ) {
            previousAsset?.let { previous ->
                DetailCarouselPhotoPage(
                    asset = previous,
                    previewQuality = PreviewQuality.Thumbnail,
                    horizontalPadding = sideHorizontalPadding,
                    pageScale = sidePageScale,
                    onClick = onPrevious,
                    modifier = Modifier
                        .fillMaxSize()
                        .offset { IntOffset((pageOffsetX - pageStridePx).roundToInt(), 0) },
                )
            }
            nextAsset?.let { next ->
                DetailCarouselPhotoPage(
                    asset = next,
                    previewQuality = PreviewQuality.Thumbnail,
                    horizontalPadding = sideHorizontalPadding,
                    pageScale = sidePageScale,
                    onClick = onNext,
                    modifier = Modifier
                        .fillMaxSize()
                        .offset { IntOffset((pageOffsetX + pageStridePx).roundToInt(), 0) },
                )
            }
            DetailCarouselPhotoPage(
                asset = asset,
                previewQuality = PreviewQuality.Detail,
                horizontalPadding = mainHorizontalPadding,
                preferredAspectRatio = displayAspectRatio,
                onPreviewAspectRatio = { aspect ->
                    if (imageAspectRatio == null) {
                        previewAspectRatio = aspect
                    }
                },
                onClick = onClick,
                onLongPress = onLongPress,
                modifier = Modifier
                    .fillMaxSize()
                    .offset { IntOffset(pageOffsetX.roundToInt(), 0) },
            )
            positionText?.let {
                Surface(
                    modifier = Modifier
                        .align(Alignment.TopStart)
                        .padding(start = 12.dp, top = 8.dp),
                    color = ElementBackground.copy(alpha = 0.82f),
                    contentColor = ElementPurple,
                    shape = RoundedCornerShape(999.dp),
                    border = BorderStroke(1.dp, ElementPurple.copy(alpha = 0.46f)),
                ) {
                    Text(
                        text = it,
                        modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp),
                        fontSize = 11.sp,
                        lineHeight = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            }
        }
    }
}

private fun detailCarouselHeight(imageAspectRatio: Float?): Dp =
    when {
        imageAspectRatio == null -> 340.dp
        imageAspectRatio >= 1.2f -> 304.dp
        imageAspectRatio <= 0.82f -> 520.dp
        else -> 420.dp
    }

@Composable
private fun DetailCarouselPhotoPage(
    asset: ProjectAsset,
    previewQuality: PreviewQuality,
    horizontalPadding: Dp,
    preferredAspectRatio: Float? = null,
    pageScale: Float = 1f,
    onClick: (() -> Unit)?,
    onLongPress: (() -> Unit)? = null,
    onPreviewAspectRatio: ((Float) -> Unit)? = null,
    onPreviewReady: (() -> Unit)? = null,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier
            .graphicsLayer {
                scaleX = pageScale
                scaleY = pageScale
            }
            .padding(horizontal = horizontalPadding),
        contentAlignment = Alignment.Center,
    ) {
        PhotoPreview(
            asset = asset,
            previewQuality = previewQuality,
            fitToImageAspect = true,
            preferredAspectRatio = preferredAspectRatio,
            contentScale = ContentScale.Fit,
            backgroundColor = ElementSurface,
            onClick = onClick,
            onLongClick = onLongPress,
            onPreviewAspectRatio = onPreviewAspectRatio,
            onPreviewReady = onPreviewReady,
            showFallbackText = false,
            modifier = Modifier.fillMaxHeight(),
        )
    }
}

private const val PHOTO_DETAIL_LOADING_ASPECT_RATIO = 0.67f

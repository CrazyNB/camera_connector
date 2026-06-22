package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.FilterList
import androidx.compose.material3.Icon
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.ProjectAsset


@Composable
internal fun PhotoListControlRow(
    selectedCollection: ProjectPhotoCollection,
    onCollectionChange: (ProjectPhotoCollection) -> Unit,
    selectedFilter: AssetFormatFilter,
    selectedSort: PhotoSortMode,
    expanded: Boolean,
    onToggle: () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            LazyRow(
                modifier = Modifier.weight(1f),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                items(ProjectPhotoCollection.entries) { collection ->
                    FilterChipButton(
                        label = collection.label,
                        selected = selectedCollection == collection,
                        onClick = { onCollectionChange(collection) },
                    )
                }
            }
            Spacer(Modifier.width(8.dp))
            Surface(
                modifier = Modifier
                    .size(42.dp)
                    .clickable(onClick = onToggle),
                color = if (expanded) ElementBlue else ElementControlSurface,
                contentColor = if (expanded) ElementOnAccent else ElementBlue,
                shape = RoundedCornerShape(14.dp),
                border = BorderStroke(1.dp, if (expanded) ElementBlue else ElementBorder),
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        imageVector = Icons.Outlined.FilterList,
                        contentDescription = if (expanded) "收起筛选" else "展开筛选",
                        modifier = Modifier.size(20.dp),
                    )
                }
            }
        }
    }
}

@Composable
internal fun PhotoSortBar(
    selectedSort: PhotoSortMode,
    onSortChange: (PhotoSortMode) -> Unit,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(PhotoSortMode.entries) { sortMode ->
            FilterChipButton(
                label = sortMode.label,
                selected = selectedSort == sortMode,
                onClick = { onSortChange(sortMode) },
            )
        }
    }
}

@Composable
internal fun AssetFormatFilterBar(
    selectedFilter: AssetFormatFilter,
    onFilterChange: (AssetFormatFilter) -> Unit,
    assets: List<ProjectAsset>,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(AssetFormatFilter.entries) { filter ->
            val count = assets.count { filter.matches(it) }
            FilterChipButton(
                label = "${filter.label} $count",
                selected = selectedFilter == filter,
                onClick = { onFilterChange(filter) },
            )
        }
    }
}

@Composable
internal fun GuestMarkFilterBar(
    selectedFilter: GuestMarkFilter,
    onFilterChange: (GuestMarkFilter) -> Unit,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(GuestMarkFilter.entries) { filter ->
            FilterChipButton(
                label = filter.label,
                selected = selectedFilter == filter,
                onClick = { onFilterChange(filter) },
            )
        }
    }
}

private val modelScoreThresholdOptions = listOf<Int?>(null, 60, 70, 80)

@Composable
internal fun ModelScoreThresholdBar(
    selectedScore: Int?,
    onScoreChange: (Int?) -> Unit,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(modelScoreThresholdOptions) { score ->
            FilterChipButton(
                label = score?.let { "评分 ≥$it" } ?: "评分不限",
                selected = selectedScore == score,
                onClick = { onScoreChange(score) },
            )
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
internal fun CompactPhotoTile(
    asset: ProjectAsset,
    selected: Boolean,
    selectionMode: Boolean,
    onClick: () -> Unit,
    onLongClick: () -> Unit,
) {
    val burstBadge = asset.burstCountBadgeText()
    val primaryBadge = asset.tilePrimaryBadgeText()
    val recommendationBadge = asset.recommendationBadgeText()
    val auxiliaryBadges = asset.tileAuxiliaryBadges()
    val tileShape = RoundedCornerShape(10.dp)
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(tileShape)
            .background(ElementSurface)
            .border(
                width = 1.dp,
                color = if (selected) ElementBlue else ElementCardBorder,
                shape = tileShape,
            )
            .semantics {
                contentDescription = listOf(
                    "照片 ${asset.filename()}",
                    primaryBadge,
                    recommendationBadge?.takeIf { asset.isBestRecommendedAsset() },
                    auxiliaryBadges.joinToString(" ").takeIf { it.isNotBlank() },
                ).filterNotNull().joinToString(" ")
                stateDescription = when {
                    selected -> "已选择"
                    selectionMode -> "未选择"
                    else -> "可打开"
                }
            }
            .combinedClickable(
                onClick = onClick,
                onLongClick = onLongClick,
            )
            .padding(1.5.dp),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(1.2f),
        ) {
            PhotoPreview(
                asset = asset,
                compactFallback = true,
                backgroundColor = asset.previewAccentColor().copy(alpha = 0.16f),
                trimLetterbox = true,
                modifier = Modifier.matchParentSize(),
            )
            burstBadge?.let {
                PhotoEdgeBadge(
                    text = it,
                    color = ElementPurple,
                    modifier = Modifier
                        .align(Alignment.TopStart)
                        .padding(6.dp),
                )
            }
            primaryBadge?.let {
                PhotoEdgeBadge(
                    text = it,
                    color = asset.modelScoreText()?.let { asset.modelScoreColor() }
                        ?: asset.tilePrimaryBadgeColor(),
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .padding(6.dp),
                )
            }
            recommendationBadge?.takeIf { asset.isBestRecommendedAsset() }?.let {
                PhotoEdgeBadge(
                    text = it,
                    color = ElementSuccess,
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(6.dp),
                )
            }
        }
        if (auxiliaryBadges.isNotEmpty()) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(start = 2.dp, top = 5.dp, end = 2.dp, bottom = 1.dp)
                    .horizontalScroll(rememberScrollState()),
                horizontalArrangement = Arrangement.spacedBy(5.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                auxiliaryBadges.forEach { badge ->
                    PhotoInlineBadge(
                        text = badge,
                        color = auxiliaryBadgeColor(badge),
                    )
                }
            }
        }
    }
}

@Composable
internal fun PhotoEdgeBadge(
    text: String,
    color: Color,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier,
        color = ElementBackground.copy(alpha = 0.78f),
        contentColor = color,
        shape = RoundedCornerShape(999.dp),
        border = BorderStroke(1.dp, color.copy(alpha = 0.46f)),
    ) {
        Text(
            text = text,
            modifier = Modifier.padding(horizontal = 7.dp, vertical = 3.dp),
            fontSize = 10.sp,
            lineHeight = 11.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
internal fun PhotoInlineBadge(
    text: String,
    color: Color,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier,
        color = color.copy(alpha = 0.12f),
        contentColor = color,
        shape = RoundedCornerShape(999.dp),
        border = BorderStroke(1.dp, color.copy(alpha = 0.36f)),
    ) {
        Text(
            text = text,
            modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
            fontSize = 9.sp,
            lineHeight = 10.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

internal fun auxiliaryBadgeColor(text: String): Color =
    when (text) {
        "收藏" -> ElementSuccess
        "标记" -> ElementBlue
        "风险", "不支持预览" -> ElementDanger
        "RAW", "JPG", "JPG+RAW" -> ElementPurple
        "视频" -> ElementInfo
        else -> ElementInfo
    }

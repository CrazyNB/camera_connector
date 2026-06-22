package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import com.cameraconnector.app.core.ProjectAsset

@Composable
internal fun ProjectFeedbackToast(
    message: String,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier,
        color = ElementSurface.copy(alpha = 0.96f),
        contentColor = ElementText,
        shape = RoundedCornerShape(999.dp),
        border = BorderStroke(1.dp, ElementCardBorder),
    ) {
        Text(
            text = message,
            modifier = Modifier.padding(horizontal = 18.dp, vertical = 10.dp),
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.SemiBold,
        )
    }
}

@Composable
internal fun BurstGroupPreviewDialog(
    item: ProjectPhotoGridItemUi,
    allProjectAssets: List<ProjectAsset>,
    onDismiss: () -> Unit,
    onOpenAsset: (ProjectAsset) -> Unit,
) {
    val previewItems = remember(item, allProjectAssets) {
        val filmstrip = burstMemberFilmstrip(item.coverAsset, allProjectAssets)
        if (filmstrip.isNotEmpty()) {
            filmstrip
        } else {
            item.members.map { asset ->
                BurstMemberFilmstripItemUi(
                    asset = asset,
                    badgeText = if (asset.assetSelectionId() == item.coverAsset.assetSelectionId()) "优选" else "备选",
                    scoreText = null,
                )
            }
        }
    }
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(18.dp),
            contentAlignment = Alignment.Center,
        ) {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(
                                "\u8fde\u62cd\u7ec4",
                                style = MaterialTheme.typography.titleLarge,
                                fontWeight = FontWeight.Bold,
                            )
                            Spacer(Modifier.height(4.dp))
                            Text(
                                "${previewItems.size} 张 · 优选 ${item.coverAsset.filename()}",
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                style = MaterialTheme.typography.bodySmall,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                        TextButton(onClick = onDismiss) {
                            Text("关闭")
                        }
                    }
                    LazyVerticalGrid(
                        columns = GridCells.Fixed(3),
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(360.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        items(
                            count = previewItems.size,
                            key = { index -> previewItems[index].asset.assetSelectionId() },
                        ) { index ->
                            BurstGroupPreviewTile(
                                item = previewItems[index],
                                tileUi = burstPreviewTileUi(
                                    item = previewItems[index],
                                    index = index,
                                    total = previewItems.size,
                                ),
                                onClick = { onOpenAsset(previewItems[index].asset) },
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun BurstGroupPreviewTile(
    item: BurstMemberFilmstripItemUi,
    tileUi: BurstPreviewTileUi,
    onClick: () -> Unit,
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(ElementSurface)
            .border(1.dp, ElementCardBorder, RoundedCornerShape(14.dp))
            .clickable(onClick = onClick)
            .padding(7.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(1f),
        ) {
            PhotoPreview(
                asset = item.asset,
                compactFallback = true,
                backgroundColor = item.asset.previewAccentColor().copy(alpha = 0.16f),
                modifier = Modifier.matchParentSize(),
            )
            PhotoEdgeBadge(
                text = tileUi.positionText,
                color = ElementPurple,
                modifier = Modifier
                    .align(Alignment.TopStart)
                    .padding(5.dp),
            )
            tileUi.scoreText?.let { scoreText ->
                PhotoEdgeBadge(
                    text = scoreText,
                    color = ElementWarning,
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .padding(5.dp),
                )
            }
            if (tileUi.modelSelected) {
                PhotoEdgeBadge(
                    text = "\u4f18\u9009",
                    color = ElementSuccess,
                    modifier = Modifier
                        .align(Alignment.BottomEnd)
                        .padding(5.dp),
                )
            }
        }
        Text(
            item.asset.filename(),
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        if (tileUi.auxiliaryBadges.isNotEmpty()) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                tileUi.auxiliaryBadges.forEach { badge ->
                    PhotoInlineBadge(
                        text = badge,
                        color = auxiliaryBadgeColor(badge),
                    )
                }
            }
        }
    }
}

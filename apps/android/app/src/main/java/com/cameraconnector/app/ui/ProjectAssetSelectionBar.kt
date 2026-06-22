package com.cameraconnector.app.ui

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.AutoAwesome
import androidx.compose.material.icons.outlined.PhotoLibrary
import androidx.compose.material.icons.outlined.SyncAlt
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp


@Composable
internal fun SelectedAssetsActionBar(
    selectedCount: Int,
    totalCount: Int,
    canOpen: Boolean,
    canEvaluate: Boolean,
    canSplitOut: Boolean,
    canMerge: Boolean,
    canDelete: Boolean,
    onOpen: () -> Unit,
    onEvaluate: () -> Unit,
    onSplitOut: () -> Unit,
    onDelete: () -> Unit,
    onMerge: () -> Unit,
    onSelectAll: () -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier.animateContentSize(),
        color = ElementPanel.copy(alpha = 0.96f),
        contentColor = MaterialTheme.colorScheme.onSurface,
        shape = RoundedCornerShape(18.dp),
        border = BorderStroke(1.dp, ElementBlue.copy(alpha = 0.35f)),
    ) {
        Column(
            modifier = Modifier.padding(horizontal = 14.dp, vertical = 10.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Text(
                    "\u5df2\u9009 $selectedCount \u9879",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    modifier = Modifier.weight(1f),
                )
                TextButton(
                    onClick = if (selectedCount < totalCount) onSelectAll else onCancel,
                    enabled = totalCount > 0,
                ) {
                    Text(if (selectedCount < totalCount) "\u5168\u9009" else "\u6e05\u7a7a")
                }
                TextButton(
                    onClick = onDelete,
                    enabled = canDelete,
                ) {
                    Text("删除", color = if (canDelete) ElementDanger else MaterialTheme.colorScheme.onSurfaceVariant)
                }
                TextButton(onClick = onCancel) {
                    Text("\u5b8c\u6210")
                }
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                SelectionActionButton(
                    icon = Icons.Outlined.PhotoLibrary,
                    text = "\u6253\u5f00",
                    enabled = canOpen,
                    primary = true,
                    modifier = Modifier.weight(1f),
                    onClick = onOpen,
                )
                SelectionActionButton(
                    icon = Icons.Outlined.AutoAwesome,
                    text = "\u8bc4\u4ef7",
                    enabled = canEvaluate,
                    modifier = Modifier.weight(1f),
                    accent = ElementBlue,
                    onClick = onEvaluate,
                )
                SelectionActionButton(
                    icon = Icons.Outlined.SyncAlt,
                    text = "\u79fb\u51fa",
                    enabled = canSplitOut,
                    modifier = Modifier.weight(1f),
                    onClick = onSplitOut,
                )
                SelectionActionButton(
                    icon = Icons.Outlined.PhotoLibrary,
                    text = "\u5408\u5e76",
                    enabled = canMerge,
                    modifier = Modifier.weight(1f),
                    accent = ElementPurple,
                    onClick = onMerge,
                )
            }
        }
    }
}

@Composable
private fun SelectionActionButton(
    icon: ImageVector,
    text: String,
    enabled: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    primary: Boolean = false,
    accent: Color = MaterialTheme.colorScheme.onSurface,
) {
    val buttonModifier = modifier.height(44.dp)
    if (primary) {
        Button(
            onClick = onClick,
            enabled = enabled,
            modifier = buttonModifier,
            shape = RoundedCornerShape(10.dp),
            colors = ButtonDefaults.buttonColors(
                containerColor = ElementBlue,
                contentColor = ElementOnAccent,
            ),
            contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
        ) {
            SelectionActionButtonContent(icon = icon, text = text)
        }
    } else {
        OutlinedButton(
            onClick = onClick,
            enabled = enabled,
            modifier = buttonModifier,
            shape = RoundedCornerShape(10.dp),
            border = BorderStroke(1.dp, if (enabled) accent.copy(alpha = 0.45f) else ElementBorder),
            colors = ButtonDefaults.outlinedButtonColors(
                containerColor = ElementControlSurface,
                contentColor = accent,
            ),
            contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp),
        ) {
            SelectionActionButtonContent(icon = icon, text = text)
        }
    }
}

@Composable
private fun SelectionActionButtonContent(
    icon: ImageVector,
    text: String,
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            modifier = Modifier.size(17.dp),
        )
        Text(
            text,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            fontSize = 11.sp,
            lineHeight = 12.sp,
            fontWeight = FontWeight.SemiBold,
        )
    }
}

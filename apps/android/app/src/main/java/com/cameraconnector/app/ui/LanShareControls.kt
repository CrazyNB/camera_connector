package com.cameraconnector.app.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.ProjectAssetQuery

internal data class LanShareScopeUi(
    val projectName: String,
    val favoriteOnly: Boolean,
    val markedOnly: Boolean,
    val minModelScore: Int?,
    val assetCount: Int,
)

private val lanShareScoreThresholdOptions = listOf<Int?>(null, 60, 70, 80)

internal fun lanShareAssetQuery(
    baseQuery: ProjectAssetQuery,
    favoriteOnly: Boolean,
    markedOnly: Boolean,
    minModelScore: Int?,
): ProjectAssetQuery =
    baseQuery.copy(
        userMarkAny = buildList {
            if (favoriteOnly) {
                add("favorite")
            }
            if (markedOnly) {
                add("marked")
            }
        },
        minModelScore = minModelScore,
    )

@Composable
internal fun LanShareModeDialog(
    mode: LanShareMenuAction,
    scope: LanShareScopeUi,
    action: LanShareActionUi,
    sharingActive: Boolean,
    activeUrl: String?,
    error: String?,
    editable: Boolean,
    onFavoriteOnlyChange: (Boolean) -> Unit,
    onMarkedOnlyChange: (Boolean) -> Unit,
    onMinModelScoreChange: (Int?) -> Unit,
    onDismiss: () -> Unit,
    onStart: () -> Unit,
    onStop: () -> Unit,
    onCopyLink: (String) -> Unit,
) {
    val projectSyncMode = mode == LanShareMenuAction.ProjectSync
    val title = when {
        projectSyncMode -> "\u5c40\u57df\u7f51\u9879\u76ee\u5171\u4eab"
        sharingActive -> "\u591a\u65b9\u7b5b\u9009\u8303\u56f4"
        else -> "\u591a\u65b9\u7b5b\u9009\u914d\u7f6e"
    }
    val description = when {
        projectSyncMode && sharingActive ->
            "\u684c\u9762\u7aef\u6253\u5f00\u5c40\u57df\u7f51\u9879\u76ee\u626b\u63cf\u5373\u53ef\u53d1\u73b0\u5e76\u540c\u6b65\u9879\u76ee\u7d22\u5f15\u3002"
        projectSyncMode ->
            "\u5c06\u5f53\u524d\u9879\u76ee\u7d22\u5f15\u5171\u4eab\u7ed9\u540c\u4e00\u5c40\u57df\u7f51\u684c\u9762\u7aef\uff0c\u7528\u4e8e\u9879\u76ee\u8fc1\u79fb\u548c\u7f3a\u5931\u6570\u636e\u8865\u9f50\u3002"
        sharingActive ->
            "\u5f53\u524d\u94fe\u63a5\u6b63\u5728\u5171\u4eab\u4ee5\u4e0b\u7b5b\u9009\u8303\u56f4\u3002"
        else ->
            "\u5c06\u5f53\u524d\u7b5b\u9009\u7ed3\u679c\u5171\u4eab\u7ed9\u540c\u4e00\u5c40\u57df\u7f51\u8bbf\u5ba2\u3002"
    }
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = ElementSurface,
        title = { Text(title) },
        text = {
            Column(
                modifier = Modifier.verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                Text(
                    text = description,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                LanShareScopeRow(label = "\u9879\u76ee", value = scope.projectName)
                if (projectSyncMode) {
                    LanShareScopeRow(label = "\u8303\u56f4", value = "\u6574\u4e2a\u9879\u76ee")
                } else if (editable) {
                    LanShareMarkOptions(
                        favoriteOnly = scope.favoriteOnly,
                        markedOnly = scope.markedOnly,
                        onFavoriteOnlyChange = onFavoriteOnlyChange,
                        onMarkedOnlyChange = onMarkedOnlyChange,
                    )
                    LanShareOptionGroup(
                        label = "\u8bc4\u5206\u9608\u503c",
                        options = lanShareScoreThresholdOptions,
                        selected = scope.minModelScore,
                        labelForValue = { value -> value?.let { "\u2265 $it" } ?: "\u4e0d\u9650" },
                        onSelected = onMinModelScoreChange,
                    )
                } else {
                    LanShareScopeRow(label = "\u6807\u7b7e", value = lanShareFilterTagSummary(scope.favoriteOnly, scope.markedOnly))
                    LanShareScopeRow(label = "\u8bc4\u5206\u9608\u503c", value = scope.minModelScore?.let { "\u2265 $it" } ?: "\u4e0d\u9650")
                }
                LanShareScopeRow(label = "\u6570\u91cf", value = "${scope.assetCount} \u5f20")
                activeUrl?.takeIf { lanShareDialogShowsUserVisibleLink(mode, sharingActive) }?.let { url ->
                    LanShareScopeRow(label = "\u94fe\u63a5", value = url)
                }
                if (!sharingActive) {
                    action.disabledReason?.let { reason ->
                        Text(
                            text = reason,
                            style = MaterialTheme.typography.bodySmall,
                            color = ElementDanger,
                        )
                    }
                }
                if (sharingActive) {
                    Text(
                        text = if (projectSyncMode) {
                            "\u9879\u76ee\u5171\u4eab\u9762\u5411\u684c\u9762\u7aef\u540c\u6b65\uff0c\u4e0d\u53d7\u5f53\u524d\u7b5b\u9009\u6761\u4ef6\u5f71\u54cd\u3002"
                        } else {
                            "\u8981\u6539\u53d8\u7b5b\u9009\u6761\u4ef6\uff0c\u8bf7\u5148\u505c\u6b62\u5f53\u524d\u591a\u65b9\u7b5b\u9009\uff0c\u518d\u8c03\u6574\u7b5b\u9009\u540e\u91cd\u65b0\u5f00\u59cb\u3002"
                        },
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                error?.takeIf { it.isNotBlank() }?.let { message ->
                    Text(
                        text = message,
                        style = MaterialTheme.typography.bodySmall,
                        color = ElementDanger,
                    )
                }
            }
        },
        confirmButton = {
            if (!sharingActive) {
                Button(
                    enabled = action.enabled,
                    onClick = onStart,
                ) {
                    Text("\u5f00\u59cb\u5171\u4eab")
                }
            } else if (!projectSyncMode && activeUrl != null) {
                Button(
                    onClick = { onCopyLink(activeUrl) },
                ) {
                    Text("\u590d\u5236\u94fe\u63a5")
                }
            } else {
                Button(onClick = onDismiss) {
                    Text("\u5173\u95ed")
                }
            }
        },
        dismissButton = {
            if (!sharingActive) {
                TextButton(onClick = onDismiss) {
                    Text("\u53d6\u6d88")
                }
            } else {
                TextButton(
                    onClick = {
                        onDismiss()
                        onStop()
                    },
                ) {
                    Text("\u505c\u6b62\u5171\u4eab")
                }
            }
        },
    )
}

@Composable
private fun LanShareScopeRow(
    label: String,
    value: String,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.width(12.dp))
        Text(
            text = value,
            modifier = Modifier.weight(1f),
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun <T> LanShareOptionGroup(
    label: String,
    options: List<T>,
    selected: T,
    labelForValue: (T) -> String,
    onSelected: (T) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(
            text = label,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            items(options) { option ->
                FilterChipButton(
                    label = labelForValue(option),
                    selected = option == selected,
                    onClick = { onSelected(option) },
                )
            }
        }
    }
}

@Composable
private fun LanShareMarkOptions(
    favoriteOnly: Boolean,
    markedOnly: Boolean,
    onFavoriteOnlyChange: (Boolean) -> Unit,
    onMarkedOnlyChange: (Boolean) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(
            text = "标签",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            item {
                FilterChipButton(
                    label = "不限",
                    selected = !favoriteOnly && !markedOnly,
                    onClick = {
                        onFavoriteOnlyChange(false)
                        onMarkedOnlyChange(false)
                    },
                )
            }
            item {
                FilterChipButton(
                    label = "收藏",
                    selected = favoriteOnly,
                    onClick = { onFavoriteOnlyChange(!favoriteOnly) },
                )
            }
            item {
                FilterChipButton(
                    label = "标记",
                    selected = markedOnly,
                    onClick = { onMarkedOnlyChange(!markedOnly) },
                )
            }
        }
    }
}

private fun lanShareFilterTagSummary(
    favoriteOnly: Boolean,
    markedOnly: Boolean,
): String =
    buildList {
        if (favoriteOnly) {
            add("\u6536\u85cf")
        }
        if (markedOnly) {
            add("\u6807\u8bb0")
        }
    }.takeIf { it.isNotEmpty() }?.joinToString(" + ") ?: "\u4e0d\u9650"

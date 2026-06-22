package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.KeyboardArrowRight
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.cameraconnector.app.core.PromptPackUi

@Composable
internal fun PromptPacksScreen(
    promptPacks: List<PromptPackUi>,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onCreatePromptPackage: () -> Unit,
    onCreatePromptPackInPackage: (String) -> Unit,
    onOpenPromptPack: (String) -> Unit,
    onDeletePromptPackage: (String) -> Unit,
    onDeletePromptPack: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var collapsedPackages by rememberSaveable { mutableStateOf(emptyList<String>()) }
    val promptPackages = promptPacks
        .groupBy { promptPackageFolder(it) }
        .toList()
        .sortedWith(
            compareBy<Pair<String, List<PromptPackUi>>> {
                when (it.first) {
                    "user" -> 0
                    "builtin" -> 2
                    else -> 1
                }
            }.thenBy { promptPackageLabel(it.first) },
        )

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            CompactBackHeader(
                title = "\u63d0\u793a\u8bcd\u914d\u7f6e",
                subtitle = "按提示词包管理摄影评价偏好",
                onBack = onBack,
                trailing = {
                OutlinedButton(
                    onClick = onCreatePromptPackage,
                    enabled = actionInFlight == null,
                    shape = elementShape,
                ) {
                    Icon(Icons.Outlined.Add, contentDescription = null, modifier = Modifier.size(18.dp))
                    Spacer(Modifier.width(6.dp))
                    Text("新建提示词包")
                }
                },
            )
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        if (promptPacks.isEmpty()) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "\u6682\u65e0\u63d0\u793a\u8bcd\u914d\u7f6e",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            promptPackages.forEach { (packageFolder, packsInPackage) ->
                val expanded = packageFolder !in collapsedPackages
                item(key = "package-$packageFolder") {
                    PromptPackageSection(
                        packageFolder = packageFolder,
                        profiles = packsInPackage.sortedBy { promptPackDisplayName(it) },
                        expanded = expanded,
                        actionInFlight = actionInFlight,
                        onToggle = {
                            collapsedPackages = if (expanded) {
                                collapsedPackages + packageFolder
                            } else {
                                collapsedPackages - packageFolder
                            }
                        },
                        onCreate = {
                            onCreatePromptPackInPackage(packageFolder)
                            collapsedPackages = collapsedPackages - packageFolder
                        },
                        onOpenPromptPack = onOpenPromptPack,
                        onDeletePackage = { onDeletePromptPackage(packageFolder) },
                        onDeletePromptPack = onDeletePromptPack,
                    )
                }
            }
        }
    }
}

@Composable
private fun PromptPackageSection(
    packageFolder: String,
    profiles: List<PromptPackUi>,
    expanded: Boolean,
    actionInFlight: String?,
    onToggle: () -> Unit,
    onCreate: () -> Unit,
    onOpenPromptPack: (String) -> Unit,
    onDeletePackage: () -> Unit,
    onDeletePromptPack: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val isBuiltinPackage = packageFolder == "builtin"
    ElementCard(modifier = modifier.fillMaxWidth()) {
        Column(Modifier.fillMaxWidth()) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable(onClick = onToggle)
                    .padding(horizontal = 14.dp, vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                Icon(
                    imageVector = if (expanded) Icons.Outlined.KeyboardArrowDown else Icons.AutoMirrored.Outlined.KeyboardArrowRight,
                    contentDescription = if (expanded) "收起提示词包" else "展开提示词包",
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(20.dp),
                )
                Column(Modifier.weight(1f)) {
                    Text(
                        promptPackageLabel(packageFolder),
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Spacer(Modifier.height(2.dp))
                    Text(
                        "${profiles.size} 个提示词",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
                if (!isBuiltinPackage) {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedButton(
                            onClick = onCreate,
                            enabled = actionInFlight == null,
                            shape = elementShape,
                            contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp),
                        ) {
                            Icon(Icons.Outlined.Add, contentDescription = null, modifier = Modifier.size(16.dp))
                            Spacer(Modifier.width(4.dp))
                            Text("新建")
                        }
                        OutlinedButton(
                            onClick = onDeletePackage,
                            enabled = actionInFlight == null,
                            shape = elementShape,
                            colors = ButtonDefaults.outlinedButtonColors(contentColor = ElementDanger),
                            border = BorderStroke(1.dp, ElementDanger),
                            contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp),
                        ) {
                            Text("删除包")
                        }
                    }
                }
            }
            if (expanded) {
                HorizontalDivider(
                    color = MaterialTheme.colorScheme.outline.copy(alpha = 0.45f),
                    thickness = 1.dp,
                )
                Column(Modifier.fillMaxWidth()) {
                    profiles.forEachIndexed { index, profile ->
                        PromptPackRow(
                            profile = profile,
                            onClick = { onOpenPromptPack(profile.promptPackId) },
                            onDelete = { onDeletePromptPack(profile.promptPackId) },
                            modifier = Modifier.fillMaxWidth(),
                        )
                        if (index != profiles.lastIndex) {
                            HorizontalDivider(
                                modifier = Modifier.padding(start = 14.dp),
                                color = MaterialTheme.colorScheme.outline.copy(alpha = 0.25f),
                                thickness = 1.dp,
                            )
                        }
                    }
                }
            } else if (!isBuiltinPackage && profiles.isEmpty()) {
                Text(
                    "展开后新建提示词",
                    modifier = Modifier.padding(start = 44.dp, end = 14.dp, bottom = 12.dp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun PromptPackRow(
    profile: PromptPackUi,
    onClick: () -> Unit,
    onDelete: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 13.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(
                promptPackDisplayName(profile),
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                promptPackMetaText(profile),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            profile.activePromptText
                ?.takeIf { it.isNotBlank() }
                ?.let { promptMarkdown ->
                    MarkdownText(
                        markdown = promptMarkdown,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.82f),
                        compact = true,
                        maxLines = 1,
                    )
                }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            if (!profile.builtIn) {
                Text(
                    "删除",
                    modifier = Modifier.clickable(onClick = onDelete),
                    color = ElementDanger,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                )
            }
            Text(
                if (profile.builtIn) "复制" else "编辑",
                color = ElementBlue,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
            )
        }
    }
}

internal fun promptPackMetaText(profile: PromptPackUi): String =
    listOf(
        promptStyleTagsText(profile),
        sceneProfileLabel(profile.sceneProfile),
        if (profile.builtIn) "内置" else "自定义",
    ).filter { it.isNotBlank() }.distinct().joinToString(" / ")

@Composable
internal fun PromptPackEditorScreen(
    profile: PromptPackUi?,
    initialDistributionFolder: String,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onSave: (PromptPackUi, String, List<String>, String, String, String) -> Unit,
    onCreate: (String, List<String>, String, String, String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val createMode = profile == null
    val builtInProfile = profile?.builtIn == true
    val editableExistingProfile = !createMode && !builtInProfile
    var name by remember(profile?.promptPackId) {
        mutableStateOf(
            when {
                createMode -> ""
                builtInProfile -> profile.let(::promptPackDisplayName).let { "自定义 $it" }
                else -> profile.let(::promptPackDisplayName)
            },
        )
    }
    var styleTagsText by remember(profile?.promptPackId) {
        mutableStateOf(
            profile?.styleTags
                ?.filter { it.isNotBlank() }
                ?.joinToString(" ") { promptStyleTagLabel(it) }
                ?: "通用 均衡",
        )
    }
    var sceneProfile by remember(profile?.promptPackId) {
        mutableStateOf(profile?.sceneProfile?.ifBlank { "general" } ?: "general")
    }
    var distributionFolder by remember(profile?.promptPackId, initialDistributionFolder) {
        mutableStateOf(
            when {
                createMode -> initialDistributionFolder
                builtInProfile -> "user"
                else -> profile.distributionFolder.takeIf { it.isNotBlank() } ?: "user"
            },
        )
    }
    var promptText by remember(profile?.promptPackId, profile?.sharedPreference, profile?.activePromptText) {
        mutableStateOf(profile?.sharedPreference ?: profile?.activePromptText.orEmpty())
    }
    var promptTab by rememberSaveable(profile?.promptPackId) { mutableStateOf("edit") }
    val cleanName = name.trim()
    val cleanPrompt = promptText.trim()
    val cleanPackage = distributionFolder.trim()
    val cleanStyleTags = parsePromptStyleTags(styleTagsText)
    val actionsEnabled = actionInFlight == null
    val canSubmit = actionsEnabled && cleanPrompt.isNotBlank() &&
        when {
            createMode -> cleanName.isNotBlank() && cleanPackage.isNotBlank()
            builtInProfile -> cleanName.isNotBlank() && cleanPackage.isNotBlank()
            else -> cleanName.isNotBlank()
        }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            CompactBackHeader(
                title = when {
                    createMode -> "新建提示词"
                    else -> profile.let(::promptPackDisplayName)
                },
                subtitle = when {
                    createMode -> "选择提示词包，保存后进入编辑"
                    builtInProfile -> "内置偏好会复制为全局自定义偏好"
                    else -> "保存后成为这套全局偏好的新版本"
                },
                onBack = onBack,
            )
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        if (createMode || builtInProfile || editableExistingProfile) {
            item {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text(if (builtInProfile) "复制后的名称" else "提示词名称") },
                    modifier = Modifier.fillMaxWidth(),
                    enabled = actionsEnabled,
                    singleLine = true,
                )
            }
        }

        if (createMode || builtInProfile) {
            item {
                OutlinedTextField(
                    value = distributionFolder,
                    onValueChange = { distributionFolder = it },
                    label = { Text("提示词包") },
                    supportingText = { Text("同一个提示词包可以整体导入、导出和共享，例如 user、portrait") },
                    modifier = Modifier.fillMaxWidth(),
                    enabled = actionsEnabled,
                    singleLine = true,
                )
            }
        } else {
            item {
                Surface(
                    modifier = Modifier.fillMaxWidth(),
                    color = ElementControlSurface,
                    shape = elementShape,
                    border = BorderStroke(1.dp, ElementBorder),
                ) {
                    Row(
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 12.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                            Text("提示词包", style = MaterialTheme.typography.labelLarge)
                            Text(
                                promptPackageLabel(promptPackageFolder(profile)),
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        Text("不可更改", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }

        if (createMode || editableExistingProfile) {
            item {
                OutlinedTextField(
                    value = styleTagsText,
                    onValueChange = { styleTagsText = it },
                    label = { Text("风格标签") },
                    supportingText = { Text("用空格或逗号分隔，例如：通用 均衡 人像") },
                    modifier = Modifier.fillMaxWidth(),
                    enabled = actionsEnabled,
                    singleLine = true,
                )
            }
            item {
                OptionRow(
                    title = "适用场景",
                    values = projectSceneProfileOptions(),
                    selected = sceneProfile,
                    enabled = actionsEnabled,
                    labelForValue = ::sceneProfileLabel,
                    onSelected = { sceneProfile = it },
                )
            }
        }

        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            "我的摄影评价偏好",
                            modifier = Modifier.weight(1f),
                            style = MaterialTheme.typography.titleMedium,
                        )
                        PromptEditorTab(
                            text = "编辑",
                            selected = promptTab == "edit",
                            enabled = actionsEnabled,
                            onClick = { promptTab = "edit" },
                        )
                        PromptEditorTab(
                            text = "预览",
                            selected = promptTab == "preview",
                            enabled = actionsEnabled,
                            onClick = { promptTab = "preview" },
                        )
                    }
                    if (promptTab == "preview") {
                        if (cleanPrompt.isBlank()) {
                            Text(
                                "输入 Markdown 后会在这里预览。",
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        } else {
                            MarkdownText(markdown = cleanPrompt)
                        }
                    } else {
                        OutlinedTextField(
                            value = promptText,
                            onValueChange = { promptText = it },
                            modifier = Modifier.fillMaxWidth(),
                            minLines = 10,
                            enabled = actionsEnabled,
                            placeholder = { Text("例如：\n# 人像偏好\n- 优先自然表情\n- 避免过度磨皮\n- 保留现场氛围") },
                            supportingText = {
                                Text(
                                    "支持 Markdown：# 标题、- 列表、> 引用、`代码`、**加粗**、*斜体*",
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            },
                        )
                    }
                }
            }
        }
        item {
            Button(
                onClick = {
                    if (createMode) {
                        onCreate(
                            cleanName,
                            cleanStyleTags.ifEmpty { listOf("general", "balanced") },
                            sceneProfile,
                            cleanPackage,
                            cleanPrompt,
                        )
                    } else {
                        onSave(
                            profile,
                            cleanName,
                            cleanStyleTags.ifEmpty { listOf("general", "balanced") },
                            sceneProfile,
                            cleanPackage.ifBlank { "user" },
                            cleanPrompt,
                        )
                    }
                },
                enabled = canSubmit,
                modifier = Modifier.fillMaxWidth(),
                shape = elementShape,
                colors = ButtonDefaults.buttonColors(
                    containerColor = ElementBlue,
                    contentColor = ElementOnAccent,
                ),
            ) {
                Text(
                    when {
                        createMode -> "创建提示词"
                        builtInProfile -> "保存偏好"
                        else -> "保存提示词"
                    },
                )
            }
        }
    }
}

@Composable
private fun PromptEditorTab(
    text: String,
    selected: Boolean,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    if (selected) {
        Button(
            onClick = onClick,
            enabled = enabled,
            shape = elementShape,
            colors = ButtonDefaults.buttonColors(
                containerColor = ElementBlue,
                contentColor = ElementOnAccent,
            ),
            contentPadding = PaddingValues(horizontal = 14.dp, vertical = 8.dp),
        ) {
            Text(text)
        }
    } else {
        OutlinedButton(
            onClick = onClick,
            enabled = enabled,
            shape = elementShape,
            contentPadding = PaddingValues(horizontal = 14.dp, vertical = 8.dp),
        ) {
            Text(text)
        }
    }
}

private fun parsePromptStyleTags(value: String): List<String> =
    value
        .split(' ', '\n', '\t', ',', '，', '/', '、')
        .map { it.trim() }
        .filter { it.isNotBlank() }
        .distinct()

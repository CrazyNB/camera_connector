package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cameraconnector.app.core.ModelProviderSettingsUi

@Composable
internal fun ModelProviderSettingsCard(
    settings: ModelProviderSettingsUi,
    settingsList: List<ModelProviderSettingsUi>,
    actionsEnabled: Boolean,
    onSaveSettings: (ModelProviderSettingsUi) -> Unit,
    onDeleteSettings: (String) -> Unit,
) {
    val providerOptions = settingsList
        .filter { it.configured || it.settingsId.isNotBlank() || it.defaultModel.isNotBlank() }
        .distinctBy { it.settingsId }
    var selectedSettingsId by remember(providerOptions.map { it.settingsId }.joinToString("|")) {
        mutableStateOf(providerOptions.firstOrNull()?.settingsId ?: settings.settingsId)
    }
    var editingNewProfile by remember { mutableStateOf(providerOptions.isEmpty()) }
    val selectedSettings = if (editingNewProfile) {
        ModelProviderSettingsUi(
            settingsId = "",
            providerKind = "custom",
            providerLabel = "\u81ea\u5b9a\u4e49",
            defaultSendMode = settings.defaultSendMode.ifBlank { "preview_only" },
            defaultBatchSize = providerBatchSizeValue(settings.defaultBatchSize),
        )
    } else {
        providerOptions.firstOrNull { it.settingsId == selectedSettingsId } ?: settings
    }
    var settingsName by remember(selectedSettings.settingsId, editingNewProfile) {
        mutableStateOf(selectedSettings.settingsId.takeUnless { editingNewProfile }.orEmpty())
    }
    var providerKind by remember(selectedSettings.settingsId, selectedSettings.providerKind, editingNewProfile) {
        mutableStateOf(selectedSettings.providerKind.ifBlank { "none" })
    }
    var baseUrl by remember(selectedSettings.settingsId, selectedSettings.baseUrl, editingNewProfile) {
        mutableStateOf(selectedSettings.baseUrl)
    }
    var model by remember(selectedSettings.settingsId, selectedSettings.defaultModel, editingNewProfile) {
        mutableStateOf(selectedSettings.defaultModel)
    }
    var apiKey by remember(selectedSettings.settingsId, selectedSettings.apiKeyConfigured, editingNewProfile) {
        mutableStateOf("")
    }
    var sendMode by remember(selectedSettings.settingsId, selectedSettings.defaultSendMode, editingNewProfile) {
        mutableStateOf(selectedSettings.defaultSendMode.ifBlank { "preview_only" })
    }
    var batchSize by remember(selectedSettings.settingsId, selectedSettings.defaultBatchSize, editingNewProfile) {
        mutableStateOf(providerBatchSizeValue(selectedSettings.defaultBatchSize))
    }
    val normalizedProviderKind = providerKind.trim().lowercase().ifBlank { "none" }
    val cleanSettingsName = settingsName.trim()
    val canSaveProvider = cleanSettingsName.isNotBlank() &&
        normalizedProviderKind != "none" &&
        (baseUrl.trim().isNotBlank() &&
            model.trim().isNotBlank() &&
            (apiKey.trim().isNotBlank() || selectedSettings.apiKeyConfigured || selectedSettings.keyAlias != null))
    val canDeleteProvider = !editingNewProfile &&
        selectedSettings.settingsId.isNotBlank() &&
        providerOptions.any { it.settingsId == selectedSettings.settingsId }

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
                    Text("模型配置", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(4.dp))
                    Text(
                        if (editingNewProfile) "填写服务地址、模型和密钥" else "编辑当前模型服务",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (providerOptions.size > 1) {
                    OutlinedButton(
                        onClick = { editingNewProfile = true },
                        enabled = actionsEnabled,
                        shape = elementShape,
                    ) {
                        Text("新建")
                    }
                }
            }
            if (providerOptions.size > 1) {
                LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    items(providerOptions, key = { it.settingsId }) { option ->
                        FilterChipButton(
                            label = modelProviderOptionLabel(option),
                            selected = !editingNewProfile && option.settingsId == selectedSettingsId,
                            onClick = {
                                editingNewProfile = false
                                selectedSettingsId = option.settingsId
                            },
                        )
                    }
                }
            }
            OutlinedTextField(
                value = settingsName,
                onValueChange = { settingsName = it },
                label = { Text("配置名称") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf("openai", "custom").forEach { kind ->
                    FilterChipButton(
                        label = modelProviderKindLabel(kind, selectedSettings.providerLabel),
                        selected = normalizedProviderKind == kind,
                        onClick = {
                            providerKind = kind
                            if (kind == "openai" && baseUrl.isBlank()) {
                                baseUrl = "https://api.openai.com/v1"
                            }
                        },
                    )
                }
            }
            OutlinedTextField(
                value = baseUrl,
                onValueChange = { baseUrl = it },
                label = { Text("服务地址") },
                modifier = Modifier.fillMaxWidth(),
                enabled = true,
                singleLine = true,
            )
            OutlinedTextField(
                value = model,
                onValueChange = { model = it },
                label = { Text("模型名称") },
                modifier = Modifier.fillMaxWidth(),
                enabled = true,
                singleLine = true,
            )
            OutlinedTextField(
                value = apiKey,
                onValueChange = { apiKey = it },
                label = { Text("API 密钥") },
                supportingText = {
                    Text(if (selectedSettings.apiKeyConfigured) "已保存密钥，留空不会覆盖" else "仅保存到本机配置，不回显明文")
                },
                modifier = Modifier.fillMaxWidth(),
                enabled = true,
                singleLine = true,
                visualTransformation = PasswordVisualTransformation(),
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf("preview_only", "detail_image").forEach { mode ->
                    FilterChipButton(
                        label = modelSendModeLabel(mode),
                        selected = sendMode == mode,
                        onClick = { sendMode = mode },
                    )
                }
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("\u6279\u5904\u7406\u6570\u91cf", color = MaterialTheme.colorScheme.onSurfaceVariant)
                BatchSizeToggle(
                    batchSize = batchSize,
                    onBatchSizeChange = { batchSize = it },
                )
            }
            Button(
                onClick = {
                    onSaveSettings(
                        selectedSettings.copy(
                            settingsId = cleanSettingsName,
                            providerKind = normalizedProviderKind,
                            providerLabel = modelProviderKindLabel(normalizedProviderKind, selectedSettings.providerLabel),
                            baseUrl = baseUrl.trim(),
                            defaultModel = model.trim(),
                            defaultSendMode = sendMode,
                            defaultBatchSize = batchSize,
                            configured = true,
                            apiKey = apiKey.trim().takeIf { it.isNotBlank() },
                        ),
                    )
                    editingNewProfile = false
                    selectedSettingsId = cleanSettingsName
                },
                enabled = actionsEnabled && canSaveProvider,
                modifier = Modifier.fillMaxWidth(),
                shape = elementShape,
            ) {
                Text(if (editingNewProfile) "创建模型配置" else "保存模型配置")
            }
            if (canDeleteProvider) {
                OutlinedButton(
                    onClick = { onDeleteSettings(selectedSettings.settingsId) },
                    enabled = actionsEnabled,
                    modifier = Modifier.fillMaxWidth(),
                    shape = elementShape,
                ) {
                    Text("删除模型配置")
                }
            }
        }
    }
}

internal fun modelProviderKindLabel(kind: String, fallback: String): String =
    when (kind.trim().lowercase()) {
        "", "none" -> "\u672a\u914d\u7f6e"
        "openai" -> "OpenAI"
        "custom" -> "\u81ea\u5b9a\u4e49"
        "mock", "local_stub" -> "\u672c\u5730\u5206\u6790"
        else -> fallback
            .takeUnless(::isPlaceholderModelProviderLabel)
            ?.ifBlank { kind }
            ?: "模型服务"
    }

internal fun modelSendModeLabel(mode: String): String =
    when (mode.trim().lowercase()) {
        "preview_only" -> "\u4ec5\u53d1\u9001\u9884\u89c8"
        "detail_image" -> "\u53d1\u9001\u5927\u56fe"
        else -> mode
    }

internal fun modelProviderOptionLabel(settings: ModelProviderSettingsUi): String =
    listOf(settings.providerLabel, settings.defaultModel)
        .map { it.trim() }
        .filter { it.isNotBlank() && !isPlaceholderModelProviderLabel(it) }
        .joinToString(" · ")
        .ifBlank { settings.settingsId }

internal fun isPlaceholderModelProviderLabel(value: String): Boolean {
    val normalized = value.trim()
    return normalized.equals("Model provider", ignoreCase = true) || normalized == "模型服务"
}

@Composable
private fun BatchSizeToggle(
    batchSize: Int,
    onBatchSizeChange: (Int) -> Unit,
) {
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        listOf(1, 2, 4, 8).forEach { value ->
            OutlinedButton(
                onClick = { onBatchSizeChange(value) },
                modifier = Modifier
                    .height(30.dp)
                    .defaultMinSize(minWidth = 1.dp, minHeight = 1.dp),
                border = BorderStroke(1.dp, if (batchSize == value) ElementBlue else ElementBorder),
                colors = ButtonDefaults.outlinedButtonColors(
                    containerColor = if (batchSize == value) ElementBlue else ElementControlSurface,
                    contentColor = if (batchSize == value) ElementOnAccent else MaterialTheme.colorScheme.onSurfaceVariant,
                ),
                shape = elementShape,
                contentPadding = PaddingValues(horizontal = 9.dp, vertical = 0.dp),
            ) {
                Text(value.toString(), fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            }
        }
    }
}

package com.cameraconnector.app.core

import org.json.JSONArray
import org.json.JSONObject
internal fun dashboardOutputLabel(paths: JSONObject?, receiverSettings: JSONObject?): String =
    jsonStringOrNull(paths, "output_dir")
        ?: jsonStringOrNull(receiverSettings, "output_dir")
        ?: "应用私有目录"

internal fun jsonStringOrNull(value: JSONObject?, key: String): String? =
    value
        ?.takeIf { it.has(key) && !it.isNull(key) }
        ?.optString(key)
        ?.trim()
        ?.takeIf { it.isNotBlank() && !it.equals("null", ignoreCase = true) }

internal fun mapProjectSummary(value: JSONObject): ProjectSummary {
    val status = value.optString("status")
    val id = value.optString("project_id")
    val active = status.equals("Active", ignoreCase = true)
    val archived = status.equals("Archived", ignoreCase = true)
    val capabilities = value.optJSONObject("capabilities")
    return ProjectSummary(
        id = id,
        name = value.optString("name"),
        slug = value.optString("slug"),
        status = status,
        createdAtMs = value.optLong("created_at_ms"),
        updatedAtMs = value.optLong("updated_at_ms"),
        canBeActiveProject = capabilities?.optBoolean("can_be_active_project", active) ?: active,
        canArchive = capabilities?.optBoolean("can_archive", active) ?: active,
        canRename = capabilities?.optBoolean("can_rename", true) ?: true,
        canRestore = capabilities?.optBoolean("can_restore", archived) ?: archived,
        canAcceptMovedGroups = capabilities?.optBoolean("can_accept_moved_groups", active) ?: active,
    )
}

internal fun projectAssetStableId(groupId: String, primaryAssetId: String): String =
    groupId.ifBlank { primaryAssetId }

internal fun mapProjectAssets(assets: JSONObject?): List<ProjectAsset> {
    val groups = assets?.optJSONArray("groups") ?: return emptyList()
    return buildList {
        for (index in 0 until groups.length()) {
            val group = groups.optJSONObject(index) ?: continue
            val primary = group.optJSONObject("primary") ?: continue
            val raw = group.optJSONObject("raw")
            val jpeg = group.optJSONObject("jpeg")
            val video = group.optJSONObject("video")
            add(
                ProjectAsset(
                    id = projectAssetStableId(
                        group.optString("group_id"),
                        primary.optString("id"),
                    ),
                    groupKey = group.optString("group_key")
                        .ifBlank { primary.optString("id") },
                    displayPath = primary.assetDisplayPath(),
                    format = primary.optString("format"),
                    receivedAt = primary.optLong("received_time_ms").toString(),
                    username = primary.optString("username").takeIf { it.isNotBlank() },
                    displaySource = primary.optString("display_source").takeIf { it.isNotBlank() },
                    originalPath = primary.optString("original_path").takeIf { it.isNotBlank() },
                    sizeBytes = primary.optLong("size_bytes").takeIf { !primary.isNull("size_bytes") },
                    previewLocation = jpeg?.assetStorageLocation()
                        ?: primary.assetStorageLocation(),
                    rawPath = raw?.assetDisplayPath(),
                    jpegPath = jpeg?.assetDisplayPath(),
                    videoPath = video?.assetDisplayPath(),
                    hasRaw = raw != null,
                    hasJpeg = jpeg != null || primary.optString("format").equals("Jpeg", ignoreCase = true),
                    hasVideo = video != null,
                    burst = group.optJSONObject("burst")?.toProjectAssetBurst(),
                    technicalGateStatus = group.optStringOrNull("technical_gate_status"),
                    technicalDefects = group.optJSONArray("technical_defects").toProjectAssetTechnicalDefects(),
                    modelStatus = group.optStringOrNull("model_status"),
                    modelScore = group.optIntOrNull("model_score"),
                    modelTier = group.optStringOrNull("model_tier"),
                    modelEvaluatorKind = group.optStringOrNull("model_evaluator_kind"),
                    modelSummary = group.optStringOrNull("model_summary"),
                    isModelSelect = group.optBoolean("is_model_select", false),
                    userMarks = mapProjectAssetUserMarks(
                        group.optJSONObject("user_marks"),
                        favoriteOverride = group.optBooleanOrNull("is_favorite"),
                        markedOverride = group.optBooleanOrNull("is_flagged"),
                    ),
                    guestMark = mapGuestMark(group.optStringOrNull("guest_mark")),
                ),
            )
        }
    }
}

internal fun mapLanShareSession(value: JSONObject): LanShareSessionUi =
    LanShareSessionUi(
        shareId = value.optString("share_id"),
        projectId = value.optString("project_id"),
        token = value.optString("token"),
        title = value.optStringOrNull("title"),
        active = value.optBoolean("active", false),
    )

internal fun mapLanShareSessionOrNull(value: JSONObject): LanShareSessionUi? =
    if (value.has("value") && value.isNull("value")) {
        null
    } else {
        mapLanShareSession(value)
    }

internal fun mapGuestMarkFromPatchResult(value: JSONObject): GuestMark? =
    if (value.has("value") && value.isNull("value")) {
        null
    } else {
        mapGuestMark(value.optStringOrNull("guest_mark"))
    }

internal fun mapGuestMark(value: String?): GuestMark? =
    when (value?.trim()?.lowercase()) {
        GuestMark.Favorite.wireName -> GuestMark.Favorite
        GuestMark.Marked.wireName -> GuestMark.Marked
        GuestMark.Reject.wireName -> GuestMark.Reject
        else -> null
    }

internal fun mapProjectAssetUserMarks(
    value: JSONObject?,
    favoriteOverride: Boolean? = null,
    markedOverride: Boolean? = null,
): ProjectAssetUserMarks =
    ProjectAssetUserMarks(
        favorite = favoriteOverride ?: value?.optBoolean("favorite") ?: false,
        marked = markedOverride ?: value?.optBoolean("marked") ?: false,
    )

private fun JSONArray?.toProjectAssetTechnicalDefects(): List<ProjectAssetTechnicalDefect> {
    if (this == null) {
        return emptyList()
    }
    return buildList {
        for (index in 0 until length()) {
            val item = optJSONObject(index) ?: continue
            add(
                ProjectAssetTechnicalDefect(
                    defectType = item.optString("defect_type"),
                    severity = item.optString("severity"),
                    confidence = item.optDoubleOrDefault("confidence", 0.0),
                    reason = item.optStringOrNull("reason"),
                ),
            )
        }
    }
}

private fun JSONObject.toProjectAssetBurst(): ProjectAssetBurst =
    ProjectAssetBurst(
        burstGroupId = optString("burst_group_id"),
        memberCount = optIntOrNull("member_count") ?: 0,
        recommendationStatus = optStringOrNull("recommendation_status"),
        bestAssetGroupId = optStringOrNull("best_asset_group_id"),
        bestScore = optDoubleOrNull("best_score"),
    )

internal fun mapModelProviderSettings(value: JSONObject): ModelProviderSettingsUi =
    ModelProviderSettingsUi(
        settingsId = value.optString("settings_id").ifBlank { "global" },
        providerKind = value.optString("provider_kind").ifBlank { "none" },
        providerLabel = value.optString("provider_label").ifBlank { "模型服务" },
        baseUrl = value.optString("base_url"),
        defaultModel = value.optString("default_model"),
        defaultMaxImageSide = value.optInt("default_max_image_side", 1536),
        defaultSendMode = value.optString("default_send_mode").ifBlank { "preview_only" },
        defaultBatchSize = value.optInt("default_batch_size", 1).coerceAtLeast(1),
        configured = value.optBoolean("configured", false),
        apiKey = null,
        apiKeyConfigured = value.optBoolean("api_key_configured", false),
        keyAlias = jsonStringOrNull(value, "key_alias"),
        updatedAtMs = value.optLong("updated_at_ms"),
    )

internal fun mapModelProviderSettingsList(settings: JSONArray?): List<ModelProviderSettingsUi> {
    if (settings == null) {
        return emptyList()
    }
    return buildList {
        for (index in 0 until settings.length()) {
            settings.optJSONObject(index)?.let { add(mapModelProviderSettings(it)) }
        }
    }
}

internal fun ModelProviderSettingsUi.toModelProviderSettingsJson(): JSONObject =
    JSONObject()
        .put("settings_id", settingsId.ifBlank { "global" })
        .put("provider_kind", providerKind.ifBlank { "none" })
        .put("provider_label", providerLabel)
        .put("base_url", baseUrl)
        .put("default_model", defaultModel)
        .put("default_max_image_side", defaultMaxImageSide)
        .put("default_send_mode", defaultSendMode.ifBlank { "preview_only" })
        .put("default_batch_size", defaultBatchSize.coerceAtLeast(1))
        .put("configured", configured)
        .put("key_alias", keyAlias ?: JSONObject.NULL)
        .put("updated_at_ms", updatedAtMs)
        .also { json ->
            apiKey?.let { json.put("api_key", it) }
        }

internal fun mapProjectEvaluationSettings(value: JSONObject): ProjectEvaluationSettingsUi =
    ProjectEvaluationSettingsUi(
        projectId = value.optString("project_id"),
        autoEvaluateOnUpload = value.optBoolean("auto_evaluate_on_upload", false),
        autoBurstRecommendationEnabled = value.optBoolean("auto_burst_recommendation_enabled", true),
        projectRecommendationMode = "manual",
        promptPackId = jsonStringOrNull(value, "prompt_pack_id"),
        modelProviderSettingsId = jsonStringOrNull(value, "model_provider_settings_id"),
        sceneProfile = value.optString("scene_profile").ifBlank { "general" },
        cvPolicy = value.optString("cv_policy").ifBlank { "standard" },
        cvPolicyOverrides = value.optJSONObject("cv_policy_overrides")?.toTechnicalAssessmentPolicyUi(),
        allowRiskyModelSelects = value.optBoolean("allow_risky_model_selects", false),
        maxImageSide = value.optIntOrNull("max_image_side"),
        batchSize = value.optIntOrNull("batch_size"),
        updatedAtMs = value.optLong("updated_at_ms"),
    )

internal fun ProjectEvaluationSettingsUi.toProjectEvaluationSettingsJson(): JSONObject =
    JSONObject()
        .put("project_id", projectId)
        .put("auto_evaluate_on_upload", autoEvaluateOnUpload)
        .put("auto_burst_recommendation_enabled", autoBurstRecommendationEnabled)
        .put("project_recommendation_mode", "manual")
        .put("prompt_pack_id", promptPackId ?: JSONObject.NULL)
        .put("model_provider_settings_id", modelProviderSettingsId ?: JSONObject.NULL)
        .put("scene_profile", sceneProfile.ifBlank { "general" })
        .put("cv_policy", cvPolicy.ifBlank { "standard" })
        .put("cv_policy_overrides", cvPolicyOverrides?.toJson() ?: JSONObject.NULL)
        .put("allow_risky_model_selects", allowRiskyModelSelects)
        .put("max_image_side", maxImageSide ?: JSONObject.NULL)
        .put("batch_size", batchSize ?: JSONObject.NULL)
        .put("updated_at_ms", updatedAtMs)

private fun JSONObject.toTechnicalAssessmentPolicyUi(): TechnicalAssessmentPolicyUi =
    TechnicalAssessmentPolicyUi(
        blurSevereEdgeThreshold = optDouble("blur_severe_edge_threshold"),
        blurSevereFrequencyThreshold = optDouble("blur_severe_frequency_threshold"),
        blurHighEdgeThreshold = optDouble("blur_high_edge_threshold"),
        blurHighFrequencyThreshold = optDouble("blur_high_frequency_threshold"),
        highlightClipThreshold = optInt("highlight_clip_threshold"),
        shadowClipThreshold = optInt("shadow_clip_threshold"),
        clippingHighRatio = optDouble("clipping_high_ratio"),
        clippingHighConnectedRatio = optDouble("clipping_high_connected_ratio"),
        clippingSevereRatio = optDouble("clipping_severe_ratio"),
        clippingSevereConnectedRatio = optDouble("clipping_severe_connected_ratio"),
        colorCastHighThreshold = optDouble("color_cast_high_threshold"),
        colorCastSevereThreshold = optDouble("color_cast_severe_threshold"),
        faceEyeOpenWarnThreshold = optDouble("face_eye_open_warn_threshold", 0.35),
        faceExposureWarnRatio = optDouble("face_exposure_warn_ratio", 0.25),
        faceColorCastWarnThreshold = optDouble("face_color_cast_warn_threshold", 0.42),
    )

private fun TechnicalAssessmentPolicyUi.toJson(): JSONObject =
    JSONObject()
        .put("blur_severe_edge_threshold", blurSevereEdgeThreshold)
        .put("blur_severe_frequency_threshold", blurSevereFrequencyThreshold)
        .put("blur_high_edge_threshold", blurHighEdgeThreshold)
        .put("blur_high_frequency_threshold", blurHighFrequencyThreshold)
        .put("highlight_clip_threshold", highlightClipThreshold)
        .put("shadow_clip_threshold", shadowClipThreshold)
        .put("clipping_high_ratio", clippingHighRatio)
        .put("clipping_high_connected_ratio", clippingHighConnectedRatio)
        .put("clipping_severe_ratio", clippingSevereRatio)
        .put("clipping_severe_connected_ratio", clippingSevereConnectedRatio)
        .put("color_cast_high_threshold", colorCastHighThreshold)
        .put("color_cast_severe_threshold", colorCastSevereThreshold)
        .put("face_eye_open_warn_threshold", faceEyeOpenWarnThreshold)
        .put("face_exposure_warn_ratio", faceExposureWarnRatio)
        .put("face_color_cast_warn_threshold", faceColorCastWarnThreshold)

internal fun mapPromptPacks(profiles: JSONArray?): List<PromptPackUi> {
    if (profiles == null) {
        return emptyList()
    }
    return buildList {
        for (index in 0 until profiles.length()) {
            profiles.optJSONObject(index)?.let { add(mapPromptPack(it)) }
        }
    }
}

internal fun mapPromptPack(value: JSONObject): PromptPackUi {
    val activePromptText = jsonStringOrNull(value, "prompt_text")
    val promptMarkdown = activePromptText
        ?.trim()
        ?.takeIf { it.isNotBlank() }
    return PromptPackUi(
        promptPackId = value.optString("prompt_pack_id"),
        distributionFolder = value.optString("distribution_folder").ifBlank { "user" },
        scope = if (value.optBoolean("built_in", false)) "built_in" else "user",
        projectId = null,
        name = value.optString("name"),
        styleTags = value.optJSONArray("style_tags").toStringList(),
        sceneProfile = value.optString("scene_profile").ifBlank { "general" },
        activeVersionId = jsonStringOrNull(value, "version"),
        builtIn = value.optBoolean("built_in", false),
        enabled = value.optBoolean("enabled", true),
        activePromptText = promptMarkdown,
        sharedPreference = promptMarkdown,
        evaluationInstruction = null,
        burstSelectionInstruction = null,
        projectSelectionInstruction = null,
    )
}

internal fun mapEvaluationRun(value: JSONObject): EvaluationRunUi =
    EvaluationRunUi(
        runId = value.optString("run_id"),
        projectId = value.optString("project_id"),
        runType = value.optString("run_type"),
        trigger = value.optString("trigger"),
        status = value.optString("status"),
        providerKind = value.optString("provider_kind").ifBlank { "none" },
        providerModel = value.optString("provider_model"),
        promptPackId = jsonStringOrNull(value, "prompt_pack_id"),
        promptVersionId = jsonStringOrNull(value, "prompt_pack_version"),
        promptHash = jsonStringOrNull(value, "prompt_hash"),
        errorMessage = jsonStringOrNull(value, "error_message"),
        startedAtMs = value.optLongOrNull("started_at_ms"),
        completedAtMs = value.optLongOrNull("completed_at_ms"),
        createdAtMs = value.optLong("created_at_ms"),
    )

internal fun mapSubjectAssessments(assessments: JSONArray?): List<SubjectAssessmentUi> {
    if (assessments == null) {
        return emptyList()
    }
    return buildList {
        for (index in 0 until assessments.length()) {
            assessments.optJSONObject(index)?.let { add(mapSubjectAssessment(it)) }
        }
    }
}

internal fun mapSubjectAssessment(value: JSONObject): SubjectAssessmentUi =
    SubjectAssessmentUi(
        assessmentId = value.optString("assessment_id"),
        projectId = value.optString("project_id"),
        assetGroupId = value.optString("asset_group_id"),
        subjectType = value.optString("subject_type"),
        detectorKind = value.optString("detector_kind"),
        detectorVersion = value.optString("detector_version"),
        status = value.optString("status"),
        gateStatus = value.optString("gate_status"),
        regionsJson = (value.opt("regions") ?: JSONArray()).toString(),
        signalsJson = (value.opt("signals") ?: JSONObject()).toString(),
        summary = value.optString("summary"),
        createdAtMs = value.optLong("created_at_ms"),
        updatedAtMs = value.optLong("updated_at_ms"),
    )

internal fun SubjectAssessmentUi.toSubjectAssessmentJson(): JSONObject =
    JSONObject()
        .put("assessment_id", assessmentId)
        .put("project_id", projectId)
        .put("asset_group_id", assetGroupId)
        .put("subject_type", subjectType)
        .put("detector_kind", detectorKind)
        .put("detector_version", detectorVersion)
        .put("status", status)
        .put("gate_status", gateStatus)
        .put("regions", parseJsonPayload(regionsJson, JSONArray()))
        .put("signals", parseJsonPayload(signalsJson, JSONObject()))
        .put("summary", summary)
        .put("created_at_ms", createdAtMs)
        .put("updated_at_ms", updatedAtMs)

private fun parseJsonPayload(raw: String, fallback: Any): Any =
    runCatching { JSONObject(raw) }
        .getOrElse {
            runCatching { JSONArray(raw) }.getOrDefault(fallback)
        }

internal fun projectRecommendationRunAfterGenerate(
    generateRecommendation: () -> JSONObject,
    latestRun: () -> JSONObject?,
): EvaluationRunUi {
    generateRecommendation()
    val run = latestRun()
        ?: error("Project recommendation completed without a latest run status")
    return mapEvaluationRun(run)
}

internal fun JSONObject.assetDisplayPath(): String =
    optString("virtual_display_path").ifBlank { optString("filename") }

private fun JSONObject.assetStorageLocation(): String? {
    val location = optJSONObject("storage_location") ?: return null
    return location.optString("path")
        .ifBlank { location.optString("uri") }
        .ifBlank { null }
}

private fun JSONObject.optStringOrNull(key: String): String? =
    if (has(key) && !isNull(key)) {
        optString(key)
            .trim()
            .takeIf { it.isNotBlank() && !it.equals("null", ignoreCase = true) }
    } else {
        null
    }

private fun JSONObject.optIntOrNull(key: String): Int? =
    if (has(key) && !isNull(key)) optInt(key) else null

private fun JSONObject.optLongOrNull(key: String): Long? =
    if (has(key) && !isNull(key)) optLong(key) else null

private fun JSONObject.optDoubleOrNull(key: String): Double? =
    if (has(key) && !isNull(key)) optDouble(key).takeUnless { it.isNaN() } else null

private fun JSONObject.optBooleanOrNull(key: String): Boolean? =
    if (has(key) && !isNull(key)) optBoolean(key) else null

private fun JSONArray?.toStringList(): List<String> {
    if (this == null) {
        return emptyList()
    }
    return buildList {
        for (index in 0 until length()) {
            optString(index).takeIf { it.isNotBlank() }?.let(::add)
        }
    }
}

private fun JSONObject?.optDoubleOrDefault(key: String, default: Double): Double =
    this?.takeIf { it.has(key) && !it.isNull(key) }
        ?.optDouble(key)
        ?.takeUnless { it.isNaN() }
        ?: default

internal fun mapPublishQueueState(value: JSONObject?): PublishQueueState =
    PublishQueueState(
        totalCount = value?.optInt("total_count") ?: 0,
        pendingCount = value?.optInt("pending_count") ?: 0,
        stagedCount = value?.optInt("staged_count") ?: 0,
        publishingCount = value?.optInt("publishing_count") ?: 0,
        completedCount = value?.optInt("completed_count") ?: 0,
        failedCount = value?.optInt("failed_count") ?: 0,
    )

internal fun mapGlobalAssetSummary(value: JSONObject?): GlobalAssetSummaryUi =
    GlobalAssetSummaryUi(
        photoCount = value?.optInt("photo_count") ?: 0,
        fileCount = value?.optInt("file_count") ?: 0,
        storageBytes = value?.optLong("storage_bytes") ?: 0L,
    )

internal fun mapPublishFailureTransfers(value: JSONArray?): List<TransferRow> {
    if (value == null) {
        return emptyList()
    }

    return buildList {
        for (index in 0 until value.length()) {
            val item = value.optJSONObject(index) ?: continue
            val displayPath = publishFailureDisplayPath(item)
            add(
                TransferRow(
                    id = item.optString("queue_id").ifBlank { "publish-failure-$index" },
                    status = "Failed",
                    displayPath = displayPath,
                    message = item.optString("last_error").takeIf { it.isNotBlank() },
                ),
            )
        }
    }
}

private fun publishFailureDisplayPath(item: JSONObject): String {
    val path = item.optString("original_path")
        .ifBlank { item.optString("final_filename") }
        .ifBlank { item.optString("transfer_id") }
        .ifBlank { "\u5199\u5165\u5931\u8d25" }
    val source = item.optString("display_source").takeIf { it.isNotBlank() }
    return if (source == null || path.startsWith("$source/")) {
        path
    } else {
        "$source/$path"
    }
}

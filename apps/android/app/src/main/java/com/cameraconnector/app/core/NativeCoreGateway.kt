package com.cameraconnector.app.core

import com.cameraconnector.app.service.ReceiverServiceController
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject

class NativeCoreGateway(
    private val nativeCore: NativeMobileCore,
    private val receiverServiceController: ReceiverServiceController,
) : CoreGateway, AutoCloseable {
    private val gatewayScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val projects = MutableStateFlow(
        ProjectState(projects = emptyList(), activeProjectId = null),
    )
    private val dashboard = MutableStateFlow(emptyDashboard())

    init {
        pollDashboard()
    }

    override fun observeDashboard(): Flow<DashboardState> = dashboard.asStateFlow()

    override fun observeProjects(): Flow<ProjectState> = projects.asStateFlow()

    override suspend fun loadInbox(query: InboxAssetQuery, offset: Int, limit: Int): List<InboxAsset> =
        withContext(Dispatchers.IO) {
            val projectId = projects.value.activeProjectId
                ?: loadProjects().activeProjectId
                ?: return@withContext emptyList()
            mapInboxAssets(
                nativeCore.projectAssetGroupPageJson(
                    projectId = projectId,
                    query = query,
                    offset = offset,
                    limit = limit,
                ),
            )
        }

    override suspend fun loadSelects(
        projectId: String?,
        strategyProfileId: String?,
        offset: Int,
        limit: Int,
    ): List<InboxAsset> =
        withContext(Dispatchers.IO) {
            val resolvedProjectId = projectId
                ?: projects.value.activeProjectId
                ?: loadProjects().activeProjectId
                ?: return@withContext emptyList()
            mapInboxAssets(
                nativeCore.projectSelectsAssetGroupPageJson(
                    projectId = resolvedProjectId,
                    strategyProfileId = strategyProfileId,
                    offset = offset,
                    limit = limit,
                ),
            )
        }

    suspend fun refresh() {
        val (nextProjects, nextDashboard) = withContext(Dispatchers.IO) {
            val loadedProjects = loadProjects()
            loadedProjects to loadDashboard(loadedProjects.activeProjectId)
        }
        projects.value = nextProjects
        dashboard.value = nextDashboard
    }

    override suspend fun createProject(name: String): ProjectSummary {
        val project = withContext(Dispatchers.IO) {
            val created = mapProjectSummary(nativeCore.createProject(name))
            nativeCore.setActiveProject(created.id)
            created
        }
        refresh()
        return project
    }

    override suspend fun setActiveProject(projectId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.setActiveProject(projectId)
        }
        refresh()
    }

    override suspend fun renameProject(projectId: String, name: String) {
        withContext(Dispatchers.IO) {
            nativeCore.renameProject(projectId, name)
        }
        refresh()
    }

    override suspend fun archiveProject(projectId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.archiveProject(projectId)
        }
        refresh()
    }

    override suspend fun restoreProject(projectId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.restoreProject(projectId)
        }
        refresh()
    }

    override suspend fun moveProjectGroup(
        sourceProjectId: String,
        groupId: String,
        targetProjectId: String,
    ) {
        withContext(Dispatchers.IO) {
            nativeCore.moveProjectGroup(sourceProjectId, groupId, targetProjectId)
        }
        refresh()
    }

    override suspend fun startReceiver() {
        receiverServiceController.startReceiver()
        refreshAfterServiceCommand()
    }

    override suspend fun stopReceiver() {
        receiverServiceController.stopReceiver()
        refreshAfterServiceCommand()
    }

    override suspend fun saveReceiverSettings(settings: ReceiverSettings) {
        withContext(Dispatchers.IO) {
            nativeCore.saveReceiverSettings(settings)
        }
        refresh()
    }

    override suspend fun saveDeviceAccount(account: DeviceAccount, password: String?) {
        withContext(Dispatchers.IO) {
            nativeCore.saveDeviceAccount(account, password = password?.takeIf { it.isNotBlank() })
        }
        refresh()
    }

    override suspend fun removeDeviceAccount(username: String) {
        withContext(Dispatchers.IO) {
            nativeCore.removeDeviceAccount(username)
        }
        refresh()
    }

    override suspend fun retryFailedPublishes() {
        withContext(Dispatchers.IO) {
            val projectId = loadProjects().activeProjectId
                ?: return@withContext
            nativeCore.releaseFailedPublishRetries(projectId)
            receiverServiceController.retryFailedPublishes()
        }
        refresh()
    }

    override suspend fun loadStrategyProfiles(): List<StrategyProfileUi> =
        withContext(Dispatchers.IO) {
            mapStrategyProfiles(nativeCore.strategyProfiles())
        }

    override suspend fun saveStrategyProfile(profile: StrategyProfileUi): StrategyProfileUi =
        withContext(Dispatchers.IO) {
            mapStrategyProfile(
                nativeCore.saveStrategyProfile(profile.toStrategyProfileJson().toString()),
            )
        }

    override suspend fun loadReviewQueueSummary(
        projectId: String?,
        strategyProfileId: String?,
    ): ReviewQueueSummary =
        withContext(Dispatchers.IO) {
            val resolvedProjectId = projectId
                ?: projects.value.activeProjectId
                ?: loadProjects().activeProjectId
                ?: return@withContext emptyReviewQueueSummary(strategyProfileId = strategyProfileId ?: "general")
            mapReviewQueueSummary(
                nativeCore.reviewQueueSummary(
                    projectId = resolvedProjectId,
                    strategyProfileId = strategyProfileId,
                ),
            )
        }

    override suspend fun acceptRecommendedBest(burstGroupId: String, strategyProfileId: String?) {
        withContext(Dispatchers.IO) {
            nativeCore.acceptRecommendedBest(burstGroupId, strategyProfileId)
        }
        refresh()
    }

    override suspend fun markBurstNeedsReview(burstGroupId: String, strategyProfileId: String?) {
        withContext(Dispatchers.IO) {
            nativeCore.markBurstNeedsReview(burstGroupId, strategyProfileId)
        }
        refresh()
    }

    override suspend fun restoreAutomaticRecommendation(
        burstGroupId: String,
        strategyProfileId: String?,
    ) {
        withContext(Dispatchers.IO) {
            nativeCore.restoreAutomaticRecommendation(burstGroupId, strategyProfileId)
        }
        refresh()
    }

    override suspend fun clearRecommendation(burstGroupId: String, strategyProfileId: String?) {
        withContext(Dispatchers.IO) {
            nativeCore.clearRecommendation(burstGroupId, strategyProfileId)
        }
        refresh()
    }

    override suspend fun keepAllCandidates(burstGroupId: String, strategyProfileId: String?) {
        withContext(Dispatchers.IO) {
            nativeCore.keepAllCandidates(burstGroupId, strategyProfileId)
        }
        refresh()
    }

    override suspend fun hideLowScoreCandidates(burstGroupId: String, strategyProfileId: String?) {
        withContext(Dispatchers.IO) {
            nativeCore.hideLowScoreCandidates(burstGroupId, strategyProfileId)
        }
        refresh()
    }

    override suspend fun overrideRecommendedBest(
        burstGroupId: String,
        bestAssetGroupId: String,
        strategyProfileId: String?,
    ) {
        withContext(Dispatchers.IO) {
            nativeCore.overrideRecommendedBest(burstGroupId, bestAssetGroupId, strategyProfileId)
        }
        refresh()
    }

    override suspend fun splitBurstMember(burstGroupId: String, memberGroupId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.splitBurstMember(burstGroupId, memberGroupId)
        }
        refresh()
    }

    override suspend fun mergeBurstMember(targetBurstGroupId: String, memberGroupId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.mergeBurstMember(targetBurstGroupId, memberGroupId)
        }
        refresh()
    }

    override fun close() {
        gatewayScope.cancel()
        nativeCore.close()
    }

    private fun loadDashboard(activeProjectId: String?): DashboardState {
        val projectId = activeProjectId
            ?: return emptyDashboard()
        val dashboardJson = nativeCore.projectDashboardJson(
            projectId,
            offset = 0,
            limit = CONTINUOUS_INBOX_LIMIT,
        )
        return mapDashboard(dashboardJson)
    }

    private fun loadProjects(): ProjectState {
        val activeProjectId = nativeCore.activeProject()
            ?.optString("project_id")
            ?.takeIf { it.isNotBlank() }
        val projectList = nativeCore.listProjects()
        return ProjectState(
            projects = buildList {
                for (index in 0 until projectList.length()) {
                    val item = projectList.optJSONObject(index) ?: continue
                    add(mapProjectSummary(item))
                }
            },
            activeProjectId = activeProjectId,
        )
    }

    private suspend fun refreshAfterServiceCommand() {
        delay(250)
        refresh()
    }

    private fun pollDashboard() {
        gatewayScope.launch {
            while (isActive) {
                runCatching {
                    val loadedProjects = loadProjects()
                    projects.value = loadedProjects
                    dashboard.value = loadDashboard(loadedProjects.activeProjectId)
                }
                delay(DASHBOARD_POLL_INTERVAL_MS)
            }
        }
    }

    private fun mapDashboard(value: JSONObject): DashboardState {
        val receiverStatus = value.optJSONObject("receiver_status")
        val receiverSettings = value.optJSONObject("receiver_settings")
        val paths = value.optJSONObject("paths")
        val assets = value.optJSONObject("assets")
        val transfers = value.optJSONObject("transfers")
        val running = receiverStatus?.optString("phase") == "Running"
        val settingsProtocol = normalizeProtocol(receiverSettings?.optString("protocol"))
        val statusProtocol = receiverStatus?.optString("protocol")
            ?.takeIf { it.isNotBlank() && !it.equals("null", ignoreCase = true) }
            ?.let(::normalizeProtocol)
        val protocol = if (running) statusProtocol ?: settingsProtocol else settingsProtocol
        val configuredHost = jsonStringOrNull(receiverSettings, "bind_host")
            ?: DEFAULT_LISTEN_HOST
        val configuredPort = when (protocol) {
            "SFTP" -> receiverSettings?.optInt("sftp_port")?.takeIf { it in 1..65_535 } ?: 2222
            else -> receiverSettings?.optInt("ftp_port")?.takeIf { it in 1..65_535 } ?: 2121
        }
        val localAddr = receiverStatus?.optString("local_addr").orEmpty()
        val (statusHost, statusPort) = splitHostAndPort(localAddr, defaultPort = configuredPort)
        val (host, port) = if (localAddr.isBlank() || localAddr.equals("null", ignoreCase = true)) {
            configuredHost to configuredPort
        } else {
            statusHost to statusPort
        }

        val transferRows = mapTransfers(transfers, assets, value.optJSONArray("recent_failures")) +
            mapPublishFailureTransfers(value.optJSONArray("recent_publish_failures"))

        return DashboardState(
            receiver = ReceiverState(
                running = running,
                phase = receiverStatus?.optString("phase").orEmpty()
                    .ifBlank { "Unknown" },
                protocol = protocol,
                authMode = receiverStatus?.optString("auth_mode").orEmpty()
                    .ifBlank { "Unknown" },
                accountCount = receiverStatus?.optInt("account_count") ?: 0,
                host = host,
                port = port,
                outputLabel = dashboardOutputLabel(paths, receiverSettings),
                message = receiverStatus?.takeIf { !it.isNull("message") }
                    ?.optString("message")
                    .orEmpty()
                    .takeIf { it.isNotBlank() },
            ),
            accounts = mapAccounts(value),
            inbox = mapInboxAssets(assets),
            transfers = transferRows,
            publishQueue = mapPublishQueueState(value.optJSONObject("publish_queue")),
        )
    }

    private fun splitHostAndPort(localAddr: String, defaultPort: Int): Pair<String, Int> {
        val trimmed = localAddr.trim()
        if (trimmed.isBlank() || trimmed.equals("null", ignoreCase = true)) {
            return DEFAULT_LISTEN_HOST to defaultPort
        }

        if (trimmed.startsWith("[")) {
            val hostEnd = trimmed.indexOf(']')
            val host = trimmed.substring(0, hostEnd + 1).takeIf { hostEnd > 0 } ?: trimmed
            val port = trimmed.substringAfter("]:", "")
                .toIntOrNull()
                ?: defaultPort
            return host to port
        }

        val splitAt = trimmed.lastIndexOf(':')
        if (splitAt <= 0 || splitAt == trimmed.lastIndex || trimmed.indexOf(':') != splitAt) {
            return trimmed to defaultPort
        }

        return trimmed.substring(0, splitAt) to
            (trimmed.substring(splitAt + 1).toIntOrNull() ?: defaultPort)
    }

    private fun normalizeProtocol(value: String?): String {
        val protocol = value.orEmpty().trim().uppercase()
        return if (protocol.isBlank() || protocol == "NULL") {
            "FTP"
        } else {
            protocol
        }
    }

    private fun mapAccounts(value: JSONObject): List<DeviceAccount> {
        val accounts = value.optJSONArray("accounts") ?: return emptyList()
        return buildList {
            for (index in 0 until accounts.length()) {
                val item = accounts.optJSONObject(index) ?: continue
                add(
                    DeviceAccount(
                        username = item.optString("username"),
                        deviceName = item.optString("device_name"),
                        passwordConfigured = item.optBoolean("password_configured"),
                        latestIp = item.optString("last_remote_addr").takeIf { it.isNotBlank() },
                        latestPort = item.optInt("last_remote_port")
                            .takeIf { !item.isNull("last_remote_port") },
                        activeConnections = item.optInt("active_connections"),
                        lastSeenAtMs = item.optLong("last_seen_at_ms")
                            .takeIf { !item.isNull("last_seen_at_ms") },
                        lastDisconnectedAtMs = item.optLong("last_disconnected_at_ms")
                            .takeIf { !item.isNull("last_disconnected_at_ms") },
                        online = item.optBoolean("online"),
                    ),
                )
            }
        }
    }

    private fun mapTransfers(
        transfers: JSONObject?,
        assets: JSONObject?,
        recentFailures: JSONArray?,
    ): List<TransferRow> {
        if (transfers == null && assets == null && recentFailures == null) {
            return emptyList()
        }

        return buildList {
            transfers?.let {
                add(
                    TransferRow(
                        id = "summary",
                        status = "completed=${it.optInt("completed_count")}",
                        displayPath = "failed=${it.optInt("failed_count")}",
                        message = "total=${it.optInt("total_count")}",
                    ),
                )
            }

            val groups = assets?.optJSONArray("groups")
            if (groups != null) {
                for (index in 0 until groups.length()) {
                    val group = groups.optJSONObject(index) ?: continue
                    val seenIds = mutableSetOf<String>()
                    listOf("primary", "jpeg", "raw", "video")
                        .mapNotNull { group.optJSONObject(it) }
                        .forEach { asset ->
                            val id = asset.optString("id")
                            if (id.isBlank() || !seenIds.add(id)) {
                                return@forEach
                            }
                            val displayPath = asset.assetDisplayPath()
                            if (displayPath.isBlank()) {
                                return@forEach
                            }
                            add(
                                TransferRow(
                                    id = id,
                                    status = "Completed",
                                    displayPath = displayPath,
                                    message = asset.optString("size_bytes").takeIf { it.isNotBlank() }
                                        ?.let { "$it bytes" },
                                ),
                            )
                        }
                }
            }

            if (recentFailures != null) {
                for (index in 0 until recentFailures.length()) {
                    val item = recentFailures.optJSONObject(index) ?: continue
                    val record = item.optJSONObject("record")
                    add(
                        TransferRow(
                            id = record?.optString("transfer_id").orEmpty()
                                .ifBlank { "failure-$index" },
                            status = record?.optString("status").orEmpty()
                                .ifBlank { "Failed" },
                            displayPath = item.optString("virtual_display_path")
                                .ifBlank { record?.optString("original_path").orEmpty() }
                                .ifBlank { record?.optString("final_filename").orEmpty() }
                                .ifBlank { "Failed transfer" },
                            message = record?.optString("error").orEmpty()
                                .takeIf { it.isNotBlank() },
                        ),
                    )
                }
            }
        }
    }

    private fun emptyDashboard(): DashboardState =
        DashboardState(
            receiver = ReceiverState(
                running = false,
                phase = "Unknown",
                protocol = "FTP",
                authMode = "Unknown",
                accountCount = 0,
                host = DEFAULT_LISTEN_HOST,
                port = 2121,
                outputLabel = "选择收件箱文件夹",
                message = null,
            ),
            accounts = emptyList(),
            inbox = emptyList(),
            transfers = emptyList(),
        )

    private companion object {
        const val DASHBOARD_POLL_INTERVAL_MS = 2_000L
        const val CONTINUOUS_INBOX_LIMIT = 2_000
    }
}

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

internal fun inboxStableId(groupId: String, primaryAssetId: String): String =
    groupId.ifBlank { primaryAssetId }

internal fun mapInboxAssets(assets: JSONObject?): List<InboxAsset> {
    val groups = assets?.optJSONArray("groups") ?: return emptyList()
    return buildList {
        for (index in 0 until groups.length()) {
            val group = groups.optJSONObject(index) ?: continue
            val primary = group.optJSONObject("primary") ?: continue
            val raw = group.optJSONObject("raw")
            val jpeg = group.optJSONObject("jpeg")
            val video = group.optJSONObject("video")
            add(
                InboxAsset(
                    id = inboxStableId(
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
                    burst = group.optJSONObject("burst")?.toInboxAssetBurst(),
                    quality = group.optJSONObject("quality")?.toInboxAssetQuality(),
                ),
            )
        }
    }
}

private fun JSONObject.toInboxAssetBurst(): InboxAssetBurst =
    InboxAssetBurst(
        burstGroupId = optString("burst_group_id"),
        memberCount = optIntOrNull("member_count") ?: 0,
        memberRank = optIntOrNull("member_rank"),
        recommendationStatus = optStringOrNull("recommendation_status"),
        bestAssetGroupId = optStringOrNull("best_asset_group_id"),
        bestScore = optDoubleOrNull("best_score"),
    )

private fun JSONObject.toInboxAssetQuality(): InboxAssetQuality =
    InboxAssetQuality(
        overall = optDoubleOrNull("overall"),
        analysisStatus = optStringOrNull("analysis_status"),
        scorerVersion = optStringOrNull("scorer_version"),
        primaryReason = optStringOrNull("primary_reason"),
        analyzedAtMs = optLongOrNull("analyzed_at_ms"),
        sharpness = optDoubleOrNull("sharpness"),
        exposure = optDoubleOrNull("exposure"),
        highlightClippingPenalty = optDoubleOrNull("highlight_clipping_penalty"),
        shadowClippingPenalty = optDoubleOrNull("shadow_clipping_penalty"),
        composition = optDoubleOrNull("composition"),
        compositionConfidence = optDoubleOrNull("composition_confidence"),
    )

internal fun mapStrategyProfiles(profiles: JSONArray?): List<StrategyProfileUi> {
    if (profiles == null) {
        return emptyList()
    }
    return buildList {
        for (index in 0 until profiles.length()) {
            profiles.optJSONObject(index)?.let { add(mapStrategyProfile(it)) }
        }
    }
}

internal fun mapStrategyProfile(value: JSONObject): StrategyProfileUi =
    StrategyProfileUi(
        profileId = value.optString("profile_id"),
        name = value.optString("name"),
        builtIn = value.optBoolean("built_in"),
        strategyVersion = value.optString("strategy_version").ifBlank { "strategy-v1" },
        burstWindowMs = value.optLong("burst_window_ms"),
        minGroupSize = value.optInt("min_group_size"),
        weights = mapStrategyWeights(value.optJSONObject("weights")),
        rejectIfSharpnessBelow = value.optDoubleOrDefault("reject_if_sharpness_below", 0.25),
        flagIfOverallBelow = value.optDoubleOrDefault("flag_if_overall_below", 0.40),
        nearDuplicateSimilarityAbove = value.optDoubleOrDefault("near_duplicate_similarity_above", 0.92),
        maxLlmCandidatesPerGroup = value.optInt("max_llm_candidates_per_group", 5),
        autoDelete = value.optBoolean("auto_delete", false),
        autoHideLowScore = value.optBoolean("auto_hide_low_score", false),
        markBest = value.optBoolean("mark_best", true),
        keepRawPairs = value.optBoolean("keep_raw_pairs", true),
        llmEnabled = value.optBoolean("llm_enabled", false),
        updatedAtMs = value.optLong("updated_at_ms"),
    )

private fun mapStrategyWeights(value: JSONObject?): StrategyWeightsUi =
    StrategyWeightsUi(
        sharpness = value.optDoubleOrDefault("sharpness", 0.40),
        exposure = value.optDoubleOrDefault("exposure", 0.22),
        composition = value.optDoubleOrDefault("composition", 0.12),
        highlightClippingPenalty = value.optDoubleOrDefault("highlight_clipping_penalty", -0.14),
        shadowClippingPenalty = value.optDoubleOrDefault("shadow_clipping_penalty", -0.08),
        diversity = value.optDoubleOrDefault("diversity", 0.04),
    )

internal fun StrategyProfileUi.toStrategyProfileJson(): JSONObject =
    JSONObject()
        .put("profile_id", profileId)
        .put("name", name)
        .put("built_in", builtIn)
        .put("strategy_version", strategyVersion)
        .put("burst_window_ms", burstWindowMs)
        .put("min_group_size", minGroupSize)
        .put(
            "weights",
            JSONObject()
                .put("sharpness", weights.sharpness)
                .put("exposure", weights.exposure)
                .put("composition", weights.composition)
                .put("highlight_clipping_penalty", weights.highlightClippingPenalty)
                .put("shadow_clipping_penalty", weights.shadowClippingPenalty)
                .put("diversity", weights.diversity),
        )
        .put("reject_if_sharpness_below", rejectIfSharpnessBelow)
        .put("flag_if_overall_below", flagIfOverallBelow)
        .put("near_duplicate_similarity_above", nearDuplicateSimilarityAbove)
        .put("max_llm_candidates_per_group", maxLlmCandidatesPerGroup)
        .put("auto_delete", autoDelete)
        .put("auto_hide_low_score", autoHideLowScore)
        .put("mark_best", markBest)
        .put("keep_raw_pairs", keepRawPairs)
        .put("llm_enabled", llmEnabled)
        .put("updated_at_ms", updatedAtMs)

internal fun mapReviewQueueSummary(value: JSONObject?): ReviewQueueSummary {
    if (value == null) {
        return emptyReviewQueueSummary()
    }
    val queues = value.optJSONArray("queues")
    return ReviewQueueSummary(
        projectId = value.optString("project_id"),
        strategyProfileId = value.optString("strategy_profile_id").ifBlank { "general" },
        totalUnits = value.optInt("total_units"),
        pendingCount = value.optInt("pending_count"),
        unconfirmedBestCount = value.optInt("unconfirmed_best_count"),
        needsReviewCount = value.optInt("needs_review_count"),
        lowScoreCandidateCount = value.optInt("low_score_candidate_count"),
        nearDuplicateCount = value.optInt("near_duplicate_count"),
        unsupportedCount = value.optInt("unsupported_count"),
        userOverriddenCount = value.optInt("user_overridden_count"),
        queues = buildList {
            if (queues != null) {
                for (index in 0 until queues.length()) {
                    val queue = queues.optJSONObject(index) ?: continue
                    add(
                        ReviewQueueCount(
                            queue = queue.optString("queue"),
                            count = queue.optInt("count"),
                        ),
                    )
                }
            }
        },
    )
}

private fun JSONObject.assetDisplayPath(): String =
    optString("virtual_display_path").ifBlank { optString("filename") }

private fun JSONObject.assetStorageLocation(): String? {
    val location = optJSONObject("storage_location") ?: return null
    return location.optString("path")
        .ifBlank { location.optString("uri") }
        .ifBlank { null }
}

private fun JSONObject.optStringOrNull(key: String): String? =
    optString(key).takeIf { it.isNotBlank() }

private fun JSONObject.optIntOrNull(key: String): Int? =
    if (has(key) && !isNull(key)) optInt(key) else null

private fun JSONObject.optLongOrNull(key: String): Long? =
    if (has(key) && !isNull(key)) optLong(key) else null

private fun JSONObject.optDoubleOrNull(key: String): Double? =
    if (has(key) && !isNull(key)) optDouble(key).takeUnless { it.isNaN() } else null

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
        .ifBlank { "Publish failed" }
    val source = item.optString("display_source").takeIf { it.isNotBlank() }
    return if (source == null || path.startsWith("$source/")) {
        path
    } else {
        "$source/$path"
    }
}

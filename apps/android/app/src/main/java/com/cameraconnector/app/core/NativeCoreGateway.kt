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

    override suspend fun loadProjectAssets(query: ProjectAssetQuery, offset: Int, limit: Int): List<ProjectAsset> =
        withContext(Dispatchers.IO) {
            val projectId = projects.value.activeProjectId
                ?: loadProjects().activeProjectId
                ?: return@withContext emptyList()
            mapProjectAssets(
                nativeCore.projectAssetGroupPageJson(
                    projectId = projectId,
                    query = query,
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

    override suspend fun deleteProject(projectId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.deleteProject(projectId)
        }
        refresh()
    }

    override suspend fun restoreProject(projectId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.restoreProject(projectId)
        }
        refresh()
    }

    override suspend fun deleteProjectGroup(projectId: String, groupId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.deleteProjectGroup(projectId, groupId)
        }
        refresh()
    }

    override suspend fun setAssetGroupUserMarks(
        projectId: String,
        groupId: String,
        favorite: Boolean?,
        marked: Boolean?,
    ): ProjectAssetUserMarks {
        val marks = withContext(Dispatchers.IO) {
            mapProjectAssetUserMarks(
                nativeCore.setAssetGroupUserMarks(
                    projectId = projectId,
                    groupId = groupId,
                    favorite = favorite,
                    marked = marked,
                ),
            )
        }
        dashboard.value = dashboard.value.copy(
            assets = dashboard.value.assets.map { asset ->
                if (asset.id == groupId) asset.copy(userMarks = marks) else asset
            },
        )
        return marks
    }

    override suspend fun createLanShareSession(
        projectId: String,
        query: ProjectAssetQuery,
        title: String?,
    ): LanShareSessionUi =
        withContext(Dispatchers.IO) {
            mapLanShareSession(nativeCore.createLanShareSession(projectId, query, title))
        }

    override suspend fun stopLanShareSession(shareId: String): LanShareSessionUi? =
        withContext(Dispatchers.IO) {
            mapLanShareSessionOrNull(nativeCore.stopLanShareSession(shareId))
        }

    override suspend fun loadLanShareAssets(
        token: String,
        offset: Int,
        limit: Int,
    ): List<ProjectAsset> =
        withContext(Dispatchers.IO) {
            mapProjectAssets(nativeCore.lanShareAssetGroupPageJson(token, offset, limit))
        }

    override suspend fun setLanShareGuestMark(
        token: String,
        groupId: String,
        guestMark: GuestMark?,
    ): GuestMark? {
        val nextMark = withContext(Dispatchers.IO) {
            mapGuestMarkFromPatchResult(nativeCore.setLanShareGuestMark(token, groupId, guestMark))
        }
        dashboard.value = dashboard.value.copy(
            assets = dashboard.value.assets.map { asset ->
                if (asset.id == groupId) asset.copy(guestMark = nextMark) else asset
            },
        )
        return nextMark
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

    override suspend fun loadModelProviderSettings(): ModelProviderSettingsUi =
        withContext(Dispatchers.IO) {
            mapModelProviderSettings(nativeCore.modelProviderSettings())
        }

    override suspend fun loadModelProviderSettingsList(): List<ModelProviderSettingsUi> =
        withContext(Dispatchers.IO) {
            mapModelProviderSettingsList(nativeCore.modelProviderSettingsList())
        }

    override suspend fun saveModelProviderSettings(settings: ModelProviderSettingsUi): ModelProviderSettingsUi =
        withContext(Dispatchers.IO) {
            mapModelProviderSettings(
                nativeCore.saveModelProviderSettings(settings.toModelProviderSettingsJson().toString()),
            )
        }

    override suspend fun deleteModelProviderSettings(settingsId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.deleteModelProviderSettings(settingsId)
        }
    }

    override suspend fun loadProjectEvaluationSettings(projectId: String): ProjectEvaluationSettingsUi =
        withContext(Dispatchers.IO) {
            mapProjectEvaluationSettings(nativeCore.projectEvaluationSettings(projectId))
        }

    override suspend fun saveProjectEvaluationSettings(
        settings: ProjectEvaluationSettingsUi,
    ): ProjectEvaluationSettingsUi =
        withContext(Dispatchers.IO) {
            mapProjectEvaluationSettings(
                nativeCore.saveProjectEvaluationSettings(
                    settings.projectId,
                    settings.toProjectEvaluationSettingsJson().toString(),
                ),
            )
        }

    override suspend fun loadGlobalPromptPacks(): List<PromptPackUi> =
        withContext(Dispatchers.IO) {
            mapPromptPacks(nativeCore.globalPromptPacks())
        }

    override suspend fun createGlobalPromptPack(
        name: String,
        styleTags: List<String>,
        sceneProfile: String,
        distributionFolder: String,
        promptText: String,
    ): PromptPackUi =
        withContext(Dispatchers.IO) {
            val tagsJson = JSONArray().apply {
                styleTags.forEach { put(it) }
            }
            mapPromptPack(
                nativeCore.createGlobalPromptPack(
                    name,
                    tagsJson.toString(),
                    sceneProfile,
                    distributionFolder,
                    promptText,
                ),
            )
        }

    override suspend fun forkGlobalPromptPack(
        sourcePackId: String,
        name: String,
        distributionFolder: String,
    ): PromptPackUi =
        withContext(Dispatchers.IO) {
            mapPromptPack(nativeCore.forkGlobalPromptPack(sourcePackId, name, distributionFolder))
        }

    override suspend fun saveGlobalPromptPack(
        promptPackId: String,
        name: String,
        styleTags: List<String>,
        sceneProfile: String,
        promptText: String,
    ): PromptPackUi =
        withContext(Dispatchers.IO) {
            val tagsJson = JSONArray()
            styleTags.forEach { tagsJson.put(it) }
            mapPromptPack(
                nativeCore.saveGlobalPromptVersion(
                    promptPackId,
                    name,
                    tagsJson.toString(),
                    sceneProfile,
                    promptText,
                ),
            )
        }

    override suspend fun deleteGlobalPromptPack(promptPackId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.deleteGlobalPromptPack(promptPackId)
        }
    }

    override suspend fun deleteGlobalPromptPackage(distributionFolder: String) {
        withContext(Dispatchers.IO) {
            nativeCore.deleteGlobalPromptPackage(distributionFolder)
        }
    }

    override suspend fun loadPromptPacks(projectId: String): List<PromptPackUi> =
        withContext(Dispatchers.IO) {
            mapPromptPacks(nativeCore.promptPacksForProject(projectId))
        }

    override suspend fun forkPromptPack(
        projectId: String,
        sourcePackId: String,
        name: String,
        distributionFolder: String,
    ): PromptPackUi =
        withContext(Dispatchers.IO) {
            mapPromptPack(nativeCore.forkPromptPack(projectId, sourcePackId, name, distributionFolder))
        }

    override suspend fun savePromptPack(
        projectId: String,
        promptPackId: String,
        name: String,
        styleTags: List<String>,
        sceneProfile: String,
        promptText: String,
    ): PromptPackUi =
        withContext(Dispatchers.IO) {
            val tagsJson = JSONArray()
            styleTags.forEach { tagsJson.put(it) }
            mapPromptPack(
                nativeCore.savePromptVersion(
                    projectId,
                    promptPackId,
                    name,
                    tagsJson.toString(),
                    sceneProfile,
                    promptText,
                ),
            )
        }

    override suspend fun generateProjectRecommendation(projectId: String): EvaluationRunUi =
        withContext(Dispatchers.IO) {
            projectRecommendationRunAfterGenerate(
                generateRecommendation = { nativeCore.generateProjectRecommendation(projectId) },
                latestRun = { nativeCore.latestProjectRecommendationRunStatus(projectId) },
            )
        }

    override suspend fun generateProjectRecommendationWithCandidateVisuals(
        projectId: String,
        candidateVisuals: List<SelectionCandidateVisualInput>,
    ): EvaluationRunUi =
        withContext(Dispatchers.IO) {
            projectRecommendationRunAfterGenerate(
                generateRecommendation = {
                    nativeCore.generateProjectRecommendationWithCandidateVisuals(
                        projectId,
                        candidateVisuals,
                    )
                },
                latestRun = { nativeCore.latestProjectRecommendationRunStatus(projectId) },
            )
        }

    override suspend fun latestProjectRecommendationRunStatus(projectId: String): EvaluationRunUi? =
        withContext(Dispatchers.IO) {
            nativeCore.latestProjectRecommendationRunStatus(projectId)?.let(::mapEvaluationRun)
        }

    override suspend fun enqueueModelEvaluation(projectId: String, assetGroupIds: List<String>): Int {
        val enqueuedCount = withContext(Dispatchers.IO) {
            nativeCore.enqueueModelEvaluationForAssetGroups(projectId, assetGroupIds)
                .optInt("enqueued_count")
        }
        refresh()
        return enqueuedCount
    }

    override suspend fun evaluateAssetGroupsWithModelInputs(
        projectId: String,
        inputs: List<ModelEvaluationPreviewInput>,
    ): Int {
        val savedCount = withContext(Dispatchers.IO) {
            nativeCore.evaluateAssetGroupsWithModelInputs(projectId, inputs)
                .optInt("saved_count")
        }
        refresh()
        return savedCount
    }

    override suspend fun recommendBurstGroupWithCandidateVisuals(
        burstGroupId: String,
        candidateVisuals: List<SelectionCandidateVisualInput>,
    ): Boolean {
        withContext(Dispatchers.IO) {
            val visualArray = JSONArray()
            candidateVisuals.forEach { visual ->
                visualArray.put(
                    JSONObject()
                        .put("asset_group_id", visual.assetGroupId)
                        .put("image_data_url", visual.imageDataUrl),
                )
            }
            nativeCore.recommendBurstGroupWithCandidateVisuals(burstGroupId, visualArray)
        }
        refresh()
        return true
    }

    override suspend fun shouldScheduleSubjectAssessment(projectId: String): Boolean =
        withContext(Dispatchers.IO) {
            nativeCore.shouldScheduleSubjectAssessment(projectId)
        }

    override suspend fun saveSubjectAssessment(assessment: SubjectAssessmentUi): SubjectAssessmentUi =
        withContext(Dispatchers.IO) {
            mapSubjectAssessment(
                nativeCore.saveSubjectAssessment(assessment.toSubjectAssessmentJson().toString()),
            )
        }

    override suspend fun loadSubjectAssessments(
        projectId: String,
        groupIds: List<String>,
    ): List<SubjectAssessmentUi> =
        withContext(Dispatchers.IO) {
            mapSubjectAssessments(
                nativeCore.subjectAssessmentsForAssetGroups(
                    projectId,
                    JSONArray(groupIds).toString(),
                ),
            )
        }

    override suspend fun splitBurstMember(burstGroupId: String, memberGroupId: String) {
        withContext(Dispatchers.IO) {
            nativeCore.splitBurstMember(burstGroupId, memberGroupId)
        }
        refresh()
    }

    override suspend fun createManualBurstGroup(projectId: String, memberGroupIds: List<String>) {
        withContext(Dispatchers.IO) {
            nativeCore.createManualBurstGroup(projectId, memberGroupIds)
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
            limit = PROJECT_DASHBOARD_ASSET_LIMIT,
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
            assets = mapProjectAssets(assets),
            transfers = transferRows,
            publishQueue = mapPublishQueueState(value.optJSONObject("publish_queue")),
            globalAssets = mapGlobalAssetSummary(value.optJSONObject("global_assets")),
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
                outputLabel = "\u9009\u62e9\u8f93\u51fa\u6587\u4ef6\u5939",
                message = null,
            ),
            accounts = emptyList(),
            assets = emptyList(),
            transfers = emptyList(),
        )

    private companion object {
        const val DASHBOARD_POLL_INTERVAL_MS = 2_000L
        const val PROJECT_DASHBOARD_ASSET_LIMIT = 2_000
    }
}

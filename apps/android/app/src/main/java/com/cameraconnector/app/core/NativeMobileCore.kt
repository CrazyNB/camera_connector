package com.cameraconnector.app.core

import com.cameraconnector.app.storage.PublishQueueCore
import com.cameraconnector.app.storage.PublishedObject
import org.json.JSONArray
import org.json.JSONObject

class NativeMobileCore(configPath: String?) : AutoCloseable, PublishQueueCore {
    private var handle: Long = create(configPath)

    fun projectDashboardJson(projectId: String, offset: Int, limit: Int): JSONObject =
        call { projectDashboardJson(handle, projectId, offset, limit) }

    fun projectAssetGroupPageJson(
        projectId: String,
        query: InboxAssetQuery,
        offset: Int,
        limit: Int,
    ): JSONObject =
        call { projectAssetGroupPageJson(handle, projectId, assetGroupQueryJson(query).toString(), offset, limit) }

    fun projectGroupAssetsJson(projectId: String, groupId: String): JSONArray =
        call { projectGroupAssetsJson(handle, projectId, groupId) }.optJSONArray("value")
            ?: JSONArray()

    fun moveProjectGroup(sourceProjectId: String, groupId: String, targetProjectId: String): JSONObject =
        call { moveProjectGroupJson(handle, sourceProjectId, groupId, targetProjectId) }

    fun setAssetGroupUserMarks(
        projectId: String,
        groupId: String,
        favorite: Boolean? = null,
        marked: Boolean? = null,
    ): JSONObject =
        call {
            setAssetGroupUserMarksJson(
                handle,
                projectId,
                groupId,
                userMarksPatchJson(favorite, marked).toString(),
            )
        }

    override fun claimNextPublishItem(): JSONObject? {
        val value = call { claimNextPublishItemJson(handle) }
        return if (value.has("value") && value.isNull("value")) {
            null
        } else {
            value
        }
    }

    fun markPublishCompleted(queueId: String): JSONObject =
        call { markPublishCompletedJson(handle, queueId) }

    override fun completePublish(queueId: String, publishedObject: PublishedObject): JSONObject =
        call {
            completePublishJson(
                handle,
                queueId,
                publishedObject.finalFilename,
                publishedObject.locationKind,
                publishedObject.location,
            )
        }

    override fun markPublishFailed(queueId: String, error: String): JSONObject =
        call { markPublishFailedJson(handle, queueId, error) }

    fun releaseFailedPublishRetries(projectId: String): JSONObject =
        call { releaseFailedPublishRetriesJson(handle, projectId) }

    fun drainAnalysisJobs(limit: Int = 32): JSONObject =
        call { drainAnalysisJobsJson(handle, limit.coerceAtLeast(0)) }

    fun drainAnalysisJobsWithProviderConfigured(
        limit: Int = 32,
        providerConfigured: Boolean,
    ): JSONObject =
        call {
            drainAnalysisJobsWithProviderConfiguredJson(
                handle,
                limit.coerceAtLeast(0),
                providerConfigured,
            )
        }

    fun scoreAssetGroupPreview(
        assetGroupId: String,
        sampleJson: String,
        scorerVersion: String = "local-v1",
    ): JSONObject =
        call { scoreAssetGroupPreviewJson(handle, assetGroupId, sampleJson, scorerVersion) }

    fun scoreAssetGroupPreviewWithProviderConfigured(
        assetGroupId: String,
        sampleJson: String,
        scorerVersion: String = "local-v1",
        providerConfigured: Boolean,
    ): JSONObject =
        call {
            scoreAssetGroupPreviewWithProviderConfiguredJson(
                handle,
                assetGroupId,
                sampleJson,
                scorerVersion,
                providerConfigured,
            )
        }

    fun recommendBurstGroup(burstGroupId: String, strategyProfileId: String? = null): JSONObject =
        call { recommendBurstGroupJson(handle, burstGroupId, strategyProfileId) }

    fun acceptRecommendedBest(burstGroupId: String, strategyProfileId: String? = null): JSONObject =
        call { acceptRecommendedBestJson(handle, burstGroupId, strategyProfileId) }

    fun markBurstNeedsReview(burstGroupId: String, strategyProfileId: String? = null): JSONObject =
        call { markBurstNeedsReviewJson(handle, burstGroupId, strategyProfileId) }

    fun restoreAutomaticRecommendation(
        burstGroupId: String,
        strategyProfileId: String? = null,
    ): JSONObject =
        call { restoreAutomaticRecommendationJson(handle, burstGroupId, strategyProfileId) }

    fun clearRecommendation(burstGroupId: String, strategyProfileId: String? = null): JSONObject =
        call { clearRecommendationJson(handle, burstGroupId, strategyProfileId) }

    fun keepAllCandidates(burstGroupId: String, strategyProfileId: String? = null): JSONObject =
        call { keepAllCandidatesJson(handle, burstGroupId, strategyProfileId) }

    fun hideLowScoreCandidates(burstGroupId: String, strategyProfileId: String? = null): JSONObject =
        call { hideLowScoreCandidatesJson(handle, burstGroupId, strategyProfileId) }

    fun overrideRecommendedBest(
        burstGroupId: String,
        bestAssetGroupId: String,
        strategyProfileId: String? = null,
    ): JSONObject =
        call { overrideRecommendedBestJson(handle, burstGroupId, bestAssetGroupId, strategyProfileId) }

    fun splitBurstMember(burstGroupId: String, memberGroupId: String): JSONObject =
        call { splitBurstMemberJson(handle, burstGroupId, memberGroupId) }

    fun mergeBurstMember(targetBurstGroupId: String, memberGroupId: String): JSONObject =
        call { mergeBurstMemberJson(handle, targetBurstGroupId, memberGroupId) }

    fun modelProviderSettings(): JSONObject =
        call { modelProviderSettingsJson(handle) }

    fun saveModelProviderSettings(settingsJson: String): JSONObject =
        call { saveModelProviderSettingsJson(handle, settingsJson) }

    fun projectEvaluationSettings(projectId: String): JSONObject =
        call { projectEvaluationSettingsJson(handle, projectId) }

    fun saveProjectEvaluationSettings(projectId: String, settingsJson: String): JSONObject =
        call { saveProjectEvaluationSettingsJson(handle, projectId, settingsJson) }

    fun promptProfilesForProject(projectId: String): JSONArray =
        call { promptProfilesForProjectJson(handle, projectId) }.optJSONArray("value")
            ?: JSONArray()

    fun globalPromptProfiles(): JSONArray =
        call { globalPromptProfilesJson(handle) }.optJSONArray("value") ?: JSONArray()

    fun forkGlobalPromptProfile(sourceProfileId: String, name: String): JSONObject =
        call { forkGlobalPromptProfileJson(handle, sourceProfileId, name) }

    fun saveGlobalPromptVersion(promptProfileId: String, promptText: String): JSONObject =
        call { saveGlobalPromptVersionJson(handle, promptProfileId, promptText) }

    fun forkPromptProfile(projectId: String, sourceProfileId: String, name: String): JSONObject =
        call { forkPromptProfileJson(handle, projectId, sourceProfileId, name) }

    fun savePromptVersion(projectId: String, promptProfileId: String, promptText: String): JSONObject =
        call { savePromptVersionJson(handle, projectId, promptProfileId, promptText) }

    fun generateProjectRecommendation(projectId: String): JSONObject =
        call { generateProjectRecommendationJson(handle, projectId) }

    fun latestProjectRecommendationRunStatus(projectId: String): JSONObject? {
        val value = call { latestProjectRecommendationRunStatusJson(handle, projectId) }
        return if (value.has("value") && value.isNull("value")) {
            null
        } else {
            value
        }
    }

    fun shouldScheduleSubjectAssessment(projectId: String): Boolean {
        val value = call { shouldScheduleSubjectAssessmentJson(handle, projectId) }
        return value.optBoolean("value", false)
    }

    fun saveSubjectAssessment(assessmentJson: String): JSONObject =
        call { saveSubjectAssessmentJson(handle, assessmentJson) }

    fun subjectAssessmentsForAssetGroups(projectId: String, groupIdsJson: String): JSONArray =
        call { subjectAssessmentsForAssetGroupsJson(handle, projectId, groupIdsJson) }
            .optJSONArray("value") ?: JSONArray()

    fun strategyProfiles(): JSONArray =
        call { strategyProfilesJson(handle) }.optJSONArray("value") ?: JSONArray()

    fun saveStrategyProfile(profileJson: String): JSONObject =
        call { saveStrategyProfileJson(handle, profileJson) }

    fun createProject(name: String): JSONObject =
        call { createProjectJson(handle, name) }

    fun listProjects(): JSONArray =
        call { listProjectsJson(handle) }.optJSONArray("value") ?: JSONArray()

    fun setActiveProject(projectId: String): JSONObject =
        call { setActiveProjectJson(handle, projectId) }

    fun renameProject(projectId: String, name: String): JSONObject =
        call { renameProjectJson(handle, projectId, name) }

    fun archiveProject(projectId: String): JSONObject =
        call { archiveProjectJson(handle, projectId) }

    fun restoreProject(projectId: String): JSONObject =
        call { restoreProjectJson(handle, projectId) }

    fun activeProject(): JSONObject? {
        val value = call { activeProjectJson(handle) }
        return if (value.has("value") && value.isNull("value")) {
            null
        } else {
            value
        }
    }

    fun saveReceiverSettings(settings: ReceiverSettings) {
        call { saveReceiverSettingsJson(handle, receiverSettingsPatch(settings).toString()) }
    }

    fun saveAndroidReceiverPaths(outputDir: String, stateDir: String) {
        call {
            saveReceiverSettingsJson(
                handle,
                androidReceiverPathsPatch(outputDir, stateDir).toString(),
            )
        }
    }

    fun saveDeviceAccount(account: DeviceAccount, password: String?) {
        call {
            saveDeviceAccountJson(
                handle,
                account.username,
                password,
                account.deviceName,
            )
        }
    }

    fun removeDeviceAccount(username: String) {
        call { removeDeviceAccountJson(handle, username) }
    }

    fun startReceiver(): JSONObject =
        call { startReceiverJson(handle) }

    fun stopReceiver(): JSONObject =
        call { stopReceiverJson(handle) }

    override fun close() {
        val current = handle
        if (current != 0L) {
            destroy(current)
            handle = 0
        }
    }

    private fun call(block: () -> String): JSONObject {
        ensureOpen()
        return NativeEnvelope.unwrap(block())
    }

    private fun ensureOpen() {
        check(handle != 0L) { "NativeMobileCore is closed" }
    }

    private external fun create(configPath: String?): Long
    private external fun destroy(handle: Long)
    private external fun projectDashboardJson(handle: Long, projectId: String, offset: Int, limit: Int): String
    private external fun projectAssetGroupPageJson(
        handle: Long,
        projectId: String,
        queryJson: String,
        offset: Int,
        limit: Int,
    ): String
    private external fun projectGroupAssetsJson(handle: Long, projectId: String, groupId: String): String
    private external fun moveProjectGroupJson(
        handle: Long,
        sourceProjectId: String,
        groupId: String,
        targetProjectId: String,
    ): String
    private external fun setAssetGroupUserMarksJson(
        handle: Long,
        projectId: String,
        groupId: String,
        patchJson: String,
    ): String
    private external fun claimNextPublishItemJson(handle: Long): String
    private external fun markPublishCompletedJson(handle: Long, queueId: String): String
    private external fun completePublishJson(
        handle: Long,
        queueId: String,
        finalFilename: String,
        locationKind: String,
        location: String,
    ): String
    private external fun markPublishFailedJson(handle: Long, queueId: String, error: String): String
    private external fun releaseFailedPublishRetriesJson(handle: Long, projectId: String): String
    private external fun drainAnalysisJobsJson(handle: Long, limit: Int): String
    private external fun drainAnalysisJobsWithProviderConfiguredJson(
        handle: Long,
        limit: Int,
        providerConfigured: Boolean,
    ): String
    private external fun scoreAssetGroupPreviewJson(
        handle: Long,
        assetGroupId: String,
        sampleJson: String,
        scorerVersion: String,
    ): String
    private external fun scoreAssetGroupPreviewWithProviderConfiguredJson(
        handle: Long,
        assetGroupId: String,
        sampleJson: String,
        scorerVersion: String,
        providerConfigured: Boolean,
    ): String
    private external fun recommendBurstGroupJson(
        handle: Long,
        burstGroupId: String,
        strategyProfileId: String?,
    ): String
    private external fun acceptRecommendedBestJson(
        handle: Long,
        burstGroupId: String,
        strategyProfileId: String?,
    ): String
    private external fun markBurstNeedsReviewJson(
        handle: Long,
        burstGroupId: String,
        strategyProfileId: String?,
    ): String
    private external fun restoreAutomaticRecommendationJson(
        handle: Long,
        burstGroupId: String,
        strategyProfileId: String?,
    ): String
    private external fun clearRecommendationJson(
        handle: Long,
        burstGroupId: String,
        strategyProfileId: String?,
    ): String
    private external fun keepAllCandidatesJson(
        handle: Long,
        burstGroupId: String,
        strategyProfileId: String?,
    ): String
    private external fun hideLowScoreCandidatesJson(
        handle: Long,
        burstGroupId: String,
        strategyProfileId: String?,
    ): String
    private external fun overrideRecommendedBestJson(
        handle: Long,
        burstGroupId: String,
        bestAssetGroupId: String,
        strategyProfileId: String?,
    ): String
    private external fun splitBurstMemberJson(
        handle: Long,
        burstGroupId: String,
        memberGroupId: String,
    ): String
    private external fun mergeBurstMemberJson(
        handle: Long,
        targetBurstGroupId: String,
        memberGroupId: String,
    ): String
    private external fun modelProviderSettingsJson(handle: Long): String
    private external fun saveModelProviderSettingsJson(handle: Long, settingsJson: String): String
    private external fun projectEvaluationSettingsJson(handle: Long, projectId: String): String
    private external fun saveProjectEvaluationSettingsJson(
        handle: Long,
        projectId: String,
        settingsJson: String,
    ): String
    private external fun promptProfilesForProjectJson(handle: Long, projectId: String): String
    private external fun globalPromptProfilesJson(handle: Long): String
    private external fun forkGlobalPromptProfileJson(
        handle: Long,
        sourceProfileId: String,
        name: String,
    ): String
    private external fun saveGlobalPromptVersionJson(
        handle: Long,
        promptProfileId: String,
        promptText: String,
    ): String
    private external fun forkPromptProfileJson(
        handle: Long,
        projectId: String,
        sourceProfileId: String,
        name: String,
    ): String
    private external fun savePromptVersionJson(
        handle: Long,
        projectId: String,
        promptProfileId: String,
        promptText: String,
    ): String
    private external fun generateProjectRecommendationJson(handle: Long, projectId: String): String
    private external fun latestProjectRecommendationRunStatusJson(handle: Long, projectId: String): String
    private external fun shouldScheduleSubjectAssessmentJson(handle: Long, projectId: String): String
    private external fun saveSubjectAssessmentJson(handle: Long, assessmentJson: String): String
    private external fun subjectAssessmentsForAssetGroupsJson(
        handle: Long,
        projectId: String,
        groupIdsJson: String,
    ): String
    private external fun strategyProfilesJson(handle: Long): String
    private external fun saveStrategyProfileJson(handle: Long, profileJson: String): String
    private external fun createProjectJson(handle: Long, name: String): String
    private external fun listProjectsJson(handle: Long): String
    private external fun setActiveProjectJson(handle: Long, projectId: String): String
    private external fun renameProjectJson(handle: Long, projectId: String, name: String): String
    private external fun archiveProjectJson(handle: Long, projectId: String): String
    private external fun restoreProjectJson(handle: Long, projectId: String): String
    private external fun activeProjectJson(handle: Long): String
    private external fun saveReceiverSettingsJson(handle: Long, patchJson: String): String
    private external fun saveDeviceAccountJson(
        handle: Long,
        username: String,
        password: String?,
        deviceName: String,
    ): String
    private external fun removeDeviceAccountJson(handle: Long, username: String): String
    private external fun startReceiverJson(handle: Long): String
    private external fun stopReceiverJson(handle: Long): String

    companion object {
        init {
            System.loadLibrary("camera_connector_ffi")
        }
    }
}

internal fun receiverSettingsPatch(settings: ReceiverSettings): JSONObject =
    receiverSettingsPatchFields(settings).entries.fold(JSONObject()) { patch, (key, value) ->
        patch.put(key, value)
    }

internal fun androidReceiverPathsPatch(outputDir: String, stateDir: String): JSONObject =
    JSONObject()
        .put("output_dir", outputDir)
        .put("state_dir", stateDir)
        .put("defer_publish", true)

internal fun receiverSettingsPatchFields(settings: ReceiverSettings): Map<String, Any> =
    mapOf(
        "protocol" to settings.protocol.lowercase(),
        "bind_host" to settings.host,
        "ftp_port" to settings.ftpPort,
        "sftp_port" to settings.sftpPort,
    )

internal fun assetGroupQueryJson(query: InboxAssetQuery): JSONObject =
    JSONObject().apply {
        query.username?.takeIf { it.isNotBlank() }?.let { put("username", it) }
        query.sourceName?.takeIf { it.isNotBlank() }?.let { put("source_name", it) }
        query.originalPath?.takeIf { it.isNotBlank() }?.let { put("original_path", it) }
        query.remoteAddr?.takeIf { it.isNotBlank() }?.let { put("remote_addr", it) }
        query.format?.takeIf { it.isNotBlank() }?.let { put("format", it) }
        query.role?.let { put("role", it.wireName) }
        put("sort", query.sort.wireName)
        query.recommendationState?.takeIf { it.isNotBlank() }?.let { put("recommendation_state", it) }
        query.scoreMin?.let { put("score_min", it) }
        query.scoreMax?.let { put("score_max", it) }
        query.analysisStatus?.takeIf { it.isNotBlank() }?.let { put("analysis_status", it) }
        query.reviewQueue?.takeIf { it.isNotBlank() }?.let { put("review_queue", it) }
        query.strategyProfileId?.takeIf { it.isNotBlank() }?.let { put("strategy_profile_id", it) }
        query.favorite?.let { put("favorite", it) }
        query.marked?.let { put("marked", it) }
    }

internal fun userMarksPatchJson(favorite: Boolean?, marked: Boolean?): JSONObject =
    JSONObject().apply {
        favorite?.let { put("favorite", it) }
        marked?.let { put("marked", it) }
    }

object NativeEnvelope {
    fun unwrap(raw: String): JSONObject {
        val envelope = JSONObject(raw)
        if (!envelope.optBoolean("ok", false)) {
            throw NativeCoreException(envelope.optString("error", "Native core call failed"))
        }

        return envelope.optJSONObject("value")
            ?: JSONObject().put("value", envelope.opt("value"))
    }
}

class NativeCoreException(message: String) : RuntimeException(message)

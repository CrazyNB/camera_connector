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
        query: ProjectAssetQuery,
        offset: Int,
        limit: Int,
    ): JSONObject =
        call { projectAssetGroupPageJson(handle, projectId, assetGroupQueryJson(query).toString(), offset, limit) }

    fun projectGroupAssetsJson(projectId: String, groupId: String): JSONArray =
        call { projectGroupAssetsJson(handle, projectId, groupId) }.optJSONArray("value")
            ?: JSONArray()

    fun moveProjectGroup(sourceProjectId: String, groupId: String, targetProjectId: String): JSONObject =
        call { moveProjectGroupJson(handle, sourceProjectId, groupId, targetProjectId) }

    fun deleteProjectGroup(projectId: String, groupId: String): JSONObject =
        call { deleteProjectGroupJson(handle, projectId, groupId) }

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

    fun enqueueModelEvaluationForAssetGroups(projectId: String, assetGroupIds: List<String>): JSONObject =
        call {
            enqueueModelEvaluationForAssetGroupsJson(
                handle,
                JSONObject()
                    .put("project_id", projectId)
                    .put("asset_group_ids", JSONArray(assetGroupIds))
                    .toString(),
            )
        }

    fun evaluateAssetGroupsWithModelInputs(
        projectId: String,
        inputs: List<ModelEvaluationPreviewInput>,
    ): JSONObject =
        call {
            val inputArray = JSONArray()
            inputs.forEach { input ->
                inputArray.put(
                    JSONObject()
                        .put("asset_group_id", input.assetGroupId)
                        .put("sample", JSONObject(input.sampleJson)),
                )
            }
            evaluateAssetGroupsWithModelInputsJson(
                handle,
                JSONObject()
                    .put("project_id", projectId)
                    .put("inputs", inputArray)
                    .toString(),
            )
        }

    fun recommendBurstGroupWithCandidateVisuals(
        burstGroupId: String,
        candidateVisuals: JSONArray,
    ): JSONObject =
        call {
            recommendBurstGroupWithCandidateVisualsJson(
                handle,
                JSONObject()
                    .put("burst_group_id", burstGroupId)
                    .put("candidate_visuals", candidateVisuals)
                    .toString(),
            )
        }

    fun assessAssetGroupPreview(
        assetGroupId: String,
        sampleJson: String,
        assessorVersion: String = "technical-v1",
    ): JSONObject =
        call { assessAssetGroupPreviewJson(handle, assetGroupId, sampleJson, assessorVersion) }

    fun assessAssetGroupPreviewWithProviderConfigured(
        assetGroupId: String,
        sampleJson: String,
        assessorVersion: String = "technical-v1",
        providerConfigured: Boolean,
    ): JSONObject =
        call {
            assessAssetGroupPreviewWithProviderConfiguredJson(
                handle,
                assetGroupId,
                sampleJson,
                assessorVersion,
                providerConfigured,
            )
        }

    fun splitBurstMember(burstGroupId: String, memberGroupId: String): JSONObject =
        call { splitBurstMemberJson(handle, burstGroupId, memberGroupId) }

    fun mergeBurstMember(targetBurstGroupId: String, memberGroupId: String): JSONObject =
        call { mergeBurstMemberJson(handle, targetBurstGroupId, memberGroupId) }

    fun modelProviderSettings(): JSONObject =
        call { modelProviderSettingsJson(handle) }

    fun modelProviderSettingsList(): JSONArray =
        call { modelProviderSettingsListJson(handle) }.optJSONArray("value") ?: JSONArray()

    fun saveModelProviderSettings(settingsJson: String): JSONObject =
        call { saveModelProviderSettingsJson(handle, settingsJson) }

    fun deleteModelProviderSettings(settingsId: String): JSONObject =
        call { deleteModelProviderSettingsJson(handle, settingsId) }

    fun projectEvaluationSettings(projectId: String): JSONObject =
        call { projectEvaluationSettingsJson(handle, projectId) }

    fun saveProjectEvaluationSettings(projectId: String, settingsJson: String): JSONObject =
        call { saveProjectEvaluationSettingsJson(handle, projectId, settingsJson) }

    fun promptProfilesForProject(projectId: String): JSONArray =
        call { PromptPacksForProjectJson(handle, projectId) }.optJSONArray("value")
            ?: JSONArray()

    fun globalPromptProfiles(): JSONArray =
        call { globalPromptPacksJson(handle) }.optJSONArray("value") ?: JSONArray()

    fun forkGlobalPromptProfile(sourceProfileId: String, name: String, distributionFolder: String): JSONObject =
        call { forkGlobalPromptPackJson(handle, sourceProfileId, name, distributionFolder) }

    fun createGlobalPromptProfile(
        name: String,
        styleTagsJson: String,
        sceneProfile: String,
        distributionFolder: String,
        promptText: String,
    ): JSONObject =
        call { createGlobalPromptPackJson(handle, name, styleTagsJson, sceneProfile, distributionFolder, promptText) }

    fun saveGlobalPromptVersion(
        promptProfileId: String,
        name: String,
        styleTagsJson: String,
        sceneProfile: String,
        promptText: String,
    ): JSONObject =
        call { saveGlobalPromptPackJson(handle, promptProfileId, name, styleTagsJson, sceneProfile, promptText) }

    fun deleteGlobalPromptPack(promptProfileId: String): JSONObject =
        call { deleteGlobalPromptPackJson(handle, promptProfileId) }

    fun deleteGlobalPromptPackage(distributionFolder: String): JSONObject =
        call { deleteGlobalPromptPackageJson(handle, distributionFolder) }

    fun forkPromptProfile(
        projectId: String,
        sourceProfileId: String,
        name: String,
        distributionFolder: String,
    ): JSONObject =
        call { forkPromptPackJson(handle, projectId, sourceProfileId, name, distributionFolder) }

    fun savePromptVersion(
        projectId: String,
        promptProfileId: String,
        name: String,
        styleTagsJson: String,
        sceneProfile: String,
        promptText: String,
    ): JSONObject =
        call { savePromptPackJson(handle, projectId, promptProfileId, name, styleTagsJson, sceneProfile, promptText) }

    fun generateProjectRecommendation(projectId: String): JSONObject =
        call { generateProjectRecommendationJson(handle, projectId) }

    fun generateProjectRecommendationWithCandidateVisuals(
        projectId: String,
        candidateVisuals: List<SelectionCandidateVisualInput>,
    ): JSONObject =
        call {
            val visualArray = JSONArray()
            candidateVisuals.forEach { visual ->
                visualArray.put(
                    JSONObject()
                        .put("asset_group_id", visual.assetGroupId)
                        .put("image_data_url", visual.imageDataUrl),
                )
            }
            generateProjectRecommendationWithCandidateVisualsJson(
                handle,
                JSONObject()
                    .put("project_id", projectId)
                    .put("candidate_visuals", visualArray)
                    .toString(),
            )
        }

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
    private external fun deleteProjectGroupJson(handle: Long, projectId: String, groupId: String): String
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
    private external fun enqueueModelEvaluationForAssetGroupsJson(handle: Long, requestJson: String): String
    private external fun evaluateAssetGroupsWithModelInputsJson(handle: Long, requestJson: String): String
    private external fun recommendBurstGroupWithCandidateVisualsJson(handle: Long, requestJson: String): String
    private external fun assessAssetGroupPreviewJson(
        handle: Long,
        assetGroupId: String,
        sampleJson: String,
        assessorVersion: String,
    ): String
    private external fun assessAssetGroupPreviewWithProviderConfiguredJson(
        handle: Long,
        assetGroupId: String,
        sampleJson: String,
        assessorVersion: String,
        providerConfigured: Boolean,
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
    private external fun modelProviderSettingsListJson(handle: Long): String
    private external fun saveModelProviderSettingsJson(handle: Long, settingsJson: String): String
    private external fun deleteModelProviderSettingsJson(handle: Long, settingsId: String): String
    private external fun projectEvaluationSettingsJson(handle: Long, projectId: String): String
    private external fun saveProjectEvaluationSettingsJson(
        handle: Long,
        projectId: String,
        settingsJson: String,
    ): String
    private external fun PromptPacksForProjectJson(handle: Long, projectId: String): String
    private external fun globalPromptPacksJson(handle: Long): String
    private external fun forkGlobalPromptPackJson(
        handle: Long,
        sourceProfileId: String,
        name: String,
        distributionFolder: String,
    ): String
    private external fun createGlobalPromptPackJson(
        handle: Long,
        name: String,
        styleTagsJson: String,
        sceneProfile: String,
        distributionFolder: String,
        promptText: String,
    ): String
    private external fun saveGlobalPromptPackJson(
        handle: Long,
        promptProfileId: String,
        name: String,
        styleTagsJson: String,
        sceneProfile: String,
        promptText: String,
    ): String
    private external fun deleteGlobalPromptPackJson(handle: Long, promptProfileId: String): String
    private external fun deleteGlobalPromptPackageJson(handle: Long, distributionFolder: String): String
    private external fun forkPromptPackJson(
        handle: Long,
        projectId: String,
        sourceProfileId: String,
        name: String,
        distributionFolder: String,
    ): String
    private external fun savePromptPackJson(
        handle: Long,
        projectId: String,
        promptProfileId: String,
        name: String,
        styleTagsJson: String,
        sceneProfile: String,
        promptText: String,
    ): String
    private external fun generateProjectRecommendationJson(handle: Long, projectId: String): String
    private external fun generateProjectRecommendationWithCandidateVisualsJson(
        handle: Long,
        requestJson: String,
    ): String
    private external fun latestProjectRecommendationRunStatusJson(handle: Long, projectId: String): String
    private external fun shouldScheduleSubjectAssessmentJson(handle: Long, projectId: String): String
    private external fun saveSubjectAssessmentJson(handle: Long, assessmentJson: String): String
    private external fun subjectAssessmentsForAssetGroupsJson(
        handle: Long,
        projectId: String,
        groupIdsJson: String,
    ): String
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

internal fun assetGroupQueryJson(query: ProjectAssetQuery): JSONObject =
    JSONObject().apply {
        query.role?.let { put("role", it.wireName) }
        put("sort", query.sort.wireName)
        query.collection?.takeIf { it.isNotBlank() }?.let { put("collection", it) }
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

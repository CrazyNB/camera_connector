package com.cameraconnector.app.service

import android.content.Context
import android.util.Log
import com.cameraconnector.app.core.ProjectAsset
import com.cameraconnector.app.core.ProjectAssetQuery
import com.cameraconnector.app.core.NativeMobileCore
import com.cameraconnector.app.core.PhotoSortMode
import com.cameraconnector.app.core.mapProjectAssets
import com.cameraconnector.app.media.loadPreviewSampleJson
import org.json.JSONArray
import org.json.JSONObject

data class SmartSelectionDrainResult(
    val assessedCount: Int,
    val recommendedCount: Int,
    val failedCount: Int,
)

interface SmartSelectionCore {
    fun activeProject(): JSONObject?
    fun modelProviderSettingsList(): JSONArray
    fun projectEvaluationSettings(projectId: String): JSONObject
    fun projectAssetGroupPageJson(
        projectId: String,
        query: ProjectAssetQuery,
        offset: Int,
        limit: Int,
    ): JSONObject
    fun assessAssetGroupPreviewWithProviderConfigured(
        assetGroupId: String,
        sampleJson: String,
        assessorVersion: String = "technical-v1",
        providerConfigured: Boolean,
    ): JSONObject
    fun drainAnalysisJobsWithProviderConfigured(
        limit: Int = 32,
        providerConfigured: Boolean,
    ): JSONObject
    fun recommendBurstGroupWithCandidateVisuals(
        burstGroupId: String,
        candidateVisuals: JSONArray,
    ): JSONObject
    fun evaluateAssetGroupsWithModelInputs(
        projectId: String,
        inputs: JSONArray,
    ): JSONObject
    fun generateProjectRecommendation(projectId: String): JSONObject
}

class NativeSmartSelectionCore(
    private val core: NativeMobileCore,
) : SmartSelectionCore {
    override fun activeProject(): JSONObject? = core.activeProject()
    override fun modelProviderSettingsList(): JSONArray = core.modelProviderSettingsList()
    override fun projectEvaluationSettings(projectId: String): JSONObject = core.projectEvaluationSettings(projectId)
    override fun projectAssetGroupPageJson(
        projectId: String,
        query: ProjectAssetQuery,
        offset: Int,
        limit: Int,
    ): JSONObject = core.projectAssetGroupPageJson(projectId, query, offset, limit)

    override fun assessAssetGroupPreviewWithProviderConfigured(
        assetGroupId: String,
        sampleJson: String,
        assessorVersion: String,
        providerConfigured: Boolean,
    ): JSONObject = core.assessAssetGroupPreviewWithProviderConfigured(
        assetGroupId = assetGroupId,
        sampleJson = sampleJson,
        assessorVersion = assessorVersion,
        providerConfigured = providerConfigured,
    )

    override fun drainAnalysisJobsWithProviderConfigured(
        limit: Int,
        providerConfigured: Boolean,
    ): JSONObject = core.drainAnalysisJobsWithProviderConfigured(limit, providerConfigured)

    override fun recommendBurstGroupWithCandidateVisuals(
        burstGroupId: String,
        candidateVisuals: JSONArray,
    ): JSONObject = core.recommendBurstGroupWithCandidateVisuals(burstGroupId, candidateVisuals)

    override fun evaluateAssetGroupsWithModelInputs(
        projectId: String,
        inputs: JSONArray,
    ): JSONObject =
        core.evaluateAssetGroupsWithModelInputs(
            projectId = projectId,
            inputs = (0 until inputs.length()).map { index ->
                val input = inputs.getJSONObject(index)
                com.cameraconnector.app.core.ModelEvaluationPreviewInput(
                    assetGroupId = input.getString("asset_group_id"),
                    sampleJson = input.getJSONObject("sample").toString(),
                )
            },
        )

    override fun generateProjectRecommendation(projectId: String): JSONObject =
        core.generateProjectRecommendation(projectId)
}

class SmartSelectionAnalysisWorker(
    private val context: Context?,
    private val core: SmartSelectionCore,
    private val previewSampleLoader: (Context?, String?) -> String = { loadContext, previewLocation ->
        loadPreviewSampleJson(requireNotNull(loadContext), previewLocation)
    },
) {
    constructor(
        context: Context,
        core: NativeMobileCore,
    ) : this(context, NativeSmartSelectionCore(core))

    fun drainOnce(maxAssessments: Int = DEFAULT_MAX_ASSESSMENTS): SmartSelectionDrainResult {
        val projectId = core.activeProject()
            ?.optString("project_id")
            ?.takeIf { it.isNotBlank() }
            ?: return SmartSelectionDrainResult(assessedCount = 0, recommendedCount = 0, failedCount = 0)
        val projectSettings = core.projectEvaluationSettings(projectId)
        val providerConfigured = providerConfiguredForProject(
            projectSettings = projectSettings,
            providerOptions = core.modelProviderSettingsList(),
        )
        val visualBurstRecommendationEnabled =
            shouldRunVisualBurstRecommendation(projectSettings, providerConfigured)
        val uploadAutoEvaluationEnabled =
            shouldRunUploadAutoEvaluation(projectSettings, providerConfigured)
        val assets = mapProjectAssets(
            core.projectAssetGroupPageJson(
                projectId = projectId,
                query = ProjectAssetQuery(sort = PhotoSortMode.LatestReceived),
                offset = 0,
                limit = QUERY_LIMIT,
            ),
        )
        var assessedCount = 0
        var failedCount = 0
        val sampleJsonByGroupId = mutableMapOf<String, String>()
        val assessedGroupIds = mutableSetOf<String>()
        val touchedBurstIds = mutableSetOf<String>()

        for (asset in assets) {
            if (assessedCount >= maxAssessments) {
                break
            }
            if (!asset.needsLocalAssessment()) {
                continue
            }
            runCatching {
                val sampleJson = previewSampleLoader(context, asset.previewLocation)
                sampleJsonByGroupId[asset.id] = sampleJson
                val deferBurstModelEvaluation =
                    visualBurstRecommendationEnabled &&
                        asset.burst?.burstGroupId?.isNotBlank() == true
                core.assessAssetGroupPreviewWithProviderConfigured(
                    assetGroupId = asset.id,
                    sampleJson = sampleJson,
                    providerConfigured = providerConfigured && !deferBurstModelEvaluation,
                )
                assessedGroupIds += asset.id
                asset.burst?.burstGroupId?.takeIf { it.isNotBlank() }?.let(touchedBurstIds::add)
                assessedCount += 1
            }.onFailure { error ->
                failedCount += 1
                Log.w(LOG_TAG, "smart selection assessment failed group=${asset.id}", error)
            }
        }

        var recommendedCount = 0
        runCatching {
            recommendedCount = core.drainAnalysisJobsWithProviderConfigured(
                providerConfigured = providerConfigured,
            ).optInt("completed_count")
        }.onFailure { error ->
            failedCount += 1
            Log.w(LOG_TAG, "smart selection analysis queue drain failed", error)
        }

        if (visualBurstRecommendationEnabled) {
            val assetsByBurst = assets
                .mapNotNull { asset ->
                    asset.burst?.burstGroupId
                        ?.takeIf { it.isNotBlank() }
                        ?.let { burstGroupId -> burstGroupId to asset }
                }
                .groupBy(keySelector = { it.first }, valueTransform = { it.second })
            for (burstGroupId in touchedBurstIds) {
                val members = assetsByBurst[burstGroupId].orEmpty()
                if (members.size < 2) {
                    continue
                }
                runCatching {
                    val candidateVisuals = burstCandidateVisuals(
                        members = members,
                        sampleJsonByGroupId = sampleJsonByGroupId,
                        context = context,
                        previewSampleLoader = previewSampleLoader,
                    )
                    if (candidateVisuals.length() >= 2) {
                        val recommendation = core.recommendBurstGroupWithCandidateVisuals(
                            burstGroupId,
                            candidateVisuals,
                        )
                        if (
                            uploadAutoEvaluationEnabled &&
                            recommendation.optString("status").equals("pending", ignoreCase = true)
                        ) {
                            val topCandidateIds = recommendationCandidateIds(recommendation)
                            val evaluationInputs = modelEvaluationInputsForIds(
                                candidateIds = topCandidateIds,
                                members = members,
                                sampleJsonByGroupId = sampleJsonByGroupId,
                                context = context,
                                previewSampleLoader = previewSampleLoader,
                            )
                            if (evaluationInputs.length() > 0) {
                                core.evaluateAssetGroupsWithModelInputs(projectId, evaluationInputs)
                                val finalVisuals = burstCandidateVisuals(
                                    members = members.filter { it.id in topCandidateIds.toSet() },
                                    sampleJsonByGroupId = sampleJsonByGroupId,
                                    context = context,
                                    previewSampleLoader = previewSampleLoader,
                                )
                                if (finalVisuals.length() > 0) {
                                    core.recommendBurstGroupWithCandidateVisuals(burstGroupId, finalVisuals)
                                }
                            }
                        }
                        recommendedCount += 1
                    }
                }.onFailure { error ->
                    failedCount += 1
                    Log.w(LOG_TAG, "smart selection visual burst recommendation failed burst=$burstGroupId", error)
                }
            }
        }

        return SmartSelectionDrainResult(
            assessedCount = assessedCount,
            recommendedCount = recommendedCount,
            failedCount = failedCount,
        )
    }

    private companion object {
        const val DEFAULT_MAX_ASSESSMENTS = 12
        const val QUERY_LIMIT = 128
        const val LOG_TAG = "SmartSelectionAnalysis"
    }
}

private fun shouldRunVisualBurstRecommendation(
    projectSettings: JSONObject,
    providerConfigured: Boolean,
): Boolean =
    providerConfigured &&
        projectSettings.optBoolean("model_evaluation_enabled", false) &&
        projectSettings.optBoolean("auto_burst_recommendation_enabled", true)

private fun shouldRunUploadAutoEvaluation(
    projectSettings: JSONObject,
    providerConfigured: Boolean,
): Boolean =
    providerConfigured &&
        projectSettings.optBoolean("model_evaluation_enabled", false) &&
        projectSettings.optBoolean("auto_evaluate_on_upload", false)

private fun burstCandidateVisuals(
    members: List<ProjectAsset>,
    sampleJsonByGroupId: MutableMap<String, String>,
    context: Context?,
    previewSampleLoader: (Context?, String?) -> String,
): JSONArray {
    val candidateVisuals = JSONArray()
    for (member in members) {
        val sampleJson = sampleJsonByGroupId.getOrPut(member.id) {
            previewSampleLoader(context, member.previewLocation)
        }
        val imageDataUrl = imageDataUrlFromSample(sampleJson) ?: continue
        candidateVisuals.put(
            JSONObject()
                .put("asset_group_id", member.id)
                .put("image_data_url", imageDataUrl),
        )
    }
    return candidateVisuals
}

private fun imageDataUrlFromSample(sampleJson: String): String? =
    runCatching {
        JSONObject(sampleJson)
            .optString("image_data_url")
            .takeIf { it.isNotBlank() && it != "null" }
    }.getOrNull()

private fun recommendationCandidateIds(recommendation: JSONObject): List<String> {
    val ids = mutableListOf<String>()
    fun append(arrayName: String) {
        val array = recommendation.optJSONArray(arrayName) ?: return
        for (index in 0 until array.length()) {
            array.optString(index).takeIf { it.isNotBlank() }?.let(ids::add)
        }
    }
    append("selected_asset_group_ids")
    append("candidate_asset_group_ids")
    return ids.distinct()
}

private fun modelEvaluationInputsForIds(
    candidateIds: List<String>,
    members: List<ProjectAsset>,
    sampleJsonByGroupId: MutableMap<String, String>,
    context: Context?,
    previewSampleLoader: (Context?, String?) -> String,
): JSONArray {
    val memberById = members.associateBy { it.id }
    val inputs = JSONArray()
    for (candidateId in candidateIds.distinct()) {
        val member = memberById[candidateId] ?: continue
        val sampleJson = sampleJsonByGroupId.getOrPut(member.id) {
            previewSampleLoader(context, member.previewLocation)
        }
        inputs.put(
            JSONObject()
                .put("asset_group_id", member.id)
                .put("sample", JSONObject(sampleJson)),
        )
    }
    return inputs
}

fun providerConfiguredForProject(
    projectSettings: JSONObject,
    providerOptions: JSONArray,
): Boolean {
    val selectedId = projectSettings.optString("model_provider_settings_id")
        .takeIf { it.isNotBlank() && it != "null" }
    if (selectedId != null) {
        for (index in 0 until providerOptions.length()) {
            val option = providerOptions.optJSONObject(index) ?: continue
            val settingsId = option.optString("settings_id").ifBlank { "global" }
            if (settingsId == selectedId) {
                return option.isReadyModelProvider()
            }
        }
        return false
    }
    return false
}

private fun JSONObject.isReadyModelProvider(): Boolean {
    if (!optBoolean("configured", false)) {
        return false
    }
    return when (optString("provider_kind").lowercase()) {
        "openai", "custom" -> optBoolean("api_key_configured", false)
        "imported" -> true
        else -> false
    }
}

private fun ProjectAsset.needsLocalAssessment(): Boolean {
    val currentModelStatus = this.modelStatus?.lowercase()
    val technicalStatus = this.technicalGateStatus?.lowercase()
    if (currentModelStatus == "ready" || currentModelStatus == "skipped") {
        return false
    }
    if (
        technicalStatus in listOf("pass", "warn", "inconclusive", "reject", "unsupported") &&
        currentModelStatus != null
    ) {
        return currentModelStatus == "pending" ||
            currentModelStatus == "running" ||
            currentModelStatus == "failed"
    }
    return currentModelStatus == null ||
        currentModelStatus == "pending" ||
        currentModelStatus == "running" ||
        currentModelStatus == "failed"
}

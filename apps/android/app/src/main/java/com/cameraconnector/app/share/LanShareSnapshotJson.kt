package com.cameraconnector.app.share

import com.cameraconnector.app.core.GuestMark
import com.cameraconnector.app.core.ProjectAsset
import org.json.JSONArray
import org.json.JSONObject

internal fun ProjectAsset.toLanShareJson(token: String): JSONObject =
    (id.ifBlank { displayPath }).let { shareId ->
    JSONObject()
        .put("id", shareId)
        .put("display_path", displayPath)
        .put("format", format)
        .put("model_score", modelScore ?: JSONObject.NULL)
        .put("preview_url", "/api/s/${encodePathPart(token)}/preview/${encodePathPart(shareId)}")
        .put("full_preview_url", "/api/s/${encodePathPart(token)}/preview-full/${encodePathPart(shareId)}")
        .put("guest_mark", guestMark?.wireName ?: JSONObject.NULL)
        .put(
            "user_marks",
            JSONObject()
                .put("favorite", userMarks.favorite)
                .put("marked", userMarks.marked),
        )
    }

internal fun List<ProjectAsset>.toProjectSyncSnapshotJson(token: String, exportedAtMs: Long): JSONObject {
    val snapshotAssets = JSONArray()
    val snapshotGroups = JSONArray()
    val userMarks = JSONArray()
    val modelEvaluations = JSONArray()
    val candidateGroupIds = JSONArray()
    val selectedGroupIds = JSONArray()

    forEach { asset ->
        val groupId = asset.id.ifBlank { asset.groupKey.ifBlank { asset.displayPath } }
        val assetId = "$groupId:primary"
        val path = asset.originalPath?.takeIf { it.isNotBlank() } ?: asset.displayPath
        val filename = syncFilename(asset, groupId)
        val normalizedStem = normalizedStem(filename).ifBlank { normalizedStem(groupId) }
        val receivedAtMs = asset.receivedAt.toLongOrNull()
        val sourceIdentity = asset.displaySource ?: asset.username

        snapshotAssets.put(
            JSONObject()
                .put("asset_id", assetId)
                .put("group_id", groupId)
                .put("original_filename", filename)
                .put("final_filename", lastPathPart(asset.displayPath).ifBlank { filename })
                .put("normalized_stem", normalizedStem)
                .put("original_path", path)
                .put("original_parent_path", parentPath(path))
                .put("format", asset.format.lowercase())
                .put("size_bytes", asset.sizeBytes ?: 0L)
                .put("capture_at_ms", JSONObject.NULL)
                .put("received_at_ms", receivedAtMs ?: JSONObject.NULL)
                .put("source_identity", sourceIdentity ?: JSONObject.NULL),
        )
        snapshotGroups.put(
            JSONObject()
                .put("group_id", groupId)
                .put("display_key", asset.groupKey.ifBlank { normalizedStem })
                .put("source_identity", sourceIdentity ?: JSONObject.NULL)
                .put("original_parent_path", parentPath(path))
                .put("member_asset_ids", JSONArray().put(assetId))
                .put("primary_asset_id", assetId)
                .put("preview_asset_id", assetId)
                .put("has_raw", asset.hasRaw)
                .put("has_jpeg", asset.hasJpeg)
                .put("has_video", asset.hasVideo),
        )

        if (asset.userMarks.favorite || asset.userMarks.marked) {
            userMarks.put(
                JSONObject()
                    .put("group_id", groupId)
                    .put("favorite", asset.userMarks.favorite)
                    .put("marked", asset.userMarks.marked),
            )
        }

        if (asset.modelScore != null) {
            modelEvaluations.put(
                JSONObject()
                    .put("evaluation_id", "lan-share:$groupId")
                    .put("group_id", groupId)
                    .put("evaluator_version", asset.modelEvaluatorKind ?: "android-lan-share")
                    .put("status", asset.modelStatus ?: "ready")
                    .put("score", asset.modelScore)
                    .put("tier", asset.modelTier ?: "")
                    .put("selectable", asset.isModelSelect)
                    .put("summary", asset.modelSummary ?: "")
                    .put("strengths", JSONArray())
                    .put("weaknesses", JSONArray())
                    .put("technical_warnings", JSONArray())
                    .put("prompt_pack_id", JSONObject.NULL)
                    .put("prompt_pack_version", JSONObject.NULL)
                    .put("prompt_hash", JSONObject.NULL)
                    .put("created_at_ms", receivedAtMs ?: exportedAtMs)
                    .put("updated_at_ms", receivedAtMs ?: exportedAtMs),
            )
        }

        candidateGroupIds.put(groupId)
        if (asset.isModelSelect) {
            selectedGroupIds.put(groupId)
        }
    }

    val recommendations = if (selectedGroupIds.length() > 0) {
        JSONArray().put(
            JSONObject()
                .put("recommendation_id", "lan-share:model-select")
                .put("scope", "project")
                .put("subject_group_id", JSONObject.NULL)
                .put("selected_group_ids", selectedGroupIds)
                .put("candidate_group_ids", candidateGroupIds)
                .put("rejected_group_ids", JSONArray())
                .put("status", "ready")
                .put("confidence", 1.0)
                .put("reason", "Imported Android model selections")
                .put("created_at_ms", exportedAtMs)
                .put("updated_at_ms", exportedAtMs),
        )
    } else {
        JSONArray()
    }

    return JSONObject()
        .put("schema_version", 1)
        .put(
            "source_device",
            JSONObject()
                .put("device_id", "lan-share:$token")
                .put("device_label", "Android LAN Share")
                .put("platform", "android"),
        )
        .put(
            "project",
            JSONObject()
                .put("project_id", "lan-share:$token")
                .put("name", "Android LAN Share")
                .put("exported_at_ms", exportedAtMs),
        )
        .put("assets", snapshotAssets)
        .put("groups", snapshotGroups)
        .put("model_evaluations", modelEvaluations)
        .put("selection_recommendations", recommendations)
        .put("user_marks", userMarks)
}

private fun lastPathPart(value: String): String =
    value.trim().replace('\\', '/').substringAfterLast('/')

private fun syncFilename(asset: ProjectAsset, fallback: String): String {
    val displayFilename = lastPathPart(asset.displayPath)
    val originalFilename = asset.originalPath
        ?.takeIf { it.isNotBlank() }
        ?.let(::lastPathPart)
        .orEmpty()
    return originalFilename
        .takeIf(::looksLikeMediaFilename)
        ?: displayFilename.takeIf(::looksLikeMediaFilename)
        ?: displayFilename.ifBlank { originalFilename.ifBlank { fallback } }
}

private fun looksLikeMediaFilename(value: String): Boolean =
    value.substringAfterLast('.', missingDelimiterValue = "")
        .lowercase()
        .let { extension ->
            extension in setOf(
                "jpg",
                "jpeg",
                "heic",
                "heif",
                "png",
                "dng",
                "cr2",
                "cr3",
                "nef",
                "arw",
                "raf",
                "rw2",
                "orf",
                "mov",
                "mp4",
                "m4v",
            )
        }

private fun parentPath(value: String): Any {
    val normalized = value.trim().replace('\\', '/')
    val parent = normalized.substringBeforeLast('/', missingDelimiterValue = "")
    return parent.takeIf { it.isNotBlank() } ?: JSONObject.NULL
}

private fun normalizedStem(value: String): String =
    lastPathPart(value).substringBeforeLast('.', missingDelimiterValue = lastPathPart(value)).lowercase()

internal fun guestMarkFromWire(value: String): GuestMark? =
    when (value.trim().lowercase()) {
        GuestMark.Favorite.wireName -> GuestMark.Favorite
        GuestMark.Marked.wireName -> GuestMark.Marked
        GuestMark.Reject.wireName -> GuestMark.Reject
        else -> null
    }

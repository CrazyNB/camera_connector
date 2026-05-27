package com.cameraconnector.app.storage

import org.json.JSONObject
import java.io.File

interface PublishQueueCore {
    fun claimNextPublishItem(): JSONObject?
    fun markPublishCompleted(queueId: String): JSONObject
    fun markPublishFailed(queueId: String, error: String): JSONObject
}

data class PublishQueueItem(
    val queueId: String,
    val projectId: String,
    val transferId: String,
    val stagedPath: String,
    val finalFilename: String,
    val sizeBytes: Long,
    val state: String,
    val attemptCount: Int,
) {
    companion object {
        fun fromJson(value: JSONObject): PublishQueueItem =
            PublishQueueItem(
                queueId = value.optString("queue_id"),
                projectId = value.optString("project_id"),
                transferId = value.optString("transfer_id"),
                stagedPath = value.optString("staged_path"),
                finalFilename = value.optString("final_filename"),
                sizeBytes = value.optLong("size_bytes"),
                state = value.optString("state"),
                attemptCount = value.optInt("attempt_count"),
            )
    }
}

data class PublishedObject(
    val finalFilename: String,
    val locationKind: String,
    val location: String,
    val sizeBytes: Long,
)

data class PublishDrainResult(
    val claimedCount: Int,
    val completedCount: Int,
    val failedCount: Int,
)

interface PublishTarget {
    fun publish(item: PublishQueueItem): PublishedObject
}

class AndroidPublishWorker(
    private val core: PublishQueueCore,
    private val publishTarget: PublishTarget,
) {
    fun drainOnce(maxItems: Int = DEFAULT_MAX_ITEMS): PublishDrainResult {
        if (maxItems <= 0) {
            return PublishDrainResult(claimedCount = 0, completedCount = 0, failedCount = 0)
        }

        var claimed = 0
        var completed = 0
        var failed = 0

        while (claimed < maxItems) {
            val item = core.claimNextPublishItem()?.let(PublishQueueItem::fromJson) ?: break
            claimed += 1
            try {
                publishTarget.publish(item)
                core.markPublishCompleted(item.queueId)
                completed += 1
            } catch (error: Throwable) {
                core.markPublishFailed(item.queueId, error.message ?: error.toString())
                failed += 1
            }
        }

        return PublishDrainResult(
            claimedCount = claimed,
            completedCount = completed,
            failedCount = failed,
        )
    }

    private companion object {
        const val DEFAULT_MAX_ITEMS = 32
    }
}

class FilePublishTarget(private val outputDir: File) : PublishTarget {
    override fun publish(item: PublishQueueItem): PublishedObject {
        val stagedFile = File(item.stagedPath)
        require(stagedFile.isFile) { "staged file is missing: ${item.stagedPath}" }

        outputDir.mkdirs()
        val finalFile = availableFile(outputDir, item.finalFilename)
        val tempFile = File(finalFile.parentFile, "${finalFile.name}.tmp")
        if (tempFile.exists()) {
            tempFile.delete()
        }

        stagedFile.inputStream().use { input ->
            tempFile.outputStream().use { output ->
                input.copyTo(output)
            }
        }
        if (!tempFile.renameTo(finalFile)) {
            tempFile.copyTo(finalFile, overwrite = true)
            tempFile.delete()
        }
        stagedFile.delete()

        return PublishedObject(
            finalFilename = finalFile.name,
            locationKind = "local_path",
            location = finalFile.absolutePath,
            sizeBytes = finalFile.length(),
        )
    }

    private fun availableFile(outputDir: File, requestedFilename: String): File {
        val flattened = File(requestedFilename).name.ifBlank { "upload" }
        val requested = File(outputDir, flattened)
        if (!requested.exists() && !File(outputDir, "${requested.name}.tmp").exists()) {
            return requested
        }

        val extensionStart = flattened.lastIndexOf('.').takeIf { it > 0 }
        val stem = extensionStart?.let { flattened.substring(0, it) } ?: flattened
        val extension = extensionStart?.let { flattened.substring(it) }.orEmpty()
        var index = 1
        while (true) {
            val candidate = File(outputDir, "$stem ($index)$extension")
            if (!candidate.exists() && !File(outputDir, "${candidate.name}.tmp").exists()) {
                return candidate
            }
            index += 1
        }
    }
}

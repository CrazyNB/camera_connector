package com.cameraconnector.app.storage

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File

class AndroidPublishWorkerTest {
    @get:Rule
    val temporaryFolder = TemporaryFolder()

    @Test
    fun drainOncePublishesClaimedStagedFileAndMarksCompleted() {
        val stagingDir = temporaryFolder.newFolder("staging")
        val outputDir = temporaryFolder.newFolder("output")
        File(outputDir, "IMG_0001.JPG").writeBytes(byteArrayOf(9))
        val staged = File(stagingDir, "upload.staged").also {
            it.writeBytes(byteArrayOf(1, 2, 3))
        }
        val core = FakePublishQueueCore(
            claimedItems = listOf(
                publishItemJson(
                    queueId = "queue-1",
                    stagedPath = staged.absolutePath,
                    finalFilename = "IMG_0001.JPG",
                ),
            ),
        )
        val worker = AndroidPublishWorker(core, FilePublishTarget(outputDir))

        val result = worker.drainOnce(maxItems = 4)

        assertEquals(1, result.completedCount)
        assertEquals(listOf("queue-1"), core.completedQueueIds)
        assertEquals(emptyList<String>(), core.failedQueueIds)
        assertTrue(File(outputDir, "IMG_0001 (1).JPG").readBytes().contentEquals(byteArrayOf(1, 2, 3)))
        assertTrue("staged file should be removed after durable publish", !staged.exists())
    }

    @Test
    fun drainOnceMarksFailedWhenPublishTargetCannotReadStagedFile() {
        val outputDir = temporaryFolder.newFolder("output")
        val missingStaged = File(temporaryFolder.root, "missing.staged")
        val core = FakePublishQueueCore(
            claimedItems = listOf(
                publishItemJson(
                    queueId = "queue-failed",
                    stagedPath = missingStaged.absolutePath,
                    finalFilename = "IMG_0002.JPG",
                ),
            ),
        )
        val worker = AndroidPublishWorker(core, FilePublishTarget(outputDir))

        val result = worker.drainOnce(maxItems = 4)

        assertEquals(1, result.failedCount)
        assertEquals(emptyList<String>(), core.completedQueueIds)
        assertEquals(listOf("queue-failed"), core.failedQueueIds)
        assertTrue(core.failedErrors.single().contains("staged file is missing"))
    }

    @Test
    fun drainOnceMarksFailedWhenCompletedUpdateFails() {
        val core = FakePublishQueueCore(
            claimedItems = listOf(
                publishItemJson(
                    queueId = "queue-update-failed",
                    stagedPath = "already-published.staged",
                    finalFilename = "IMG_0003.JPG",
                ),
            ),
            failCompletedUpdate = true,
        )
        val worker = AndroidPublishWorker(
            core,
            object : PublishTarget {
                override fun publish(item: PublishQueueItem): PublishedObject =
                    PublishedObject(
                        finalFilename = item.finalFilename,
                        locationKind = "local_path",
                        location = item.stagedPath,
                        sizeBytes = item.sizeBytes,
                    )
            },
        )

        val result = worker.drainOnce(maxItems = 4)

        assertEquals(1, result.failedCount)
        assertEquals(listOf("queue-update-failed"), core.failedQueueIds)
        assertTrue(core.failedErrors.single().contains("completed update failed"))
    }


    private fun publishItemJson(
        queueId: String,
        stagedPath: String,
        finalFilename: String,
    ): JSONObject =
        JSONObject()
            .put("queue_id", queueId)
            .put("project_id", "project-1")
            .put("transfer_id", "ftp:1:$finalFilename")
            .put("staged_path", stagedPath)
            .put("final_filename", finalFilename)
            .put("size_bytes", 3)
            .put("state", "Publishing")
            .put("attempt_count", 1)

    private class FakePublishQueueCore(
        claimedItems: List<JSONObject>,
        private val failCompletedUpdate: Boolean = false,
    ) : PublishQueueCore {
        private val claims = ArrayDeque(claimedItems)
        val completedQueueIds = mutableListOf<String>()
        val failedQueueIds = mutableListOf<String>()
        val failedErrors = mutableListOf<String>()

        override fun claimNextPublishItem(): JSONObject? =
            claims.removeFirstOrNull()

        override fun markPublishCompleted(queueId: String): JSONObject {
            if (failCompletedUpdate) {
                error("completed update failed")
            }
            completedQueueIds += queueId
            return JSONObject().put("queue_id", queueId).put("completed", true)
        }

        override fun markPublishFailed(queueId: String, error: String): JSONObject {
            failedQueueIds += queueId
            failedErrors += error
            return JSONObject().put("queue_id", queueId).put("failed", true)
        }
    }
}

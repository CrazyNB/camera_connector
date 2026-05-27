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
        val outputDir = temporaryFolder.newFolder("completion-failed-output")
        val staged = File(temporaryFolder.newFolder("completion-failed-staging"), "upload.staged").also {
            it.writeBytes(byteArrayOf(1, 3, 5))
        }
        val core = FakePublishQueueCore(
            claimedItems = listOf(
                publishItemJson(
                    queueId = "queue-update-failed",
                    stagedPath = staged.absolutePath,
                    finalFilename = "IMG_0003.JPG",
                ),
            ),
            failCompletedUpdate = true,
        )
        val worker = AndroidPublishWorker(core, FilePublishTarget(outputDir))

        val result = worker.drainOnce(maxItems = 4)

        assertEquals(1, result.failedCount)
        assertEquals(listOf("queue-update-failed"), core.failedQueueIds)
        assertTrue(core.failedErrors.single().contains("completed update failed"))
        assertTrue("staged file should remain retryable", staged.exists())
    }

    @Test
    fun safPublishTargetWritesDocumentUriAndPreservesDuplicateFilenameUntilCompletion() {
        val stagingDir = temporaryFolder.newFolder("saf-staging")
        val staged = File(stagingDir, "upload.staged").also {
            it.writeBytes(byteArrayOf(4, 5, 6))
        }
        val documentStore = FakeDocumentTreeStore(existingNames = setOf("IMG_0100.JPG"))
        val target = SafPublishTarget(documentStore)

        val published = target.publish(
            PublishQueueItem(
                queueId = "queue-saf",
                projectId = "project-1",
                transferId = "ftp:1:IMG_0100.JPG",
                stagedPath = staged.absolutePath,
                finalFilename = "IMG_0100.JPG",
                sizeBytes = 3,
                state = "Publishing",
                attemptCount = 1,
            ),
        )

        assertEquals("IMG_0100 (1).JPG", published.finalFilename)
        assertEquals("document_uri", published.locationKind)
        assertEquals("content://picked-tree/IMG_0100%20(1).JPG", published.location)
        assertEquals(byteArrayOf(4, 5, 6).toList(), documentStore.writes.single().bytes.toList())
        assertTrue("staged file is removed only after core completion", staged.exists())
    }

    @Test
    fun safPublishTargetKeepsStagedFileWhenDocumentWriteFails() {
        val staged = File(temporaryFolder.newFolder("saf-failed-staging"), "upload.staged").also {
            it.writeBytes(byteArrayOf(7, 8, 9))
        }
        val target = SafPublishTarget(FakeDocumentTreeStore(failWrite = true))

        val error = runCatching {
            target.publish(
                PublishQueueItem(
                    queueId = "queue-saf-failed",
                    projectId = "project-1",
                    transferId = "ftp:1:IMG_0200.JPG",
                    stagedPath = staged.absolutePath,
                    finalFilename = "IMG_0200.JPG",
                    sizeBytes = 3,
                    state = "Publishing",
                    attemptCount = 1,
                ),
            )
        }.exceptionOrNull()

        assertTrue(error?.message?.contains("document write failed") == true)
        assertTrue("staged file should remain retryable", staged.exists())
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

        override fun completePublish(queueId: String, publishedObject: PublishedObject): JSONObject {
            if (failCompletedUpdate) {
                error("completed update failed")
            }
            completedQueueIds += queueId
            return JSONObject()
                .put("queue_id", queueId)
                .put("final_filename", publishedObject.finalFilename)
        }

        override fun markPublishFailed(queueId: String, error: String): JSONObject {
            failedQueueIds += queueId
            failedErrors += error
            return JSONObject().put("queue_id", queueId).put("failed", true)
        }
    }

    private class FakeDocumentTreeStore(
        existingNames: Set<String> = emptySet(),
        private val failWrite: Boolean = false,
    ) : DocumentTreeStore {
        private val names = existingNames.toMutableSet()
        val writes = mutableListOf<DocumentWrite>()

        override fun exists(displayName: String): Boolean = names.contains(displayName)

        override fun write(
            displayName: String,
            mimeType: String,
            source: File,
        ): String {
            if (failWrite) {
                error("document write failed")
            }
            names += displayName
            writes += DocumentWrite(displayName, mimeType, source.readBytes())
            return "content://picked-tree/${displayName.replace(" ", "%20")}"
        }
    }

    private data class DocumentWrite(
        val displayName: String,
        val mimeType: String,
        val bytes: ByteArray,
    )
}

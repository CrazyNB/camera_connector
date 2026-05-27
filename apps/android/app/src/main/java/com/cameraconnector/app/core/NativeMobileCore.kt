package com.cameraconnector.app.core

import org.json.JSONArray
import org.json.JSONObject

class NativeMobileCore(configPath: String?) : AutoCloseable {
    private var handle: Long = create(configPath)

    fun projectDashboardJson(projectId: String, offset: Int, limit: Int): JSONObject =
        call { projectDashboardJson(handle, projectId, offset, limit) }

    fun projectGroupAssetsJson(projectId: String, groupId: String): JSONArray =
        call { projectGroupAssetsJson(handle, projectId, groupId) }.optJSONArray("value")
            ?: JSONArray()

    fun claimNextPublishItem(): JSONObject? {
        val value = call { claimNextPublishItemJson(handle) }
        return if (value.has("value") && value.isNull("value")) {
            null
        } else {
            value
        }
    }

    fun markPublishCompleted(queueId: String): JSONObject =
        call { markPublishCompletedJson(handle, queueId) }

    fun markPublishFailed(queueId: String, error: String): JSONObject =
        call { markPublishFailedJson(handle, queueId, error) }

    fun createProject(name: String): JSONObject =
        call { createProjectJson(handle, name) }

    fun listProjects(): JSONArray =
        call { listProjectsJson(handle) }.optJSONArray("value") ?: JSONArray()

    fun setActiveProject(projectId: String): JSONObject =
        call { setActiveProjectJson(handle, projectId) }

    fun archiveProject(projectId: String): JSONObject =
        call { archiveProjectJson(handle, projectId) }

    fun restoreProject(projectId: String): JSONObject =
        call { restoreProjectJson(handle, projectId) }

    fun ensureActiveProject(): JSONObject =
        call { ensureActiveProjectJson(handle) }

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
        val patch = JSONObject()
            .put("output_dir", outputDir)
            .put("state_dir", stateDir)
        call { saveReceiverSettingsJson(handle, patch.toString()) }
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
    private external fun projectGroupAssetsJson(handle: Long, projectId: String, groupId: String): String
    private external fun claimNextPublishItemJson(handle: Long): String
    private external fun markPublishCompletedJson(handle: Long, queueId: String): String
    private external fun markPublishFailedJson(handle: Long, queueId: String, error: String): String
    private external fun createProjectJson(handle: Long, name: String): String
    private external fun listProjectsJson(handle: Long): String
    private external fun setActiveProjectJson(handle: Long, projectId: String): String
    private external fun archiveProjectJson(handle: Long, projectId: String): String
    private external fun restoreProjectJson(handle: Long, projectId: String): String
    private external fun activeProjectJson(handle: Long): String
    private external fun ensureActiveProjectJson(handle: Long): String
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

internal fun receiverSettingsPatchFields(settings: ReceiverSettings): Map<String, Any> =
    mapOf(
        "protocol" to settings.protocol.lowercase(),
        "bind_host" to settings.host,
        "ftp_port" to settings.ftpPort,
        "sftp_port" to settings.sftpPort,
    )

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

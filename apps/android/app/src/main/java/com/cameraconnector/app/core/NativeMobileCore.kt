package com.cameraconnector.app.core

import org.json.JSONObject

class NativeMobileCore(configPath: String?) : AutoCloseable {
    private var handle: Long = create(configPath)

    fun dashboardJson(stateDir: String?, offset: Int, limit: Int): JSONObject =
        call { dashboardJson(handle, stateDir, offset, limit) }

    fun saveReceiverSettings(settings: ReceiverSettings) {
        val patch = JSONObject()
            .put("protocol", settings.protocol.lowercase())
            .put("bind_host", settings.host)
            .put("ftp_port", settings.ftpPort)
            .put("sftp_port", settings.sftpPort)
            .put("output_dir", settings.outputLabel)
        call { saveReceiverSettingsJson(handle, patch.toString()) }
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
    private external fun dashboardJson(handle: Long, stateDir: String?, offset: Int, limit: Int): String
    private external fun saveReceiverSettingsJson(handle: Long, patchJson: String): String
    private external fun saveDeviceAccountJson(
        handle: Long,
        username: String,
        password: String?,
        deviceName: String,
    ): String
    private external fun startReceiverJson(handle: Long): String
    private external fun stopReceiverJson(handle: Long): String

    companion object {
        init {
            System.loadLibrary("camera_connector_ffi")
        }
    }
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

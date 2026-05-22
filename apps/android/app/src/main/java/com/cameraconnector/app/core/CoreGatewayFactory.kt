package com.cameraconnector.app.core

import android.content.Context
import com.cameraconnector.app.BuildConfig
import com.cameraconnector.app.service.ReceiverServiceController
import java.io.File

object CoreGatewayFactory {
    fun create(context: Context): CoreGateway {
        if (!BuildConfig.USE_NATIVE_CORE) {
            return PreviewCoreGateway()
        }

        return runCatching {
            val appContext = context.applicationContext
            val configFile = File(appContext.filesDir, "camera-connector.json")
            val inboxDir = File(appContext.filesDir, "inbox").also { it.mkdirs() }
            val stateDir = File(appContext.filesDir, "state").also { it.mkdirs() }
            val nativeCore = NativeMobileCore(configFile.absolutePath).also {
                it.saveAndroidReceiverPaths(
                    outputDir = inboxDir.absolutePath,
                    stateDir = stateDir.absolutePath,
                )
            }

            NativeCoreGateway(
                nativeCore = nativeCore,
                stateDir = stateDir.absolutePath,
                receiverServiceController = ReceiverServiceController(
                    appContext,
                    configFile.absolutePath,
                    stateDir.absolutePath,
                ),
            )
        }.getOrElse { error ->
            if (BuildConfig.NATIVE_CORE_FALLBACK_TO_PREVIEW) {
                PreviewCoreGateway()
            } else {
                throw error
            }
        }
    }
}

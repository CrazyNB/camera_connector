package com.cameraconnector.app.core

import android.content.Context
import com.cameraconnector.app.BuildConfig
import java.io.File

object CoreGatewayFactory {
    fun create(context: Context): CoreGateway {
        if (!BuildConfig.USE_NATIVE_CORE) {
            return PreviewCoreGateway()
        }

        return runCatching {
            val appContext = context.applicationContext
            val configFile = File(appContext.filesDir, "camera-connector.json")
            val stateDir = File(appContext.filesDir, "state").also { it.mkdirs() }

            NativeCoreGateway(
                nativeCore = NativeMobileCore(configFile.absolutePath),
                stateDir = stateDir.absolutePath,
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

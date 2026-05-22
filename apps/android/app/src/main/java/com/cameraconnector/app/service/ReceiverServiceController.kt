package com.cameraconnector.app.service

import android.content.Context
import androidx.core.content.ContextCompat

class ReceiverServiceController(
    context: Context,
    private val configPath: String,
    private val stateDir: String,
) {
    private val appContext = context.applicationContext

    fun startReceiver() {
        ContextCompat.startForegroundService(
            appContext,
            ReceiverForegroundService.startIntent(appContext, configPath, stateDir),
        )
    }

    fun stopReceiver() {
        appContext.startService(
            ReceiverForegroundService.stopIntent(appContext, configPath, stateDir),
        )
    }
}

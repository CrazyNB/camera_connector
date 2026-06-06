package com.cameraconnector.app.service

import android.content.Context
import androidx.core.content.ContextCompat

class ReceiverServiceController internal constructor(
    private val configPath: String,
    private val stateDir: String,
    private val starter: ReceiverServiceStarter,
) {
    constructor(
        context: Context,
        configPath: String,
        stateDir: String,
    ) : this(
        configPath = configPath,
        stateDir = stateDir,
        starter = AndroidReceiverServiceStarter(context.applicationContext),
    )

    fun startReceiver() {
        starter.startReceiver(configPath, stateDir)
    }

    fun stopReceiver() {
        starter.stopReceiver(configPath, stateDir)
    }

    fun retryFailedPublishes() {
        starter.retryFailedPublishes(configPath, stateDir)
    }
}

internal interface ReceiverServiceStarter {
    fun startReceiver(configPath: String, stateDir: String)
    fun stopReceiver(configPath: String, stateDir: String)
    fun retryFailedPublishes(configPath: String, stateDir: String)
}

private class AndroidReceiverServiceStarter(private val appContext: Context) : ReceiverServiceStarter {
    override fun startReceiver(configPath: String, stateDir: String) {
        ContextCompat.startForegroundService(
            appContext,
            ReceiverForegroundService.startIntent(appContext, configPath, stateDir),
        )
    }

    override fun stopReceiver(configPath: String, stateDir: String) {
        appContext.startService(
            ReceiverForegroundService.stopIntent(appContext, configPath, stateDir),
        )
    }

    override fun retryFailedPublishes(configPath: String, stateDir: String) {
        ContextCompat.startForegroundService(
            appContext,
            ReceiverForegroundService.retryPublishIntent(appContext, configPath, stateDir),
        )
    }
}

package com.cameraconnector.app.service

import org.junit.Assert.assertEquals
import org.junit.Test

class ReceiverServiceControllerTest {
    @Test
    fun retryFailedPublishesStartsOneShotPublishDrain() {
        val starter = RecordingReceiverServiceStarter()
        val controller = ReceiverServiceController(
            configPath = "config.json",
            stateDir = "state",
            starter = starter,
        )

        controller.retryFailedPublishes()

        assertEquals(
            listOf("retry:config.json:state"),
            starter.commands,
        )
    }

    private class RecordingReceiverServiceStarter : ReceiverServiceStarter {
        val commands = mutableListOf<String>()

        override fun startReceiver(configPath: String, stateDir: String) {
            commands += "start:$configPath:$stateDir"
        }

        override fun stopReceiver(configPath: String, stateDir: String) {
            commands += "stop:$configPath:$stateDir"
        }

        override fun retryFailedPublishes(configPath: String, stateDir: String) {
            commands += "retry:$configPath:$stateDir"
        }
    }
}

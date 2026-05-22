package com.cameraconnector.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.CoreGatewayFactory
import com.cameraconnector.app.storage.AndroidStorageGateway
import com.cameraconnector.app.ui.CameraConnectorApp

class MainActivity : ComponentActivity() {
    private lateinit var coreGateway: CoreGateway

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        coreGateway = CoreGatewayFactory.create(this)
        val storageGateway = AndroidStorageGateway(this)

        setContent {
            CameraConnectorApp(
                coreGateway = coreGateway,
                storageGateway = storageGateway,
            )
        }
    }

    override fun onDestroy() {
        (coreGateway as? AutoCloseable)?.close()
        super.onDestroy()
    }
}

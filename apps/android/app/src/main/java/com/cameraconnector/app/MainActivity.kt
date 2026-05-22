package com.cameraconnector.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import com.cameraconnector.app.core.PreviewCoreGateway
import com.cameraconnector.app.storage.AndroidStorageGateway
import com.cameraconnector.app.ui.CameraConnectorApp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val coreGateway = PreviewCoreGateway()
        val storageGateway = AndroidStorageGateway(this)

        setContent {
            CameraConnectorApp(
                coreGateway = coreGateway,
                storageGateway = storageGateway,
            )
        }
    }
}

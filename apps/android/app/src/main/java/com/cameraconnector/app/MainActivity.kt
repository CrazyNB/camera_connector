package com.cameraconnector.app

import android.os.Bundle
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.CoreGatewayFactory
import com.cameraconnector.app.permissions.AndroidPermissionGateway
import com.cameraconnector.app.storage.AndroidStorageGateway
import com.cameraconnector.app.ui.CameraConnectorApp
import kotlinx.coroutines.flow.MutableStateFlow

class MainActivity : ComponentActivity() {
    private lateinit var coreGateway: CoreGateway
    private lateinit var permissionGateway: AndroidPermissionGateway
    private val notificationPermissionGranted = MutableStateFlow(true)
    private val requestNotificationPermission = registerForActivityResult(
        ActivityResultContracts.RequestPermission(),
    ) {
        notificationPermissionGranted.value = permissionGateway.hasNotificationPermission()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        coreGateway = CoreGatewayFactory.create(this)
        permissionGateway = AndroidPermissionGateway(this)
        notificationPermissionGranted.value = permissionGateway.hasNotificationPermission()
        val storageGateway = AndroidStorageGateway(this)

        setContent {
            CameraConnectorApp(
                coreGateway = coreGateway,
                storageGateway = storageGateway,
                notificationPermissionRequired = permissionGateway.notificationPermissionRequired(),
                notificationPermissionGranted = notificationPermissionGranted,
                onRequestNotificationPermission = {
                    requestNotificationPermission.launch(permissionGateway.notificationPermission())
                },
            )
        }
    }

    override fun onDestroy() {
        (coreGateway as? AutoCloseable)?.close()
        super.onDestroy()
    }
}

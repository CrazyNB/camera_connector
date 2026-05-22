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
    private lateinit var storageGateway: AndroidStorageGateway
    private val notificationPermissionGranted = MutableStateFlow(true)
    private val selectedInboxLabel = MutableStateFlow<String?>(null)
    private val requestNotificationPermission = registerForActivityResult(
        ActivityResultContracts.RequestPermission(),
    ) {
        notificationPermissionGranted.value = permissionGateway.hasNotificationPermission()
    }
    private val requestInboxDirectory = registerForActivityResult(
        ActivityResultContracts.OpenDocumentTree(),
    ) { uri ->
        if (uri != null) {
            storageGateway.persistInboxDirectory(uri)
            selectedInboxLabel.value = storageGateway.selectedInboxLabel()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        coreGateway = CoreGatewayFactory.create(this)
        permissionGateway = AndroidPermissionGateway(this)
        notificationPermissionGranted.value = permissionGateway.hasNotificationPermission()
        storageGateway = AndroidStorageGateway(this)
        selectedInboxLabel.value = storageGateway.selectedInboxLabel()

        setContent {
            CameraConnectorApp(
                coreGateway = coreGateway,
                storageGateway = storageGateway,
                notificationPermissionRequired = permissionGateway.notificationPermissionRequired(),
                notificationPermissionGranted = notificationPermissionGranted,
                selectedInboxLabel = selectedInboxLabel,
                onRequestNotificationPermission = {
                    requestNotificationPermission.launch(permissionGateway.notificationPermission())
                },
                onChooseInboxDirectory = {
                    requestInboxDirectory.launch(null)
                },
            )
        }
    }

    override fun onDestroy() {
        (coreGateway as? AutoCloseable)?.close()
        super.onDestroy()
    }
}

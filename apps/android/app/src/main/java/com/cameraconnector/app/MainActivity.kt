package com.cameraconnector.app

import android.os.Bundle
import android.view.Gravity
import android.view.ViewGroup
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.TextView
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.lifecycle.lifecycleScope
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.CoreGatewayFactory
import com.cameraconnector.app.permissions.AndroidPermissionGateway
import com.cameraconnector.app.storage.AndroidStorageGateway
import com.cameraconnector.app.ui.CameraConnectorApp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class MainActivity : ComponentActivity() {
    private val coreGateway = MutableStateFlow<CoreGateway?>(null)
    private lateinit var permissionGateway: AndroidPermissionGateway
    private lateinit var storageGateway: AndroidStorageGateway
    private val notificationPermissionGranted = MutableStateFlow(true)
    private val selectedOutputLabel = MutableStateFlow<String?>(null)
    private val requestNotificationPermission = registerForActivityResult(
        ActivityResultContracts.RequestPermission(),
    ) {
        notificationPermissionGranted.value = permissionGateway.hasNotificationPermission()
    }
    private val requestOutputDirectory = registerForActivityResult(
        ActivityResultContracts.OpenDocumentTree(),
    ) { uri ->
        if (uri != null) {
            storageGateway.persistOutputDirectory(uri)
            selectedOutputLabel.value = storageGateway.selectedOutputLabel()
            lifecycleScope.launch {
                runCatching { coreGateway.value?.retryFailedPublishes() }
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        permissionGateway = AndroidPermissionGateway(this)
        notificationPermissionGranted.value = permissionGateway.hasNotificationPermission()
        storageGateway = AndroidStorageGateway(this)
        selectedOutputLabel.value = storageGateway.selectedOutputLabel()

        showStartupView {
            startCoreInitialization()
        }
    }

    private fun startCoreInitialization() {
        lifecycleScope.launch(Dispatchers.IO) {
            val createdGateway = CoreGatewayFactory.create(this@MainActivity)
            withContext(Dispatchers.Main) {
                coreGateway.value = createdGateway
                setMainContent(createdGateway)
            }
        }
    }

    private fun showStartupView(onStartupVisible: () -> Unit) {
        val root = FrameLayout(this).apply {
            setBackgroundColor(0xFF071018.toInt())
        }
        val content = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
        }
        val progress = ProgressBar(this).apply {
            isIndeterminate = true
        }
        val title = TextView(this).apply {
            text = "\u6b63\u5728\u521d\u59cb\u5316\u6838\u5fc3"
            setTextColor(0xFFEAF6FF.toInt())
            textSize = 16f
            gravity = Gravity.CENTER
            setPadding(0, 24, 0, 0)
        }
        content.addView(
            progress,
            LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT,
                ViewGroup.LayoutParams.WRAP_CONTENT,
            ),
        )
        content.addView(
            title,
            LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT,
                ViewGroup.LayoutParams.WRAP_CONTENT,
            ),
        )
        root.addView(
            content,
            FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.MATCH_PARENT,
            ),
        )
        setContentView(root)
        root.postDelayed(onStartupVisible, STARTUP_CORE_INIT_DELAY_MS)
    }

    private fun setMainContent(activeCoreGateway: CoreGateway) {
        setContent {
            CameraConnectorApp(
                coreGateway = activeCoreGateway,
                storageGateway = storageGateway,
                notificationPermissionRequired = permissionGateway.notificationPermissionRequired(),
                notificationPermissionGranted = notificationPermissionGranted,
                selectedOutputLabel = selectedOutputLabel,
                onRequestNotificationPermission = {
                    requestNotificationPermission.launch(permissionGateway.notificationPermission())
                },
                onChooseOutputDirectory = {
                    requestOutputDirectory.launch(null)
                },
            )
        }
    }

    override fun onDestroy() {
        (coreGateway.value as? AutoCloseable)?.close()
        super.onDestroy()
    }
}

private const val STARTUP_CORE_INIT_DELAY_MS = 500L

package com.cameraconnector.app.service

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.cameraconnector.app.MainActivity
import com.cameraconnector.app.core.NativeMobileCore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch

class ReceiverForegroundService : Service() {
    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var nativeCore: NativeMobileCore? = null

    override fun onCreate() {
        super.onCreate()
        ensureNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return when (intent?.action) {
            ACTION_STOP -> {
                serviceScope.launch {
                    stopNativeReceiver()
                    stopForeground(STOP_FOREGROUND_REMOVE)
                    stopSelf(startId)
                }
                START_NOT_STICKY
            }

            else -> {
                startForeground(NOTIFICATION_ID, notification("Starting receiver"))
                val configPath = intent?.getStringExtra(EXTRA_CONFIG_PATH)
                serviceScope.launch {
                    startNativeReceiver(configPath)
                }
                START_STICKY
            }
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        stopNativeReceiver()
        serviceScope.cancel()
        super.onDestroy()
    }

    private fun startNativeReceiver(configPath: String?) {
        runCatching {
            if (nativeCore != null) {
                startForeground(NOTIFICATION_ID, notification("Receiver is already running"))
                return
            }
            val core = nativeCore ?: NativeMobileCore(configPath).also { nativeCore = it }
            val status = core.startReceiver()
            val localAddr = status.optString("local_addr").ifBlank { "ready" }
            startForeground(NOTIFICATION_ID, notification("Receiver running at $localAddr"))
        }.onFailure { error ->
            startForeground(NOTIFICATION_ID, notification("Receiver failed: ${error.message}"))
        }
    }

    private fun stopNativeReceiver() {
        runCatching {
            nativeCore?.stopReceiver()
        }
        nativeCore?.close()
        nativeCore = null
    }

    private fun notification(message: String) =
        NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.stat_sys_upload_done)
            .setContentTitle("Camera Connector")
            .setContentText(message)
            .setContentIntent(openAppPendingIntent())
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .addAction(0, "Stop", stopReceiverPendingIntent())
            .build()

    private fun openAppPendingIntent(): PendingIntent =
        PendingIntent.getActivity(
            this,
            OPEN_APP_REQUEST_CODE,
            Intent(this, MainActivity::class.java)
                .setFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )

    private fun stopReceiverPendingIntent(): PendingIntent =
        PendingIntent.getService(
            this,
            STOP_RECEIVER_REQUEST_CODE,
            Intent(this, ReceiverForegroundService::class.java).setAction(ACTION_STOP),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )

    private fun ensureNotificationChannel() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
            return
        }

        val channel = NotificationChannel(
            CHANNEL_ID,
            "Receiver status",
            NotificationManager.IMPORTANCE_LOW,
        )
        val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.createNotificationChannel(channel)
    }

    companion object {
        const val ACTION_START = "com.cameraconnector.app.receiver.START"
        const val ACTION_STOP = "com.cameraconnector.app.receiver.STOP"

        private const val CHANNEL_ID = "camera_connector_receiver"
        private const val NOTIFICATION_ID = 1201
        private const val OPEN_APP_REQUEST_CODE = 1202
        private const val STOP_RECEIVER_REQUEST_CODE = 1203
        private const val EXTRA_CONFIG_PATH = "config_path"
        private const val EXTRA_STATE_DIR = "state_dir"

        fun startIntent(context: Context, configPath: String, stateDir: String): Intent =
            Intent(context, ReceiverForegroundService::class.java)
                .setAction(ACTION_START)
                .putExtra(EXTRA_CONFIG_PATH, configPath)
                .putExtra(EXTRA_STATE_DIR, stateDir)

        fun stopIntent(context: Context, configPath: String, stateDir: String): Intent =
            Intent(context, ReceiverForegroundService::class.java)
                .setAction(ACTION_STOP)
                .putExtra(EXTRA_CONFIG_PATH, configPath)
                .putExtra(EXTRA_STATE_DIR, stateDir)
    }
}

package com.cameraconnector.app.service

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import com.cameraconnector.app.MainActivity
import com.cameraconnector.app.core.NativeMobileCore
import com.cameraconnector.app.storage.AndroidDocumentTreeStore
import com.cameraconnector.app.storage.AndroidPublishWorker
import com.cameraconnector.app.storage.AndroidStorageGateway
import com.cameraconnector.app.storage.FilePublishTarget
import com.cameraconnector.app.storage.ResolvingPublishTarget
import com.cameraconnector.app.storage.SafPublishTarget
import com.cameraconnector.app.storage.ThumbnailingPublishTarget
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import org.json.JSONObject
import java.io.File

class ReceiverForegroundService : Service() {
    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var nativeCore: NativeMobileCore? = null
    private var publishWorkerJob: Job? = null

    override fun onCreate() {
        super.onCreate()
        ensureNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return when (intent?.action) {
            ACTION_STOP -> {
                serviceScope.launch {
                    Log.i(LOG_TAG, "receiver stop requested")
                    stopNativeReceiver()
                    stopForeground(STOP_FOREGROUND_REMOVE)
                    stopSelf(startId)
                }
                START_NOT_STICKY
            }

            ACTION_RETRY_PUBLISH -> {
                startForeground(NOTIFICATION_ID, notification("\u6b63\u5728\u91cd\u8bd5\u5199\u5165"))
                val configPath = intent.getStringExtra(EXTRA_CONFIG_PATH)
                Log.i(LOG_TAG, "publish retry requested configPath=$configPath")
                serviceScope.launch {
                    drainPublishQueueOnce(configPath)
                    if (nativeCore == null) {
                        stopForeground(STOP_FOREGROUND_REMOVE)
                        stopSelf(startId)
                    } else {
                        startForeground(NOTIFICATION_ID, notification("\u63a5\u6536\u670d\u52a1\u8fd0\u884c\u4e2d"))
                    }
                }
                START_NOT_STICKY
            }

            else -> {
                startForeground(NOTIFICATION_ID, notification("正在启动接收服务"))
                val configPath = intent?.getStringExtra(EXTRA_CONFIG_PATH)
                Log.i(LOG_TAG, "receiver start requested configPath=$configPath")
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
                Log.i(LOG_TAG, "receiver start ignored because native core is already running")
                nativeCore?.let(::startPublishWorker)
                startForeground(NOTIFICATION_ID, notification("接收服务已在运行"))
                return
            }
            val core = nativeCore ?: NativeMobileCore(configPath).also { nativeCore = it }
            val status = core.startReceiver()
            val localAddr = status.optString("local_addr").ifBlank { "就绪" }
            Log.i(LOG_TAG, "receiver started localAddr=$localAddr")
            startPublishWorker(core)
            startForeground(NOTIFICATION_ID, notification("接收服务运行中：$localAddr"))
        }.onFailure { error ->
            Log.e(LOG_TAG, "receiver start failed", error)
            stopPublishWorker()
            nativeCore?.close()
            nativeCore = null
            startForeground(NOTIFICATION_ID, notification("接收服务失败：${error.message}"))
        }
    }

    private fun stopNativeReceiver() {
        stopPublishWorker()
        runCatching {
            nativeCore?.stopReceiver()
            Log.i(LOG_TAG, "receiver stopped")
        }.onFailure { error ->
            Log.e(LOG_TAG, "receiver stop failed", error)
        }
        nativeCore?.close()
        nativeCore = null
    }

    private fun startPublishWorker(core: NativeMobileCore) {
        if (publishWorkerJob?.isActive == true) {
            return
        }

        val worker = createPublishWorker(core)
        val smartSelectionWorker = SmartSelectionAnalysisWorker(this, core)
        publishWorkerJob = serviceScope.launch {
            while (isActive) {
                runCatching {
                    worker.drainOnce()
                }.onSuccess { result ->
                    val analysis = runCatching { drainAnalysisJobsWithProviderState(core) }.getOrNull()
                    val smartSelection = runCatching { smartSelectionWorker.drainOnce() }.getOrNull()
                    if (result.completedCount > 0 || result.failedCount > 0) {
                        Log.i(
                            LOG_TAG,
                            "publish queue drained completed=${result.completedCount} failed=${result.failedCount}",
                        )
                    }
                    if (analysis?.optInt("completed_count")?.takeIf { it > 0 } != null) {
                        Log.i(
                            LOG_TAG,
                            "analysis queue drained completed=${analysis.optInt("completed_count")}",
                        )
                    }
                    if (
                        smartSelection != null &&
                        (smartSelection.assessedCount > 0 ||
                            smartSelection.recommendedCount > 0 ||
                            smartSelection.failedCount > 0)
                    ) {
                        Log.i(
                            LOG_TAG,
                            "smart selection drained assessed=${smartSelection.assessedCount} recommended=${smartSelection.recommendedCount} failed=${smartSelection.failedCount}",
                        )
                    }
                }.onFailure { error ->
                    Log.e(LOG_TAG, "publish queue drain failed", error)
                }
                delay(PUBLISH_QUEUE_POLL_INTERVAL_MS)
            }
        }
    }

    private fun drainPublishQueueOnce(configPath: String?) {
        val existingCore = nativeCore
        val ownsCore = existingCore == null
        val core = existingCore ?: NativeMobileCore(configPath)
        runCatching {
            val result = createPublishWorker(core).drainOnce()
            val analysis = drainAnalysisJobsWithProviderState(core)
            val smartSelection = SmartSelectionAnalysisWorker(this, core).drainOnce()
            Triple(result, analysis, smartSelection)
        }.onSuccess { result ->
            Log.i(
                LOG_TAG,
                "publish retry drained completed=${result.first.completedCount} failed=${result.first.failedCount} analysis=${result.second.optInt("completed_count")} smart_assessed=${result.third.assessedCount} smart_recommended=${result.third.recommendedCount}",
            )
        }.onFailure { error ->
            Log.e(LOG_TAG, "publish retry drain failed", error)
        }
        if (ownsCore) {
            core.close()
        }
    }

    private fun createPublishWorker(core: NativeMobileCore): AndroidPublishWorker {
        val outputDir = File(filesDir, "output").also { it.mkdirs() }
        val storageGateway = AndroidStorageGateway(this)
        val publishTarget = ResolvingPublishTarget {
            val target = storageGateway.selectedOutputUri()
                ?.let { uri -> SafPublishTarget(AndroidDocumentTreeStore(this, uri)) }
                ?: FilePublishTarget(outputDir)
            ThumbnailingPublishTarget(this, target)
        }
        return AndroidPublishWorker(
            core = core,
            publishTarget = publishTarget,
        )
    }

    private fun drainAnalysisJobsWithProviderState(core: NativeMobileCore): JSONObject {
        val smartSelectionCore = NativeSmartSelectionCore(core)
        val projectSettings = smartSelectionCore.activeProject()
            ?.optString("project_id")
            ?.takeIf { it.isNotBlank() }
            ?.let { projectId -> smartSelectionCore.projectEvaluationSettings(projectId) }
        val providerConfigured = projectSettings?.let { settings ->
            providerConfiguredForProject(
                projectSettings = settings,
                providerOptions = smartSelectionCore.modelProviderSettingsList(),
            )
        } ?: false
        return smartSelectionCore.drainAnalysisJobsWithProviderConfigured(providerConfigured = providerConfigured)
    }

    private fun stopPublishWorker() {
        publishWorkerJob?.cancel()
        publishWorkerJob = null
    }

    private fun notification(message: String) =
        NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.stat_sys_upload_done)
            .setContentTitle("\u76f8\u673a\u8fde\u63a5\u5668")
            .setContentText(message)
            .setContentIntent(openAppPendingIntent())
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .addAction(0, "停止", stopReceiverPendingIntent())
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
            "\u63a5\u6536\u670d\u52a1\u72b6\u6001",
            NotificationManager.IMPORTANCE_LOW,
        )
        val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.createNotificationChannel(channel)
    }

    companion object {
        const val ACTION_START = "com.cameraconnector.app.receiver.START"
        const val ACTION_STOP = "com.cameraconnector.app.receiver.STOP"
        const val ACTION_RETRY_PUBLISH = "com.cameraconnector.app.receiver.RETRY_PUBLISH"

        private const val LOG_TAG = "CameraConnectorReceiver"
        private const val CHANNEL_ID = "camera_connector_receiver"
        private const val NOTIFICATION_ID = 1201
        private const val OPEN_APP_REQUEST_CODE = 1202
        private const val STOP_RECEIVER_REQUEST_CODE = 1203
        private const val PUBLISH_QUEUE_POLL_INTERVAL_MS = 2_000L
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

        fun retryPublishIntent(context: Context, configPath: String, stateDir: String): Intent =
            Intent(context, ReceiverForegroundService::class.java)
                .setAction(ACTION_RETRY_PUBLISH)
                .putExtra(EXTRA_CONFIG_PATH, configPath)
                .putExtra(EXTRA_STATE_DIR, stateDir)
    }
}

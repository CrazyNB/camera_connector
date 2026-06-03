package com.cameraconnector.app.storage

import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.net.Uri
import android.provider.Settings
import com.cameraconnector.app.core.DEFAULT_CAMERA_CONNECT_HOST

class AndroidStorageGateway(private val context: Context) {
    private val preferences: SharedPreferences =
        context.getSharedPreferences("camera_connector_storage", Context.MODE_PRIVATE)

    fun createInboxDirectoryIntent(): Intent =
        Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
            addFlags(Intent.FLAG_GRANT_PREFIX_URI_PERMISSION)
        }

    fun persistInboxDirectory(uri: Uri) {
        val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or
            Intent.FLAG_GRANT_WRITE_URI_PERMISSION
        context.contentResolver.takePersistableUriPermission(uri, flags)
        preferences.edit()
            .putString(KEY_INBOX_URI, uri.toString())
            .putString(KEY_INBOX_LABEL, uri.lastPathSegment ?: uri.toString())
            .apply()
    }

    fun selectedInboxLabel(): String? {
        return preferences.getString(KEY_INBOX_LABEL, null)
    }

    fun selectedInboxUri(): Uri? =
        preferences.getString(KEY_INBOX_URI, null)?.let(Uri::parse)

    fun projectPhotoGridColumnCount(): Int =
        preferences.getInt(KEY_PROJECT_PHOTO_GRID_COLUMNS, DEFAULT_PROJECT_PHOTO_GRID_COLUMNS).coerceIn(2, 3)

    fun persistProjectPhotoGridColumnCount(columnCount: Int) {
        preferences.edit()
            .putInt(KEY_PROJECT_PHOTO_GRID_COLUMNS, columnCount.coerceIn(2, 3))
            .apply()
    }

    fun cameraConnectHost(): String =
        preferences.getString(KEY_CAMERA_CONNECT_HOST, null)
            ?.trim()
            ?.takeIf { it.isNotBlank() }
            ?: DEFAULT_CAMERA_CONNECT_HOST

    fun persistCameraConnectHost(host: String) {
        preferences.edit()
            .putString(KEY_CAMERA_CONNECT_HOST, host.trim().ifBlank { DEFAULT_CAMERA_CONNECT_HOST })
            .apply()
    }

    fun smartSelectionStrategyProfileId(): String =
        preferences.getString(KEY_SMART_SELECTION_STRATEGY_PROFILE_ID, null)
            ?.trim()
            ?.takeIf { it.isNotBlank() }
            ?: DEFAULT_SMART_SELECTION_STRATEGY_PROFILE_ID

    fun persistSmartSelectionStrategyProfileId(profileId: String) {
        preferences.edit()
            .putString(
                KEY_SMART_SELECTION_STRATEGY_PROFILE_ID,
                profileId.trim().ifBlank { DEFAULT_SMART_SELECTION_STRATEGY_PROFILE_ID },
            )
            .apply()
    }

    fun modelProviderKeyAlias(): String? =
        preferences.getString(KEY_MODEL_PROVIDER_KEY_ALIAS, null)
            ?.trim()
            ?.takeIf { it.isNotBlank() }

    fun modelProviderConfigured(): Boolean =
        modelProviderKeyAlias() != null || preferences.getBoolean(KEY_MODEL_PROVIDER_CONFIGURED, false)

    fun persistModelProviderConfigured(configured: Boolean, keyAlias: String? = null) {
        preferences.edit()
            .putBoolean(KEY_MODEL_PROVIDER_CONFIGURED, configured)
            .putString(KEY_MODEL_PROVIDER_KEY_ALIAS, keyAlias?.trim()?.takeIf { it.isNotBlank() })
            .apply()
    }

    fun createAppNotificationSettingsIntent(): Intent =
        Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).apply {
            putExtra(Settings.EXTRA_APP_PACKAGE, context.packageName)
        }

    private companion object {
        const val KEY_INBOX_URI = "inbox_uri"
        const val KEY_INBOX_LABEL = "inbox_label"
        const val KEY_PROJECT_PHOTO_GRID_COLUMNS = "project_photo_grid_columns"
        const val KEY_CAMERA_CONNECT_HOST = "camera_connect_host"
        const val KEY_SMART_SELECTION_STRATEGY_PROFILE_ID = "smart_selection_strategy_profile_id"
        const val KEY_MODEL_PROVIDER_CONFIGURED = "model_provider_configured"
        const val KEY_MODEL_PROVIDER_KEY_ALIAS = "model_provider_key_alias"
        const val DEFAULT_PROJECT_PHOTO_GRID_COLUMNS = 3
        const val DEFAULT_SMART_SELECTION_STRATEGY_PROFILE_ID = "general"
    }
}

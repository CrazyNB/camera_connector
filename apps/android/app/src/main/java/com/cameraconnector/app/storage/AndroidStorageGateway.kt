package com.cameraconnector.app.storage

import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.net.Uri
import android.provider.Settings

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

    fun createAppNotificationSettingsIntent(): Intent =
        Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).apply {
            putExtra(Settings.EXTRA_APP_PACKAGE, context.packageName)
        }

    private companion object {
        const val KEY_INBOX_URI = "inbox_uri"
        const val KEY_INBOX_LABEL = "inbox_label"
    }
}

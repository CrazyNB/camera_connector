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

    fun selectedInboxUri(): Uri? =
        preferences.getString(KEY_INBOX_URI, null)?.let(Uri::parse)

    fun inboxGridColumnCount(): Int =
        preferences.getInt(KEY_INBOX_GRID_COLUMNS, DEFAULT_INBOX_GRID_COLUMNS).coerceIn(2, 3)

    fun persistInboxGridColumnCount(columnCount: Int) {
        preferences.edit()
            .putInt(KEY_INBOX_GRID_COLUMNS, columnCount.coerceIn(2, 3))
            .apply()
    }

    fun createAppNotificationSettingsIntent(): Intent =
        Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).apply {
            putExtra(Settings.EXTRA_APP_PACKAGE, context.packageName)
        }

    private companion object {
        const val KEY_INBOX_URI = "inbox_uri"
        const val KEY_INBOX_LABEL = "inbox_label"
        const val KEY_INBOX_GRID_COLUMNS = "inbox_grid_columns"
        const val DEFAULT_INBOX_GRID_COLUMNS = 3
    }
}

package com.cameraconnector.app.storage

import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.net.Uri

class AndroidStorageGateway(private val context: Context) {
    private val preferences: SharedPreferences =
        context.getSharedPreferences("camera_connector_storage", Context.MODE_PRIVATE)

    fun persistOutputDirectory(uri: Uri) {
        val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or
            Intent.FLAG_GRANT_WRITE_URI_PERMISSION
        context.contentResolver.takePersistableUriPermission(uri, flags)
        preferences.edit()
            .putString(KEY_OUTPUT_URI, uri.toString())
            .putString(KEY_OUTPUT_LABEL, uri.lastPathSegment ?: uri.toString())
            .apply()
    }

    fun selectedOutputLabel(): String? {
        return preferences.getString(KEY_OUTPUT_LABEL, null)
    }

    fun selectedOutputUri(): Uri? =
        preferences.getString(KEY_OUTPUT_URI, null)?.let(Uri::parse)

    fun projectPhotoGridColumnCount(): Int =
        preferences.getInt(KEY_PROJECT_PHOTO_GRID_COLUMNS, DEFAULT_PROJECT_PHOTO_GRID_COLUMNS).coerceIn(2, 3)

    fun persistProjectPhotoGridColumnCount(columnCount: Int) {
        preferences.edit()
            .putInt(KEY_PROJECT_PHOTO_GRID_COLUMNS, columnCount.coerceIn(2, 3))
            .apply()
    }

    private companion object {
        const val KEY_OUTPUT_URI = "output_uri"
        const val KEY_OUTPUT_LABEL = "output_label"
        const val KEY_PROJECT_PHOTO_GRID_COLUMNS = "project_photo_grid_columns"
        const val DEFAULT_PROJECT_PHOTO_GRID_COLUMNS = 3
    }
}

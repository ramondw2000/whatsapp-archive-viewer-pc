package com.ramon.whatsapp_archive_viewer

import android.content.Intent
import android.database.Cursor
import android.net.Uri
import android.os.Bundle
import android.provider.OpenableColumns
import androidx.activity.enableEdgeToEdge
import java.io.File
import java.io.FileOutputStream
import java.util.UUID

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    handleShareIntent(intent)
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    handleShareIntent(intent)
  }

  private fun handleShareIntent(intent: Intent?) {
    if (intent == null) return
    val action = intent.action ?: return

    val uri: Uri? = when (action) {
      Intent.ACTION_SEND -> intent.getParcelableExtra(Intent.EXTRA_STREAM)
      Intent.ACTION_VIEW -> intent.data
      else -> null
    }

    uri ?: return

    try {
      // Resolve the original filename from the content URI so import can derive chat name
      val originalName = getDisplayName(uri) ?: "${UUID.randomUUID()}.zip"
      val safeFileName = originalName.replace(Regex("[^a-zA-Z0-9._\\- ]"), "_")

      // Copy the content URI into a cache file preserving the original filename
      val inputStream = contentResolver.openInputStream(uri) ?: return
      val cacheFile = File(cacheDir, safeFileName)
      FileOutputStream(cacheFile).use { out ->
        inputStream.copyTo(out)
      }
      inputStream.close()

      // Write the resolved path into the app files dir so Rust can find it
      val flagFile = File(filesDir, "pending_share.txt")
      flagFile.writeText(cacheFile.absolutePath)
    } catch (e: Exception) {
      e.printStackTrace()
    }
  }

  private fun getDisplayName(uri: Uri): String? {
    var name: String? = null
    val cursor: Cursor? = contentResolver.query(uri, null, null, null, null)
    cursor?.use {
      if (it.moveToFirst()) {
        val idx = it.getColumnIndex(OpenableColumns.DISPLAY_NAME)
        if (idx >= 0) {
          name = it.getString(idx)
        }
      }
    }
    // Fallback: try to get filename from URI path
    if (name == null) {
      name = uri.lastPathSegment
    }
    return name
  }
}

package com.qurb

import androidx.lifecycle.lifecycleScope
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.qurb_mobile.DeletedFile

/**
 * Recently deleted (decision 0042): files deleted on this phone or on another
 * device, kept here for thirty days. Restoring one puts it back on every
 * device, the way any change travels. Opened from Settings and from the Vault.
 */
object RecentlyDeleted {

    fun show(app: MainActivity) {
        app.lifecycleScope.launch {
            val deleted = try {
                withContext(Dispatchers.IO) { Engine.open(app).recentlyDeleted() }
            } catch (e: Exception) {
                app.fail("Could not read Recently deleted", e)
                return@launch
            }
            if (deleted.isEmpty()) {
                app.say("Nothing recently deleted")
                return@launch
            }
            val lines = deleted.map { d ->
                val by = d.deletedBy?.let { " on $it" } ?: ""
                "${d.path.substringAfterLast('/')}\n${Words.size(d.size)} · deleted$by ${Words.ago(d.deletedAt)}"
            }
            MaterialAlertDialogBuilder(app)
                .setTitle("Recently deleted")
                .setItems(lines.toTypedArray()) { _, which -> choose(app, deleted[which]) }
                .setNegativeButton("Close", null)
                .show()
        }
    }

    private fun choose(app: MainActivity, d: DeletedFile) {
        MaterialAlertDialogBuilder(app)
            .setTitle(d.path.substringAfterLast('/'))
            .setMessage(
                "Restoring puts it back in qurb on this phone, and it returns on your other " +
                    "devices at their next sync." + (d.why?.let { "\n\n${it.replaceFirstChar(Char::uppercase)}." } ?: "")
            )
            .setPositiveButton("Restore") { _, _ -> restore(app, d) }
            .setNeutralButton("Delete for good") { _, _ -> forget(app, d) }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun restore(app: MainActivity, d: DeletedFile) {
        app.lifecycleScope.launch {
            try {
                val at = withContext(Dispatchers.IO) { Engine.open(app).restoreDeleted(d.id) }
                app.say(if (at == d.path) "Restored" else "Restored as ${at.substringAfterLast('/')}")
                SyncWorker.runNow(app)
            } catch (e: Exception) {
                app.fail("Could not restore it", e)
            } finally {
                app.changed()
            }
        }
    }

    private fun forget(app: MainActivity, d: DeletedFile) {
        app.lifecycleScope.launch {
            try {
                withContext(Dispatchers.IO) { Engine.open(app).forgetDeleted(d.id) }
                app.say("Deleted for good")
            } catch (e: Exception) {
                app.fail("Could not delete it", e)
            } finally {
                app.changed()
            }
        }
    }
}

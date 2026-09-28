package com.qurb

import android.view.View
import android.widget.EditText
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.qurb.databinding.ScreenPageBinding
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.qurb_mobile.DeletedFile
import uniffi.qurb_mobile.ShareTarget
import uniffi.qurb_mobile.SharedFolder
import uniffi.qurb_mobile.Usage

/**
 * The rest: this phone's name and privacy default, what qurb costs in space,
 * how syncing is arranged, and what version this is.
 */
class SettingsScreen(app: MainActivity) : Screen(app) {

    private val views = ScreenPageBinding.inflate(app.layoutInflater)
    override val view: View get() = views.root

    init {
        views.title.text = "Settings"
        views.refresh.setOnRefreshListener { refresh() }
    }

    override fun refresh() {
        scope.launch {
            try {
                val (usage, background, deleted) = withContext(Dispatchers.IO) {
                    // `state` waits on WorkManager's own database; off the main
                    // thread like everything else.
                    Triple(engine().usage(), SyncWorker.state(app), engine().recentlyDeleted())
                }
                show(usage, background, deleted)
            } catch (e: Exception) {
                app.fail("Could not read the settings", e)
            } finally {
                views.refresh.isRefreshing = false
            }
        }
    }

    private fun show(usage: Usage, background: String, deleted: List<DeletedFile>) {
        val page = views.sections
        page.removeAllViews()

        page.heading("This phone")
        page.setting("Name", "${android.os.Build.MODEL ?: "phone"} — what your other devices call it")
        val private = Engine.ownFilesPrivate(app)
        page.setting(
            "Keep new files private",
            if (private) {
                "Files added here stay on this phone, and on devices you choose to keep them"
            } else {
                "Files added here go to all your devices"
            },
        ).apply {
            toggle.visibility = View.VISIBLE
            toggle.isChecked = private
            root.setOnClickListener { toggle.toggle() }
            toggle.setOnCheckedChangeListener { _, on -> setPrivate(on) }
        }

        page.heading("Your key")
        page.setting(
            "Recovery phrase",
            "The 24 words that are your key. Show them to write out a new copy.",
        ) { warnThenShowPhrase() }

        page.heading("Space")
        page.setting("Your files", Words.size(usage.logical))
        page.setting("qurb on this phone", Words.size(usage.onDisk))
        page.setting(
            "Recently deleted",
            if (deleted.isEmpty()) {
                "Nothing. Files deleted here or on another device are kept 30 days"
            } else {
                "${Words.files(deleted.size)}, ${Words.size(deleted.sumOf { it.size })} — " +
                    "kept 30 days, and restorable"
            },
        ) { if (deleted.isNotEmpty()) showDeleted(deleted) }
        page.setting("Free unused space", "Clears what nothing needs any more") { tidy() }

        page.heading("Folders")
        page.setting(
            "Shared folders",
            "Which devices each folder is on. Every device, unless you choose.",
        ) { chooseFolder() }

        page.heading("Syncing")
        page.setting("Background sync", background) { explainBackground() }
        page.setting("Rendezvous service", Engine.signalUrl(app)) { editSignal() }
        page.setting(
            "Relay",
            Engine.relayAddress(app) ?: "None — devices must reach each other directly",
        ) { editRelay() }

        page.heading("About")
        val version = runCatching {
            app.packageManager.getPackageInfo(app.packageName, 0).versionName
        }.getOrNull() ?: "unknown"
        page.setting("Version", version)
    }

    /**
     * The words again, for a new paper copy before the old one is lost.
     *
     * Not a secret kept from the person holding the phone: anyone who can
     * open this app can read every file already, so the words give away
     * nothing new (decision 0033). But they are the key, so the screen says
     * so first, and the window is kept out of screenshots while they are on it.
     */
    private fun warnThenShowPhrase() {
        MaterialAlertDialogBuilder(app)
            .setTitle("Show your recovery phrase?")
            .setMessage(
                "Anyone who sees these 24 words can read every file you keep in " +
                    "qurb, on any device. Make sure nobody is looking."
            )
            .setPositiveButton("Show") { _, _ ->
                scope.launch {
                    try {
                        val phrase = withContext(Dispatchers.IO) { engine().recoveryPhrase() }
                        MaterialAlertDialogBuilder(app)
                            .setTitle("Your recovery phrase")
                            .setView(Words.phraseView(app, phrase))
                            .setPositiveButton("Hide", null)
                            .create()
                            .apply {
                                window?.addFlags(android.view.WindowManager.LayoutParams.FLAG_SECURE)
                            }
                            .show()
                    } catch (e: Exception) {
                        app.fail("Could not show the words", e)
                    }
                }
            }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun setPrivate(on: Boolean) {
        scope.launch {
            try {
                Engine.setOwnFilesPrivate(app, on)
                app.say(if (on) "New files stay private" else "New files go to all your devices")
            } catch (e: Exception) {
                app.fail("Could not change that", e)
            } finally {
                refresh()
            }
        }
    }

    /**
     * Recently deleted (decision 0042): files deleted on this phone or on
     * another device, kept here for thirty days. Restoring one puts it back
     * on every device, the way any change travels.
     */
    private fun showDeleted(deleted: List<DeletedFile>) {
        val lines = deleted.map { d ->
            val by = d.deletedBy?.let { " on $it" } ?: ""
            "${d.path.substringAfterLast('/')}\n${Words.size(d.size)} · deleted$by ${Words.ago(d.deletedAt)}"
        }
        MaterialAlertDialogBuilder(app)
            .setTitle("Recently deleted")
            .setItems(lines.toTypedArray()) { _, which -> chooseDeleted(deleted[which]) }
            .setNegativeButton("Close", null)
            .show()
    }

    private fun chooseDeleted(d: DeletedFile) {
        MaterialAlertDialogBuilder(app)
            .setTitle(d.path.substringAfterLast('/'))
            .setMessage(
                "Restoring puts it back in qurb on this phone, and it returns on your other " +
                    "devices at their next sync." + (d.why?.let { "\n\n$it." } ?: "")
            )
            .setPositiveButton("Restore") { _, _ -> restoreDeleted(d) }
            .setNeutralButton("Delete for good") { _, _ -> forgetDeleted(d) }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun restoreDeleted(d: DeletedFile) {
        scope.launch {
            try {
                val at = withContext(Dispatchers.IO) { engine().restoreDeleted(d.id) }
                app.say(if (at == d.path) "Restored" else "Restored as ${at.substringAfterLast('/')}")
                SyncWorker.runNow(app)
            } catch (e: Exception) {
                app.fail("Could not restore it", e)
            } finally {
                app.changed()
            }
        }
    }

    private fun forgetDeleted(d: DeletedFile) {
        scope.launch {
            try {
                withContext(Dispatchers.IO) { engine().forgetDeleted(d.id) }
                app.say("Deleted for good")
            } catch (e: Exception) {
                app.fail("Could not delete it", e)
            } finally {
                refresh()
            }
        }
    }

    /**
     * Which devices a folder is shared with (decision 0044): the folders,
     * then the devices for the one chosen.
     */
    private fun chooseFolder() {
        scope.launch {
            val (folders, devices) = try {
                withContext(Dispatchers.IO) { engine().sharing() to engine().shareTargets() }
            } catch (e: Exception) {
                app.fail("Could not read the folders", e)
                return@launch
            }
            if (folders.isEmpty()) {
                app.say("No folders yet")
                return@launch
            }
            val names = folders.map { f ->
                val who = if (f.everyone) {
                    "every device"
                } else {
                    f.members.joinToString(", ") { id -> devices.find { it.id == id }?.name ?: "a removed device" }
                }
                "${f.folder}\n$who"
            }
            MaterialAlertDialogBuilder(app)
                .setTitle("Shared folders")
                .setItems(names.toTypedArray()) { _, which -> chooseDevices(folders[which], devices) }
                .setNegativeButton("Close", null)
                .show()
        }
    }

    private fun chooseDevices(folder: SharedFolder, devices: List<ShareTarget>) {
        val ticked = BooleanArray(devices.size) { folder.everyone || devices[it].id in folder.members }
        MaterialAlertDialogBuilder(app)
            .setTitle(folder.folder)
            .setMultiChoiceItems(devices.map { it.name }.toTypedArray(), ticked) { _, which, on ->
                ticked[which] = on
            }
            .setPositiveButton("Save") { _, _ ->
                val chosen = devices.filterIndexed { i, _ -> ticked[i] }.map { it.id }
                // All ticked is no rule at all, so a device paired later is in too.
                setSharing(folder.folder, if (chosen.size == devices.size) emptyList() else chosen)
            }
            .setNeutralButton("Every device") { _, _ -> setSharing(folder.folder, emptyList()) }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun setSharing(folder: String, members: List<String>) {
        scope.launch {
            try {
                withContext(Dispatchers.IO) { engine().setSharing(folder, members) }
                app.say(
                    if (members.isEmpty()) "$folder is on every device"
                    else "Saved. A device left out keeps what it has, and gets nothing new."
                )
                SyncWorker.runNow(app)
            } catch (e: Exception) {
                app.fail("Could not change that", e)
            }
        }
    }

    private fun tidy() {
        scope.launch {
            try {
                val tidied = withContext(Dispatchers.IO) { engine().housekeep() }
                app.say(
                    if (tidied.freed > 0uL) "Freed ${Words.size(tidied.freed)}" else "Nothing to free"
                )
            } catch (e: Exception) {
                app.fail("Could not free space", e)
            } finally {
                refresh()
            }
        }
    }

    /**
     * What the background scheduler does, in plain words. The honest answer is
     * "roughly every fifteen minutes, when Android allows", and an app that
     * quietly does nothing for hours while claiming to sync is worse than one
     * that says so.
     */
    private fun explainBackground() {
        MaterialAlertDialogBuilder(app)
            .setTitle("Background sync")
            .setMessage(
                "Android decides when this runs. Fifteen minutes is the shortest period it " +
                    "accepts, and an idle phone may go much longer between attempts.\n\n" +
                    "Both devices have to be switched on at the same moment for a sync to " +
                    "happen, so a computer that is off is missed until next time."
            )
            .setPositiveButton("OK", null)
            .setNeutralButton("Run one now") { _, _ ->
                // Through the scheduler rather than directly, so this exercises
                // the same path the periodic schedule uses.
                SyncWorker.runNow(app)
                app.say("Queued. It runs when Android allows.")
            }
            .show()
    }

    /**
     * The relay, for when two devices cannot reach each other directly --
     * which on mobile data is often. Checked by the engine's own rule as it is
     * saved, so a mistyped address is refused here rather than found out by
     * every sync after.
     */
    private fun editRelay() {
        val input = EditText(app).apply {
            setText(Engine.relayAddress(app).orEmpty())
            hint = "relay.example.com:9001"
            setPadding(48, 32, 48, 8)
        }
        val dialog = MaterialAlertDialogBuilder(app)
            .setTitle("Relay")
            .setMessage(
                "When this phone and another device cannot reach each other directly — " +
                    "common on mobile data — their encrypted traffic goes through a relay " +
                    "instead. It cannot read it.\n\nRun `qurb relay` on a server of your own " +
                    "and enter its address and port. Leave it empty for none."
            )
            .setView(input)
            .setPositiveButton("Save", null)
            .setNegativeButton("Cancel", null)
            .create()
        dialog.show()
        dialog.getButton(android.content.DialogInterface.BUTTON_POSITIVE).setOnClickListener {
            val text = input.text.toString().trim()
            val problem = if (text.isEmpty()) null else uniffi.qurb_mobile.relayAddressProblem(text)
            if (problem != null) {
                input.error = problem
                return@setOnClickListener
            }
            Engine.setRelayAddress(app, text.ifEmpty { null })
            dialog.dismiss()
            app.say(if (text.isEmpty()) "No relay" else "Saved")
            refresh()
        }
    }

    /**
     * Where the rendezvous service is: on a server of your own, run
     * `qurb signal` and point the phone at it.
     */
    private fun editSignal() {
        val input = EditText(app).apply {
            setText(Engine.signalUrl(app))
            setPadding(48, 32, 48, 8)
        }
        MaterialAlertDialogBuilder(app)
            .setTitle("Rendezvous service")
            .setMessage(
                "Two devices find each other through this when they are not on the same " +
                    "network. Run `qurb signal` on a server of your own and enter the address " +
                    "it prints — wss://… for one reachable from anywhere."
            )
            .setView(input)
            .setPositiveButton("Save") { _, _ ->
                Engine.setSignalUrl(app, input.text.toString().trim())
                app.say("Saved")
                refresh()
            }
            .setNegativeButton("Cancel", null)
            .show()
    }
}

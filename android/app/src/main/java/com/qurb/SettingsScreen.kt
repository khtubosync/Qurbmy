package com.qurb

import android.view.View
import android.widget.EditText
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.qurb.databinding.ScreenPageBinding
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
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
                val (usage, background) = withContext(Dispatchers.IO) {
                    // `state` waits on WorkManager's own database; off the main
                    // thread like everything else.
                    engine().usage() to SyncWorker.state(app)
                }
                show(usage, background)
            } catch (e: Exception) {
                app.fail("Could not read the settings", e)
            } finally {
                views.refresh.isRefreshing = false
            }
        }
    }

    private fun show(usage: Usage, background: String) {
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
        page.setting("Free unused space", "Deleted files are kept 7 days, then cleared") { tidy() }

        page.heading("Syncing")
        page.setting("Background sync", background) { explainBackground() }
        page.setting("Rendezvous service", Engine.signalUrl(app)) { editSignal() }

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
     * Where the rendezvous service is. Editable because there is no hosted one
     * yet: to try this, run `qurb signal` on a computer and point the phone at
     * it.
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
                    "network. Run `qurb signal` on a computer and use ws://<that machine>:9000."
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

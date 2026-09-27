package com.qurb

import android.view.View
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.qurb.databinding.ScreenPageBinding
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.qurb_mobile.Happening
import uniffi.qurb_mobile.Waiting

/**
 * What is on its way, and what happened.
 *
 * One screen for both because they are the same question asked at different
 * times: a file sent is *waiting* until the other device collects it, and then
 * it is *history*. The brief lists Activity as a place of its own; a phone's
 * tab bar holds five, and this is where it went.
 *
 * Not shown: a byte-by-byte progress bar. A phone syncs in short windows,
 * mostly in the background, and nothing is moving while this screen is open
 * unless a sync from Home is running.
 */
class TransfersScreen(app: MainActivity) : Screen(app) {

    private val views = ScreenPageBinding.inflate(app.layoutInflater)
    override val view: View get() = views.root

    /** How far back the history goes on screen. Grows with "Show older". */
    private var shown = PAGE

    init {
        views.title.text = "Transfers"
        views.refresh.setOnRefreshListener { refresh() }
    }

    override fun refresh() {
        scope.launch {
            try {
                val (waiting, history) = withContext(Dispatchers.IO) {
                    val engine = engine()
                    engine.waiting() to engine.history(shown.toUInt(), null)
                }
                show(waiting, history)
            } catch (e: Exception) {
                app.fail("Could not read what happened", e)
            } finally {
                views.refresh.isRefreshing = false
            }
        }
    }

    private fun show(waiting: List<Waiting>, history: List<Happening>) {
        val page = views.sections
        page.removeAllViews()

        page.heading("Waiting to be collected")
        if (waiting.isEmpty()) {
            page.line("Nothing waiting", "Files you send wait here until the device collects them.")
        }
        for (w in waiting) {
            page.line(w.path, "for ${w.to} · ${Words.size(w.size)} · tap to stop") { cancel(w) }
        }

        page.heading("What happened")
        if (history.isEmpty()) page.line("Nothing yet", "")
        for (h in history) {
            val (what, detail) = Words.happened(h)
            page.line(what, detail)
        }
        if (history.size == shown) {
            page.line("Show older", "") {
                shown += PAGE
                refresh()
            }
        }
    }

    private fun cancel(w: Waiting) {
        MaterialAlertDialogBuilder(app)
            .setTitle("Stop sending ${w.path.substringAfterLast('/')}?")
            .setMessage("${w.to} has not collected it yet, so it never arrives there.")
            .setPositiveButton("Stop sending") { _, _ ->
                scope.launch {
                    try {
                        withContext(Dispatchers.IO) { engine().cancelSend(w.path, w.toFingerprint) }
                        app.say("Stopped")
                    } catch (e: Exception) {
                        // Most likely collected between the list and the tap:
                        // the engine refuses then, and says why.
                        app.fail("Could not stop that", e)
                    } finally {
                        app.changed()
                    }
                }
            }
            .setNegativeButton("Keep sending", null)
            .show()
    }

    private companion object {
        const val PAGE = 50
    }
}

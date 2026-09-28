package com.qurb

import android.view.View
import com.qurb.databinding.ScreenHomeBinding
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import uniffi.qurb_mobile.ConflictInfo
import uniffi.qurb_mobile.ConflictSide
import uniffi.qurb_mobile.Happening
import uniffi.qurb_mobile.Outstanding
import uniffi.qurb_mobile.PeerInfo
import uniffi.qurb_mobile.Usage

/**
 * The first thing anyone sees: which devices this phone knows, whether
 * anything on it exists nowhere else, and what happened lately.
 *
 * The second of those is the one the whole product exists to answer, so it
 * gets a card of its own -- shown only while it is true, and saying what to do
 * about it.
 */
class HomeScreen(app: MainActivity) : Screen(app) {

    private val views = ScreenHomeBinding.inflate(app.layoutInflater)
    override val view: View get() = views.root

    /** Whether any device is connected, as of the last read. Decides the buttons. */
    private var paired = false

    init {
        views.secondary.setOnClickListener { app.pair() }
        views.refresh.setOnRefreshListener { refresh() }
    }

    private fun buttons() {
        views.primary.text = if (paired) "Sync now" else "Connect a device"
        views.primary.setIconResource(if (paired) R.drawable.ic_sync else R.drawable.ic_link)
        views.primary.setOnClickListener { if (paired) app.sync() else app.pair() }
        views.primary.isEnabled = !(paired && app.syncing)
        views.secondary.visibility = if (paired) View.VISIBLE else View.GONE
        views.progress.visibility = if (app.syncing) View.VISIBLE else View.GONE
    }

    /** Everything the screen draws, read in one go off the main thread. */
    private class State(
        val peers: List<PeerInfo>,
        val holders: List<PeerInfo>,
        val outstanding: Outstanding,
        val usage: Usage,
        val recent: List<Happening>,
        val conflicts: List<ConflictInfo>,
    )

    override fun refresh() {
        buttons()

        scope.launch {
            try {
                val state = withContext(Dispatchers.IO) {
                    val engine = engine()
                    State(
                        engine.peers(),
                        engine.holders(),
                        engine.outstanding(),
                        engine.usage(),
                        engine.history(RECENT.toUInt(), null),
                        engine.conflicts(),
                    )
                }
                show(state)
            } catch (e: Exception) {
                app.fail("Could not read this phone's files", e)
            } finally {
                views.refresh.isRefreshing = false
            }
        }
    }

    private fun show(state: State) {
        val devices = when (state.peers.size) {
            0 -> "Not connected to any device yet"
            1 -> "Connected to ${state.peers[0].name}"
            else -> "Connected to ${state.peers.size} devices"
        }
        views.status.text = "$devices · ${Words.size(state.usage.logical)} of files"

        paired = state.peers.isNotEmpty()
        buttons()

        showOnlyHere(state)
        showConflicts(state.conflicts)

        views.recent.removeAllViews()
        if (state.recent.isEmpty()) {
            views.recent.line("Nothing yet", "What happens between your devices shows up here.")
        }
        for (h in state.recent) {
            val (what, detail) = Words.happened(h)
            views.recent.line(what, detail)
        }
        if (state.recent.size == RECENT) {
            views.recent.line("See everything", "") { app.go(R.id.tab_transfers) }
        }
    }

    /**
     * The card for files that exist on this phone and nowhere else.
     *
     * Three different situations, each with its own next step: no device at
     * all; this phone's own files with nobody chosen to keep them; or a device
     * that will take them and has not been reached yet.
     */
    private fun showOnlyHere(state: State) {
        val waiting = state.outstanding.files
        if (waiting.isEmpty()) {
            views.keepCard.visibility = View.GONE
            return
        }
        views.keepCard.visibility = View.VISIBLE
        val count = waiting.size
        views.keepTitle.text = if (count == 1) {
            "1 file is only on this phone"
        } else {
            "$count files are only on this phone"
        }
        val size = Words.size(state.outstanding.bytes)
        val ownWithNoKeeper = waiting.any { it.private } && state.holders.isEmpty()

        when {
            state.peers.isEmpty() -> {
                views.keepText.text =
                    "$size that would be lost with the phone. Connect a device to keep a copy."
                views.keepAction.text = "Connect a device"
                views.keepAction.setOnClickListener { app.pair() }
            }
            ownWithNoKeeper -> {
                views.keepText.text =
                    "$size of this phone's own files, and no device is keeping them yet. " +
                        "Choose one, and it keeps a copy where only you can get at it."
                views.keepAction.text = "Choose a device"
                views.keepAction.setOnClickListener { app.go(R.id.tab_devices) }
            }
            else -> {
                val to = (state.holders.ifEmpty { state.peers }).joinToString(", ") { it.name }
                views.keepText.text =
                    "$size, waiting for $to. It goes at the next sync, when both are " +
                        "switched on at the same time."
                views.keepAction.text = "Sync now"
                views.keepAction.setOnClickListener { app.sync() }
            }
        }
    }

    /** Files two devices changed at once (brief §24). */
    private fun showConflicts(conflicts: List<ConflictInfo>) {
        views.conflictCard.visibility = if (conflicts.isEmpty()) View.GONE else View.VISIBLE
        if (conflicts.isEmpty()) return
        views.conflictTitle.text = if (conflicts.size == 1) {
            "Two devices changed ${conflicts[0].path.substringAfterLast('/')}"
        } else {
            "Two devices changed ${conflicts.size} files"
        }
        views.conflictAction.setOnClickListener {
            if (conflicts.size == 1) {
                choose(conflicts[0])
            } else {
                MaterialAlertDialogBuilder(app)
                    .setTitle("Changed on two devices")
                    .setItems(conflicts.map { it.path }.toTypedArray()) { _, which -> choose(conflicts[which]) }
                    .setNegativeButton("Close", null)
                    .show()
            }
        }
    }

    /**
     * One conflict: both versions described, three choices. Whichever is not
     * kept goes to Recently deleted, so no choice here loses anything.
     */
    private fun choose(c: ConflictInfo) {
        fun describe(s: ConflictSide) = "${s.by}, ${Words.ago(s.changedAt)} · ${Words.size(s.size)}" +
            if (s.here) "" else " · not on this phone yet"
        val text = "This version: " + (c.`this`?.let { describe(it) } ?: "since deleted or renamed") +
            "\nThe other: " + describe(c.other) +
            "\n\nEach device changed it without having seen the other's change, so neither " +
            "replaced the other. Whichever you do not keep goes to Recently deleted."

        // Keeping the other one, or both, needs its bytes here.
        val choices = buildList {
            add("Keep this version" to "this")
            if (c.other.here) {
                add("Keep the other" to "other")
                add("Keep both" to "both")
            }
        }
        MaterialAlertDialogBuilder(app)
            .setCustomTitle(explained(c.path.substringAfterLast('/'), text))
            .setItems(choices.map { it.first }.toTypedArray()) { _, which -> settle(c, choices[which].second) }
            .setNegativeButton("Cancel", null)
            .show()
    }

    /** A title with an explanation under it, for a dialog whose body is a list. */
    private fun explained(title: String, text: String): View {
        val pad = (20 * app.resources.displayMetrics.density).toInt()
        return android.widget.LinearLayout(app).apply {
            orientation = android.widget.LinearLayout.VERTICAL
            setPadding(pad, pad, pad, 0)
            addView(android.widget.TextView(app).apply {
                this.text = title
                setTextAppearance(com.google.android.material.R.style.TextAppearance_Material3_HeadlineSmall)
            })
            addView(android.widget.TextView(app).apply {
                this.text = text
                setPadding(0, pad / 2, 0, 0)
                setTextAppearance(com.google.android.material.R.style.TextAppearance_Material3_BodyMedium)
            })
        }
    }

    private fun settle(c: ConflictInfo, keep: String) {
        scope.launch {
            try {
                val kept = withContext(Dispatchers.IO) { engine().settleConflict(c.other.path, keep) }
                app.say(if (keep == "both") "Kept both" else "Kept ${kept.substringAfterLast('/')}")
                SyncWorker.runNow(app)
            } catch (e: Exception) {
                app.fail("Could not settle that", e)
            } finally {
                app.changed()
            }
        }
    }

    private companion object {
        /** How many recent events Home shows; the rest are under Transfers. */
        const val RECENT = 6
    }
}

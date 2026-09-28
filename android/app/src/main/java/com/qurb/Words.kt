package com.qurb

import android.content.Context
import android.view.View
import androidx.core.content.ContextCompat
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.google.android.material.snackbar.Snackbar
import uniffi.qurb_mobile.Available
import uniffi.qurb_mobile.FileEntry
import uniffi.qurb_mobile.Happening
import uniffi.qurb_mobile.QurbException

/**
 * How the app says things, in one place.
 *
 * The product's vocabulary is deliberate -- *on this phone*, *kept on*, *free
 * phone space*, *sent*, never *cloud* or *upload* -- and a screen that
 * improvised its own wording would drift from it. So every screen asks here.
 */
object Words {

    fun size(bytes: ULong): String {
        val units = listOf("B", "KB", "MB", "GB", "TB")
        var value = bytes.toDouble()
        var unit = 0
        while (value >= 1024 && unit < units.size - 1) {
            value /= 1024
            unit++
        }
        return if (unit == 0) "${bytes} B" else "%.1f %s".format(value, units[unit])
    }

    /** Unix seconds as a relative time, the way the desktop says it. */
    fun ago(seconds: Long): String {
        val now = System.currentTimeMillis() / 1000
        val gone = (now - seconds).coerceAtLeast(0)
        return when {
            gone < 90 -> "just now"
            gone < 5400 -> plural(gone / 60, "minute")
            gone < 86400 -> plural(gone / 3600, "hour")
            // Rounded rather than truncated past a day: 46 hours is "2 days".
            else -> plural((gone + 43200) / 86400, "day")
        }
    }

    private fun plural(n: Long, unit: String) = "$n $unit${if (n == 1L) "" else "s"} ago"

    /**
     * Where a file's bytes are, as one short line, with the colour of its dot.
     *
     * `keeper` is the one other device that can be named as having it, when
     * there is exactly one that could; otherwise the line says "another
     * device" rather than guess. The third case is the one that matters: a
     * file on this phone and nowhere else is lost with the phone, and is never
     * offered as space to free.
     */
    fun where(context: Context, file: FileEntry, keeper: String?): Pair<String, Int> {
        val kept = keeper ?: "another device"
        val shared = if (file.private) "" else " · shared"
        return when (file.available) {
            Available.HERE ->
                "${size(file.size)} · on this phone and on $kept$shared" to
                    ContextCompat.getColor(context, R.color.safe)
            Available.ONLY_HERE ->
                "${size(file.size)} · only on this phone$shared" to
                    ContextCompat.getColor(context, R.color.notice)
            Available.ELSEWHERE ->
                "${size(file.size)} · on $kept, not on this phone$shared" to
                    ContextCompat.getColor(context, R.color.ink_soft)
        }
    }

    fun files(n: Int) = if (n == 1) "1 file" else "$n files"

    /** One line of history, in words rather than the engine's event names. */
    fun happened(h: Happening): Pair<String, String> {
        val what = h.path ?: h.device ?: ""
        val who = h.device ?: "another device"
        val headline = when (h.kind) {
            "stored" -> "Added $what"
            "deleted" -> "Deleted $what"
            "received" -> if (h.detail?.startsWith("sent to this device") == true) {
                "$who sent $what"
            } else {
                "$what arrived"
            }
            "sent" -> "Sending $what to $who"
            "collected" -> "$who has $what"
            "evicted" -> "Freed phone space: $what"
            "restored" -> "Downloaded $what again"
            "conflicted" -> "Two devices changed $what"
            "paired" -> "Connected to $what"
            "removed" -> "Removed $what from this phone"
            "cancelled" -> "Stopped sending $what"
            "failed" -> "Could not finish $what"
            else -> what.ifEmpty { h.kind }
        }
        // The detail only where it adds something: why it failed, what a
        // conflict was filed as. A connection's detail is the device's name,
        // which the headline already says.
        val extra = h.detail?.takeIf {
            h.kind in setOf("failed", "conflicted") && it.isNotBlank()
        }
        return headline to listOfNotNull(ago(h.at), extra).joinToString(" · ")
    }

    /**
     * The 24 words laid out to copy onto paper: numbered, two columns, in a
     * fixed-width face so the columns line up. The realistic failure is losing
     * one's place halfway down the list.
     */
    fun phraseView(context: Context, phrase: String): View {
        val words = phrase.trim().split(Regex("\\s+"))
        val half = (words.size + 1) / 2
        val rows = (0 until half).joinToString("\n") { row ->
            val left = "%2d. %-9s".format(row + 1, words[row])
            val right = words.getOrNull(row + half)?.let { "%2d. %s".format(row + half + 1, it) } ?: ""
            "$left   $right"
        }
        val scale = context.resources.displayMetrics.density
        return android.widget.TextView(context).apply {
            text = rows
            typeface = android.graphics.Typeface.MONOSPACE
            textSize = 16f
            setLineSpacing(0f, 1.25f)
            setTextColor(ContextCompat.getColor(context, R.color.ink))
            setPadding((24 * scale).toInt(), (8 * scale).toInt(), (24 * scale).toInt(), 0)
        }
    }

    /** Say a failure, in words a person can act on. */
    fun fail(context: Context, title: String, e: Throwable) {
        MaterialAlertDialogBuilder(context)
            .setTitle(title)
            // `readable()` rather than `e.message`: UniFFI generates
            // "detail=${detail}", which puts a struct field name in front of
            // the person, and the detail alone rarely says what to try next.
            .setMessage(if (e is QurbException) e.readable() else e.message ?: e.toString())
            .setPositiveButton("OK", null)
            .show()
    }

    /**
     * A passing message. `above` is the view it must not cover -- the tab bar,
     * which a snackbar otherwise sits on top of for its whole three seconds.
     */
    fun say(anchor: View, message: String, above: View? = null) {
        Snackbar.make(anchor, message, Snackbar.LENGTH_LONG).setAnchorView(above).show()
    }
}

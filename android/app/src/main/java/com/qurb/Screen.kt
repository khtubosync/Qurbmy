package com.qurb

import android.util.TypedValue
import android.view.View
import android.view.ViewGroup
import androidx.lifecycle.lifecycleScope
import com.qurb.databinding.RowHeadingBinding
import com.qurb.databinding.RowHistoryBinding
import com.qurb.databinding.RowSettingBinding
import uniffi.qurb_mobile.Qurb

/**
 * One of the five places under the tab bar.
 *
 * Plain classes holding their views, not Fragments. The five are built the
 * first time each is shown and kept for the life of the activity, and changing
 * tab is swapping one child view for another: no transactions, no back stack,
 * no state to restore beyond which tab was showing. Decision 0039 asks for an
 * app that is light, and this is the least machinery that does the job.
 */
abstract class Screen(protected val app: MainActivity) {

    abstract val view: View

    /**
     * Read what may have changed and redraw. Called whenever the screen comes
     * into view, when the app comes back to the front, and after anything that
     * changes what the engine knows. Never blocks: the reading happens off the
     * main thread, and what is on screen stays until the new answer is in.
     */
    abstract fun refresh()

    /**
     * Back was pressed while this screen is showing. Returns whether it used
     * it -- the Vault going up a folder -- or the app should do the usual.
     */
    open fun back(): Boolean = false

    protected val scope get() = app.lifecycleScope

    protected suspend fun engine(): Qurb = Engine.open(app)

    // Building blocks for the screens that are a page of sections rather than
    // one list. Inflated per refresh: a page is a few dozen rows at most, and
    // rebuilding them is cheaper than the bookkeeping to update them in place.

    protected fun ViewGroup.heading(text: String) {
        RowHeadingBinding.inflate(app.layoutInflater, this, true).heading.text = text
    }

    protected fun ViewGroup.line(what: String, detail: String, onTap: (() -> Unit)? = null) {
        val row = RowHistoryBinding.inflate(app.layoutInflater, this, true)
        row.what.text = what
        row.detail.text = detail
        row.detail.visibility = if (detail.isEmpty()) View.GONE else View.VISIBLE
        onTap?.let { row.root.tappable(it) }
    }

    protected fun ViewGroup.setting(
        label: String,
        value: String,
        onTap: (() -> Unit)? = null,
    ): RowSettingBinding {
        val row = RowSettingBinding.inflate(app.layoutInflater, this, true)
        row.label.text = label
        row.value.text = value
        row.value.visibility = if (value.isEmpty()) View.GONE else View.VISIBLE
        if (onTap != null) row.root.setOnClickListener { onTap() } else row.root.isClickable = false
        return row
    }

    /** The platform's own touch ripple, and something to do on a tap. */
    protected fun View.tappable(onTap: () -> Unit) {
        val ripple = TypedValue()
        app.theme.resolveAttribute(android.R.attr.selectableItemBackground, ripple, true)
        setBackgroundResource(ripple.resourceId)
        setOnClickListener { onTap() }
    }
}

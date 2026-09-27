package com.qurb

import android.content.DialogInterface
import android.content.Intent
import android.os.Bundle
import android.text.InputType
import android.view.View
import android.view.WindowManager
import android.view.inputmethod.EditorInfo
import android.widget.LinearLayout
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.updatePadding
import androidx.lifecycle.lifecycleScope
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.google.android.material.textfield.TextInputEditText
import com.google.android.material.textfield.TextInputLayout
import com.qurb.databinding.ActivitySetupBinding
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.qurb_mobile.PhraseAnswer
import uniffi.qurb_mobile.QurbException

/**
 * First launch: make a key, or bring one over from another device.
 *
 * The whole screen exists to get one thing right — that the user writes the 24
 * words down. Nothing else here can be undone by a support ticket, because
 * there is no support ticket: the words are the only copy of the key.
 */
class SetupActivity : AppCompatActivity() {

    private lateinit var views: ActivitySetupBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        views = ActivitySetupBinding.inflate(layoutInflater)
        setContentView(views.root)

        // Android 15 draws edge to edge whether an app asks or not; without
        // this the first button sits under the status bar.
        ViewCompat.setOnApplyWindowInsetsListener(views.root) { view, windowInsets ->
            val bars = windowInsets.getInsets(
                WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            )
            view.updatePadding(top = bars.top, bottom = bars.bottom)
            windowInsets
        }

        views.create.setOnClickListener { create() }
        views.restore.setOnClickListener { showRestore() }
        views.restoreConfirm.setOnClickListener { restore() }

        // Set up, and closed before the words were typed back: straight to
        // them again, rather than to an app whose key was never checked.
        if (Engine.isSetUp(this) && !Engine.phraseConfirmed(this)) {
            views.chooser.visibility = View.GONE
            showAgain()
        }
    }

    private fun create() {
        busy(true)
        lifecycleScope.launch {
            try {
                showPhrase(Engine.create(this@SetupActivity))
            } catch (e: Exception) {
                busy(false)
                fail("Could not set up", e)
            }
        }
    }

    /**
     * The words, and the one dialog in this app that cannot be dismissed by
     * tapping outside it.
     *
     * Friction on purpose: every other consumer product can recover an
     * account, and this one genuinely cannot. The window is marked secure
     * while the words are on it, so they reach neither a screenshot nor the
     * recent-apps thumbnail.
     */
    private fun showPhrase(phrase: String) {
        MaterialAlertDialogBuilder(this)
            .setTitle("Write these down")
            .setMessage(
                "These 24 words are your key, not a backup of it. Nobody else " +
                    "has a copy — not a server, not us. Lose them and lose this " +
                    "phone, and your files cannot be recovered by anyone.\n\n" +
                    "Write them on paper, in order, now."
            )
            .setView(Words.phraseView(this, phrase))
            .setCancelable(false)
            .setPositiveButton("I have written them down") { _, _ -> confirm() }
            .create()
            .apply { window?.addFlags(WindowManager.LayoutParams.FLAG_SECURE) }
            .show()
    }

    /**
     * Three of the words, typed back from the paper.
     *
     * Somebody who has not written them down cannot answer, and finding that
     * out now, while the words can still be shown, is the point
     * (decision 0033). Three positions at random, chosen again each time, so
     * that looking at the words again is not a way to learn the answer to the
     * same question. Checked by the engine against the key, so the app keeps
     * no copy of the words to compare with.
     */
    private fun confirm() {
        val positions = (1..24).shuffled().take(3).sorted()
        val scale = resources.displayMetrics.density
        val fields = positions.map { position ->
            TextInputLayout(this).apply {
                hint = "Word $position"
                addView(
                    TextInputEditText(context).apply {
                        // Not learned by the keyboard: these are the key.
                        inputType = InputType.TYPE_CLASS_TEXT or
                            InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS or
                            InputType.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD
                        imeOptions = EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING
                        importantForAutofill = View.IMPORTANT_FOR_AUTOFILL_NO
                        isSingleLine = true
                    }
                )
            }
        }
        val form = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            val side = (24 * scale).toInt()
            setPadding(side, (8 * scale).toInt(), side, 0)
            fields.forEach { field ->
                addView(field)
                (field.layoutParams as LinearLayout.LayoutParams).topMargin = (8 * scale).toInt()
            }
        }

        val dialog = MaterialAlertDialogBuilder(this)
            .setTitle("Check your words")
            .setMessage("Type these three words from your paper.")
            .setView(form)
            .setCancelable(false)
            .setPositiveButton("Check", null)
            .setNeutralButton("Show the words again") { _, _ -> showAgain() }
            .create()
            .apply { window?.addFlags(WindowManager.LayoutParams.FLAG_SECURE) }
        dialog.show()

        // Set after showing, so a wrong answer keeps the dialog open.
        dialog.getButton(DialogInterface.BUTTON_POSITIVE).setOnClickListener {
            val answers = positions.zip(fields).map { (position, field) ->
                PhraseAnswer(position.toUInt(), field.editText?.text?.toString().orEmpty())
            }
            lifecycleScope.launch {
                val right = runCatching {
                    withContext(Dispatchers.IO) { Engine.open(this@SetupActivity).phraseMatches(answers) }
                }.getOrDefault(false)
                if (right) {
                    Engine.setPhraseConfirmed(this@SetupActivity, true)
                    dialog.dismiss()
                    done()
                } else {
                    dialog.setMessage(
                        "Those are not the words at those places. Check your paper, " +
                            "or look at the words again."
                    )
                }
            }
        }
    }

    /**
     * The words again, from the engine, which derives them from the key; this
     * screen kept no copy. Then a fresh check.
     */
    private fun showAgain() {
        lifecycleScope.launch {
            try {
                val phrase = withContext(Dispatchers.IO) {
                    Engine.open(this@SetupActivity).recoveryPhrase()
                }
                showPhrase(phrase)
            } catch (e: Exception) {
                fail("Could not show the words", e)
            }
        }
    }

    private fun showRestore() {
        views.chooser.visibility = View.GONE
        views.restorePanel.visibility = View.VISIBLE
    }

    private fun restore() {
        val phrase = views.phrase.text.toString().trim()
        if (phrase.isEmpty()) return

        busy(true)
        lifecycleScope.launch {
            try {
                Engine.restore(this@SetupActivity, phrase)
                // All 24 typed just now: nothing further to check.
                Engine.setPhraseConfirmed(this@SetupActivity, true)
                done()
            } catch (e: Exception) {
                busy(false)
                // The commonest failure by far, and the message says which word
                // count was seen because a missing word is the usual cause.
                fail("That phrase was not accepted", e)
            }
        }
    }

    private fun done() {
        startActivity(Intent(this, MainActivity::class.java))
        finish()
    }

    private fun busy(working: Boolean) {
        views.progress.visibility = if (working) View.VISIBLE else View.GONE
        views.create.isEnabled = !working
        views.restore.isEnabled = !working
        views.restoreConfirm.isEnabled = !working
    }

    private fun fail(title: String, e: Exception) {
        MaterialAlertDialogBuilder(this)
            .setTitle(title)
            .setMessage(if (e is QurbException) e.readable() else e.message ?: e.toString())
            .setPositiveButton("OK", null)
            .show()
    }
}

package com.qurb

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.widget.EditText
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.updatePadding
import androidx.lifecycle.lifecycleScope
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.qurb.databinding.ActivityMainBinding
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.qurb_mobile.FileEntry
import uniffi.qurb_mobile.PeerInfo
import java.io.File

/**
 * The app: five places under a tab bar, and the actions more than one of them
 * offers.
 *
 * Home says whether this phone's files are safe and connects devices; Vault is
 * what is on the phone; Devices is who it knows; Transfers is what is moving
 * and what happened; Settings is the rest. The screens are in their own files.
 * What lives here is what they share -- pairing, syncing, opening a file,
 * saving a copy, sending -- because each needs an activity to launch a picker
 * or a camera from, and there is one.
 */
class MainActivity : AppCompatActivity() {

    private lateinit var views: ActivityMainBinding
    private val screens = mutableMapOf<Int, Screen>()
    private var current: Screen? = null

    /** Whether a sync this screen started is still running. Home shows it. */
    var syncing = false
        private set

    /** Whether this launch has freed what nothing needs yet. Once is enough. */
    private var housekept = false

    private val adder = registerForActivityResult(
        ActivityResultContracts.OpenMultipleDocuments()
    ) { uris -> if (uris.isNotEmpty()) addFiles(uris) }

    /** The device files are being picked for, while the picker is open. */
    private var sendingTo: PeerInfo? = null

    private val sender = registerForActivityResult(
        ActivityResultContracts.OpenMultipleDocuments()
    ) { uris ->
        val to = sendingTo
        sendingTo = null
        if (to != null && uris.isNotEmpty()) sendPicked(uris, to)
    }

    private val scanner = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        result.data?.getStringExtra(ScanActivity.EXTRA_CODE)?.let { joinWith(it) }
    }

    /** The file waiting for a destination, while the save dialog is open. */
    private var pendingSave: FileEntry? = null

    private val saver = registerForActivityResult(
        ActivityResultContracts.CreateDocument("*/*")
    ) { destination ->
        val entry = pendingSave
        pendingSave = null
        if (destination != null && entry != null) writeCopy(entry, destination)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        if (!Engine.isSetUp(this)) {
            startActivity(Intent(this, SetupActivity::class.java))
            finish()
            return
        }

        views = ActivityMainBinding.inflate(layoutInflater)
        setContentView(views.root)
        insetContent()

        // Registered on every launch: `KEEP` makes it a no-op when the work is
        // already scheduled, and re-establishes it if the app's data was
        // cleared.
        SyncWorker.schedule(this)

        val tab = savedInstanceState?.getInt(TAB) ?: R.id.tab_home
        views.tabs.selectedItemId = tab
        views.tabs.setOnItemSelectedListener { item ->
            show(item.itemId)
            current?.refresh()
            true
        }
        views.tabs.setOnItemReselectedListener { current?.refresh() }
        // Not refreshed here: onResume follows, and refreshes whatever is showing.
        show(tab)
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        if (::views.isInitialized) outState.putInt(TAB, views.tabs.selectedItemId)
    }

    override fun onResume() {
        super.onResume()
        if (!::views.isInitialized) return
        current?.refresh()
        catchUp()
    }

    private fun show(tab: Int) {
        val screen = screens.getOrPut(tab) {
            when (tab) {
                R.id.tab_vault -> VaultScreen(this)
                R.id.tab_devices -> DevicesScreen(this)
                R.id.tab_transfers -> TransfersScreen(this)
                R.id.tab_settings -> SettingsScreen(this)
                else -> HomeScreen(this)
            }
        }
        if (screen === current) return
        views.screen.removeAllViews()
        views.screen.addView(screen.view)
        current = screen
    }

    /** Move to another tab, as a screen's action does ("Choose a device"). */
    fun go(tab: Int) {
        views.tabs.selectedItemId = tab
    }

    /**
     * Keep the screens out from under the status bar and the tab bar out from
     * under the gesture bar.
     *
     * Android 15 draws every app edge to edge, so without this a heading sits
     * beneath the status bar and taps near the top go to the system instead.
     * The tab bar pads itself for the navigation bar; everything else gets the
     * top and the sides here, once.
     */
    private fun insetContent() {
        ViewCompat.setOnApplyWindowInsetsListener(views.root) { _, insets ->
            val bars = insets.getInsets(
                WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            )
            views.screen.updatePadding(top = bars.top, left = bars.left, right = bars.right)
            insets
        }
    }

    /**
     * Whatever changed while the app was away.
     *
     * The screen has already drawn what the index knows. Then the scan, because
     * nothing delivers filesystem events to a process that was not running;
     * redrawn only if it found something. Then, once per launch, freeing what
     * nothing needs -- the routine the background worker runs after each sync,
     * here too so a phone the worker has not reached lately does not wait.
     */
    private fun catchUp() {
        lifecycleScope.launch {
            try {
                val engine = Engine.open(this@MainActivity)
                val found = withContext(Dispatchers.IO) { engine.scan() }
                if (found.stored > 0u || found.deleted > 0u) current?.refresh()

                if (!housekept) {
                    housekept = true
                    val tidied = withContext(Dispatchers.IO) {
                        runCatching { engine.housekeep() }.getOrNull()
                    }
                    if (tidied != null && tidied.freed > 0uL) current?.refresh()
                }
            } catch (e: Exception) {
                Words.fail(this@MainActivity, "Could not read this phone's files", e)
            }
        }
    }

    /** Something every screen might need to say. */
    fun say(message: String) = Words.say(views.root, message)

    fun fail(title: String, e: Throwable) = Words.fail(this, title, e)

    /** After anything that changes what the engine knows. */
    fun changed() = current?.refresh()

    fun sync() {
        if (syncing) return
        syncing = true
        changed()

        lifecycleScope.launch {
            try {
                val engine = Engine.open(this@MainActivity)
                // 25 seconds: generous for someone watching, and still inside
                // what a background window would grant. The deadline is the
                // point of `syncWithin` -- see decision 0020.
                val outcome = withContext(Dispatchers.IO) {
                    engine.scan()
                    // Holding the multicast lock, or the phone cannot hear the
                    // devices on its own Wi-Fi answering.
                    Engine.hearingTheNetwork(this@MainActivity) { engine.syncWithin(25u) }
                }
                say(
                    when {
                        outcome.reached == 0u && outcome.unreachable == 0u && outcome.timedOut ->
                            "Ran out of time before reaching a device. Sync again."
                        outcome.reached == 0u && outcome.unreachable == 0u ->
                            "No devices connected yet"
                        outcome.reached == 0u ->
                            "No device answered. It has to be switched on and running qurb."
                        outcome.adopted == 0u && outcome.conflicts == 0u ->
                            "Up to date"
                        else -> buildString {
                            append("${Words.files(outcome.adopted.toInt())} updated")
                            if (outcome.conflicts > 0u) {
                                append(", ${outcome.conflicts} changed on two devices at once")
                            }
                            if (outcome.timedOut) append(" — ran out of time, sync again")
                        }
                    }
                )
            } catch (e: Exception) {
                fail("Sync failed", e)
            } finally {
                syncing = false
                changed()
            }
        }
    }

    /**
     * Connect a device: scan the code it shows, or type it.
     *
     * The code carries the other device's whole identity, which is why it
     * travels across the room by camera rather than over the network.
     */
    fun pair() {
        MaterialAlertDialogBuilder(this)
            .setTitle("Connect a device")
            .setMessage(
                "On your computer, open qurb, go to Devices and choose Show a code. " +
                    "Then point this phone's camera at it.\n\n" +
                    "From a terminal, `qurb pair` shows the same code."
            )
            // Scanning first, because it is what anyone will actually do. A
            // pairing code is over a hundred characters; typing one is possible
            // and nobody does it twice.
            .setPositiveButton("Scan the code") { _, _ ->
                scanner.launch(Intent(this, ScanActivity::class.java))
            }
            .setNeutralButton("Type it") { _, _ -> typeCode() }
            .setNegativeButton("Cancel", null)
            .show()
    }

    /** The fallback, for a phone with no camera or a refused permission. */
    private fun typeCode() {
        val input = EditText(this).apply {
            hint = "qurb1-..."
            setPadding(48, 32, 48, 8)
        }
        MaterialAlertDialogBuilder(this)
            .setTitle("Type the code")
            .setView(input)
            .setPositiveButton("Connect") { _, _ ->
                val code = input.text.toString().trim()
                if (code.isNotEmpty()) joinWith(code)
            }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun joinWith(code: String) {
        lifecycleScope.launch {
            try {
                val peer = withContext(Dispatchers.IO) {
                    Engine.open(this@MainActivity).joinPairing(code)
                }
                say("Connected to ${peer.name}")
                changed()
            } catch (e: Exception) {
                fail("Could not connect", e)
            }
        }
    }

    fun pickFilesToAdd() = adder.launch(arrayOf("*/*"))

    /**
     * Copy files from elsewhere on the phone into it. Through the cache rather
     * than memory -- see [Engine.importUri] -- so a long video never has to fit
     * in the heap.
     */
    private fun addFiles(uris: List<Uri>) {
        lifecycleScope.launch {
            var added = 0
            try {
                for (uri in uris) {
                    Engine.importUri(this@MainActivity, uri)
                    added++
                }
                say("Added ${Words.files(added)}")
            } catch (e: Exception) {
                fail(if (added == 0) "Could not add that file" else "Added $added, then stopped", e)
            } finally {
                changed()
            }
        }
    }

    /** Ask which device, then do something with it. */
    fun chooseDevice(title: String, then: (PeerInfo) -> Unit) {
        lifecycleScope.launch {
            val peers = try {
                withContext(Dispatchers.IO) { Engine.open(this@MainActivity).peers() }
            } catch (e: Exception) {
                fail("Could not read your devices", e)
                return@launch
            }
            if (peers.isEmpty()) {
                MaterialAlertDialogBuilder(this@MainActivity)
                    .setTitle(title)
                    .setMessage("No devices connected yet. Connect one first.")
                    .setPositiveButton("Connect a device") { _, _ -> pair() }
                    .setNegativeButton("Cancel", null)
                    .show()
                return@launch
            }
            MaterialAlertDialogBuilder(this@MainActivity)
                .setTitle(title)
                .setItems(peers.map { it.name }.toTypedArray()) { _, which -> then(peers[which]) }
                .setNegativeButton("Cancel", null)
                .show()
        }
    }

    fun pickFilesToSend(to: PeerInfo) {
        sendingTo = to
        sender.launch(arrayOf("*/*"))
    }

    private fun sendPicked(uris: List<Uri>, to: PeerInfo) {
        lifecycleScope.launch {
            var sent = 0
            try {
                for (uri in uris) {
                    Engine.sendUri(this@MainActivity, uri, to.fingerprint)
                    sent++
                }
                say("${Words.files(sent)} for ${to.name}. It collects them at the next sync.")
            } catch (e: Exception) {
                fail(if (sent == 0) "Could not send that" else "Sent $sent, then stopped", e)
            } finally {
                changed()
            }
        }
    }

    /**
     * Send a file already on this phone. The engine keeps its own copy until
     * the other device collects it, so this works while that device is off.
     */
    fun send(entry: FileEntry, to: PeerInfo) {
        lifecycleScope.launch {
            try {
                withContext(Dispatchers.IO) {
                    val source = File(Engine.root(this@MainActivity), entry.path)
                    Engine.open(this@MainActivity)
                        .sendFile(source.absolutePath, entry.path.substringAfterLast('/'), to.fingerprint)
                }
                say("Sending to ${to.name}. It collects it at the next sync.")
            } catch (e: Exception) {
                fail("Could not send that", e)
            } finally {
                changed()
            }
        }
    }

    /**
     * Hand the file to whatever app handles its type, through the app's own
     * DocumentsProvider, so there is one way out of the store rather than two.
     */
    fun open(entry: FileEntry) {
        val intent = Intent(Intent.ACTION_VIEW).apply {
            setDataAndType(documentUri(entry.path), mimeType(entry.path))
            // Without this the receiving app cannot read the URI, and fails
            // with something that looks like a corrupt file.
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }
        try {
            startActivity(Intent.createChooser(intent, "Open with"))
        } catch (e: Exception) {
            fail("Nothing on this phone can open that", e)
        }
    }

    /**
     * Copy a file out to wherever the person chooses: Downloads, the gallery,
     * anywhere they keep things. The folder is this app's private storage, so
     * a file that lives only there is invisible to everything else.
     */
    fun saveCopy(entry: FileEntry) {
        pendingSave = entry
        try {
            saver.launch(entry.path.substringAfterLast('/'))
        } catch (e: Exception) {
            pendingSave = null
            fail("Could not open the save dialog", e)
        }
    }

    /**
     * Stream a stored file out to where the person picked. Exported to a cache
     * file and copied from there rather than held in memory: `export` writes a
     * chunk at a time so a large file never has to fit in the heap.
     */
    private fun writeCopy(entry: FileEntry, destination: Uri) {
        lifecycleScope.launch {
            try {
                withContext(Dispatchers.IO) {
                    val staging = File(cacheDir, "save-${System.nanoTime()}")
                    try {
                        Engine.open(this@MainActivity).export(entry.path, staging.absolutePath)
                        staging.inputStream().use { input ->
                            contentResolver.openOutputStream(destination)?.use { output ->
                                input.copyTo(output)
                            } ?: error("could not open the destination")
                        }
                    } finally {
                        staging.delete()
                    }
                }
                say("Saved a copy")
            } catch (e: Exception) {
                fail("Could not save that file", e)
            }
        }
    }

    private fun documentUri(path: String): Uri =
        android.provider.DocumentsContract.buildDocumentUri("com.qurb.documents", "qurb/$path")

    private fun mimeType(path: String): String {
        val extension = path.substringAfterLast('.', "").lowercase()
        return android.webkit.MimeTypeMap.getSingleton().getMimeTypeFromExtension(extension)
            ?: "application/octet-stream"
    }

    private companion object {
        const val TAB = "tab"
    }
}

package com.qurb

import android.content.res.ColorStateList
import android.text.Editable
import android.text.TextWatcher
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.qurb.databinding.RowFileBinding
import com.qurb.databinding.ScreenVaultBinding
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.qurb_mobile.Available
import uniffi.qurb_mobile.FileEntry

/**
 * What is on this phone, folder by folder, and where each file's bytes are.
 *
 * Every file says one of three things -- here and on another device, only
 * here, or on another device and not here -- because that is what decides
 * what a person can safely do with it, and the actions offered follow from it.
 *
 * Browsed a folder at a time from the engine's index, the same way the system
 * file picker lists qurb, so the two cannot disagree. Search looks at every
 * folder at once. Back goes up a folder before it leaves.
 */
class VaultScreen(app: MainActivity) : Screen(app) {

    private val views = ScreenVaultBinding.inflate(app.layoutInflater)
    override val view: View get() = views.root

    private val items = ItemAdapter()

    /** The folder being looked at; the empty string is the top. */
    private var dir = ""
    private var query = ""
    private var sort = Sort.NAME
    private var searching: Job? = null

    /** The files in the current list, for "save everything here". */
    private var shown: List<FileEntry> = emptyList()

    /** Who can be named as having a file, per [Words.where]. */
    private var ownKeeper: String? = null
    private var sharedKeeper: String? = null

    private enum class Sort(val label: String) { NAME("By name"), NEWEST("Newest"), LARGEST("Largest") }

    private sealed class Item {
        data class Folder(val name: String, val path: String) : Item()
        data class File(val entry: FileEntry) : Item()
    }

    init {
        views.action.setOnClickListener { app.pickFilesToAdd() }
        views.refresh.setOnRefreshListener { refresh() }
        views.list.layoutManager = LinearLayoutManager(app)
        views.list.adapter = items

        views.up.setOnClickListener { goUp() }
        views.sort.setOnClickListener {
            sort = Sort.entries[(sort.ordinal + 1) % Sort.entries.size]
            views.sort.text = sort.label
            refresh()
        }
        views.more.setOnClickListener { more() }
        views.search.addTextChangedListener(object : TextWatcher {
            override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {}
            override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {}
            override fun afterTextChanged(s: Editable?) {
                query = s?.toString()?.trim() ?: ""
                // A quarter of a second after the last key, not on every one.
                searching?.cancel()
                searching = scope.launch {
                    delay(250)
                    refresh()
                }
            }
        })
    }

    override fun back(): Boolean = when {
        query.isNotEmpty() -> {
            views.search.setText("")
            true
        }
        dir.isNotEmpty() -> {
            goUp()
            true
        }
        else -> false
    }

    private fun goUp() {
        if (dir.isEmpty()) return
        dir = dir.substringBeforeLast('/', "")
        refresh()
    }

    private fun open(folder: String) {
        dir = folder
        refresh()
    }

    override fun refresh() {
        views.up.text = if (dir.isEmpty()) "All files" else "‹  $dir"
        views.up.isEnabled = dir.isNotEmpty() && query.isEmpty()
        scope.launch {
            try {
                val (folders, files, usage) = withContext(Dispatchers.IO) {
                    val engine = engine()
                    val peers = engine.peers()
                    val holders = engine.holders()
                    // A private file reaches only the devices that keep this
                    // phone's files; a shared one, any connected device. Named
                    // only when exactly one could have it.
                    ownKeeper = holders.singleOrNull()?.name
                    sharedKeeper = peers.singleOrNull()?.name
                    val usage = engine.usage()
                    if (query.isNotEmpty()) {
                        Triple(emptyList(), engine.search(query, SEARCH_LIMIT.toUInt()), usage)
                    } else {
                        val listing = engine.browse(dir)
                        Triple(listing.folders, listing.files, usage)
                    }
                }
                views.subtitle.text =
                    "${Words.size(usage.logical)} of files · qurb uses ${Words.size(usage.onDisk)} of this phone"
                shown = sorted(files)
                val prefix = if (dir.isEmpty()) "" else "$dir/"
                items.submit(folders.map { Item.Folder(it, prefix + it) } + shown.map { Item.File(it) })
                views.empty.visibility = if (folders.isEmpty() && files.isEmpty()) View.VISIBLE else View.GONE
                views.empty.text = when {
                    query.isNotEmpty() -> "Nothing called “$query”."
                    dir.isNotEmpty() -> "Nothing in this folder."
                    else -> "Nothing here yet.\n\nAdd files from this phone, or connect a device " +
                        "and sync to see what it shares."
                }
            } catch (e: Exception) {
                app.fail("Could not read this phone's files", e)
            } finally {
                views.refresh.isRefreshing = false
            }
        }
    }

    private fun sorted(files: List<FileEntry>): List<FileEntry> = when (sort) {
        Sort.NAME -> files.sortedBy { it.path.lowercase() }
        Sort.NEWEST -> files.sortedByDescending { it.modifiedAt }
        Sort.LARGEST -> files.sortedByDescending { it.size }
    }

    /** What applies to the whole list rather than one file. */
    private fun more() {
        val here = shown.filter { it.available != Available.ELSEWHERE }
        val actions = buildList<Pair<String, () -> Unit>> {
            if (here.isNotEmpty()) {
                add("Save ${Words.files(here.size)} here to this phone…" to { app.saveAll(here) })
            }
            add("Recently deleted" to { RecentlyDeleted.show(app) })
        }
        MaterialAlertDialogBuilder(app)
            .setItems(actions.map { it.first }.toTypedArray()) { _, which -> actions[which].second() }
            .setNegativeButton("Cancel", null)
            .show()
    }

    /** What can be done with this file, which depends on where its bytes are. */
    private fun choose(entry: FileEntry) {
        val here = entry.available != Available.ELSEWHERE
        val actions = buildList<Pair<String, () -> Unit>> {
            if (here) {
                add("Open" to { app.open(entry) })
                add("Save a copy to this phone" to { app.saveCopy(entry) })
                add("Send to a device…" to {
                    app.chooseDevice("Send to") { to -> app.send(entry, to) }
                })
            } else {
                add("Download to this phone" to { fetch(entry) })
            }
            if (entry.available == Available.HERE) add("Free phone space" to { free(entry) })
            add("Delete" to { delete(entry) })
        }
        MaterialAlertDialogBuilder(app)
            .setTitle(entry.path.substringAfterLast('/'))
            .setItems(actions.map { it.first }.toTypedArray()) { _, which -> actions[which].second() }
            .setNegativeButton("Cancel", null)
            .show()
    }

    /**
     * Free this phone's copy. The engine refuses when no other device is known
     * to hold the file, so the menu offering it only for files marked as held
     * elsewhere is a courtesy rather than the protection.
     */
    private fun free(entry: FileEntry) {
        scope.launch {
            try {
                val freed = withContext(Dispatchers.IO) { engine().freeLocal(entry.path) }
                app.say("Freed ${Words.size(freed)}. Tap the file to download it again.")
            } catch (e: Exception) {
                app.fail("Could not free that", e)
            } finally {
                app.changed()
            }
        }
    }

    /** Ask for it back. Acted on at the next sync with a device that has it. */
    private fun fetch(entry: FileEntry) {
        scope.launch {
            try {
                withContext(Dispatchers.IO) { engine().fetch(entry.path) }
                app.say("It comes back at the next sync")
                app.sync()
            } catch (e: Exception) {
                app.fail("Could not ask for that", e)
            }
        }
    }

    private fun delete(entry: FileEntry) {
        val where = if (entry.private) {
            "It is deleted from this phone, and from any device keeping your files at its next sync."
        } else {
            "It is shared, so it is deleted on every device at its next sync."
        }
        MaterialAlertDialogBuilder(app)
            .setTitle("Delete ${entry.path.substringAfterLast('/')}?")
            .setMessage("$where Recently deleted keeps it on this phone for 30 days.")
            .setPositiveButton("Delete") { _, _ ->
                scope.launch {
                    try {
                        withContext(Dispatchers.IO) { engine().remove(entry.path) }
                    } catch (e: Exception) {
                        app.fail("Could not delete that", e)
                    } finally {
                        app.changed()
                    }
                }
            }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private inner class ItemAdapter : RecyclerView.Adapter<ItemHolder>() {
        private var list: List<Item> = emptyList()

        fun submit(next: List<Item>) {
            list = next
            notifyDataSetChanged()
        }

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int) = ItemHolder(
            RowFileBinding.inflate(LayoutInflater.from(parent.context), parent, false)
        )

        override fun onBindViewHolder(holder: ItemHolder, position: Int) = holder.bind(list[position])
        override fun getItemCount() = list.size
    }

    private inner class ItemHolder(private val row: RowFileBinding) : RecyclerView.ViewHolder(row.root) {
        fun bind(item: Item) {
            when (item) {
                is Item.Folder -> {
                    row.root.setOnClickListener { open(item.path) }
                    row.name.text = "${item.name}/"
                    row.where.text = "Folder"
                    row.dot.visibility = View.INVISIBLE
                }
                is Item.File -> {
                    val entry = item.entry
                    row.root.setOnClickListener { choose(entry) }
                    // The whole path when searching, where the folder is part
                    // of the answer; just the name inside a folder.
                    row.name.text = if (query.isNotEmpty()) entry.path else entry.path.substringAfterLast('/')
                    val (where, colour) = Words.where(
                        app,
                        entry,
                        if (entry.private) ownKeeper else sharedKeeper,
                    )
                    row.where.text = "$where · ${Words.size(entry.size)}"
                    row.dot.visibility = View.VISIBLE
                    row.dot.imageTintList = ColorStateList.valueOf(colour)
                }
            }
        }
    }

    private companion object {
        const val SEARCH_LIMIT = 500
    }
}

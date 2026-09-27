package com.qurb

import android.content.res.ColorStateList
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.qurb.databinding.RowFileBinding
import com.qurb.databinding.ScreenListBinding
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.qurb_mobile.Available
import uniffi.qurb_mobile.FileEntry
import uniffi.qurb_mobile.PeerInfo
import uniffi.qurb_mobile.Usage

/**
 * What is on this phone, and where each file's bytes are.
 *
 * Every row says one of three things -- here and on another device, only
 * here, or on another device and not here -- because that is what decides
 * what a person can safely do with it. The actions offered follow from it:
 * space can be freed only where another copy exists, and a file that is not
 * here can be asked for back rather than opened.
 *
 * Read a page at a time. A phone that has synced a large library would
 * otherwise cross the FFI boundary with every file in it before drawing one.
 */
class VaultScreen(app: MainActivity) : Screen(app) {

    private val views = ScreenListBinding.inflate(app.layoutInflater)
    override val view: View get() = views.root

    private val files = FileAdapter()
    private var loading = false
    private var more = false

    /** Who can be named as having a file, per [Words.where]. */
    private var ownKeeper: String? = null
    private var sharedKeeper: String? = null

    init {
        views.title.text = "Vault"
        views.action.text = "Add files"
        views.action.setIconResource(R.drawable.ic_add)
        views.action.setOnClickListener { app.pickFilesToAdd() }
        views.empty.text = "Nothing here yet.\n\nAdd files from this phone, or connect a " +
            "device and sync to see what it shares."
        views.refresh.setOnRefreshListener { refresh() }

        val layout = LinearLayoutManager(app)
        views.list.layoutManager = layout
        views.list.adapter = files
        views.list.addOnScrollListener(object : RecyclerView.OnScrollListener() {
            override fun onScrolled(list: RecyclerView, dx: Int, dy: Int) {
                if (more && !loading && layout.findLastVisibleItemPosition() >= files.itemCount - 20) {
                    load(files.itemCount)
                }
            }
        })
    }

    override fun refresh() = load(0)

    private fun load(offset: Int) {
        loading = true
        scope.launch {
            try {
                val loaded = withContext(Dispatchers.IO) {
                    val engine = engine()
                    // The rest only with the first page: they change what the
                    // whole list says, and a later page is the same list.
                    val first = offset == 0
                    Loaded(
                        engine.page(offset.toUInt(), PAGE.toUInt()),
                        if (first) engine.usage() else null,
                        if (first) engine.peers() else null,
                        if (first) engine.holders() else null,
                    )
                }
                val page = loaded.page
                if (loaded.peers != null && loaded.holders != null) {
                    // A private file reaches only the devices that keep this
                    // phone's files; a shared one, any connected device. Named
                    // only when exactly one could have it.
                    ownKeeper = loaded.holders.singleOrNull()?.name
                    sharedKeeper = loaded.peers.singleOrNull()?.name
                }
                loaded.usage?.let {
                    views.subtitle.text =
                        "${Words.size(it.logical)} of files · qurb uses ${Words.size(it.onDisk)} " +
                            "of this phone"
                }
                more = page.size == PAGE
                if (offset == 0) files.replace(page) else files.append(page)
                views.empty.visibility = if (files.itemCount == 0) View.VISIBLE else View.GONE
            } catch (e: Exception) {
                app.fail("Could not read this phone's files", e)
            } finally {
                loading = false
                views.refresh.isRefreshing = false
            }
        }
    }

    private class Loaded(
        val page: List<FileEntry>,
        val usage: Usage?,
        val peers: List<PeerInfo>?,
        val holders: List<PeerInfo>?,
    )

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
            "It is deleted from this phone, and from any device keeping your files at " +
                "its next sync."
        } else {
            "It is shared, so it is deleted on every device at its next sync."
        }
        MaterialAlertDialogBuilder(app)
            .setTitle("Delete ${entry.path.substringAfterLast('/')}?")
            .setMessage(where)
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

    private inner class FileAdapter : RecyclerView.Adapter<FileHolder>() {
        private val items = mutableListOf<FileEntry>()

        fun replace(next: List<FileEntry>) {
            items.clear()
            items.addAll(next)
            notifyDataSetChanged()
        }

        fun append(next: List<FileEntry>) {
            val start = items.size
            items.addAll(next)
            notifyItemRangeInserted(start, next.size)
        }

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int) = FileHolder(
            RowFileBinding.inflate(LayoutInflater.from(parent.context), parent, false)
        )

        override fun onBindViewHolder(holder: FileHolder, position: Int) = holder.bind(items[position])
        override fun getItemCount() = items.size
    }

    private inner class FileHolder(private val row: RowFileBinding) : RecyclerView.ViewHolder(row.root) {
        fun bind(entry: FileEntry) {
            row.root.setOnClickListener { choose(entry) }
            row.name.text = entry.path
            val (where, colour) = Words.where(
                app,
                entry,
                if (entry.private) ownKeeper else sharedKeeper,
            )
            row.where.text = where
            row.dot.imageTintList = ColorStateList.valueOf(colour)
        }
    }

    private companion object {
        const val PAGE = 200
    }
}

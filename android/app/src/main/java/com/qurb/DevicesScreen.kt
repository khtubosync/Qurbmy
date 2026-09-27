package com.qurb

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.qurb.databinding.RowDeviceBinding
import com.qurb.databinding.ScreenListBinding
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.qurb_mobile.PeerInfo

/**
 * The devices this phone knows, and which of them keep its files.
 *
 * Keeping is the choice decision 0036 is about: this phone's own files go to
 * no other device unless the person picks one to keep them, and the device
 * picked keeps them where nobody using it sees them. The choice lives here
 * because it is a choice about a device.
 */
class DevicesScreen(app: MainActivity) : Screen(app) {

    private val views = ScreenListBinding.inflate(app.layoutInflater)
    override val view: View get() = views.root

    private val devices = DeviceAdapter()

    init {
        views.title.text = "Devices"
        views.subtitle.text = "A device that keeps your files has a copy of what this phone " +
            "adds, where only you can get at it."
        views.action.text = "Connect a device"
        views.action.setIconResource(R.drawable.ic_link)
        views.action.setOnClickListener { app.pair() }
        views.empty.text = "No devices yet.\n\nOpen qurb on your computer, go to Devices and " +
            "choose Show a code. Then tap Connect a device here."
        views.refresh.setOnRefreshListener { refresh() }
        views.list.layoutManager = LinearLayoutManager(app)
        views.list.adapter = devices
    }

    override fun refresh() {
        scope.launch {
            try {
                val (peers, holders) = withContext(Dispatchers.IO) {
                    val engine = engine()
                    engine.peers() to engine.holders().map { it.fingerprint }.toSet()
                }
                devices.submit(peers, holders)
                views.empty.visibility = if (peers.isEmpty()) View.VISIBLE else View.GONE
            } catch (e: Exception) {
                app.fail("Could not read your devices", e)
            } finally {
                views.refresh.isRefreshing = false
            }
        }
    }

    private fun choose(peer: PeerInfo, keeps: Boolean) {
        val actions = listOf<Pair<String, () -> Unit>>(
            (if (keeps) "Stop keeping my files here" else "Keep my files here") to {
                if (keeps) stopKeeping(peer) else keep(peer)
            },
            "Send files…" to { app.pickFilesToSend(peer) },
        )
        MaterialAlertDialogBuilder(app)
            .setTitle(peer.name)
            .setItems(actions.map { it.first }.toTypedArray()) { _, which -> actions[which].second() }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun keep(peer: PeerInfo) {
        MaterialAlertDialogBuilder(app)
            .setTitle("Keep your files on ${peer.name}?")
            .setMessage(
                "${peer.name} keeps a copy of the files this phone adds, starting at the next " +
                    "sync. Nobody using ${peer.name} sees them; they are yours, and come back " +
                    "to this phone when you ask.\n\n" +
                    "Once it has a file, this phone can free its own copy to save space."
            )
            .setPositiveButton("Keep them there") { _, _ -> setKeeping(peer, true) }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun stopKeeping(peer: PeerInfo) {
        MaterialAlertDialogBuilder(app)
            .setTitle("Stop keeping your files on ${peer.name}?")
            .setMessage(
                "Nothing new goes to ${peer.name}. What it already has, it keeps: this phone " +
                    "does not reach into another device.\n\n" +
                    "A file this phone freed because ${peer.name} had it is still there."
            )
            .setPositiveButton("Stop") { _, _ -> setKeeping(peer, false) }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun setKeeping(peer: PeerInfo, keep: Boolean) {
        scope.launch {
            try {
                withContext(Dispatchers.IO) {
                    if (keep) engine().addHolder(peer.fingerprint) else engine().removeHolder(peer.fingerprint)
                }
                app.say(if (keep) "${peer.name} keeps your files from the next sync" else "Stopped")
            } catch (e: Exception) {
                app.fail("Could not change that", e)
            } finally {
                app.changed()
            }
        }
    }

    private inner class DeviceAdapter : RecyclerView.Adapter<DeviceHolder>() {
        private var peers: List<PeerInfo> = emptyList()
        private var keepers: Set<String> = emptySet()

        fun submit(next: List<PeerInfo>, holders: Set<String>) {
            peers = next
            keepers = holders
            notifyDataSetChanged()
        }

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int) = DeviceHolder(
            RowDeviceBinding.inflate(LayoutInflater.from(parent.context), parent, false)
        )

        override fun onBindViewHolder(holder: DeviceHolder, position: Int) =
            holder.bind(peers[position], peers[position].fingerprint in keepers)

        override fun getItemCount() = peers.size
    }

    private inner class DeviceHolder(private val row: RowDeviceBinding) : RecyclerView.ViewHolder(row.root) {
        fun bind(peer: PeerInfo, keeps: Boolean) {
            row.root.setOnClickListener { choose(peer, keeps) }
            row.name.text = peer.name
            val seen = peer.lastSeen?.let { "Last reached ${Words.ago(it)}" } ?: "Not reached yet"
            row.detail.text = "$seen · ${peer.short}"
            row.keeps.visibility = if (keeps) View.VISIBLE else View.GONE
        }
    }
}

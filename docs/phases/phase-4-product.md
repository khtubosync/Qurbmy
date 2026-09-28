# Phase 4 — Desktop product

**Status:** in progress
**Target:** months 8–10

Turning a working engine into something a person can run. The phase the roadmap
warns is not the fun part and is a full quarter.

## Progress

| area | status |
|---|---|
| a daemon that runs | ✅ [`qurb`](../../crates/qurb/) |
| the commands around it | ✅ init, enrol, pair, join, run, status, verify, reclaim, fetch, send, activity, ls, find, config, protect |
| running the services | ✅ `qurb signal`, `qurb relay` |
| push, rather than polling | ✅ ~430ms, measured |
| protecting the key at rest | ✅ keystore and passphrase |
| storing files in parallel | ✅ 487 → 830-888 files/s |
| onboarding and the recovery phrase | ◐ in the window, phrase shown and confirmed; the storage question is not asked yet ([0038](../decisions/0038-the-storage-question-during-setup.md)) |
| installers | ⬜ not started |
| signed updates with rollback | ⬜ not started |
| observability | ◐ structured logs, nothing more |
| the interface | ◐ a window: set up, pair, send, and where every file is — see *The window* |
| one daemon per folder | ✅ an advisory lock, not a convention |
| pairing | ✅ scan a QR code, once, and it stays paired |
| a folder that needs no path | ✅ `~/qurb`, with a registry (it was `~/Downloads/qurb` until [0037](../decisions/0037-a-file-sent-to-a-desktop-is-an-ordinary-file.md)) |
| files sent to this desktop | ✅ saved to `Downloads/qurb` as ordinary files |
| transfer progress | ✅ both directions, live, with rate and time left |
| cancelling a send | ✅ before it is collected; ⬜ stopping one mid-transfer |
| storing each file once | ✅ the folder *is* the payload store |
| a storage cap | ✅ limit, eviction, fetch-back — and a slider |
| a replica anybody can run | ✅ `qurb replica` |
| syncing from another network | ✅ needs a reachable rendezvous; see anywhere.md |
| changes crossing at once | ✅ ~1s between idle daemons, no polling |
| the services deployable | ✅ systemd units, TLS, ports written down |
| garbage collection running | ✅ every 5 minutes, 7-day retention |

696 tests pass in 81 test binaries on Linux (2026-09-28, debug build, the
development laptop); clippy is clean.

## The interface

A system tray icon — [`crates/tray`](../../crates/tray/) — that runs the daemon
rather than talking to one. A socket between them would buy attaching to an
already-running daemon and cost a second surface to design, version and secure;
two daemons on one store collide on the SQLite lock regardless, so the choice is
really which single process runs. `qurb run` stays for machines with no screen.

This needed the daemon to say what it is doing. It had only ever had its log,
which is right for a terminal and useless to an interface: a person wants "up to
date, three devices, last synced two minutes ago", not a stream of events to
reconstruct it from. The daemon now publishes a `Status` on a `watch` channel —
watch rather than broadcast, because a display only wants the current value and
an icon catching up through stale summaries would be showing the past.

**It will not say "up to date" when no device has been reached.** A contented
icon beside a store that has not spoken to another device in a week is the kind
of lie that makes people stop trusting a sync tool, so `State::Alone` exists and
reads "no devices reachable". It also separates *no paired devices* from *paired
but unreachable*: one is a setup step, the other a network problem.

### GNOME has no tray, and says nothing about it

The nastiest part. GNOME removed the system tray and ships no StatusNotifier
host, so an icon there is invisible — and creating one still **succeeds**.
Nothing returns an error. A program trusting that return value is running,
unseen and unquittable.

So the session bus is asked whether anything has registered
`org.kde.StatusNotifierWatcher` before anything is built. With no watcher the
program says so, explains how to get a tray back, and keeps syncing with status
on stderr.

The check has to come before GTK is touched for a second reason, found by
running it: `tray-icon` is GTK-backed on Linux and *panics* if a menu is
constructed before `gtk::init`. An error path alone would never have run.

### Two bugs the fallback exposed at once

Having a display made two invisible problems visible immediately.

It reported **"0 files" with two in the store**, because counting happened after
the networking that had just failed. What a device holds is knowable without
reaching anything, and an interface showing zero because a service is down is
worse than showing nothing.

And when the daemon died, the status still said **"syncing"** — the failure
returned without reporting it, so the display kept showing the last thing that
had been true. It now records why it is stopping before it stops.

Neither was a new bug. Both had been there all along, invisible because nothing
was looking.

## Protecting the key at rest

The largest security gap the project had. The master key sat in a file readable
only by its owner, which defends against other users of the machine and against
nothing that can read the disk.

Three options now, and the difference between them is worth stating because a
user reading "end-to-end encrypted" will assume the strongest:

| | defends against | starts unattended |
|---|---|---|
| `file` | other users of the machine | yes |
| `keystore` | anyone reading the disk while it is locked | yes |
| `passphrase` | anyone who takes the disk *and* the session | no |

File remains the default, which looks like timidity and is not: a headless
machine may have neither a keystore nor anybody to type a passphrase, and a
device that cannot unlock itself is worse than one whose key sits in a file.
`qurb protect` changes it, and says what each option does before it does
anything.

Changing the protection changes the lock and not the contents — the key is read
out and written back, so nothing it protects becomes unreadable. The old copy is
removed only once the new one is in place, because a device that loses its key
halfway through being made safer has been made catastrophically less safe.

The keystore path is tested here against the Secret Service. Keychain and the
Windows Credential Manager go through the same library and are exercised by
nothing, which the documentation says rather than implying otherwise.

## The daemon

Until now nothing outside tests and examples could be run, and the demo put two
devices in one process — which meant the one thing that mattered for the Phase 3
kill criterion, *two machines on two networks*, was impossible.

`qurb run` watches a directory, keeps the store in step with it, serves peers on
every path it has, and pulls from each paired device it can reach.

## Three bugs that only running it could find

Every one of these passed the whole test suite.

### An invite offered an address nobody could dial

`qurb pair` bound `0.0.0.0` and put that in the invite. It is a true statement of
where the socket is listening — every interface — and completely useless to a
peer, which cannot connect to it.

Every test bound `127.0.0.1` explicitly and so never saw it. The failure appeared
at the *joining* device, which reported that it could not connect, making it look
like the joiner's problem.

The fix asks the routing table rather than listing interfaces: a UDP socket
*connected* to an arbitrary address sends nothing but makes the operating system
choose a source address, and the one it chooses is the one that would really be
used. Listing interfaces instead means guessing between a wired connection, a
wireless one, a virtual machine bridge and three container networks.

### Two devices started together took two minutes to find each other

The first sync attempt happens before the other device has announced, which fails
— and the retry backoff went straight to two minutes. Starting both daemons at
once, as anyone would, produced a system that appeared to do nothing at all.

The two failures are indistinguishable at the moment they happen and want
opposite treatment: a device still starting up should be retried in seconds, one
that is switched off should be left alone. So the backoff now doubles from five
seconds to a two-minute cap. Simultaneous startup went from 120 seconds to **9**.

### An edit took thirty seconds to cross

A device syncs when *it* changes something, or when its timer fires. An edit made
on the laptop therefore reached the desktop only when the desktop next asked.

**Now fixed.** A device holds one request open against each peer — "tell me when
your state differs from this" — and the answer arrives when it does.

Measured, two devices on this machine, the same edit three times: **433ms, 432ms,
431ms**, against up to ten seconds before. Most of what remains is the watcher
deliberately waiting to see whether the file is still being written, which is a
floor worth having rather than latency to remove. Thirty seconds of complete
idleness produced no log activity at all, so the responsiveness is not bought
with chatter.

It does not break the rule that **a peer can ask and never tell**: the device
that wants to know is the one asking, and the answer simply arrives later than
usual. A peer still cannot make anything happen.

A counter rather than a flag, because a flag can be missed — a peer told
"something changed" cannot distinguish a notification it has already acted on
from a new one. It says what it last saw, and gets an immediate answer if
anything has happened since. The counter deliberately does not survive a restart:
a peer holding a number from before sees one that does not match, concludes the
device it was watching has been away, and looks. Which is right.

## What running it confirmed

Two devices, two directories, real sockets: paired over the LAN, synced in both
directions, propagated a deletion, and resolved a genuine conflict — both
versions kept, the same conflict filename computed independently on each device,
and both agreeing on the resulting file list afterwards. `qurb verify --deep`
reports everything agrees.

## Pairing, once — and a folder nobody has to name

Three things stood between the daemon and something a person could be handed.

### "Paired" did not survive the daemon restarting — or rather, starting

The TLS verifier held its list of trusted devices as a `Vec<Fingerprint>`
snapshot, taken when the connection layer was built. `qurb pair` runs as a
*separate process*, so a daemon that was already running never learned about a
device paired afterwards. Pairing appeared to work — it wrote the fingerprint —
and then nothing connected until the daemon was restarted.

The snapshot is now a `TrustList`: an `Arc<RwLock<Vec<Fingerprint>>>` shared
between the verifier and the daemon, replaced wholesale every five seconds from
disk. Wholesale rather than appended, so that *forgetting* a device takes effect
too. Learning a new fingerprint also triggers an immediate sync rather than
waiting for the next interval.

Measured on two fresh devices on the development laptop, 2026-09-22: **five
seconds** from `qurb pair` completing to the devices connected, against never
without a restart.

### The pairing code was eight lines of base32 read off a screen

Now `qurb pair` renders the same invite as a QR code in the terminal, and the
Android app scans it with CameraX and ML Kit. The invite is unchanged — same
bytes, same expiry, same one-time use — so nothing about the security argument
moves. What changes is that a phone no longer needs a keyboard, and a person no
longer needs to transcribe a fingerprint to know they paired with the right
machine.

### Every command wanted a path

`qurb run ~/qurb` is fine once. It is not fine as the thing a person types
daily, and it is impossible as the thing a desktop launcher does. Commands now
default to a folder: `$XDG_DOWNLOAD_DIR/qurb`, falling back to
`~/Downloads/qurb`, with a small registry under `$XDG_CONFIG_HOME/qurb/folders`
recording which folders exist so the most recently used one wins. The legacy
`~/qurb` is still found if it is there.

Since 2026-09-25 a *new* folder goes in `~/qurb` again. `Downloads/qurb` became
where files sent to this device are saved, and the two must never overlap — see
[decisions/0037](../decisions/0037-a-file-sent-to-a-desktop-is-an-ordinary-file.md).
A folder already in Downloads stays there and is still found.

Several people on one computer is answered by several operating-system
accounts rather than by a qurb-level notion of a user — see
[decisions/0023](../decisions/0023-one-person-per-account.md).

## Every file cost twice its size

Content lived in two places on every syncing device: the file in `~/qurb`, and
compressed, encrypted chunks of the same bytes in `~/qurb/.qurb/chunks`. Nobody
had noticed because the test corpora were small and the store was never
compared against the folder that produced it.

Measured on the development laptop, 2026-09-22: 7.6 MB of files in `~/qurb`,
7.5 MiB of chunk payloads holding exactly that same content.

The fix is that a materialised file *is* the payload store for the chunks it
holds — the chunk store keeps only what the folder cannot supply, and reads
that find no payload seek into the file and hash-check what they find.
[decisions/0024](../decisions/0024-the-file-is-the-payload-store.md) records
what that costs, which is not nothing: editing a file now makes its superseded
versions unreadable rather than recoverable.

`qurb reclaim` frees the duplicates an older store still holds. On the real
laptop store: 7.5 MiB across 15 chunks, `qurb verify --deep` clean afterwards.

This was found while sizing up the storage cap, and it had to be fixed first: a
cap that counts every file twice is a cap on half of what the user thinks they
are limiting.

## Telling a device how much disk it may use

`qurb config <dir> limit=10G`. Over the limit, qurb drops local copies of the
coldest files and keeps everything the index knows about them, so the path
still syncs, still lists, and comes back on `qurb fetch`.

The interesting part of this feature is not the freeing. It is the two refusals.

**A file is dropped only when another device is known to hold those exact
bytes**, recorded when this device adopts a version another device made. So a
device never evicts content it originated — the phone keeps the photo it took,
the desktop may drop the copy it was sent. A device that cannot free enough
stays over its limit and says so in the log, which looks like a bug and is not:
a limit is a promise about disk, and no number in a settings box outranks the
only copy of someone's work.

**Dropping a file must not look like deleting it.** A syncing device decides a
file was deleted by not finding it during a scan — so without care, a device
short of disk drops a file, the next scan calls that a deletion, and the file
is deleted on every other device. The index now records whether this device is
*holding* each file separately from whether the file is there, set before the
unlink and never after, and both the scan and the watcher skip a file that is
missing on purpose.

Measured on the development laptop, 2026-09-22, two devices on loopback over a
real QUIC connection with a 2 MiB limit and 4.8 MiB of content:

| | device that received the files | device that made them |
|---|---|---|
| dropped | 1 file, 3.0 MB | nothing |
| ended at | 1.9 MiB, under the limit | 5.0 MB, 2.8 MB over, deliberately |

Neither device recorded a deletion, and the other device still held both files.
`qurb fetch` brought the dropped file back on the following sweep, byte-identical.

Garbage collection is part of this and had never run: `Store::gc` was written
and tested in Phase 1 and had no caller outside tests, so superseded chunks
accumulated without limit — 82 MB of them on this laptop for one deleted file.
The daemon now collects every five minutes with a seven-day retention window,
before it checks the limit. Collecting first costs the user nothing; only then
is it fair to drop copies of files they still have.

Two things this leaves undone. An evicted file **vanishes from the folder** on
Linux, which has no placeholder API — `qurb status` is the only place that says
it still exists. And `qurb fetch` waits for the next sweep rather than nudging
the daemon, so a file can take up to two minutes to come back.

## A window with a slider in it

The tray icon was the interface, and on GNOME there is no tray, so on the most
common Linux desktop the interface was a window that could only be *read*. The
one setting people actually want to change — how much of the disk qurb may
have — was reachable only by typing `qurb config <dir> limit=10G`.

So the window now has it: a usage bar, a checkbox for whether there is a limit
at all, and a slider that runs from 1 GiB to the size of the filesystem the
folder is on. Offering more than the disk holds would be offering a number that
cannot mean anything.

Two details decided the design.

**The slider writes the settings file, and nothing else.** Not a socket to the
daemon, not shared memory — the file the daemon already reads. That makes
`qurb config limit=10G` in a terminal and the slider in the window literally
the same act, and neither can leave the other showing something stale. The
daemon re-reads it on every maintenance pass, so a change applies without a
restart; a setting that needs a restart is a setting people think is broken.

**The window reads the file too, rather than the daemon's published status.**
The daemon only republishes on a sync pass, so a slider driven from the status
would sit at its old value for up to two minutes after someone moved it —
visibly snapping back under their finger.

## Two daemons on one folder

Found by running one: the desktop app launched from the applications menu and
`qurb run` typed into a terminal are the same daemon with different faces, and
neither knew about the other. Two of them on one store contend on the SQLite
write lock, answer as the same device on the network, both reconcile the same
directory and both enforce the same cap — and none of that reports an error. It
is slow and confusing rather than broken, which is worse.

`crates/qurb/src/lock.rs` takes an advisory `flock` on the store for the life
of the daemon. The kernel releases it when the process dies, which a lock file
holding a PID could not promise after a crash. The second daemon now refuses
with a sentence saying what is already running.

The same run turned up two smaller things. The window showed nothing when the
daemon failed underneath it — it reported files and folders from a process that
had stopped — so a problem now appears in the window in red. And the folder
registry had test folders in it from an afternoon's work, which meant the app
launched from the applications menu opened a throwaway directory in `/tmp`
rather than the real one. That one is a lesson about test hygiene rather than a
defect: the registry is per-user configuration, and tests must not write to it.

## Nobody waits for a poll

Two idle daemons, both connected to the same rendezvous service, still took up
to two minutes to move a file neither was busy with — because a device that had
just changed something had no way to say so, and the peer had no reason to ask.

`Waiting { to }` says there is something for a member: who, never what. The
service forwards it to peers that are connected and keeps it for peers that are
not, delivering it the moment they appear. That second half is the one that
matters, because the device most in need of telling is the one that was asleep
when the change happened.

Measured, two idle daemons on this laptop: a file written on one was on the
other **one second later**. The sender recorded the change at 27.087, the
recipient logged the notice at 27.088, and the transfer finished at 27.099.

Notes collapse — fifty changes for one absent peer leave one note, since the
answer to "should I sync" is the same either way — and a device never nudges
back the peer the news came from, which would loop for ever.

## Two devices that were never reachable enough

Two connection defects turned up together, and both had been there all along.

**A device announced one address**: whichever its default route used. On a
laptop at home that is `192.168.1.5`, which is nothing at all to a phone on a
mobile carrier — so every connection from outside the house depended on
punching a hole through the home NAT, and an overlay network's address, which
would have worked first time, was never mentioned. `Endpoints.local` was
already a list and the candidates were already raced; only the filling-in was
missing.

That broke two tests immediately, and both were real.

Announcing the machine's other interfaces when the socket is bound to *one*
address advertises places nothing is accepting — a peer races addresses that
can only fail and concludes the device is unreachable. It enumerates only for a
wildcard bind now.

And racing four addresses instead of one grew a 128 MiB transfer from about
30 MiB of heap to 105. Dropping a handshake that has *already finished* leaves a
connection established at the far end holding the buffers of a transfer nobody
will use, and a device reachable on both a local network and an overlay hits
that every time. The losing paths are closed as they land.

**A rendezvous service restart disconnected every device permanently.** The
signalling connection went with it and never came back; the daemon kept syncing
on its timer, so nothing looked broken — it had simply stopped being reachable
and stopped being able to say it had news. Every deploy of a hosted service
would have done that to everyone. It reconnects now, doubling from a second to
a minute and resetting after a connection that lasted. Verified live: a daemon
left for two minutes with no service reconnected twenty-six seconds after one
appeared.

## The window

Recorded here late: the desktop application was built in this phase and this
document did not say so. It is [`qurb-desktop`](../../crates/desktop/README.md),
a Tauri window that runs the daemon inside itself
([decision 0032](../decisions/0032-the-interface-hosts-the-daemon.md)): home,
files with where each one's contents are, devices, activity, storage with the
allowance, sending by dropping a file on the window, settings, and setting a
device up from nothing — the 24 words shown, three of them confirmed, and never
written down ([decision 0033](../decisions/0033-the-phrase-on-a-screen.md)).
Pairing is a QR code, a typed code or a spoken one, with a countdown. Three
things raise a notification and nothing else does: a file sent to you, one
collected, and a failure.

The applications menu launched `qurb-tray`, the smaller front end, rather than
this window, until 2026-09-27: it opens the window now
([decision 0040](../decisions/0040-the-menu-opens-the-window.md)).

## Files sent to a desktop go to Downloads

**2026-09-25.** A file somebody sends this desktop is saved as an ordinary file
in `Downloads/qurb`, outside the folder, and qurb stops tracking it: it is not
scanned, not counted against the limit, and deleting it is the person tidying
their Downloads. That is
[decision 0037](../decisions/0037-a-file-sent-to-a-desktop-is-an-ordinary-file.md),
which records what was built, how it was verified, and what is not done yet —
notably that a phone cannot send one yet, so phone to desktop is unexercised.

Building it turned up two defects that had nothing to do with Downloads and
were serious anyway.

### A path is an instruction

A version from another device names a path, and the receiving device joins it
onto its folder and writes there — or, for a deletion, deletes there. **Nothing
checked the path.** Not the wire format, which bounds its length and checks it
is UTF-8, and not the engine. A paired device could have sent `../../.bashrc`,
an absolute path, or `.qurb/config`, and this device would have written it, or
deleted it.

Pairing does not make that safe. It proves which device is talking; a device
that is stolen or compromised keeps its pinned identity, and the whole point of
the hostile-peer tests in Phase 2 was that authentication says nothing about
whether a peer is telling the truth. Those tests covered wrong bytes, nonsense
and silence. They did not cover a well-formed message naming somewhere it
should not.

Now refused in two places, with one rule —
[`qurb_sync::is_safe_path`](../../crates/sync/src/path.rs): relative, no empty,
`.` or `..` components, no NUL. The wire refuses a tree containing such a path
as a whole, since a peer that sends one is not a working copy of this software.
The engine refuses it again before writing or deleting, and also refuses
anything its ignore rules exclude, which keeps a peer out of qurb's own store
inside the folder.

Checked by disabling the engine's check and running
`crates/engine/tests/hostile_paths.rs`: every hostile version was written, one
file landed a directory above the test's own temporary directory, and a file
outside the folder was deleted. With the check, all of them are refused.

Backslashes are not refused. On Linux they are ordinary characters in a name;
on Windows they would be separators, and that is one of the things supporting
Windows will mean revisiting.

### Taken once, for good

A delivery is taken once, keyed by content, so that the sender offering it
again changes nothing. The only record of having taken one was the received
file's own row — and once the person deleted the file, that row was a tombstone,
which garbage collection expires after the retention window. The daemon keeps
tombstones for seven days, so a file sent to a desktop and deleted there would
have arrived again once its tombstone expired. (A phone was spared only because
nothing on a phone collects garbage at all — which is its own gap: tombstones
and released chunks there are never reclaimed.) Shown by
`a_deleted_delivery_is_not_offered_again_after_collection` in
`crates/engine/tests/received_files.rs`, which collects with no retention
window and fails without the fix; not waited out for a real week.

A `taken` table, never expired, is the record now; the index migration fills
it from everything an existing device had received. Found because a delivery
saved to Downloads has no row at all, which would have made the same thing
happen on the very next sync.

## A transfer you can watch

**2026-09-25.** A file moving between this device and another now shows on a
Transfers screen: how much of it, how fast, and about how long is left — in
both directions.

The engine reports bytes as they are written, but only for content that
actually crosses from the other device. A rename or a copy of something
already here costs a lookup, and showing it as a transfer would be showing work
that is not happening. The daemon publishes what it is told on the same status
channel that carries "syncing" and "up to date" — live state, never the index,
as [decision 0032](../decisions/0032-the-interface-hosts-the-daemon.md) says it
must be — and at most four times a second, so a fast transfer is not spending
its time describing itself.

**Seen working**, on the development laptop: the real window receiving from a
second device on the same machine, both release builds, stores on tmpfs. A
1.2 GiB file showed its bar filling at 153 to 207 MiB/s with the time left
counting down, and moved to *Finished* the moment it arrived. Those rates are
what the screen displayed over loopback, not a measurement of anything a real
network would do.

Watching it found one thing no test did: *Finished* was drawn only when the
screen opened, so a file that had arrived vanished from *Arriving now* and
appeared nowhere. It is redrawn now whenever something stops arriving.

**Sending is the harder direction**, because a sender never sees a file move.
The other device asks for chunks by hash and never says which file they belong
to, and it never says it has finished — it just stops asking. So the peer
server reports each chunk it serves, and to whom, and the daemon traces the
chunk back to the send it is part of with one indexed query. Chunks of shared
files are not traced: they are synced, not sent, and the receiving device is
the one with something to say about them. A send that has not moved for ten
seconds stops being drawn as moving; the other device's `Got` is what moves it
to *Finished*.

Seen working the same way: the window on the sending device, a second device
collecting a 500 MiB send, the bar reading "phone is collecting it" at
138 MiB/s and the file moving to *Finished — delivered to phone* once it was
collected. Watching it found two more things the tests did not: *Finished*
listed a send as done the moment it was queued, and did not update when the
send was collected. Both fixed.

Not built: cancel, pause and retry.

## Several files, and folders

**2026-09-25.** A send can be any number of files and folders, from the window
or from `qurb send`. A folder goes whole under its own name and arrives as a
folder, on a desktop's Downloads and on a phone alike. What to send is worked
out in one place, `qurb_cli::send`, which reads only the filesystem so the
window can store each file separately rather than hold its lock for a whole
folder.

Three rules came out of writing the tests for it: links inside a folder are not
followed, since they can point anywhere including back up the tree; a qurb store
inside a folder is never sent, since it holds the device's keys; and two things
picked together with one name are both sent, the second as `notes (2).txt`,
because under one name the second would replace the first before either
arrived.

Two older defects were found on the way. `qurb send report.pdf to laptop`, the
short form, opened `report.pdf` as the qurb folder; only the long form had ever
worked. And sending an empty file recorded no *sent* event, so it never
appeared in history.

**Verified** with two devices on the laptop: from the command line, a folder
with a nested subfolder, an empty file and a link, plus two files both called
`notes.txt`, arrived as sent — the link skipped and named, the second
`notes.txt` as `notes (2).txt`, a 2 MiB file byte-identical. From the window, a
folder chosen through the real GTK folder chooser arrived as `Trip/b.txt` and
`Trip/day1/a.txt`. Dragging onto the window was not exercised: nothing here
can synthesise a drag from another application.

## Taking a send back

**2026-09-25.** A send nobody has collected can be cancelled, from the window's
Transfers and Send screens or with `qurb cancel`. It becomes a tombstone in the
recipient's vault, which a recipient never takes as a delivery, so it does not
arrive however long the device was away. Once the device has reported holding
it, cancelling is refused: the file is theirs, as decision 0030 already said.

The window's button is pressed twice — the first press asks, in place, and the
second within five seconds cancels. In the page rather than a dialog, because
the list redraws every second and a half and would otherwise redraw under the
question.

Verified in the window with two devices on the laptop: two files queued, one
cancelled with the two presses, the other device started an hour later — only
the other file arrived, and *Finished* read "taken back before desk collected
it".

Not built: stopping a transfer already moving, in either direction, and pause.
A failed file needs no retry button — the next sync retries it — which the
Transfers screen does not yet say.

## Finding what was sent to you

**2026-09-25.** A received file on the Transfers screen has *Show in folder*,
and Settings has where files sent here go, with *Open that folder*. The folder
opened is looked up from the history entry on the Rust side, never taken from
the page, and only a folder inside the downloads directory is opened — checked
after resolving links, so a link inside Downloads pointing elsewhere does not
count. A changed location is refused in Settings if it overlaps the synced
folder, and a running daemon picks it up on its next pass.

**Verified:** the path check by three unit tests, including a `../` path and a
link out of Downloads. The live pickup with two devices on the laptop: the
location changed with `qurb config` on a running device, and the next file sent
landed in the new place with no restart, the daemon logging "files sent here
now go to". An overlapping location was refused while it ran.

**Not verified:** the Settings screen itself and the two buttons were not
pressed in the window. Screenshots of the window came back blank for this run,
though the page was drawing — the accessibility tree read the setting off the
screen — and filling a text field needs keystrokes sent into the desktop, which
was stopped rather than risk them reaching another window. `xdg-open` was not
run, since it opens a file manager on the screen.

**Measured on the way:** with no rendezvous service, a receiving device already
running took **19 seconds** to collect from a sender that started after it —
two daemons on one laptop, loopback. The sender reached the receiver at once;
the receiver found the sender only on its own next look. Recorded rather than
changed here.

## Directly, or through the relay

**2026-09-25.** The Devices screen says, for each device, whether it is
connected now and how: *connected directly*, or *connected through an
encrypted relay* — named that way because "relay" alone sounds like somebody
else holding your files. The address, the path and the transport are under
*Details*.

A connection records whether it went through the relay when it is made, since
afterwards the relay looks like any other address. The daemon publishes the
connections it actually holds, and drops ones that have ended every five
seconds, rather than only at a sync pass.

That second part was found by watching: the first version published the list
only at sync passes, which can be minutes apart, and a device that had been
switched off was still shown *connected directly* two and a half minutes later.
A device that goes away without a word is now noticed by the connection's
thirty-second idle timeout.

**Verified:** in the window, a second device on the same laptop showed as
*connected directly*. The route is asserted in the end-to-end tests for both
the direct and the relayed case. Clearing it is covered by a daemon test
against a real local peer, which fails with the fix removed. The live
check of clearing it in the window did not complete: screen captures of the
window stopped working partway through the session, and it was not repeated
by simulated clicks. *Details* unfolding was fixed after it was seen to fold
itself again on the screen's five-second redraw, and was not seen again
afterwards.

The relay path in the window is untested: on one machine every direct attempt
succeeds, so nothing here falls back to the relay by itself.

## How much space, asked first

**2026-09-27.** Setting up in the window now asks how much of the disk qurb may
use, between choosing the folder and making the key — the brief's order and
[decision 0038](../decisions/0038-the-storage-question-during-setup.md), which
records what was built, the unit it counts in, and how it was checked: unit
tests and the fixture page in a headless browser, not the real window, which
synthetic input cannot reach under Wayland.

The last setting-up screen also stopped saying devices are introduced "on the
command line for now". The window's Devices screen has done that since pairing
by code was built.

## Through a server of your own

**2026-09-27.** The project owner means to run the rendezvous service and the
relay on a server of his own, so that his phone and laptop sync whenever both
are on, wherever they are, without Tailscale. Getting ready for that found:

- **The phone never used a relay** — the app passed none — so on mobile data,
  behind carrier address translation that a direct connection often cannot
  cross, it reached nothing. Settings now has a Relay row, checked as it is
  saved by the engine's own rule.
- **A relay could only be given as an address**, while the server guide's
  example used a name. Names now work, looked up at each start, trying each
  address until one answers.
- **A relay that was down stopped a device syncing at all**, even with the
  other device on the same network. It is best effort now.
- **The guide said the relay was UDP**; it is TCP. And its examples, and the
  CLI's own help, named `~/Downloads/qurb` as the folder, which is now where
  received files go. Both corrected, with the firewall rules spelled out.
- **The rendezvous service held about 150 KiB per connected device**, in the
  WebSocket library's default buffers. Now about 22 KiB, 30 KiB with its own
  TLS. Measured with a load generator in
  [experiments/service-capacity](../../experiments/service-capacity/README.md),
  which also measured the relay: 241 MiB/s for one transfer, 726 MiB/s for
  eight, at 4–5 CPU-seconds per GiB — over loopback on the laptop.

**And the laptop's own idle cost**, measured the next day: an idle daemon with
its phone offline used 0.02 CPU-seconds in three minutes but woke 2.7 times a
second, most of them for a check of the index every five seconds for new
pairings and sends. The window now tells its daemon when it pairs or sends, the
check is a thirty-second backstop, and the connected-devices display refreshes
only while something is connected: 1.1 wakeups a second, 0.01 CPU-seconds.

**A server to try it on for free**: Oracle Cloud's Always Free tier, with the
caveats in [the server guide](../../packaging/server/README.md#a-free-server-to-try-it-on).
`packaging/server/deploy.sh user@host` sets one up from the laptop and checks it
from outside — written and not yet run against a real server.

The answer to "is it scalable": for one person's devices a small server is
nowhere near any limit; by memory, it would hold tens of thousands of devices,
and the relay's limit is the server's bandwidth and its bill. **Not yet done**:
any of it on a real server across the internet, a phone on mobile data actually
syncing through the relay, and the churn of many phones connecting and leaving.

## Still to do

- **Running the *daemon* as a service** — a user unit, a launch agent, a
  Windows service. The two *server* services have units. On the laptop, the
  window now keeps syncing when closed and starts hidden at login
  ([decision 0040](../decisions/0040-the-menu-opens-the-window.md#closing-is-not-quitting)),
  which covers what a service was wanted for; a daemon with no window at all is
  still `qurb run`.
- **Installers**, and the update mechanism with rollback. `packaging/install.sh`
  puts qurb in one user's applications menu and is not a package.
- **A replica that can free space.** It keeps every payload, because with no
  folder there is nowhere else for the bytes to live — and eviction works by
  deleting a file from a folder, so a cap on a replica reports the overrun
  rather than acting on it. Dropping chunk payloads is a different operation
  and is not written.
- **The rest of the interface.** Recovering a deleted file. Progress, cancel,
  several files per send, "Open folder", the `downloads` setting, the storage
  question during setup and the menu opening the window were on this list and
  are built — see the sections above and decision 0040. See [product-plan.md](../product-plan.md) for the order.

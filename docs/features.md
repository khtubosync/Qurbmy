# What qurb does today

Every feature that exists, grouped by where a person meets it: the desktop
window, the phone, the command line, the services. The last sections cover
what the engine guarantees underneath all of them, and what is not built.
Written 2026-09-28 as the starting point for the design and UX pass — the
thing to decide against, screen by screen.

Each line says how far it has been checked, because "built" and "works between
two real devices" are different claims:

- **✅** — verified between real devices: the Galaxy S23 and the laptop.
- **🧪** — checked by automated tests with several devices, on the Android
  emulator, or in the real desktop window by the smoke test — not yet between
  real devices.
- **◻** — built and compiled, not yet exercised end to end.

For how any of it works, [CODEBASE.md](CODEBASE.md); for why, the decision
records it links.

---

## 1. The desktop window (Linux)

One program, `qurb-desktop`, that *is* the sync daemon with a window on it
([0032](decisions/0032-the-interface-hosts-the-daemon.md)). Closing the window
leaves it syncing; *Quit qurb* in Settings stops it. It starts at login without
a window, and a second launch shows the running one
([0040](decisions/0040-the-menu-opens-the-window.md)).

### Before it runs

| | what a person can do | |
|---|---|---|
| Set up a new device | choose the folder, answer how much disk qurb may use, see the 24 words, type three of them back | 🧪 |
| Join with the 24 words | set this computer up as another of the same person's devices | 🧪 |
| Unlock | type the passphrase, when the key is protected by one; at login the window shows itself to ask ([0046](decisions/0046-the-window-asks-for-the-passphrase.md)) | 🧪 |

### The eight tabs

| tab | what it shows and does | |
|---|---|---|
| **Home** | syncing / up to date / no devices reachable; devices reachable, files, space in use, and how many files are only on this device; recently changed files; what is still on its way to another device; a notice when two devices changed the same file | 🧪 |
| **Files** | the folder, paged, with search by name; for each file whether its bytes are *here*, *elsewhere* or *only here*; fetching a freed file back | 🧪 |
| | **Changed on two devices**: each conflict with both versions, who made each and when; keep this one, the other, or both ([0043](decisions/0043-settling-a-conflict.md)) | 🧪 |
| | **Folders, and which devices have them**: per folder, which devices it is shared with ([0044](decisions/0044-sharing-with-chosen-devices.md)); *free space here* / *keep on this computer* ([0045](decisions/0045-a-folder-kept-remotely.md)) | 🧪 |
| | **Recently deleted**: thirty days; restore — on every device — or delete for good ([0042](decisions/0042-recently-deleted.md)) | 🧪 |
| **Devices** | paired devices, whether each is connected now and whether directly or through the relay, when each was last reached | 🧪 |
| | pairing: **show a code** (QR, typed or read aloud, with a countdown) or **enter one** | 🧪 |
| | removing a device, with what that will and will not do said first ([0041](decisions/0041-removing-a-device.md)) | 🧪 |
| **Activity** | everything this device did, newest first, paged, with the reason for failures ([0031](decisions/0031-what-happened-is-written-down.md)) | 🧪 |
| **Storage** | how much qurb uses and of what; a slider for the most it may use ([0025](decisions/0025-a-storage-cap-that-cannot-lose-data.md)) | 🧪 |
| **Send** | drop files or folders on the window, or choose them; pick one device; it goes to that device only ([0030](decisions/0030-sending-a-file-to-one-device.md)) | 🧪 — sending itself ✅ from the command line |
| **Transfers** | arriving now and waiting to be collected, live, with rate and time left; cancelling a send not yet collected; finished and older | 🧪 |
| **Settings** | see below | |

### Settings

| section | | |
|---|---|---|
| Facts | the folder, this device's identity, how the key is kept, and the version — `qurb 0.1.0 · protocol qurb/2 · index schema 15` ([0047](decisions/0047-versions-and-upgrades.md)) | 🧪 |
| This device | name, the rendezvous service, the relay, the port | 🧪 |
| Files sent to this computer | where they are saved — `Downloads/qurb` by default, as ordinary files qurb stops tracking ([0037](decisions/0037-a-file-sent-to-a-desktop-is-an-ordinary-file.md)); *Open that folder* | 🧪 — nothing has been sent from the phone to the laptop yet |
| Security | this device's fingerprint; how the key is kept, in plain words; protect it with a passphrase, change it, or move it to the system keystore; pairings and removals | 🧪 |
| Recovery phrase | show the 24 words again | 🧪 |
| In the background | start at login; *Quit qurb* | 🧪 |

### Notifications

Three things only: a file sent to you, one of yours collected, one that
failed. 🧪

### The tray icon

`qurb-tray`: the same daemon with an icon — recently synced files, *Open
folder*, *Quit*, and a small window with the status and the storage slider.
Where there is no tray (GNOME), it opens that window instead. Superseded as the
main way in by the desktop window; kept for desktops that want only an icon. 🧪

---

## 2. The Android app

Kotlin over the engine, five tabs. The key is kept in the Android Keystore
([0021](decisions/0021-the-platform-supplies-the-keystore.md)). Files added on
the phone are **private by default** — they stay on the phone and on devices
chosen to keep them — and that can be switched off
([0036](decisions/0036-a-phone-keeps-its-own-files.md)).

### Setting up

| | | |
|---|---|---|
| A new key | the 24 words shown once; three typed back | 🧪 |
| An existing key | the 24 words from another device | 🧪 |

The S23 was set up before the three-word check existed; the flow as it is now
was walked through on the emulator.

### The five tabs

| tab | what it shows and does | |
|---|---|---|
| **Home** | connected to which devices; *Sync now*; files only on this phone; a card when two devices changed the same file — keep this version, the other, or both | ✅ |
| **Vault** | a folder at a time, with Back going up; search across every folder; sort by name, newest or largest | 🧪 |
| | per file: open, save a copy to the phone, send to a device, download, free phone space, rename, move to a folder, delete | 🧪 |
| | *Add files*; new folder; save everything in a folder to the phone at once; Recently deleted | 🧪 |
| **Devices** | paired devices and when each was last reached; *Connect a device* — scan a code, **show a code on this phone**, or type one | ✅ scan · ◻ show |
| | *Keep my files here*: a device that keeps a copy of the phone's own files, so the phone can free space | ✅ |
| | send files to a device; remove a device | 🧪 |
| **Transfers** | files waiting to be collected, with *Stop sending*; what happened, newest first | ✅ history · 🧪 stop sending |
| **Settings** | name; keep new files private or not; the recovery phrase; space used and *free unused space*; Recently deleted; who has each folder — choose devices, keep on this phone, free space; background sync; rendezvous and relay; version | 🧪 |

### Outside the app

| | | |
|---|---|---|
| **Share sheet** | anything on the phone can be shared into qurb, with no network and no other device switched on | ✅ |
| | it asks where: *Save to My Vault*, or *Send to* a paired device | 🧪 |
| **The system file picker and Files app** | qurb's files appear there, listed from the index; a freed file downloads when opened; other apps can save into qurb | 🧪 |
| **Background sync** | WorkManager, every 15 minutes — every hour once a push has arrived in the last week | ✅ |
| **Push** | a change on the laptop wakes the sleeping phone, through Firebase; about five seconds from a change on the laptop to the phone syncing it, on mobile data with the screen off | ✅ |
| **Syncing from mobile data** | through the rendezvous service, directly to the laptop | ✅ |

---

## 3. The command line

`qurb` with no arguments lists these. `[dir]` defaults to the folder already
set up, so most commands need no path.

| command | what it does |
|---|---|
| `init [dir]` | set up a device with a new key and show the 24 words |
| `enrol <dir> "<24 words>"` | set up a device with an existing key |
| `pair` / `join <code>` | show a pairing code (QR in the terminal) / join one |
| `run` | the daemon, without a window |
| `replica <dir> [--only <path>]` | an always-on device that holds content for the others and shows nothing |
| `status` | what this device holds and trusts |
| `ls [path]` / `find <text>` | what the folder holds and where each file's bytes are / search names |
| `activity [path]` | what happened, newest first, or to one file |
| `fetch <path>` / `free <path>` | bring a freed file back / free a local copy another device keeps |
| `send <files and folders> to <device>` / `cancel <name> to <device>` | send to one device / take a send back before it is collected |
| `holders [add\|remove <device>]` | which devices keep this one's own files |
| `conflicts [keep <copy> this\|other\|both]` | list conflicts, settle one |
| `share [<folder> with <device>,… \| with everyone]` | which devices a folder goes to |
| `keep <folder> here\|remote` | keep a folder here, or only list it |
| `deleted` / `restore <#n or path>` | Recently deleted / put one back, everywhere |
| `remove-device <device> [--delete-kept] [--yes]` | stop trusting a device; says what that does first |
| `config [key=value …]` | name, rendezvous, relay, port, limit, own-files, downloads |
| `protect file\|keystore\|passphrase` | change how the key is kept |
| `verify [--deep]` / `reclaim` | check the store against itself / free duplicates an older store holds |
| `version` | the build, its protocol, its index schema |
| `signal` / `relay` / `netcheck` | run the rendezvous service / run the relay / what this network allows |

---

## 4. The services

Both run by the owner, on their own machine or server. Neither ever sees a
file, a filename or who a person is
([0016](decisions/0016-what-signalling-learns.md)).

| service | what it does | where it runs today |
|---|---|---|
| **Rendezvous** (`qurb signal`) | introduces devices on different networks, tells a device at once when another has work for it, keeps that message for a device that is away, and wakes a sleeping phone by push ([0022](decisions/0022-the-service-announces-arrivals.md), [0028](decisions/0028-waking-a-sleeping-device.md)) | on the laptop, as a user unit, reachable from outside through Tailscale Funnel ✅ |
| **Relay** (`qurb relay`) | carries encrypted traffic when no direct path exists, over TCP 443 | built and tested 🧪; **not running anywhere yet** — next after design |
| **Local discovery** | devices on the same Wi-Fi find each other with encrypted beacons and no server at all ([0034](decisions/0034-finding-each-other-with-no-server.md)) | ✅ |

`packaging/server/` has the units, TLS and firewall rules for a server of your
own, and `deploy.sh` for setting one up — written, not yet run against a real
server.

---

## 5. What the engine guarantees, everywhere

The parts nobody sees, and the reason the features above can be trusted.

| guarantee | |
|---|---|
| **Files go directly between devices, encrypted.** The servers never hold them. | ✅ |
| **A file costs its size once.** The file in the folder is its own storage; no second copy ([0024](decisions/0024-the-file-is-the-payload-store.md)). | ✅ |
| **An edit sends only what changed.** A 16-byte insert into a 200 MB file moved 248 KiB, between two devices on one machine. | 🧪 |
| **Nothing is waiting on the other device being awake.** Added while every other device is off, a file goes when one is next reachable. | ✅ |
| **An edit is never silently lost.** Two devices changing one file keep both versions ([0005](decisions/0005-conflict-resolution.md)). | ✅ |
| **A deletion can be undone for thirty days**, on every device. | ✅ |
| **The storage limit never deletes the only copy.** It frees only what another device is known to hold, and says when it cannot. Freeing on the phone, with the laptop keeping its files, is verified. | 🧪 limit · ✅ freeing |
| **A folder shared with chosen devices is refused to the others**, by the device serving it, however they ask — tree, manifest or chunk. | 🧪 |
| **A file sent to one device reaches only that device.** | ✅ |
| **A removed device is refused at once**, on connections already open too. | 🧪 |
| **Hostile input goes nowhere**: `../` paths, qurb's own store, wrong bytes, nonsense and silence from a peer are refused before anything is written. | 🧪 |
| **A crash cannot corrupt the index**: payload before index, killed at seven points in tests; migrations atomic. | 🧪 |
| **An upgrade keeps everything**: the index is copied before it migrates; a newer index is refused by an older build. The S23 upgraded from schema 12 to 15 with its data intact. | ✅ |
| **Filenames are the same everywhere** (NFC), and names that collide ignoring case are refused. | 🧪 |
| **Tested at 100,000 files and 4.4 GiB** between two devices. | 🧪 |

---

## 6. Installing

| | | |
|---|---|---|
| Arch and derivatives | `cd packaging/arch && makepkg -si` — built, not yet installed on the laptop | ◻ |
| Any Linux, one user | `packaging/install.sh` — what the laptop runs | ✅ |
| The rendezvous service | `packaging/install-rendezvous.sh`, with push | ✅ |
| Android | `./scripts/android-app.sh release`, signed with the project's key; the phone itself runs a debug build | ◻ release · ✅ debug |

---

## 7. Not built

Stated plainly so that a design does not assume it:

- **The relay on a server.** Next, after design and UX. Until then a phone on a
  network that blocks a direct path cannot sync.
- **A designed look.** Both apps are plain and functional; design and UX is the
  next piece of work.
- **A formal release** — after the relay.
- **iOS, macOS and Windows.** Linux and Android first.
- **Placeholders on Linux.** A freed file is absent from the folder, not shown
  greyed out ([0025](decisions/0025-a-storage-cap-that-cannot-lose-data.md)).
- **An automatic updater**, deliberately ([0047](decisions/0047-versions-and-upgrades.md)).
- **Key recovery beyond the 24 words.** Lose them and every device, and the
  files are gone — that is the promise, and no escape hatch is decided.
- **A passphrase on the phone** (the Android Keystore and the phone's own lock
  stand in), stopping a send mid-transfer, a replica freeing space, push
  without Firebase, and accounts or billing of any kind.
- **Unmeasured**: battery over a day on the phone, and how often a direct
  connection works across other networks.

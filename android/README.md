# The Android app

A real app: it installs, sets up an identity, keeps the key in the Android
Keystore, lists files, pairs with another device and syncs.

```bash
./scripts/android-app.sh            # debug APK, about 45 MB: two architectures, unshrunk
./scripts/android-app.sh install    # and install it on a connected device
./scripts/android-app.sh release    # unsigned release APK, about 11 MB
```

**The app is meant to be light and snappy**, and the release build is where
that is decided: arm64 only, the code shrunk by R8, and the engine built with
the `mobile` profile — link-time optimisation across every crate. Measured on a
Galaxy S23: 10.7 MB installed against 46.7 MB for the debug build, the code it
keeps in memory down from 26.4 MB to 8.1 MB, and a cold start of about 175 ms.
The measurements and what they do and do not show are in
[decision 0039](../docs/decisions/0039-a-light-android-app.md), which also
measures the five-tab app against the one-list app it replaced: the same
startup, and a few megabytes more memory. A release build takes longer, because
link-time optimisation does; the debug build stays quick.

## What it does

Five places under a tab bar, plus setup, scanning and the share sheet:

| screen | what it is for |
|---|---|
| Home | which devices this phone knows; a card, shown only while it is true, saying how many files exist on this phone and nowhere else and what to do about it; what happened lately. The big button is *Connect a device* until one is connected, and *Sync now* after |
| Vault | every file, and where its bytes are: on this phone and another device, only on this phone, or on another device and not here. Tapping one offers what that allows — open, save a copy, send to a device, free phone space, download, delete |
| Devices | the connected devices, which of them keep this phone's files, a way to send one files, and removing one after saying what that does ([0041](../docs/decisions/0041-removing-a-device.md)) |
| Transfers | what this phone has sent that has not been collected, with a way to stop it, and the history |
| Settings | this phone's name, *Keep new files private*, the recovery phrase shown again, what qurb takes in space and a way to free what nothing needs, background sync, the rendezvous service, the version |
| setup | create an identity, show the 24 words and have three of them typed back; or restore from them |
| scan | the camera, reading the code another device shows when connecting |
| share | anything on the phone, sent into qurb from the system share sheet |

**Files added on the phone are private by default** — decision
[0036](../docs/decisions/0036-a-phone-keeps-its-own-files.md). They go to no
other device until the person chooses one on the Devices screen to keep them,
and that device keeps them where nobody using it sees them. *Keep new files
private* in Settings turns that off, from the next file on; files already here
stay where they are.

**Freeing space is refused for the only copy.** *Free phone space* is offered
only for a file another device is known to hold, and the engine refuses it
anyway when that is not so, so no screen can get it wrong. A freed file stays in
the list, marked as not on this phone, and tapping it asks for it back at the
next sync.

The screens are plain classes holding their views, not Fragments: built the
first time each is shown, kept for the life of the activity, and changing tab
swaps one child view for another. No screen reads anything on the main thread.
Each draws what the index already knows, and the scan for changes made while the
app was closed runs after that and redraws only if it found something.

**It is a share target.** `ACTION_SEND` and `ACTION_SEND_MULTIPLE`, for any
type, and it needs no network to work: the file is written into the folder and
indexed there and then, with every other device switched off. There is no
outbox — "what is waiting to be delivered" is a question asked of the index,
not a list that could drift from it. Home says how many files are held only by
this phone, which is the honest form of "it will get there".

**It can be woken.** When another device has something and this one is asleep,
the rendezvous service pokes it and it syncs immediately — measured at seven
hundred milliseconds from the change. That needs a Firebase project; without
one the phone learns at its next scheduled look, and the SDK is not even linked.
See [decision 0028](../docs/decisions/0028-waking-a-sleeping-device.md).

**Opening a file, and saving a copy to the phone, matter more than they
sound.** The synced directory is this app's private storage, so a file that
arrives from another device and stays there is invisible to everything else on
the phone. Without a way out, a sync product syncs into a hole. Both actions go
through the app's own `DocumentsProvider`, so there is one path out of the store
rather than two implementations of reading it.

**In the system file picker and the Files app, qurb lists what the engine
knows**, not what is on disk: a file freed from this phone is still there, says
it is not on this phone, and is downloaded when it is opened — asked for and
synced while the opening app waits, up to 25 seconds. Another app can save into
qurb through the picker; a name already taken gets `(2)`, and closing the file
starts a scan and a sync. Deleting and renaming from a file manager are still
refused: a deletion becomes a tombstone on every device, and there is no undo.

**The 24 words are checked, as on the desktop** (decision
[0033](../docs/decisions/0033-the-phrase-on-a-screen.md)). After "I have written
them down" the phone asks for three at random positions, drawn again each time
the words are looked at again, and the engine checks them against the key: the
app draws the words once and keeps no copy to compare with. Closed before the
check, the app comes back to it. Settings shows the words again, after saying
what they are. The windows that show them are kept out of screenshots and the
recent-apps view, and the fields that take them tell the keyboard not to learn
what is typed.

Saving streams through a cache file rather than a byte array, because `export`
writes a chunk at a time precisely so a large file never has to fit in the heap
— reading it back into memory at the last step would throw that away.

## Three pieces, kept apart on purpose

**The engine is Rust.** Everything here is a thin layer over
[`qurb-mobile`](../crates/mobile-ffi/), which is a thin layer over the same
engine the desktop daemon runs. No sync logic lives in Kotlin.

**Cargo is not wired into Gradle.** `scripts/android-app.sh` builds the native
libraries, strips them, copies them into `jniLibs`, regenerates the Kotlin
bindings, and *then* runs Gradle. That means a Rust change is an explicit step
rather than something that happens invisibly inside an IDE, and the app builds
from a clean checkout on a machine with no NDK as long as the `.so` files are
already in place.

**`jniLibs/` and `java/uniffi/` are generated and not committed.** Both are
regenerated every build. Committing them would mean a stale binary or stale
bindings could ship without anyone noticing — and bindings that drift from the
library they describe fail at runtime rather than at compile time.

## The keystore

[`AndroidKeyStore.kt`](app/src/main/java/com/qurb/AndroidKeyStore.kt) implements
the `KeyStore` interface that `qurb-mobile` declares but deliberately does not
fill in — see
[decision 0021](../docs/decisions/0021-the-platform-supplies-the-keystore.md).

Two layers, because the Android Keystore holds *keys* and performs operations
with them rather than storing arbitrary bytes:

- a 256-bit AES key lives in the Keystore, generated once, never exported;
- the 32-byte master key is encrypted under it, and the ciphertext goes in
  ordinary SharedPreferences.

Verified on a device. After setup, the vault file is five bytes —

```
$ adb shell run-as com.qurb od -An -c files/qurb/.qurb/master.key
   Q   R   B   K 004
```

— which is the magic and a format byte saying "the platform has it". The key
itself is nowhere in the app's files. Reading the whole data directory yields a
ciphertext and an IV and nothing that decrypts them.

`setUserAuthenticationRequired` is deliberately **not** set: it would demand a
fingerprint every time the key is touched, including during a background sync
when nobody is holding the phone, and sync would simply stop happening.

## Window insets

Android 15 draws apps edge to edge whether they ask or not. Without handling
insets a screen's heading sits *beneath* the status bar — which looks wrong,
and, worse, taps in that strip go to the status bar instead. The bug is
invisible in a screenshot until you try to press something, and it was found
exactly that way, on the first version of the app.

The shell pads the screen area for the status bar and any display cutout, once;
the tab bar pads itself for the gesture bar.

## Permissions

`INTERNET` and `ACCESS_NETWORK_STATE`, to sync; `CHANGE_WIFI_MULTICAST_STATE`,
to hear devices on the same Wi-Fi answer; `CAMERA`, asked for only when
scanning a code to connect a device, and refusable — the code can be typed
instead. Nothing else: no storage permission, because the synced directory is
the app's own private storage, and no location, contacts or anything like them.

`allowBackup` is `false` on purpose. Android's backup would copy the vault to
Google's servers, and the Keystore key wrapping it does **not** travel — so a
restored copy would be an unreadable store that looks like a working one. The
recovery phrase is the supported way to move to a new device.

## Trying it with a computer

There is no hosted rendezvous service yet, so run one:

```bash
qurb signal 0.0.0.0:9000
```

In the app: Settings → **Rendezvous service** → `ws://<that machine's LAN
IP>:9000`. From an emulator use `ws://10.0.2.2:9000`, which is how it reaches
its host.

Then, on the computer, Devices → **Show a code** in the desktop app, or
`qurb pair <dir>`; on the phone, **Connect a device**, and point the camera at
the code.

## Background sync

`SyncWorker` is the half of [decision
0020](../docs/decisions/0020-sync-takes-a-deadline.md) that Rust cannot do.
`syncWithin(seconds)` makes a sync that ends when its window does; this decides
when to ask for a window and what to tell the system when one runs out.

WorkManager rather than an alarm or a foreground service: an alarm does not
survive Doze or a reboot, and a foreground service means a permanent
notification plus — since Android 14 — a declared type that "syncing files" does
not cleanly fit.

Every fifteen minutes, which is not a choice: it is the shortest period
WorkManager accepts, and asking for less silently becomes fifteen anyway. In
practice it is a floor rather than a promise, because Doze batches background
work and an idle phone may go hours between runs.

The three outcomes are mapped deliberately:

| what happened | what the worker says |
|---|---|
| ran out of time | `retry` — ask for another window sooner |
| nothing answered | `retry` — the other device is asleep, which is ordinary |
| keystore locked, no vault | `failure` — retrying will reach the same conclusion |

It records what it did, because a background worker is otherwise invisible:
nobody is watching when it runs, so without a trace there is no way to tell a
sync that works from one that silently stopped — and "silently stopped" is the
failure mode a sync app actually dies of. Settings → **Background sync** shows
it.

A pass that reached a device and has something waiting for it stays open up
to ten seconds, until that device has collected it: every device pulls, and a
phone that finishes its own syncing in a second would otherwise close before
the desktop could dial back. Found when a desktop chosen to keep the phone's
files never received one.

A device that does not answer before the window closes is *unreachable*, not
*out of time*. The difference decides what happens next — out of time is a
retry, with exponential backoff — and it was once got wrong, so that a phone
whose computer was switched off reported "no paired devices" and pushed its
next sync further and further away. See
[decision 0020](../docs/decisions/0020-sync-takes-a-deadline.md#found-on-a-phone).

## Not built

- **No bulk save.** One file at a time; there is no "save everything".
- **No folders in the Vault.** It is one list of paths, in the order the index
  keeps them, with no grouping, search or sorting.
- **No progress while a file moves.** A phone syncs in short windows, mostly in
  the background; Transfers shows what is waiting and what happened, not bytes
  in flight.
- **This phone cannot show a code**, only scan one. Connecting two phones to
  each other needs a computer's code, or typing.
- **No storage question during setup, by design.** Phones have no allowance;
  the question is the desktop's, and is built there
  ([decision 0038](../docs/decisions/0038-the-storage-question-during-setup.md)).
- **Files received before 22 September show as only on this phone** even when
  the device that sent them still has them. Builds before then did not record
  where a received file came from, and a phone cannot learn it afterwards
  without asking; it errs the safe way, never offering to free such a file.
- **Nothing for conflicts.** They arrive as extra files with long names and no
  explanation.
- **Sharing into qurb only adds.** The share sheet puts a file in the Vault; it
  does not offer to send it straight to one device, which the Devices screen
  does.
- **Not signed.** `assembleRelease` produces an unsigned APK.

# qurb-desktop

The desktop application: the same daemon, in a window.

```bash
cargo build --release -p qurb-desktop
./target/release/qurb-desktop ~/Sync
```

With no argument it opens the folder the CLI would, so launching from the
applications menu and running `qurb status` in a terminal address the same one.

## What it is

Eight screens over `qurb_cli::View` and the daemon's status channel, plus setting
a device up in the first place:

| screen | what it answers |
|---|---|
| Home | is it working, how many devices, what moved lately, what is still on its way |
| Files | what is in the folder and **where each file's contents actually are** |
| Files, sharing | which devices each folder is on, and whether this computer keeps it or only lists it, under "Folders, and which devices have them" ([0044](../../docs/decisions/0044-sharing-with-chosen-devices.md), [0045](../../docs/decisions/0045-a-folder-kept-remotely.md)) |
| Files, conflicts | a file two devices changed at once is shown at the top of Files, with both versions and three choices ([0043](../../docs/decisions/0043-settling-a-conflict.md)); Recently deleted is under the list ([0042](../../docs/decisions/0042-recently-deleted.md)) |
| Devices | who is paired, whether each is connected now — directly or through the encrypted relay — pairing with another (show a code or enter one), and removing one, after saying what that does ([0041](../../docs/decisions/0041-removing-a-device.md)) |
| Activity | what this device did — the answer to "why is my file not here?" |
| Storage | what qurb costs on this disk, and the allowance |
| Send | a file to one device, by dropping it on the window or choosing one |
| Transfers | what is arriving now, with a rate and the time left; what is waiting to be collected; what finished |
| Settings | this device's name, how it finds the others, where files sent here go, and the 24 words |

A folder with no device in it opens the setting-up flow instead: make a new
qurb, or add this device to one that exists. Setting a device up is the job of
a screen, so it cannot be a precondition of the screen existing.

The distinction the Files screen exists for is three-way. A file that is here
and also on the phone, and a file that is here and nowhere else in the world,
look identical to anything that only checks whether the bytes are on disk — and
offering to free the second is offering to delete it. So: **here**, **not
here**, **only here**, and only the last is drawn in a colour that asks for
attention.

## How it is put together

Tauri 2, a single stylesheet, and one file of plain JavaScript. No framework and
no build step: the application is a handful of screens of lists and numbers, and a
bundler would be more moving parts than the thing it was moving.

```
src/main.rs       opens the window, and the daemon too if there is a device
src/session.rs    whether there is a device yet, the daemon once there is, and
                  the recovery phrase for the moment between showing it and
                  having it confirmed
src/commands.rs   every question the window may ask, each a thin wrapper over
                  the engine
ui/index.html     the screens, and the setting-up steps
ui/app.css        one stylesheet, both colour schemes from the system
ui/app.js         what to do with an answer
```

Nothing in `ui/` decides anything about syncing. If it looks like it is
deciding something, that is a bug in the layering.

## Two rhythms

The daemon's live state — syncing, devices reachable — is polled every 1.5
seconds, because it changes many times a second and only the latest value is
useful. Lists are fetched when their screen is opened and refreshed on a much
slower beat, because redrawing a list somebody is reading is a cost rather than
a feature. See
[decision 0032](../../docs/decisions/0032-the-interface-hosts-the-daemon.md).

## Looking at the screens without a daemon

[`experiments/desktop-fixtures`](../../experiments/desktop-fixtures) serves this
window's real markup, stylesheet and script against made-up data, so layout can
be worked on without a folder, a paired device or a running daemon.

## Driving the real window

`./scripts/desktop-smoke.sh` runs the real application, commands and engine
included, and drives it through WebKit's WebDriver on a display of its own: it
sets a device up through the window, opens every tab, pairs a second device by
the code the window shows, and sends it a file. It fails if any command the
page calls returns an error — the page keeps the last fifty as
`window.qurbFailures` for that, and for reading from the web inspector.

The fixture page cannot do this, because it answers the commands itself. For
five days, from 2026-09-23, *Show a code* failed in the application every time
("no async runtime found": a synchronous command binding a QUIC endpoint
outside the Tokio runtime) while the fixture showed a working screen. The
project owner found it by using the window; this finds it in a minute.

## The 24 words

The phrase is held in the session rather than in the page: created, fetched
once to be drawn, checked against when three of the words are confirmed, and
dropped the moment that succeeds. The page can therefore drop its copy as soon
as it has drawn the list, and what crosses back is a yes or no rather than a
key. It is never logged, never persisted, and never put in debugging output.

It can be shown again from Settings, derived from the key that is already in
the folder — anybody who can read that folder can read the files, so this
reveals nothing new. See
[decision 0033](../../docs/decisions/0033-the-phrase-on-a-screen.md).

## Pairing

Show a code — a QR to scan, the same code to type, and a spoken form to read
down a telephone — or enter one from another device. The screen counts down to
the code's expiry rather than saying "waiting" under a code that stopped
working five minutes ago.

Two things worth knowing. The QR is drawn black on white whatever colour scheme
the desktop is in, because a scanner finds a code by its finder patterns
against a light ground and a code drawn dark-on-dark is not a code. And the
window binds port zero rather than the configured port, because the daemon in
this process already has that one — see
[decision 0032](../../docs/decisions/0032-the-interface-hosts-the-daemon.md).

## Sending

Drop files or folders on the window, or choose them, then pick a device. They
go to that device and to nowhere else: your other devices never see them, and
they are not added to the synced folder. A folder arrives as a folder. What
cannot be sent is named, with why, and the rest still goes.

Each file is stored separately and the session let go of in between, so sending
a large folder does not stop the rest of the window answering. What to send is
worked out by `qurb_cli::send`, the same code `qurb send` uses.

Dragging is the better gesture and needs no plugin — Tauri reports the drop to
the window, and only the *path* crosses into the page, never the contents. The
two buttons do the same for anybody who cannot drag: one for files and one for a
folder, because no platform's dialog picks both at once.

## Notifications

Three things, and nothing else:

- somebody sent you a file — a thing another person did, on purpose, for you;
- a device collected what you sent it;
- something failed.

Ordinary syncing is silent, and so is pairing, eviction, and every file that
arrives because it was in a shared folder. A sync application that announced
every file it moved would be switched off within a day.

Raised from Rust rather than from the page, because a notification is most
useful exactly when nobody is looking at the window.

## Closing it does not stop it

Closing the window hides it; qurb keeps syncing, because the other devices can
only reach this one while it runs. Opening qurb from the applications menu again
shows the same window: a second launch finds the first through a socket in the
runtime directory (`src/instance.rs`), asks it to show itself and exits.
*Quit qurb* in Settings stops it, and it starts at login with `--hidden`
through an autostart entry (`src/autostart.rs`) that the installer writes and
Settings turns off. See [decision 0040](../../docs/decisions/0040-the-menu-opens-the-window.md#closing-is-not-quitting).

## What it does not do yet

- **No pause, and no stopping a transfer already moving.** A send nobody has
  collected yet can be cancelled — pressed twice, since it cannot be undone.
  One the other device is collecting may still finish, and a file arriving
  cannot be stopped from this end. A failed file is retried at the next sync
  on its own; there is no button for it.
- **No passphrase prompt.** A passphrase-protected key is asked for on the
  terminal the application was launched from. The window cannot ask, because
  opening the key is what decides whether there is anything to show; launched
  from a menu it says so on the first screen.
- **No folder picker.** A text field with `~` expansion and a live description
  of what is already there.
- **Linux only, in practice.** The Rust is portable and Tauri is
  cross-platform; this has never been built or run on Windows or macOS.

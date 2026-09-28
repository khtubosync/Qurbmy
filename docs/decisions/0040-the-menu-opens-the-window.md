# 0040 — The applications menu opens the window

**Status:** Accepted. Amended 2026-09-28: closing the window no longer stops
syncing — see [Closing is not quitting](#closing-is-not-quitting)
**Date:** 2026-09-27

## Decision

The `qurb` entry in the applications menu starts **`qurb-desktop`**, the window,
instead of `qurb-tray`. `packaging/install.sh` installs all three programs —
`qurb`, `qurb-tray` and `qurb-desktop` — and the menu entry names the window.
The tray stays installed and can be started by name, for a desktop that has a
tray and a person who prefers one.

Phase 4 left this open ("choosing one is packaging work, and it is not done");
the product plan put it under packaging, step 10.

## Why the window

It is the application the product brief describes: setting a device up with its
folder, allowance and 24 words; Home, Files, Devices, Activity, Storage, Send,
Transfers and Settings; pairing by code. The tray has a status line and a
storage slider.

And on GNOME — the desktop this was decided on, and the default on the
distributions most people install — the tray is not a tray at all. GNOME ships
no StatusNotifier host, so `qurb-tray` falls back to a small window of its own.
The menu was therefore already opening a window; it was opening the smaller of
the two.

## What it did not change, at first

**Syncing still stopped when the window closed.** Both front ends host the daemon
in their own process ([decision 0032](0032-the-interface-hosts-the-daemon.md)),
so this was already true of the tray's fallback window, and on a desktop with
no tray the window cannot hide instead of closing: a window hidden with no icon
to bring it back is a process nobody can see or stop — the failure phase 4 found
with invisible tray icons. Syncing with no window open is the daemon running as
a user service, which is still to do (phase 4, *Still to do*). Changed the
next day; see below.

## How it was checked

`desktop-file-validate` on the entry, the installer run, and the installed
`Exec` resolved to the installed binary. The installer now always writes that
full path into the entry: it used to do so only when the shell running it
lacked `~/.local/bin`, which says nothing about the PATH the desktop session
launches with. The window itself was not launched from
the menu here, because doing so would put a window over whatever the person at
this machine was using. The first launch from the menu is theirs.

## Reversing it

One line in `packaging/qurb.desktop`.

## Closing is not quitting

2026-09-28. The project owner's phone is to sync with his laptop whenever both
are on, wherever they are — and it can only reach the laptop while qurb runs
there. So closing the window hides it and qurb keeps syncing, and the objection
above is answered rather than ignored:

- **The menu brings it back.** A second launch finds the first through a socket
  in the runtime directory, asks it to show its window, and leaves — 29 ms,
  measured. No plugin: `crates/desktop/src/instance.rs`.
- **It can be quit.** Settings has *Quit qurb*, and the first time the window is
  closed a notification says qurb is still syncing and where to stop it. On
  GNOME there is no tray to quit from, so the button is the way.
- **It starts at login, hidden** — an ordinary freedesktop autostart entry, which
  the installer writes (and `--uninstall` removes) and Settings turns on and
  off. A device not yet set up shows its window anyway, since setting it up is
  what it needs.

Not a user service: the window still hosts the daemon
([decision 0032](0032-the-interface-hosts-the-daemon.md)), so there is still one
process. Checked by starting it hidden against a test folder — running, its
daemon listening — and by a second launch handing over to a stand-in listener
and exiting; the window itself was not made to appear on the screen of the
person at this machine, so showing and hiding it is theirs to see first.


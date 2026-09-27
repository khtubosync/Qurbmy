# 0040 — The applications menu opens the window

**Status:** Accepted
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

## What it does not change

**Syncing still stops when the window closes.** Both front ends host the daemon
in their own process ([decision 0032](0032-the-interface-hosts-the-daemon.md)),
so this was already true of the tray's fallback window, and on a desktop with
no tray the window cannot hide instead of closing: a window hidden with no icon
to bring it back is a process nobody can see or stop — the failure phase 4 found
with invisible tray icons. Syncing with no window open is the daemon running as
a user service, which is still to do (phase 4, *Still to do*).

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

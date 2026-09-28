# 0046 — The window asks for the passphrase, and has a Security section

**Status:** Accepted — amends [0033](0033-the-phrase-on-a-screen.md)'s "no
passphrase prompt in the window"
**Date:** 2026-09-28 (brief §33)

## Decision

**A key protected by a passphrase is unlocked from the window.** When the
folder is set up and its key is wrapped with a passphrase, the window opens on
"qurb is locked": a passphrase field and Unlock. Nothing else answers until it
is open. Started hidden at login, the window shows itself for this, because
nothing syncs until the passphrase is typed. A wrong passphrase says so and
changes nothing; the field is cleared whatever the answer, and the passphrase
goes to the key-derivation function and nowhere else — not kept, not logged.

`qurb run` still asks on its terminal, as it always has.

**Settings has a Security section:**

- this device's identity — the fingerprint its paired devices pinned;
- how the key is kept, in plain words, with what each option does and does not
  protect against — a file only you can read; the system keystore; a
  passphrase;
- changing that: protect with a passphrase (at least eight characters, typed
  twice), change the passphrase, or move to the system keystore. The key is
  untouched — this changes the lock, not what it protects — and the running
  daemon carries on;
- pairings and removals, newest first;
- the recovery phrase, as before, and a pointer to the Devices screen for
  trusted devices and removing one.

## Why 0033 said no, and why that no longer holds

0033: "The window cannot ask, because opening the key is what decides whether
there is anything to show." True when the window refused to open without a
key. It has since learned to open in two situations — nothing set up, and
running — and a third, *locked*, is the same shape: set up, not running, and
the reason known. The page asks for what is missing, the way it asks for 24
words when setting up.

The old behaviour was worse than inconvenient. Launched from the applications
menu or at login, a passphrase-protected qurb could not ask anyone, so it did
not start, and said so in a line on a window nobody had opened.

## What the brief asks that this does not do

- **Security events** are pairings and removals only. A failed unlock is not
  recorded: the history lives in the index, which cannot be opened without the
  key that just failed to open.
- **Re-authentication** before showing the 24 words is not asked. 0033's
  reasoning stands: anyone who can open the window can read every file already.
- **The phone** keeps its key in the Android Keystore, behind the phone's own
  lock; a passphrase there is not built.
- **Forgetting the passphrase** is not recoverable from the window except by
  the 24 words setting the computer up again, which the locked screen says.

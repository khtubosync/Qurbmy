# 0037 — A file sent to a desktop is an ordinary file in Downloads

**Status:** Accepted — not built
**Date:** 2026-09-25

Amends [0023](0023-one-person-per-account.md) on the default location, and
[0030](0030-sending-a-file-to-one-device.md) rule 3 on devices that name a
downloads directory.

## Decision

A device may name a **downloads directory**. When it takes a delivery, it writes
the file there as an ordinary file and stops tracking it. The file is not
indexed as content, not scanned, not counted against the storage allowance and
not evictable, and deleting it there has nothing to do with qurb.

**Desktops name one by default:** `qurb` inside the Downloads folder, with
`XDG_DOWNLOAD_DIR` honoured as in 0023. **Phones do not.** A delivery to a phone
is filed in the phone's own vault, which is the brief's model there (§22).

That path is where 0023 put the synced folder, so the default for a **new**
synced folder moves back to `~/qurb`. Existing folders stay where they are, as
0023 already promised.

Chosen by the project owner on 2026-09-25; see
[product-plan §3.2](../product-plan.md).

## Why

The brief separates two kinds of storage on a desktop (§5): qurb storage, which
the allowance governs, and ordinary files that happen to have arrived through
qurb. Somebody who deletes a received file in their file manager is tidying
their Downloads folder, and qurb has no business treating that as anything.

It also removes, on the desktop, a class of bug found on 2026-09-24: received
files sitting in the synced folder, where every scan has to be taught that they
are not part of the shared area. A file that is not in the folder is never seen
by the scan at all.

## What qurb still remembers

- **That it took the delivery**, keyed by content, so it is not taken twice
  (0030 rule 2). The record survives the file being deleted from Downloads;
  rule 2 counts tombstones for exactly this reason.
- **That the sender was told, and only once it was true.** The file is written
  under a temporary name in the same directory, synced to disk, and renamed into
  place. `Got` is sent after that and not before (brief §51.4).
- **What happened**, in the activity record (0031): arrived, from whom, and
  where it went. That is what "Received from Phone A — Saved to Downloads/qurb —
  Open folder" (§12) is drawn from.

For the sender nothing changes. The desktop's copy is still recorded as private,
a copy the sender cannot ask back. Under
[0036](0036-a-phone-keeps-its-own-files.md), if the desktop also holds the
sender's vault, that is a separate copy with its own record.

## The two directories must never overlap

If the downloads directory were inside the synced folder, or the other way
round, a received file would be scanned into the shared area and advertised to
every device. That is the leak 0030 exists to prevent. An existing install whose
synced folder *is* `~/Downloads/qurb` is already in that position.

So qurb checks at startup, and whenever either setting changes, and refuses an
overlap rather than resolving it quietly. Where the synced folder already sits
at the default downloads path, the downloads directory defaults to
`qurb-received` inside Downloads instead.

## Names

A name that is already taken gets 0030's treatment:
`report.from-<device>-<when>.pdf`. It says who sent it, which a bare `(1)` does
not.

## What this costs

- **A received file on a desktop is no longer in qurb.** qurb does not back it
  up, the storage cap does not free it, and qurb's Files screen does not list
  it. Activity still says it arrived and where it went.
- **Sending it on is an ordinary send** of a file picked from disk, like any
  other.

## Reversing it

Cheap. Files already written to Downloads stay there; turning the setting off
goes back to filing deliveries into the folder.


# 0044 — Sharing a folder with chosen devices

**Status:** Accepted
**Date:** 2026-09-28 — the semantics chosen by the project owner the same day
(product plan §4.6, brief §23)

## Decision

A folder in the shared area can be shared with **chosen devices** instead of
every device:

```text
Family Photos        Available on:  ✓ This computer  ✓ SM-S911B  ☐ Work laptop
```

- **Every chosen device keeps a full, two-way synced copy.** Edits, additions
  and deletions travel between them exactly as the shared area always has.
- **A device not chosen never receives what is in the folder.** Enforced where
  vaults are: the index query that decides what a peer is shown, and the one
  that decides whether a chunk or content hash may be fetched, leave the
  folder out for it. It is not a screen choosing not to draw it.
- **Unticking a device stops new changes reaching it. It keeps what it already
  has.** Nothing reaches into a device to delete its files. What it changes in
  its old copy afterwards goes nowhere: the others do not take changes to the
  folder from it.
- **Every folder starts shared with every device**, which is what the shared
  area always meant. Nothing changes until somebody chooses.
- Where: the Files screen's "Folders, and which devices have them" on the
  desktop, Settings → Shared folders on the phone, `qurb share` on the command
  line.

### The answers the brief asks for

| question | answer |
|---|---|
| Does it create a copy? | Each chosen device has its own full copy, as with anything synced. |
| Do changes propagate? | Between chosen devices, yes, both ways. |
| Does deletion propagate? | Between chosen devices, yes — into each one's Recently deleted ([0042](0042-recently-deleted.md)). |
| Are folders bidirectional? | Yes. No device is the folder's owner. |
| A device offline? | Catches up when it next meets any chosen device, like any sync. |
| Conflicts? | As anywhere ([0043](0043-settling-a-conflict.md)). |
| How is access revoked? | Untick the device: nothing new reaches it, and it keeps what it had. The key is not taken back — see below. |

## How it works

**The rules are files.** Each folder's rule is a small text file,
`.qurb-sharing/<folder>`, listing member device ids, one per line. It is in the
shared area, so it reaches every device by the sync that already exists — no
protocol change, no second channel, every device enforcing the same rules. Each
device derives two index tables from the files (`shares`, `share_members`) and
rebuilds them when the files change, so that "what may this device see" stays a
single SQL query.

**Receiving is checked as well as serving.** A peer's server leaves a folder
out by *its* copy of the rules, which may be behind; and a device left out of a
folder still has its old copy and still offers it. So a device also filters
what a peer offers by its own rules before planning: nothing in a folder shared
without the peer — or without itself — is taken from it.

**A device left out cannot write itself back in.** A change to a folder's rule
is taken only from a device the folder is shared with, so editing the rule file
on a left-out device changes nothing anywhere else. A test does exactly that.

**Rules do not nest,** and the whole shared area cannot have one. One rule per
path keeps "which rule covers this file" a single answer in SQL.

**A rule that cannot be read closes its folder** rather than opening it.

## What this is not

- **Not a cryptographic boundary.** Every device of one person holds the same
  master key ([0012](0012-key-hierarchy-and-recovery.md),
  [0023](0023-one-person-per-account.md)). A folder not shared with a device is
  not *sent* to it; a device that already holds the bytes could decrypt them.
  This is the same honest limit vaults have, and it is stated in the same
  words.
- **Not sharing with other people.** One person's devices; there is no second
  person in the model.
- **Folder names are not private.** Every device receives every rule file, so a
  left-out device can see that a folder called "Family Photos" exists and who
  it is shared with. Not its contents.

## Not done

- **Renaming a shared folder does not carry its rule.** The rule is by path; a
  folder renamed from `Family Photos` to `Family` is, to the rules, a new
  folder shared with everyone. Moving files out of a restricted folder shares
  them. Said here because it is the way this can surprise somebody.
- A left-out device's screens do not yet say "this folder is no longer shared
  with this device" beside its frozen copy.
- The phone chooses devices for a folder from Settings; its Vault does not yet
  browse by folder.

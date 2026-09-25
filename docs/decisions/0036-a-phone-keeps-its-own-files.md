# 0036 — A phone keeps its own files, and another device holds them for it

**Status:** Accepted — not built
**Date:** 2026-09-25

## Decision

A file a device adds itself goes into **that device's own private vault**, not
the shared area. On a phone that means everything added through the app or the
share sheet. It leaves the phone only when the person sends it somewhere.

So that the person can free the phone's copy and still have the file, a device
they choose **holds the vault for its owner**: a copy of its content, kept in
qurb storage, counted against that device's allowance, never shown there, and
served back to the owner and to nobody else. The owner may then drop its local
copy, because a device holding its vault will give it back.

Chosen by the project owner on 2026-09-25, from the three options set out in
[product-plan §3.1](../product-plan.md).

## Why

It is the product brief's model, and the brief specifies it in five places,
including its acceptance test: a photo added on the phone, sent to the desktop,
freed on the phone, still listed as "Available on Desktop", and downloaded
again.

The two alternatives each lose half of that:

- **Shared by default** — today's behaviour — makes every photo on the phone
  appear on the desktop. That is the opposite of a private vault.
- **Private, held nowhere else** means "Free phone space" can never be offered.
  [Decision 0030](0030-sending-a-file-to-one-device.md) forbids counting a copy
  the phone cannot ask back, and without a holder there is no copy it can.

## What holding means, precisely

- **The holder keeps the content the way a sender keeps a delivery**
  ([0030](0030-sending-a-file-to-one-device.md)): chunked in its store, never
  backed by a file in its folder, never materialised, and absent from its
  listings, search and notifications.
- **It is served back only to the owner.** The protocol already lets a device
  see its own vault on any other device
  ([0029](0029-two-areas-shared-and-private.md)); nothing changes about who
  else may ask.
- **It is not released when the owner has it.** A delivery is released once the
  recipient takes it. A held vault is kept until the owner deletes the file, and
  then for the retention window like any other tombstone.
- **On the owner, the holder's copy counts.** It is recorded as a copy the owner
  can ask back — the non-private kind of replica record — so the storage cap and
  "Free local space" may rely on it. This is the one case in which a copy inside
  a vault counts, and it counts only for that vault's owner.
- **The holder's allowance applies, and it never drops a held copy to make
  room.** The owner may already have freed theirs. A holder with no room takes
  nothing more, and the owner's file stays "only on this phone", which cannot be
  freed.

## Privacy, stated rather than implied

All of one person's devices hold the same master key
([0012](0012-key-hierarchy-and-recovery.md),
[0023](0023-one-person-per-account.md)). A holder could decrypt what it holds.
What stops the desktop showing the phone's vault is the protocol and the
desktop's own code, not cryptography.

Making it cryptographic was considered and rejected. It needs a vault key the
holder never has, and the 24 words could not restore that key either — so
losing the phone would also lose the copy the desktop was keeping for it. The
backup would fail in exactly the case it exists for.

So the product says what is true: *"Kept on Desktop for this phone. Desktop
doesn't show it."* Never *"Desktop can't read it."*

## What changes in the engine

1. **Vault rows the owner creates.** Today only a delivery creates one.
2. **A holding grant.** The owner names the devices that may fetch its vault,
   and its server checks the grant on `Tree`, `Manifest` and `Chunk` the way
   0029's checks work. The `Audience` enum gains the case "holds this vault".
3. **A replica record the owner counts** for eviction.
4. **The owner's rename, move and delete reach the holder.** 0029 records that
   vaults do not converge. A held vault has one writer, its owner, so this is
   propagation from one device rather than reconciliation between several.
5. **Collecting must not undo freeing.** The owner keeps a row for every held
   path, freed or not, and a held copy is never offered back to its owner as a
   delivery. Otherwise the next sync would re-download everything the person
   had just freed.

Point 2 is a new request or a new meaning for an old one, so it probably means a
protocol version bump like 0030's. That is settled when it is built.

## What does not change

- **The shared area.** Existing files stay where they are and nothing migrates,
  as in 0029.
- **Sending.** A send into another device's vault is still 0030.
- **What a desktop adds to its own folder** is still the shared area, until
  sharing (brief §23) is decided.

## Settled when it is built, not now

- **One folder, two namespaces.** A phone's folder already holds the shared
  area and its vault side by side, and a name can mean either. Found on
  2026-09-24 and patched by refusing the clash (see
  [phase 5](../phases/phase-5-mobile.md), "A file sent to the phone came straight
  back"). Making the vault a phone's *default* makes clashes ordinary rather
  than rare, so this decision has to give the two areas separate places on
  disk, or an equally structural answer — not rely on the refusal.
- **Which devices hold by default.** The brief's picture is "the desktop". The
  engine has no notion of device kind, so the first version lets the owner
  choose, offering every paired device that has a folder and an allowance.
- **Whether a replica can hold.** It is the natural holder, being always on.
  0030 records that a replica cannot usefully carry a *delivery*; holding is a
  different shape and may fit.
- **What the owner sees while its holder is off.** The file stays "Available on
  Desktop"; asking for it says Desktop is offline (brief §25).

## Reversing it

Moderate before anybody has used it and expensive after. Once phones hold
private files that exist nowhere in the shared area, returning to
shared-by-default means deciding, file by file, whether to publish each one.

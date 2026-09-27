# 0038 — The storage question is asked during setup

**Status:** Accepted — built 2026-09-27
**Date:** 2026-09-25

Amends [0033](0033-the-phrase-on-a-screen.md), which left the storage
allowance out of setup.

## Decision

Setting up a desktop — a new qurb or joining an existing one — asks how much
disk qurb may use, **before** the identity and the recovery phrase, as the
brief's onboarding sequence has it (§71).

The choices are the brief's: **50, 100, 250 or 500 GB, or a custom figure**
(§6). The free space on the disk that will hold qurb's storage is shown beside
them. A preset larger than that free space is shown but cannot be chosen, and
says why.

"No limit" is not offered on this screen. It stays what existing devices have
and what `qurb config <dir> limit=0` sets, and the Storage screen still changes
the allowance at any time.

Phones are not asked. They have no allowance (§16).

Chosen by the project owner on 2026-09-25; see
[product-plan §3.3](../product-plan.md).

## Why 0033's reasoning no longer decides it

0033 argued that somebody who has not yet put a file in the folder cannot
budget for it. That is true of *how much their files will need*, and it is not
the question the brief asks. The brief treats the allowance as a promise made
up front about *this disk*: how much of it qurb may take. A person can answer
that before owning a single file, and the brief makes it step 3 of its
acceptance test.

The question also stops being costly to get wrong, because the answer can be
changed any time. The screen says so.

## What it depends on

The storage cap as it stands ([0025](0025-a-storage-cap-that-cannot-lose-data.md)):
it refuses rather than approximates. A small allowance chosen on day one cannot
cost anybody a file. At worst the device stays over its limit and says why.

## Reversing it

Cheap. It is one screen in the setting-up flow and writes the same setting the
Storage screen does.

## Built

2026-09-27. The window's setting-up flow is now welcome, folder, **storage**,
then the key and its words or the words from another device. The storage step
offers the four figures and *Another amount*, with the free space on the
folder's disk beside them, taken from the look the folder step already had.
A preset larger than that is shown disabled with the reason in its tooltip and
beneath; a custom amount larger than it is refused the same way. The largest
of 100 GB and 50 GB that fits is chosen to begin with, so Continue works at once
for somebody content with a sensible figure.

**The unit.** The brief says GB, and qurb counts in 1024s everywhere else —
`qurb config limit=50G` is 50×2³⁰ bytes, and the Storage screen shows GiB. The
presets are 50×2³⁰ bytes and so on, the step says "GB" throughout rather than
offering GB and answering GiB, and the figure chosen is the figure the Storage
screen shows afterwards, there labelled GiB.

A custom amount is read by `qurb_cli::setup::allowance`: a bare number is
gigabytes, and `75 GB`, `1.5T`, `500 MB` and `2 TiB` all work; nothing, zero and
nonsense are refused. The chosen figure is written by `setup::allow` into the
device's config, the same setting the Storage screen changes, right after the
key is made or installed and before anything syncs.

**How it was checked.** Unit tests for the parser and for where the allowance
is written. The screens against the fixture page
(`experiments/desktop-fixtures/?setup&at=storage`), in a headless browser: the
preselected figure, the two presets too large for the fixture disk's 188 GB,
a custom 300 refused and a custom 75 accepted. Not driven in the real window —
synthetic input does not reach a Tauri window under Wayland — so the first
real setup through it is still to be watched.


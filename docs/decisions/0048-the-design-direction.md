# 0048 — The design direction, and designing in Figma first

**Status:** Accepted — revised the same day when the owner gave the full
direction; the visual values are confirmed at the first checkpoint
**Date:** 2026-09-28

## Decision

The desktop window and the Android app are designed together, in one visual
language, **in Figma before any code**: Figma's MCP server builds the frames in
a file the owner reviews, and the approved design is then implemented. The
owner's direction is [design/direction.md](../design/direction.md), word for
word; how it meets the product, and the table that places every feature on a
screen, is [design/brief.md](../design/brief.md). The parts with consequences
beyond looks:

- **The desktop's eight tabs become a translucent sidebar**: Home, Files,
  Devices, Storage; Private Vault set apart; Settings. Send is Home's primary
  action and a drop target everywhere; Transfers appear as a panel only while
  something moves; the history is reached from Home's *Recent*.
- **The phone's five tabs become four** — Home, Files, Devices, Settings.
  Transfers appear when active; Private Vault is reached from Files.
- **The phone's "Vault" becomes "Private Vault"**, the device's own area, and
  exists on the desktop too — which needs moving a file into and out of it, an
  engine addition that follows the design.
- **One look on both** — glass materials over a quiet environment, Qurb green
  `#2F6B57` fixed rather than Android's wallpaper colours, Inter — qurb's own
  rather than GNOME's.
- **A file-state language of nine plain states**, and no implementation words
  on any screen; *free local space* never looks like deleting.
- **The design draws only what is true.** Where the direction asks for
  something qurb does not do — accounts, versions beyond a conflict, a
  percentage on the phone — the brief records what is drawn instead.

The first answers, given before the direction — a teal accent, an Activity
section, "Files" replacing "Vault" — are kept in the brief, struck through
where the direction replaced them.

## Why

**One language** because the product is one thing on two devices; a person
moving from phone to laptop should find the same words in the same places.
Fewer sections because several of the old ones were one idea split by how it
was built — *Send* and *Transfers* are both about sending, *Activity* is what
Home's *Recent* already begins — and because four tabs fit a phone better than
five. Storage keeps a section on the desktop because freeing space without
losing files is the one idea qurb has that other storage apps do not, and it
needs room to be explained.

**Figma first** because the owner wants to see, adjust and own the design in a
design tool before code is written against it; a mockup in code is harder for
him to change and easier to mistake for a finished screen.

## What it costs

- The Figma MCP server writes to the canvas only through Figma's remote
  server, with its plugin authorised in Claude Code; on the owner's free
  Starter plan, about 200 calls a day and three design files. The work is
  batched to fit.
- Renaming the tabs and moving Transfers changes words the phone's users have
  seen; there are none yet but the owner, so now is the cheap time.
- Glass is expensive to draw live on a phone. The environment behind it is
  still, so most surfaces can be a translucent fill over a pre-blurred
  background, with real blur kept for sheets and dialogs (brief §4).
- Inter is a font the phone must carry; subset, it costs a few hundred
  kilobytes against a 10.7 MB install
  ([0039](0039-a-light-android-app.md)).

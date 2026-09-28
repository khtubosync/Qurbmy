# 0048 — The design direction, and designing in Figma first

**Status:** Accepted — the visual values are confirmed at the brief's first
checkpoint
**Date:** 2026-09-28

## Decision

The desktop window and the Android app are designed together, in one visual
language, **in Figma before any code**: Figma's MCP server builds the frames in
a file the owner reviews, and the approved design is then implemented. The
owner's choices, and the table that places every feature on a screen, are in
[design/brief.md](../design/brief.md). The ones with consequences beyond looks:

- **The desktop's eight tabs become a sidebar of five** — Home, Files,
  Devices, Activity, Settings. Send becomes a header button and a drop target
  on every screen; Transfers joins Activity; Storage joins Home (a summary) and
  Settings (the limit).
- **The phone's five tabs become four** — Home, Files, Devices, Settings.
  Transfers and the history move to an Activity screen reached from Home.
- **"Vault" is renamed "Files"** on the phone, matching the desktop. A private
  file is marked as such in the list instead.
- **One look on both**, qurb's own rather than GNOME's, with a fixed teal
  accent rather than Android's wallpaper colours, and Inter as the typeface.
- **Privacy is stated once**, in setup and in Security, rather than badged on
  every screen.

## Why

**One language** because the product is one thing on two devices; a person
moving from phone to laptop should find the same words in the same places.
Fewer sections because several of the old ones were one idea split by how it
was built — *Send* and *Transfers* are both about sending, *Storage* is a
setting and a number — and because four tabs fit a phone better than five.

**Figma first** because the owner wants to see, adjust and own the design in a
design tool before code is written against it; a mockup in code is harder for
him to change and easier to mistake for a finished screen.

## What it costs

- The Figma MCP server writes to the canvas only through Figma's remote
  server, with its plugin authorised in Claude Code; on the owner's free
  Starter plan, about 200 calls a day and three design files. The work is
  batched to fit.
- Renaming Vault to Files and moving Transfers changes words the phone's users
  have seen; there are none yet but the owner, so now is the cheap time.
- Inter is a font the phone must carry; subset, it costs a few hundred
  kilobytes against a 10.7 MB install
  ([0039](0039-a-light-android-app.md)).

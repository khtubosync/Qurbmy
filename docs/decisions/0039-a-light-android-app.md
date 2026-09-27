# 0039 — A light Android app, on the platform's own views

**Status:** Accepted
**Date:** 2026-09-27

## Decision

The Android app is built to be **light and snappy**, which the project owner
set as the goal when asked which interface toolkit to use. Concretely:

- **The platform's own views**, with ViewBinding and RecyclerView, as the app
  already uses. Not Jetpack Compose.
- **Release builds are shrunk and optimised by R8**, with unused resources
  removed.
- **A release carries arm64 only.** Every phone sold in the last decade is
  arm64; x86_64 stays in debug builds, for the emulator.
- **The engine is built for the phone with its own profile** (`mobile` in
  `Cargo.toml`): link-time optimisation across every crate and one codegen
  unit, symbols stripped. Panics still unwind, so a Rust panic becomes an error
  the app can show rather than a crash.
- **The main thread never waits on the engine**, and a screen shows what the
  index already knows before doing anything slow.

## Measured

Galaxy S23 (SM-S911B), Android 16, 2026-09-27. Each build installed over the
previous one with the same signing key, so the app's data — 21 files, one
paired device — was the same throughout. Cold start is `am start -W` TotalTime,
15 launches, force-stopped between, screen woken before each; a launch behind
the lock screen aborts the run.

| | A: debug, as installed | B: release, as configured before | C: this decision |
|---|---|---|---|
| APK | 46.7 MB | 26.6 MB | **10.7 MB** |
| engine library, arm64 | 9.1 MB | 9.1 MB | **7.7 MB** |
| app code, dex | — | 14.2 MB | **3.0 MB** |
| code in memory (PSS) | 26.4 MB | 16.5 MB | **8.1 MB** |
| Java heap (PSS) | 13.6 MB | 6.3 MB | **5.5 MB** |
| cold start, median | 524 ms ¹ | 177 ms / 200 ms ² | **174 ms / 176 ms** ² |

¹ Seven launches, at the start of the session, before the screen-on check
existed; the phone was believed to be unlocked. Not directly comparable.
² First figure with the app compiled to `verify`, what a sideloaded install
gets; second to `speed-profile`, what it reaches once Android has optimised it.

What that says, plainly:

- **Size and code memory are where the gains are.** The APK is less than a
  quarter of what was installed, and the code the app keeps in memory is less
  than a third.
- **Cold start is already fast, and C does not change it measurably.** B and C
  are within a few milliseconds of each other. The large difference is debug
  against release, not anything in this decision.
- **Total memory is dominated by the screen.** The window's graphics buffers are
  about 45 MB at this phone's resolution when counted, and nothing in the app
  changes that. Totals therefore swung between 34 and 83 MB from one reading to
  the next, which is why the table gives the parts the app controls.

## Measured and rejected

**Checking whether the phone is set up without the engine.** The first call
into the engine loads its native library and the bridge to it, and every launch
makes that call on the main thread before anything is drawn. Replacing it with a
plain file check was measured: medians of 370 against 377 ms and 351 against
363 ms, within the noise. Reverted, since it would have meant a rule the engine
owns written down twice. (Those runs predate the screen-on check, so the
absolute numbers are too high; the comparison was run under the same
conditions for both.)

## Why not Compose

Compose would bring its own UI runtime and compiler plugin into every build,
and on a fresh install its code runs interpreted until Android compiles it —
which is exactly the cold start the goal is about. The app is a handful of
lists, a few forms and a camera preview; the platform's views do all of that,
are already here, and cost nothing extra. The design system the product brief
asks for (§73) is styles, colours and a few custom views, which views handle.

## What else was found

**The phone never collects garbage.** Its screen says 100.7 MB on disk for
30.9 MB of files. Only the desktop daemon runs garbage collection, so on a phone
deleted files' chunks are never reclaimed. Recorded in
[phase 5](../phases/phase-5-mobile.md); the fix is running collection from the
background worker.

## Not done

- **Baseline Profiles.** They would compile the startup path ahead of time. With
  cold start at ~175 ms there is nothing yet to justify another library and a
  benchmark module; revisit if the rebuilt app is slower.
- **A signed release.** The APKs measured here were signed with the debug key.
  Release signing is packaging work (brief §69).
- **The camera and barcode-scanning libraries** are used only to scan a pairing
  code. What they add to the APK was not measured, and whether a lighter
  scanner is worth it is left for the rebuild.

## Reversing it

Cheap for the build settings, which are a few lines each. Moving to Compose
later would be a rewrite of the screens, which is the cost this decision avoids.

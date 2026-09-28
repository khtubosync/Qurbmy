# The design brief

What the design pass must produce, and every choice the project owner made for
it on 2026-09-28. The design is made **in Figma first**, through Figma's MCP
server, reviewed there, and only then coded
([decision 0048](../decisions/0048-the-design-direction.md)). Anyone — or any
session — picking up the design work starts here.

What the design covers is fixed by what exists: [features.md](../features.md)
lists every feature, and every one needs a place on a screen. Nothing that is
not built gets drawn as if it were (no thumbnails, no accounts, no iOS).

---

## 1. Choices made

| question | answer |
|---|---|
| Surfaces | the desktop window **and** the Android app, in one visual language |
| Feel | **calm and quiet**: space, soft neutrals, one accent, nothing shouting |
| Theme | **light and dark**, following the system |
| Accent | **deep teal-green**, fixed — no Material You wallpaper colours |
| Type | **Inter** |
| Desktop look | **qurb's own**, the same as the phone; window controls stay native |
| Desktop navigation | a **left sidebar with five sections**: Home, Files, Devices, Activity, Settings |
| Android navigation | **four bottom tabs**: Home, Files, Devices, Settings |
| Naming | **"Files"** on both — the phone's "Vault" is renamed |
| Home | **the sync status leads**; below it, only what needs attention and a short activity list, with *See all* |
| Files | a **list with file-type icons**; where the bytes are shown by a small icon |
| Privacy | **quiet, always true**: explained once in setup and in Security, not badged everywhere |
| Icons | **outline, rounded** (Lucide-style); filled only for the selected nav item |
| Logo | **a new mark**, two or three options, plus a lowercase `qurb` wordmark, an Android adaptive icon and a Linux app icon |
| Recovery phrase | **calm but firm**: plain words, one clear warning, a numbered grid, the three-word check |
| Shape | **soft and roomy**: 12–16 px corners, generous spacing, 44–48 px rows |
| Empty states | **an outline icon and one line**, with the action to take |
| First pass | all four groups: setup; Home and Activity; Files, conflicts, deleted; Devices, pairing, Settings |
| Review | **three checkpoints**, each approved in Figma before the next |
| Figma | a **new file** the design session creates; the owner's account is on the **free Starter plan** |

## 2. Where everything goes

The eight desktop tabs and five phone tabs become one shape. Nothing is lost;
this table is the check.

| feature | desktop | Android |
|---|---|---|
| sync status, devices reachable, space | Home | Home |
| conflicts to settle, files only here | Home, *needs attention* | Home, *needs attention* |
| recent activity, transfers in progress | Home (short) → **Activity** | Home (short) → Activity, a screen pushed from Home |
| full history, waiting to be collected, cancelling a send | **Activity** | Activity |
| browsing, search, sort, where the bytes are | **Files** | **Files** |
| file actions: open, fetch, free, rename, move, delete, send, save a copy | Files, row menu | Files, row sheet |
| settling a conflict | a sheet from Home or the top of Files | the same |
| Recently deleted | Files, at the foot and in its menu | Files menu |
| a folder's devices; keep here / free space | Files, folder menu → *Folder options*; Settings → Folders | the same |
| sending | a **Send** button in the header on every screen, and dropping files anywhere | a file's *Send to…*; a device's *Send files*; the share sheet |
| paired devices, direct or relay, last seen | **Devices** | **Devices** |
| pairing: show a code, enter a code, scan | Devices → *Add a device* | Devices → *Connect a device* |
| who keeps the phone's own files | — | Devices, per device |
| removing a device | Devices, per device | the same |
| name, rendezvous, relay, port | Settings → This device, Connection | Settings → This phone, Connection |
| storage limit (slider) | Settings → Storage; a summary on Home | — (phones have no limit) |
| space used, free unused space | Settings → Storage | Settings → Space |
| where received files go | Settings → Received files | — |
| keep new files private | — | Settings → Privacy |
| key protection, fingerprint, pairings and removals | Settings → Security | — (Keystore) |
| the 24 words again | Settings → Security | Settings → Security |
| start at login, Quit | Settings → Background | — |
| background sync | — | Settings → Background |
| version | Settings → About | Settings → About |
| setup: create, join, storage question, phrase, check | first run | first run (no storage question) |
| unlock with a passphrase | first screen when locked | — |
| notifications | three kinds, native | — |

## 3. Starting values for the foundations

A proposal for checkpoint 1, to be adjusted there — not yet decided.

**Colour** (roles, light / dark):

| role | light | dark |
|---|---|---|
| background | `#FFFFFF` | `#0E1413` |
| surface (cards, sidebar) | `#F6F8F7` | `#161D1C` |
| line | `#E3E8E6` | `#26302E` |
| text | `#111816` | `#E6EDEB` |
| text, quiet | `#5E6B68` | `#93A39F` |
| accent | `#0F766E` | `#2DD4BF` |
| accent, soft (selected, tints) | `#E6F4F2` | `#12302D` |
| waiting | `#B45309` | `#F5B454` |
| problem | `#B42318` | `#F97066` |

Every text colour at least WCAG AA against the background it sits on.

**Type**: Inter, tabular figures for sizes and times. Display 28/34
semibold · Title 20/28 semibold · Heading 16/24 semibold · Body 15/22 (14/20 on
the desktop) · Caption 13/18. Codes and fingerprints in the platform's
monospace, so the phone carries no second font.

**Space**: a 4-point scale — 4, 8, 12, 16, 24, 32, 48. **Corners**: 8 for
small controls, 12 for cards and fields, 16 for sheets and dialogs, full for
pills. **Rows**: 48 on the phone, 44 on the desktop. Touch targets at least
48 dp.

**Frames**: the desktop window at 1200 × 800 (and its minimum, 900 × 600); the
phone at **360 × 780** — the Galaxy S23's size in dp.

## 4. How the work goes

**Checkpoint 1 — foundations.** Logo options, the colour roles in both themes,
type, spacing, corners, the icon set. Approved before any component exists.

**Checkpoint 2 — components.** Buttons; list rows (a file with where its bytes
are; a device with its connection; an activity entry); the status block;
attention cards; fields; sheets and dialogs; the sidebar and the bottom bar;
progress; toasts; empty states. Built as Figma components with variables, so a
change reaches every screen.

**Checkpoint 3 — screens**, both platforms: the four groups in §1, every
screen in light, and Home, Files and the recovery phrase in dark too. Flows
connected where it helps review.

**The Figma file**: *qurb — design*, pages Cover, Foundations, Logo,
Components, Desktop, Android, Flows. On the Starter plan a person has three
design files and about 200 MCP calls a day, so each call builds a whole
component set or screen rather than one element.

**Then code.** The approved tokens become CSS variables in
`crates/desktop/ui/app.css` and resources in `android/app/src/main/res`; the
screens are rebuilt to match, screen by screen, keeping the app light (Inter
subset to the characters used; no new UI framework on either side).

## 5. Rules

- Every feature in [features.md](../features.md) has a place; a screen that
  hides one is wrong.
- Nothing unbuilt is drawn: file-type icons, not thumbnails.
- Words follow how the apps already speak — plain, specific, no exclamation
  marks (`android/.../Words.kt` is the reference).
- An *only here* file must never look the same as one that is safe elsewhere.
- Error, empty and offline states are designed, not left to the code.

# The design brief

How the design pass turns the owner's direction into qurb's screens: what was
decided, how the direction meets what qurb actually does, and how the work in
Figma is ordered. The design is made **in Figma first**, reviewed there, and
only then coded ([decision 0048](../decisions/0048-the-design-direction.md)).
Anyone — or any session — picking up the design starts here.

Two sources, in this order of authority:

1. **[direction.md](direction.md)** — the owner's design direction, 57
   sections, word for word. It decides the look, the materials, the motion, the
   words and the structure. Every section is checked, not approximated.
2. **This file** — where the direction meets qurb: the choices the owner made
   when asked (§1), how each place the direction and the product disagree was
   resolved (§2), and where every existing feature lives (§3).

[features.md](../features.md) is the list of what exists; every feature must
have a place on a screen.

---

## 1. Choices made

Asked on 2026-09-28, before and after the direction was given. Where the two
disagree, the direction won; the earlier answer is struck through.

| question | answer |
|---|---|
| Surfaces | the desktop window **and** the Android app, one language, not one layout (direction §50) |
| Feel | calm and quiet — and, per the direction, *private infrastructure wrapped in glass* |
| Accent | ~~deep teal~~ → **Qurb green `#2F6B57`** (direction §8); fixed, no Material You |
| Theme | light first; **dark after light is approved**, as a Dark mode on the colour variables |
| Type | **Inter** |
| Desktop look | qurb's own, not GNOME's; window controls stay native |
| Desktop navigation | ~~Home, Files, Devices, Activity, Settings~~ → **Home, Files, Devices, Storage; Private Vault set apart; Settings** in a translucent sidebar (§24) |
| Android navigation | **Home, Files, Devices, Settings** (§25); Transfers appear when active; Private Vault reached from Files |
| Naming | **Files** on both; the phone's "Vault" becomes **Private Vault**, the device's own area |
| Home | the state leads, one primary action (*Send to device*), attention only when needed, a subtle Recent (§5) |
| Files | a list with file-type icons and a subtle availability state (§18) |
| Privacy | stated quietly — Private Vault and Security say it; nothing is badged everywhere |
| Icons | Lucide, 1.75–2 px, the set in §10 |
| Logo | **a new mark**, two or three options, and the wordmark; Android adaptive icon and Linux app icon |
| Recovery phrase | calm but firm, intentionally sparse (§47) |
| Corners | ~~12–16~~ → 8 / 12–16 / 18–20, sheets 20–24 (§45) |
| Empty states | an outline icon and one line, with the action |
| Review | checkpoints: foundations → components → screens, each approved in Figma |
| Figma | a new file; the owner's account is on the **free Starter plan** |

## 2. Where the direction and qurb disagree, and what was decided

The direction describes the product it wants; a few things it shows are not
what qurb does, or would say something untrue. Each is resolved here, so the
design never draws a promise the software cannot keep.

| the direction asks | qurb today | decided |
|---|---|---|
| Settings → **Account** (§23, §49) | there are no accounts: no sign-in, only the 24 words and each device's key | **This device** — its name, how its key is protected, its identity |
| **Private Vault** on the desktop (§15, §24) | private files exist, but nothing moves a file into or out of the vault; the phone simply saves its new files there | **designed fully on both**, with *Move to Private Vault* and *Move to shared*; the move is built after the design (a small engine addition) |
| "Files here are only accessible **from this device**" (§15) | a device chosen to keep the phone's files holds an encrypted copy, and all your devices share one key | copy that stays true: *"Only this device can see these. A device you choose can keep a backup."* |
| a percentage while sending, on the phone (§21) | the phone sends in short background windows; only the desktop measures progress | the phone shows *Sending to Laptop…* / *Waiting for Laptop*; the desktop shows the percentage |
| a file's **versions** (§19) | only the two versions of a conflict are kept | versions appear only for conflicts |
| **version comparison** (§49) | a conflict knows each version's device, time and size | compare by device, time and size, with a preview for images and text |
| Settings → **Notifications**, **Appearance** (§23) | the desktop raises three kinds of notification; there is no theme setting | designed; both are small app-side additions after the design |
| **devices holding the file** (§19) | the index records which device holds which content (`replicas`) | designed; one query to add |
| **space that can safely be freed** (§20) | availability is known per file; nothing sums it | designed; one query to add |
| "148 GB used · **4 devices connected**" (§5) | both known | as written |

**Words never on a screen** (§11, §52): replica, materialised, node, peer,
chunk, eviction, tombstone, scope, rendezvous and relay outside *Advanced*.
The file states are the direction's nine: *On this device, Available
elsewhere, Only copy here, Downloading, Sending, Synced, Conflict, Restoring,
Recently deleted* — and, from the vault, *Private*.

## 3. Where everything goes

The direction's structure (§24, §25, §49) with every feature from
[features.md](../features.md) placed. A feature missing from this table is a
bug in the design.

| feature | desktop | Android |
|---|---|---|
| is everything okay: synced, syncing, offline, needs attention | **Home** | **Home** |
| conflicts to settle, *only copy here* files | Home, as attention (§14) | Home, as attention |
| send to a device | Home's primary action; a Send button; dropping files anywhere | Home's primary action; a file's *Send to…*; the share sheet |
| transfers in progress and finished; cancelling a send not yet collected | a Transfers panel that appears when active (§21) | a Transfers bar that appears when active, opening a sheet |
| history of what happened | Home → *Recent* → *See all* | the same |
| browsing, search, breadcrumbs, folders apart from files | **Files** (§18) | **Files** |
| file details: type, size, location, availability, devices holding it, modified, conflict versions, transfer state, actions | an elevated panel (§19) | a sheet |
| open, keep on this device, free local space, rename, move, delete, send, save a copy, move to/from Private Vault | File details and the row's menu | the sheet |
| settling a conflict: keep this, keep the other, keep both | Conflicts, from Home or the file (§14) | the same |
| Recently deleted: restore, delete for good, expiry | Files → Recently deleted (§22) | the same |
| a folder shared with chosen devices; a folder kept only elsewhere | the folder's details | the folder's sheet |
| paired devices, whether each is reachable, last seen | **Devices** (§16) | **Devices** |
| adding a device: show a code, enter a code, scan | Devices → *Add a device* | Devices → *Add a device* |
| a device that keeps this device's private files | Devices → the device | the same |
| removing a device, saying what it does first | Devices → the device | the same |
| local storage used and available, what can safely be freed, the files that can | **Storage** (§20) | Settings → Storage |
| the storage limit | Storage | — (phones have none) |
| Private Vault: its files, and what "private" means | **Private Vault**, set apart in the sidebar | Files → Private Vault |
| keep new files private | Settings → Privacy | Settings → Privacy |
| name, key protection, identity | Settings → This device | Settings → This device |
| the 24 words again | Settings → Recovery | Settings → Recovery |
| where received files go | Settings → Storage | — |
| notifications | Settings → Notifications | Settings → Notifications |
| appearance (light, dark, system) | Settings → Appearance | Settings → Appearance |
| rendezvous, relay, port, background sync, start at login, the activity log in full, version | Settings → Advanced | Settings → Advanced |
| Quit | Settings, and the tray | — |
| setup: welcome, create or join, recovery phrase, verification, device setup (folder, storage) | **Onboarding** (§47) | Onboarding (no storage question) |
| unlock with a passphrase | a locked screen before Home | — |

## 4. Building it so it can be coded

The direction's glass is drawn with **background blur, a translucent fill, a
light edge and a soft shadow** — the same four things CSS
`backdrop-filter` and Android draw — rather than Figma's refractive *Glass*
effect, which neither can reproduce. What is approved should be buildable.

The environment behind the glass is a still, soft gradient. So on the phone,
where live blur behind a view is expensive, a surface over it can be drawn as
a translucent fill over a pre-blurred background and look the same — which
keeps the app light ([0039](../decisions/0039-a-light-android-app.md)). Real
blur is kept for sheets and dialogs, where Android blurs behind a window.

Motion (§27–§42) is specified in Figma as start and end frames with their
timing and easing written beside them, and as prototype transitions where
Figma can show them; the flowing light is drawn as its key frames.

## 5. The work in Figma

The direction's twenty phases (§56), grouped into the three checkpoints the
owner asked for:

| checkpoint | direction phases | what the owner approves |
|---|---|---|
| **1. Foundations** | 1 visual foundation · 2 glass materials · 3 lighting and environment · 4 type and icons · 5 motion principles — plus the logo | colours, the three glass materials on the environment, type, icons, the motion sheet, the logo options |
| **2. Components** | 6 core components · 7 navigation | every component in §48 with its states, the sidebar and the bottom bar |
| **3. Screens** | 8 Home · 9 Files · 10 File details · 11 Devices · 12 Send and transfers · 13 Conflicts · 14 Storage · 15 Recently deleted · 16 Private Vault · 17 Settings · 18 Onboarding · 19 edge states · 20 responsive refinement | each screen on both platforms, reviewed against §54 |

**The file**: *Qurb — Design*, pages Cover, Foundations, Logo, Components,
Desktop, Android, Motion. Colours, spacing and radii are variables (a Light
mode now, Dark added later); type and shadows are styles; components bind to
them, so a change reaches every screen. On the Starter plan a person has three
design files and about 200 MCP calls a day, so each call builds a whole set or
screen.

Frames: the desktop window at 1280 × 820; the phone at 360 × 780, the Galaxy
S23's size in dp.

**Then code.** The approved variables become CSS custom properties in
`crates/desktop/ui/app.css` and resources in `android/app/src/main/res`; the
screens are rebuilt one at a time, with no new UI framework on either side.

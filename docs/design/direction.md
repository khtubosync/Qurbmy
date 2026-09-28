# The design direction

Given by the project owner on 2026-09-28, recorded here word for word. It is
the specification the design is checked against, section by section. Where it
asks for something qurb does not do, or would make a claim that is not true,
the resolution is in [brief.md](brief.md) §2 — this text is not edited to
match.

---

You are the lead product designer, UX architect, visual designer, interaction designer, and motion designer for Qurb.

Your responsibility is to design Qurb as a complete, coherent, premium product.

Do not design isolated screens.

Do not create a generic cloud-storage interface.

Do not blindly copy Dropbox, Google Drive, OneDrive, iCloud, or any other existing product.

Qurb has its own mental model, and the design must make that model feel simple, beautiful, intuitive, and trustworthy.

============================================================
1. PRODUCT VISION
============================================================

Qurb is a private, device-aware file storage and transfer system.

Qurb is NOT a traditional cloud drive.

A user's files exist within a shared Qurb namespace, while the actual bytes may physically exist on one or more devices.

Each device understands:
- which files it currently stores locally
- which files exist elsewhere in Qurb
- which files can safely be removed locally
- which files are transferring
- which files are the only known copy
- which files are private to that device

The underlying system may be technically sophisticated.

The user experience must NOT feel technically sophisticated.

The user's mental model should simply be:

"My files are here."

"They are available across my devices."

"This file isn't stored on this device right now."

"I can keep it here."

"I can free space without losing the file."

"This file has another version."

"This device is receiving the file."

"This area is private to this device."

The product should hide implementation complexity while accurately representing what the system is doing.

============================================================
2. DESIGN NORTH STAR
============================================================

The fundamental design principle is:

DO NOT DESIGN THE COMPLEXITY OF QURB.

DESIGN THE SIMPLE HUMAN EXPERIENCE THAT THE COMPLEXITY ENABLES.

The interface should feel:

- gorgeous
- premium
- calm
- fluid
- private
- spatial
- trustworthy
- modern
- tactile
- extremely intuitive

The product should feel like sophisticated private infrastructure wrapped in an exceptionally polished consumer interface.

The user should never feel like they are operating a distributed storage system.

============================================================
3. VISUAL DIRECTION
============================================================

Use an Apple-inspired spatial glass interface.

IMPORTANT:

Take inspiration from Apple's principles of:
- material depth
- translucency
- spatial hierarchy
- fluid motion
- physicality
- restraint
- beautiful typography
- subtle lighting
- contextual surfaces

Do NOT simply copy Apple's UI.

Do NOT make Qurb look like a visionOS clone.

Do NOT turn the interface into generic glassmorphism.

Glass is a MATERIAL, not decoration.

Qurb should feel like:

"Private infrastructure, wrapped in glass."

The visual language should combine:

- translucent surfaces
- warm luminous backgrounds
- subtle depth
- soft reflections
- restrained color
- strong typography
- fluid motion
- spatial transitions
- extremely clear information hierarchy

============================================================
4. CORE UX PRINCIPLE
============================================================

Every screen must answer:

1. What is happening?
2. Does the user need to do anything?
3. What can the user do next?

Use progressive disclosure.

Do not display information simply because the system knows it.

Information should appear because it helps the user make a decision.

Prefer:

ONE PRIMARY QUESTION
ONE PRIMARY ACTION
MINIMAL SECONDARY INFORMATION

over:

MANY CARDS
MANY METRICS
MANY STATUS INDICATORS
MANY ACTIONS

The interface should feel simple even when the underlying system is complex.

============================================================
5. HOME SCREEN
============================================================

Home exists primarily to answer:

"Is my Qurb space okay?"

The healthy default state should be:

"Everything is synced."

"Your files are safe."

"Nothing needs your attention."

Home must NOT feel like a dashboard.

Do not create:
- large metric dashboards
- storage graphs
- network statistics
- device-health panels
- synchronization statistics
- technical system-status panels
- excessive cards

The primary Home experience should contain:

1. Overall state
2. One primary action
3. Attention only when necessary
4. A small amount of recent/contextual information
5. Very limited secondary information

Example:

Everything is synced.

Your files are safe. Nothing needs your attention.

[ Send to device ]

Then a subtle Recent section.

Secondary information such as:

148 GB used
4 devices connected

may exist, but must remain visually subordinate.

A healthy Home screen should feel almost boring.

That is intentional.

The visual beauty comes from material, depth, typography, lighting, and motion, not information overload.

============================================================
6. QURB GLASS SYSTEM
============================================================

Create a consistent glass-material hierarchy.

There should be three primary glass materials.

------------------------------------------------------------
GLASS 01 — CLEAR
------------------------------------------------------------

Use for:
- navigation
- floating controls
- top bars
- lightweight overlays

Characteristics:
- high transparency
- low-to-medium blur
- subtle border
- minimal shadow
- background remains visible

Conceptually:

background:
rgba(255,255,255,0.25–0.45)

backdrop blur:
approximately 16–24px

border:
very subtle translucent white

------------------------------------------------------------
GLASS 02 — FROSTED
------------------------------------------------------------

Use for:
- primary cards
- file groups
- device cards
- important content surfaces

Conceptually:

background:
rgba(255,255,255,0.45–0.65)

backdrop blur:
approximately 24–40px

border:
rgba(255,255,255,0.50–0.70)

shadow:
soft and diffused

This should feel like translucent material floating above the environment.

------------------------------------------------------------
GLASS 03 — ELEVATED
------------------------------------------------------------

Use only for:
- dialogs
- bottom sheets
- contextual menus
- transfer panels
- important confirmation surfaces

Characteristics:
- higher opacity
- stronger blur
- stronger edge highlight
- deeper but soft shadow
- clear separation from the environment

Do not use the strongest glass material everywhere.

Material hierarchy must remain obvious.

============================================================
7. BACKGROUND ENVIRONMENT
============================================================

Do not use a completely flat background.

Create a subtle environmental layer behind the glass.

The environment should use:

- warm off-white
- very subtle green
- extremely subtle cool neutral tones
- large soft gradients
- extremely low saturation
- large blur radius

The background should feel like light passing through a physical environment.

Avoid obvious colorful blobs.

Avoid neon gradients.

Avoid strong purple/blue AI-startup gradients.

The environment should be almost still in its default state.

Motion should become more noticeable during meaningful system activity.

============================================================
8. COLOR SYSTEM
============================================================

Primary background:

#F7F7F4

Primary surface:

#FFFFFF

Secondary surface:

#F0F0EC

Primary text:

#171816

Secondary text:

#5F625C

Tertiary text:

#8A8D86

Border:

#E2E3DD

Strong border:

#D2D4CD

Primary Qurb green:

#2F6B57

Qurb green hover:

#285B4A

Qurb green pressed:

#214B3D

Qurb green soft background:

#E8F0EC

Use a softer luminous green for environmental glow:

approximately #6FAF98

but only at very low opacity.

The green is an identity and interaction color.

It must NOT dominate every screen.

------------------------------------------------------------
SEMANTIC COLORS
------------------------------------------------------------

Healthy:

Foreground:
#287052

Background:
#E8F3EC

Attention:

Foreground:
#946A24

Background:
#F7F0DD

Error:

Foreground:
#A3423A

Background:
#F7E9E7

Neutral / Available elsewhere:

Foreground:
#626A68

Background:
#ECEEEC

Do not rely on color alone to communicate state.

============================================================
9. TYPOGRAPHY
============================================================

Use Inter or a very similar modern sans-serif.

Typography must be:

- clean
- highly readable
- confident
- restrained
- spacious

Desktop:

Page title:
28–32px / Semibold

Section title:
18–20px / Semibold

Body:
14–16px

Secondary:
13–14px

Metadata:
12–13px

Mobile:

Page title:
24–28px

Section:
17–18px

Body:
15–16px

Metadata:
12–13px

Do not make text unnecessarily small in the name of minimalism.

Minimalism means reducing unnecessary information, not reducing readability.

============================================================
10. ICONOGRAPHY
============================================================

Use a consistent Lucide-style icon system.

Characteristics:

- simple
- geometric
- elegant
- rounded
- approximately 1.75–2px stroke
- consistent optical weight

Do not mix icon families.

Do not use:
- cartoon icons
- 3D icons
- colorful icon illustrations
- random filled icons
- decorative iconography

Core icons:

Home:
House

Files:
Folder / Files

Devices:
MonitorSmartphone

Storage:
HardDrive

Settings:
Settings

Send:
Send

Transfers:
ArrowUpDown

Private Vault:
LockKeyhole

Search:
Search

Back:
ArrowLeft

More:
Ellipsis

Keep on device:
Download

Free local space:
HardDrive / CloudOff style metaphor

Synced:
Check

Available elsewhere:
Cloud

Conflict:
GitCompare

Recently deleted:
Trash / Clock

Restore:
RotateCcw

Attention:
TriangleAlert

Error:
CircleAlert

Icons support text.

They do not replace understandable labels.

============================================================
11. FILE STATE LANGUAGE
============================================================

Qurb must have a reusable file-state system.

Supported human-readable states:

ON THIS DEVICE

AVAILABLE ELSEWHERE

ONLY COPY HERE

DOWNLOADING

SENDING

SYNCED

CONFLICT

RESTORING

RECENTLY DELETED

Do not expose implementation terminology.

Never use:
- replica
- materialized
- node
- peer
- chunk
- eviction
- remote object
- distributed state

unless explicitly inside an advanced technical/debug context.

============================================================
12. STORAGE MODEL
============================================================

Qurb can free local device storage without deleting the file from Qurb.

This distinction is fundamental.

Never present:

"Free local space"

as:

"Delete file"

The user must understand:

"The file stays in Qurb.
This device simply stops keeping its local bytes."

When a user frees local space:

THE FILE REMAINS VISIBLE.

Only its local availability changes.

This distinction should be communicated visually through animation whenever appropriate.

============================================================
13. ONLY-COPY PROTECTION
============================================================

Qurb protects the only known copy.

If an action would remove the only known copy:

clearly communicate:

"This is the only copy currently stored in Qurb."

Do not use technical replication terminology.

Do not silently allow the user to destroy the only known copy.

============================================================
14. CONFLICTS
============================================================

Qurb retains conflicting versions rather than silently overwriting them.

A conflict should appear as an attention state.

Example:

"1 thing needs attention"

"Project Plan.pdf has two versions."

[ Review ]

Conflict resolution should provide:

Keep this version
Keep the other version
Keep both

Do not overwhelm Home with conflict-management controls.

============================================================
15. PRIVATE VAULT
============================================================

Private Vault belongs to the current device.

It must feel distinct from shared Qurb space.

However, it should NOT look like a secret government bunker.

Use subtle privacy cues.

Example:

Private Vault

Private to this device.

Files here are only accessible from this device.

The lock icon should be restrained.

============================================================
16. DEVICES
============================================================

Devices should feel spatial.

The user should think:

"My phone"
"My laptop"
"My desktop"

not:

"Node A"
"Node B"

Devices can display:

- device name
- device type
- availability
- local storage where relevant
- recent activity
- transfer status

Do not show networking topology by default.

Do not show:
- latency
- peer IDs
- protocol information
- replication factor
- node health
- internal identifiers

unless explicitly designing an advanced diagnostic interface.

============================================================
17. SEND
============================================================

Send is a deliberate user action.

It is NOT the same thing as Sync.

The flow should communicate:

WHAT
is being sent

TO WHICH DEVICE

WHAT IS HAPPENING

WHEN IT IS COMPLETE

Keep the interaction focused.

The send experience should feel spatial and alive.

============================================================
18. FILES PAGE
============================================================

Files should feel like a beautiful, familiar file browser.

Not a dashboard.

Structure:

Files

Search

Qurb / Folder

Folders

Files

Each row should contain:

icon
name
optional metadata
subtle availability state
contextual action when appropriate

Do not display dozens of metadata fields.

Folders should visually stand apart from files.

Use glass surfaces sparingly.

The actual file list should remain highly readable.

============================================================
19. FILE DETAILS
============================================================

File Details should be a focused information surface.

Show:

Name
Type
Size
Location
Availability
Devices holding the file
Last modified
Versions where relevant
Transfer state
Actions

Hide technical information unless explicitly requested.

Use an elevated glass panel or sheet.

============================================================
20. STORAGE PAGE
============================================================

Storage is where Qurb can explain local storage behavior.

Primary message:

"Free space without losing your files."

Show:

Local storage used
Local storage available
Space that can safely be freed

Then:

files that can be removed locally while remaining available in Qurb.

Do not frame this as deletion.

============================================================
21. TRANSFERS
============================================================

Transfers should feel alive.

Show:

Active

Project.zip
Sending to Phone
82%

Then completed transfers.

When nothing is happening:

"No active transfers"

Do not make Transfers a giant monitoring dashboard.

============================================================
22. RECENTLY DELETED
============================================================

Show:

File
Deletion time
Retention period
Restore action

Example:

Project Plan.pdf
Deleted today
Expires in 29 days

[ Restore ]

The interface should communicate recoverability and calmness.

============================================================
23. SETTINGS
============================================================

Settings should be visually quieter.

Use grouped lists rather than card-heavy layouts.

Categories:

Account
Devices
Storage
Privacy
Notifications
Recovery
Appearance
Advanced / Diagnostics

Settings should not look like a dashboard.

============================================================
24. DESKTOP NAVIGATION
============================================================

Primary navigation:

Home
Files
Devices
Storage
Settings

Private Vault can appear as a visually distinct secondary area.

Example:

Qurb

Home
Files
Devices
Storage

PRIVATE

Private Vault

Settings

Use a translucent sidebar.

Do not create navigation items simply because backend subsystems exist.

============================================================
25. MOBILE NAVIGATION
============================================================

Bottom navigation:

Home
Files
Devices
Settings

Maximum four primary destinations.

Transfers should become contextually visible when active.

Private Vault should be accessible without consuming a permanent bottom navigation slot.

Mobile must be intentionally designed for mobile.

Do not simply shrink the desktop layout.

============================================================
26. SPATIAL DESIGN
============================================================

The interface should have depth.

Establish:

Background
↓
Environmental light
↓
Clear glass
↓
Frosted glass
↓
Elevated glass
↓
Focused controls

Depth should communicate hierarchy.

Do not use heavy shadows.

Use:

- soft shadows
- translucent borders
- subtle highlights
- backdrop blur
- slight surface contrast

Glass should feel physical.

============================================================
27. MOTION DESIGN
============================================================

Motion is a core part of Qurb's identity.

Do NOT treat animation as decoration.

Every animation must communicate one of:

- where something came from
- where something is going
- what changed
- why something changed
- whether something is still happening
- whether something completed

The animation system must reinforce Qurb's mental model.

============================================================
28. QURB SIGNATURE MOTION
============================================================

Qurb should have one recognizable motion language:

FLOWING QURB LIGHT.

A subtle luminous green flow represents data movement.

It can appear during:

- Send
- Sync
- Download
- Restore
- Device connection
- Storage state changes

Do NOT literally draw cables everywhere.

The motion should be abstract.

Think:

light
flow
particles
soft trails
spatial movement

rather than:

arrows
technical diagrams
network graphs

The user should eventually associate:

"flowing green light = Qurb is moving something."

============================================================
29. FILE TRANSFER ANIMATION
============================================================

When:

Project.pdf

moves from:

Laptop → Phone

do not simply change text from:

Sending

to:

Complete.

Represent the movement.

The source file can emit a subtle Qurb-green light.

A small abstract representation can travel toward the destination.

Then the destination receives a soft pulse.

Final state:

Sent

The animation should teach the user that the file moved between devices.

Do not make it theatrical.

Target duration:

approximately 500–900ms for the meaningful transition.

============================================================
30. KEEP FILE ON DEVICE ANIMATION
============================================================

When a remote-only file becomes local:

AVAILABLE ELSEWHERE
↓
DOWNLOADING
↓
ON THIS DEVICE

Use subtle visual flow.

The file remains visually stable.

A soft stream of light enters the file representation.

The local-storage indicator appears.

The state settles.

This should make the concept of downloading intuitive without requiring an explanation.

============================================================
31. FREE LOCAL SPACE ANIMATION
============================================================

This is one of the most important Qurb animations.

When the user chooses:

Free local space

the file itself must remain visually stable.

Only the LOCAL BYTES should visually disappear.

Conceptually:

File
+
Local bytes
↓
Local bytes dissolve
↓
File remains
↓
Available elsewhere

The animation should visually communicate:

"The file was not deleted."

This is more important than decorative motion.

============================================================
32. SYNC ANIMATION
============================================================

Do not use generic spinning loaders everywhere.

During sync:

Use subtle movement around the relevant state.

Possible visual language:

soft flowing ring
small moving light
subtle pulse
short directional flow

When complete:

Syncing
↓
Synced

The transition should feel satisfying but restrained.

============================================================
33. HEALTHY STATE ANIMATION
============================================================

When Qurb reaches:

Everything is synced.

use a very subtle environmental response.

A soft green light can briefly pass through the glass environment.

Then the interface becomes still.

The message remains.

The animation happens once.

Calm is the reward.

============================================================
34. DEVICE CONNECTION ANIMATION
============================================================

When a new device joins Qurb:

Do not simply add a card.

Allow the device representation to gently materialize.

For example:

translucent outline
↓
soft glow
↓
device becomes solid
↓
connected state

The motion should communicate:

"This device has entered your Qurb space."

============================================================
35. NAVIGATION MOTION
============================================================

Navigation should feel spatial.

Avoid abrupt page replacement.

Home → Files:

Home subtly recedes.

Files subtly moves forward.

Use:

- scale
- opacity
- translation
- blur

very carefully.

Target:

approximately 250–350ms.

Do not overanimate.

============================================================
36. BOTTOM SHEETS
============================================================

Use glass bottom sheets heavily on mobile.

When opening file actions:

The sheet rises from the bottom.

The background:

- slightly dims
- slightly blurs
- remains visible

The sheet:

- elevated glass
- rounded upper corners
- subtle edge highlight
- soft shadow

Motion should use spring-like behavior.

Target:

approximately 400–550ms.

============================================================
37. DESKTOP PANELS
============================================================

Desktop should feel like a spatial application.

Use floating translucent panels rather than a page filled with rectangular cards.

The main content should appear to float within the environment.

Sidebar can be clear/frosted glass.

Primary content can use frosted glass selectively.

Elevated surfaces should be reserved for active interactions.

============================================================
38. HOVER MOTION
============================================================

Desktop hover states should be extremely subtle.

For a file row:

- surface slightly brightens
- border becomes slightly more visible
- action controls gently appear
- icon may translate 1–2px

Target:

150–200ms.

Do not make rows bounce or dramatically scale.

============================================================
39. BUTTON MOTION
============================================================

Buttons should feel tactile.

Hover:
- slight brightness increase
- slight elevation

Press:
- scale approximately 0.98
- shadow compresses

Release:
- spring back smoothly

The interaction should feel physical but restrained.

============================================================
40. MOTION TIMING
============================================================

Small interactions:

150–200ms

Standard transitions:

250–350ms

Sheets / panels:

400–550ms

Large spatial transitions:

500–700ms

Use spring-like easing for:
- sheets
- floating panels
- spatial transitions
- buttons

Use smooth easing for:
- opacity
- background
- progress
- ambient lighting

Do not use the same easing for everything.

============================================================
41. MOTION RESTRAINT
============================================================

The product must never feel like an animation showcase.

Do NOT animate:
- every icon
- every card
- every page element
- background continuously
- decorative objects
- random floating particles

Motion should happen when the system or user has caused something meaningful.

Dynamic environment + stable information.

This is critical.

============================================================
42. REDUCED MOTION
============================================================

Support reduced-motion preferences.

When reduced motion is enabled:

- remove spatial transitions
- remove particle effects
- remove ambient movement
- remove unnecessary scaling
- use simple fades
- keep state changes immediate and understandable

The product must remain beautiful and usable.

============================================================
43. CARDS
============================================================

Use cards sparingly.

A card should represent a meaningful conceptual boundary.

Good:

Device
Conflict
Storage recommendation
Transfer

Bad:

Files today
Storage
Devices
Activity
Network
Health
Sync statistics
Usage

Do not create card soup.

Glass surfaces should replace unnecessary card proliferation.

============================================================
44. BUTTONS
============================================================

Primary button:

Qurb green glass/filled material depending on context.

Secondary:

Translucent neutral glass with subtle border.

Tertiary:

Text / ghost action.

Destructive:

Muted red.

"Free local space" is NOT destructive and should not look like a destructive action.

============================================================
45. CORNER RADIUS
============================================================

Use a consistent radius system.

Small:
8px

Medium:
12–16px

Large:
18–20px

Bottom sheets:
20–24px top corners

Do not make every element excessively rounded.

The interface should feel premium and precise.

============================================================
46. ACCESSIBILITY
============================================================

Maintain:

- sufficient contrast
- readable typography
- clear focus states
- keyboard navigation
- minimum touch targets
- non-color state communication
- understandable labels
- predictable interaction
- reduced motion support

Beautiful must never mean difficult to use.

============================================================
47. ONBOARDING
============================================================

Onboarding should be visually beautiful but extremely calm.

Welcome:

Qurb

Your files.
Your devices.
Your space.

[ Get started ]

Recovery phrase screens should be intentionally sparse.

The recovery phrase is a high-trust interaction.

Avoid:
- decorative animation
- unnecessary gradients
- marketing language
- distracting visual effects

Use the visual system, but prioritize trust and comprehension.

============================================================
48. DESIGN SYSTEM
============================================================

Create reusable components for:

- navigation
- glass surfaces
- buttons
- icon buttons
- file rows
- folder rows
- device rows
- state badges
- dialogs
- sheets
- menus
- search
- breadcrumbs
- progress
- transfer states
- empty states
- error states
- attention states
- success states
- avatars

Components must behave consistently.

Do not manually recreate the same pattern differently on every page.

============================================================
49. PAGE STRUCTURE
============================================================

Design the product around:

ONBOARDING
- Welcome
- Create / Join
- Recovery phrase
- Recovery verification
- Device setup

HOME
- Healthy
- Attention
- Transfer
- Offline

FILES
- File browser
- Search
- Folder
- File details

DEVICES
- Device list
- Device details
- Device availability

SEND / TRANSFER
- Choose source
- Choose destination
- Progress
- Complete
- Failed

CONFLICTS
- Conflict list
- Conflict details
- Version comparison
- Resolution

RECENTLY DELETED
- Deleted files
- Retention
- Restore

STORAGE
- Local storage
- Safe local space
- Storage management

PRIVATE VAULT
- Private files
- Device-private explanation

SETTINGS
- Account
- Devices
- Storage
- Privacy
- Recovery
- Notifications
- Appearance
- Advanced

============================================================
50. RESPONSIVE DESIGN
============================================================

Desktop and mobile must share the same mental model but NOT be identical layouts.

Desktop:
- more spatial context
- floating panels
- translucent sidebar
- larger content areas
- richer hover interactions

Mobile:
- bottom navigation
- sheets
- large touch targets
- fewer simultaneous elements
- focused actions
- shorter information hierarchy

Never simply shrink desktop.

============================================================
51. INFORMATION DENSITY
============================================================

This is a HARD constraint.

When choosing between:

more information

and

more understanding

choose understanding.

When choosing between:

more features visible

and

a clearer primary action

choose the clearer primary action.

When choosing between:

technical accuracy presented directly

and

technical accuracy translated into human language

choose the human language.

The interface should expose complexity progressively.

============================================================
52. COPYWRITING
============================================================

Use plain human language.

Prefer:

"Everything is synced."

instead of:

"Synchronization status: healthy."

Prefer:

"Available elsewhere."

instead of:

"Remote-only materialization."

Prefer:

"Free local space."

instead of:

"Evict local replica."

Prefer:

"Only copy here."

instead of:

"Unique replica."

Prefer:

"Send to device."

instead of:

"Initiate device transfer."

============================================================
53. ANTI-PATTERNS
============================================================

Never:

- create a generic cloud-drive clone
- turn Home into a dashboard
- overload screens with cards
- expose backend architecture
- use excessive glass
- make every element translucent
- use glass without hierarchy
- use heavy blur everywhere
- use neon cyberpunk colors
- use excessive gradients
- use excessive animation
- animate everything
- use decorative particles constantly
- use technical terminology unnecessarily
- call freeing local space "deleting"
- silently overwrite conflicts
- hide the distinction between shared space and Private Vault
- make every action destructive
- create navigation for backend subsystems
- prioritize visual spectacle over comprehension

============================================================
54. DESIGN REVIEW
============================================================

Before considering any screen complete, evaluate:

CLARITY
Can a first-time user understand this?

HIERARCHY
Is the most important thing obvious?

SIMPLICITY
Can anything be removed?

VISUAL QUALITY
Does the glass, lighting, typography, spacing, and depth feel premium?

MOTION
Does animation communicate something meaningful?

MATERIAL
Does glass have a clear purpose?

TRUST
Does the user understand what will happen?

CONSISTENCY
Does the screen use existing Qurb patterns?

PROGRESSIVE DISCLOSURE
Is unnecessary complexity hidden?

ACCESSIBILITY
Can the experience be understood and operated by different users?

============================================================
55. DESIGN EXECUTION
============================================================

Build Qurb as a coherent design system rather than a collection
of isolated screens.

Establish the visual foundation before designing the full product.

Create reusable foundations for:

1. Colors
2. Typography
3. Spacing
4. Radius
5. Glass materials
6. Blur levels
7. Borders
8. Shadows
9. Icons
10. State colors
11. Motion principles
12. Component variants

Create reusable components before duplicating patterns across
screens.

Every screen must inherit the same:

- visual language
- material hierarchy
- typography
- spacing system
- iconography
- state language
- interaction patterns
- motion language

Do not invent a new visual treatment for individual screens
unless there is a strong UX reason.

============================================================
56. DESIGN ORDER
============================================================

Do NOT immediately create every page.

Build the product in this order:

PHASE 1
Visual foundation

PHASE 2
Glass material system

PHASE 3
Lighting/environment

PHASE 4
Typography and iconography

PHASE 5
Motion principles

PHASE 6
Core components

PHASE 7
Navigation

PHASE 8
Home

PHASE 9
Files

PHASE 10
File Details

PHASE 11
Devices

PHASE 12
Send / Transfers

PHASE 13
Conflicts

PHASE 14
Storage

PHASE 15
Recently Deleted

PHASE 16
Private Vault

PHASE 17
Settings

PHASE 18
Onboarding

PHASE 19
Edge states

PHASE 20
Responsive refinement

============================================================
57. FINAL NORTH STAR
============================================================

Qurb should feel like this:

A beautiful, living space where your files move naturally
between your devices.

The technology should feel powerful.

The interface should feel effortless.

The glass should create depth.

The motion should explain the system.

The typography should create clarity.

The color should establish identity.

The interface should never make the user think about
distributed storage.

The user should simply feel:

"My files are here."

"They are safe."

"They are available where I need them."

"Qurb understands where everything is."

The ultimate design principle is:

COMPLEX SYSTEM UNDERNEATH.
SIMPLE TRUTH ON TOP.

MAKE QURB GORGEOUS.

BUT NEVER LET BEAUTY COMPETE WITH UNDERSTANDING.

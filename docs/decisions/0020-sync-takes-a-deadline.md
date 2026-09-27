# 0020 — Sync takes a deadline

**Status:** Accepted. Amended 2026-09-27: what `timed_out` means, and the
deadline now covers starting — see [Found on a phone](#found-on-a-phone).
**Date:** 2026-09-17

## Decision

The mobile entry point for syncing is:

```
sync_within(seconds) -> SyncOutcome
```

not `sync()`. It stops when the time is up and says so in
`SyncOutcome.timed_out`, which is **not an error**. A connector is built for
each pass and dropped at the end of it, rather than held for the life of the
process.

## Why a deadline

Both mobile platforms hand background work a window and kill anything that
outstays it. iOS `BGTaskScheduler` grants short and unpredictable windows and
penalises an app that overruns by granting fewer of them later. Android's
`WorkManager` is more generous and still finite.

So "sync until finished" is not a neutral default on a phone — it is the way an
app loses its background privileges, and it does so gradually and invisibly.
An app cannot wrap a blocking call in its own timeout either, because killing a
thread mid-transfer is not something either platform offers.

The deadline therefore has to be inside, where the loop is.

## Why running out of time is not an error

Because nothing was lost.

The engine commits each file as it lands rather than at the end of a plan, so a
pass that stops halfway leaves every completed file stored, indexed and
correct. The remainder is simply work the next window will do. Reporting that as
a failure would be false, and would push an app towards showing the user an
error for the normal case.

`timed_out` exists so the platform side can schedule another window rather than
wait for the next scheduled one. That is a signal, not a fault.

## Why a connector per pass

The desktop daemon builds one `Connector` and keeps it for hours, because a
laptop's address is stable for hours and rebuilding it would mean re-announcing,
re-discovering, and briefly disappearing from the rendezvous service.

A phone's address is not stable for hours. It changes every time the device
moves between Wi-Fi and cellular, which on a normal day is several times. A
long-lived connector holding a public address discovered an hour ago announces
somewhere nothing can reach, and the failure looks like the *other* device being
absent.

Building one per pass costs a STUN round trip and a re-announce each time. That
is a few hundred milliseconds against a window measured in tens of seconds, and
it buys an address that is correct now.

It also happens to fix, for mobile only, the Phase 3 gap where a connection that
fell back to the relay stays relayed after the network improves: the next pass
starts fresh and gets a fresh chance at a direct path.

## Serving only while syncing

The desktop daemon accepts incoming connections continuously. A phone accepts
them only for the duration of a pass.

This is not a compromise — it is what the platform permits. A backgrounded app
is suspended the moment its window closes, and a socket it left open stops being
serviced regardless of intent. Pretending otherwise would mean holding resources
the system is about to take back.

The consequence is stated plainly because it is significant: **two devices that
are both only briefly awake may never meet.** Sync needs both ends announced and
listening at the same moment, since a QUIC handshake's opening packets are the
hole punch. Two desktops manage this by being on all the time. Two phones, each
awake for seconds a day at their platform's discretion, may not overlap for days.

This is the strongest argument in the project for a storage-only replica
([decision 0006](0006-availability-gap.md)) being part of a normal setup rather
than an advanced option, and it is why the roadmap plans iOS as a good viewer
rather than a peer equal to a desktop.

## Alternatives rejected

**Async functions across the FFI.** UniFFI supports them, and they map to Kotlin
`suspend` and Swift `async`. Rejected because cancellation across an FFI
boundary is genuinely hard to get right, and the thing being cancelled is a
transfer holding a database transaction. A deadline the Rust side enforces is
one mechanism in one place; cancellation would be two mechanisms in three
languages.

**A callback the platform can use to say "stop now".** Same objection, plus it
puts the decision on the far side of the boundary, where it would be called from
a thread that knows nothing about what the loop is in the middle of.

**Let the app pass a huge deadline and effectively sync forever.** Still
available — `sync_within(3600)` is legal — and that is deliberate: in the
foreground, with the user watching a progress bar, it is the right call. What
matters is that the parameter exists and has to be thought about.

## Consequences

- An app must schedule its own repeat passes. There is no background loop inside
  this crate, because a loop that outlives its window is the problem being
  avoided.
- A pass that reaches no peers is indistinguishable from one where every peer
  was asleep, and both are ordinary. `SyncOutcome` counts `unreachable`
  separately from raising an error for exactly this reason.
- The per-pass connector means STUN runs per pass. On a metered connection that
  is a small repeated cost, which is why `Settings::discover` can turn it off —
  at the price of only reaching peers on the same network.

## Found on a phone

2026-09-27, Galaxy S23. The rebuilt app's Settings screen showed the background
worker's last record as *"no paired devices"*, on a phone paired with a desktop
that was switched off. Two defects were behind it, both in how the deadline was
applied rather than in the idea.

**A device that never answered was reported as time running out.** Reaching a
device through the rendezvous service waits up to twenty seconds for an
introduction, and the background worker's window was twenty seconds. So the
window closed while the phone was still waiting, and the pass came back as
`timed_out` with nothing reached and nothing unreachable. The worker read that
as "no paired devices" — wrong — and answered `timed_out` with a retry, which on
Android is exponential backoff. That is the three-hour gap between syncs that
the worker had already been fixed once for, arrived at by a different route: a
switched-off computer is the ordinary state of things, and the ordinary
schedule is the answer to it.

`timed_out` now means what this record always said it meant — work left over:
a device that answered was still being synced, or devices were left untried. A
device that has not answered when the window closes is counted in
`unreachable`, however long it was waited for.

**Starting was outside the deadline.** Each pass starts a connector, which
connects to the rendezvous service, and that handshake had no bound at all. A
service that accepts the connection and never answers held the pass for ever,
whatever deadline was asked for — found while writing the test for the first
defect, which hung for ten minutes. The handshake is now bounded at five
seconds (`qurb_signal::client::HANDSHAKE_TIMEOUT`), which also bounds the
desktop daemon's start and its reconnect loop, and the phone's connector start
now runs inside the pass's deadline; a window that closes before the phone
could try anybody counts every device as unreachable.

What was observed and what was inferred: the phone's record and the paired
device are observed. The phone's own log of that run had rolled over before it
was looked for, so its outcome was not read directly; the mechanism was
reproduced instead. `a_device_that_never_answers_is_unreachable_not_out_of_time`
in `crates/mobile-ffi/tests/syncing.rs` gives the phone a rendezvous address that
never answers and an eight-second window, and without the fix returns exactly
`reached: 0, unreachable: 0, timed_out: true`. The older test for an unreachable
device accepted either answer, which is how this went unnoticed.

**Not fixed:** devices are tried one after another. With two paired devices and
the first switched off, waiting for the first can use the whole window, and the
second is never tried; the pass is then `timed_out`, retried, and the same thing
happens next time. Trying devices concurrently is the fix, and has not been
made.

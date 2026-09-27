# Service capacity

Throwaway. How much a small server running the rendezvous service and the relay
carries, measured rather than asserted.

`src/main.rs` is a load generator built on the real client libraries
(`qurb-signal`, `qurb-relay`); `measure.sh` starts the real `qurb signal` or
`qurb relay`, runs the load, and reads the service process's resident memory
and CPU time from `/proc` at the points the load pauses.

```bash
cargo build --release -p qurb-cli -p service-capacity
experiments/service-capacity/measure.sh signal 1000 4000 9000
experiments/service-capacity/measure.sh signal-tls 1000 4000
experiments/service-capacity/measure.sh relay-idle 1000
experiments/service-capacity/measure.sh relay "1 512" "8 128"
```

## Conditions

2026-09-27. The development laptop: Intel i5-13420H (12 threads), 16 GB, Linux
7.2 (CachyOS). Release builds. **Everything over loopback, on one machine** —
the load generator and the service share the CPU, and there is no network in
between: these are the services' own costs, not what a real network adds.

Devices come in pairs, one group per pair — one person's phone and laptop —
each connected and announced. An *introduction* is one device asking for the
other and both being told to connect (`connect_to` until `Punch`), timed from
the asking device, 200 of them, while every device stays connected.

## Results

**The rendezvous service**, plain `ws://`:

| devices connected | resident memory | per device, marginal | introduction p50 / p95 |
|---|---|---|---|
| none | 6.2 MB | — | — |
| 1,000 | 30.9 MB | | 0.09 / 0.12 ms |
| 4,000 | 97.2 MB | 22 KiB | 0.09 / 0.11 ms |
| 9,000 | 207.6 MB | 22 KiB | 0.10 / 0.12 ms |

Before the change this measurement prompted, 1,000 devices took **159 MB**,
about 150 KiB each: the WebSocket library's default buffers are 128 KiB to read
and 128 KiB to write, per connection, for messages the service caps at 16 KiB.
They are now 4 KiB and unbuffered writes, with the unsent backlog bounded.

**The rendezvous service presenting its own certificate** (`--tls`, what a
server with no domain name runs; devices pin its fingerprint):

| devices connected | resident memory | per device, marginal | introduction p50 / p95 |
|---|---|---|---|
| 1,000 | 40.3 MB | | 0.10 / 0.15 ms |
| 4,000 | 128.7 MB | 29.5 KiB | 0.10 / 0.14 ms |

A TLS handshake cost the service about 0.26 ms of CPU.

**The relay**, idle: 1,000 devices registered and sending nothing — every
device that is on and configured to fall back to it — took it from 6.4 MB to
25.4 MB, about 19 KiB each.

**The relay**, carrying QUIC sessions the way a sync through it would:

| transfers at once | carried | time | throughput | relay CPU | relay memory, peak |
|---|---|---|---|---|---|
| 1 | 512 MiB | 2.13 s | 241 MiB/s | 2.47 s | 11 MB |
| 8 | 8 × 128 MiB | 1.41 s | 726 MiB/s | 4.24 s | 21 MB |

About 4–5 CPU-seconds per GiB relayed, on this CPU.

The worst introduction in every run was about 40 ms, against a p95 near
0.1 ms. Not investigated; it looks like scheduling on a machine running the load
and the service at once.

## What it says about a small server

- **Memory is not the limit for anyone's own devices.** Two devices cost the
  rendezvous service about 60 KiB. A 1 GB server would hold tens of thousands
  of devices by memory — the service's own default cap, `max_connections`,
  is 10,000.
- **The relay's limit is bandwidth, not the machine.** It carried hundreds of
  MiB/s using a few CPU-seconds per GiB; a small server's network link, and its
  monthly transfer allowance, run out long before its CPU. Every byte relayed
  is a byte paid for, which is why the relay is only a fallback.

## Corners cut

- Loopback only. Real network latency, packet loss and a real server's slower
  cores are not in these numbers.
- Connections are opened and held; nothing measures the churn of phones
  appearing and leaving every few minutes, or push wake-ups.
- The time taken to connect all devices is the load generator's pace, not the
  service's, and is not reported as a result.
- The relay load accepts any certificate; it measures carrying bytes, which the
  relay does without any key either way.

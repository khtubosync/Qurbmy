# Architecture

The target design, subsystem by subsystem, as first written. This describes
where the project was going; [CODEBASE.md](CODEBASE.md) section 5 describes
what actually exists today, and [features.md](features.md) what a person can
do with it.

Decisions with lasting consequences have their own records in
[decisions/](decisions/); this document summarises and links rather than
repeating the reasoning.

## Where the built system departs from this

Checked 2026-09-28. The sections below are left as they were planned; where
building it chose differently, the decision record is the authority:

| planned here | built | why |
|---|---|---|
| Go for the control plane (§3) | Rust for the rendezvous service and the relay; accounts and billing not built | [0015](decisions/0015-control-plane-in-rust.md) |
| Jetpack Compose on Android (§2) | the platform's own Views | [0039](decisions/0039-a-light-android-app.md) |
| Noise `IK` in transit (§7) | TLS 1.3 inside QUIC, each side pinning the other's certificate fingerprint | [0011](decisions/0011-peer-identity-pinning.md) |
| ChaCha20-Poly1305 or AES-GCM at rest (§7) | XChaCha20-Poly1305 | [crates/storage](../crates/storage/README.md) |
| per-file keys (§7) | keys per purpose from the master key; per-file keys not built | [0012](decisions/0012-key-hierarchy-and-recovery.md) |
| a CAS holding every chunk (§4) | on a device with a folder, the file *is* its own payload; the CAS keeps only what the folder cannot | [0024](decisions/0024-the-file-is-the-payload-store.md) |
| chunk GC "not yet designed" (§4) | built in Phase 1, two-stage, running every five minutes | [phases/phase-1-engine.md](phases/phase-1-engine.md) |
| Merkle DAGs for tree comparison (§5) | the whole tree is exchanged and compared — 3.5 s to sync 1,000 changed files among 100,000 | [phases/phase-1-engine.md](phases/phase-1-engine.md) |
| deletions undone from chunks kept a while (§5) | a deleted file kept whole for thirty days in *Recently deleted* | [0042](decisions/0042-recently-deleted.md) |
| DERP on port 443 (§6) | qurb's own relay, TCP on 443, carrying a QUIC session it cannot read | [0017](decisions/0017-relay.md) |
| gRPC for signalling (§10) | JSON over a WebSocket, because a readable capture matters more than bytes there | [crates/signal](../crates/signal/README.md) |
| FTS5 and CLIP search (§8) | names only, by substring; nothing on contents or images | — |
| a central device registry (§1, §7) | each device's own trust list, filled by pairing and emptied by removal | [0041](decisions/0041-removing-a-device.md) |
| signed delta updates with rollback (§11) | no automatic updater; the index is copied before it migrates | [0047](decisions/0047-versions-and-upgrades.md) |
| an OCI container for a NAS (§12) | `qurb replica` and a systemd unit; no container | [packaging/server](../packaging/server/README.md) |
| aggregate metrics and crash reports (§13) | logs, and each device's own history of what happened; nothing leaves the device | [0031](decisions/0031-what-happened-is-written-down.md) |

Not in this design at all, and built: private vaults and sending to one device
([0029](decisions/0029-two-areas-shared-and-private.md),
[0030](decisions/0030-sending-a-file-to-one-device.md)), a device keeping a
phone's files for it ([0036](decisions/0036-a-phone-keeps-its-own-files.md)),
a storage limit that cannot lose data
([0025](decisions/0025-a-storage-cap-that-cannot-lose-data.md)), finding
devices on one network with no server
([0034](decisions/0034-finding-each-other-with-no-server.md)), sharing a folder
with chosen devices ([0044](decisions/0044-sharing-with-chosen-devices.md)).

---

## 1. Topology

Hybrid peer-to-peer with a central coordination plane. File data moves directly
between the user's devices; cloud services handle identity, discovery, NAT
traversal coordination, and encrypted relay fallback.

Full reasoning: [0001](decisions/0001-hybrid-p2p-topology.md).

```
        DEVICES (data plane)              SERVERS (control plane)
   ┌──────────────────────┐          ┌───────────────────────────┐
   │ desktop              │◄────────►│ accounts, billing         │
   │  engine · index · CAS│ metadata │ device registry           │
   └──────────┬───────────┘          │ STUN · signalling         │
              │ direct QUIC          │ DERP relay (fallback)     │
   ┌──────────▼───────────┐          └───────────────────────────┘
   │ mobile               │
   └──────────────────────┘
```

## 2. Clients

Rust core shared across all platforms via FFI. Tauri on desktop, SwiftUI on
iOS, Jetpack Compose on Android. Reasoning:
[0002](decisions/0002-rust-core-with-native-shells.md).

## 3. Backend

Go for control-plane APIs — registration, authentication, device registry,
billing, push routing. STUN and DERP relay nodes are separate services optimised
for network throughput.

Go is chosen for development velocity on ordinary API work and for cheap
concurrency across many idle websocket connections. Its garbage collector is
irrelevant here because the servers handle metadata and packet forwarding, not
latency-critical work.

## 4. Local storage

SQLite in WAL mode for metadata; a content-addressable store on the filesystem
for chunk payloads. Reasoning: [0003](decisions/0003-sqlite-plus-cas.md).

```
/Photos/Summer.jpg
      │
      ▼  SQLite: path → ordered chunk hashes
   [9f86d0…] [132f95…] [a14b7c…]
      │
      ▼  filesystem
   chunks/9f/86d0…   chunks/13/2f95…   chunks/a1/4b7c…
   (compressed, then encrypted)
```

Chunk deletion requires reference counting, since deduplication means a chunk may
belong to many files and versions. **Not yet designed. First correctness problem
of Phase 1.**

## 5. Synchronisation engine

Content-defined chunking with FastCDC, BLAKE3 hashing, and Merkle DAGs for
tree comparison.

Chunk bounds are 128 KiB / 512 KiB / 2 MiB, chosen by measurement rather than
by intuition — see [0004](decisions/0004-chunk-parameters.md), which corrects the
original proposal.

**State tracking.** Each device keeps a vector clock, `{deviceA: 42, deviceB:
12}`. Devices exchange these small vectors to detect divergence before
exchanging anything larger.

**Conflicts.** Concurrent edits are both retained; the automatic rule decides
only which keeps the original filename. Never discards an edit. See
[0005](decisions/0005-conflict-resolution.md).

**Deletions.** Recorded as tombstones and propagated, because the absence of a
file is not self-describing — a peer cannot otherwise distinguish "deleted" from
"not yet received." Underlying chunks survive a retention window so deletions
can be undone.

## 6. Networking

QUIC via `quinn`, over UDP. Multiple independent streams per connection, so one
stalled transfer does not block others, and 1-RTT connection setup with
encryption included.

NAT traversal:

```
1. each device asks STUN for its public address
2. addresses exchanged via the coordination plane
3. both send UDP simultaneously — hole punching
4. if that fails, relay through DERP on port 443
```

Expect roughly 80% of connections to go direct. The remainder cost bandwidth
indefinitely, which makes the direct-connection rate a business metric and not
only a technical one.

## 7. Security

Zero-knowledge and end-to-end encrypted. Servers see ciphertext and routing
metadata; never filenames, directory structure, or content.

- **In transit:** Noise Protocol Framework, `IK` pattern, static Curve25519
  identities, ephemeral Diffie-Hellman for forward secrecy.
- **At rest:** ChaCha20-Poly1305, or AES-256-GCM where hardware acceleration
  exists.
- **Keys:** a master secret generated on the first device from a cryptographically
  secure RNG, presented as a 24-word recovery phrase. Per-file keys derived via
  HKDF-SHA256, so compromising one file key reveals nothing about others.
- **Pairing:** out-of-band verification by QR code. Confirmed identities are
  stored in an append-only device trust graph held locally on each device, with
  no central authority deciding who is trusted.

The unavoidable consequence: **a lost key is unrecoverable data.** See
[roadmap.md](roadmap.md), "the two risks that are not technical."

## 8. Search

Local only. SQLite FTS5 over filenames, paths, and extracted document text.

Image search runs a quantised CLIP model on-device via `onnxruntime`, producing
embeddings stored in a local vector index. This gives natural-language image
search — "dog on a beach" — without any data leaving the machine.

Rejected: Elasticsearch, which cannot reasonably run inside a desktop tray
application.

## 9. Background execution

Tokio async runtime with a prioritised worker pool.

Filesystem events come from native interfaces — `inotify`, `FSEvents`,
`ReadDirectoryChangesW` — through the `notify` crate, so there is no polling.

```
high     path changes, moves, locks, metadata
medium   chunking, network transfer
low      thumbnails, CLIP embeddings, full-text indexing
```

Interrupted transfers resume from the last completed chunk boundary, with
exponential backoff from 1s to a 300s ceiling.

## 10. API surface

Two protocols, deliberately.

- **Control and signalling:** Protocol Buffers over gRPC. Type-safe, compact,
  cross-language.
- **Bulk data:** a custom binary protocol directly over QUIC streams, avoiding
  HTTP framing overhead when moving chunk payloads.

Rejected: REST/JSON, whose parsing overhead and verbosity waste mobile battery
during large sync passes; GraphQL, whose flexibility buys nothing for a fixed
set of machine-to-machine messages.

## 11. Updates

Signed with Ed25519 and verified against a public key embedded in the
application before installation. Delta patches via bsdiff to keep downloads
small. The previous binary is retained; if the new one fails a health check
within 60 seconds of starting, it is restored automatically.

## 12. Deployment

Two shapes. A native installer for ordinary users — tray application,
zero configuration, automatic NAT traversal. A headless OCI container for NAS
and home-lab users, with `/data` and `/config` volumes, targeting under 30 MB
of RAM at idle on an ARM64 single-board computer.

## 13. Observability

Aggregate metrics only: CPU, memory, throughput, and whether NAT traversal went
direct or via relay. No filenames, no paths, no addresses.

Crash reports are scrubbed of paths and user data, written locally, and
transmitted only after the user explicitly agrees. Local logs use the `tracing`
crate, capped at 50 MB with rotation.

## 14. Scaling

Because the servers carry metadata and signalling rather than file data, they
scale far more cheaply than the user count suggests.

```
100         one instance, SQLite
1k          dedicated relay nodes in primary regions
10k         PostgreSQL, regional discovery behind anycast
100k        horizontally scaled discovery, global relay mesh
1M          sharded coordination database; P2P still carries ~95% of bytes
```

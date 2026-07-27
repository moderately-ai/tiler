---
schema: "tiler-doc/v1"
id: "ADR-0083"
kind: "decision"
title: "Keep process-crash as the default cache durability"
topics: ["cache", "durability", "measurement", "artifacts"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.artifact-abi"]
evidence: ["tiler.research.cache.crash-race-protocol"]
depends_on: ["ADR-0050", "ADR-0082"]
ticket: "measure-expansion-cache-durability-policies"
---

# 0083: Keep process-crash as the default cache durability

**Status:** accepted. `ProcessCrash` remains the default expansion-cache durability policy, now on measurement rather than only on ADR 0050's recommendation. This closes the fourth follow-up gate of the crash-and-race research note.

## Context

`tiler_cache::expansion::Durability` implements two policies, and neither changes a published byte — a test asserts that, because a policy that changed the bytes would make an entry written under one unreadable under the other.

- **`ProcessCrash`** writes, validates, closes, and renames. A killed writer cannot expose a partial temporary at the final path.
- **`Fsync`** additionally synchronizes the temporary before the rename and the containing entry directory afterwards.

`ProcessCrash` was the default because ADR 0050 recommended it, and the type's own documentation said so explicitly rather than implying the choice had been measured.

## Measurement

`hot_path_publication_by_durability` in `crates/tiler-cache/src/expansion/hot_path.rs`. Each round publishes into a fresh cache under a fresh scratch root, because a second publication of the same subject is a hit and would time a read; the test asserts `published: true` on every round rather than assuming it. Minimum of N is reported because host noise only ever makes a run slower.

**Two hosts, both local APFS**, which is the supported profile:

| envelope | host | `ProcessCrash` | `Fsync` | ratio |
| --- | --- | --- | --- | --- |
| 4 KiB | M4 Max, macOS | 421.6 µs | 7.88 ms | 18.7× |
| 26 KiB | M4 Max, macOS | 575.7 µs | 8.09 ms | 14.1× |
| 256 KiB | M4 Max, macOS | 652.7 µs | 8.70 ms | 13.3× |
| 4 KiB | M3 Pro, macOS 27.0 | 474.3 µs | 5.54 ms | 11.7× |
| 26 KiB | M3 Pro, macOS 27.0 | 517.8 µs | 3.43 ms | 6.6× |
| 256 KiB | M3 Pro, macOS 27.0 | 670.8 µs | 4.39 ms | 6.5× |

Reproduce with `cargo nextest run -p tiler-cache -E 'test(hot_path_publication_by_durability)' --no-capture`. Both hosts run APFS on an internal NVMe volume; the M3 root is `apfs, sealed, local, read-only, journaled` with the cache under the writable data volume.

**Fact — the cost is flat in the payload and therefore is not the bytes.** `ProcessCrash` grows from ~0.42 ms to ~0.65 ms across a 64× increase in envelope size, while `Fsync` stays within its own noise band across the same range. What `Fsync` buys is a fixed number of synchronization round-trips per publication, and those dominate everything else a publication does.

**Fact — the two hosts differ in magnitude and agree in shape.** The ratio spans 6.5× to 18.7×, so the exact multiplier is a property of a host rather than of the policy. The ordering, the size-independence, and the order of magnitude reproduce on both.

## The measurement this deliberately does not make

**Survival is not measured, and on this platform the timing above cannot stand in for it.** Darwin's `fsync(2)` documents that data may remain in a device's volatile cache, so `Fsync` requests persistence through the operating system's APIs and does not claim a drive-cache flush. Establishing power-loss survival would need `F_FULLFSYNC` — a different call this policy does not make — and a way to cut power to the device mid-publication. Neither is available to a test on the supported development host.

So the trade is stated with one side measured and the other bounded by what the platform documents: `Fsync` costs 6.5× to 18.7× per publication and buys persistence across an *operating-system* crash, not across power loss.

## Decision

`ProcessCrash` stays the default, and the reason is now the measurement plus a property of what the cache is.

**Inference — the worst outcome the weaker policy admits is duplicate work.** `docs/artifact-abi.md` records that every expansion-cache failure resolves to a miss, a reported unavailability, an unpublished result, or repeated work; complete identity, immutable final entries, validation on every hit, and atomic publication do not rest on the filesystem. An entry lost to an operating-system crash is therefore a recompile, never an incorrect artifact. Paying an order of magnitude on *every* publication to avoid recompiling the entries published shortly before an OS crash is a poor trade for an optional accelerator, and it is a worse one given the policy does not extend to power loss anyway.

`Fsync` remains implemented and selectable through `ExpansionCache::with_durability`. This decision fixes the default; it does not withdraw the choice from a caller whose deployment makes the trade differently.

## Consequences

- The default is now backed by evidence, and `Durability::ProcessCrash`'s documentation no longer has to say the choice is unmeasured.
- The fourth follow-up gate of the crash-and-race note is closed.
- **Reopening trigger:** a host or filesystem profile where the ratio is small enough that the trade inverts, or a deployment that needs power-loss survival — which would need `F_FULLFSYNC` rather than a change of default, and is a different decision.

---
id: measure-expansion-cache-durability-policies
title: Measure expansion cache durability policies
status: done
priority: p2
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [research/cache, implementation/cache, contracts/decisions, contracts/artifacts]
shared_scopes: [contracts/navigation]
paths: []
tags: [cache, durability, measurement]
---
The research note's fourth follow-up gate: "Measure cache latency and survival for `process-crash` versus `fsync`; only then decide the default in an ADR."

Both policies are implemented in `tiler_cache::expansion::Durability`, and neither changes a published byte — a test asserts that, because a policy that changed the bytes would make an entry written under one unreadable under the other. `ProcessCrash` is the default because ADR 0050 recommends it, **not because it has been measured here**, and the type's own documentation says so.

## What this ticket owes

- Measure both policies on supported macOS host and filesystem profiles at
  realistic bundle sizes. Record hardware, OS, filesystem, mount options,
  storage device, and exact procedure. Other platforms require a support-policy
  change before their measurements become product evidence.
- Measure survival, with the limits stated precisely. `fsync` on Darwin does not claim a drive-cache flush — `fsync(2)` documents that data may remain in a device's volatile cache — so a survival measurement is about the operating system's behaviour and not about power loss, and must not be reported as if it were.
- End in an ADR that fixes the default, or in an explicit statement that the measurement did not distinguish them and the recommendation stands on its original argument.

## Outcome — ADR 0083, and the measurement distinguishes them decisively (2026-07-27)

The ticket's two permitted endings were an ADR fixing the default or a statement that the measurement did not distinguish the policies. It distinguishes them by an order of magnitude, so: **[ADR 0083](../docs/decisions/0083-keep-process-crash-as-the-default-cache-durability.md), accepted, `ProcessCrash` stays the default** — now on evidence rather than only on ADR 0050's recommendation.

**Measured on two hosts, both local APFS, the supported profile.** `Fsync` costs **6.5× to 18.7×** more per publication:

| envelope | M4 Max | M3 Pro (macOS 27.0) |
| --- | --- | --- |
| 4 KiB | 421.6 µs → 7.88 ms | 474.3 µs → 5.54 ms |
| 26 KiB | 575.7 µs → 8.09 ms | 517.8 µs → 3.43 ms |
| 256 KiB | 652.7 µs → 8.70 ms | 670.8 µs → 4.39 ms |

**The finding is the flatness, not the ratio.** `ProcessCrash` grows from ~0.42 ms to ~0.65 ms across a 64× increase in envelope size while `Fsync` stays inside its own noise band, so what `Fsync` buys is a fixed number of synchronization round-trips rather than anything proportional to the payload. The two hosts differ in magnitude and agree in shape, which says the exact multiplier is a property of a host and the ordering is not.

**Survival is not measured, and the ticket required that limit be stated precisely rather than implied.** Darwin's `fsync(2)` documents that data may remain in a device's volatile cache, so `Fsync` requests persistence through the operating system's APIs and claims no drive-cache flush. Establishing power-loss survival needs `F_FULLFSYNC` — a call this policy does not make — plus a way to cut power mid-publication, and neither is available to a test on the supported development host. The trade therefore has one side measured and the other bounded by what the platform documents.

**The decision rests on what the cache is, not only on the ratio.** `docs/artifact-abi.md` records that every expansion-cache failure resolves to a miss, a reported unavailability, an unpublished result, or repeated work — complete identity, immutable entries, validation on every hit, and atomic publication do not rest on the filesystem. An entry lost to an operating-system crash is a recompile, never an incorrect artifact. Paying an order of magnitude on every publication to avoid recompiling entries published shortly before an OS crash is a poor trade for an optional accelerator, and worse still given the policy does not extend to power loss.

`Fsync` stays implemented and selectable through `with_durability`; this fixes the default without withdrawing the choice.

### The harness

`hot_path_publication_by_durability`. Each round publishes into a fresh cache under a fresh scratch root, because a second publication of the same subject is a hit — and the test **asserts `published: true` on every round** rather than assuming it. Without that, a fixture bug would have every row timing a read, and reads are unaffected by durability, so all six numbers would agree for a reason unrelated to the policy.

### Propagated

`Durability::ProcessCrash`'s documentation no longer says the default is unmeasured; the crash-and-race note's fourth follow-up gate is marked closed with its result; and ADR 0083 is in both catalog blocks of `docs/decisions/README.md`.

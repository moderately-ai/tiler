---
id: measure-expansion-cache-durability-policies
title: Measure expansion cache durability policies
status: in-progress
priority: p2
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [research/cache, implementation/cache, contracts/decisions, contracts/artifacts]
shared_scopes: [contracts/navigation]
paths: []
tags: [cache, durability, measurement]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785194688
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

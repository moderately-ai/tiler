---
id: measure-the-expansion-cache-hot-path-efficiency
title: Measure the expansion cache hot-path efficiency
status: in-progress
priority: p2
dependencies: []
related: [decide-the-expansion-cache-collection-schedule, exercise-the-expansion-cache-under-cargo-and-rust-analyzer, admit-an-age-bounded-automatic-eviction-into-the-expansion-cache]
scopes: [research/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, performance, measurement]
claimed_from: todo
assignee: agent-cache-measure
lease_expires_at: 1785877012
---
## User-visible outcome

A bounded measurement of what the expansion cache actually costs on its hot paths, so "the cache is properly efficient" is a supported claim or a located defect rather than an assumption. Requested by Tom 2026-08-04 alongside the eviction decision.

## Measurement boundary

Follow the performance loop in AGENTS.md: correctness oracle first, workload defined, warm-up and repetitions stated, environment recorded (note the toolchain moved to Xcode 27.0 beta / Metal Toolchain 27A5228f on 2026-08-04 — earlier retained cache measurements cite the prior environment).

Measure, on the measured real process patterns (one `rustc` per crate; one long-lived analyzer server):

- `get_or_publish` hit latency: lock acquisition, validation read, and the bytes-to-caller copy, at realistic entry sizes (the retained 32–48 KB envelopes) and at populated shard counts.
- Publish latency off the hit path, including atomic-publication rename cost.
- Shard layout behaviour as entry count grows: whether any hot-path operation degrades with total cache size (it should not — a hit should touch one key path; verify, don't assume).
- Validation-on-every-hit cost, since the cache contract requires it — quantify what fail-closed integrity costs per hit.
- The eviction pass cost (once the eviction ticket lands): full-scan cost at realistic populations, to size the amortization the wiring ticket needs.

## What this ticket does not do

No optimization in this ticket. If a dominant cost is found, file the narrowest change as its own ticket with the measurement attached; an optimization that weakens validation is a defect by contract. Empirical results qualify this host's bounded profile only.

## Closes when

The measurements exist under `spikes/` with their procedure and environment, the research record states which costs dominate with evidence, and either "efficient at the measured scales" is supported with its boundary stated or the located inefficiency has its own narrow ticket.

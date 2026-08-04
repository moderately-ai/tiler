---
id: measure-the-expansion-cache-hot-path-efficiency
title: Measure the expansion cache hot-path efficiency
status: done
priority: p2
dependencies: []
related: [decide-the-expansion-cache-collection-schedule, exercise-the-expansion-cache-under-cargo-and-rust-analyzer, admit-an-age-bounded-automatic-eviction-into-the-expansion-cache]
scopes: [research/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, performance, measurement]
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

## Outcome

**Efficient at the measured scales, supported.** `spikes/cache/hot-path-efficiency/` drives the public `ExpansionCache` with the real `decode_artifact` validator, and [`docs/research/cache/hot-path-efficiency.md`](../docs/research/cache/hot-path-efficiency.md) carries the derivation. Two runs at this commit are retained under the spike's `results/`.

Headline, Apple M4 Max, macOS 27.0, release, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, minimum of 8,000 samples unless stated, reproduced by a second run at the same commit:

- **Hit: 55.5 µs (32,136 B) and 67.2 µs (47,803 B), flat from 10 to 10,000 stored entries** — the four population minima span 0.6% and 0.3%. The build closure it spares is 3.6–5.4 ms, so a hit is 65–97× cheaper than producing the artifact, with no external backend compiler in the comparison.
- **Fail-closed integrity is 73.5–79.3% of a hit** — `decode_artifact` 54.0–55.3%, bundle section digests 19.4–24.0%. The file read is 18.0–23.1% and syscall-bound; keying and path work are under 1%; the residual is 0.3–3.3%.
- **No lock, no copy, no population term on the hit path**, each measured rather than read off the source. A *separate process* held the probe key's lock — proven held by a refused non-blocking acquisition — and the hit was served at the uncontended latency to within 0.2%; had the read taken the lock the run would have deadlocked, not slowed.
- **Publication is 543–601 µs by default and 8.04–8.36 ms under `Fsync`** (13.4–15.1×, flat in payload, inside ADR 0083's band), with the atomic rename 11–17% of a default publication.
- **The eviction full scan is 29.1–32.6 ms at 10,000 entries — 460–590× one hit** — with a marginal cost of 2.5–2.8 µs per entry once the 256 shards saturate, and 73–86 µs per entry actually removed.

**The named open question is answered.** [The collection design](../docs/research/cache/bounded-collection.md) states that whether the per-eviction scan can run on a continuously expanding `rust-analyzer` server "is a measurement this note does not have". It cannot run per expansion. `wire-the-env-configured-eviction-policy-through-the-deliver-path` already requires its trigger to amortize; the number it has to be sized against is now recorded. That ticket is in progress under another worker and its body is not edited from here.

**No inefficiency inside `tiler-cache` was located, and one narrow question was filed rather than acted on:** `decide-whether-the-bundle-envelope-section-digest-is-redundant`, carrying the 19.4–24.0% measurement and the three things — coverage, typed-reason classification, and the ADR 0050 sentence — that have to be established before that digest could retire.

Also filed: `restore-the-cache-build-tool-exercise-against-the-current-artifact-api` (a retained spike stopped compiling against two artifact-API drifts) and `catalog-the-cache-hot-path-efficiency-records` (both catalog entries live in `contracts/navigation`, which this branch does not hold). `bounded-collection.md`'s racing-a-reader position had its copy wording sharpened in place.

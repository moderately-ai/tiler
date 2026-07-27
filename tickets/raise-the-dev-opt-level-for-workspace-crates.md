---
id: raise-the-dev-opt-level-for-workspace-crates
title: Decide the dev optimization level for workspace crates
status: in-progress
priority: p2
dependencies: []
related: [audit-the-suite-s-slowest-tests]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [tooling, performance]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785195539
---
Filed by `audit-the-suite-s-slowest-tests`. Workspace crates compile unoptimized in the dev profile, and the test suite pays for it.

## Fact

`Cargo.toml` sets `[profile.dev.package."*"] opt-level = 1`. Cargo's `"*"` package glob matches **dependencies**, not workspace members, so every `tiler-*` crate compiles at `opt-level = 0` for `make check`.

## Measurement — Apple M4 Max, the same test in both profiles

`decode` of a 26,126-byte artifact envelope:

| | dev | release | ratio |
| --- | --- | --- | --- |
| valid envelope | 2.878 ms | 531 µs | 5.4× |
| damaged envelope | 962 µs | 182 µs | 5.3× |

The workload is digest-dominated, which is the shape that suffers most at `opt-level = 0`. `single_byte_corruptions_are_rejected` performed 8,451 such decodes: 13.0s in dev against a projected ~1.5s at release speed. **Superseded as motivation on 2026-07-27:** that sweep is now exhaustive over every byte and runs in **132 ms**, because the decode path fell from 662 µs to 18.7 µs and the envelope from 26,126 bytes to 15,030. This ticket's question stands on its own merits; it no longer has this test as its headline case.

**Inference.** A meaningful part of the suite's wall clock is unoptimized workspace code rather than the work the tests describe. The effect is not confined to that one test — every digest, encode, and identity derivation in the suite pays the same multiplier — though that test is where it is most visible.

## The trade, which is why this is a ticket and not an edit

`opt-level = 1` on workspace members costs compile time on exactly the code being edited, which is the code recompiled most often. The global Rust hygiene rules this repository inherits set `opt-level = 1` for dependencies deliberately and say not to regress it; extending it to members is additive to that rule rather than a reversal, but it is a different trade — dependencies are compiled once and members on every keystroke-to-`make check` cycle.

Measure both halves before choosing: the suite's wall clock at `opt-level = 1` for members, and the incremental rebuild time of a one-line change in a member crate. A middle option exists and should be compared rather than skipped — raising it for the few digest-heavy crates by name (`tiler-artifact` is the demonstrated one) instead of all members.

## Closes when

The dev-profile optimization level for workspace members is decided with both the suite wall clock and the incremental rebuild cost measured; the choice and its measurements are recorded at the profile; and if the answer is to leave it at zero, that is recorded too so it is not re-measured next time somebody notices a slow test.

## Outcome — raised to 1 for members, 2026-07-27

**Measurement.** Apple M4 Max, interleaved A/B, min of 4 rounds. A "rebuild every member" round is `touch`ing every `lib.rs` and timing `cargo nextest run --workspace --no-run`; toggling the profile alone does not measure this, because Cargo keeps a separate fingerprint per profile and simply reuses the cached artifacts.

| | `opt-level = 0` | `opt-level = 1` |
| --- | --- | --- |
| rebuild every member | 5.52 s | 5.22 s |
| rebuild one changed file | 2.46 s | 2.27 s |
| `cargo nextest` suite | 4.11 s | 4.06 s |
| dev-profile `decode` | 40.9 µs | 18.3 µs |

**The trade this ticket was filed to weigh does not exist.** Its premise was that members are "recompiled on every keystroke-to-`make check` cycle", so optimizing them buys suite time at the cost of rebuild time. Both halves were measured and the cost half is empty: level 1 is LLVM's cheapest real tier and it removes more downstream work — less IR through later passes, less code to link — than it adds. Every build measurement came out flat to slightly faster. Meanwhile dev-profile code runs about 2.2× faster.

**The suite does not move, and that is the superseded-motivation note above coming true.** At ~4 s for 1,013 tests the suite is bound by process spawn and harness overhead, not by the code under test; the compute that used to dominate it is gone. So the change was taken for the execution speedup, not for the suite — and anyone who reads a flat suite number as "this did nothing" should read the `decode` row.

**The middle option is eliminated, not skipped.** Raising it for named digest-heavy crates exists solely to dodge a build cost. There is no build cost to dodge, so it buys nothing over the workspace-wide setting and costs a hand-maintained package list that goes stale the moment a new crate is digest-heavy.

**Nothing about dev-profile safety is traded.** `cargo build -Z unstable-options --unit-graph` confirms `debug_assertions=true`, `overflow_checks=true`, and `debuginfo=line-tables-only` are all unchanged at level 1, and Rust never enables fast-math, so no numerical result moves. Variable inspection was already off via `line-tables-only`, so the debugger cost was paid before this change.

Recorded at the profile in `Cargo.toml` and in the `AGENTS.md` dev-profile bullet, which previously said level 1 applied "for dependencies" and was true only because members had been missed.

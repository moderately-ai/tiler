---
id: raise-the-dev-opt-level-for-workspace-crates
title: Decide the dev optimization level for workspace crates
status: todo
priority: p2
dependencies: []
related: [audit-the-suite-s-slowest-tests]
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [tooling, performance]
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

The workload is digest-dominated, which is the shape that suffers most at `opt-level = 0`. `single_byte_corruptions_are_rejected` performs 8,451 such decodes: 13.0s in dev against a projected ~1.5s at release speed.

**Inference.** A meaningful part of the suite's wall clock is unoptimized workspace code rather than the work the tests describe. The effect is not confined to that one test — every digest, encode, and identity derivation in the suite pays the same multiplier — though that test is where it is most visible.

## The trade, which is why this is a ticket and not an edit

`opt-level = 1` on workspace members costs compile time on exactly the code being edited, which is the code recompiled most often. The global Rust hygiene rules this repository inherits set `opt-level = 1` for dependencies deliberately and say not to regress it; extending it to members is additive to that rule rather than a reversal, but it is a different trade — dependencies are compiled once and members on every keystroke-to-`make check` cycle.

Measure both halves before choosing: the suite's wall clock at `opt-level = 1` for members, and the incremental rebuild time of a one-line change in a member crate. A middle option exists and should be compared rather than skipped — raising it for the few digest-heavy crates by name (`tiler-artifact` is the demonstrated one) instead of all members.

## Closes when

The dev-profile optimization level for workspace members is decided with both the suite wall clock and the incremental rebuild cost measured; the choice and its measurements are recorded at the profile; and if the answer is to leave it at zero, that is recorded too so it is not re-measured next time somebody notices a slow test.

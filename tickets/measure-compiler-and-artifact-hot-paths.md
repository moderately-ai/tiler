---
id: measure-compiler-and-artifact-hot-paths
title: Measure the compiler and artifact hot paths, and guard the work counts
status: todo
priority: p1
dependencies: []
related: []
scopes: [research/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [performance, testing]
---
Phase 0 of the infrastructure performance programme. Everything after this is verified against it, so it lands first.

## Why this is first

Every later phase claims a reduction. Without a repeatable measurement those are assertions, and without a work-count guard each reduction silently erodes the first time somebody adds a call site.

## Measured baseline to reproduce — Apple M4 Max

| | value |
| --- | --- |
| Compile, 5-operation program | 3.5 ms release / 14 ms dev, **flat across every tensor shape** |
| — request construction + verification | ~10 µs (0.3%) |
| — planning | >99%, ~35 µs per recorded decision, 2 alternatives, ~100 explain records |
| Artifact decode, 26,126-byte envelope | 548 µs; the re-encode within it 274 µs (**50%**) |
| Kernel-program identity, 5-operation program | 13,623 bytes (materialized), 8,241 (fused) |
| Request-subject rebuilds per compile | 55 |
| Dev vs release | 4.3x compiler, 5.4x codec |

Compile time being flat across shapes is the important shape of the result: the cost is fixed per compilation and independent of problem size, so it is structural rather than data-driven.

## What to build

**No third-party benchmark dependency.** `tiler-artifact` deliberately has none and `unsafe_code = "forbid"` rules out the intrinsics an audited crate would bring. A timing harness in the repository's existing measurement idiom is sufficient — the numbers above were taken with `std::time::Instant` in a `#[cfg(test)]` module.

Cover: full compile, artifact decode, and a cache hit.

**Work-count guards.** A `#[cfg(test)]` `AtomicUsize` counter incremented at the operation, and a test asserting the count for one compile or one decode. The 55-rebuild figure above was measured exactly this way. Each memoisation in Phase 1 lands with its guard.

These are cheap, they document the intended structure, and they turn performance into a checked property rather than one that erodes unobserved.

## Closes when

Compile, decode, and cache-hit are reproducibly measurable; the work-count guard pattern exists with at least one guard in place; and the baseline above is recorded where the later phases can compare against it.

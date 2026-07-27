---
id: measure-compiler-and-artifact-hot-paths
title: Measure the compiler and artifact hot paths, and guard the work counts
status: done
priority: p1
dependencies: []
related: []
scopes: [research/workspace, implementation/compiler, implementation/artifact, implementation/cache]
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

## Outcome

Done. Measurement and work-count guards exist for all three paths; every later phase is verified against them.

**What landed.** `crates/tiler-compiler/src/hot_path.rs` (compile time by shape, planning volume, the subject-rebuild ratchet), measurements in `tiler-artifact`'s codec tests (decode, re-encode share, identity size), and one in `tiler-cache` (hit cost). All `#[cfg(test)]`, no third-party dependency — the crate that matters most, `tiler-artifact`, deliberately has none, and `unsafe_code = "forbid"` rules out the intrinsics an audited benchmark crate would bring anyway.

**Reproduce:** `cargo nextest run --release -p <crate> -E 'test(hot_path)' --no-capture`. Release matters — workspace crates build at `opt-level = 0` and the paths measure 4–5× slower in dev.

**Measured on Apple M4 Max, release:**

| | value |
| --- | --- |
| Compile, 5-operation program | 3.3–3.6 ms, **flat at 4×3, 1024×3, 4×1024** |
| Planning volume | 2 alternatives, 100 explain records, 26,500 explain bytes |
| Request-subject rebuilds per compile | **57** |
| Artifact decode, 26,126-byte envelope | 662 µs; re-encode alone 328 µs (**49.5 %**) |
| Artifact identity | **13,320 bytes for a 26,126-byte envelope** — over half the artifact |
| Cache hit, protocol only | 18.5 µs |

Compile time being flat across shapes is the finding rather than the absolute number: the cost is fixed per compilation and independent of problem size, so it is structural.

**Two honesty notes recorded at the sites, because both numbers are easy to misread.**

The cache-hit figure is the *protocol alone*. These fixtures store a short byte string under `any_payload`, which accepts anything, so no artifact decode runs. A production hit pins the validator to `decode_artifact` and adds the hundreds of microseconds measured in `tiler-artifact`. The test says so, so 18.5 µs cannot be quoted as "a hit is cheap".

The subject-rebuild count came out at 57, not the 55 measured during the investigation. The difference is the public `CompileRequest` path landed since. Recorded as measured rather than reconciled to the earlier figure.

**The guard is a ratchet, deliberately.** `the_request_subject_rebuild_count_does_not_regress` asserts against the *measured current* count rather than a target, because a red test cannot land. `store-the-verified-request-subject-instead-of-rebuilding-it` lowers the count and tightens the bound in the same change. Until then it catches a new call site adding a rebuild, which is worth knowing immediately given the existing ones sit inside per-region loops.

**Why counts and not timings.** The measurements print and assert nothing about duration: a timing assertion fails on a loaded machine and passes on a fast one, which is a flake rather than a guard. The cost being removed is duplicated work, not slow work, and a count is stable across hosts and profiles — so it can live in the ordinary gate and fail loudly when a memoisation regresses.

Gate: `make full` green (981 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck).

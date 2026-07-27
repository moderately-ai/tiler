---
id: derive-region-formation-once-per-target-compile
title: Derive region formation once per target compile
status: todo
priority: p1
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [performance, compiler]
---
The single largest performance item in the infrastructure, by orders of magnitude.

## Fact

`form_region_candidates` runs a whole-program Weisfeiler-Leman canonicalisation, `N` singleton `form_candidate` calls, and up to `region_expansions` = 10,000 growth calls, each doing a connectivity BFS, a convexity BFS, and content and occurrence encoding.

It is re-derived, unmemoized, at every one of these per target compile:

| Site | Multiplicity |
| --- | --- |
| `pipeline.rs:770` | 1 |
| `cover.rs:509` (inside `enumerate_covers`) | 1 — re-derives what `pipeline.rs:770` just built and discarded |
| `cover.rs:601` via `selection.rs:841` (`verify_cover`) | once per cover, up to 1,024 |
| `selection.rs:905` (`verify_selected_plan`) | once per retained plan |
| `pipeline.rs:2579` (`verify_equivalence`) | once per alternative |

`grep -n 'cache\|memo\|OnceLock' region.rs cover.rs` returns nothing.

Total is roughly `(C + P + A + 2) x Θ(10,000 x form_candidate)` — on the order of 10^7 candidate formations per target where 10^4 suffice.

`derive_fusion_legality` (`fusion_legality.rs:775`) rebuilds the whole region graph per call, and `verify_fusion_legality` (`:829`) re-derives the entire proof just to compare it.

## Why this costs nothing in correctness

The formation outcome is a **pure function of `(program, budgets, contract)`**, and all three are fixed for the duration of a target compile. Computing it once and threading it through is semantically identical to recomputing it, so nothing currently verified stops being verified. The verify-by-reconstruction discipline is kept; only its repetition is removed.

## Scope

Thread one `RegionFormationOutcome` through the target compile, reaching `enumerate_covers`, `verify_cover`, `verify_selected_plan`, `verify_equivalence`, and the fusion-legality derivation.

Prefer threading a value over an interior-mutability cache: it makes the "one formation per target" invariant structural and visible in the signatures rather than a property a reader must trust.

## Closes when

One target compile performs exactly one region formation, pinned by a work-count guard; compile time is measured before and after; artifact identity for the serial-sum program is byte-identical; and `make full` passes.

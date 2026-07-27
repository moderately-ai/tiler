---
id: derive-region-formation-once-per-target-compile
title: Derive region formation once per target compile
status: done
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

## Outcome

Done. **One region formation per target compile, pinned by a guard, and compile time fell from 3.5 ms to 1.4 ms — a 2.5× reduction.**

| | before | after |
| --- | --- | --- |
| Region formations per compile | 5 sites, unmemoized | **1** |
| Compile, 5-op program, release | 3.5 ms | **1.4 ms** |

Measured with the harness `measure-compiler-and-artifact-hot-paths` landed, at 4×3, 1024×3 and 4×1024 — still flat across shapes, as expected, because the cost was never shape-driven.

**What changed.** `enumerate_covers` and `verify_cover` now take `&RegionFormationOutcome` instead of deriving it, and `verify_cover` drops the `budgets` and `contract` parameters it only forwarded to the derivation. The value threads from the authoritative site in `compile_target_with_explain` through `enumerate_complete_plans`, `select_physical_plans`, `verify_selected_plan`, `verify_selected_portfolio`, `verify_portfolio`, `verify_alternative`, and `verify_equivalence`.

The clearest instance of the waste: `enumerate_complete_plans` **already received the formation as a parameter**, and called `enumerate_covers` one line later, which re-derived it.

**Nothing verified stopped being verified.** The outcome is a pure function of `(program, budgets, contract)`, all fixed for a target compile, so computing it once and threading it is semantically identical to recomputing it. The verify-by-reconstruction discipline is intact; only its repetition is gone. Artifact identity for the serial-sum program is unchanged, which the producer's determinism test and the 30-case hardware matrix both pin.

**The maintainability half, which is why this shape was chosen over an interior cache.** Every stale call site became a *type error*, so none could be missed — the compiler enumerated all 30, production and test. And after the change, "the formation is derived once per target compile" is a property of the signatures: a new call site that wanted its own would have to call `form_region_candidates` explicitly rather than get one by default. An interior-mutability cache would have bought the same speed and left the invariant invisible.

**Two of the five sites were not in the original list**, found by the guard rather than by reading: the count sat at 3 after the first pass, and `verify_equivalence` (reached through `verify_alternative`) was the remaining pair. That is the guard doing its job on its first day.

`one_compile_derives_the_region_formation_once` asserts equality with 1, not a bound — there is no legitimate second derivation.

Gate: `make full` green (982 nextest + 11 doc-tests, rustdoc, release numerical tests, `tkt lint`, shellcheck).

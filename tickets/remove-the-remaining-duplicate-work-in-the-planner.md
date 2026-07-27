---
id: remove-the-remaining-duplicate-work-in-the-planner
title: Remove the remaining duplicate work in the planner
status: in-progress
priority: p2
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [performance, compiler]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785179841
---
The Phase 1 remainder, after region formation and the request subject are dealt with. Each is a pure function recomputed; none changes semantics.

## Facts

**`enumerate_frontier` runs per (cover, region) for ~3 distinct inputs.** `pipeline.rs:1035`, `C x R` calls. The code proves the redundancy itself: `pipeline.rs:1048` dedups the explain record by `region_role`, and `region_role` (`pipeline.rs:708`) returns one of four `&'static str` values. Memoize on the key already computed one line later.

**Every alternative is built, then rebuilt identically to compare.** `build_alternative` (`pipeline.rs:1228`) produces the scheduled regions, kernels, program, and artifact plan; `verify_alternative` (`pipeline.rs:2469`) recomputes all of it and compares. Exactly 2x per alternative with no shared intermediate.

**`verify_request` runs twice per compilation** (`pipeline.rs:365`, `:393`), redoing whole-program recognition and per-target contract resolution. `pipeline.rs:362-363` clones `target_profiles` and `capabilities` unconditionally before knowing whether normalisation committed a rewrite — dead in the common case, where `normalized == None` at `:381`.

**The Pareto scan is O(P^2) and runs twice** — `selection.rs:492` via `pipeline.rs:829` and again at `pipeline.rs:2448`. Each allocates a `String` per plan via `label()` for comparisons that could use raw identity bytes. Same pattern at `pipeline.rs:1090`/`:1103-1106`, which is O(refused x P) FNV digests and `String` allocations.

**`encode_record(&record).len()`** (`explain.rs:1078`) builds the full canonical encoding of every explain record solely to measure its length, then drops it. `explain.rs:934` does the same for the trace header.

**`RecognizedSerialSumMembers::all()`** (`request.rs:632`) allocates, chains, sorts and dedups on every call, from `pipeline.rs:718`, `frontier.rs:1352`, `lowering.rs:285` and `:320`.

**`assemble_cover` runs before the dedup check** (`cover.rs:787`), so every duplicate partition pays full assembly. **`derive_materializations`** (`cover.rs:905`) allocates two `Vec<u8>` inside the sort comparator. **`singleton_occurrence_identity`** (`lowering.rs:360`) linear-scans all candidates per member.

## Closes when

Each item above is removed or justified in place; compile time is measured before and after; work-count guards cover the memoisations; artifact identity is byte-identical; `make full` passes.

## Outcome — the profile reprioritized this ticket (2026-07-27)

**Measurement.** A sampling profile of the compile loop, recorded with `samply --rate 4000 --unstable-presymbolicate` against a `CARGO_PROFILE_RELEASE_DEBUG=true` build and cross-checked against macOS `sample`. The harness is `hot_path_profile_loop` in `crates/tiler-compiler/src/hot_path.rs`, which documents the exact recording commands. Active self time, excluding the parked test-harness thread:

| | share of active self time |
| --- | --- |
| `_platform_memmove` | 13.6% |
| allocator (`malloc`/`free`/`realloc`) | 12.7% |
| **`region::canonical_member_order`** | **10.6%** |
| `_platform_memcmp` | 8.5% |
| `region::form_candidate` | 2.3% |
| `pipeline::verify_portfolio` | 1.0% |
| `pipeline::select_non_dominated` | 0.7% |
| `explain::encode_record` | 0.3% |

**Fact: this ticket's own item list was written from code reading, and the profile does not support its ordering.** The four items it names most prominently — the double alternative build, the twice-run Pareto scan, `encode_record().len()`, and `derive_materializations` — sum to under 2.5% of active self time. The single hottest function in the crate is not mentioned in the ticket at all.

**Fact: `RegionGraph::from_program` ran 15 times per compile.** Counted by `REGION_GRAPH_BUILDS`, not inferred. `derive_fusion_legality` (`fusion_legality.rs`) built its own whole-program graph per candidate, and `RegionGraph::from_program` ends by running `canonical_member_order` over every operation — a colour refinement that rebuilds and re-digests a byte buffer per member per round, so it is quadratic in the program and is the source of much of the `memmove`/allocator/`memcmp` traffic above it in the table.

**Inference: this is the same defect [`enumerate_covers`](../crates/tiler-compiler/src/cover.rs) already had.** `RegionFormationOutcome` owns the graph, every caller holds the formation, and the pipeline's own call site was already reading `formation.candidates()` two lines above the derivation that rebuilt the graph. `derive_fusion_legality` and `verify_fusion_legality` now take `&RegionFormationOutcome` and use `formation.graph()`, with the same "taken rather than derived" rationale recorded on the signature.

**Measurement: 1.08 ms → 0.95 ms per compile, ~12%.** Minimum of 200 compiles, three runs each, before and after, on a quiet host. Graph builds per compile 15 → 1, pinned by `one_compile_builds_the_region_graph_once`.

**The measurement harness was wrong and is fixed.** It reported the mean of five compiles. The first reading of this change was 1.96 ms against a 1.49 ms baseline — an apparent 30% regression — and three reruns read 1.16–1.30 ms. Host noise only ever makes a compile slower, so the distribution has a hard floor and an unbounded tail; the harness now reports the minimum of 200 runs beside the mean. Every earlier number in this programme was a mean of five and is not comparable to a number produced after this change.

## Remaining

Every item in the Facts section above is untouched. They are still real duplicated work and still worth removing, but the profile says the whole set is worth a low single-digit percentage, so they should be attacked as maintainability rather than as performance — or reprioritized behind whatever a re-profile now shows at the top. Re-profile before picking the next one; the composition will have shifted.

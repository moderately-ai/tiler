---
id: remove-the-remaining-duplicate-work-in-the-planner
title: Remove the remaining duplicate work in the planner
status: done
priority: p2
dependencies: [measure-compiler-and-artifact-hot-paths]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [performance, compiler]
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

## Outcome — re-profiled, and the premise is largely spent (2026-07-27)

The previous outcome ended "Re-profile before picking the next one; the composition will have shifted." It had. This is that re-profile, plus the two items it justified acting on.

**Measurement.** `samply --rate 4000 --unstable-presymbolicate` against a `CARGO_PROFILE_RELEASE_DEBUG=true` build of `hot_path_profile_loop`, 20 s, 28,032 compiles, analyzed with the quiltdb attribution script. Self time alone is misleading here — it puts `_platform_memmove` and the allocator on top without naming a caller — so each sample was also charged to the nearest enclosing non-generic frame of ours. `RawVecInner::finish_grow` is monomorphized into our binary and masks its caller unless generic frames are skipped explicitly.

Where the active leaves land: **47.1% our own code, 33.3% allocator, 18.0% memmove/memcmp.** Half the compile is allocation and copying, diffused across everything rather than concentrated in one place.

### Every item in the Facts section, with its measured share

| item | inclusive share | disposition |
| --- | --- | --- |
| alternative built then rebuilt to compare (`verify_portfolio`) | **23.26%** | **justified in place — it is the check, not waste** |
| `assemble_cover` before the dedup check | 2.49% | justified — cannot be reordered, see below |
| `SelectedPlanIdentity::label` allocating for comparisons | 2.14% | **partly removed** |
| `derive_materializations` | 2.00% | **claim was stale — already fixed** |
| `enumerate_frontier` per (cover, region) | 1.57% | justified — available, but see the ceiling below |
| `verify_request` twice per compilation | 0.35% | justified — below the noise floor |
| `RecognizedSerialSumMembers::all()` | 0.26% | justified — below the noise floor |
| Pareto scan run twice (`select_non_dominated`) | 0.06% | justified — below the noise floor |
| `encode_record(&record).len()` | 0.00% | justified — never appeared in the profile |
| `singleton_occurrence_identity` | 0.00% | justified — never appeared in the profile |

### The 23% is not duplicate work, and removing it would be a defect

`verify_portfolio` is called unconditionally on the compile path (`pipeline.rs:851`), not behind a debug assertion. Underneath it the profile finds `KernelBuilder::emit`, `resolve_capabilities`, `build_artifact_plan`, `verify_kernel`, `verify_artifact_plan`, `canonical_member_order`, `region::assemble` — it re-derives the whole downstream pipeline and requires it to reproduce the receipt exactly, so a tampered plan, cost, program, or artifact receipt fails closed.

**This is the one item on the list that must not be memoized.** The ticket framed it as "exactly 2× per alternative with no shared intermediate", as though the missing shared intermediate were the defect. The independence *is* the mechanism: a verifier that reuses the value it is checking compares that value to itself and can never say no. Sharing an intermediate here would convert the most expensive part of the compile into a check that always passes, which is worse than the cost it saves. So the largest single cost in the compiler is deliberate, and reducing it is a decision about the verification architecture rather than a performance cleanup.

The corollary is that allocation reductions in the shared building blocks pay twice, once in the build and once in the verify — which is where the two changes below land.

### `assemble_cover` cannot be moved after the dedup

The dedup key *is* the assembled cover identity, and it depends on the sorted regions, the duplication policy, and the derived materializations. There is no cheaper key that decides the same question: the recursion already emits each exact partition once (the anchor rule guarantees it), so the map is not catching repeated partitions — it is catching *distinct* partitions that canonicalize to one cover identity. A candidate-index-set key would therefore retain covers the current code discards, which changes which plans exist and what the budgets count. That is a semantic change, not an optimization.

### What was removed

**`push_slice` reserves once instead of twice** (`tiler-ir/src/identity.rs`). The profile put it at 8.93% of active self time, spread over twenty-odd encoders with no dominant caller — systemic to the primitive, not to any encoder. Its two `extend_from_slice` calls each tested capacity and each could reallocate and move the buffer; one exact `reserve` makes that at most one growth.

**`is_labelled` replaces a formatted-`String` comparison** (`selection.rs`, `pipeline.rs:2537`). The verification pass built a fresh label per alternative purely to compare and drop it. The replacement is exactly equivalent, not looser: `{:016x}` over a `u64` emits exactly sixteen lowercase hex digits, so prefix + length + alphabet + value admits precisely the one string `label` returns. The alphabet check is load-bearing — `u64::from_str_radix` also accepts uppercase, and `is_labelled_admits_only_the_label_it_replaces` was watched failing on `selected-plan:4C9BD785BA1158B3` with that check deleted, so the guard is known reachable rather than assumed.

**Measured: 686.05 µs → 683.31 µs, −0.40%.** M3 Pro, quiet, twelve interleaved pairs, min-of-200 per reading, all twelve in the same clock state. **12/12 pairs favour the candidate** (sign test p ≈ 0.02%), so the direction is solid even though the magnitude is small. An earlier six-pair run read ≈1% only because it compared readings across the host's two clock states (≈640 µs and ≈686 µs); that spread is 7% and swamps the effect, which is why the pairs must land in one state to mean anything.

Artifact identity is byte-identical — the pinned serial-sum identity and the two-process determinism test are unmoved. No memoization was added, so no work-count guard was needed; the one test added guards an equivalence, not a count.

### Why this closes rather than continuing

Everything left on the list is under 2.5% and the largest single entry is a correctness check that must stay. The remaining cost is not duplicated computation any more — it is 51% allocator-and-memmove traffic spread thinly across every encoder and builder, which is a different problem with a different shape: it wants fewer transient buffers, not fewer recomputations. That is filed as `reduce-transient-allocation-in-the-compile-path` rather than left implied here, because attacking it under this ticket's title would keep re-deriving the same profile.

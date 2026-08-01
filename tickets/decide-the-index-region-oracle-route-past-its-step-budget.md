---
id: decide-the-index-region-oracle-route-past-its-step-budget
title: Decide the index-region oracle's route past its evaluation-step budget
status: in-progress
priority: p2
dependencies: []
related: [bound-the-reference-contraction-comparison-for-the-profile-cells, route-the-contraction-conformance-through-the-staged-oracle]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, contraction, language-model]
claimed_from: todo
assignee: worker-idx-oracle
lease_expires_at: 1785603010
---
## User-visible outcome

The emitted index region can be executed against a profile cell larger than `w_decode_kv`, or the reach of the index-region oracle is a stated design limit with its cost — so "the emitted region reproduces a measured device result" stops being a claim about one cell by accident.

## Why the contraction staging did not settle this

**Fact — `bound-the-reference-contraction-comparison-for-the-profile-cells` (2026-08-01).** That ticket staged the *contraction evaluator*'s fold and reached all six L3 cells. It did **not** touch the index-region oracle, and the two bounds are different things:

- `MAX_REFERENCE_TENSOR_ELEMENTS` bounds `output_count * contracted_count`, a product computable from the shapes before a single step. Staging it is arithmetic: partition the output, and each slab's product is known in advance.
- `MAX_EVALUATION_STEPS` (`crates/tiler-reference/src/oracle.rs:49`) is a *running counter*, incremented by `step()` on every scalar evaluation, reducer-body evaluation, and index-expression evaluation in one region evaluation. It is not a product of extents and cannot be predicted from the region's shape without modelling the region's own body.

So `the_index_region_oracle_refuses_the_vocabulary_cell_under_its_step_budget` in `crates/tiler-compiler/src/governed/contraction_conformance.rs` still refuses `w_vocab_slice`'s 8,388,608 contracted points, and every larger cell with it.

## What this must decide, with the elimination stated

Three candidates, and the cheap one is not obviously wrong here:

1. **Partition the region evaluation over its output domain**, the way the contraction staging partitions the fold — but a `VerifiedIndexRegion`'s output domain is a property of the region rather than of a declared index structure, so this needs a public notion of "evaluate this region for this sub-domain" and an argument that the sub-domain does not change any value. That argument is *not* the contraction's: a region may carry reductions whose contributor sequences are region-declared, so the independence has to be re-derived from what the region verifier guarantees.
2. **Carry the counter across calls** so a caller pays in bounded instalments without the oracle needing a domain partition — cheaper, but it turns a per-evaluation budget into caller-held state, and a budget a caller can reset is not a budget.
3. **State the reach as a limit.** The index-region oracle exists to be a second, independent implementation; if the cost of making it reach large cells is a weaker bound or a second independence argument, keeping its reach small and saying so may be the correct answer. If chosen, the refusal's meaning must be documented where the oracle lives rather than only in a consumer's test.

Whichever survives, the measured cost at `w_vocab_slice` and at the largest prefill cell is part of the answer — the contraction evaluator's own six-cell profile costs 10.8 s in the dev profile, and the region oracle is the slower of the two implementations.

## Closes when

The index-region oracle either reaches a stated set of profile cells with its independence argument written where the procedure lives, or its reach is a documented design limit with the cost that decided it — and the consumer test that asserts the refusal cites the decision rather than an open question.

## Outcome

**The evaluation stages over the region's parallel domain. `MAX_EVALUATION_STEPS` does not move by one step, and neither does the whole-region path's behaviour.** `w_vocab_slice`'s region — the cell the direct-path landing observed refused — now reproduces the retained `direct` `result_sha256` through the region oracle's own walk, and the refusal that protected the host is byte-for-byte the one that was there before.

### The elimination

**Candidate A — raise the bound.** Eliminated, and what eliminates it is the property that made this ticket different from the contraction's. `MAX_REFERENCE_TENSOR_ELEMENTS` bounds a product the caller could have computed itself; `MAX_EVALUATION_STEPS` bounds a walk whose cost is a property of the region's own expression graph and reducer body. So there is no caller — not `IndexRegionEvaluator::evaluate` on an arbitrary verified region, which is the caller the bound exists for — that can preflight its own ask. A running bound is the *only* form this protection can take, and its number is the only thing standing between a pathological region and an unbounded host walk. **Measurement:** the one cell that would have justified a raise exhausts the whole budget after 8.66 s of a walk that costs 55.6 s in total, so admitting it in a single span means roughly a sixfold raise — paid by every region nothing has ever measured, to serve one test cell. This is the shape AGENTS.md names: the saved cost *is* the part that made it correct.

**Candidate B — carry the counter across calls.** Eliminated as the ticket suspected, and for a sharper reason than "a budget a caller can reset is not a budget". Resuming mid-walk with a fresh counter needs the interpreter's own position — frames, depth, the point half-evaluated — to survive the call boundary, so the unit a caller pays for is *an arbitrary interior of a point*, which nobody can state and no reviewer can check. The route that survives instead resumes only **between** parallel points, where the unit is a stated set of output positions.

**Candidate C — state the reach as a limit.** Eliminated by B succeeding, but it was live: it was the honest answer if the independence argument had needed a second authority or a weaker check. It needed neither — the argument comes out of proofs the region already carries, and every check the whole-region path applies still applies unchanged. Retained as the fallback had B failed; B did not fail.

**Candidate D — partition the output domain. Survives.** It costs no constant, no new authority, and no weakened check. The whole-region path is now literally *the staged path walked in one span*, so the two cannot drift: `evaluate` is three lines calling `stage`, one unbounded `evaluate_points`, and `finish`.

### The independence citation, from the verifier's own proof

Written where the procedure lives, in `StagedIndexRegionEvaluation`'s rustdoc. Three facts about what a `VerifiedIndexRegion` *proves*, not about the shape of the loop:

- **A write's iteration domain is exactly the region's parallel dimension set.** `crates/tiler-ir/src/index/builder.rs` `prepare_access` refuses any other write domain with `IndexBuildError::InvalidWriteDomain` (`if mode == AccessMode::Write && domain_set != self.parallel_dimensions()`). So "the parallel points the oracle walks" and "the domain each output is written over" are one set rather than two that coincided for the regions tried so far. This is the fact the ticket asked for and it is stronger than expected — it is a construction-time refusal, not a per-region proof obligation.
- **That write is total and injective over that domain.** Every write access retains a `WriteOwnershipProofView`: `CoordinatePermutation`, discharged by `proof.rs`'s `write_is_permutation` — every coordinate *is* a distinct domain dimension whose extent the environment proves equal to the axis it indexes — or `Exhaustive`, discharged by `verify_access_exhaustively`'s bitset walk, which refuses both a repeat (`bits & mask != 0`) and a gap (the trailing `any(... == 0)` scan). Either way the point-to-element map is a bijection, so **a partition of the parallel points is a partition of each output's elements**, and no span can land on an element another span produced.
- **No value in a region can read a boundary the region writes.** The same `prepare_access` refuses it with `IndexBuildError::ReadFromOutput`. Written elements are the only state spans share, and this makes them unobservable to the computation.

The mechanical statement checked *against* those facts, recorded as the code rather than as the argument: `evaluate_point` builds a fresh `Frame` per point, a read resolves only against the immutable bound inputs, and a reduction's state is built from its `init` values inside the point's own frame and folded through a `BodyContext` that does not outlive the point. **The one-read/one-loop shape never enters it**, checked against the direct path's two-read widening: `reduce_step` evaluates every declared contributor, so a fold taking two operand reads per contributor composes exactly as one taking a single read does.

### What was built

`ParallelWalk` (crate-private) is the lexicographic cursor over the parallel space; `RegionEvaluation::evaluate_span` walks up to *n* points from where it stands and is **the only place the step counter is set to zero**, which is what makes "one span, one budget" a property of the code rather than a claim about it. Two callers, and neither carries its own arithmetic:

- `IndexRegionEvaluator::evaluate` — `stage`, one span of `u64::MAX`, `finish`. Same refusals in the same order as before (parallel domain resolved before output planning, both before any point).
- `StagedIndexRegionEvaluation` — the caller's own loop over `evaluate_points`.

The slab entry point mirrors `StagedStrictTensorContractionF32`'s discipline: **no `evaluate()` convenience that walks every remaining point**. The loop is the authorization, and the one-call form already exists under its real name.

Two departures from that precedent, both forced by the difference the ticket identified. The staged type cannot *derive* an admissible span width, because a region's per-point cost is discovered by walking rather than divided out of a step product — so the width is the caller's and the budget is what tells them it was too wide. And a failed span **poisons** the evaluation (sticky failure, the discipline `ScalarReferenceOutputs` already sets in this file), because a point can fail after writing some of the region's outputs; the contraction's slabs return their elements or nothing.

### Executed-cell evidence

`crates/tiler-reference/tests/contraction_profile_cells.rs`, extended rather than duplicated: the operand reconstruction, the retained digests, the SHA-256 helper and its two FIPS 180-4 vectors were already there for the *other* oracle, and both now answer the same six cells. The region is a hand-written mirror of `GovernedStrictTensorContractionF32::lower` for a rank-one contracted space, necessary for the reason `index_region_oracle.rs`'s `governed` module already states — `tiler-reference` is a dependency of `tiler-compiler` and inverting that edge would put the oracle downstream of the compiler.

**Measurement — Apple M4 Max, 2026-08-01, nightly-2026-07-19, dev profile, `cargo nextest run -p tiler-reference --run-ignored only --no-capture -E 'binary(contraction_profile_cells)'`.**

| observation | value |
| --- | --- |
| one span over `w_vocab_slice`'s region | refused at step 16,777,217, after 8.66 s |
| implied region step rate | **516 ns/step**, against the contraction fold's 9 |
| `w_vocab_slice` walked in 16 spans of 512 points | 55.6 s, digest `88b01ae7…` matches |
| whole `#[ignore]`d test | 64.4 s |
| sibling six-cell contraction test, same run | 11.4 s against its recorded 10.8 |

The last row is the calibration: this host reproduced the earlier landing's per-cell times (`w_prefill_mlp_in` 4,019 ms against 3,799), so the two measurements are on one footing rather than one host apart. The refusal's rate is read off an *exact* step count rather than estimated from a guessed per-point cost.

Cost is why it is not gate-resident: 64 s to run, against a default pair that costs under 50 ms. What runs every gate is `span_boundaries_do_not_change_any_region_value` — five partitions of a 33-point region (widths 1, 3, 5, 11, and one wider than the space) agreeing with each other, with the *registered contraction evaluator's* result, and again after one contributing element is advanced by a single representable value — and `an_incomplete_staged_walk_is_refused_rather_than_finished`.

### The protection, watched refusing

Four perturbations, each run against a case that had to fail, each reverted; the final tree carries none of them (`grep -n "if false" crates/tiler-reference/src/oracle.rs` → no matches).

1. **Span-boundary independence.** The per-point `Frame`'s scalar-value memo carried across the points of a span. `span_boundaries_do_not_change_any_region_value` fails at `a span of 3 parallel points changed a committed value`, with every group of three outputs repeating the first point's value — while the span of 1 still passes. That discrimination is the point: only a span wider than one point notices. (The first attempt at this perturbation carried the *whole* frame, and was caught earlier by `DuplicateWrite` instead — the oracle's own coverage check, firing before any value could be compared.)
2. **The step budget still refuses.** `step()`'s comparison disabled: `tiler-compiler`'s gate-resident `the_index_region_oracle_refuses_the_vocabulary_cell_under_its_step_budget` fails after evaluating the whole 8,192-point region in 55.4 s and returning a result where it expected a refusal. That test — unchanged, and on the region the governed lowering emits — is the standing watcher, so this ticket adds no second one.
3. **The incomplete-walk refusal.** `finish`'s exhaustion check disabled: `an_incomplete_staged_walk_is_refused_rather_than_finished` fails with `left: IncompleteWrite { … }, right: IncompleteStagedWalk { evaluated: 30 }` — which also confirms the documented ordering, that a caller who stopped early is named as such before the region is accused of an incomplete write.
4. **The empty-span refusal.** The `points == 0` check disabled: the same test fails with `left: Ok(0), right: Err(EmptyStagedSpan)`.

### Public items, for review — none self-accepted

- `tiler_reference::StagedIndexRegionEvaluation<'a>` — new public struct with `parallel_point_count() -> Option<u64>`, `evaluated_points()`, `is_exhausted()`, `evaluate_points(u64) -> Result<u64, _>`, `finish()`, and a hand-written `Debug` that renders the walk and never the borrowed input tensors. `parallel_point_count` is `Option` deliberately: a parallel extent product past `u64` has no count to report, and a saturated one would be a number a caller could divide by.
- `tiler_reference::IndexRegionEvaluator::stage(&'a self, &'a VerifiedIndexRegion, IndexRegionAuthority<'a>, &[IndexRegionInput<'a>]) -> Result<StagedIndexRegionEvaluation<'a>, IndexRegionEvaluationError>` — new method. `evaluate`'s signature is unchanged.
- Two variants on the `#[non_exhaustive]` `IndexRegionEvaluationError`: `EmptyStagedSpan` and `IncompleteStagedWalk { evaluated: u64 }`. Both are staged-only; the whole-region path cannot reach either.

No existing public signature changed and no constant moved.

### Follow-up filed rather than absorbed

**[`route-the-index-region-conformance-through-the-staged-oracle`](route-the-index-region-conformance-through-the-staged-oracle.md)** — `crates/tiler-compiler/src/governed/contraction_conformance.rs` is now stale in one paragraph. Its "The index-region oracle reaches the smaller of the two" section says the vocabulary cell's refusal is asserted because "raising it belongs to `tiler-reference`, which this work does not own" — true, and it was not raised; but a route past it now exists, so the emitted region can be compared at `w_vocab_slice` and the file's boundary statement should cite this decision rather than an open question. That is `implementation/compiler` work this ticket does not own, and it must decide whether an ~55 s comparison is one that crate wants at all.

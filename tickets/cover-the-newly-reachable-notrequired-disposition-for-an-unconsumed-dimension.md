---
id: cover-the-newly-reachable-notrequired-disposition-for-an-unconsumed-dimension
title: Cover the newly reachable NotRequired disposition for an unconsumed dimension
status: done
priority: p2
dependencies: []
related: [derive-per-locus-numerical-obligations, wire-the-delivered-realization-record-into-the-artifact]
scopes: [implementation/compiler, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, fail-closed, test-coverage]
---
## What changed, and why it needs its own node

[`derive-per-locus-numerical-obligations`](derive-per-locus-numerical-obligations.md) narrowed the delivered-realization producer so an obligation row is emitted only for an occurrence whose operation can actually consume the dimension. That was the point of the ticket. **It also made a disposition reachable that previously could not be**, and its worker flagged this rather than landing it silently.

**Before**, the producer emitted one row per honoured dimension at `PolicyLocus::Computation` of *every* covered occurrence. A dimension therefore always had a non-empty row set, so the artifact builder could never derive `NotRequired` for an honoured dimension. The superseded module comment stated this in terms: over-stating "never under-states it, which is the safe direction … a missing one would let a dimension's disposition be derived as `NotRequired` — the one producer assertion the neutral artifact cannot check."

**After**, a dimension that *no* covered occurrence consumes yields an empty row set and the artifact derives `NotRequired`. The producer's new module header argues this is correct — "a dimension no covered occurrence consumes is genuinely not required by any packaged route, which is the claim `NotRequired` makes" — and the coordinator agrees the reasoning is sound and that the change is **forced rather than chosen**: you cannot emit a founded locus for an occurrence that founds none, so the alternative is the unfounded `Computation` row this ticket exists to remove.

**But it is untested, and it is the one assertion the neutral artifact cannot re-check.** The worker reports that no current fixture reaches it — all four honoured dimensions retain rows in every program in the suite. So a semantically load-bearing path is newly reachable and no test exercises it.

## What this owes

- A program in which some honoured dimension is consumed by **no** covered occurrence, carried through to a packaged artifact, with `NotRequired` asserted as the derived disposition. Name the program and why that dimension is unconsumed.
- The safety direction pinned: the correctness of `NotRequired` rests entirely on `operation_capability(..).can_consume` never returning `false` for an operation that *can* consume the dimension. `policy.rs` states that `operation_capabilities` is written conservatively and that `unrepresentable_dimension` independently refuses any consumable dimension the realization cannot carry. **Turn that from a stated intention into a check** — a capability row that under-claims must be caught, because a false `can_consume` now silently produces a `NotRequired` claim rather than a redundant row.
- The perturbation watched failing: narrow a capability row so a genuinely consumed dimension reports unconsumed, and observe the artifact assert `NotRequired` where it must not. Restore.

## Explicit non-goals

Not a revert of the narrowing — it is correct and forced. Not a change to the locus derivation, the strictness rule, or the founded-locus refusal, all of which landed with their own evidence.

## Worker record, 2026-08-07

**Fact — the program is the bare reduction, and contraction is the dimension.** `input: f32[4, 3]` into one `tiler::strict-serial-sum-f32@1` over axis 1, with no pointwise multiply and no add — `session::tests::bare_reduction_program` and its `[2, 3]` sibling `custom_backend::bare_reduction_program`. The dimension is unconsumed because `policy::operation_capabilities`' `REDUCTION` row omits contraction: a strict serial sum's per-contributor step is `accumulator + contributor`, so there is no product for ADR 0015's fused multiply-add permission to act on. It is the only honoured dimension for which a single-family program can do this — `physical::region_proposal` asks every candidate about exactly `{InputSubnormals, ResultSubnormals, Contraction, Reassociation}`, and the other three are consumed by every arithmetic family this build can plan alone.

**Fact — the dimension is honoured, not merely unasked, and both halves are asserted.** `session::tests::an_honoured_dimension_no_covered_occurrence_consumes_carries_no_row` reads the *retained plan's own* honoured facts and pins the set to those four by name, then reads the delivered-realization view and counts 1 covered occurrence and 3 rows with contraction absent. Either assertion alone would be consistent with a defect: a producer that had dropped the requirement fails the first, and one still emitting an unfounded `Computation` row fails the second. `semantic_program` is compiled beside it under the identical contract and profile and yields 2 contraction rows, so the difference is the program.

**Fact — the artifact derives `NotRequired`, through the non-Metal producer.** `custom_backend::an_unconsumed_honoured_dimension_is_packaged_as_not_required` packages the reduction program through `assemble_plan_artifact`, encodes, decodes, and walks all eleven dimensions: 3 `Required` with one obligation each (`InputSubnormals` at `Input`, `ResultSubnormals` at `Result`, `Reassociation` at `Accumulator`), 8 `NotRequired`, contraction among them. The record still *states* contraction's resolution as `Transform(Forbidden)`, so the disposition is a claim about reliance rather than a silence about the contract. The same backend over `semantic_program` returns `Required` with 2 obligations.

**The safety direction is now a check with an independent oracle.** `policy::tests::an_arithmetic_family_claims_the_whole_arithmetic_core` reads `governed::governed_index_access_capabilities`' `emitted` declaration — the scalar operations each family's *lowering* may apply — and requires that a family emitting any rounding binary32 operation (`multiply-f32`, `add-f32`, `divide-f32`, `exp-f32`, `rsqrt-f32`) consume all six dimensions whose freedoms act on rounding itself, and that a family emitting none consume nothing at all. That declaration is a different statement written for a different purpose and is independently held honest by `legality::refine_index_region`'s containment proof, so a row narrowed to make an obligation disappear has to contradict what the lowering emits. `rounds_binary32` is total over the emitted keys and panics on an unclassified one rather than answering `false`. Populations: 6 rounding families, 4 exact, and the three with no governed lowering — `tiler::softmax-f32@1`, `tiler::assemble-strict-affine@1`, `tiler::quantize-strict-affine@1` — named rather than skipped, their rows pinned by name elsewhere. `Contraction` and `Permutation` stay outside the core because they depend on an operation's structure rather than its rounding; `the_fold_bearing_families_are_exactly_the_reducing_ones` pins the second.

**Watched failing.** Removing `NumericalDimension::Reassociation` from `REDUCTION` — a genuinely consumed dimension made to report unconsumed — produced, verbatim:

```text
thread 'policy::tests::an_arithmetic_family_claims_the_whole_arithmetic_core' panicked at crates/tiler-compiler/src/policy.rs:1687:21:
tiler::strict-serial-sum-f32@1 emits a rounding binary32 operation and must consume numerics.reassociation: a row missing it now yields no obligation, and the artifact asserts `NotRequired` for a dimension the route genuinely relies on
```

```text
thread 'an_unconsumed_honoured_dimension_is_packaged_as_not_required' panicked at crates/tiler-build/tests/custom_backend/main.rs:495:9:
the packaged fold consumes numerics.reassociation, so `NotRequired` here is a false producer assertion the neutral artifact cannot re-check
```

Restored and re-run green: `2 tests run: 2 passed, 740 skipped` for the compiler pair and `1 test run: 1 passed, 87 skipped` for the artifact case.

**No identity moved.** No pinned digest, golden, or artifact identity changed: the delta is three tests, one `#[cfg(test)]` accessor, and this record. `metal_plan.rs` still pins artifact identity `23c46a19…`, cache subject `e89c4d82…`, and 64,542 fixed content bytes, and `make full` gates green on them.

**Scope added.** `implementation/build`, for `crates/tiler-build/tests/custom_backend/main.rs`. The artifact's disposition is *derived* by `tiler-artifact`'s builder from the obligations that arrive and `tiler-compiler` has no artifact edge, so no compiler-side test can carry a program to a packaged artifact; `custom_backend` is the non-Metal producer that already does. `tkt why` reports no conflict with either live sibling claim.

## Graph maintenance

Filed 2026-08-07 by the coordinator at integration of the producing ticket, from a consequence its worker named and asked for explicit sign-off on rather than treating as authorized. Kept separate because it is a test and safety-direction obligation on a behaviour change, not part of the locus derivation itself.

## Outcome — delivered 2026-08-07 at `cae15d26`

**The newly reachable path is now exercised.** A bare reduction — one `tiler::strict-serial-sum-f32@1` over an `f32` input, with the scaling multiply and bias add removed so the fold is the only covered occurrence — leaves **contraction** honoured but consumed by nothing, because `policy.rs`'s `REDUCTION` row omits it: a strict serial sum's per-contributor step is `accumulator + contributor`, with no product for ADR 0015's fused multiply-add permission to act on. That follows from the rows rather than from search, and contraction is the *only* honoured dimension a single-family program can leave unconsumed, since `region_proposal` asks every candidate about exactly four dimensions and the other three are consumed by every arithmetic family this build can plan alone.

The dimension is **honoured, not merely unasked** — the compiler test reads the retained plan's own honoured facts and pins that four-element set by name, so the empty row set cannot be "nobody asked". Carried through to a packaged artifact in `crates/tiler-build/tests/custom_backend/`, which walks all eleven dimensions: **three `Required`** (input subnormals at `Input`, result subnormals at `Result`, reassociation at `Accumulator`) and **eight `NotRequired`** including contraction. The record still *states* contraction's resolution as `Transform(Forbidden)`, so the disposition is a claim about reliance rather than a silence.

**The safety direction is now a check rather than an intention, and this is the part that matters.** An under-claiming capability row used to be harmless — the producer emitted a row per honoured dimension at every occurrence, so a missing entry cost nothing. It now decides whether a row is emitted at all, and a dimension left with no row is derived as `NotRequired`: a positive claim the neutral artifact cannot re-check. **The failure direction inverted when the locus derivation landed**, and `an_arithmetic_family_claims_the_whole_arithmetic_core` is what catches it.

What makes it a check rather than the table restated: the oracle is `governed_index_access_capabilities()`'s `emitted` declaration — the scalar operations each family's *lowering* may apply. That is a different statement written for a different purpose, and `legality::refine_index_region` proves the region a family actually emits is contained in it, so an under-claim cannot be hidden by editing the declaration to match. `rounds_binary32` is total over the emitted keys and **panics on an unclassified one rather than answering `false`**, which is the direction that would otherwise make a family silently look exact. Populations: 6 rounding families, 4 exact, and the 3 with no governed lowering named rather than skipped, so a new one cannot arrive unchecked.

**Watched failing**, by removing `Reassociation` from the `REDUCTION` row: the policy check reports that the family emits a rounding binary32 operation and must consume the dimension, and the build-side test reports that `NotRequired` there would be a false producer assertion the neutral artifact cannot re-check. Both restored green.

**No identity moved.** `metal_plan.rs` still pins `23c46a19…`, `e89c4d82…`, and 64,542 bytes, recomputed green on the merged tree. `make full` exit 0 on the branch and again after merge: 2,953 workspace tests, 1,033 release numerical.

**Scope added:** `implementation/build`. The disposition is derived by `tiler-artifact`'s builder from the obligations that arrive, and `tiler-compiler` has no artifact edge, so no compiler-side test can carry a program to a packaged artifact. `crates/tiler-build/tests/custom_backend/` is the non-Metal producer that already does exactly that, against public surfaces only.

**Remainder.** Three admitted families ship no governed index-access lowering, so the oracle cannot speak for them; their rows stay pinned by name, and if `softmax-f32` gains a lowering the `unlowered` assertion fails and forces the row under the oracle — deliberate.

---
id: register-the-softmax-realization-law
title: Register the softmax realization law
status: done
priority: p1
dependencies: []
related: [admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold, admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence, widen-the-staged-realization-law-to-the-registered-elementary-families, admit-the-softmax-family]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, indexing, numerics]
---
## User-visible outcome

`tiler::softmax-f32@1` carries a registered `IndexRealizationLaw`, so `FrozenIndexRealizationLawRegistry::resolve` stops answering `MissingRealizationLaw` for it and refinement can prove a provider's emitted region sequence realizes the occurrence. It is the third and last piece of the softmax vertical: the two walls are down.

## Why it has no dependencies

**Fact.** Both walls landed together on `tkt/admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`. [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md) registered `tiler.scalar::maximum-f32@1`, and [`admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence`](admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence.md) widened `VerifiedIndexRegionSequence` to a published value with several readers. Both are labelled drafts with acceptance nodes parked for Tom; neither node blocks *use* inside `tiler-ir`, exactly as the normalization law's did not.

## What the law must realize

**Fact.** `softmax_f32_reference_semantics` (`crates/tiler-ir/src/semantic/softmax.rs`) pins, over the single reduced axis and in this exact order: `m` = the strict left fold of the NaN-propagating `Maximum` family over the canonical contributor sequence seeded at the first contributor; `e_i = Exp(s_i - m)`; `d` = the strict left fold sum of `e` over the same sequence seeded at the first contributor; `c = 1.0 / d` as one division of one by the denominator; `r_i = e_i * c` as a multiplication by that reciprocal and deliberately not `e_i / d`.

**The staging, now expressible.** `the_softmax_staging_publishing_the_exponentials_chains` in `crates/tiler-ir/src/index/sequence.rs` checks the shape at the sequence layer: four stages, sources `[[Occurrence(0)], [Occurrence(0), Intermediate(0)], [Intermediate(1)], [Intermediate(1), Intermediate(2)]]`, with the exponentials published by stage one and `retained_through` stage three. That test proves the *chain* is well formed; it does not emit the softmax's scalar programs, which is this ticket's work.

## The three capabilities the existing emitters do not have

Stated so the elimination starts from what is missing rather than from a template guess. Compare `realize_root_mean_square_scale` in `crates/tiler-ir/src/index/law.rs`:

1. **A fold whose combiner is not `add-f32`.** `SumPlan::fold` hard-codes `add_reducer`, and the maximum fold has no identity, so it is seeded at the first contributor rather than at a constant — which `SumPlan`'s empty/tail split already contemplates for the sum but with a `0.0` seed. `SOFTMAX_F32_FACT_EMPTY_REDUCED_AXIS` says a zero-length axis yields a zero-length output with no scalar softmax evaluated, so the identity-less fold must never face an empty contributor domain, and the shape rule is what makes that unreachable rather than merely undefined.
2. **A middle stage that is neither a fold nor a two-operand pointwise pass.** Stage one reads the scores at their own coordinates and the row maximum at the kept coordinates, and writes `Exp(s_i - m)` — a subtraction and an elementary function between the read and the write. `emit_pointwise` applies exactly one scalar.
3. **A final stage reading two published values of different ranks.** `e` at the point coordinates and `d` at the kept coordinates, plus the reciprocal `c = 1.0 / d` computed once per row rather than once per point.

**Where generality should go, per the worked-examples discipline and the precedent this vertical already set.** The normalization's ticket closed three gaps as reusable *emitters* rather than as one family's inline code. The same rule applies here: a fold parameterized by its combiner and its seeding rule, and a stage that reads a reduced-rank published value at kept coordinates (which already exists), are the reusable pieces. The next staged family instantiates those.

## The scalar-program sibling need, named by the fold-with-epilogue acceptance

[`accept-the-root-mean-square-scale-realization-law`](accept-the-root-mean-square-scale-realization-law.md) records, as a choice worth objecting to and accepted with no exclusion: "**The chain is fixed; only the two attribute identifiers are law data.** … Carrying them would need a scalar-program language inside a law, which is the universal IR `law.rs`'s header refuses, or five independently settable keys whose combinations this module could no longer claim to interpret. The counter-argument, and the honest one: a second width's normalization would need a second variant rather than a second row."

**That consequence lands here, and it is why this ticket must not reach for a general template.** The softmax is the second family whose chain is fixed by the template rather than carried in it, so it takes its own variant with its own tag — tags `1..=10` are taken — and its own record-local attribute identifier. Two fixed-chain variants is where the trade the acceptance named starts costing: a reviewer should read this ticket's outcome asking whether a third one is still the right shape, or whether the accepted refusal of a scalar-program language inside a law needs reopening with new evidence. Reopening it is *not* in this ticket's scope; naming the evidence it produces is. Record, in the outcome, how much of the two variants' emission is shared machinery and how much is per-family chain, because that ratio is the measurement the reopening question would need.

## Public boundary

`IndexRealizationLaw` is `pub` and `#[non_exhaustive]`, so a new variant lands as a labelled draft with its own acceptance node parked for Tom, as `StagedRootMeanSquareScaleF32` did. The encoding tag must be appended with per-tag injectivity reasoning recorded at the encoding site, and `the_root_mean_square_law_tag_is_append_only_and_distinct` is the pattern.

## Non-goals

Making the compiler *recognize* the softmax as a program stage. Region formation's synthetic-intermediate record carries one consumer stage per handed value and needs widening first — [`carry-a-multi-reader-intermediate-through-region-formation`](carry-a-multi-reader-intermediate-through-region-formation.md). A registered law is useful without it: it lets refinement verify an emitted sequence.

## Closes when

`tiler::softmax-f32@1` resolves to a registered law that realizes a verified `VerifiedIndexRegionSequence` whose stages match the pinned reference step for step — the extrema family, the maximum subtraction, the exponential, the sum's seeding and order, the single reciprocal division, and the reciprocal multiplication — the new encoding tag is proved append-only and injective, every declared attribute is consumed or refused by name, every existing chain's identity is unchanged byte for byte, and the acceptance node is filed.

## Outcome — 2026-08-06

`tiler::softmax-f32@1` carries a registered `IndexRealizationLaw`. `FrozenIndexRealizationLawRegistry::resolve` answers for it, `family_realizes_region_sequence` answers `true`, and the resolved law realizes a four-stage `VerifiedIndexRegionSequence` matching `softmax_f32_reference_semantics` step for step.

### The variant and its tag

`IndexRealizationLaw::StagedSoftmaxF32 { axes_attribute: AttributeFieldId }`, constructed by `pub const fn staged_softmax_f32()` naming `SOFTMAX_REDUCED_AXES_ATTRIBUTE`, encoded under **tag 11**. It is a labelled draft at its definition, pointing at the acceptance node [`accept-the-softmax-realization-law`](accept-the-softmax-realization-law.md) (`awaiting-decision`, parked for Tom). Registered in the standard provider transaction beside the normalization's row.

**Tag-11 injectivity, recorded at the encoding site.** The first byte discriminates, which is the whole of the separation from tags 4, 5, 6, and 7 — each writes the same payload shape this one does, one fixed-width attribute identifier and nothing else. Within the tag that payload is a single injection on a fixed offset, so two rows differing in the axes identifier differ in the four bytes it owns; there is no second field, so no ordering question arises of the kind tag 10 has to answer for its pair. `the_softmax_law_tag_is_append_only_and_distinct` asserts the tag, distinctness from all fourteen constructible rows of tags 1..=10, and payload separation.

### The chain

Four stages, sources `[[Occ(0)], [Occ(0), Int(0)], [Int(1)], [Int(1), Int(2)]]` — the exact shape `the_softmax_staging_publishing_the_exponentials_chains` pinned at the sequence layer.

- **S0** folds the scores with `maximum-f32`, seeded at the first contributor with no identity, and publishes `m` (the reduced shape).
- **S1** reads the scores at their own coordinates and `m` at the kept coordinates and publishes `e_i = Exp(s_i - m)`. The subtraction is `add(s_i, multiply(m, -1.0))`: there is no subtraction scalar key, negation is exact, and `SOFTMAX_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED` already names this adjacency as the operation's only multiply-add pair. The negation carries only the kept dimensions, so it is evaluated once per row.
- **S2** folds `e` with `add-f32`, seeded at the first contributor, and publishes `d`.
- **S3** reads `e` **again** at the point coordinates and `d` at the kept coordinates, computes `c = divide(1.0, d)` — free dimensions = the kept ones, so one division per folded row — and writes `multiply(e_i, c)`, deliberately not `e_i / d`.

For `[3, 4]` axis 1 the reads are `[(0,1,1), (1,2,3), (1,3,3), (2,3,3)]`: `e` is published by stage one and retained through stage three, across the stage that reads it and publishes something else.

### The three general emitters

Landed as reusable machinery in `crates/tiler-ir/src/index/law.rs`, private to the module, instantiated by both fixed-chain variants — never softmax-inlined.

1. **`FoldPlan` (was `SumPlan`), parameterized by combiner and seeding rule.** `combining(combiner, empty_identity)` sets the two together, because an identity is a property of the combiner. `for_boundaries` defaults to `add-f32` with `0.0`, which is byte-for-byte what every fold this emitter produced before the parameters existed. `combine_with(body, key)` replaced `add_reducer`; the contraction now passes `add-f32` explicitly. An identity-less plan refuses an empty contributor domain under `fold-empty-domain-without-identity`.
2. **`emit_fold_region(context, plan, epilogue)`** — the whole folding region: kept domain, boundaries, fold, an arbitrary scalar epilogue inside the producing region, write. The plain sum passes the identity epilogue and emits exactly what it did; the normalization's `a/N + eps` then `Rsqrt` is now an instantiation rather than inline code.
3. **`emit_row_broadcast_stage(context, reduced, inputs, result, body)`** with `enum StageAccess { Point, FoldedRow }` — any number of input boundaries, each read at its own point coordinates or at the kept coordinates of the folded axes, with arbitrary scalar work between the reads and the write. The per-row prologue needs no rule of its own: a value computed from a `FoldedRow` read alone carries only the kept dimensions as its free dimensions, so `1.0 / d` is once per row and `e_i * c` once per point, and the region model says so.

The normalization's own two stages were refactored onto (2) and (3). Both `realize_root_mean_square_scale` and `realize_softmax` now consist of interface checks, plan construction, four small closures naming their families' scalar chains, and the stage-source wiring.

### The ratio the fold-with-epilogue acceptance asked for

**Measurement**, over `crates/tiler-ir/src/index/law.rs` at commit `28c6e3807a618ef49213d9114856674c9157b451`, counting code lines (blank lines, `//`, and `///` excluded) of the items the two fixed-chain variants' emission consists of:

| | code lines | items |
| --- | ---: | ---: |
| Shared machinery | **434** | 13 |
| Per-family chain | **263** | 3 |
| | **697** | 16 |

**62% of the emission is shared machinery.** Shared: `FoldPlan` struct 12 + impl 222, `emit_row_broadcast_stage` 68, `emit_fold_region` 28, `reduction_axes` 26, `scalar_constant` 16, `combine_with` 12, `declare_parallel_domain` 11, `dimension_expressions` 10, `single_result` 9, `apply_one` 8, `scalar_attributes` 8, `StageAccess` 4. Per-family: `realize_root_mean_square_scale` 120, `realize_softmax` 129, `folded_extent_bits` 14.

**And the number the reopening question actually needs is smaller than that.** Within the 263 per-family lines:

- **46 lines (17%) are the scalar chain proper** — the four closures that name which scalars are applied to what. This is the part a scalar-program language inside a law would carry as *data*: rms fold epilogue 14, rms pass 10, softmax exponential 10, softmax normalization 12.
- **61 lines (23%) are interface and attribute checks** — arity, operand binding, value type, shape, the declared field set. Data-carrying removes none of these.
- **28 lines (11%) are the stage-source wiring** — which occurrence input or published value each stage boundary reads.
- The remaining ~128 lines are boundary derivation and region-builder plumbing (one builder, one `LawContext`, one `build()` per stage).

So the scalar chain the accepted refusal keeps out of law data is **46 of 697 code lines, 6.6% of the two variants' emission**. A third fixed-chain variant would add roughly another 120–130 per-family lines of which roughly 20–25 would be chain. **This ticket does not reopen the refusal**; the measurement is recorded so that a future reopening argues from it rather than from impression. The honest reading in both directions: the marginal cost of a variant is dominated by checks and plumbing that a scalar-program language would not remove, which argues *against* reopening — and the chain lines are precisely the ones that are per-family rather than reusable, which is what a reopening would be trying to fix.

### Identity evidence

- **Every existing chain identity is unchanged, byte for byte.** `the_landed_one_reader_chain_identities_are_unchanged_byte_for_byte` passes untouched across the `FoldPlan` generalization and the refactor of the normalization's two stages onto the new emitters: `rms-norm-3x4-axis1` 4072 bytes / `77a5cd34…`, `rms-norm-rank1-4-axis0` 3649 / `b318507a…`, `staged-template-rank1-4-axis0` 2023 / `3ddd3268…`. `an_existing_law_payload_is_unchanged_by_the_appended_tag` passes.
- **New pins for the softmax chain**, computed on this tree: `softmax-3x4-axis1` 5589 bytes / `5f091f6d2421d119f661c6cb2af8a8e66495324b49e3d29d362a670af280638a`; `softmax-rank1-4-axis0` 4887 / `65f06df9750048397fddbdf751610684a77af203072c7dd1d067695a4a84116b` (`the_softmax_chain_identity_is_pinned`).
- **One pin moved, and it is the sidecar's.** Registering a law moves the count-prefixed realization-law sidecar and therefore `FrozenIndexRealizationLawRegistry`'s identity, which moves `explain::tests::deterministic_trace_is_sealed_and_rendered_separately`'s request digest from `9478647f38ab8df5` to `7bba54bcb59ec2cc`, recomputed on this tree by that pin's own documented mechanics. `implementation/compiler` was added to this ticket's scopes for it. The semantic snapshot identity is computed without the sidecar and does not move, so no artifact or kernel-program identity is touched.

### Every declared attribute consumed or refused by name

The occurrence's field set must be exactly the one field the law names (`softmax-attributes`), which is what stops `reduction_axes`' tolerance for a wider record from letting a payload go unread. Refusals, all watched except where noted: `softmax-arity` (an occurrence with two operands), `softmax-value-type` (a one-operand occurrence of another element type), `softmax-shape` (a one-operand `f32` occurrence whose result drops the axis — a reduction), `softmax-attributes` (a law naming a field the record does not carry), `fold-empty-domain-without-identity` (a zero-length reduced axis), `staged-law-requires-region-sequence` (the single-region API). **`softmax-reduced-axis-rank` has no watched perturbation** and is stated as such in the acceptance node: a multi-axis reduced-axes sequence is unreachable from a verified occurrence because the family's own inferencer refuses it before a subject exists, and the check is kept because a law is interpreted against a subject rather than against the inferencer that produced it.

### Watched-failing perturbations

Each was applied, observed to fail the named test, and reverted.

1. `r_i = e_i / d` in place of the reciprocal chain → `the_softmax_law_realizes_the_pinned_reference_step_for_step` fails (`["divide-f32"]` vs `["constant-f32", "divide-f32", "multiply-f32"]`) and `the_softmax_chain_identity_is_pinned` fails (5218 vs 5589 bytes).
2. The extrema fold combining with `add-f32` → the step-for-step test fails (`["add-f32"]` vs `["maximum-f32"]`).
3. `fold_empty` falling back to a `0.0` seed when the plan has no identity → both `an_identity_less_fold_refuses_the_empty_contributor_domain` and `a_zero_length_reduced_axis_is_refused_rather_than_seeded` fail.
4. Giving *both* softmax folds an invented empty-domain seed → `a_zero_length_reduced_axis_is_refused_rather_than_seeded` fails. (Perturbing only one does not, because the other still refuses — which is why perturbation 3's emitter-level test exists and separates the seeding rule from the family-level outcome.)

### Tests added

In `crates/tiler-ir/src/index/law.rs`: `the_softmax_law_realizes_the_pinned_reference_step_for_step`, `a_zero_length_reduced_axis_is_refused_rather_than_seeded`, `an_identity_less_fold_refuses_the_empty_contributor_domain`, `the_softmax_law_refuses_the_occurrences_it_does_not_name`, `the_softmax_family_resolves_to_its_registered_law`, `the_softmax_chain_identity_is_pinned`, `the_softmax_law_tag_is_append_only_and_distinct`, `the_softmax_law_refuses_the_single_region_realization`, plus the `softmax_subject`, `serial_sum_subject`, and `reducer_body_steps` helpers. Population: `tiler-ir` went 893 → 901 tests, all passing.

### Two claims about current behaviour that this made stale, and fixed

- `index::refinement::tests::the_family_region_sequence_query_agrees_with_the_resolved_law` used the softmax as its "registered operation the authority carries no law for" row. That row is now `tiler::slice-f32@1`, and the softmax joined the normalization as a `true` row.
- `crates/tiler-compiler/tests/softmax_recognizer_boundary.rs` claimed the softmax "carries no law at all, so the same arm answers `false` for it". **Measurement:** on base `f0132c88` the softmax program refuses under `UnsupportedCapability { rule: "operation-set" }` — the recognizer had no shape for it; on this tree it refuses under `UnsupportedCapability { rule: "missing-capability" }` — it is recognized, and nothing installed lowers what its realization needs. Both are the same *class*, so the test's `is_err()` could not tell them apart; it now asserts the exact class-and-rule and the header states where the wall moved to and what still stands in front of it ([`carry-a-multi-reader-intermediate-through-region-formation`](carry-a-multi-reader-intermediate-through-region-formation.md)). This is a boundary *moving*, not the ticket's non-goal being done: the compiler still compiles no softmax program.

### Support matrix

Advances `tiler::softmax-f32@1` from "registered operation with no realization law" to "registered law realizing a verified four-stage region sequence"; it does not advance the row to whole-program compilation or dispatch, which remain behind region formation's multi-reader carry and a lowering provider.

### Commands

```sh
cargo fmt --all --check
cargo clippy -p tiler-ir -p tiler-compiler --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-ir -p tiler-compiler
cargo nextest run --workspace     # 2910 passed, 7 skipped
cargo test --workspace --doc
make full
tkt lint
git diff --check
tkt guard tkt/register-the-softmax-realization-law --format json
```

Exact commit: `28c6e3807a618ef49213d9114856674c9157b451` — every code and ticket change is in it. `make full` was run on that commit and passed end to end (`2910 passed, 7 skipped`; release `1005 passed, 3 skipped`; `tkt lint` ok; shellcheck ok). The follow-up commit that fills this hash in and moves the status touches `tickets/` only, which is outside the gate-carry list, so the green gate carries; `tkt lint` was rerun on it.

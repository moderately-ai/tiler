---
id: correct-the-bf16-reference-evaluation-status-outside-the-dtype-ledger
title: Correct the BF16 reference-evaluation status outside the dtype ledger
status: in-progress
priority: p1
dependencies: []
related: [evaluate-bf16-reference-semantics, register-the-bf16-semantic-operation-signatures]
scopes: [contracts/navigation, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, documentation]
claimed_from: todo
assignee: worker-bf16-status
lease_expires_at: 1785586663
---
## User-visible outcome

Two documents outside `evaluate-bf16-reference-semantics`'s scope still assert that `tiler-reference` refuses every BF16 key. A reader planning the next BF16 rung would conclude the oracle does not exist, and would either duplicate it or gate work on it that is no longer gated.

## What is stale, exactly

**Fact**, at `evaluate-bf16-reference-semantics`'s landing commit:

- `docs/roadmap.md`, the reduced-precision-float row of the R-rung table (search `R4 through R7 are unmoved`): "the standard reference provider registers no BF16 evaluator and refuses each key with `MissingCapability`". R4 moved; R5 through R7 did not, and the sentence has to separate them. The same row's closing sentence gates BF16's remaining rungs on `evaluate-bf16-reference-semantics` (R4), which is now satisfied.
- `docs/research/numerics/bf16-computation-accumulator-and-conversion.md`, the maturity table row `Defined but unimplemented`: "`MissingCapability`, which is exactly what `ReferenceEvaluator::standard()` returns for the three landed BF16 keys today". The example is no longer that; the *class* is still a real one and needs a different instance, not deletion.

`docs/dtype-support.md`'s BF16 `Reference evaluation` cell and its family note were moved by the landing ticket and are not part of this one.

## Why this is a separate ticket

`docs/roadmap.md` was held under `contracts/navigation` by a concurrently live ticket when the reference landed, and `docs/research/numerics/**` is a scope the landing ticket did not hold. Both edits are one paragraph each and neither is urgent enough to justify a cross-scope edit under someone else's claim.

## Closes when

Both sentences state the layer that actually moved and the layers that did not, the roadmap row's R-rung claim separates R4 from R5 through R7, and the research document's `Defined but unimplemented` row names a class instance that is still true.

## Outcome

Both corrections landed on top of `5564ca9`.

**`docs/roadmap.md`, the reduced-precision-float row.** The rung cell reads `R4 for BF16 constant, multiply, and add` rather than `R3`. The evidence cell's registration paragraph no longer claims "and no further rung", and a new **Fact** states R4: the `tiler::bf16@1` value validator and the three evaluators in `crates/tiler-reference/src/bf16.rs`, why the oracle is exact rational rather than an `f32` composition, and its evidence — exhaustive-finite over all 65,536 encodings with the class census asserted, 30 hand-derived witnesses across six categories, the overflow boundary decided at the midpoint from both sides, and the tie-rule perturbation that disagrees at exactly four witnesses — with the per-layer cell left to the [dtype support ledger](../docs/dtype-support.md). "R4 through R7 are unmoved" is now "R5 through R7 are unmoved", carrying forward only the two checks that are still true. The trigger cell drops the satisfied R4 gate, keeps `admit-a-bf16-scalar-arithmetic-subject` for R5/R6, and names `conform-the-bf16-vertical-end-to-end` for R7.

**`docs/research/numerics/bf16-computation-accumulator-and-conversion.md`, the `Defined but unimplemented` row.** The class is kept and its instance replaced. No *standard* semantic key lacks a reference capability any more — `FrozenSemanticRegistry::standard()` registers fifteen operations and `StandardReferenceProvider` (with `register_standard_bf16` and `register_standard_quantization`) registers fifteen capabilities — so the row now names the case that is still live: an operation registered by a second semantic provider that no reference provider implements, pinned from both sides by `missing_and_external_reference_capabilities_are_explicit`.

**Verified rather than asserted.** `cargo nextest run -p tiler-reference` over `missing_and_external_reference_capabilities_are_explicit`, `the_registered_bf16_operations_resolve_a_reference_capability`, `every_encoding_round_trips_except_the_nans_that_canonicalize`, and `the_hand_derived_witness_corpus_agrees_in_every_category` (4 passed) supports the substituted instance and the R4 evidence; `cargo nextest run -p tiler-compiler` over `a_pure_bf16_program_is_statable_and_refused_at_the_request_boundary`, `the_capability_table_names_exactly_the_admitted_operations`, and `every_unplanned_operation_is_registered_and_consumes_no_dimension` (3 passed) supports the two retained R5-through-R7 checks. The witness count is the test's own `assert_eq!(total, 30)`.

## Found outside this ticket

`docs/roadmap.md`'s sequence-extension row (search `none of the thirteen is a concatenation`) enumerates the standard registry as four scalar F32 definitions plus contraction, three BF16, reindex, broadcast, and three quantization operations. `StandardSemantics::register` also calls `register_standard_silu` and `register_standard_rms_norm`, so the registry holds fifteen operations, not thirteen. The row's own claim — that none of them is a concatenation — is still true, and nothing tonight falsified the count; it went stale when the activation and normalization families landed. Reproduce with `rg -n 'register_standard_' crates/tiler-ir/src/semantic/registry.rs`.

---
id: produce-typed-strict-affine-assemble-semantic-precondition
title: Produce the strict-affine Assemble scale precondition
status: in-progress
priority: p2
dependencies: [produce-typed-strict-affine-quantize-semantic-preconditions]
related: [enforce-resolved-encoded-value-binding-conformance, own-the-dtype-support-maturity-matrix]
scopes: [implementation/ir, implementation/reference, contracts/foundation, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantic-ir, validation, quantization]
---

## User-visible outcome

`AssembleStrictAffine` cannot construct a governed encoded value from a non-positive or non-finite scale. Its exact scale restriction is a typed occurrence-owned semantic precondition with the same proof, disproof, residual, identity, and invalid-input behavior established for `Quantize`, but it remains a distinct declaration on the Assemble occurrence.

## Implementation keys

- Declare `PositiveFiniteScalar` on Assemble operand 1 over the whole rank-zero f32 logical value with an Assemble-specific stable invalid-input code.
- Reuse the reviewed predicate declaration, static exact-constant proof, assessment, obligation identity, and program-view vocabulary from `produce-typed-strict-affine-quantize-semantic-preconditions`. Do not copy another evaluator or editable string authority.
- Keep codes and zero-point payload domains in resolved-value conformance. Keep packed-tail, padding, alignment, bit order, and storage canonicality in physical representation validation.
- A directly governed positive finite f32 constant proves the declaration; zero, negative, infinite, or NaN constants disprove transactionally; every other producer remains residual.
- Preserve the exact Assemble occurrence, subject value/view/type, declaration ordinal, and invalid-input code in obligation identity. Never reuse a Quantize obligation merely because predicate and subject shape resemble one another.
- Reuse the shared `PositiveFiniteScalar` evaluator through one Assemble-owned `assemble_preconditions()` declaration and an Assemble-specific stable diagnostic code. The reference path already calls `read_scale`; do not add a second validator.

## Closes when

Valid constants prove; positive and negative zero, positive and negative finite values, subnormals, positive and negative infinity, quiet NaN, and signaling NaN take their exact proved/refused class transactionally; runtime-unknown scale leaves one exact residual; dead Assemble occurrences compact their assessment away; the normative reference and typed declaration agree for U4 and U8; Assemble and Quantize retain distinct declaration and obligation identities; every new check has been observed failing under perturbation; targeted `tiler-ir` and `tiler-reference` tests and Clippy pass; and the batch gate passes.

## Graph maintenance

- Update ADR 0033 and numerical/IR maturity text only for the newly implemented Assemble producer.
- Relate the residual to `enforce-resolved-encoded-value-binding-conformance`, which separately owns direct encoded program inputs and does not replace this operation precondition.
- Update the dtype maturity matrix only for the exact Assemble semantic-validation cell.
- Advance the semantic definition projection, standard registry provenance/identity, and owning provider revision exactly once, then recompute every pinned identity on the merged tree rather than copying Quantize fixtures.

## Outcome

**Most of this ticket landed under another ID, and the delta was the class table.** [`admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode`](admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode.md) delivered the declaration itself at `05a429a`: `scale_domain_preconditions("strict-affine-assemble")` declares `PositiveFiniteScalar` *and* `PositiveNormalScalar` on Assemble operand 1 over the whole rank-zero f32 value, under `tiler::strict-affine-assemble-scale-not-positive-finite@1` and `tiler::strict-affine-assemble-scale-subnormal@1`, through the one shared reviewed evaluator and one Assemble-owned declaration function. This ticket delivered what that landing did not: the exhaustive per-class outcome table on Assemble occurrences, Assemble's residual and compaction behaviour, Assemble-occurrence obligation exactness, and the reference route's own agreement.

**The class table, on Assemble occurrences.** `every_exact_constant_scale_class_takes_its_assemble_outcome_transactionally` applies one table of fifteen `f32` classes to Assemble over both u4 and u8: positive and negative zero, negative finite normal and `f32::MIN`, negative subnormal, positive and negative infinity, quiet and signalling NaN, the smallest, an interior, and the largest positive subnormal, and the smallest, an interior, and the largest positive normal. Each refused class asserts its exact predicate, invalid-input code, and declaration ordinal; each admitted class proves *both* declarations through `StandardConstantF32BitsV1` with no obligation identity, and the built program retains exactly three assemble occurrences of the expected encoded type. Transactionality is asserted per class rather than once — a disproved apply must leave `retained_canonical_work_bytes` exactly where it was, which is also why the loop can keep using the same builder.

The ordinals are why this table is Assemble's own rather than a shared parameterization: Assemble declares no `NoNaN`, so its scale predicates sit at ordinals 0 and 1 where Quantize's sit at 1 and 2. A shared assertion would have passed against a subject bound to the wrong occurrence. The negative-subnormal row is the one that pins the code ordering — it fails both predicates and must report the general cause.

**Residual, compaction, and occurrence exactness.** `runtime_unknown_u4_and_u8_assemble_scales_retain_two_ordered_residuals` pins two residuals in declaration order with two distinct obligation identities over both code widths — two, because the scale bears both predicates and collapsing them would lose which cause a bound payload hit. `dead_assemble_assessments_are_removed_by_output_compaction` shows an Assemble occurrence whose result reaches no output taking its whole assessment with it. `assemble_obligations_are_occurrence_exact_over_one_shared_scale` builds two Assemble occurrences over *one* scale value, so their four declarations agree on predicate, operand, view, subject, type, shape, and ordinal, and only the occurrence separates them; it also includes a Quantize occurrence over that same scale for seven distinct identities in total. The cross-operation half of that assertion is deliberately labelled over-determined at the site: the two operations already differ in both ordinal and code, so no single dropped field collapses them, and the guarantee it rests on is the declaration-level one.

**The reference path.** `assemble` reaches `read_scale` on its own line (`crates/tiler-reference/src/quantization.rs`), not through the compound validator, and `read_scale_value` refuses on `!value.is_normal() || value <= 0.0` — exactly the conjunction of the two declared predicates, with no second validator added. `residual_bearing_u4_and_u8_assemble_graphs_reject_every_invalid_runtime_scale_class` replays the same twelve invalid classes as runtime payloads through the evaluator for both profiles and requires each to be refused at `assemble_strict_affine_op` specifically; the three admitted scales assemble and preserve their exact scale bits.

**No provider revision was advanced, and the derivation is why.** `encode_definition_projection` is provider-*independent* and folds in each definition's semantic precondition declarations; `encode_admission_provenance` is what carries `ProviderIdentity`. So the Assemble declarations moved the definition projection when they landed at `05a429a`, which is also what rebaselined the explain request qualifier to `bae4788d2fc79631` there — verified unchanged on this branch, and not moved again. This ticket's delta registers no new definition and changes no registered content, so neither identity moves, and a bump would assert an admitting-authority change that did not happen while invalidating every pinned provenance. That matches the corpus precedent: the dtype-catalog, contraction, and normal-scale landings each declined to bump for meaning-preserving additions, and the one landing that did bump (6→7, `caa4b52`) introduced the declaration mechanism itself. The rule is now recorded at `StandardSemantics::identity` so the next worker does not bump reflexively.

**Contract and decision text.** [ADR 0033](../docs/decisions/0033-semantic-validation-enforcement.md)'s implementation status now names both implemented producers rather than `Quantize` alone, states that they declare separately under their own codes and ordinals with occurrence-distinct obligations, states `Dequantize`'s structural absence, and separates the reference evaluator's honesty about the declared domain from enforcement of a residual. [Numerical semantics](../docs/numerical-semantics.md) gains the class-table measurement with its boundary, corrects a stale "the two predicates" to three, and records occurrence-exact obligations and dead-occurrence compaction.

**Flagged, not done.** The dtype maturity matrix lives in `contracts/navigation`, which this ticket does not hold. No cell is owed regardless: the strict-affine rows' `Semantic operation signatures` cells are already `tested guarantee`, and `Runtime semantic validation` is correctly `absent/unsupported` because nothing enforces a residual against a payload. Two stale sentences in that file are for its owner: `docs/dtype-support.md` line 126 still says the reference tests "positive finite scale" where the admitted domain is positive *normal*, and the same line does not mention the assemble scale route. Both predate this ticket and belong to whoever next holds that scope.

**Perturbations watched failing** (each applied, run, reverted):
1. Assemble registered with `scale_domain_preconditions("strict-affine-quantize")` → the class table reports the quantize code, and the declaration test fails.
2. The two scale declarations reordered in `scale_domain_declarations` → the class table's ordinals invert and the residual-order test fails.
3. Assemble declares only `PositiveFiniteScalar` → 5 tests fail, including the reference one, on the three positive-subnormal classes and the residual count.
4. `read_scale_value` uses `is_finite` instead of `is_normal` → the reference assemble test fails on the smallest positive subnormal.
5. `encode_obligation_identity` writes a constant in place of `operation_coordinate` → the two Assemble occurrences over one scale collapse to two identities.
6. The dead-compaction test outputs the assembled value instead of the codes input → operation count 1 rather than 0.
7. `push_operation` reserves canonical work before assessing preconditions → the per-class transactional assertion fails on the first refused class.

**Closes-when, item by item.** Valid constants prove — this ticket (class table, three admitted classes, both widths). Both zeros, positive and negative finite, subnormals, both infinities, quiet and signalling NaN take their exact class transactionally — this ticket. Runtime-unknown scale leaves exact residuals — this ticket (two, ordered, distinct). Dead Assemble occurrences compact their assessment away — this ticket. Reference and typed declaration agree for U4 and U8 — this ticket (reference route), on the validator `05a429a` narrowed. Assemble and Quantize retain distinct declaration identities — `05a429a`; distinct obligation identities — this ticket. Every new check observed failing under perturbation — this ticket, seven above. Targeted `tiler-ir` and `tiler-reference` tests and Clippy — pass. Batch gate — one `make full`.

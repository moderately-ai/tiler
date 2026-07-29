---
id: produce-typed-strict-affine-quantize-semantic-preconditions
title: Produce typed strict-affine Quantize semantic preconditions
status: todo
priority: p2
dependencies: [prototype-quantized-value-vertical]
related: [implement-first-runtime-semantic-value-precondition-enforcement]
scopes: [implementation/ir, implementation/reference, contracts/foundation, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantic-ir, validation, quantization]
---

## User-visible outcome

Strict-affine `Quantize` owns machine-readable semantic preconditions for every value restriction its accepted semantics impose. Known-valid inputs can be proved, known-invalid inputs reject as invalid application, and runtime-unknown inputs become exact residual obligations without collapsing into applicability, representation, or backend feasibility.

## Evidence and boundary

- **Fact:** `QuantizeStrictAffine` currently advertises the descriptive fact `nan-reject`, while its inferencer emits only result `ValueFact`s and `rg -n 'SemanticPrecondition' crates` returns no implementation.
- **Fact:** the normative reference rejects NaN expressed values and rejects a dynamic rank-zero f32 scale unless it is positive and finite. Both restrictions are required for a sound strict-affine application; a NaN-only producer would be incomplete.
- **Fact:** governed U4/U8 zero-point domain is structurally represented by the accepted encoded-value contract. Packed-tail canonicality belongs to physical representation validation, not this semantic predicate family.
- **Inference:** operation-owned bounded typed declarations instantiated against an exact occurrence are the smallest durable authority. A string fact, an inferencer-private side channel, or a route-global list cannot preserve identity and protected-stage dependencies.

## Implementation keys

- Define a closed first typed predicate vocabulary containing at least expressed-value `NoNaN` and rank-zero `PositiveFiniteScalar`, with stable predicate key, revision, invalid-input code, subject selector, and declaration ordinal. Reserve a governed extension seam without claiming unknown predicates are evaluable.
- Attach bounded declarations to the governed operation definition and include them in definition projection, registry, and provider provenance identities. Do not duplicate definitions as caller-authored graph facts.
- Instantiate declarations against the exact operation occurrence and semantic value handle, including operand or result role, exact logical view transformation, complete resolved value type, encoded component declarations, and parameter maps.
- Represent proof, static disproof, and residual obligation as distinct typed outcomes. Static disproof is invalid application; unknown remains residual; neither becomes an applicability miss or a physical capability failure.
- Make strict-affine `Quantize` produce both required predicates when their subjects are runtime-unknown. Elide a predicate only from authoritative proof evidence; do not infer value contents from type, shape, pointer, or descriptive facts.
- Keep index-domain obligations, physical access bounds, packed-tail canonicality, storage encoding support, and target honourability in their existing owners.
- Bound predicate counts, names, revisions, subject selectors, and encoded identity before allocation or canonical encoding.
- Advance semantic definition, registry, and standard-provider revisions once on the merged tree, then recompute every semantic identity fixture rather than selecting an old pin.

## Adversarial evidence

- Known-valid authoritative values prove each predicate independently; a known NaN or non-positive/non-finite scale rejects before physical planning; runtime-unknown values emit ordered residuals.
- Same predicate on another occurrence, operand, view, resolved type, component role, map, or ordinal has a different obligation identity.
- Reclassifying either predicate as applicability, an index-domain obligation, or packed-tail canonicality fails a typed test.
- Unknown proof evidence cannot silently become proof, and a missing evaluator revision cannot silently become residual under another key.
- Perturb predicate key/revision, subject selector, declaration ordinal, or reached-definition encoding and observe the relevant identity/check fail before restoring it.

## Closes when

The public typed declaration and instantiated-obligation boundary has Tom's required interface review; strict-affine `Quantize` produces every currently accepted runtime-semantic predicate; proof, disproof, and residual behavior are tested independently; semantic identity and provider revisions are rebaselined; docs distinguish semantic validity from representation validity; every new check has been observed failing under deliberate perturbation; targeted `tiler-ir` and `tiler-reference` tests and Clippy pass; `tkt lint` and `git diff --check` pass; and the batch gate passes before integration.

## Graph maintenance

- Update ADR 0033 application status only for the producer boundary actually implemented; do not imply planning, artifact, or runtime enforcement exists.
- Release `admit-strict-affine-quantize-physical-candidate` when this producer and the selected profile/grouping prerequisites are complete.
- Keep the accepted built-in dtype catalog and dtype maturity matrix honest: this advances semantic validation for the exact strict-affine U4/U8 contract, not generic integer, packed, quantized, or backend support.

---
id: produce-typed-strict-affine-quantize-semantic-preconditions
title: Produce typed strict-affine Quantize semantic preconditions
status: done
priority: p2
dependencies: [prototype-quantized-value-vertical]
related: [implement-first-runtime-semantic-value-precondition-enforcement, produce-typed-strict-affine-assemble-semantic-precondition, enforce-resolved-encoded-value-binding-conformance, own-the-dtype-support-maturity-matrix]
scopes: [implementation/ir, implementation/compiler, implementation/reference, contracts/foundation, contracts/numerics, contracts/decisions, research/numerics, research/runtime, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantic-ir, validation, quantization]
---

## User-visible outcome

Strict-affine `Quantize` owns machine-readable semantic preconditions for every value restriction its accepted semantics impose. Known-valid inputs can be proved, known-invalid inputs reject as invalid application, and runtime-unknown inputs become exact residual obligations without collapsing into applicability, representation, or backend feasibility.

## Evidence and boundary

- **Fact at ticket creation:** `QuantizeStrictAffine` advertised the descriptive fact `nan-reject`, while its inferencer emitted only result `ValueFact`s and `rg -n 'SemanticPrecondition' crates` returned no implementation.
- **Fact:** the normative reference rejects NaN expressed values and rejects a dynamic rank-zero f32 scale unless it is positive and finite. Both restrictions are required for a sound strict-affine application; a NaN-only producer would be incomplete.
- **Fact:** governed U4/U8 zero-point domain is declared by the accepted encoded-value type, but type identity alone does not validate runtime payload bytes. `enforce-resolved-encoded-value-binding-conformance` owns that reusable value-boundary evidence. Packed-tail canonicality belongs to physical representation validation.
- **Inference:** operation-owned bounded typed declarations instantiated against an exact occurrence are the smallest durable authority. A string fact, an inferencer-private side channel, or a route-global list cannot preserve identity and protected-stage dependencies.

## Implementation keys

- Define distinct bounded predicate-identity and invalid-input-code types. Governed `NoNaN` and `PositiveFiniteScalar` keys identify dtype-independent predicate meaning; assessment still dispatches on the exact resolved type and logical view. Unknown extension keys remain exactly identified and residual but are not provable or executable without separately admitted authority.
- Attach bounded declarations to the governed operation definition and include them in definition projection, registry, and provider provenance identities. Do not duplicate definitions as caller-authored graph facts.
- Derive declaration ordinal from bounded list order; callers cannot author it. Validate every operand selector against the operation schema. The first projection vocabulary admits only `WholeValue`; keep the recognizer enum exhaustive so adding an encoded-component projection breaks every downstream recognizer at compile time rather than entering a wildcard, and do not claim that projection before a producer can define and validate one.
- Instantiate declarations against the exact operation occurrence and semantic value handle, including operand role, exact logical view, shape, and complete resolved value type. A future component view must add its role and parameter-map transformation to this identity before it can be admitted. Result invariants and resolved-value conformance are separate contracts, not precondition subjects.
- Represent proof, static disproof, and residual obligation as distinct typed outcomes. A retained proof names its closed host-owned proof basis; provider self-identification is not proof authority. Static disproof is invalid application; unknown remains residual; neither becomes an applicability miss or a physical capability failure.
- Make strict-affine `Quantize` produce both required predicates when their subjects are runtime-unknown. Elide a predicate only from authoritative proof evidence; do not infer value contents from type, shape, pointer, or descriptive facts.
- Retain proved and residual assessment records for explainability; disproof returns an owned typed build error and commits no operation, value, or canonical-work mutation.
- Mint and cache each residual obligation identity only after output compaction from the graph identity, reached definition projection, canonical operation coordinate, declaration ordinal and encoding, canonical subject coordinate, view, shape, and complete resolved type. Preflight the exact aggregate encoded byte count before allocating or encoding any obligation identity. Exclude transient arena handles, provider revision, full registry snapshot, physical storage, pointer, value version, coherence, and checker identity.
- Keep `OperationInferencer` result-only. The host builder owns occurrence instantiation and recognizes only exact governed `constant-f32` producer bits for the first static proof authority; inputs, arithmetic results, external producers, and caller claims remain residual.
- Keep index-domain obligations, physical access bounds, packed-tail canonicality, storage encoding support, and target honourability in their existing owners.
- Bound predicate counts, names, revisions, subject selectors, and encoded identity before allocation or canonical encoding.
- Advance semantic definition projection v4 to v5, registry v6 to v7, and standard provider revision 6 to 7 once on the merged tree. Registry v7 includes the host-sealed static-evidence authority tag so a lookalike provider cannot share complete provenance with the governed proof authority. Keep semantic graph v2 and admission provenance v1 because their grammars do not change. Add a dedicated obligation-identity v1 domain and recompute every pin rather than selecting an old value.
- Remove `nan-reject` and `scale-positive-finite` as independently editable machine authorities. Any descriptive rendering must derive from the typed declarations.

## Adversarial evidence

- Known-valid authoritative values prove each predicate independently with an exact typed basis; a lookalike provider using the governed provider identity cannot forge that authority; a known NaN or non-positive/non-finite scale rejects before physical planning; runtime-unknown values emit ordered residuals.
- Same predicate on another occurrence, operand, view, resolved type, or ordinal has a different obligation identity. A future component-view producer must additionally prove role/map sensitivity before widening the vocabulary.
- Reclassifying either predicate as applicability, an index-domain obligation, or packed-tail canonicality fails a typed test.
- Exact finite, signed-zero, and infinite expressed f32 constants prove `NoNaN`; qNaN and sNaN disprove. Smallest positive subnormal through maximum finite scale prove `PositiveFiniteScalar`; positive/negative zero, negative finite/subnormal, positive/negative infinity, qNaN, and sNaN disprove.
- Both runtime unknowns produce two residuals; each valid constant independently removes only its own residual; both valid constants produce zero residuals; dead Quantize assessments disappear during output compaction.
- Simultaneous invalid scale and NaN evidence uses the accepted `(logical index, stable code, ordinal)` priority rather than declaration callback order.
- Unknown proof evidence or predicate identity cannot silently become proof or a residual under another key.
- Equivalent graphs authored in different valid live topological orders mint equal corresponding obligation identities, while two identical calls at distinct semantic occurrences remain distinct.
- Perturb predicate key/revision, invalid code, subject selector, view tag, shape, resolved type, declaration ordinal, or canonical occurrence/subject coordinate and observe the relevant identity/check fail before restoring it. Exercise the aggregate cached-identity byte bound at the exact boundary and one byte over.

## Closes when

The public typed declaration and instantiated-obligation boundary has Tom's required interface review; strict-affine `Quantize` produces every currently accepted runtime-semantic predicate; proof, disproof, and residual behavior are tested independently; semantic identity and provider revisions are rebaselined; docs distinguish semantic validity from representation validity; every new check has been observed failing under deliberate perturbation; targeted `tiler-ir` and `tiler-reference` tests and Clippy pass; `tkt lint` and `git diff --check` pass; and the batch gate passes before integration.

## Graph maintenance

- Update ADR 0033 application status only for the producer boundary actually implemented; do not imply planning, artifact, or runtime enforcement exists.
- Correct ADR 0031, ADR 0032, numerical semantics, IR, operation-extension, correctness-testing, and retained affine research maturity text together. Record exact reference/IR support as partial and do not promote runtime or backend support.
- Release `admit-strict-affine-quantize-physical-candidate` when this producer and the selected profile/grouping prerequisites are complete.
- Keep the accepted built-in dtype catalog and dtype maturity matrix honest: this advances semantic validation for the exact strict-affine U4/U8 contract, not generic integer, packed, quantized, or backend support.

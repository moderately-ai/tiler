---
id: enforce-resolved-encoded-value-binding-conformance
title: Enforce resolved encoded-value binding conformance
status: todo
priority: p2
dependencies: [prototype-quantized-value-vertical, produce-typed-strict-affine-assemble-semantic-precondition]
related: [implement-first-runtime-semantic-value-precondition-enforcement, produce-typed-strict-affine-assemble-semantic-precondition]
scopes: [implementation/ir, implementation/artifact, implementation/reference, implementation/runtime, contracts/foundation, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, validation, compound-values, runtime]
---

## User-visible outcome

A runtime-bound encoded semantic value is accepted only when its authoritative logical compound view conforms to the complete resolved-value contract. Direct strict-affine U4/U8 inputs cannot reach any consumer with a missing, swapped, malformed, out-of-domain, stale, or incoherent component, and the resulting conformance evidence is bound to the exact value provenance rather than inferred from type identity or slots.

## Why this is not an operation precondition

- **Fact:** a direct encoded program input has no producing `Quantize` or `Assemble` occurrence whose operation precondition could validate its bytes.
- **Fact:** `DequantizeStrictAffine` consumes an already governed encoded value; duplicating scale/code checks as Dequantize predicates would make every consumer a second value-type authority.
- **Inference:** resolved-value conformance belongs at semantic value binding/production boundaries and is reusable by every consumer. Physical packed-tail canonicality remains separate because unused storage bits are not part of the logical value.

## Implementation keys

- Derive the complete validation contract from the admitted `ResolvedValueType`: exact scheme/version, logical shape, ordered component declarations, roles, component resolved types, component shapes, parameter maps, and any governed value-domain predicates. `BindingSpec` remains kind-only.
- For the first exact strict-affine U4/U8 per-tensor profiles, validate every logical code and zero-point against its complete U4/U8 domain and every scale as positive finite f32. Exclude padding and unused packed tail bits from logical scans.
- Bind evidence to exact logical subject/view/type/components/maps, value version or immutability provenance, producer completion, coherence epoch, validator key/revision, and route dependency. Pointer equality and decoded slot position are never evidence.
- Internally produced values compose evidence from their verified producer semantics: conformance of Quantize's zero-point operand plus its operation preconditions and operation semantics establish its result; Assemble scale precondition plus conformance of its code and zero-point operands and operation semantics establish its result. Do not rescan when a complete same-provenance proof exists.
- Direct runtime inputs require the binding validator. Missing logical-view reconstruction, inaccessible memory, absent coherence, unsupported map/scheme/encoding, or resource shortfall rejects before routing by exact capability name.
- Semantic invalidity is not applicability and cannot fall back to another interpretation. Malformed or dishonest artifact/binding metadata is an invariant error. Physical tail noncanonicality is reported by its physical owner.
- Keep dynamic bytes/version/coherence out of static artifact identity and inside the execution-scoped evidence. Static validator schema and revision remain identity-bearing.

## Adversarial evidence

- Missing, duplicate, extra, swapped, wrong-type, wrong-shape, wrong-map, cross-value, and stale-version components reject.
- U4 and U8 minimum/maximum codes and zero points pass; every out-of-domain payload fails without approximating through a wider integer.
- Smallest positive f32 subnormal scale passes; positive/negative zero, negative finite/subnormal, positive/negative infinity, qNaN, and sNaN fail.
- Equivalent packed and byte-addressed logical views produce the same semantic diagnostic index while unused tail bits remain unobserved by the semantic scan.
- Direct input, Quantize-produced, and Assemble-produced values use distinct proof construction paths but one conformance vocabulary.
- Packed Boolean, other sub-byte layouts, complex, codebook, hierarchical/MX, mask/outlier, nested, sparse, ragged, private, and incoherent representations refuse by exact type/scheme/map/encoding until an admitted evaluator exists.
- Perturb every subject/view/type/role/map/version/coherence/validator/dependency field and observe evidence reuse fail before restoration.

## Closes when

The public value-conformance evidence and adapter binding boundary has Tom's required interface review; the selected direct strict-affine U4/U8 bindings are validated over exact logical views; internally produced proof composition is distinct from direct input validation; every unsupported representation is named; typed failure ordering and no-fallback behavior are demonstrated; every new check is fault-proved; targeted package tests and Clippy pass; `tkt lint`, `git diff --check`, and one batch gate pass.

## Graph maintenance

- Make `implement-first-runtime-semantic-value-precondition-enforcement` depend on this ticket before claiming direct encoded-value execution.
- Update artifact ABI preflight prose to distinguish logical value conformance from physical packed-tail canonicality and post-routing operation-precondition enforcement.
- Update the dtype maturity matrix only for exact resolved-value binding cells proven here.
- Add new scheme-specific conformance tickets rather than widening strict-affine evaluators by resemblance.

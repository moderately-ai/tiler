---
id: enforce-resolved-encoded-value-binding-conformance
title: Enforce resolved encoded-value binding conformance
status: review
priority: p1
dependencies: [prototype-quantized-value-vertical, produce-typed-strict-affine-assemble-semantic-precondition]
related: [implement-first-runtime-semantic-value-precondition-enforcement, produce-typed-strict-affine-assemble-semantic-precondition, own-the-dtype-support-maturity-matrix]
scopes: [implementation/ir, implementation/artifact, implementation/reference, implementation/runtime, contracts/foundation, contracts/numerics, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, validation, compound-values, runtime]
claimed_from: todo
assignee: agent-value-conformance
lease_expires_at: 1785876737
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

## Outcome (2026-08-04)

**The contract is derived from the type, and there is exactly one authority for it.** `ResolvedValueConformanceContract::derive` reads the complete obligation set out of an admitted `ResolvedValueType` and its logical shape: exact scheme, ordered component declarations, roles, component resolved types, derived component shapes, parameter maps, and the value domains the governed contract fields name (`ENCODED_NUMERIC_CODE_MIN`/`_MAX` for the inclusive code range, `ENCODED_NUMERIC_SCALE_DOMAIN` for `positive-normal-f32`). The structural half is generic over whatever roles a scheme declares; only the domains are scheme knowledge. `check_bound_value` discharges that set against a consumer-supplied logical view, and the reference evaluator's registered strict-affine and unsigned-code validators now reach it rather than restating a domain — so narrowing a scheme narrows both paths at once and they cannot drift into disagreeing about what a valid value is.

**Unsupported representations refuse by exact name.** `UnsupportedValueRepresentation` names the refused subject in each of five classes: no admitted contract for a logical type (packed Boolean's logical type, complex, sparse, ragged, and any admitted scalar with no governed domain), an unadmitted encoded scheme (a codebook scheme, a hierarchical microscaling scheme), the admitted scheme under a contract this validator has not admitted, a component type with no admitted logical scan (mask/outlier carriers), and a parameter map with no admitted evaluator. Nested components are refused at the `EncodedNumericContract` constructor that owns them. `LogicalViewFault` names inaccessible memory, absent coherence, an unreconstructable logical view, and an unrepresentable scalar. Nothing is approximated through a resembling scheme or a wider integer.

**The scan is logical, and the diagnostic coordinate is logical.** `EncodedLogicalView` exposes a logical scalar at a canonical row-major logical index and offers no method by which padding, alignment, bit order, or an unused packed tail could be observed. The retained test reconstructs five logical nibbles from a three-byte packed carrier and from byte-addressed elements: both produce byte-identical evidence, both report the identical refusal at the identical logical index when the value is invalid, and a deliberately noncanonical tail changes neither — with the packed view counting past-the-end reads so "the tail is unobserved" is a counted fact rather than a claim. A refusal reports the minimum of `(logical index, invalid-input code, component ordinal)`; a resource shortfall is refused against `MAX_CONFORMANCE_SCAN_ELEMENTS` before the first read, asserted by a read counter at zero.

**Evidence binds provenance and cannot be forged or transplanted.** `ValueConformanceSubject` carries origin (interface key, or completed occurrence coordinate and ordered result position), stability (`ImmutableHost`, or a `ValueVersion` and `CoherenceEpoch`), route dependency, logical view, complete type, and shape; the evidence encodes those plus the static validator key and revision and the derived obligations. Perturbing any one — including the validator revision — is a different subject the proof does not authorize, and each perturbation also changes the durable canonical bytes, so the property survives a process boundary rather than resting on `PartialEq`. Two values with identical payloads under two interface keys mint different evidence and neither authorizes the other, which is what pointer equality and slot position would both get wrong.

**Three proof paths, one vocabulary, and no rescan.** A direct binding is scanned. `Assemble` carries the conformance of its codes and zero-point operands into its result; `Quantize` carries only its zero point, because its codes are established by its own declared clamp-then-nearest-even semantics; the scale component of either is established by the occurrence's discharged normal-scale precondition. `SemanticPreconditionsDischarged::for_occurrence` cannot be minted while a residual is undischarged and refuses an obligation belonging to another occurrence, so a composed proof is a proof rather than an assumption. Composition reads no payload, and `conform_bound_value` returns a complete same-provenance proof without touching the view — asserted by a read counter, together with a changed provenance being rescanned and a stale proof failing to rescue an invalid payload.

**Identity: no pin moved.** The validator key and revision are static and identity-bearing *in the execution-scoped evidence*, not in static artifact identity. Folding them there would be an identity-domain step — `ARTIFACT_DOMAIN` v14 to v15 and `MANIFEST_SCHEMA` 12.0 to 13.0 — for a field no artifact producer can fill, since nothing an artifact carries selects a validator. The first artifact-side consumer is the enforcement plan, and `carry-semantic-enforcement-plans-through-program-and-artifact` owns introducing it and stepping the domain. Dynamic bytes, value version, and coherence epoch stay out of static identity by construction. No provider revision was advanced: the semantic definition projection and admission provenance are unchanged, and the reference provider registers no new capability and changes no accept/reject verdict — the five reference refusals that moved became *more* specific under the same verdict.

**Boundary this ticket did not cross, and its owner.** Reconstructing a logical view from an artifact's declared `StorageEncoding` and enforcing it at the runtime binding site is `implement-first-runtime-semantic-value-precondition-enforcement`'s, which the `prototype-quantized-value-vertical` outcome already assigns it ("owns logical-view reconstruction and validation across packed boolean/sub-byte, complex, quantized, extension, sparse, and ragged representations") and which already depends on this ticket. What this ticket delivers is the contract, the evidence, the scan, and the composition it will call. Physical packed-tail canonicality remains unowned by any reachable code: `PackedTailRule::Zero` is declared and identity-bearing, and its only implementation is `tiler-compiler`'s `unpack_codes`, which has no non-test caller.

**Fault-proofs.** Seventeen perturbations were applied one at a time, run, observed failing, and restored: the component role check; the component count check; the inclusive code-domain bound; `is_normal` weakened to `is_finite`; the minimum-diagnostic selection inverted; the scan bound extended one element past the logical end; `stability` dropped from the evidence encoding; the same-provenance reuse branch removed; the unadmitted-scheme guard disabled; the scan-budget check removed; the residual-discharge requirement bypassed; the operand route binding disabled; `Quantize`'s codes rule changed from operation semantics to operand conformance; the reference bind-input path short-circuited; the reference element-width check replaced by a first-byte read; `Assemble`'s carried codes operand moved to the wrong position; and the reference validator's delegation to the shared authority removed.

## Remaining before this ticket closes

The public value-conformance evidence and adapter binding boundary needs Tom's interface review. The ticket sits at `review` with the complete tested draft and one atomic question drafted.

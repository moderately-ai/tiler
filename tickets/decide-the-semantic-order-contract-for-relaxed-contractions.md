---
id: decide-the-semantic-order-contract-for-relaxed-contractions
title: Decide the semantic order contract for relaxed contractions
status: done
priority: p1
dependencies: []
related: [decide-the-algebraic-capability-authority-for-contraction-splits, admit-reassociated-contraction-schedule-alternatives, decide-the-adr-0013-plan-determinism-stability-subject, implement-the-adr-0013-plan-determinism-stability-subject]
scopes: [implementation/ir, implementation/compiler, implementation/reference, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, numerics, identity, public-boundary, needs-tom]
---
## User-visible outcome

Tiler decides whether `tiler::strict-tensor-contraction-f32@1` remains a single strict-fold value under every numerical contract or is completely replaced in the standard vertical by a new operation identity with an explicitly permission-indexed order contract. The answer must settle the semantic facts, reference oracle, operation identity, and unsupported population before any algebraic capability or physical split can be admitted.

This is a prerequisite decision, not implementation authorization. Do not queue it for Tom until its own exact-base Fact audit and Pareto-complete packet have been independently reviewed.

## Source-first filing evidence — exact base `fe60f992cc20b37a52aff815897170516490667a`

**Fact — current registered meaning is strict and singular.** `register_standard_contraction` in `crates/tiler-ir/src/semantic/contraction.rs` names `binary32 products folded strictly in ascending lexicographic order`; `contraction_f32_facts` sets `CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED` and `CONTRACTION_F32_FACT_PERMUTATION_PERMITTED` to `false`; and its registration comment says declaring ordered associativity would hand a rewrite facts the family forbids.

**Fact — the independent reference treats either freedom as a different semantic population.** `crates/tiler-reference/src/contraction.rs`, anchor `The three permissions`, requires the contraction definition's arithmetic-contraction, reassociation, and permutation facts all to be `false`. `a_declaration_this_reference_does_not_compute_is_refused_by_field` perturbs each to `true` and requires the field-specific refusal. `ReferenceNumericalConformance::from_realization` in `crates/tiler-reference/src/conformance.rs` separately refuses permitted reassociation and permutation rather than evaluating the strict value and mislabelling it as the requested result set.

**Fact — accepted ADR prose leaves a future conditional route but does not instantiate it.** [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md), anchor `unless a registered permission authorizes otherwise`, accepts one keyed contraction family with a strict lexicographic fold unless a registered permission authorizes another order. [ADR 0014](../docs/decisions/0014-reassociation-vs-permutation.md), anchor `Each transformation requires two independent facts`, requires algebraic capability and numerical permission independently. Neither accepted record changes the current definition's two `false` values or installs a result-set oracle merely by describing the future seam.

**Fact — the compiler says this family can consume the dimensions but does not grant them.** `const TENSOR_CONTRACTION` in `crates/tiler-compiler/src/policy.rs` includes reassociation and permutation so a target/request is asked about freedoms capable of changing the fold. That table is numerical applicability, not semantic or algebraic authority. The distinction is exposed by the source anchor `would answer false` in `crates/tiler-compiler/src/fusion_legality.rs`: classifying the contraction as permission-free under a reassociating contract would silently accept the regrouping its own facts forbid.

**Fact — a capability-only repair is insufficient.** [`decide-the-algebraic-capability-authority-for-contraction-splits`](decide-the-algebraic-capability-authority-for-contraction-splits.md), anchor `Decision result — no current authorizing capability`, proves that an operation-wide capability names the wrong operands and a capability on the realized scalar add cannot override the semantic operation the realization refines. Both fixed split memberships therefore remain unavailable until this ticket decides whether a relaxed semantic population exists at all.

## Exact decision this ticket owns

Decide one coherent semantic contract across these inseparable subjects:

- whether the existing key stays point-valued under every request, is incompatibly redefined, coexists with another key, or is completely replaced in the standard vertical by a successor identity that denotes a strict value under forbidden permissions and a typed result set under permitted reassociation/permutation;
- whether the two current boolean fact fields remain booleans, change value, or are replaced by a typed order-contract declaration, including their exact validation and canonical encoding;
- how an effective numerical permission intersects with that operation-owned declaration without a request overriding a restriction or a definition granting a caller freedom it did not request;
- which reference question is answered for a permitted contract: membership in an exact finite/result-set oracle, a topology-parameterized evaluator, another sound witness, or typed refusal—and the boundedness/unsupported cases of that answer;
- whether changing the current definition under the same `OpKey` is compatible with ADR 0087, ADR 0072, provider-independent definition identity, and concrete artifact joins, or correctness requires a new operation identity;
- canonical-NaN, signed-zero, subnormal, infinity, FMA/contraction, accumulator, empty-domain, seed, determinism, and distributivity consequences; and
- the complete semantic-registry, scalar/law/lowering registry, request, explain, refinement, program, artifact, cache, schema, and pin cascade, including `CanonicalReferenceRegistryIdentity` over the semantic snapshot and `CanonicalScalarReferenceRegistryIdentity` over the scalar snapshot.

This ticket must leave the later algebraic-authority decision with a fixed semantic subject. It must not choose the scalar/semantic combiner API itself.

## Required Pareto frontier

At minimum compare, and eliminate only with source-backed reasons:

1. keep `tiler::strict-tensor-contraction-f32@1` strict forever and close both split strategies;
2. keep the one key but make its order contract permission-indexed, preserving the exact strict answer when both freedoms are forbidden;
3. keep the one key and admit reassociation only, leaving permutation unavailable, if that narrower result set has a strictly smaller sound oracle/identity surface;
4. compare a coexisting relaxed key with a complete operation-key successor replacement, including exact namespace/name/version spelling, the accepted one-key rationale, immutable-key discipline, and every frontend/reference/lowering/artifact consumer;
5. replace the current boolean facts with a typed internal reducer/result-set descriptor if booleans cannot express the needed authority without contradiction;
6. perform bounded numerical/reference research with exact stop conditions; and
7. defer with the current typed refusal and an evidence-based reopening trigger.

Eliminate any option that lets request permission overwrite an operation restriction, infers an internal combiner from prose or definition facts, returns the strict oracle value for a result-set request, conflates reassociation with permutation, changes graph meaning without moving canonical graph bytes, or claims a complete result while leaving reference/conformance support implicit.

## Required independent derivations and perturbations

- Derive the legal result population from the definition, not from the measured split kernels. Cover all F32 bit patterns or state the exact bounded corpus; canonical NaN payload/order, both signed zeros, infinities, subnormal preserve and each FTZ zero-sign mode, separate multiply/add rounding, and every excluded FMA/distributive rewrite must be explicit.
- Derive operation key/definition/snapshot and downstream identity movement independently from the reference design. A same-key semantic-definition change moves the semantic snapshot embedded in `CanonicalReferenceRegistryIdentity`; a same-key scalar-definition change moves the scalar snapshot embedded in `CanonicalScalarReferenceRegistryIdentity`. Those outer values and pins move even if their domain tags and reference-provider revisions do not. A matrix entry that says “domain unchanged” must still say whether values and pins move.
- Perturb only the same-key semantic definition and show `CanonicalReferenceRegistryIdentity` plus its pins move while its domain tag and reference-provider revisions stay fixed. Perturb only the same-key scalar definition and show `CanonicalScalarReferenceRegistryIdentity` plus its pins move under the same controls.
- Perturb each of the two current `false` facts independently and show which semantic/reference check fails. Perturb the effective permission independently to prove operation declaration and request ceiling cannot substitute for each other.
- If a result-set oracle is proposed, perturb contributor membership while holding partition counts and merge order fixed, and show the contiguous/lane-strided distinction is observable. Perturb grouping while preserving leaf order separately from permutation.
- If the existing key is retained, demonstrate why an old strict occurrence and a new permission-indexed occurrence cannot collide at every concrete semantic/request/program/artifact join, including joins that correctly inspect only the graph subject. If a new key is proposed, compare coexistence with a complete standard-vertical replacement and derive the exact key spelling rather than treating `@2` as an unscoped generation number.

## Consequences and non-goals

This ticket does not implement the accepted physical carrier or either split, add algebraic capability vocabulary, grant a caller numerical permission, infer authority from the current scalar lowering, mutate the preserved failed attempt, admit distributivity/FMA/nondeterminism, or make a kernel-performance claim.

The present executable outcome remains the strict direct contraction only. Host runtime and memory are unchanged until Tom accepts a different semantic population and separate implementation work lands.

## Downstream graph and release conditions

[`decide-the-algebraic-capability-authority-for-contraction-splits`](decide-the-algebraic-capability-authority-for-contraction-splits.md) depends directly on this ticket; [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md) depends on the algebraic-authority ticket and therefore inherits this prerequisite transitively. Admission also depends directly on [`implement-the-adr-0013-plan-determinism-stability-subject`](implement-the-adr-0013-plan-determinism-stability-subject.md), which depends on [`decide-the-adr-0013-plan-determinism-stability-subject`](decide-the-adr-0013-plan-determinism-stability-subject.md). If the accepted answer keeps the key strict, close or supersede both split outcomes with the exact refusal. If it admits a relaxed population, reopen the algebraic-authority ticket at the accepted commit; only that reopened ticket may choose the exact operation/combiner capability and verifier join, and admission remains blocked until the determinism carrier is complete.

## Closes when

Tom has accepted one exact semantic order/result-set contract, with the strongest counterargument and reversal evidence for every frontier survivor, and the complete reference, identity, schema, unsupported-population, and downstream graph consequences are explicit. Only then can an algebraic capability question be meaningful rather than contradictory.

## Exact-base Fact audit — `368dcd255079162c116716ca807cb0f23dfa3a59`

Audited 2026-08-17 before editing this packet, then re-audited after independent review found the artifact join below. The ticket was filed against `fe60f992cc20b37a52aff815897170516490667a`; every Fact was re-read from source at this branch's exact base. The new evidence changes the recommendation and option dispositions, not the ticket's purpose: operation identity was already an owned decision subject.

1. **Verified — current registered meaning is strict and singular.** `register_standard_contraction`, anchor `binary32 products folded strictly in ascending lexicographic order`, remains the normative definition. `contraction_f32_facts` still writes `false` at both `CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED` and `CONTRACTION_F32_FACT_PERMUTATION_PERMITTED`. The registration comment at `No algebraic capability is declared, deliberately` still explains why the operation declares none — declaring ordered associativity there `would hand a rewrite the numerical facts below forbid` (the phrase "ordered associativity" itself wraps across a comment line, so grep the fragments quoted here, not the rendered sentence).
2. **Verified — the independent reference treats either freedom as another semantic population.** `ContractionContract::decode`, anchor `The three permissions`, requires arithmetic contraction, reassociation, and permutation all to be false. `a_declaration_this_reference_does_not_compute_is_refused_by_field` perturbs every declaration field independently. `ReferenceNumericalConformance::from_realization` still returns `ReassociationPermitted` or `PermutationPermitted` before any strict evaluator can answer a result-set request with one value.
3. **Verified — accepted ADRs leave a conditional seam and instantiate no relaxed contraction.** ADR 0087 still says `unless a registered permission authorizes otherwise`; ADR 0014 still says `Each transformation requires two independent facts`. Neither source changes the two current false facts or supplies an algebraic capability/reference result-set implementation.
4. **Verified — compiler applicability is not semantic authority.** `const TENSOR_CONTRACTION` in `policy.rs` still includes reassociation and permutation. The fusion-legality anchor `would answer false` still shows why treating the family as permission-free under a permissive request could silently admit arithmetic its own definition forbids. The policy table decides which questions apply; it does not answer them.
5. **Verified — the related algebraic decision cannot repair semantics by itself.** `Decision result — no current authorizing capability` still concludes that operation-wide capability names the tensor operands rather than the internal fold and that a realized scalar add cannot override the semantic operation it refines. Both fixed memberships remain unavailable at this base.
6. **Verified — a graph-meaning change must move canonical graph bytes, and the artifact builder relies on that rule.** [ADR 0072](../docs/decisions/0072-separate-semantic-meaning-from-provider-provenance.md), anchor `change to graph meaning requires changed canonical graph bytes` (the leading "A" sits on the previous source line), requires a new semantic type or operation-key version for changed graph meaning. `ArtifactProgramBuilder::new` retains the complete `SemanticIdentity`, but `ArtifactProgramBuilder::push_variant` correctly joins a `VerifiedKernelProgram` to it by `semantic_graph_identity()` alone. A same-key definition-only change therefore leaves a concrete old/new hybrid join possible; this is not a deficient consumer that may be assumed away.
7. **Verified — the accepted partition is not an arbitrary-tree witness.** `ContributorPartition`, anchor `Contributor order is preserved` (the rendered "partition `p` covers the contiguous contributor range" carries inline code markup and a line break, so it fails as a byte anchor), carries only a regular contiguous partition count and contributors-per-partition. It can derive the currently accepted split tree, but cannot encode every full ordered binary tree in the proposed semantic result set.
8. **Verified — the determinism string is not a realized stability contract.** [ADR 0012](../docs/decisions/0012-physical-reduction-topology.md), anchor `Unrealized — the explicit stability scope for deterministic order`, states that `plan-deterministic` does not yet define which executions, artifacts, targets, or toolchains share the promise. [ADR 0013](../docs/decisions/0013-scoped-determinism.md) supplies the normative scope but leaves the target-environment compatibility identity unresolved. A relaxed implementation therefore needs a separate prerequisite rather than treating the fact string as proof.
9. **Verified — the ordinary semantic reference evaluator has no topology carrier.** `ReferenceEvaluator`, anchor `An evaluator carries one` (the type name that follows is doc-link markup in the source bytes), passes only that `ReferenceNumericalConformance` into `ReferenceEvaluationRequest`. `ReferenceNumericalConformance` stores only subject plus input/result subnormal modes, and `from_realization`, anchor `return Err(UnsupportedReferenceContract::ReassociationPermitted)`, refuses the relaxed contract before ordinary evaluation. A distinct ordinary no-witness relaxed error would therefore be unreachable without a broader public constructor this packet does not need.
10. **Verified — current live-extent contraction has no finite tree at plan-construction time.** `ReductionTopology::LiveContraction`, anchor `contracted trip count is the named input axis`, carries runtime `K`. The proposed finite postorder witness deliberately returns `LiveContributorCount`; its first implementation covers only static direct/regular-split topologies and cannot claim the live direct topology.
11. **Verified — an operation-key replacement leaves shape-environment identity unchanged.** `SemanticIdentity` has five subjects, but `shape_environment()`, anchor `environment symbolic extents resolve in`, is built from bindings, root-binding provenance, and constraints rather than operation keys. Graph, reached definitions, admission provenance, and registry snapshot move; `ShapeEnvIdentity` does not.

Complete governing sources read for this audit include ADRs 0009, 0011, 0012, 0013, 0014, 0015, 0018, 0019, 0022, 0034, 0044, 0072, 0087, and 0095; `docs/numerical-semantics.md`, `docs/correctness-and-testing.md`, and `docs/operation-extensions.md`; the semantic contraction definition and registry/identity encoders; semantic and scalar reference registries and evaluators; numerical conformance; policy, fusion, request, lowering, law, refinement, program, artifact, explain, and cache construction, verification, consumption, and refusal paths; the correctness-bearing contraction tests; the related algebraic-authority decision; and the accepted fixed-membership carrier. Searchable anchors, not stale line numbers, are authoritative.

## Decision readiness result

**Inference — two nondominated product choices remain.** Keeping `tiler::strict-tensor-contraction-f32@1` strict forever is maximally narrow and has no new public or reference surface. If Tiler wants the already demanded ordered regrouping capability, correctness requires a complete standard-vertical replacement by a new key; a same-key semantic mutation is not a survivor. The relaxed replacement is still fail-closed for every freedom the operation or caller withholds, but pays a typed semantic/reference/witness surface. The capability-versus-surface trade-off is genuine, so Tom retains one product decision.

**Recommendation — replace the standard key with `tiler::tensor-contraction-f32@1` and admit ordered reassociation only.** Every standard frontend, registry, reference, law, lowering, fixture, and test moves atomically; the old strict key does not coexist as a second frontend-selectable family. Under a request withholding reassociation, the successor has exactly the old strict left-fold result. Under a request permitting reassociation, it denotes all full ordered binary trees over the unchanged canonical leaves. Permutation remains operation-owned unsupported. The fired contiguous-split demand justifies the surface; its measurement is motivation, not a kernel-performance promise.

**Draft Tom question — deliberately unqueued pending review.** Should Tiler keep `tiler::strict-tensor-contraction-f32@1` strict forever and close both fixed split outcomes, or retire it from the standard vertical and replace it with `tiler::tensor-contraction-f32@1`, whose strict request cell preserves the old answer and whose reassociation-permitted cell denotes the exact ordered-tree result set below while permutation remains unsupported? **Recommendation: the complete replacement.**

This packet does not move status, queue the question, accept the draft public surface below, or authorize production implementation.

## Recommended semantic and identity contract

### Complete operation-key replacement

The successor key is exactly `OpKey::new("tiler", "tensor-contraction-f32", 1)`, exposed by `tiler_ir::semantic::tensor_contraction_f32_op()`. `strict_tensor_contraction_f32_op()` and `strict_tensor_contraction_f32_facts()` are removed with the old standard registration. The old key is absent from the standard semantic, reference, law, lowering, compiler-recognition, and frontend vertical; no alias, equivalence rule, fallback, or duplicate selection policy exists. Generic historical bytes can remain decodable, but an installed standard compiler/reference has no authority to compile or execute that retired operation.

The spelling is Pareto-derived, not cosmetic:

| Candidate | Disposition |
| --- | --- |
| `tiler::strict-tensor-contraction-f32@2` | Rejected: it preserves lineage but its public name falsely says every permitted cell is strict. |
| `tiler::tensor-contraction-f32@2` | Rejected: a semantic version is scoped to namespace plus name; generation 2 of a newly introduced name invents an absent generation 1. |
| `tiler::tensor-contraction-f32@1` | Selected: a truthful neutral name, conventional first generation of that named family, and an explicitly documented successor to the retired `tiler::strict-tensor-contraction-f32@1`. |

ADR 0087's one-family rationale is preserved: a frontend still emits one contraction key and never selects strict versus relaxed families. ADR 0034/0072 immutable-meaning discipline is also preserved: subsequent incompatible meaning changes require another key generation and must not mutate this key in place.

### Exact typed definition descriptor

The successor keeps the structural meanings of outer fact fields 1–7 and 10–14, retires old Boolean fields 8 and 9 without reuse, and adds the sole public constant `CONTRACTION_F32_FACT_REDUCTION_DESCRIPTOR = AttributeFieldId::new(15)`. The resulting outer record has exactly thirteen fields. Field 15 is a canonical record with these private schema-local IDs and exact closed values:

| Inner ID | Meaning | Exact successor value |
| --- | --- | --- |
| 1 | leaf primitive | `input-transform-each-factor-round-binary32-nearest-ties-even-multiply-canonicalize-nan-result-transform` |
| 2 | reducer primitive | `input-transform-each-addend-round-binary32-nearest-ties-even-add-canonicalize-nan-result-transform` |
| 3 | result-class rule | `strict-left-fold-or-ordered-full-binary-trees-by-effective-reassociation` |
| 4 | maximum reassociation | `permission-gated` |
| 5 | maximum permutation | `unsupported` |
| 6 | maximum signed-zero elimination | `unsupported` |

All six inner fields are required once, unknown fields refuse, and IDs are never reused. `CanonicalValue::record` already rejects duplicate IDs before this decoder can receive a value; the decoder does not need a misleading unreachable duplicate arm.

Outer field 14 no longer carries the ambiguous UTF-8 atom `plan-deterministic`. Under the successor key it is a required seven-field canonical record whose closed values say: scope `plan-deterministic`; equal inputs `same-input-bits-and-runtime-bindings`; artifact `same-artifact-digest`; plan `same-selected-plan-variant`; environment `same-declared-target-environment`; result `identical-output-bits`; and recompilation boundary `different-artifact-may-select-a-different-legal-result`. Private inner IDs 1–7 encode that order. This makes the ADR 0013 scope part of provider-independent definition bytes rather than an explanatory convention.

The exact draft public boundary in `tiler_ir::semantic` is:

- exhaustive enums `ContractionF32ContributorSequence::{AscendingLexicographicCanonicalContractedIndexSpace}`, `ContractionF32LeafPrimitive::{TransformOperandsRoundBinary32NearestTiesEvenMultiplyCanonicalizeNanTransformResult}`, `ContractionF32ReducerPrimitive::{TransformOperandsRoundBinary32NearestTiesEvenAddCanonicalizeNanTransformResult}`, `ContractionF32Seed::{FirstProduct}`, `ContractionF32EmptyDomain::{Refused}`, `ContractionF32OrderFreedom::{Unsupported, PermissionGated}`, `ContractionF32ResultClass::{StrictLeftFold, OrderedFullBinaryTrees}`, `ContractionF32NanCanonicalization::{AfterEachArithmeticOperationAndResultBoundary}`, and `ContractionF32StabilityScope::{PlanDeterministic}`;
- opaque `ContractionF32ReductionDescriptor` with read-only accessors `contributors() -> ContractionF32ContributorSequence`, `leaf() -> ContractionF32LeafPrimitive`, `reducer() -> ContractionF32ReducerPrimitive`, `seed() -> ContractionF32Seed`, `empty_domain() -> ContractionF32EmptyDomain`, `reassociation() -> ContractionF32OrderFreedom`, `permutation() -> ContractionF32OrderFreedom`, `signed_zero_elimination() -> ContractionF32OrderFreedom`, `arithmetic_contraction_supported() -> bool`, `distributivity_supported() -> bool`, `canonical_nan_bits() -> u32`, `nan_canonicalization() -> ContractionF32NanCanonicalization`, and `stability() -> ContractionF32StabilityScope`;
- `ContractionF32ReductionDescriptor::decode(definition: &OperationDefinition) -> Result<Self, ContractionF32DescriptorError>` and `tensor_contraction_f32_reduction_descriptor(registry: &FrozenSemanticRegistry) -> Result<ContractionF32ReductionDescriptor, ContractionF32DescriptorError>`; and
- no public constructor from parts, mutable field, raw-text accessor, topology field, target field, or fallback decoder.

`ContractionF32DescriptorField::{Outer(AttributeFieldId), Reduction(AttributeFieldId), Stability(AttributeFieldId)}` identifies error paths. `ContractionF32DescriptorError` is exhaustive with `OperationMissing { operation: OpKey }`, `WrongOperation { expected: OpKey, actual: OpKey }`, `MalformedFacts { actual: CanonicalValueKind }`, `FactCount { expected: usize, actual: usize }`, `MissingField { field }`, `UnexpectedField { field }`, `WrongKind { field, expected: CanonicalValueKind, actual: CanonicalValueKind }`, `UnsupportedValue { field }`, and `ContradictoryFields { first, second }`. Standard registration runs this decoder and maps a failure to the new `RegistryError::InvalidGovernedContractionDescriptor { source }`; it never registers an untyped governed definition.

The decoder validates every surviving outer fact, not only field 15: F32 computation/accumulator/result/conversion, canonical contributor sequence, first-product seed, empty refusal, absent distributivity, forbidden arithmetic contraction, `0x7fc00000`, arithmetic-NaN canonicalization after each combine and at the boundary, and the ADR-0013-bound stability tag. A second compiler/reference decoder is forbidden. The descriptor names arithmetic meaning but does not name `tiler::add-f32@1`, `tiler.scalar::add-f32@1`, or choose the later algebraic-capability owner.

### Effective profile: operation maximum intersected with caller ceiling

**Fact — the ceiling's `NumericalRealization` cannot be rewritten under its existing key.** `docs/numerical-semantics.md`, anchor `the contract key is derived, not chosen`, says the key is the injective encoding of the dimension vector and names scheduled `profile_key` as a consumer that relies on the key alone. `NumericalRealization`, anchor `Stable key of the governing numerical contract`, carries that key beside the dimensions it names. Copying the key while forcing three fields would manufacture a value that no longer matches its contract identity.

`tiler_ir::schedule` therefore gains opaque, `Copy` `EffectiveContractionF32Profile`, constructible only by `ContractionF32ReductionDescriptor::resolve(ceiling: NumericalRealization) -> Result<EffectiveContractionF32Profile, EffectiveContractionF32ProfileError>`. It stores the raw ceiling byte-for-byte plus the derived result class. It exposes `ceiling() -> NumericalRealization`, `result_class() -> ContractionF32ResultClass`, `permits_arithmetic_contraction() -> bool`, `permits_reassociation() -> bool`, `permits_permutation() -> bool`, and `permits_signed_zero_elimination() -> bool`. There is no `new`, `Default`, mutable field, conversion back to `NumericalRealization`, or path from a raw request that bypasses the descriptor.

`EffectiveContractionF32ProfileError` is exhaustive and has exactly one variant: `CanonicalNanMismatch { expected: u32, actual: u32 }`. Descriptor malformation is impossible at this boundary because `resolve` is a method on the already decoded opaque descriptor; unsupported operation freedoms are effective `false` values, not construction errors.

Resolution is exact:

- retain the complete ceiling unchanged, including its profile key and all eight dimensions;
- require the ceiling's canonical arithmetic NaN bits to equal descriptor `0x7fc00000`, else `EffectiveContractionF32ProfileError::CanonicalNanMismatch { expected, actual }`;
- derive the effective arithmetic-contraction, permutation, and signed-zero accessors as `false` because the operation maximum forbids them, without altering the stored ceiling;
- derive effective reassociation as true only when descriptor maximum is `PermissionGated` and the stored ceiling permits it; and
- derive `StrictLeftFold` for effective forbidden reassociation and `OrderedFullBinaryTrees` for effective permitted reassociation.

This is an operation-specific view over an immutable declared contract: a request may withhold a supported freedom and cannot grant an unsupported one, while `ceiling().profile_key` always remains truthful about `ceiling()`'s own fields. The future per-occurrence restriction ADR 0011 mentions does not exist at this base and is not defaulted into this API; admitting it later must change the resolver signature and identities explicitly. Schedule retains the raw `NumericalRealization` for contract identity and, for this occurrence, consumes or rederives the effective carrier before legality/reference decisions; it never treats the raw ceiling alone as operation authority. Algebraic capability remains a separate later prerequisite for a physical regrouping.

## Exact F32 result population

**Fact — the existing evaluator fixes the primitive arithmetic sites.** `ContractionFold::evaluate_outputs`, anchor `One rounding for the product and one for the accumulation`, applies input-subnormal handling to every factor, product, and accumulator entering an operation; applies result-subnormal handling after every multiplication and addition; canonicalizes every arithmetic NaN after each product/add and at the result boundary; and begins an unseeded fold with its first product.

Define three transforms from the effective profile:

```text
I(x)       = the resolved input-subnormal transform of x
R(x)       = the resolved result-subnormal transform of x
N(x)       = 0x7fc00000 when x is NaN, otherwise x
P(a, b)    = R(N(round_f32_RNE(I(a) * I(b))))
A(x, y)    = R(N(round_f32_RNE(I(x) + I(y))))
```

For one output with nonempty canonical contributor sequence `(a_i, b_i)` and leaves `p_i = P(a_i, b_i)`:

- `K = 0` is refused before a result exists;
- `K = 1` has the unique result `p_0`, independent of order permission;
- strict is `A(...A(A(p_0, p_1), p_2)..., p_(K-1))`;
- reassociation-only is the set of results of all full ordered binary trees whose in-order leaf traversal is exactly `p_0, ..., p_(K-1)`, at most Catalan number `C_(K-1)` after exact-bit duplicates are collapsed;
- permutation-only, if a future operation admitted it, is the strict left fold of each contributor permutation, at most `K!` results; and
- both freedoms, if a future operation admitted them, are every full binary tree over every permutation, at most `K! * C_(K-1)` results.

The successor admits only strict and reassociation-only. Every internal node is `A`; a tree cannot alter a leaf product, reverse multiplication factors, widen the accumulator, insert a zero, duplicate a seed, fuse multiply-add, distribute multiplication over a sum, or regroup a contraction chain.

For a tensor result, semantic membership is the pointwise product of the per-output sets. An implementation must additionally carry a deterministic plan-owned witness/function selecting one legal tree for every output coordinate. Scalar set membership is insufficient: a body claiming tree A must be checked against A even when its wrong result happens to equal a value tree B can produce.

### Exceptional values, rounding, empty domains, and determinism

- **NaNs:** input payload/sign bits reach the primitive unchanged except that input FTZ cannot affect a NaN. Every NaN product or sum becomes exactly `0x7fc00000` immediately and again at the boundary. Grouping may change whether a path reaches NaN, not which payload an arithmetic NaN returns.
- **Signed zero:** `-0 + -0` is `-0`; opposite zeros and exact round-to-nearest cancellation are `+0`. A tree may obtain a different zero sign only through its authorized additions. Reassociation never grants signed-zero elimination.
- **Infinities:** finite overflow and ordinary infinity arithmetic are included. `0 * infinity` and `+infinity + -infinity` produce the canonical NaN. No exceptional-value absence is inferred.
- **Subnormals/FTZ:** the definition covers the Cartesian three-by-three input/result population: preserve, flush preserving sign, and flush to always-positive zero. `I` applies to both operands of every multiply/add, including a partial re-entering an add; `R` follows every result.
- **Rounding/FMA:** every `P` has one round-to-nearest-ties-to-even F32 multiply and every `A` one separate F32 add. FMA is outside the result set even if the caller ceiling permits contraction.
- **Empty/seed/padding:** no `+0` seed exists and `K = 0` is invalid. A nonempty subtree starts at its first leaf because that leaf is part of the tree. Padding needs a separately proved neutral value and is not admitted here.
- **Determinism:** bind `PlanDeterministic` to ADR 0013 exactly: identical input bits and runtime bindings, the same artifact digest and selected plan variant, and the same declared target environment produce identical output bits. The same artifact/variant cannot select a tree by timing, arrival, or atomics. Recompilation or a different artifact may select a different legal tree and result.

The final determinism bullet is normative scope, not current implementation evidence. ADR 0012 records that the fact string is unrealized and ADR 0013 leaves target-environment compatibility fields open. [`decide-the-adr-0013-plan-determinism-stability-subject`](decide-the-adr-0013-plan-determinism-stability-subject.md) therefore owns the hard public/identity/schema decision prerequisite. Its structurally blocked carrier, [`implement-the-adr-0013-plan-determinism-stability-subject`](implement-the-adr-0013-plan-determinism-stability-subject.md), must then implement the accepted subject before artifact manifest or explain can claim the stability scope, selected topology identity, and target-environment compatibility identity. [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md) has a hard dependency on that carrier, so the claim is graph-enforced rather than advisory prose.

Distributivity remains absent under ADR 0095. Nothing here changes product semantics, contributor axes, or the logical graph, so it cannot authorize a contraction-chain rewrite.

## Exact topology witness and reference boundary

### Arbitrary ordered-tree representation and current plan binding

`ContributorPartition` remains the accepted compact carrier for a regular physical split, but is not used as the general result-set witness. `tiler_ir::schedule` gains:

```text
ContractionF32TreeNode::Leaf { contributor: u64 }
ContractionF32TreeNode::Add { left: u32, right: u32 }

OrderedContractionF32Tree::try_from_postorder(
    contributor_count: u64,
    nodes: Vec<ContractionF32TreeNode>,
    limits: ContractionF32TopologyLimits,
) -> Result<OrderedContractionF32Tree, ContractionF32TreeError>
```

`OrderedContractionF32Tree` is opaque with `contributor_count() -> u64`, `nodes() -> &[ContractionF32TreeNode]`, `root() -> u32`, and `depth() -> usize`. `ContractionF32TopologyLimits::new(max_nodes: usize, max_depth: usize) -> Result<Self, InvalidContractionF32TopologyLimits>` requires both nonzero and `max_nodes <= u32::MAX`, exposes `max_nodes() -> usize` and `max_depth() -> usize`, and has no `Default`. `InvalidContractionF32TopologyLimits::{ZeroNodes, ZeroDepth, NodeIndexCapacity { maximum: usize }}` is exhaustive.

Validation requires `K > 0`, checked `2K-1` node count, root last, children strictly earlier than their `Add`, every non-root referenced exactly once, root never referenced, every leaf ordinal `0..K` present exactly once, and no other ordinal. Each node derives a half-open contributor interval; `Add` requires the left interval to end exactly where the right begins, and the root must cover `0..K`. This simultaneously rejects cycles, DAG sharing, disconnected nodes, gaps, overlap, reversal, and permutation. Limits are checked before retaining the vector; arithmetic overflow is typed refusal.

`ContractionF32TreeError` is exhaustive: `EmptyContributors`, `NodeCountOverflow`, `NodeLimit { limit: usize, actual: usize }`, `DepthLimit { limit: usize, actual: usize }`, `NodeCount { expected: usize, actual: usize }`, `RootNotLast { root: u32 }`, `ChildNotEarlier { node: u32, child: u32 }`, `ReferenceCount { node: u32, expected: u32, actual: u32 }`, `ContributorOutOfRange { contributor: u64, count: u64 }`, `ContributorMultiplicity { contributor: u64, actual: u32 }`, `NonAdjacentChildren { node: u32 }`, and `RootCoverage { expected: u64, actual_start: u64, actual_end: u64 }`.

`tiler_ir::program` gains opaque `ContractionF32PlanWitness`, built only by:

```text
ContractionF32PlanWitness::from_program(
    semantic: &SemanticProgram,
    program: &VerifiedKernelProgram,
    occurrence: SemanticOccurrence,
    limits: ContractionF32TopologyLimits,
) -> Result<ContractionF32PlanWitness, ContractionF32PlanWitnessError>
```

It exposes `semantic_graph_identity() -> &SemanticGraphIdentity`, `kernel_program_identity() -> &CanonicalKernelProgramIdentity`, `occurrence() -> SemanticOccurrence`, and `tree() -> &OrderedContractionF32Tree`. It validates the semantic graph join, the reached contraction occurrence, its unique effective realization, and the verified program topology before deriving the tree. A static-`K` direct `ReductionTopology::Contraction` becomes the canonical left chain. An exact static-`K` regular split becomes a left chain inside each positive contiguous partition, followed by a left-chain merge of partials in ascending partition/round order. `LiveContraction`, padded coverage, lane-strided membership, unfixed arrival, and any topology whose exact binary combine tree cannot be derived are refused.

`ContractionF32PlanWitnessError` is exhaustive: `SemanticGraphMismatch`, `OccurrenceOutOfRange`, `WrongOperation { actual: OpKey }`, `OccurrenceNotCovered`, `MissingRealization`, `AmbiguousRealization`, `LiveContributorCount`, `MalformedPartition`, `PaddedCoverageUnsupported`, `PermutationUnsupported`, `ArrivalNotFixed`, `TopologyUnsupported`, `PerOutputTopologyUnsupported`, and `Tree(ContractionF32TreeError)`.

The first public surface supports one uniform tree template for every output coordinate of a **static-`K`** occurrence. That exactly matches the current static direct contraction and proposed exact regular static split. It does not match the accepted current `LiveContraction`, whose contributor count arrives from a runtime input-axis extent and is refused as `LiveContributorCount`; relaxed live-`K` conformance is unsupported in this packet. The semantic result set itself remains pointwise; a future live- or coordinate-dependent tree mapping must become identity-bearing in schedule/kernel/artifact encoding and gain a new witness representation. It cannot reuse `ContributorPartition`, call the static uniform constructor, or infer a mapping from backend code.

### Concrete reference evaluator, request, result, budget, and errors

The ordinary registered `ReferenceOperation` and `ReferenceEvaluator` remain strict-only, with their current request and conformance surfaces unchanged. `ReferenceNumericalConformance::from_realization` constructs the strict conformance for a strict raw realization, after which the ordinary evaluator returns the current exact tensor. A raw realization permitting reassociation returns the existing `UnsupportedReferenceContract::ReassociationPermitted` before an ordinary evaluator can be used. There is no ordinary no-witness relaxed route and no new ordinary operation-error variant; the explicit topology evaluator below is the only relaxed reference route, and its request always requires a witness. Neither route substitutes the strict member for a relaxed request.

`tiler-reference` adds a concrete, non-extensible first-vertical `ContractionF32TopologyEvaluator`. Its private constructor is reached by `FrozenReferenceRegistry::contraction_f32_topology_evaluator() -> Result<ContractionF32TopologyEvaluator, ContractionF32TopologyEvaluatorUnavailable>`, which requires the registered successor F32 signature, compatible reached semantic authority, and the standard reference provider/capability revision that owns both strict and topology evaluation. This design adds no unversioned callback and no second general registry role. Third-party topology-reference providers are explicitly outside the first surface; adding them later requires its own registry-role and identity review.

The exact call is:

```text
ContractionF32TopologyEvaluator::evaluate(
    &self,
    request: ContractionF32TopologyEvaluationRequest<'_>,
) -> Result<ContractionF32TopologyEvaluation, ContractionF32TopologyEvaluationError>

ContractionF32TopologyEvaluationRequest::new(
    semantic: &SemanticProgram,
    occurrence: SemanticOccurrence,
    operands: [&Tensor; 2],
    extent_bindings: &ExtentBindingContext,
    profile: EffectiveContractionF32Profile,
    witness: &ContractionF32PlanWitness,
    budget: ContractionF32ReferenceBudget,
) -> Self
```

The owned result exposes `tensor() -> &Tensor`, `into_tensor() -> Tensor`, `reference_registry_identity() -> &CanonicalReferenceRegistryIdentity`, `kernel_program_identity() -> &CanonicalKernelProgramIdentity`, and `occurrence() -> SemanticOccurrence`. The evaluator rechecks registry authority, the request semantic graph and occurrence against the witness, descriptor/profile, attributes, operands, live bindings, and witness before arithmetic. The result's kernel-program identity is the identity already bound inside the validated witness; the request supplies no second program identity to compare.

The caller owns and must pass `ContractionF32ReferenceBudget`; the crate never chooses a default. `new(max_arithmetic_steps: u64, max_topology_nodes: usize, max_topology_node_visits: u64, max_topology_depth: usize) -> Result<Self, InvalidContractionF32ReferenceBudget>` requires each allowance nonzero and `max_topology_nodes <= u32::MAX`; it exposes accessors with the same names and return types as those four parameters. `InvalidContractionF32ReferenceBudget::{Zero { resource: ContractionF32ReferenceResource }, NodeIndexCapacity { maximum: usize }}` is exhaustive. Resource units are exact: retained topology nodes are `2K-1` once; topology visits are `output_count * (2K-1)`; arithmetic steps are `output_count * (K + K-1)` (one multiplication per leaf and one addition per internal node); depth is the validated tree depth. Every sum/product uses checked arithmetic and all four bounds are preflighted before result allocation or arithmetic. Existing tensor-element, retained-byte, extent-binding, and output-window limits remain independently enforced and are not widened by this budget.

`ContractionF32ReferenceResource::{ArithmeticSteps, TopologyNodes, TopologyNodeVisits, TopologyDepth}` is exhaustive. `ContractionF32TopologyEvaluationError` is exhaustive: `SemanticSubjectMismatch`, `OccurrenceMismatch`, `ResultClass { expected: ContractionF32ResultClass, actual: ContractionF32ResultClass }`, `ExceptionalAssumptionUnsupported { dimension: NumericalDimension }`, `Witness(ContractionF32PlanWitnessError)`, `BudgetExceeded { resource, limit: u64, actual: u64 }`, `BudgetArithmeticOverflow { resource }`, and `Operation(ReferenceOperationError)`. `ContractionF32TopologyEvaluatorUnavailable` distinguishes `CapabilityMissing`, `ProviderMismatch`, `RevisionMismatch`, and `SemanticAuthorityMismatch`. Effective-profile construction makes forbidden contraction/permutation/signed-zero and canonical-NaN mismatch unrepresentable here; the evaluator requires `OrderedFullBinaryTrees` and refuses exceptional absence assumptions it cannot discharge. No error defaults to strict evaluation.

Evaluation is `O(output_count * K)` time and `O(K)` witness/traversal memory. It evaluates exactly the selected tree with `P` and `A`; it never enumerates the Catalan-sized semantic set. Exhaustive deduplicated result-set enumeration is allowed only as bounded conformance evidence for named small `K`, with the corpus and budget in the test. Unsupported populations are missing/untyped topology, current `LiveContraction` and every other runtime-`K` occurrence, coordinate-dependent topology, permutation, unfixed arrival, malformed coverage, `K = 0`, seed/padding, non-F32 arithmetic, FMA/distributivity, signed-zero elimination, exceptional-value absence assumptions the reference bridge cannot discharge, budget excess, or foreign semantic/reference authority.

The concrete topology evaluator is output-affecting behavior of the standard semantic reference provider. Its implementation must move `standard-reference@7` and the contraction capability revision from 7 to 8, replace the old key row with the successor row, and update pins. `tiler.reference-registry.v2` need not step because the existing operation/signature/authority/provider/revision encoding carries the changed row and behavior revision. `standard-scalar-reference@1`, scalar capability revision 1, and `tiler.scalar-reference-registry.v1` remain unchanged.

## Subject perturbations and negative controls

### Declaration and request controls

- Changing only current field 8 from `false` to `true`, with provider and request unchanged, makes `cargo test -p tiler-reference registry_identity_is_deterministic_and_revision_complete -- --nocapture` fail at construction with `UnsupportedContraction ... UnrealizableFact { field: AttributeFieldId(8) }`.
- Changing only field 9 produces the same typed construction failure for `AttributeFieldId(9)`.
- Leaving both facts false and perturbing only the request is covered by `a_contract_admitting_a_result_set_is_refused_by_name` and `every_new_dimension_is_accounted_for_by_the_reference_boundary`: reassociation returns `UnsupportedReferenceContract::ReassociationPermitted`; permutation returns `UnsupportedReferenceContract::PermutationPermitted`.

These current-state controls prove declaration and request are independent. Successor implementation must add the inverse controls: descriptor supports reassociation plus strict ceiling gives `StrictLeftFold`; permissive ceiling gives `OrderedFullBinaryTrees`; unsupported permutation remains forbidden under both ceilings; malformed descriptor never resolves.

### Grouping, membership, and permutation controls

The exact F32 fixture uses leaves `[2^24, 1, -2^24, 1]`. Holding two 2-leaf partitions and ascending merge fixed but changing only membership gives contiguous `1.0 / 0x3f800000` and lane-strided `2.0 / 0x40000000`. On `[2^24, 1, -2^24]`, preserving order but changing grouping gives left `0.0 / 0x00000000` and right `1.0 / 0x3f800000`. Holding a strict-left tree and multiset but changing order gives `[2^24, -2^24, 1] = 1.0 / 0x3f800000` and `[2^24, 1, -2^24] = 0.0 / 0x00000000`.

```sh
python3 -c 'import struct
f=lambda x: struct.unpack("!f",struct.pack("!f",x))[0]
b=lambda x: struct.unpack("!I",struct.pack("!f",x))[0]
a=lambda x,y:f(f(x)+f(y))
P=f(16777216.0); N=f(-16777216.0); O=f(1.0)
for k,v in [("membership.contiguous",a(a(P,O),a(N,O))),("membership.lane",a(a(P,N),a(O,O))),("grouping.left",a(a(P,O),N)),("grouping.right",a(P,a(O,N))),("order.original",a(a(P,N),O)),("order.permuted",a(a(P,O),N))]: print(k,repr(v),f"0x{b(v):08x}")'
```

The fixture proves why physical membership, algebraic commutativity, and numerical permutation permission are separate obligations.

### Same-key identity controls and the artifact hybrid

A semantic-only perturbation to the governed contraction definition moved the semantic registry snapshot from fingerprint `8713bca5743244ee` to `ade9f0ef5a2d1065`, `CanonicalReferenceRegistryIdentity` from `4a8beb980eb2e809` to `38959df97a87b85b`, and explain request qualifier from `17e0dd47e48b7c18` to `20c5a2ca9d9bb91a`, while the bare graph, scalar snapshot/reference identity, domain tags, and provider/capability revisions stayed fixed. A scalar-only perturbation left semantic subjects fixed, moved scalar snapshot `cafea63674e87c24` to `3c090fda627fb35c`, scalar reference identity `1b3b214b5f575ffc` to `6c306ffaad09697d`, and explain qualifier to `ad601507e9ec98eb`. These controls remain useful but do not prove every subject join compares the nested values.

**Measurement — independent review reproduced the missing join.** A disposable `definition_generation_perturbation_reaches_artifact_graph_only_join` test built two governed semantic registries with identical keys/graphs and different reached definition projections, then offered a generation-2 `VerifiedKernelProgram` to a generation-1 `ArtifactProgramBuilder`. `push_variant` accepted it and printed:

```text
artifact graph-only join accepted foreign reached definition: true
test result: ok. 1 passed
```

The reproducing command was `cargo test -p tiler-artifact definition_generation_perturbation_reaches_artifact_graph_only_join -- --nocapture`; the disposable test was removed after restoration. This is the required negative subject perturbation: the changed definition was the subject, not the assertion. It proves same-key mutation unsafe under the accepted ADR 0072 graph contract.

The successor-key control is the inverse: replacing only the occurrence key must move `tiler.semantic-graph.v3`, and `ArtifactProgramBuilder::push_variant` must return `SemanticSubjectMismatch` when old and successor programs are crossed. The current `rejects_a_variant_realizing_another_semantic_graph` test exercises that graph-only refusal path; implementation must add the exact old-key/successor-key perturbation and pin both graph values.

No checked-in fixed-byte pin exists for either complete outer reference-registry identity at this base. Implementation must add direct semantic-only and scalar-only movement watches in addition to the successor-key pins, so a future encoder that silently drops either nested snapshot fails.

The frontier survivors have distinct negative controls. Strict forever is guarded by the current independent field-8/field-9/request perturbations: any attempt to admit a relaxed result remains a typed refusal. The successor is guarded by changing only old versus new `OpKey` and requiring graph, request, refinement, kernel-program, artifact, and cache subjects to move while scalar subjects stay fixed; crossing the old/new program at the artifact join must fail. For the witness itself, change only one postorder child edge and require either an exact different result for the new legal tree or a structural refusal—never membership against some other tree. For ADR 0013 stability, independently perturb artifact digest, selected variant, declared target environment, and topology after its separate decision supplies those subjects; each must leave the claimed equal-execution class or refuse.

## Identity, schema, request, artifact, and cache cascade

| Subject | Complete successor replacement | Domain/schema consequence |
| --- | --- | --- |
| `OpKey` and graph | move for every contraction occurrence because `strict-tensor-contraction-f32@1` becomes `tensor-contraction-f32@1` | `tiler.semantic-graph.v3` grammar unchanged; content and graph pins move |
| reached semantic definition | moves for every contraction program through the new key and thirteen-field facts | `tiler.semantic-definition-projection.v6` unchanged; content/pins move |
| complete semantic registry | moves because old definition/key row is replaced | `tiler.semantic-registry.v8` unchanged; content/pins move |
| semantic provider | `standard-semantics@8` may stay because provider-independent definition bytes carry the change completely | bump only if implementation behavior is not fully encoded there |
| complete `SemanticIdentity` | the complete value moves because graph, reached definitions, admission provenance, and registry snapshot move; graph now closes graph-only joins, while `ShapeEnvIdentity` stays byte-identical | no layout step; four of five component values move |
| semantic reference registry | moves through semantic snapshot, successor capability key/authority, and provider/capability revision 8 | `tiler.reference-registry.v2` unchanged; add outer fixed-byte/movement pins |
| scalar definition/snapshot/reference | byte-identical for this semantic-only replacement | `tiler.scalar-registry-snapshot.v1`, `tiler.scalar-reference-registry.v1`, `standard-scalar-reference@1`, and scalar capability revision 1 unchanged |
| law registry | complete identity moves through semantic snapshot; strict law rows migrate to the successor key and remain one legal member | `tiler.ir.index-realization-law-registry.v1` unchanged unless row grammar changes |
| lowering registry | complete identity and reached operation authority move; capability row migrates to successor key | `tiler.compiler.lowering-capability-registry.v2` unchanged; changed emitted behavior requires its own revision bump |
| request | the complete request subject and pinned qualifier move through graph, reached definitions, admission provenance, semantic registry snapshot, normalized contraction operation, and the migrated law/lowering registry identities; `ShapeEnvIdentity`, numerical contract and preferences, budgets, and target profile stay byte-identical | `REQUEST_SCHEMA_VERSION = 2` and `tiler.compiler.request-subject.v6` unchanged |
| explain | compilation subject/trace qualifier and operation spelling move | `EXPLAIN_SCHEMA_VERSION = 11` and renderer `tiler-explain-v9` unchanged unless new event fields are serialized |
| fusion/refinement | contraction occurrence proofs, authorities, resolutions, complete receipts, and reached coverage move | current refinement domains unchanged; unrelated reached-only coverage need not move |
| schedule | existing direct/regular topology bytes may remain structurally equal, but their enclosing reached operation/refinement moves | `tiler.schedule.v6` unchanged for a derived uniform witness; serializing a new arbitrary-tree/mapping field requires a schedule-domain step |
| kernel program | moves through graph, `CoveredOccurrence.refinement`, and selected topology/program identity | `tiler.kernel-program.v12` unchanged for existing topology grammar |
| artifact construction | old/new cross-generation `push_variant` now fails at its correct graph-only join; successor artifacts store the new complete semantic identity | `tiler.artifact-program.v18` and `ArtifactSchema::GOVERNED` component versions `1.0/1.0/1.0/3.0` unchanged unless determinism/tree fields are added; the ADR-0013 prerequisite owns those decisions |
| artifact/cache | artifact identity moves through graph, reached definitions, admission, refinements, programs, providers, and realization; expansion/runtime cache misses | cache grammar unchanged; no fallback from successor to retired artifact |

This replacement needs no repair to `ArtifactProgramBuilder` merely to distinguish old and new contraction meaning; the new key makes its intended graph subject move. If a future design wants same-key meaning generations, it must first supersede ADR 0072 and make every graph-only join carry a generation authority. That broader migration is not hidden inside this ticket.

A later algebraic-authority decision that changes a `ScalarOperationDefinition` is a separate migration: scalar definition projection/snapshot, `CanonicalScalarReferenceRegistryIdentity`, law/lowering/refinement/request identities, provider revisions, and pins must be rederived then. This ticket does not pre-pay it.

The descriptor and effective profile use existing `CanonicalValue`, `NumericalRealization`, and subject layouts, so public Rust types alone do not step durable schemas. The uniform witness is derived from already identity-bearing topology. A future stored arbitrary tree, coordinate mapping, target-environment compatibility subject, or new canonical tag must step its owning schedule/kernel/artifact/domain/schema rather than inheriting the “unchanged” entries above.

## Pareto-complete options

| Option | Disposition | Strongest counterargument / reversal evidence |
| --- | --- | --- |
| Keep `strict-tensor-contraction-f32@1` strict forever | **Frontier survivor.** Maximum narrowness, no new public/reference surface, and zero added host work. It permanently closes both fixed memberships. | Strongest objection: it strands an accepted topology carrier and fired useful contiguous-plan demand. It wins if Tom judges that capability below the surface cost, or if exact witness/reference implementation proves materially broader than specified. |
| Same key, booleans changed to `true` | **Eliminated.** It cannot name reducer/result class/seed/witness and misreads definition maximum as caller permission. | Reverse only if another typed owner already supplies every term and every consumer joins it; none exists. |
| Same key, typed reassociation-only descriptor | **Eliminated as unsafe at this base.** Reached definitions and complete snapshots move, but canonical graph bytes do not; the artifact hybrid measurement proves a correct graph-only consumer accepts mixed generations. The old key name also becomes false. | Its advantage is less frontend migration. Reversal requires superseding ADR 0072 and repairing every graph-only construction/verification/cache join first, not asserting those consumers are wrong. |
| Same key after a complete artifact/authority-join migration | **Sound only behind a broad prerequisite, then dominated by the successor.** It changes more cross-layer identity machinery, retains the false `strict` name, and buys no capability over a new key in this pre-production tree. | It would become competitive only if an external immutable graph key made key replacement costlier than a proven complete generation-authority migration. No external consumer exists. |
| Coexisting distinct relaxed key | **Eliminated.** Sound identity separation, but duplicates frontend choice, semantic/reference/law/lowering/conformance verticals and violates ADR 0087's one-family result. | Reopen only if two user-visible operation meanings must remain simultaneously selectable rather than being request cells of one family. |
| Complete replacement by `tensor-contraction-f32@1`, reassociation-only | **Frontier survivor and recommendation.** Exact strict cell plus ordered-tree set under explicit request; canonical graph bytes move; one standard vertical remains; selected-tree reference is linear and bounded. | Strongest objection: key migration and consequential public surface for a capability whose algebraic proof is not yet accepted. Reverse if the fired strategy is withdrawn or the exact downstream witness cannot be implemented under its stated bounds. |
| Permutation-only successor | **Eliminated as dominated by strict today.** The semantic set is well-defined (`K!` strict-left folds), so lack of a commutativity capability does not make it mathematically unsound. But no current authority or consumer can spend permutation, leaving zero reachable capability over strict while adding a larger oracle/public surface. | Reopen when an accepted commutativity authority plus a target/workload/strategy requires changed contributor order. Missing capability is a reopening prerequisite, not a semantic impossibility. |
| Reassociation plus permutation successor | **Eliminated as dominated by reassociation-only today.** It weakens the result class and adds permutation proof/oracle surface without a reachable consumer. | Reopen on the same commutativity and measured-demand evidence as permutation-only. |
| Typed reducer/result-set descriptor | **Required carrier, not an alternative.** Every relaxed survivor needs it; adding it while staying strict only restates current meaning at extra cost. | Its public cost is real, which is the central argument for strict forever. |
| Request/attribute-selected canonical tree or finite topology allowlist | **Eliminated.** It places physical HOW in graph semantics/frontends, forces a choice before planning, and makes adding a schedule change tensor meaning. | It offers a point-valued oracle, but ADR 0012 deliberately separates semantic result class from selected physical tree. |
| More bounded numerical/reference research | **Stop condition met.** All F32 inputs and nine subnormal cells are defined; tree sets are finite for each `K`; production evaluates one witnessed tree in linear work. | Reopen for a concrete `P`/`A` counterexample, unrepresentable current topology, failed authority binding, or omitted consumer—not for more examples of already defined arithmetic. |
| Defer | **Process fallback, not a semantic answer.** Executable behavior remains strict and typed refusals stay correct, but the fired decision and downstream tickets remain blocked. | Appropriate only if independent review finds a named gap; unchanged external state is not itself new evidence. |

Among relaxed choices, the complete successor strictly dominates same-key repair and coexistence on correctness discipline and long-term maintainability, with no worse kernel behavior. Strict forever and the successor remain nondominated because the former minimizes host/reference/public surface while the latter supplies the demanded capability. No survivor silently defaults authority or returns a wrong oracle value.

## Downstream graph, host cost, and unsupported scope

If Tom selects strict forever, close or supersede `decide-the-algebraic-capability-authority-for-contraction-splits` and `admit-reassociated-contraction-schedule-alternatives` with the operation-owned refusal. The accepted physical carrier remains reserved vocabulary with no admissible contraction split.

If Tom selects the successor:

1. accept the exact public surface or revise it before implementation;
2. accept [`decide-the-adr-0013-plan-determinism-stability-subject`](decide-the-adr-0013-plan-determinism-stability-subject.md), then complete [`implement-the-adr-0013-plan-determinism-stability-subject`](implement-the-adr-0013-plan-determinism-stability-subject.md), before any relaxed variant claims plan determinism; the admission ticket already depends on that carrier;
3. reopen the algebraic-authority ticket at the accepted semantic commit to choose the exact ordered-associativity subject for the internal F32 reducer;
4. create one implementation carrier that atomically replaces the standard key across semantic/frontend/reference/law/lowering/compiler/test/pin paths and adds the descriptor/effective-profile/witness/evaluator boundary, and record the accepted contract as a decision record whose application sweeps the documents that name the retired key — ADR 0087's traceability correction, the support-matrix contraction row, and every catalog or contract sentence spelling `strict-tensor-contraction-f32@1` as the standard key — because the accepted-ADR application rule requires the sweep and nothing else in this list names it;
5. revise `admit-reassociated-contraction-schedule-alternatives` so contiguous membership is the only reachable delivery under this contract; and
6. leave lane-strided admission behind the permutation/commutativity trigger above.

No existing ticket status or Tom queue is changed by this packet. The determinism decision records a discovered public/identity/schema prerequisite and neither answers that question nor authorizes implementation. Its new `todo` implementation carrier invents no surface, remains blocked by the decision, and is now a hard dependency of relaxed schedule admission.

Present executable behavior, target runtime, and device memory remain the strict direct contraction. A future selected-tree conformance evaluation remains `O(output_count * K)` arithmetic, `O(K)` witness memory, and existing output storage, under explicit caller budgets. It never materializes Catalan or factorial result sets. No kernel-performance or target-feasibility claim follows from the semantic choice.

Excluded from the recommended contract: permutation, commutativity authority, scalar/semantic capability API, FMA, distributivity, signed-zero elimination, exceptional-value absence assumptions, timing-dependent/atomic arrival, padding, ragged membership, coordinate-dependent trees, runtime-selected split width, widened accumulator, seeded/empty contraction, non-F32 arithmetic, contraction-chain rewrites, historical-key fallback, or production implementation. Each remains typed unsupported rather than defaulted.

## Packet verification — 2026-08-17

The clean source tree plus this ticket-only superseding packet passed:

```sh
cargo test -p tiler-reference a_declaration_this_reference_does_not_compute_is_refused_by_field -- --nocapture
cargo test -p tiler-reference a_contract_admitting_a_result_set_is_refused_by_name -- --nocapture
cargo test -p tiler-reference every_new_dimension_is_accounted_for_by_the_reference_boundary -- --nocapture
cargo test -p tiler-ir the_contraction_declares_no_algebraic_capability -- --nocapture
cargo test -p tiler-compiler deterministic_trace_is_sealed_and_rendered_separately -- --nocapture
cargo test -p tiler-artifact rejects_a_variant_realizing_another_semantic_graph -- --nocapture
tkt lint --format json
make citations
git diff --check
tkt guard tkt/decide-the-semantic-order-contract-for-relaxed-contractions --ticket decide-the-semantic-order-contract-for-relaxed-contractions --base 368dcd255079162c116716ca807cb0f23dfa3a59 --config-ref 368dcd255079162c116716ca807cb0f23dfa3a59 --format json
```

All six targeted tests reported exactly one selected test passed and zero failed. Ticket lint reported no diagnostics; `make citations` resolved every pinned citation and local link; `git diff --check` was empty. The exact-base guard is run before and after commit so the committed packet, including the determinism decision, implementation carrier, and admission dependency repair, is visible.

## Independent review — 2026-08-18

Adversarial review by `worker-semantic-order-review` at exact base `075d2d447b89d8f9b96fe6baa90157334a4359f6`, on `tkt/decide-the-semantic-order-contract-for-relaxed-contractions`, from a clean tree. Between the packet's audited base `368dcd25` and this one, the only production-source movement is `crates/tiler-runtime/src/load/host.rs`, which no Fact cites; the ticket graph gained the two ADR-0013 tickets and the admission dependency edge the packet describes. Every named source below was read in full at this base; every anchor verdict is a `grep -c` run against the file the citation names.

### Per-Fact verdicts

1. **Verified, one anchor repaired.** The normative-reference anchor and both `boolean(false)` facts hold, and the fact record has exactly fourteen fields. The quoted registration-comment anchor beginning `Declaring the contraction itself` matched nothing in `crates/` at this base or at `368dcd25` — `git log -S` finds that string only in this packet's own commits, so it was composed for the audit rather than copied from source. The underlying claim is true of the real comment, whose fragments now cited were each grep-verified at count 1. Severity: moderate for process (an anchor that fails as absence invites a false "restoration"), nil for the claim.
2. **Verified.** `ContractionContract::decode` requires all three permissions false at `The three permissions`; `a_declaration_this_reference_does_not_compute_is_refused_by_field` is a table over all fourteen fields, each refused under its own `AttributeFieldId`; `from_realization` returns `ReassociationPermitted`/`PermutationPermitted` before any evaluator runs.
3. **Verified.** Both ADR anchors hit at count 1; neither record changes the two false facts or installs a result-set oracle.
4. **Verified.** `TENSOR_CONTRACTION` lists both order dimensions beside contraction; the `would answer false` comment states exactly the silently-wrong-accept consequence the Fact claims, and the contraction's fusion role is `PrologueCarryingOrderedReduction`, not `ElementwiseArithmetic`.
5. **Verified.** The `Decision result` heading, the wrong-operands argument, and the scalar-add non-override argument are all present; the algebraic ticket's frontmatter depends on this ticket and is `blocked`.
6. **Verified, ADR 0072 anchor repaired (line-wrap only).** Independently re-derived from source rather than from the packet's measurement: `ArtifactProgramBuilder::new` clones the full `SemanticIdentity`; `push_variant` compares only `program.semantic_graph_identity() != self.semantic.graph()`; `check_subject` compares interface, numerical realization, and target profile, none of which carries reached definitions; and `SemanticGraphIdentity` encodes domain, inputs, operation keys, attributes, operands, and types/shapes with definitions deliberately excluded. A same-key definition-only change therefore leaves graph bytes byte-identical and the join accepts the cross-generation program — the hybrid follows deductively without rerunning the disposable test. `rejects_a_variant_realizing_another_semantic_graph` was read and does exercise the graph-only `SemanticSubjectMismatch` refusal.
7. **Verified, anchor repaired.** `ContributorPartition` is exactly `{partitions, contributors_per_partition}`; a regular contiguous split cannot spell an arbitrary full ordered binary tree.
8. **Verified.** ADR 0012's `Unrealized` stability-scope paragraph and ADR 0013's open target-environment-identity sentence both hold; ADR 0013 is `implementation_status: not-started`.
9. **Verified, anchor repaired.** The evaluator's three constructors (`new`, `under`, `standard`) all carry exactly one conformance; a relaxed realization cannot reach one through `from_realization`, so no ordinary no-witness relaxed route exists.
10. **Verified.** `LiveContraction` carries `live_access`/`live_axis` and no shape; anchor at count 1.
11. **Verified.** `SemanticIdentity` has exactly the five named subjects; `shape_environment` is documented and built from bindings, root-binding provenance, and constraints, so an operation-key replacement cannot move it.

### Derivations reproduced

- **Fixture arithmetic.** The embedded python one-liner was rerun; all six printed values and bit patterns match the packet exactly, and each was re-derived by hand (`2^24 + 1` rounds to `2^24` under RNE; `-2^24 + 1` is exactly representable), so membership, grouping, and order really are three separately observable freedoms.
- **Result-population formulas.** `P` and `A` were checked term-by-term against `evaluate_outputs`: `I` on both operands of every multiply and add (the accumulator re-enters as an operand), `N` then `R` on every product and sum, boundary canonicalization idempotent over `A`'s output, first-product seed, `K = 0` refused. Tree-count bounds (full binary trees with `K` leaves = Catalan `C_(K-1)`; `K!`; product), witness node count `2K-1`, visits `output_count * (2K-1)`, and steps `output_count * (K + K-1)` are all arithmetically consistent. The postorder validation rules were checked as a set: in-degree-one non-roots plus children-strictly-earlier plus per-node interval adjacency plus root coverage do jointly exclude cycles, sharing, gaps, overlap, reversal, and permutation.
- **Identity cascade.** Spot-verified at the encoders: `tiler.semantic-graph.v3` encodes each operation's key, so a key replacement moves graph bytes; `tiler.reference-registry.v2` embeds the semantic snapshot at its head plus per-row operation key, signature, authority, and provider/capability revisions, so both the same-key and successor movements follow structurally; `REQUEST_SCHEMA_VERSION = 2`, `tiler.compiler.request-subject.v6`, `tiler.artifact-program.v18`, `ArtifactSchema::GOVERNED` `1.0/1.0/1.0/3.0`, `EXPLAIN_SCHEMA_VERSION = 11`, renderer `tiler-explain-v9`, `standard-semantics@8`, `standard-reference@7` with contraction capability revision 7, `standard-scalar-reference@1`, `tiler.scalar-reference-registry.v1`, and the law/lowering/schedule/kernel/scalar-snapshot domains all match the table at this base.
- **Key spelling.** `OpKey` is `(namespace, name, semantic_version)` with the version scoped to the name, confirming the `@2`-invents-an-absent-generation argument; the `strict-` rejection is the symmetric truthfulness argument, and an adjective-bearing alternative (`relaxed-`, `ordered-`) fails the same test in the other direction, so the neutral name is the unique truthful fixed point rather than one of several cosmetic picks.
- **Declaration perturbations, rerun as subject perturbations.** Field 8 flipped to `true` alone: `registry_identity_is_deterministic_and_revision_complete` fails at construction with `UnsupportedContraction { operation: ...strict-tensor-contraction-f32... , source: UnrealizableFact { field: AttributeFieldId(8) } }`. Field 9 alone: the same failure with `AttributeFieldId(9)`. Tree restored byte-identical after each (`git status` clean).
- **Ticket graph.** `implement-the-adr-0013-plan-determinism-stability-subject` depends on `decide-the-adr-0013-plan-determinism-stability-subject` (now `awaiting-decision`, queued as decision-queue item 15 behind the reviewed queue); `admit-reassociated-contraction-schedule-alternatives` lists both `decide-the-algebraic-capability-authority-for-contraction-splits` and the implement carrier as hard dependencies. The determinism claim is graph-enforced exactly as stated, and this ticket appears nowhere in `.ticketsplease/decision-queue.md`, consistent with "deliberately unqueued".

### Commands rerun — 2026-08-18, base `075d2d44`

All six targeted tests pass, each `1 test run: 1 passed`, via `cargo nextest run -p <crate> -E 'test(<name>)'` — this host's harness now requires nextest in place of the packet's `cargo test` spellings; the selected populations are identical. `tkt lint --format json` reports `ok: true` with no diagnostics; `make citations` resolves 1166 pinned citations and 6579 local links with no failure; `git diff --check` is empty; `tkt guard tkt/decide-the-semantic-order-contract-for-relaxed-contractions --ticket decide-the-semantic-order-contract-for-relaxed-contractions --base 075d2d447b89d8f9b96fe6baa90157334a4359f6 --config-ref 075d2d447b89d8f9b96fe6baa90157334a4359f6 --format json` reports severity `ok` with no collisions and no under-declared scopes.

### Discrepancies

1. **Fabricated-or-drifted anchor on audit item 1** (moderate; repaired in place above). The registration-comment quotation never existed in `crates/` at either base. The claim survives under the real fragments.
2. **Three rendered-view anchors that fail as bytes** (minor; repaired): ADR 0072's sentence-initial line wrap, `ContributorPartition`'s inline-code-plus-wrap doc line, and the evaluator's doc-link markup. Each is exactly the AGENTS.md failure mode; each repaired anchor was grep-verified at count 1 before landing.
3. **Implementation-carrier documentation sweep was implicit** (minor; repaired). The successor branch's carrier list named code, tests, and pins but not the decision record and the document population spelling the retired key (ADR 0087 traceability, support matrix, catalogs); downstream item 4 now names it.

### Not reverified

The same-key/scalar-only movement fingerprints (`8713bca5...`→`ade9f0ef...`, `cafea636...`→`3c090fda...`, and the explain-qualifier movements) are session measurements from the author's disposable harness; only the `17e0dd47e48b7c18` baseline is pinned in-tree (`explain.rs` golden). Their structural conclusion — outer identities move when a nested snapshot moves while domain tags and revisions stay — is independently verified from the `tiler.reference-registry.v2` encoder, so nothing decision-bearing rests on the unpinned hex values; the packet itself already requires implementation to add movement watches. The disposable artifact-hybrid test was likewise not rerun; its conclusion is re-derived from the builder and identity sources as recorded under Fact 6.

### Frontier attack result

No materially distinct option is missing: every candidate the ticket's own frontier requires is dispositioned, an adjective-bearing successor spelling is excluded by the same truthfulness argument that eliminates `strict-...@2`, and a dtype-generic key would contradict the corpus-wide per-dtype key convention without any consumer asking for it. Every elimination reason was re-checked at this base: the same-key elimination rests on the verified graph-only join; the permutation-only elimination rests on ADR 0014's two `Unrealized` paragraphs (no commutativity capability exists to declare, and nothing consumes a permutation permission), both re-read and current. The recommendation is genuinely nondominated rather than manufactured — strict-forever minimizes surface, the successor alone delivers the fired capability, and the packet asks Tom exactly that one question. No correctness-bearing API, proof, or identity choice was found left implicit for the implementer beyond the documentation sweep now repaired.

**Verdict: ready for Tom with the named repairs, which are made in this commit.** The packet's Facts, eliminations, arithmetic, identity cascade, and dependency wiring all survive independent re-derivation at `075d2d44`; the discrepancies found are citation-quality and carrier-completeness issues, none of which moves the option set, the recommendation, or the draft question.


## Accepted decision — 2026-08-18

Tom accepted **the complete standard-vertical replacement: retire `tiler::strict-tensor-contraction-f32@1` and replace it with `tiler::tensor-contraction-f32@1`, reassociation-only**, in the live coordination session with the orchestrator, relayed first-hand by the coordinator, by replying `agreed, next decision` to the packet's exact draft question presented in explain-then-recommend form after independent review (`5a48c9ce`, merged `abfb0948`).

The accepted semantic contract is exactly the packet's: the successor's strict request cell preserves the old strict left-fold answer bit for bit; its reassociation-permitted cell denotes the set of all full ordered binary trees over the unchanged canonical leaves; permutation remains operation-owned unsupported; the old key leaves the standard vertical completely, with no alias, equivalence rule, fallback, or duplicate selection policy. Same-key mutation remains eliminated on the measured artifact-hybrid evidence.

Per the packet's downstream list: (1) the exact drafted public surface (descriptor, effective profile, witness, topology evaluator) remains a **labelled draft** whose exact included/excluded acceptance is its own next question — carried by `accept-the-tensor-contraction-successor-public-surface`; (2) the ADR-0013 decision was accepted the same session and its implementation carrier is queued; (3) `decide-the-algebraic-capability-authority-for-contraction-splits` reopens at this accepted commit to choose the internal reducer capability; (4) one implementation carrier owns the atomic key replacement and boundary delivery, blocked on the surface acceptance and the determinism carrier; (5) `admit-reassociated-contraction-schedule-alternatives` is revised so contiguous membership is the only reachable delivery; (6) lane-strided admission stays behind the permutation/commutativity trigger.

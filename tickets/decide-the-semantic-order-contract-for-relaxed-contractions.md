---
id: decide-the-semantic-order-contract-for-relaxed-contractions
title: Decide the semantic order contract for relaxed contractions
status: in-progress
priority: p1
dependencies: []
related: [decide-the-algebraic-capability-authority-for-contraction-splits, admit-reassociated-contraction-schedule-alternatives]
scopes: [implementation/ir, implementation/compiler, implementation/reference, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, numerics, identity, public-boundary, needs-tom]
claimed_from: todo
assignee: worker-contraction-semantics
lease_expires_at: 1787011562
---
## User-visible outcome

Tiler decides whether `tiler::strict-tensor-contraction-f32@1` remains a single strict-fold value under every numerical contract or gains an explicitly permission-indexed result set for reassociation and permutation. The answer must settle the semantic facts, reference oracle, operation identity, and unsupported population before any algebraic capability or physical split can be admitted.

This is a prerequisite decision, not implementation authorization. Do not queue it for Tom until its own exact-base Fact audit and Pareto-complete packet have been independently reviewed.

## Source-first filing evidence — exact base `fe60f992cc20b37a52aff815897170516490667a`

**Fact — current registered meaning is strict and singular.** `register_standard_contraction` in `crates/tiler-ir/src/semantic/contraction.rs` names `binary32 products folded strictly in ascending lexicographic order`; `contraction_f32_facts` sets `CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED` and `CONTRACTION_F32_FACT_PERMUTATION_PERMITTED` to `false`; and its registration comment says declaring ordered associativity would hand a rewrite facts the family forbids.

**Fact — the independent reference treats either freedom as a different semantic population.** `crates/tiler-reference/src/contraction.rs`, anchor `The three permissions`, requires the contraction definition's arithmetic-contraction, reassociation, and permutation facts all to be `false`. `a_declaration_this_reference_does_not_compute_is_refused_by_field` perturbs each to `true` and requires the field-specific refusal. `ReferenceNumericalConformance::from_realization` in `crates/tiler-reference/src/conformance.rs` separately refuses permitted reassociation and permutation rather than evaluating the strict value and mislabelling it as the requested result set.

**Fact — accepted ADR prose leaves a future conditional route but does not instantiate it.** [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md), anchor `unless a registered permission authorizes otherwise`, accepts one keyed contraction family with a strict lexicographic fold unless a registered permission authorizes another order. [ADR 0014](../docs/decisions/0014-reassociation-vs-permutation.md), anchor `Each transformation requires two independent facts`, requires algebraic capability and numerical permission independently. Neither accepted record changes the current definition's two `false` values or installs a result-set oracle merely by describing the future seam.

**Fact — the compiler says this family can consume the dimensions but does not grant them.** `const TENSOR_CONTRACTION` in `crates/tiler-compiler/src/policy.rs` includes reassociation and permutation so a target/request is asked about freedoms capable of changing the fold. That table is numerical applicability, not semantic or algebraic authority. The distinction is exposed by the source anchor `would answer false` in `crates/tiler-compiler/src/fusion_legality.rs`: classifying the contraction as permission-free under a reassociating contract would silently accept the regrouping its own facts forbid.

**Fact — a capability-only repair is insufficient.** [`decide-the-algebraic-capability-authority-for-contraction-splits`](decide-the-algebraic-capability-authority-for-contraction-splits.md), anchor `Decision result — no current authorizing capability`, proves that an operation-wide capability names the wrong operands and a capability on the realized scalar add cannot override the semantic operation the realization refines. Both fixed split memberships therefore remain unavailable until this ticket decides whether a relaxed semantic population exists at all.

## Exact decision this ticket owns

Decide one coherent semantic contract across these inseparable subjects:

- whether the existing key stays point-valued under every request or denotes a strict value under forbidden permissions and a typed result set under permitted reassociation/permutation;
- whether the two current boolean fact fields remain booleans, change value, or are replaced by a typed order-contract declaration, including their exact validation and canonical encoding;
- how an effective numerical permission intersects with that operation-owned declaration without a request overriding a restriction or a definition granting a caller freedom it did not request;
- which reference question is answered for a permitted contract: membership in an exact finite/result-set oracle, a topology-parameterized evaluator, another sound witness, or typed refusal—and the boundedness/unsupported cases of that answer;
- whether changing the current definition under the same `OpKey` is compatible with ADR 0087 and the provider-independent definition-projection identity, or whether correctness requires superseding the one-key decision with a new operation identity;
- canonical-NaN, signed-zero, subnormal, infinity, FMA/contraction, accumulator, empty-domain, seed, determinism, and distributivity consequences; and
- the complete semantic-registry, scalar/law/lowering registry, request, explain, refinement, program, artifact, cache, schema, and pin cascade, including `CanonicalReferenceRegistryIdentity` over the semantic snapshot and `CanonicalScalarReferenceRegistryIdentity` over the scalar snapshot.

This ticket must leave the later algebraic-authority decision with a fixed semantic subject. It must not choose the scalar/semantic combiner API itself.

## Required Pareto frontier

At minimum compare, and eliminate only with source-backed reasons:

1. keep `tiler::strict-tensor-contraction-f32@1` strict forever and close both split strategies;
2. keep the one key but make its order contract permission-indexed, preserving the exact strict answer when both freedoms are forbidden;
3. keep the one key and admit reassociation only, leaving permutation permanently unavailable, if that narrower result set has a strictly smaller sound oracle/identity surface;
4. introduce a distinct relaxed key, explicitly accounting for the accepted one-key rationale and the duplicate frontend/reference/lowering vertical;
5. replace the current boolean facts with a typed internal reducer/result-set descriptor if booleans cannot express the needed authority without contradiction;
6. perform bounded numerical/reference research with exact stop conditions; and
7. defer with the current typed refusal and an evidence-based reopening trigger.

Eliminate any option that lets request permission overwrite an operation restriction, infers an internal combiner from prose or definition facts, returns the strict oracle value for a result-set request, conflates reassociation with permutation, changes a semantic definition without moving its provider-independent identity, or claims a complete result while leaving reference/conformance support implicit.

## Required independent derivations and perturbations

- Derive the legal result population from the definition, not from the measured split kernels. Cover all F32 bit patterns or state the exact bounded corpus; canonical NaN payload/order, both signed zeros, infinities, subnormal preserve and each FTZ zero-sign mode, separate multiply/add rounding, and every excluded FMA/distributive rewrite must be explicit.
- Derive operation key/definition/snapshot and downstream identity movement independently from the reference design. A same-key semantic-definition change moves the semantic snapshot embedded in `CanonicalReferenceRegistryIdentity`; a same-key scalar-definition change moves the scalar snapshot embedded in `CanonicalScalarReferenceRegistryIdentity`. Those outer values and pins move even if their domain tags and reference-provider revisions do not. A matrix entry that says “domain unchanged” must still say whether values and pins move.
- Perturb only the same-key semantic definition and show `CanonicalReferenceRegistryIdentity` plus its pins move while its domain tag and reference-provider revisions stay fixed. Perturb only the same-key scalar definition and show `CanonicalScalarReferenceRegistryIdentity` plus its pins move under the same controls.
- Perturb each of the two current `false` facts independently and show which semantic/reference check fails. Perturb the effective permission independently to prove operation declaration and request ceiling cannot substitute for each other.
- If a result-set oracle is proposed, perturb contributor membership while holding partition counts and merge order fixed, and show the contiguous/lane-strided distinction is observable. Perturb grouping while preserving leaf order separately from permutation.
- If the existing key is retained, demonstrate why an old strict occurrence and a new permission-indexed occurrence cannot collide in semantic/request/artifact identity. If a new key is proposed, demonstrate why that duplication dominates superseding neither ADR nor existing consumers.

## Consequences and non-goals

This ticket does not implement the accepted physical carrier or either split, add algebraic capability vocabulary, grant a caller numerical permission, infer authority from the current scalar lowering, mutate the preserved failed attempt, admit distributivity/FMA/nondeterminism, or make a kernel-performance claim.

The present executable outcome remains the strict direct contraction only. Host runtime and memory are unchanged until Tom accepts a different semantic population and separate implementation work lands.

## Downstream graph and release conditions

[`decide-the-algebraic-capability-authority-for-contraction-splits`](decide-the-algebraic-capability-authority-for-contraction-splits.md) depends directly on this ticket; [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md) depends on the algebraic-authority ticket and therefore inherits this prerequisite transitively. If the accepted answer keeps the key strict, close or supersede both split outcomes with the exact refusal. If it admits a relaxed population, reopen the algebraic-authority ticket at the accepted commit; only that reopened ticket may choose the exact operation/combiner capability and verifier join.

## Closes when

Tom has accepted one exact semantic order/result-set contract, with the strongest counterargument and reversal evidence for every frontier survivor, and the complete reference, identity, schema, unsupported-population, and downstream graph consequences are explicit. Only then can an algebraic capability question be meaningful rather than contradictory.

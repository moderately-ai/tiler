---
id: share-identical-constants-in-the-pointwise-expression-canonical-form
title: Share identical constants in the pointwise expression canonical form
status: awaiting-decision
priority: p2
dependencies: [determine-whether-compiler-lowering-mints-duplicate-pointwise-constants]
related: [implement-the-realization-witness-vocabulary, enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle]
scopes: [implementation/ir, research/reference, implementation/compiler, implementation/artifact, implementation/build, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, identity, conformance]
---
## What was found, and where

**Measurement — [`implement-the-realization-witness-vocabulary`](implement-the-realization-witness-vocabulary.md) at base `61414b91`.** [The freedom-sites record's](../docs/research/reference/plan-freedom-sites.md) Part 5 claims that `PointwiseF32ExpressionBuilder`'s canonicalization makes "the canonical form a function of the program rather than of the spelling", on two named mitigations: one leaf per input ordinal shared on repeat request, and a deterministic root-first topological order. That ticket tested the claim, as it was required to before building a witness on it.

It **holds** for exactly those two mitigations, and **fails** for a third spelling neither reaches. Nothing shares an identical constant: `PointwiseF32ExpressionBuilder::constant` pushes a draft node unconditionally, and `canonicalize_nodes` maps draft ordinals to canonical ones with no hash-consing. So `x * 2.0 + 2.0` spelled with one constant value and spelled with two `constant()` calls produces a four-node and a five-node expression, two different `RealizationWitness::pointwise_f32` values, and two different `CanonicalScheduledRegionIdentity` values for one binary32 function. `a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse` in `crates/tiler-ir/src/schedule/witness/tests.rs` is the standing evidence, and it asserts the *current* behaviour so this ticket's fix must flip it if sharing is accepted.

`PointwiseBf16ExpressionBuilder::constant` and its `canonicalize_nodes` have the same append-and-retain structure. Compiler reachability at that width is intentionally left to the prerequisite rather than inferred from builder symmetry.

## Why it is filed rather than fixed

Two reasons, and the second is the one that needs a decision.

**It is not unsound.** The witness is too *fine* here rather than too coarse: a caller comparing two witnesses may conclude they differ when the bits do not, never that they agree when the bits differ. What fails is the converse of the enumeration's determination property, so a conformance oracle built on the witness stays fail-closed. `RealizationWitness` derives no `PartialEq` for exactly this reason.

**Sharing constants moves canonical schedule identity.** Every region whose scalar program mints a repeated constant would encode different bytes, which reaches the schedule identity domain and everything downstream of it — request subjects, artifact identity, cache subjects, and the checked-in Metal goldens. Whether that is a `tiler.schedule.v6` step or an unversioned change to a population that is provably empty in the retained corpus is the question this ticket has to answer, and it is an identity-domain decision the implementing ticket had no evidence for.

## Correction — 2026-08-09

**Fact — this ticket combined research with an identity decision.** `mint_into` reuses the value already associated with one semantic `ValueId`, while two distinct planned constant steps call the sink twice. Whether semantic construction and elementwise planning can preserve two equal-payload occurrences is not established. The prerequisite [`determine-whether-compiler-lowering-mints-duplicate-pointwise-constants`](determine-whether-compiler-lowering-mints-duplicate-pointwise-constants.md) now owns that bounded source-and-compiled-pair question.

**Decision boundary.** Even if the compiler population is empty today, changing the builder's canonical form changes the schedule spelling admitted through a public builder. If the compiler population is nonempty it also moves downstream identities. Choosing whether to share constants, and whether any affected schedule identity domain must step or may hold, is consequential identity authority retained by Tom. This ticket therefore remains `awaiting-decision`; the prerequisite supplies evidence but does not authorize implementation.

## What this ticket must produce

1. Read the prerequisite's result and decide whether builder-level equal-constant sharing is the canonical schedule contract.
2. Derive `step` or `hold` for every affected identity domain from the chosen byte-evolution rule; do not assume either answer from reachability alone.
3. If sharing is accepted and a live compiler population exists, enumerate the exact moved pins — schedule identities, request subjects, artifact identities, cache subjects, and Metal goldens — before editing and recompute them on the merged tree.
4. Implement equal-bit constant sharing at both widths, exactly as input-leaf sharing already does, only after the decision authorizes it.
5. Flip `a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse` and reconcile the freedom-sites record with the accepted answer.

## Closes when

Tom has answered the canonical-form and identity-evolution questions using the prerequisite's reachability evidence, and the accepted answer is implemented with every moved pin and dependent record reconciled — or the ticket records the accepted decision to retain distinct constant occurrences.

## Scope repair — 2026-08-09

The accepted-sharing branch explicitly requires reconciling the reference freedom-sites record and any moved compiler request, artifact, build/cache, and Metal-golden pins. `research/reference`, `implementation/compiler`, `implementation/artifact`, `implementation/build`, and `implementation/metal` are therefore declared now rather than left for an implementation worker to discover after the decision.

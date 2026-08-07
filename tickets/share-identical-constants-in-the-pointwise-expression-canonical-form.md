---
id: share-identical-constants-in-the-pointwise-expression-canonical-form
title: Share identical constants in the pointwise expression canonical form
status: todo
priority: p2
dependencies: []
related: [implement-the-realization-witness-vocabulary, enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, identity, conformance]
---
## What was found, and where

**Measurement — [`implement-the-realization-witness-vocabulary`](implement-the-realization-witness-vocabulary.md) at base `61414b91`.** [The freedom-sites record's](../docs/research/reference/plan-freedom-sites.md) Part 5 claims that `PointwiseF32ExpressionBuilder`'s canonicalization makes "the canonical form a function of the program rather than of the spelling", on two named mitigations: one leaf per input ordinal shared on repeat request, and a deterministic root-first topological order. That ticket tested the claim, as it was required to before building a witness on it.

It **holds** for exactly those two mitigations, and **fails** for a third spelling neither reaches. Nothing shares an identical constant: `PointwiseF32ExpressionBuilder::constant` (`crates/tiler-ir/src/schedule/pointwise.rs:322-327`) pushes a draft node unconditionally, and `canonicalize_nodes` (`:551`) maps draft ordinals to canonical ones with no hash-consing. So `x * 2.0 + 2.0` spelled with one constant value and spelled with two `constant()` calls produces a four-node and a five-node expression, two different `RealizationWitness::pointwise_f32` values, and two different `CanonicalScheduledRegionIdentity` values for one binary32 function. `a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse` in `crates/tiler-ir/src/schedule/witness/tests.rs` is the standing evidence, and it asserts the *current* behaviour so this ticket's fix must flip it.

`PointwiseBf16ExpressionBuilder` has the identical structure (`crates/tiler-ir/src/schedule/pointwise_bf16.rs:326`) and is presumed to carry the same gap; it was not separately exercised.

## Why it is filed rather than fixed

Two reasons, and the second is the one that needs a decision.

**It is not unsound.** The witness is too *fine* here rather than too coarse: a caller comparing two witnesses may conclude they differ when the bits do not, never that they agree when the bits differ. What fails is the converse of the enumeration's determination property, so a conformance oracle built on the witness stays fail-closed. `RealizationWitness` derives no `PartialEq` for exactly this reason.

**Sharing constants moves canonical schedule identity.** Every region whose scalar program mints a repeated constant would encode different bytes, which reaches the schedule identity domain and everything downstream of it — request subjects, artifact identity, cache subjects, and the checked-in Metal goldens. Whether that is a `tiler.schedule.v6` step or an unversioned change to a population that is provably empty in the retained corpus is the question this ticket has to answer, and it is an identity-domain decision the implementing ticket had no evidence for.

## What is not established

**Whether the compiler can produce the duplicated spelling.** `recognize_elementwise` (`crates/tiler-compiler/src/request.rs`) states that "two operands naming one value share the node already minted", so one `tiler.constant-f32` occurrence read twice mints one node. Two *distinct* occurrences carrying the same payload are two `ValueId`s and would mint two nodes — but no compiled counterexample was produced, and the semantic layer may or may not already share identical constant occurrences. That reading is what this ticket must settle first: if the compiler cannot mint the spelling, the retained population is empty and the repair is cheap.

## What this ticket must produce

1. A compiled pair, or a proof that none exists: build two semantic programs computing one binary32 function that differ only in whether an identical constant is one occurrence or two, and compare their minted `ScalarProgram::PointwiseF32` payloads. If none exists, say so with the rule that forbids it and close on that.
2. If one exists, enumerate the pinned identity population it moves — schedule identities, request subjects, artifact identity, cache subject, Metal goldens — before editing, and recompute every moved pin on the merged tree.
3. The fix, if it is one: share a draft constant node on repeat request, at both widths, exactly as the input-leaf sharing already does.
4. Flip `a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse` and the corresponding paragraph of the record's Part 7.5 and Part 5.

## Closes when

The compiler-reachability question is answered from source or from a compiled pair; the identity population is enumerated; and either the canonical form shares identical constants at both widths with every moved pin recomputed, or the ticket records why the spelling is unreachable and the gap is a builder-only one.

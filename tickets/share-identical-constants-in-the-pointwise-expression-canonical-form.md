---
id: share-identical-constants-in-the-pointwise-expression-canonical-form
title: Share identical constants in the pointwise expression canonical form
status: done
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

## Decision — accepted 2026-08-12

Tom accepted **retaining distinct equal-bit constant occurrences** in the physical pointwise-expression DAG. `PointwiseF32ExpressionBuilder::constant` and `PointwiseBf16ExpressionBuilder::constant` continue to add one physical definition per call. Reusing the returned handle remains the spelling for one shared definition. No builder-level constant hash-consing, numeric-equality folding, identity-domain step, downstream rebaseline, or implementation ticket follows from this decision.

### Source-first Fact audit at `4845c6b4`

- **Verified — the builders and canonicalizers preserve the authored DAG.** `pub fn constant` in `crates/tiler-ir/src/schedule/pointwise.rs` and `crates/tiler-ir/src/schedule/pointwise_bf16.rs` appends one draft node per call. `fn canonicalize_nodes` gives every reachable draft node a deterministic root-first-derived ordinal and does not perform value numbering. The retained expression documentation says it preserves operand order and DAG sharing; it does not say it quotients distinct physical definitions by value.
- **Verified — the distinction reaches physical output rather than presentation alone.** `push_scalar_program` in `crates/tiler-ir/src/schedule/model.rs` encodes the complete node run. `emit_pointwise` and `emit_pointwise_bf16` in `crates/tiler-ir/src/kernel/lower.rs` emit one `KernelBuilder::constant` operation per retained constant node. `emit_constant` in `crates/tiler-metal/src/emit.rs` emits each resulting KIR constant as its own typed statement. Shared and duplicated constant occurrences therefore are distinct physical DAGs, KIRs, and backend source even when they compute the same binary function.
- **Verified — the existing witness evidence is fail-closed.** `a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse` still proves four versus five retained nodes, different realization witnesses, and different scheduled-region identities. The witness remains intentionally non-`PartialEq`; the distinction cannot make two bit-different realizations compare equal.
- **Verified — semantic equality is already owned by semantic normalization.** The prerequisite's `normalization_converges_duplicated_and_shared_constants_on_one_portfolio` proves that ordinary compilation's pure-operation common-subexpression rule merges equal semantic constant invocations before physical planning. The direct-recognition regression separately proves that the unnormalized physical mint preserves distinct semantic occurrences. These are different authorities rather than two incomplete implementations of one rule.
- **False — the affected compiler population is provably empty.** The prerequisite already names rewrite-budget exhaustion as a path that retains the original graph. Independently, `root_mean_square_scale_plan` in `crates/tiler-compiler/src/physical.rs` synthesizes a folded-extent constant and an `eps` constant after semantic normalization. The semantic RMS-normalization contract admits any finite positive nonzero `eps`, so a one-contributor fold with `eps == 1.0` can give those two physical definitions equal bits. No retained-corpus census can turn builder behavior into a no-op authority.
- **Imprecise — one binary function is the schedule-identity equivalence class.** The scheduled expression is a physical execution graph. Equal compound operations can likewise be represented once and shared or represented twice and recomputed; constants are not a principled exception to that occurrence axis. Deterministic canonicalization removes insertion-order spelling differences within one authored DAG. It does not erase topology or claim to produce a semantic normal form.
- **Imprecise — input-leaf sharing is precedent for constant sharing.** One input leaf per ordinal is a binding invariant: two input nodes naming one ordinal would give one boundary read two canonical definitions. A constant has no corresponding external binding uniqueness rule. The two populations are therefore deliberately different.

### Accepted boundary and ranking

The ownership split is:

1. semantic common-subexpression elimination decides when equal pure semantic operations are one semantic value;
2. a physical producer decides whether its schedule reuses one returned value or defines the same literal more than once;
3. schedule canonicalization deterministically orders that exact authored DAG; and
4. schedule, KIR, artifact, and cache identities retain the resulting physical choice.

This keeps the properties mutually exclusive and collectively exhaustive: semantic value equality, physical occurrence/sharing, canonical numbering, and downstream identity each have one authority. It also preserves constant insertion's O(1) host cost. Ordinary compilation already gets the node/KIR/source savings from semantic CSE; a measured remaining synthesis hotspot may explicitly reuse a physical handle without changing the public builder contract.

Ranked by correctness, maintainability, then performance:

1. **Retain exact constant occurrences and let producers reuse handles deliberately.** This is the accepted contract and the only option that preserves the exact physical DAG without adding work to every builder call.
2. **Apply producer-local reuse at a measured synthesis site.** This can remove a known redundant physical definition while keeping the public occurrence rule explicit, but no measurement currently justifies such a follow-up.
3. **Adopt a complete physical-expression value-numbering contract in a future decision.** If the project later decides schedules identify value-equivalent expression DAGs rather than exact physical computation DAGs, it must cover constants and compound pure nodes coherently and deliberately step the schedule identity authority. It is a different architecture, not this ticket's local cleanup.
4. **Implicitly hash-cons constants alone.** Rejected: it silently removes one class of physical occurrence while leaving identical compound recomputations distinct, so it is neither a complete semantic canonical form nor an exact physical DAG.
5. **Share by floating-point numeric equality.** Rejected: it would additionally collapse exact-bit distinctions such as signed zero and NaN payloads.

The strongest counterpoint is that a duplicate literal consumes node budget and produces redundant KIR/backend source. The ordinary path already removes the semantic case. For the exceptional and physical-synthesis cases, preserving the producer's exact choice is preferable to a hidden builder optimization; evidence of a material hotspot would authorize explicit producer reuse or a separately decided complete value-numbering policy.

### Identity and verification

`tiler.schedule.v5` and every downstream identity domain, artifact/schema version, cache subject, and checked-in golden hold unchanged. A future complete value-numbering design would have to derive its own domain step and moved population; this decision does not grant an unversioned collapse.

The acceptance audit reran:

- `cargo test -p tiler-compiler request::tests::equal_constant_occurrences_remain_distinct_through_initial_recognition -- --exact --nocapture`;
- `cargo test -p tiler-compiler pipeline::tests::normalization_converges_duplicated_and_shared_constants_on_one_portfolio -- --exact --nocapture`; and
- `cargo test -p tiler-ir schedule::witness::tests::a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse -- --exact --nocapture`.

All three passed. This ticket is complete by recording the accepted retain-distinct-occurrences branch; no production edit is owed.

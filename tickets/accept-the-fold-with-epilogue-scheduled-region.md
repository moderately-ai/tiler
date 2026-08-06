---
id: accept-the-fold-with-epilogue-scheduled-region
title: Accept the fold-with-epilogue scheduled region
status: done
priority: p2
dependencies: []
related: [admit-a-scheduled-region-for-a-staged-elementary-family, accept-the-root-mean-square-scale-realization-law, accept-the-registered-family-realization-law-query]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, scheduling]
---
## What is being accepted

One further variant of the public `tiler_ir::schedule::ScalarProgram`, landed as a labelled draft by [`admit-a-scheduled-region-for-a-staged-elementary-family`](admit-a-scheduled-region-for-a-staged-elementary-family.md). It is implemented and tested; a tested implementation is a concrete draft, not implicit approval of its interface, so this node parks until Tom closes it. Only Tom closes it.

## The exact surface

New in `tiler_ir::schedule`:

```rust
pub enum ScalarProgram {
    // ... eight existing variants, none changed ...
    SquaredSerialSumThenEpilogue {
        axes: Vec<Axis>,
        order: ContributorOrder,
        canonical_nan_bits: u32,
        empty_identity_bits: u32,
        epilogue: PointwiseF32Expression,
    },
}
```

Nothing else in the public surface moves. `ScalarProgram` is deliberately not `#[non_exhaustive]`, so the addition is a compile error at every out-of-crate total map — which is the design: two such maps exist (`tiler-compiler`'s `boundary_carrier` and its request-subject binding) and both gained an arm rather than a wildcard. The type's own doctest, which pins that an out-of-crate exhaustive match keeps compiling, gained the variant too.

`SquaredSerialSum` keeps its shape, its tag, and its bytes; what changed there is one paragraph of its doc-comment (see below).

## Why it exists

The producing stage of a staged elementary family computes a fold *and* a chain over the fold's value, once per folded row. `tiler::rms-norm-f32@1` is the shipped instance: [`accept-the-root-mean-square-scale-realization-law`](accept-the-root-mean-square-scale-realization-law.md)'s accepted law folds `x²` over the named axes and then applies `/N`, `+eps`, and `Rsqrt` **inside the producing region**, because `r` is computed once per row and read once per point — putting the chain in the consuming pass evaluates it `N` times per row, which is a different scalar program rather than a different schedule. No variant of the previous vocabulary could state that, so the family had no scheduled region and `RegionVocabularyWall::StagedFamilyUnspellable` was what a normalization program hit.

## The choices worth objecting to

- **A general epilogue on one specific fold.** The epilogue is a whole verified `PointwiseF32Expression`, so *any* chain the physical `f32` vocabulary spells is expressible without a further variant — a mean is `a / N`, this family's scale is `Rsqrt(a / N + eps)`, a reciprocal-sum normalizer is `c / a`. The *fold* stays one variant per (prologue, combiner) pair, which is the grain `SquaredSerialSum` and `StrictSerialMaximum` already set. **The named consequence, and the thing to object to if it is wrong:** the softmax's shifting stage folds a *maximum*, so it will need its own sibling here rather than a field on this one. What it inherits unchanged is the epilogue field and every derivation threaded for it — the verifier's two rules, the identity payload, the lowering's epilogue hook, and the split refusal. The alternative was a `fold: SerialFoldKind` field crossing three combiners with the epilogue; it was rejected because most of that product denotes programs nothing registers, and admitting unreachable combinations into an accepted vocabulary is what this module's own law header argues against.
- **The epilogue's sole input ordinal names the fold's value, not a boundary tensor.** The region reads exactly one tensor — its contributor domain — so the ordinal has no buffer to name and the lowering supplies the accumulator for it. The verifier requires exactly one leaf. The cost is that one ordinal space means two things depending on which scalar program frames it; the alternative is a second expression type differing from `PointwiseF32Expression` in nothing but that reading.
- **An epilogue that computes nothing is refused.** An expression whose root is its own input leaf returns the fold unchanged, which is exactly `SquaredSerialSum` — admitting it would give one program two spellings and two canonical identities. This is the canonicality rule `broadcast_decodes_are_replicating` states for its own degenerate case. The cost is that a producer generating epilogues uniformly must special-case the empty one.
- **No parallel topology may split it**, and `multi_pass_family` and `cooperative_family` answer `None`. The epilogue applies to the *complete* fold: a partial pass applying it transforms a fragment, and one that does not is computing `SquaredSerialSum` under this variant's name. A split of this family is therefore a *pair* of scalar programs rather than a partition of one, which no cover states. The cost is real — a normalization over a long axis gets no multi-pass or cooperative alternative — and the widening is a separate ticket rather than a relaxation here.
- **`empty_identity_bits` is the *fold's* identity and the epilogue transforms it.** The program is "fold, then epilogue", so the empty case differs only in where the fold's value came from. Nothing in the shipped law reaches it (`rms-scale-empty-fold` refuses an empty fold a layer up); stating it keeps the variant's meaning total rather than conditional on its one producer.
- **`SquaredSerialSum`'s doc-comment was rewritten in place.** It said the division, the `eps` addition, the reciprocal square root, and the two multiplies "belong to the pointwise pass that consumes this reduction's result". That claim predates the accepted law and contradicts it. The variant itself is unchanged and still correct for its own uses; what the rewritten paragraph now says is that this variant is the fold *alone*, and that where a transform belongs is the operation's question rather than a schedule's.

## Evidence

Landed with five tests in `crates/tiler-ir/src/schedule/builder.rs` and `crates/tiler-ir/src/kernel/tests.rs`, each watched failing under a named deliberate perturbation: the region verifies as a serial pass with two accesses; an identity epilogue is refused (perturbed by dropping the identity-root guard); a two-leaf epilogue is refused (perturbed by dropping the one-leaf rule); the epilogue payload separates canonical identity from the bare fold's *and* from another epilogue's (perturbed by encoding under `0x26` without the payload); and no parallel topology admits it (perturbed by giving it the squaring fold's partial-pass admission). The lowering emits the chain once per output position rather than once per contributor (perturbed by dropping the epilogue argument). `every_serial_fold_family_may_commit_to_a_materialized_intermediate` counts five fold arms rather than four.

Identity is appends-only: tag `0x2A`, `0x22` through `0x29` unchanged, `tiler.schedule.v5` unstepped, and `cargo nextest run --workspace` green with **no pin edited at all**.

Bit-for-bit evidence that the vocabulary computes the right function is `the_staged_regions_compute_the_normalization_bit_for_bit` in `tiler-compiler`, against `tiler-reference`'s own normalization.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases on this node meanwhile; the variant is in use from `tiler-compiler` and labelled a draft at its definition.

## Outcome — accepted

**Accepted by Tom on 2026-08-06, as-is with no exclusion, at the live session's decision round (presented by the orchestrator, explain-then-recommend, relay source this ticket).** The variant, its sibling-per-combiner grain, the identity-epilogue and one-leaf refusals, and the no-split refusal are accepted public surface exactly as landed. The in-code draft label rewrite rides with [`account-for-a-staged-realization-stage-in-the-kernel-program`](account-for-a-staged-realization-stage-in-the-kernel-program.md)'s branch, which holds `implementation/ir`.

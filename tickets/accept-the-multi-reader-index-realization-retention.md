---
id: accept-the-multi-reader-index-realization-retention
title: Accept the multi-reader index realization retention
status: awaiting-decision
priority: p2
dependencies: []
related: [admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence, accept-the-multi-region-index-realization-surface]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, indexing]
---
## What is being accepted

A widening of the region-sequence surface [`accept-the-multi-region-index-realization-surface`](accept-the-multi-region-index-realization-surface.md) accepted on 2026-08-06. That node ruled on a vocabulary in which a handed value has exactly one reader, the immediately following stage. This one asks Tom to rule on the vocabulary in which a published value may be read by any number of later stages, with its retention recorded. Landed as a labelled draft by [`admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence`](admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence.md); only Tom closes this node.

## The exact surface

**Added to `tiler_ir::index`:**

- **`StagedIntermediate::retained_through() -> usize`** — the last ordered stage across which this published value stays live, so its lifetime is `producer()..=retained_through()`. Equal to `consumer()` on every record of a value with one reader, which is every record any registered law produces today.

**Changed in `tiler_ir::index`, behaviourally rather than in signature:**

- **`VerifiedIndexRegionSequence::try_new`** admits `StagedInputSource::Intermediate(p)` for *any* earlier stage `p`, and admits several reads of one published value — at two boundaries of one stage, or at boundaries of two stages. It previously refused both with `UnavailableIntermediate`.
- **`StagedIntermediate` records one *read*, not one value.** A value with two readers yields two records agreeing on `producer`, `producer_output`, `value_type`, `shape`, and `retained_through`, and differing in `consumer` and `consumer_input`. For a one-reader chain the record set is exactly what it always was.
- **`IndexRegionSequenceError::UnavailableIntermediate`** now means the named producer is not an earlier stage — this stage, a later one, or an ordinal no stage occupies. **`IntermediateNeverRead`** is now checked over the whole chain rather than at the following stage.

**Unchanged, deliberately:** the eight named refusals (no variant added or removed), `MAX_INDEX_REGION_SEQUENCE_STAGES`, `CanonicalIndexRegionSequenceIdentity` and its encoder, `StagedInputSource` (still not `#[non_exhaustive]`, still two variants), and the rule that a non-final stage publishes exactly one value.

## The fork this resolves, and why one arm survived

The deriving ticket names a real fork: a value surviving multiple stages, or a multi-value handoff. Run against the softmax's four refused stagings *and* against the next family with a shared intermediate, exactly one survives.

**The multi-value handoff reaches the softmax only through a copy-through, and that is disqualifying.** Two shapes were tried:

- **One stage publishes `(e, d)`.** In one region the parallel dimension and the reduction dimension are distinct, so a region writing `e_i` per point *and* folding `e` would read the scores at both and evaluate the exponential twice per element. That is a different scalar program by the standard `StagedStrictSerialSumThenPointwiseF32`'s own doc-comment already sets — and it doubles the operation's *one inexact step*, the step that carries the resolved ADR 0042 accuracy contract.
- **The folding stage publishes `(e, d)`, passing `e` through verbatim.** This works structurally and costs a full-size identity copy: an output boundary and a write that are no part of what the operation means, inside a region whose canonical identity carries them. It expresses retention by duplication rather than as a lifetime.

**The multi-reader arm is what the architecture already asks for.** AGENTS.md requires lifetimes represented explicitly and the public graph kept about what operations mean rather than how hardware runs them; a copy-through output is a *how* injected into a *what*. And the shape generalizes: layer normalization's `x - m` is read by the variance fold and again by the output pass, and any log-sum-exp sibling has the same shape. Nothing was found that the multi-reader arm cannot express and the handoff arm can, *except* a genuinely different capability — one region producing two independent folds consumed by one pass. That is filed separately at [`widen-the-region-sequence-to-a-multi-value-handoff`](widen-the-region-sequence-to-a-multi-value-handoff.md) and deferred with a trigger, rather than parked as an unresolved fork: it is not what the softmax needs and no registered family asks for it.

## The choices worth objecting to

- **Retention is derived from the declared readers, not separately declared.** A producing stage does not state how far its value is retained; `try_new` computes the span from the sources and records it. This follows the module's own "derived and checked, never declared and believed", and a separately declared span would be a second authority that could disagree with the readers. The cost is that a caller cannot express "retain this further than anything reads it" — which is a physical-planning statement, not a realization one.
- **`StagedIntermediate` stays per read rather than becoming per value.** Per value is arguably the cleaner model, and it would remove `consumer()`/`consumer_input()` in favour of a reader list. It was not taken: those two accessors are read from `crates/tiler-compiler/src/region.rs`, which this ticket's scope could not edit, and the per-boundary granularity is what this record has always had — it names one *consuming boundary*. The honest question for Tom is whether `intermediates()` should instead answer values, with reads underneath.
- **The refusal vocabulary did not grow.** A forward or self reference reuses `UnavailableIntermediate` rather than minting a named acyclicity refusal. The alternative reads better in a diagnostic and costs a ninth variant on an accepted `#[non_exhaustive]` enum for a condition the existing message now states.
- **A value may stay live across up to 63 stages with no bound of its own.** `MAX_INDEX_REGION_SEQUENCE_STAGES` bounds it transitively, since each stage publishes at most one value. No separate live-value ceiling is stated.

## Identity evidence — every existing chain unchanged, byte for byte

**By construction, not by survey.** `encode_sequence_identity` is untouched: `StagedInputSource::Intermediate` already wrote its producer ordinal in full under tag `2`, and `push_len` is injective over the whole `usize` range rather than over the range the chain rules happened to admit. The admitted preimage set widened while the map did not, so every chain expressible before encodes exactly as before, and injectivity over the wider domain follows from the same length-prefixed, tagged, ordered argument. The reasoning is recorded at the encoding site.

**Pinned anyway.** `the_landed_one_reader_chain_identities_are_unchanged_byte_for_byte` in `crates/tiler-ir/src/index/law.rs` asserts the exact length and a SHA-256 over the identity bytes of three chains — the normalization's own law at rank two and rank one (the live instance) and the plain staged template — captured on base commit `dd9def76` before either widening landed. Zero pins moved, which is what was expected. The same test covers the scalar-key widening, because a sequence identity is built from region identities and a region identity carries the projection of the scalar definitions it *reaches*.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases meanwhile; the surface is in use inside `tiler-ir` and labelled a draft there.

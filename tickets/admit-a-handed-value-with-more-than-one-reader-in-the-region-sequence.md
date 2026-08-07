---
id: admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence
title: Admit a handed value with more than one reader in the region sequence
status: in-progress
priority: p2
dependencies: []
related: [widen-the-staged-realization-law-to-the-registered-elementary-families, admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold, accept-the-multi-region-index-realization-surface]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-softmax-law
lease_expires_at: 1786070865
---
## User-visible outcome

`VerifiedIndexRegionSequence` can express a realization in which one stage's published value is read by more than one later stage, so `tiler::softmax-f32@1` has a chain its law could be written as. Today it does not, and this is the second wall holding the softmax's law — not the one the graph currently records.

## Why this exists: the softmax is unrealizable under the current chain rules, with or without the maximum key

**Fact, and it corrects a premise.** [`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md) recorded the softmax half as waiting on [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md). That key is necessary and not sufficient. The blocking fact is in `crates/tiler-ir/src/index/sequence.rs`, and it is checked rather than documented:

- a non-final stage publishes **exactly one** value, or `try_new` answers `NotChained` (`sequence.rs:306-317`);
- a handed value is read by the **immediately following** stage and by **nothing else**, or `try_new` answers `UnavailableIntermediate` — the `owed` slot is cleared by its one consumer and a second claim finds nothing (`sequence.rs:248-301`).

**The derivation.** `softmax_f32_reference_semantics` (`crates/tiler-ir/src/semantic/softmax.rs:394-408`) pins `m = max fold over x`, `e_i = Exp(x_i - m)`, `d = sum fold over e`, `c = 1.0 / d`, `r_i = e_i * c`. Every staging is refused:

1. **Publish `e`.** `S0 -> m`, `S1` reads `x` and `m` and publishes `e`, `S2` folds `sum(e)` and publishes `d`, `S3` computes `r_i = e_i * c`. `S3` needs `e`, whose producer is `S1` and whose one reader was `S2`. `UnavailableIntermediate`.
2. **Publish `d`.** `S0 -> m`, `S1` reads `x` and `m`, computes `e_i` internally and folds it to `d`. `S2` then needs `m` to recompute `e_i`, and `m` was consumed by `S1`. Nothing available.
3. **Publish the pair.** `S1` hands `(m, d)` on as two values. `NotChained`.
4. **Recompute `m` per point in the final stage.** This is a different scalar program, by exactly the argument `StagedStrictSerialSumThenPointwiseF32`'s own doc-comment already makes about a fold read more than once: the reference computes `m` once per row.

So the softmax needs either a value that survives more than one stage, or a multi-value handoff, and neither is expressible. This is a vocabulary gap, not a missing scalar.

## Scope

The design question is which of the two to admit, and it is a real fork rather than an implementation detail: a value with several readers makes the intermediate's *lifetime* span stages, which the current model deliberately refuses ("A value handed further down the chain would have to stay live across a stage that does not mention it, which the sequence deliberately cannot express rather than leaving the retention implied by stage order" — `sequence.rs:75-78`). Whatever lands must state where retention is recorded rather than implying it from stage order, and must move `CanonicalIndexRegionSequenceIdentity` coherently: the identity encodes each stage's source list, so a source naming a non-adjacent producer is a new preimage and needs its own injectivity reasoning at the encoding site.

The sequence surface is public and accepted ([`accept-the-multi-region-index-realization-surface`](accept-the-multi-region-index-realization-surface.md)), so a change to it is a public-boundary redesign and lands as a labelled draft with its own acceptance node.

## Non-goals

Writing the softmax's law. That additionally needs the maximum scalar key, and it should be one ticket once both walls are down.

## Closes when

A realization in which one published value has more than one reader is expressible and checked, its retention contract is stated rather than implied, the sequence identity encodes the wider source vocabulary injectively with the reasoning recorded at the encoding site, and every existing one-reader chain's identity is unchanged byte for byte.

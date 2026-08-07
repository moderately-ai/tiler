---
id: carry-a-multi-reader-intermediate-through-region-formation
title: Carry a multi-reader intermediate through region formation
status: in-progress
priority: p2
dependencies: []
related: [admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence, register-the-softmax-realization-law, accept-the-multi-region-index-realization-surface]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, indexing]
claimed_from: todo
assignee: agent-region-multiread
lease_expires_at: 1786075734
---
## User-visible outcome

`RegionGraph::with_realizations` derives a correct stage topology for a realization in which one published value is read by more than one stage, so the softmax's four-stage chain reaches region formation as one retained value with several readers rather than as several values.

## Why this exists

**Fact.** [`admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence`](admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence.md) widened `VerifiedIndexRegionSequence` so a published value may be read by any number of later stages. `StagedIntermediate` records one *read*: a value with two readers yields two records agreeing on `producer`, `producer_output`, `value_type`, `shape`, and `retained_through`, and differing in `consumer` and `consumer_input`.

**Fact.** `crates/tiler-compiler/src/region.rs`'s `with_realizations` walks `sequence.intermediates()` and, per record, appends one synthetic `GraphValue` and one `SyntheticIntermediate { value, producer_stage, consumer_stage }`. Under the widened vocabulary a value read twice therefore becomes **two synthetic values** with two producer/consumer edges, where the realization has one value with two readers. That misdescribes the topology: liveness, boundary derivation, and the staged identity domains would all see two independent intermediates.

**Fact — nothing is wrong today.** Every registered law publishes one value per non-final stage and every published value has exactly one reader, so the per-read walk and a per-value walk coincide record for record. `cargo nextest run --workspace` is green on the widening's own branch. This is a gap that opens when the first multi-reader law is registered, which is [`register-the-softmax-realization-law`](register-the-softmax-realization-law.md).

## Scope

Group the reads by published value before synthesizing. `SyntheticIntermediate` needs a reader list (or the topology needs a separate per-value record), and `StagedIntermediate::retained_through` is the checked span to carry rather than a re-derivation. Whatever lands must say what the two staged compiler identity domains — `tiler.compiler.index-refinement-content.staged.v1\0` and `tiler.compiler.index-refinement-occurrence.staged.v1\0` — encode for a value with several readers, and must leave every one-reader binding's bytes unchanged. The 26 distinct 16-hex and 6 distinct 64-hex pins over `crates/tiler-compiler` that [`accept-the-multi-region-index-realization-surface`](accept-the-multi-region-index-realization-surface.md) surveyed are the population to re-survey.

## Non-goals

Registering the softmax's law, and widening the sequence vocabulary further. Both are elsewhere.

## Closes when

A multi-reader realization forms a stage topology carrying one synthetic value per published value with all its readers, the staged identity domains state what they encode for it, and every one-reader binding's identity is unchanged byte for byte.

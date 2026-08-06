---
id: accept-the-registered-family-region-sequence-query
title: Accept the registered-family region-sequence query
status: done
priority: p2
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, accept-the-root-mean-square-scale-realization-law, accept-the-multi-region-index-realization-surface]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, indexing]
---
## What is being accepted

One method on the public `tiler_ir::index::FrozenIndexRealizationLawRegistry`, landed as a labelled draft by [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md). It is implemented and tested; a tested implementation is a concrete draft, not implicit approval of its interface, so this node parks until Tom closes it. Only Tom closes it.

## The exact surface

```rust
impl FrozenIndexRealizationLawRegistry {
    pub fn family_realizes_region_sequence(&self, operation: &OpKey) -> bool;
}
```

Nothing else moves. No type, no variant, no existing signature, and no encoding: the method reads the same registry row `resolve` reads and answers the same predicate `ResolvedIndexRealization::realizes_region_sequence` answers.

## Why it exists, and the alternative it replaces

**Fact.** `tiler-compiler`'s request boundary has to decide, for each occurrence, whether its registered law realizes a region *sequence* — that decision is what admits a staged elementary family as a program stage, and it is deliberately family-blind. **Fact.** The only public route to that answer before this method was `resolve(&IndexRefinementSubject)`, and `IndexRefinementSubject::derive` requires a `NumericalContractIdentity`.

**Inference.** Recognition runs once per request and is shared by every requested target, while a numerical contract is resolved *per target*. Routing the classification through a subject would therefore have made the recognizer either pick one target's contract to speak for all of them, or probe with a contract no caller stated. Both are answers derived from something the question does not depend on: `IndexRealizationLaw::realizes_region_sequence` reads the law variant alone.

## The choices worth objecting to

- **An operation key rather than a subject.** The cost is a second entry point to one fact. It is mitigated rather than argued away: the method reads `FrozenSemanticRegistry::index_realization_law`, which is the same row `resolve` reads, and `the_family_region_sequence_query_agrees_with_the_resolved_law` asserts the two answers agree for a derived subject of each family. The alternative — exposing a contract-free subject constructor — is a larger surface for the same answer.
- **`false` for an unregistered operation rather than an `Option`.** The caller's question is "is this occurrence a staged stage", and an operation with no law is not one. An `Option` would make every caller flatten the absent case to `false` anyway, and the flattening is where a fail-open would be written by accident. The cost is that "no law" and "a single-region law" are indistinguishable here; refinement reports the absent law by name when the occurrence is lowered, which is the site where the distinction is actionable.
- **`bool` rather than a stage count.** A stage count would be a second account of the law's realization, which region formation already reads through `realize_sequence`. The cost is that a caller wanting the count still needs a subject.

## Evidence

`the_family_region_sequence_query_agrees_with_the_resolved_law` (`crates/tiler-ir/src/index/refinement.rs`): `tiler::rms-norm-f32@1` true, `tiler::multiply-f32@1` false, `tiler::softmax-f32@1` (a registered operation with no law) false, and agreement with `resolve(...).realizes_region_sequence()` for a two-occurrence program holding both families. Watched failing under a deliberate perturbation: weakening the body to `is_some()` fails the multiply row.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases on this node meanwhile; the method is in use from `tiler-compiler` and labelled a draft at its definition.

## Outcome — accepted

**Accepted by Tom on 2026-08-06, as-is with no exclusion, at the live session's decision round (presented by the orchestrator, explain-then-recommend, relay source this ticket).** The exact surface above — `family_realizes_region_sequence(&OpKey) -> bool`, `false` for an unregistered operation — is accepted public surface on `FrozenIndexRealizationLawRegistry`.

**One alignment deliberately deferred, with its trigger.** The method's own doc-comment still opens "**Labelled draft.**" and points here; rewriting it to record this acceptance is a `crates/tiler-ir` edit, and `implementation/ir` is exclusively held by the in-flight [`admit-a-scheduled-region-for-a-staged-elementary-family`](admit-a-scheduled-region-for-a-staged-elementary-family.md) worker. The coordinator applies the doc-comment update at that ticket's integration, and this ticket's closure is complete when that lands.

---
id: accept-the-registered-family-realization-law-query
title: Accept the registered-family realization-law query
status: done
priority: p2
dependencies: []
related: [admit-a-scheduled-region-for-a-staged-elementary-family, accept-the-registered-family-region-sequence-query, accept-the-root-mean-square-scale-realization-law]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, indexing]
---
## What is being accepted

One method on the public `tiler_ir::index::FrozenIndexRealizationLawRegistry`, landed as a labelled draft by [`admit-a-scheduled-region-for-a-staged-elementary-family`](admit-a-scheduled-region-for-a-staged-elementary-family.md). It is implemented and tested; a tested implementation is a concrete draft, not implicit approval of its interface, so this node parks until Tom closes it. Only Tom closes it.

## The exact surface

```rust
impl FrozenIndexRealizationLawRegistry {
    pub fn family_realization_law(&self, operation: &OpKey) -> Option<&IndexRealizationLaw>;
}
```

Nothing else moves. No type, no variant, no existing signature, and no encoding: the method reads the same registry row `resolve` and `family_realizes_region_sequence` read, and performs no contract check, no authority projection, and no realization.

## Why it exists, and what could not serve instead

**Fact.** A physical planner spelling one stage of a staged family has to know what that stage *computes* — which axes it folds, which payload its epilogue carries, what its consuming pass evaluates. **Fact.** That is the law's content: `IndexRealizationLaw::StagedRootMeanSquareScaleF32` names two attribute field identifiers, and the occurrence's typed attribute record holds the values.

**Inference — no other route reaches it.** Deriving it from the *operation key* would key the planner to a family, so a second family registering one of these laws would need a second arm for one template, which is exactly the family-blindness [`accept-the-registered-family-region-sequence-query`](accept-the-registered-family-region-sequence-query.md) exists to protect. Deriving it from the *shapes* is impossible: a `[2, 2]` operand handed a `[2]` value names two different reductions. Deriving it from `resolve` needs an `IndexRefinementSubject`, which needs the whole `SemanticProgram`, which the physical layer deliberately does not carry.

Answering with the closed typed law is what lets a consumer be written against the **vocabulary** — one arm per law, a fail-closed wildcard for the rest — which is the same discipline `law.rs`'s own interpretation follows, and it is how `tiler-compiler`'s `staged_plan` is written.

## The choices worth objecting to

- **A second entry point beside `family_realizes_region_sequence`, which is itself still awaiting a decision.** Both read `FrozenSemanticRegistry::index_realization_law`, and the bool query is a projection of this one: `family_realizes_region_sequence(op)` is `family_realization_law(op).is_some_and(realizes_region_sequence)`. **Collapsing them into this method alone is the live alternative and is Tom's to take** — it would need `IndexRealizationLaw::realizes_region_sequence` to become public, which is a further widening of the same enum's surface, and it would edit a node already parked for decision. The two were kept apart rather than merged during implementation precisely because merging them changes a parked node's stated surface. The cost of keeping both is one fact with two entry points; the cost of merging is a second public predicate on the law enum.
- **Returning the law rather than a projection of it.** A narrower answer — "the stage count", "the axes attribute" — would be a second account of the law's content, and each new consumer would need its own. The cost is that the whole closed enum becomes reachable from outside `tiler-ir`, so a consumer *can* match on a variant this crate meant to interpret alone. It is `#[non_exhaustive]`, so such a match must carry a wildcard, and the one consumer's wildcard is fail-closed.
- **`None` for an unregistered operation rather than an error.** A family with no registered law has no realization this authority describes, and the caller's question is "what does this stage compute". Refinement reports the absent law by name when the occurrence is lowered, which is where the distinction is actionable.
- **A borrow rather than a clone.** The registry is `Arc`-backed and immutable, so the borrow costs nothing and a caller that needs an owned law clones one variant rather than every caller paying for it.

## Evidence

`tiler-compiler`'s `staged_plan` is the only consumer, and `the_staged_regions_compute_the_normalization_bit_for_bit` drives it end to end: the law read through this method decides the reduced axes, the folded extent, and the `eps` payload of a region whose interpreted result matches `tiler-reference`'s normalization bit for bit. `a_staged_family_program_spells_both_stages_and_names_the_program_scope_wall` drives the same route through `compile()`.

## Closes when

Tom accepts, accepts with a named exclusion, rejects, or takes the collapse named above. Nothing releases on this node meanwhile; the method is in use from `tiler-compiler` and labelled a draft at its definition.

## Outcome — accepted

**Accepted by Tom on 2026-08-06, as-is with no exclusion, at the live session's decision round (presented by the orchestrator, explain-then-recommend, relay source this ticket).** Both methods stand: the previously accepted `family_realizes_region_sequence` for recognition and this law query for physical planning — the collapse was presented as the live alternative and declined in favour of keeping the recognizer's minimal dependency; it remains available later if the duplication warrants it. The in-code draft label rewrite rides with [`account-for-a-staged-realization-stage-in-the-kernel-program`](account-for-a-staged-realization-stage-in-the-kernel-program.md)'s branch, which holds `implementation/ir`.

## Current source correction — 2026-08-09

The deferred label rewrite landed. `FrozenIndexRealizationLawRegistry::family_realization_law` now carries an **Accepted public surface** paragraph naming this ticket and the 2026-08-06 decision; no draft label remains. The method's surface and the decision to keep the two queries separate are unchanged.

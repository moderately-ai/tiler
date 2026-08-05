---
id: admit-the-sub-tensor-selection-family
title: Admit the sub-tensor selection operation family
status: in-progress
priority: p2
dependencies: []
related: [admit-a-position-selecting-slice-for-the-rotary-table, project-only-the-final-position-logits, admit-live-extent-operands-to-payload-indexing, admit-the-reindex-and-broadcast-operation-families, own-operation-family-support-matrix, reclassify-language-model-work-as-a-conformance-track]
scopes: [contracts/foundation, implementation/ir, implementation/reference, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, operation-families, slice, breadth, class-generic-capability]
claimed_from: todo
assignee: agent-sub-tensor
lease_expires_at: 1785895177
---
## User-visible outcome

A program can read a rectangular sub-region of a tensor — an injective, non-surjective coordinate map — as one governed operation family with its own refusals and its own reference evaluator, instead of each workload that needs a selection inventing a narrower one.

## Why this is its own ticket

**Fact.** [The support matrix](../docs/roadmap.md#operation-family-support-matrix) holds sub-tensor selection at R1: no contract defines a slice and no key exists. `tiler::reindex-f32@1` admits bijective permutation, split, merge, unit-axis insertion or removal, and within-axis reversal, and its registered `reindex.split.not-surjective` refusal names a non-surjective map as outside that family; `tiler::broadcast-f32@1` does not admit one either.

**Fact — the family had two consumers and no owner.** Two tickets each need a selection and neither is the family: [`admit-a-position-selecting-slice-for-the-rotary-table`](admit-a-position-selecting-slice-for-the-rotary-table.md) carries a decode step's position-identity argument, and [`project-only-the-final-position-logits`](project-only-the-final-position-logits.md) carries a prefill residency argument. Until 2026-08-04 the first also carried the family's design in a "Required design" section and depended on [`integrate-the-autoregressive-decode-loop`](integrate-the-autoregressive-decode-loop.md), so a generic operation family was scheduled behind a complete consumer decode loop. Filed under [`reclassify-language-model-work-as-a-conformance-track`](reclassify-language-model-work-as-a-conformance-track.md), whose classification is that the capability is generic even though both of its triggers are workload observations.

**Fact — a constant-offset form is reachable today and a symbolic-offset form is not.** `IndexNode` at `crates/tiler-ir/src/index/model.rs:94`–`109` has five variants: `Constant`, `Dimension`, `LinearCombination` whose constant and per-term coefficients are `IndexInteger`, and `FloorDiv` and `Modulo` whose `divisor` is a `SourcedExtent`. `SourcedExtent` is the only carrier of a possibly-symbolic extent and it appears in no other position, so `t + k` for a literal `k` is expressible and `t + C` for a bound symbol `C` is not. Reproduce with `grep -n 'enum IndexNode' -A 16 crates/tiler-ir/src/index/model.rs`.

## Required delivery

- **The family's form, decided before it is registered.** A general `Slice`, a bounded offset-and-extent selection along one or more axes, or a strided form — argued against the rule [ADR 0087](../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md) already applied to contraction: one keyed family carrying a canonical structure attribute, or a key per shape class, with a frontend never choosing among keys.
- **Semantic identity and validation.** A governed `OpKey`, a canonical attribute carrying the per-axis selection, and construction-time refusals for a selection that leaves the source's declared extent, an empty selection, a duplicated axis, and an axis out of range — each a typed provider diagnostic in the shape `StrictSerialSumF32::infer` already sets in `crates/tiler-ir/src/semantic/registry.rs`.
- **A normative reference and a reference evaluator** for the exact signature.
- **The symbolic-offset boundary, stated rather than assumed.** Either the constant-offset form is admitted and the symbolic-offset form refuses by name with its own reconsideration trigger, or the `IndexNode` gap above is closed as part of this work. Those are different sizes and this ticket records which it took. [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md) owns the physical half of a live extent and is not a substitute for the index-layer gap.
- **A support-matrix row movement**, to the rung the delivered layers actually support, with both existing triggers retained and their evidence attached.

## Non-goals

Any lowering capability, physical schedule, or backend emission; those follow and are separately scheduled. Gather and scatter, which are a different access class under [Q-SHAPE-007](../docs/open-questions.md#q-shape-007--indirect-gatherscatter-relations) and owned by [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md). Deciding which of the two triggers is delivered first — this ticket owes them the family, not an ordering.

## Closes when

The family's form is decided with its ground, a governed key verifies and reference-evaluates a constant-offset selection, every listed refusal is watched firing, the symbolic-offset boundary is explicit, and the support-matrix row moves off R1 with both triggers preserved.

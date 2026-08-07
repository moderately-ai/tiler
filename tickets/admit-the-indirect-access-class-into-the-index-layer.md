---
id: admit-the-indirect-access-class-into-the-index-layer
title: Decide whether the index layer admits a data-dependent access class
status: todo
priority: p2
dependencies: [accept-adr-0107-indirect-gather-semantic-family]
related: [admit-an-indirect-gather-family-for-tied-embedding-lookup, emit-the-indirect-gather-on-metal, implement-index-domain-predicates]
scopes: [implementation/ir, contracts/foundation, contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, indexing, gather, class-generic-capability, needs-tom]
---
## User-visible outcome

The question of whether an index region may name a second tensor as a coordinate source is answered, with its consequence for the direct-access verifier stated, rather than left as the boundary `tiler::gather-f32@1` currently fails closed at.

## Why this exists, and why it is a decision rather than an implementation

**Fact.** `tiler::gather-f32@1` is registered and reference-evaluated at `fab1f6db`'s successor under [ADR 0107](../docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md), and no program containing one reaches a plan. Reproduce the boundary with `grep -n 'gather-f32' crates/tiler-compiler/src/policy.rs` — one hit, in `UNPLANNED_OPERATIONS` — and `grep -c 'gather' crates/tiler-compiler/src/fusion_legality.rs`, which returns 0.

**Fact — the obstacle is the access record's shape, not a missing expression form.** `crates/tiler-ir/src/index/model.rs:138`'s `AccessData` carries `tensor: u32`, a single tensor ordinal, so an access has nowhere to name a second tensor as a coordinate source. `IndexNode` at `model.rs:105` has five variants and every operand of every one is a literal, a domain-dimension ordinal, or one declared shape symbol. `IndexExprClass` at `model.rs:58` has three variants and no data-dependent member, and its `join` at `:83` is an exhaustive match written so that adding a class is a build error.

**Inference — this cannot be answered by implementing it.** [ADR 0046](../docs/decisions/0046-separate-logical-access-from-storage-addressing.md)'s consequences admit indirect operations *on the condition* that the verifier for the initial direct-access language is not weakened. Every bounds proof, interval propagation, and totality argument in `crates/tiler-ir/src/index/builder/proof.rs` is written over expressions whose operands are literals, dimensions, and symbols; a variant that reads tensor data makes `verify_accesses` unable to decide the very property it exists to decide. So the question is whether that condition can be met at all, and by what construct — not how to add a variant.

## What this ticket must answer

- Whether a data-dependent coordinate enters as a *second tensor on the access* or as an *expression variant*, and what each does to `verify_accesses`' three routes (interval, cheap-predicate, exhaustive).
- What the bounds obligation becomes when it cannot be discharged: a retained `IndexDomainUnknownReason`, a required host-side pre-dispatch validation with a named publication boundary, or a refusal.
- Whether `LogicalAccess` gains a variant, and what a `FusionOperationRole` for the family would then discharge — today it deliberately has none, because `CoordinateRelation`'s contract asserts a discharge the index verifier cannot perform for a coordinate it cannot see.
- Whether ADR 0046 is amended, extended by a second subordinate record, or left untouched.

## Non-goals

Scatter, and any data-dependent output shape. Backend emission, which `emit-the-indirect-gather-on-metal` owns and which depends on this.

## Closes when

The question is answered with its verifier consequence stated and its ADR consequence landed, or it is deliberately deferred with a reconsideration trigger and a `## Trigger check log`.

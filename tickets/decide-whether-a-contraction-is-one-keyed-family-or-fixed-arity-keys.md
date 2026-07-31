---
id: decide-whether-a-contraction-is-one-keyed-family-or-fixed-arity-keys
title: Decide whether a contraction is one keyed family or fixed-arity keys per shape class
status: todo
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface]
related: [scope-einsum-contraction-support, spike-first-metal-contraction-vertical, decide-whether-to-admit-a-distributivity-permission, own-operation-family-support-matrix]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [decision, semantics, contraction, language-model]
---
## User-visible outcome

The reserved question that rung L3 cannot start without is put to Tom as one atomic choice with the workload evidence attached, and its answer is recorded in an accepted ADR — so "how many governed keys does an attention einsum need" stops being a blocker with no owner.

## Evidence prerequisite

**Fact — the question is reserved and nothing schedules it.** The [Milestone 6 framing](../docs/roadmap.md#framing-what-a-tensor-contraction-family-would-impose) reserves it for Tom: "Whether a contraction is one keyed family carrying an index-structure attribute, or a set of fixed-arity keys per shape class." [Q-SEM-015](../docs/open-questions.md#q-sem-015--tensor-contraction-matmul-batched-matmul-and-einsum) names it as one of three choices the semantic half's closure requires. The [ladder](../docs/roadmap.md#the-ladder) makes L3's activation trigger depend on it — "L3 cannot be specified until it is settled, because the answer decides how many governed keys an attention einsum needs" — and [`scope-einsum-contraction-support`](scope-einsum-contraction-support.md) is `done` having delivered the framing rather than the decision. Of the three reserved choices, only the distributivity one has a ticket, in [`decide-whether-to-admit-a-distributivity-permission`](decide-whether-to-admit-a-distributivity-permission.md).

**Fact — the demand trigger has fired.** Q-SEM-015's trigger is "a named workload or frontend lowering requires a tensor contraction". The pinned `Qwen/Qwen3-0.6B-Base` profile at rung L1 is that workload.

**Fact — the workload evidence, from the L2 derivation.** [The transformer operation and shape surface derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md) resolves 253 contraction occurrences per forward pass into exactly three index structures: `td,od->to` at 197 occurrences over six distinct weight shape classes; `grtd,gsd->grts` at 28; and `grts,gsd->grtd` at 28. Three facts in that derivation bear directly on the choice and are the reason this ticket exists rather than a restatement of the framing.

- **Six shape classes are one index structure.** All seven weighted projections plus the vocabulary projection instantiate `td,od->to`, differing only in extents — `[2048, 1024]`, `[1024, 1024]`, `[1024, 2048]`, `[3072, 1024]`, `[1024, 3072]`, `[151936, 1024]`. A reader who counts shape classes concludes six keys; a reader who counts index structures concludes three. A key is identity and an extent is not.
- **No structure is `[M, K] x [K, N]`.** The checkpoint stores every projection weight as `[out_features, in_features]`, so the contracted index is the last axis of both operands. A fixed matmul key therefore requires a transposing `Reindex` on all 197 weighted occurrences, or a pre-transposed copy of 2.22 GiB of F32 weights.
- **Grouped-query attention is free under one form and materialized under the other.** Indexing heads as eight groups of two repetitions makes the score contraction `grtd,gsd->grts`, where the key operand simply does not mention the repetition index — an ordinary free index, and precisely the property the framing names as a contraction's defining feature. Under fixed-arity `[B, M, K] x [B, K, N]` keys the same relation must be materialized as a `Broadcast` to sixteen heads plus a `Reindex`, adding 56 occurrences per forward pass and a `[16, S, 128]` intermediate the general form never builds.

## Required delivery

- Present the choice as one atomic question with a small worked tensor program — the score contraction is the discriminating example, because it is where the two answers differ observably rather than only in registry size. State what each option enables and prevents, with point, counterpoint, and a recommendation backed by the evidence above.
- Do not bundle the other two reserved choices. Whether a contraction node may consume more than two operands is independent, and this workload supplies no evidence on it, which is worth saying rather than manufacturing pressure. Distributivity has its own ticket.
- Record the answer in an ADR under `docs/decisions/`, with its acceptance carried by a separate `accept-adr-NNNN-*` ticket so the work graph can distinguish written from decided.
- Update Q-SEM-015, the Milestone 6 framing's reserved-decisions section, and the contraction row of the [support matrix](../docs/roadmap.md#operation-family-support-matrix) in the same change that accepts the ADR.

## Non-goals

Admitting the family. This ticket settles identity alone. Registering a key, writing an inference routine, emitting an access relation, and every part of contraction *planning* stay where they are: the planning half is gated behind [`prototype-optimizer-conformance-gate`](prototype-optimizer-conformance-gate.md) and an executed compiled program, and that gate is unaffected by this decision.

## Reconsideration trigger

Active now. If the workload is superseded before the decision is taken, re-derive the index-structure count from the replacement — the three-structure result is a property of this checkpoint's layout and grouped-query configuration, not of transformers generally.

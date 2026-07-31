---
id: design-attention-program-vertical
title: Design the first complete attention-program vertical
status: in-progress
priority: p1
dependencies: [spike-first-metal-contraction-vertical, scope-transformer-nonlinear-normalization-and-reductions]
related: [implement-general-dag-partitioning, implement-boundary-property-enforcers, implement-analytical-component-cost-model, admit-the-attention-contraction-structures, compose-rotary-position-embedding-from-reindex-and-broadcast, admit-the-grouped-query-head-layout-reindex-profile, assemble-the-causal-self-attention-block-program, realize-the-attention-contractions-on-metal, plan-the-materialized-attention-decomposition, integrate-the-attention-block-into-the-runtime, retain-the-c1-attention-block-conformance-evidence, plan-the-recomputing-attention-decomposition, scope-causal-structure-aware-attention-schedules]
scopes: [research/program-planning, contracts/optimizer, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [design, attention, transformer, vertical-slice, metal, language-model]
claimed_from: todo
assignee: loop-design-atten
lease_expires_at: 1785533981
---
Design one complete, fixed-profile causal self-attention tensor program for the
selected workload and use it to expose the next planning and runtime gaps.

## User-visible outcome

The resulting delivery graph must culminate in compiling, executing, and
validating one attention block on Metal, not merely implementing isolated
attention-related operators.

## Required design

- Show the ordered inputs, typed operations, intermediate values, named
  outputs, shapes, masks, head layout, scaling, positional transformation, and
  observable result.
- Identify legal candidate boundaries for Q/K/V projection, positional
  encoding, score formation, masking, softmax, value composition, and output
  projection.
- Compare a conventional multi-kernel program with any specialized fused
  alternative without assuming the largest fused region is best.
- State required boundary properties, materializations, synchronization,
  lifetimes, feasibility predicates, cost inputs, and numerical evidence.
- Explain why rejected candidates are incorrect, infeasible, or dominated.

## Ticket-producing outcome

File dependency-ordered tickets for the smallest conventional attention
vertical first. File specialized fusion or opaque-call alternatives only when
their prerequisites and evidence are named. Include an integration ticket whose
success is a complete attention result compared with the normative reference.

## Closes when

The complete attention program and its correctness boundaries are durably
specified; at least one realizable program decomposition survives; all missing
capabilities have scoped tickets; and unsupported workload cases fail closed.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L4** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** both L3 and L3′ deliver.

**Rests on:** L3 and L3′.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Delivered — 2026-07-31

The durable record is [First attention program vertical](../docs/research/program-planning/first-attention-program-vertical.md), with the [C1 attention-block reference probe](../spikes/program-planning/attention-block-reference/README.md) as its retained experiment. What it delivers is the complete fixed-profile prefill block as twenty-two typed steps over exact C1 and B1 shapes, twelve ordered inputs and three ordered named outputs, the elimination over its decompositions, and the feasibility predicates. What it does *not* deliver is the block: the L4 row's stated capability is ticket 7 below.

**Two decompositions survive.** D-A materializes the `[8, 2, T, S]` tensors and is the correctness baseline; D-B materializes only two `[8, 2, T]` row statistics and recomputes the score contraction, needing 1,150,287,880 bytes at the B1-d prefill row where the only reachable D-A plan needs 18,329,108,488. Neither dominates: D-A is what ships first and the two are unmeasured against each other because neither attention contraction has been timed at any shape.

**Eight candidates are rejected with grounds**, of which the load-bearing one is the flash-attention shape: factoring the softmax denominator out of the value contraction consumes **distributivity**, a dimension no contract Tiler can express grants, so it is a settled legality position rather than an unimplemented optimization, and its rejection must not name reassociation.

**Three new unresolved decisions**, continuing L3′'s D-1…D-5 and L3's D-6…D-8: **D-9**, whether a schedule may omit masked contributors from the value contraction — measured to be a signed-zero value change, forbidden today, scoped by ticket 10; **D-10**, whether `Reindex` admits a within-axis coordinate permutation, which `rotate_half` needs, landing on [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md); **D-11**, the transient-memory feasibility threshold, which no target profile declares.

**Ten dependency-ordered delivery tickets**, wired to the existing contraction, non-linear, and structural verticals rather than duplicating them:

| Order | Ticket | Waits on |
| --- | --- | --- |
| 1 | [`admit-the-attention-contraction-structures`](admit-the-attention-contraction-structures.md) | [`admit-the-contraction-normative-reference`](admit-the-contraction-normative-reference.md) |
| 2 | [`compose-rotary-position-embedding-from-reindex-and-broadcast`](compose-rotary-position-embedding-from-reindex-and-broadcast.md) | [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md) |
| 3 | [`admit-the-grouped-query-head-layout-reindex-profile`](admit-the-grouped-query-head-layout-reindex-profile.md) | [`admit-the-reindex-and-broadcast-operation-families`](admit-the-reindex-and-broadcast-operation-families.md) |
| 4 | [`assemble-the-causal-self-attention-block-program`](assemble-the-causal-self-attention-block-program.md) | 1, 2, 3, [`admit-the-softmax-family`](admit-the-softmax-family.md) |
| 5 | [`realize-the-attention-contractions-on-metal`](realize-the-attention-contractions-on-metal.md) | 1, [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md) |
| 6 | [`plan-the-materialized-attention-decomposition`](plan-the-materialized-attention-decomposition.md) | 4, 5 |
| 7 | [`integrate-the-attention-block-into-the-runtime`](integrate-the-attention-block-into-the-runtime.md) | 6, [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) |
| 8 | [`retain-the-c1-attention-block-conformance-evidence`](retain-the-c1-attention-block-conformance-evidence.md) | 7 |
| 9 | [`plan-the-recomputing-attention-decomposition`](plan-the-recomputing-attention-decomposition.md) | 7 |
| 10 | [`scope-causal-structure-aware-attention-schedules`](scope-causal-structure-aware-attention-schedules.md) | 8 |

**Rung L5's trigger has not fired.** It reads "L4 delivers a block", and this rung delivered L4's design. What it hands L5 is the seam: `k_rope` and `v_heads` as retained program outputs, and `S` kept a separate extent symbol from `T` so a decode step is a binding change rather than a graph change.

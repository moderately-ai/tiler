---
id: design-attention-program-vertical
title: Design the first complete attention-program vertical
status: todo
priority: p1
dependencies: [spike-first-metal-contraction-vertical, scope-transformer-nonlinear-normalization-and-reductions]
related: [implement-general-dag-partitioning, implement-boundary-property-enforcers, implement-analytical-component-cost-model]
scopes: [research/program-planning, contracts/optimizer, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [design, attention, transformer, vertical-slice, metal, language-model]
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

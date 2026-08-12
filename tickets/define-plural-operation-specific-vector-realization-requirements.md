---
id: define-plural-operation-specific-vector-realization-requirements
title: Define plural operation-specific vector realization requirements
status: todo
priority: p1
dependencies: [admit-vector-lane-bindings-into-the-schedule-vocabulary, admit-fixed-vector-ssa-and-unmasked-memory-into-kernel-ir]
related: [declare-cpu-vector-realization-facts-in-the-target-profile, separate-vector-operand-alignment-from-target-realization, establish-vector-execution-form-numerical-authority]
scopes: [implementation/ir, implementation/compiler, contracts/decisions, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, vector, feasibility, public-boundary, identity, fail-closed]
---
## User-visible outcome

A vectorized region declares every exact vector operation it requires, without invalid field combinations or an optional singleton that silently drops simultaneous obligations.

## Accepted boundary — 2026-08-11

Use a canonical sorted, duplicate-refusing collection of operation-specific atomic subjects. Fixed and scalable shapes are distinct. Arithmetic, contiguous load, and contiguous store are closed variants carrying only meaningful fields. Gather waits for index type/width semantics; horizontal accumulation remains below the schedule boundary.

The alpha design adds no independent row-count cap. Existing complete structural budgets still refuse oversized subjects atomically.

## Required delivery

- Derive the population from the accepted vector schedule/lowering algebra rather than inventing operations in the target layer.
- Make impossible operation/masking/address-space combinations unconstructible. Use a neutral memory-domain vocabulary rather than depending upward on kernel-owned `AddressSpace`.
- Carry a canonical collection through intrinsic derivation, feasibility, explanation, and identity. Every member must resolve; empty means no vector requirement.
- Begin with the smallest exact arithmetic-only population whose numerical and host authorities exist. Unsupported operations stay unrepresentable or `Unknown`, never generic `lane arithmetic`.
- Perturb subject kind, shape, lane count, dtype, operation, memory domain, masking/access form, duplicate ordering, and omission independently.

## Closes when

One plural carrier describes the exact vector operation conjunction and no legal construction can omit or synthesize a required member.

---
id: separate-vector-operand-alignment-from-target-realization
title: Separate vector operand alignment from target realization identity
status: todo
priority: p1
dependencies: [admit-vector-lane-bindings-into-the-schedule-vocabulary]
related: [declare-cpu-vector-realization-facts-in-the-target-profile, define-plural-operation-specific-vector-realization-requirements]
scopes: [implementation/ir, implementation/compiler, contracts/optimizer, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, vector, alignment, applicability, public-boundary]
---
## User-visible outcome

A vector form requiring alignment `A` accepts an operand proved more strongly aligned and refuses an insufficient or unknown alignment, without multiplying exact target rows.

## Fact — 2026-08-11

The adopted research explicitly left alignment as a subject-dimension-versus-applicability question. Exact equality between target-required and operand-proved alignment is wrong: proof `32` must satisfy requirement `16`.

## Required delivery

- Keep exact operation/form realization identity separate from the operand's proven alignment.
- State the selected realization's minimum alignment and compare the exact candidate operand proof by the governed `>=`/divisibility relation before selection commits.
- Unknown alignment is an applicability miss, not zero/default alignment and not target infeasibility.
- Aggregate repeated uses conservatively and explain the exact operand/form that failed.
- Perturb stronger, equal, weaker, unknown, non-power-of-two, overflow, and mismatched-operand cases independently.

## Closes when

Stronger proof satisfies weaker requirement, insufficient/unknown proof refuses, and no exact-subject row is duplicated merely to encode operand alignment.

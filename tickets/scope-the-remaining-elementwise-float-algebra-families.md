---
id: scope-the-remaining-elementwise-float-algebra-families
title: Scope the remaining elementwise float algebra families
status: deferred
priority: p2
dependencies: []
related: [scope-the-fused-multiply-add-semantic-family, select-the-first-general-elementary-function-keys, scope-the-standalone-extrema-and-clamp-families, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, numerics, pointwise, deferred]
---
## User-visible outcome

`Subtract`, `Divide`, and the unary algebraic operations reach the same delivered state `tiler::add-f32@1` and `tiler::multiply-f32@1` already have, so that a frontend lowering that needs a subtraction stops being blocked on a family that four accepted ADRs already describe.

## Why this is deferred rather than open, and what this track is *not*

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md) splits elementwise float arithmetic by arity and by numerical obligation: F-05 unary float arithmetic is one operand and one result, "exactly rounded per [ADR 0024](../docs/decisions/0024-initial-arithmetic-rounding.md) where the operation is algebraic", with `abs`, `negate`, and `sign` exact and owing separate signed-zero and NaN statements; F-06 binary float arithmetic is two operands of one identical resolved type with **no ambient promotion or autocast**, "separate rounding per operation", and `divide` owing its zero-divisor and inexactness behaviour.

**Fact — two of F-06's operations are delivered and the rest of both families are not.** [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) records `constant-f32`, `add-f32`, and `multiply-f32` at R6 with a bounded R7 execution row, and holds `Subtract`, `Divide`, negation, and required `Fma` at R2 in one row whose own evidence says "no key for `Subtract`, `Divide`, negation, or `Fma` exists in the standard registry". This ticket carries the first three of those four; the fourth has its own track and its own ticket, because a single-rounding fused operation is a different correctness argument and a different physical precondition.

**Inference — these three are one track.** They share the exact-rational-then-one-rounding oracle shape, the equal-shape rank rule with the narrow rank-zero scalar admission, the refusal of mixed precision by name, and the same minimum physical route: [the minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) places F-05 and F-06 together in its *covered — direct scalar or map route* class, where "the profile must have a serial or direct kernel, and refusing one is a defect". `Divide` adds exactly one thing the other two do not owe — a zero-divisor rule and the reciprocal permission [Q-SEM-004's sibling Q-SEM-001](../docs/open-questions.md) holds — and that is an added field rather than a second correctness argument.

## Activation trigger

A named workload or frontend lowering needs one of the three, stated as the matrix row's own trigger states it: "Each entering operation requires a key, an evaluator, a fusion role, and a backend realization before it may be claimed above R2; `Divide` additionally needs its reciprocal permission resolved under Q-SEM-001."

## What the work would be, when it starts

Per entering operation: the key and its schema, the exact-rational evaluator with one rounding at materialization, the signed-zero and NaN statements that ADR 0024 does not supply because it fixes rounding and not special values, the fusion role, the `ScalarProgram` spelling, and the backend emission — plus, for `Divide` alone, the zero-divisor result and the reciprocal-substitution permission, which must be resolved rather than assumed because two delivered families already sit on opposite sides of it: the activation and normalization pin a division and withhold the permission to become a reciprocal multiply, and the softmax pins the multiply and withholds the permission to become a division.

## Explicit non-goals

- `Fma`, which is [`scope-the-fused-multiply-add-semantic-family`](scope-the-fused-multiply-add-semantic-family.md)'s and is not a composition of a multiply and an add.
- Extrema and clamp, which are [`scope-the-standalone-extrema-and-clamp-families`](scope-the-standalone-extrema-and-clamp-families.md)'s under ADR 0023.
- Transcendentals, which are [`select-the-first-general-elementary-function-keys`](select-the-first-general-elementary-function-keys.md)'s: an algebraic operation owes exact rounding and no accuracy contract, and conflating the two is what the taxonomy's D5 independence rule forbids.
- Any implicit promotion. A mixed-precision application is refused by name, which is on the taxonomy's intentionally-invalid list and must never become a ticket.

## Closes when

Each of the three has a key, an evaluator, a fusion role, and a backend realization, or is explicitly recorded as unneeded by any named consumer; and `Divide`'s reciprocal permission is resolved under Q-SEM-001 rather than inherited from either delivered precedent.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-17** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-05 and the undelivered half of F-06 and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No named workload needs a standalone subtraction, division, or negation: the pinned workload's subtractions and divisions are all *embedded* in registered composite families — the softmax's maximum subtraction and reciprocal multiply, the activation's division — each pinned inside its own key's normative reference rather than expressed as a general operation. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.

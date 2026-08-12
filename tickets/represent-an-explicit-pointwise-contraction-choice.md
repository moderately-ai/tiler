---
id: represent-an-explicit-pointwise-contraction-choice
title: Represent an explicit pointwise contraction choice
status: blocked
priority: p2
dependencies: [admit-the-fused-multiply-add-pointwise-body-under-a-contracting-contract]
related: [scope-the-fused-multiply-add-semantic-family, record-the-contraction-choice-a-fused-fold-actually-made]
scopes: [implementation/ir, implementation/compiler, implementation/metal, research/reference, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, scheduling, identity, fma]
---
## User-visible outcome

When a numerical contract permits contraction, the planner may retain a target-qualified pointwise alternative that explicitly contracts named multiply/add sites, compare it against the fully materialized baseline, and select it only when it is the lowest-cost complete valid plan.

## Accepted boundary — 2026-08-12

**Decision — accepted by Tom in the live coordination session.** This is a physical realization choice over an unfused semantic body under ADR 0015, not the semantic fused-multiply-add family whose one rounding is required program meaning. The materialized plan owned by [`admit-the-fused-multiply-add-pointwise-body-under-a-contracting-contract`](admit-the-fused-multiply-add-pointwise-body-under-a-contracting-contract.md) lands first and remains a complete costed alternative; this ticket adds a performance alternative rather than a fallback or replacement.

A contract permission or one region-wide boolean is not a realization. One pointwise DAG may contain several or overlapping multiply/add adjacencies, so the admitted physical vocabulary must identify the exact contracted sites in a deterministic bounded carrier. Verification must prove that every named site is a legal multiply feeding the named add, that no site is duplicated or ambiguously overlaps another, that every uncontracted operation retains its stated rounding boundary, and that the selected target/provider can realize the exact form. Absence, silence, malformed sites, or unsupported overlap refuse by a typed cause; nothing infers contraction from backend flags or from writing separate multiply and add operations.

The exact choice must be consumed, not merely stored: schedule and kernel identity encode it injectively; kernel lowering emits an explicit ternary fused construct; the verifier and emitter reject decomposition; reference/conformance distinguishes single from double rounding on a discriminating input; `RealizationWitness` reports the chosen sites rather than `BackendOrderUndeclared`; explain names both admitted and rejected choices. Append-only tags may preserve old bytes, but each owning encoder must prove that locally. This ticket does not authorize revising the semantic FMA operation or treating a backend's incidental contraction as evidence.

## Performance and boundedness

Enumerating eligible sites is bounded by the already-bounded pointwise expression. The provider must state a deterministic retention budget if it proposes combinations of independently contractible sites; an exponential powerset is not an implicit entitlement. Cost comparison includes the materialized baseline, and no runtime fallback occurs after a route commits.

## Closes when

At least one multiply/add body under a contraction-permitting contract retains both the complete materialized baseline and an explicitly contracted target-qualified alternative; a discriminating input proves their rounding difference; perturbing site identity, overlap, target capability, lowering, and emitted ternary form each causes a named failure; and every affected canonical identity and pin is accounted for without moving unrelated existing bytes.

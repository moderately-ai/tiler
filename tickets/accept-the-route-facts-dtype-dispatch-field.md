---
id: accept-the-route-facts-dtype-dispatch-field
title: Accept the RouteFacts dtype-dispatch field
status: done
priority: p2
dependencies: []
related: [declare-host-dtype-dispatchability-at-the-consumer-boundary]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, frontend, dtype]
---
## What is being accepted

One new public field on the macro-emitted `RouteFacts` record (`crates/tiler/src/route.rs`):

```rust
pub dtype_dispatch: &'static [(ArithmeticType, DTypeDispatch)],
```

filled by the expansion from `BoundMetalCompileDeclaration::dtype_dispatchability_rows()` (an additive accessor over a field the declaration already held), consumed by `execution_environment`, which refuses a repeated arithmetic type as `MalformedRouteFacts`. Omission is the emitted answer for a dtype the profile resolves `Unknown` or `Deferred`, and a host that states nothing about a dtype refuses it — silence stays fail-closed.

## The choices worth objecting to

- **A slice of pairs rather than a map**, because a `&'static` literal is what an expansion can emit; the map is built (and duplicates refused) at the one consuming site.
- **Only exact declarations are emitted.** `Unknown`/`Deferred` rows are absent rather than carried, so the absence is load-bearing and documented at the field.
- **The inline-region path is recorded as structurally unable to earn a host row** — the environment exists before any adapter, so no device is reachable; the contract states the only place a host-earned row can arise (`RuntimeAdapter::bind_execution_context`).

## Evidence

The implementing ticket's Outcome: the end-to-end watched refusal fixture, four perturbations each with its failure population, and the hardware-identical Candle run.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. The implementing worker reported the field as a labelled draft with a node; the label was not in the landed code and this node was filed by the coordinator at integration.

## Accepted — 2026-08-06

**Tom accepted the surface as stated**, at the live session's decision round via the coordinator's presentation (AskUserQuestion, option "Accept"), witnessed first-hand by the coordinator; no exclusion was named. What was accepted is the field, the slice-of-pairs shape, the omission-encodes-non-exact convention, and the duplicate refusal at the consuming site. Acceptance is not stabilization: accepted pre-alpha vocabulary, not a published API with compatibility obligations.

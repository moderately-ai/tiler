---
id: check-the-stated-contract-on-the-semantic-fallback-path
title: Check a region's stated numerical contract on the semantic fallback path
status: deferred
priority: p3
dependencies: []
related: [state-the-numerical-contract-in-the-region-grammar, decide-the-inline-frontend-numerical-contract]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [frontend, numerics, deferred]
---
## Deferred: the activation trigger has not fired

This ticket is filed `deferred` rather than `todo` because there is nothing to check yet. Do not claim it until a trigger below has fired.

## The gap

**Fact.** Since `state-the-numerical-contract-in-the-region-grammar`, every `tensor!` region must state a numerical contract. Only a region whose `deliver` statement selects an artifact family has that contract *acted on*: `crate::aot::deliver` passes it to `tiler_compiler::session::compile`, and a contract the target cannot honour is a hard refusal. A `fallback-only` region's contract reaches nothing — `crate::expand` resolves it and then takes the branch that never calls `deliver`.

**Fact — why that is currently harmless.** The facade's fallback path constructs the region's *declared result* and does not evaluate the expression; `spikes/runtime/inline-dispatch/src/main.rs` records the same thing about its own fallback region. A contract is a statement about which results arithmetic may return, and the fallback performs no arithmetic, so there is no behaviour for the stated contract to constrain and nothing it could contradict.

**Inference.** The statement is therefore inert rather than wrong on that path, and making it *look* checked before there is anything to check would be the worse outcome: a test asserting the fallback honours a contract would be asserting a vacuous truth, and a reader would take it as evidence the path is governed.

## Activation triggers

Any one of these fires it:

- The semantic fallback begins evaluating a region's expression rather than only constructing its declared result.
- A reference or CPU execution path is admitted that a `fallback-only` region routes to.
- A region's stated contract becomes an input to the emitted `RegionFacts`, artifact identity, or route facts on a non-delivering path.

## What the work would be

Decide whether the fallback must refuse a contract it cannot honour, honour it, or state explicitly that it is unconstrained — and record which, because all three are defensible and only one can be true. Whatever is chosen, the evidence is a test that can fail: a fallback path that ignores a stated contract must be shown ignoring it, not assumed to.

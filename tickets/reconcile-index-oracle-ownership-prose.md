---
id: reconcile-index-oracle-ownership-prose
title: Reconcile index-oracle ownership prose and authority construction
status: todo
priority: p2
dependencies: []
related: []
scopes: [contracts/numerics, implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, implementation, ergonomics]
---
Two residuals the oracle implementation could not close inside its own scopes.

First, `docs/correctness-and-testing.md` still says the generic slow evaluator "remains owned by" the now-complete `prototype-index-region-reference-oracle` ticket. Restate it as implemented, naming `tiler_reference::IndexRegionEvaluator` and the independence property that matters: the oracle shares no arithmetic implementation with the structural verifier it checks, so one shared defect cannot make both agree on an incorrect coordinate.

Second, `IndexRegionAuthority` still takes both the scalar and semantic
registries even though `FrozenScalarRegistry::semantic_authority()` now exposes
the exact semantic registry it was frozen against. That accessor removes the
ticket's former public-API blocker. Make it impossible for a caller to pair a
scalar authority with a different semantic authority by deriving the latter
inside `IndexRegionAuthority`.

## User-visible outcome

The documentation must describe the checked index-region evaluator as present,
and a caller must be able to select its scalar authority without separately
supplying a semantic authority that could disagree. Preserve the evaluator's
independence from the structural verifier; the refactor is not permission to
share their arithmetic implementation.

## Closes when

The stale ownership prose names the implemented evaluator and its independence
property, `IndexRegionAuthority` derives its semantic authority from the scalar
registry, redundant caller arguments are removed, and the full gate passes.

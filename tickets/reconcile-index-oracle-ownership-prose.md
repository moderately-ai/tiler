---
id: reconcile-index-oracle-ownership-prose
title: Reconcile index-oracle ownership prose and authority construction
status: done
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

## Outcome (2026-07-27)

**The prose was stale in a way that understated the tree.** `docs/correctness-and-testing.md` said a generic evaluator for checked `IndexRegion` values "remains owned by `prototype-index-region-reference-oracle`", and that "until that ticket passes, the compiler's graph-specific proof is not evidence that arbitrary registered lowering agrees with semantic meaning". That ticket is `done`, and `tiler_reference::oracle` implements the evaluator. The paragraph now names it, and states its independence from the structural verifier as the property that makes agreement between them mean anything — a shared arithmetic implementation would make it vacuous, which is the same failure this section already names one layer down for a shared evaluator/lowering bug.

**`IndexRegionAuthority` now takes one authority, not two.** A `FrozenScalarRegistry` is frozen *against* a `FrozenSemanticRegistry` and exposes it as `semantic_authority()`, so accepting both let a caller name a semantic authority the scalar authority was never registered under — two authorities governing one evaluation, with nothing comparing them. The semantic half is derived rather than accepted, which removes the disagreement instead of checking for it.

**Fact: every caller was supplying a redundant second authority, and this is what proves the risk was real rather than theoretical.** Removing the parameter left `semantic_authority()` in `tiler-reference`'s oracle test suite entirely unused, along with eight `let semantic = semantic_authority();` bindings and call sites in `tiler-compiler`'s `legality.rs` and `governed.rs`. Each was constructing a *second* `FrozenSemanticRegistry::standard()` that happened to agree with the one inside the scalar registry. Nothing checked that they agreed; they agreed because every caller reached for the same standard profile. A caller that had not would have evaluated under two authorities and been told nothing.

The evaluator's independence from the structural verifier is preserved: nothing here shares arithmetic between them, and the change is to which authority an evaluation names rather than to how it computes.

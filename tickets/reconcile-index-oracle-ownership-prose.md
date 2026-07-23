---
id: reconcile-index-oracle-ownership-prose
title: Reconcile index-oracle ownership prose and registry accessor
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

Second, `IndexRegionAuthority` currently takes both the scalar and semantic registries because `tiler_ir::index::FrozenScalarRegistry` exposes no accessor for the semantic registry it was frozen against; the oracle verifies the pairing through `ScalarAuthorityEvidence::semantic_snapshot` instead. Adding that accessor to `tiler-ir` would let the authority take one registry and derive the other, removing a redundant parameter and a class of caller mismatch. Treat the accessor as a public-boundary change on `tiler-ir`: present it before hardening, and leave the current two-registry form in place if the accessor is not accepted.

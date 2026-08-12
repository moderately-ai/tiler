---
id: define-the-composed-realization-driver-subject-bridge
title: Define the composed realization driver's subject bridge
status: awaiting-decision
priority: p2
dependencies: [retain-the-selected-semantic-candidate-for-the-conformance-oracle, decide-the-safe-cross-crate-composed-reference-boundary]
related: [retain-each-plan-alternative-s-verified-semantic-candidate, implement-the-composed-realization-evaluation-driver, accept-the-composed-realization-evaluation-surface, accept-the-realization-witness-surface]
scopes: [implementation/compiler, implementation/conformance, implementation/ir, implementation/reference, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, conformance, reference, correctness]
---
## User-visible outcome

The already-accepted composed conformance driver receives one compiler-minted, inseparable subject for the semantic candidate and its ordered physical realization, rather than asking callers to assemble a program and witness sequence that could come from different alternatives.

## Fixed constraints — accepted 2026-08-12

- The driver lives in `tiler-conformance`, the top evidence layer that already depends on compiler and reference.
- Its public entry accepts a complete `PlanAlternative` plus declared inputs, not a free `(SemanticProgram, witnesses)` pair.
- The exact retained candidate stays mandatory and private inside `ProgramAlternative` until this driver and bridge land atomically.
- `tiler-reference` never names a scheduled plan, and its `ValueId` pin/observe primitive remains crate-private.
- No caller may inject a tensor produced by the implementation under test, reconstruct a missing candidate, use the baseline, or substitute another alternative's witness.
- No artifact/schema/identity change is authorized.

## Readiness audit — repaired 2026-08-12 at `1ff5e90b`

The compiler projection is not the first undecided boundary after all. The source-first audit found two accepted sentences that cannot both be implemented in Rust: a driver in sibling crate `tiler-conformance` cannot call a genuinely crate-private item in `tiler-reference`, because Rust has no friend-crate visibility. The original acceptance record allowed the pin/observe item to be either crate-private **or** `#[doc(hidden)] pub`; the retention decision later narrowed that to crate-private without rechecking the external driver home. A hidden public raw-pinning item would recreate the exact device-tensor injection hole the sole-entry decision exists to close, so widening visibility is not a mechanical repair.

The same audit verified a second missing boundary. `ReferenceNumericalConformance::from_realization` deliberately refuses `ReassociationPermitted`, while the composed driver exists precisely to evaluate the one reassociation a retained physical witness pins. Using `ReferenceNumericalConformance::new` would silently erase the arithmetic subject, and using the strict reading would answer a different contract. The reference side therefore needs a safe witness-discharged composition operation, not merely visibility on its current raw evaluator state.

Current `tiler-conformance` is also entirely `#[cfg(test)]`, exports no public item, is `publish = false`, and its accepted architecture says nothing may depend on it. The current in-tree consumers are its own serial-sum tests; no named non-test consumer justifies promoting a first reusable API now. Whether the accepted public driver remains reserved or lands immediately is consequently part of the prerequisite decision, not a fact this bridge may inherit.

[`decide-the-safe-cross-crate-composed-reference-boundary`](decide-the-safe-cross-crate-composed-reference-boundary.md) now owns those corrections. This ticket remains the narrower compiler-subject decision and is blocked on that answer. Its later audit must also distinguish the cover's canonical region/edge order from `CoverAssembly::from_plan`'s producer-before-consumer execution order.

## Decision still required

Read the accepted composed-evaluation and realization-witness records, then derive the smallest public compiler projection that lets the external top-layer driver consume:

1. the retained candidate `P'`;
2. the verified ordered stage cover and materialization edges;
3. each stage's existing `RealizationWitness`; and
4. the semantic `ValueId` bindings needed to pin and observe intermediate values.

Compare at least: a single opaque `ComposedRealizationSubject<'a>` borrowed from `PlanAlternative`; narrowly documented borrowed accessors on `PlanAlternative`; and a compiler-owned visitor/projection that prevents free recombination. Reject any shape whose parts can be mixed across alternatives without revalidation. State the driver signature, lifetimes, ownership, refusal boundary, and which items are public versus crate-private.

## Required Fact audit

Verify rather than inherit: `PlanAlternative`'s owner link and existing public evidence; the private `SelectedPlan`/cover/materialization population; `RealizationWitness::of`; the accepted plain-scalar redirection; `tiler-conformance`'s current test-only/no-public-surface contract; and the compiler/reference/conformance Cargo edges.

## Closes when

Tom has accepted the exact bridge and driver signature, every free-composition/cross-alternative substitution route is either structurally impossible or explicitly revalidated, the conformance crate's first public non-test surface is stated honestly, and the implementation ticket is updated with the resulting exact dependency.

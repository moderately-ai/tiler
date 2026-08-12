---
id: define-the-composed-realization-driver-subject-bridge
title: Define the composed realization driver's subject bridge
status: awaiting-decision
priority: p2
dependencies: [retain-the-selected-semantic-candidate-for-the-conformance-oracle]
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

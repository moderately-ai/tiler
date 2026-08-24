---
id: define-the-conformance-obligation-and-evidence-requirement-algebra
title: Define the conformance obligation and evidence-requirement algebra
status: in-progress
priority: p1
dependencies: [inventory-the-closed-world-conformance-claim-universe-by-owner]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification, contracts/navigation]
paths: []
tags: [research, design, conformance-progress, verification]
claimed_from: todo
assignee: codex
lease_expires_at: 1787609714
---
# Define the conformance obligation and evidence-requirement algebra

## Goal

A bounded schema and decision packet that represents family-specific obligations and their required evidence without treating semantic, reference, compiled, executed, measured, exhaustive, proof, and normative-authority evidence as one false total maturity order.

## Work

1. Re-read the accepted evidence classes and every existing conformance/result vocabulary before choosing fields.
2. Separate raw **observation**, evaluated **obligation verdict**, evidence kind, evidence authority, exact scope/context, and freshness.
3. Represent positive obligations, expected typed refusals, invariants, availability requirements, independent-oracle comparisons, proofs, bounded exhaustive evidence, and cost claims without overloading `failed` or `passed`.
4. Define requirement composition only where it is necessary: conjunction, acceptable alternatives, exact authority, exact context, minimum freshness, and explicit `not applicable` authority. Refuse a universal numeric rank unless one is proven for a named family.
5. Show how a yellow-to-red transition can add knowledge, how a required refusal becomes a satisfied obligation, and how historical evidence remains retained after it becomes too stale for current qualification.
6. Compare an ordered maturity enum, an unordered evidence set, a typed requirement predicate, family-specific schemas, and deferral. Eliminate shapes that silently coerce incomparable evidence.
7. Produce worked cells for semantic/reference, optimizer preservation, compile-only availability, real execution, normative ownership, and performance measurement.
8. Demonstrate negative controls for wrong authority, wrong context, stale receipt, missing evidence kind, and a falsely ordered substitute.

## Non-goals

- Do not let a run stamp implementation maturity or normative support.
- Do not create one scalar completion authority.
- Do not define the first goal profile or public Rust API.

## Stop conditions

Stop and split the disputed family when one schema cannot represent its authority without defaults, lossy coercion, or a new public contract.

## Acceptance

- Every worked obligation derives its color from observation, verdict, and an exact evidence requirement.
- Negative and unavailable outcomes remain distinguishable from defects and satisfied refusal obligations.
- Incomparable evidence is never ordered by convenience.
- The design states identity/versioning consequences and supplies downstream schema requirements.

## Refs

- [`derive-the-conformance-evidence-ledger-cells-from-executed-runs`](derive-the-conformance-evidence-ledger-cells-from-executed-runs.md)
- [`docs/correctness-and-testing.md`](../docs/correctness-and-testing.md)
- [`inventory-the-closed-world-conformance-claim-universe-by-owner`](inventory-the-closed-world-conformance-claim-universe-by-owner.md)

## Outcome

**Research complete at `6b6787f0f26b9775769e3cee9e1c5779c9eb431e`; schema remains private and unimplemented.** [The retained design packet](../docs/research/verification/conformance-obligation-evidence-algebra.md) selects family-owned obligation declarations compiled into a small canonical `Atom`/`All`/`Any` predicate algebra over immutable evidence atoms. It rejects a total maturity order, scalar authority, wildcard context, generic negation, and a universal family schema.

### Fact audit

- **Verified:** accepted documentation evidence classes are categories, not a total strength order.
- **Verified:** accuracy, index proof, feasibility, measurement, applicability, and run-result vocabularies answer different owner questions; their differences are not aliases to reconcile away.
- **Verified:** `Measured::{Ran, Unavailable, Failed}` establishes the raw machine-outcome split needed by the design, while the deferred ledger ticket establishes that a run cannot mint maturity or normative authority.
- **Verified:** the owner-universe report keeps subjects separate from tests and receipts and retains unknown populations as positive blockers.

### Selected design

The common layer separates subject, case, obligation, observation, evidence atom, evaluated verdict, and derived color. Verdicts are `Passed`, `Failed`, `NotObserved`, and authority-backed `NotApplicable`; green/red/yellow/gray are derived only after receipt, context, freshness, applicability, and authority validation. Evidence kinds remain an unordered tagged set, and family-owned predicates state exact conjunctions or acceptable alternatives. A correct expected refusal passes; unavailable current evidence stays yellow; reached-stage failure is red; stale evidence remains historical but cannot qualify the current profile.

Six worked cells cover semantic/reference agreement, optimizer preservation, compile-only availability, real execution, normative ownership, and bounded performance measurement. Eight negative controls cover wrong authority, wrong context, stale evidence, missing evidence kind, false ordering, wrong refusal, receipt tampering, and denominator omission. Identity consequences and downstream requirements are explicit.

### Boundary

No Rust API, public boundary, first goal profile, receipt serialization, owner visibility mechanism, authority provider, or family obligation manifest is decided here. A family that cannot map without defaults, lossy coercion, opaque callbacks, or a new public contract must split and stop rather than widening the common schema by convenience.

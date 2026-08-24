---
id: define-the-conformance-obligation-and-evidence-requirement-algebra
title: Define the conformance obligation and evidence-requirement algebra
status: in-progress
priority: p1
dependencies: [inventory-the-closed-world-conformance-claim-universe-by-owner]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
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

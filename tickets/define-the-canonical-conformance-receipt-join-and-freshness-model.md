---
id: define-the-canonical-conformance-receipt-join-and-freshness-model
title: Define the canonical conformance receipt join and freshness model
status: todo
priority: p1
dependencies: [define-the-conformance-obligation-and-evidence-requirement-algebra, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles, decide-how-owner-private-conformance-inventories-cross-crate-boundaries, specify-the-canonical-owner-conformance-manifest-protocol]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, design, conformance-progress, verification]
---
# Define the canonical conformance receipt join and freshness model

## Goal

A bounded cross-layer receipt design that joins existing owner-minted identities without copying their fields or granting conformance authority to reconstruct them, and that separates retained historical evidence from evidence sufficient for current qualification.

## Work

1. Read in full the construction, verification, decoding, consumption, refusal, identity, and tamper tests for `PreparedCompilation`, `ArtifactProvenance`, `CompiledArtifact`, `VerifiedArtifactProgram`, `DeliveredRealizationRecord`, `PayloadPlanDeterminismReceipt`, `ProofSidecarBuilder`, `Compilation`, `LiveExecutionContext`, `MeasurementBoundary`, and terminal runtime completion.
2. Derive the minimum join subject for one execution: source revision; system universe; goal profile; feature/obligation/case; semantic and reference authorities; selected plan; schedule/KIR/program; proof sidecar; artifact and compilation provenance; runtime route; terminal completion; environment; and exact comparison.
3. For each field decide whether the receipt references an accepted identity, carries an owner-produced value, or must not be present. Refuse copied projections whose owner can drift independently.
4. Model compilation, eligibility, preparation, execution, completion, comparison, and publication as separate stages with typed observations.
5. Define freshness and comparison: current qualification, retained historical best, expired/unavailable evidence, changed environment, and source/profile changes. A stale receipt remains historical evidence and cannot qualify a different subject.
6. Compare a monolithic receipt, a typed envelope of owner receipts, an identity-only join, event sequence, and deferral on missing authorities.
7. Specify subject perturbations for wrong oracle, wrong plan, wrong program/artifact, stale environment, missing terminal completion, and cross-profile reuse, retaining exact expected refusals.
8. Bound canonical size, runtime, retention, and migration cost.

## Non-goals

- Do not replace existing receipts or mint owner identities in conformance.
- Do not implement the serial-sum pilot or expose a public schema.
- Do not infer device execution from compilation or submission.

## Stop conditions

Stop and split the missing owner when any join field would need a guessed identity, duplicated schema, or unstated freshness policy.

## Acceptance

- Every receipt component has one authoritative producer and verifier.
- Historical retention and current qualification are independently representable.
- Stage loss and every identity substitution fail closed in the design.
- The packet identifies the smallest safe implementation slice and all public-boundary dependencies.

## Refs

- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)
- [`define-the-conformance-obligation-and-evidence-requirement-algebra`](define-the-conformance-obligation-and-evidence-requirement-algebra.md)
- [`decide-how-owner-private-conformance-inventories-cross-crate-boundaries`](decide-how-owner-private-conformance-inventories-cross-crate-boundaries.md)
- [`specify-the-canonical-owner-conformance-manifest-protocol`](specify-the-canonical-owner-conformance-manifest-protocol.md)

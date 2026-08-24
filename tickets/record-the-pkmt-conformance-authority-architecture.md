---
id: record-the-pkmt-conformance-authority-architecture
title: Record the P+K+M+T conformance authority architecture
status: in-progress
priority: p1
dependencies: [decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
related: [spike-a-red-yellow-first-full-conformance-suite, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
scopes: []
shared_scopes: [project/tickets, contracts/decisions, contracts/navigation, contracts/numerics]
paths: []
tags: [decision, architecture, conformance-progress, conformance-authority, verification]
claimed_from: todo
assignee: codex
lease_expires_at: 1787610285
---
# Record the P+K+M+T conformance authority architecture

## Goal

An accepted ADR and aligned catalogs/contracts preserving Tom's 2026-08-24 `P+K+M+T` decision, its five authority classes, fail-closed bootstrap boundary, change policy, staged dependencies, terminal trust, and reversal triggers without reopening the selected product.

## Work

1. Re-audit the decision carrier and governing ADRs at the exact carrier base.
2. Copy the carrier's accepted-decision, singular-authority, change-policy, bootstrap, and counterargument sections without semantic edits into one new ADR; allocate the next ADR identity from the live catalog.
3. Record acceptance provenance: Tom, 2026-08-24, coordination conversation, with the decision carrier as the relay source.
4. Update the decisions catalog, correctness/testing contract, and any live design/status entry point that would otherwise describe protected review or signing as optional.
5. Make explicit that `P+K+M+T` governs authoritative qualification while partial deployments remain provisional, and that no runtime/kernel fast path consumes this authority.
6. Link every selected mechanism ticket and retain its dependency/stop boundary rather than embedding an unaccepted provider or schema in the ADR.

## Non-goals

- Do not choose providers, algorithms, thresholds, key holders, classifier infrastructure, witness topology, or retention service.
- Do not implement a schema, CLI, host rule, key, signature, log, or public API.
- Do not reinterpret `P+K+M+T` as a menu of optional long-term controls.

## Stop conditions

Stop if the carrier conflicts with an accepted ADR, if a proposed sentence expands a public boundary, or if recording the decision would silently resolve a mechanism ticket's provider/schema choice. Return the exact conflict instead of weakening the accepted target.

## Acceptance

- The ADR and catalogs identify all five authority classes and all four selected properties.
- Partial/bootstrap outputs are explicitly provisional and cannot establish authoritative progress or qualification.
- Change, tombstone, lineage, unavailability, terminal-trust, and reversal semantics match the carrier.
- `tkt lint`, `make citations`, and the ticket scope guard pass.

## Refs

- [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md)
- [`cost-protected-review-versus-signed-conformance-authority`](cost-protected-review-versus-signed-conformance-authority.md)
- [`docs/correctness-and-testing.md`](../docs/correctness-and-testing.md)

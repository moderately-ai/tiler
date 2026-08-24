---
id: design-threshold-signed-five-class-conformance-authority
title: Design threshold-signed five-class conformance authority
status: todo
priority: p1
dependencies: [decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles, define-the-canonical-conformance-receipt-join-and-freshness-model]
related: [spike-a-red-yellow-first-full-conformance-suite, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, design, security, identity, conformance-progress, conformance-authority]
---
# Design threshold-signed five-class conformance authority

## Goal

A Pareto-complete `K` contract for a canonical monotone manifest binding all five authority classes, policy-approver thresholds, independent client roots/state, signing semantics, validity, rotation, compromise, and recovery.

## Work

1. Derive canonical transitive closure from concrete owner and receipt identities; immutable canonical bytes and fail-closed unknown versions are mandatory.
2. Define manifest/root versions, predecessor continuity, thresholds/roles, validity/freshness, semantic diff, client monotone state, and independent root distribution.
3. Compare minimal cryptographic/provider choices on correctness, canonicalization risk, offline verification, recovery, host cost, memory, portability, and compatibility.
4. Separate policy-approver signing from automation and define perturbations for wrong role/quorum, rollback/freeze, substitution, unknown fields, closure drift, key loss, and threshold compromise.
5. Design generation/custody, old-plus-new rotation, revocation, quorum outage, incident response, and out-of-band root recovery, naming impossible automatic recovery.
6. Produce a bounded executable spike and verbatim implementation/ceremony packet for the establishment ticket.

## Non-goals

- Do not sign only profile/exception bytes, treat automation as semantic approval, choose `P`/`M`/`T`, deploy keys, or accept a profile.
- Do not add a public API without separate acceptance.

## Stop conditions

Stop for Tom if multiple nondominated mechanism/custody placements survive. Stop as evidence-blocked if closure, canonicalization, policy-signer independence, root distribution, or fail-closed recovery is unresolved.

## Acceptance

- One design binds all five classes and one exact source/closure identity.
- Roles, root/state, offline behavior, rotation, revocation, outage, compromise, recovery, negative controls, cost, trust, and reversal evidence are complete.
- The establishment ticket receives exact schemas, algorithms/providers, ceremonies, scopes, and stop conditions.

## Refs

- [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md)
- [`define-the-canonical-conformance-receipt-join-and-freshness-model`](define-the-canonical-conformance-receipt-join-and-freshness-model.md)

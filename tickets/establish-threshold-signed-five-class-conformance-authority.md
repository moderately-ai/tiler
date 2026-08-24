---
id: establish-threshold-signed-five-class-conformance-authority
title: Establish threshold-signed five-class conformance authority
status: todo
priority: p2
dependencies: [design-threshold-signed-five-class-conformance-authority, authorize-the-pkmt-conformance-authority-mechanism-implementation]
related: [spike-a-red-yellow-first-full-conformance-suite, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [implementation, operations, security, identity, conformance-progress, conformance-authority]
---
# Establish threshold-signed five-class conformance authority

## Goal

An implemented and operated `K` authority realizing the accepted canonical manifest, policy-approver threshold, independent client root/state, and recovery design.

## Work

1. Re-audit the accepted schema/algorithm/provider/ceremony packet, scopes, root-distribution authority, and Tom's implementation/operations authorization.
2. Implement the canonical resolver, five-class manifest, semantic diff, signing workflow, verifier, independent root distribution, and client-retained monotone state.
3. Provision policy roles/threshold under the accepted custody ceremony without exposing secrets in repository output.
4. Run every canonicalization, role/quorum, rollback/freeze, substitution, unknown-field, closure-drift, and unauthorized-automation perturbation.
5. Execute below-threshold loss, rotation, revocation, quorum outage, threshold-compromise response, and out-of-band root-recovery drills.
6. Report installed identities, exact supported/unsupported states, measured host cost/memory, offline behavior, and operating/recovery runbooks.

## Non-goals

- Do not redesign the accepted mechanism, sign only profile/exception bytes, use automation as semantic approval, or make cryptography an oracle.
- Do not add `P`, `M`, or `T`, or accept a profile before the exact composition ticket is complete.
- Do not expose a public API without a separate accepted boundary.

## Stop conditions

Stop if implementation diverges from the accepted design, canonical closure/canonicalization fails, policy signers are not independent of repository automation, root distribution is unresolved, recovery cannot fail closed, or Tom has not authorized the relevant implementation/operations.

## Acceptance

- One canonical versioned manifest binds all five authority classes and one exact source/closure identity.
- Threshold roles, client root/state, rotation, revocation, outage, compromise, and recovery are explicit and subject-perturbed.
- Unauthorized automation and repository history rewrite cannot mint an accepted authority while the threshold/root remain honest.
- Costs, unsupported threats, reversal evidence, and operations authority are explicit.

## Refs

- [`design-threshold-signed-five-class-conformance-authority`](design-threshold-signed-five-class-conformance-authority.md)
- [`authorize-the-pkmt-conformance-authority-mechanism-implementation`](authorize-the-pkmt-conformance-authority-mechanism-implementation.md)
- [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md)
- [`define-the-canonical-conformance-receipt-join-and-freshness-model`](define-the-canonical-conformance-receipt-join-and-freshness-model.md)

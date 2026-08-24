---
id: establish-external-mixed-diff-conformance-attestation
title: Establish external mixed-diff conformance attestation
status: todo
priority: p2
dependencies: [design-the-external-mixed-diff-conformance-attestation, authorize-the-pkmt-conformance-authority-mechanism-implementation, bind-protected-review-and-signed-conformance-authority]
related: [spike-a-red-yellow-first-full-conformance-suite, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [implementation, operations, security, conformance-progress, conformance-authority]
---
# Establish external mixed-diff conformance attestation

## Goal

An installed externally governed `M` classifier and non-replayable attestation implementing the accepted taxonomy and exact-diff binding in both `P` and `K` acceptance.

## Work

1. Re-audit the accepted taxonomy/service/attestation packet, external authority, exact scopes, and Tom's operations authorization.
2. Deploy the external classifier/configuration and require its exact-diff attestation in both `P` merge and `K` signing/client paths.
3. Run every class-pair, single-class, unclassified, stale/replayed, latest-push, outage, rotation, and taxonomy-update perturbation.
4. Demonstrate configuration independence from ordinary repository writes and retain exact matched paths/classes and provider failures.
5. Execute outage, credential/configuration rotation, compromise, and recovery drills; report measured latency/cost and unsupported threats.

## Non-goals

- Do not redesign the accepted taxonomy/mechanism or implement it as a repository-local script trusted against repository rewrite.
- Do not claim protection against a dishonest `P+K` authority using separate work items.
- Do not choose profile content, oracle semantics, or evidence sufficiency.

## Stop conditions

Stop if the taxonomy cannot be complete, if configuration/enforcement is writable by the protected repository actor, if exact diff/source binding is unavailable, or before external deployment without Tom's operations authorization.

## Acceptance

- Every relevant path has exactly one authority or implementation/evidence disposition, with fail-closed unknown handling.
- All class-pair negative controls reject and print both matched classes; single-class controls proceed to the proper authority.
- One non-replayable attestation binds taxonomy, predecessor, successor, and the selected `P+K` identity.
- Outage, rotation, recovery, cost, terminal trust, and unsupported threats are explicit.

## Refs

- [`design-the-external-mixed-diff-conformance-attestation`](design-the-external-mixed-diff-conformance-attestation.md)
- [`authorize-the-pkmt-conformance-authority-mechanism-implementation`](authorize-the-pkmt-conformance-authority-mechanism-implementation.md)
- [`bind-protected-review-and-signed-conformance-authority`](bind-protected-review-and-signed-conformance-authority.md)
- [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md)

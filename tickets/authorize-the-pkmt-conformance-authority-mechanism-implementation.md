---
id: authorize-the-pkmt-conformance-authority-mechanism-implementation
title: Authorize the P+K+M+T conformance authority mechanism implementation
status: awaiting-decision
priority: p1
dependencies: [record-the-pkmt-conformance-authority-architecture, design-protected-review-authority-for-conformance-policy, design-threshold-signed-five-class-conformance-authority, design-the-exact-source-pk-conformance-authority-composition, design-the-external-mixed-diff-conformance-attestation, design-witnessed-conformance-authority-history-and-recovery]
related: [spike-a-red-yellow-first-full-conformance-suite, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [decision, implementation, operations, security, conformance-progress, conformance-authority]
---
# Authorize the P+K+M+T conformance authority mechanism implementation

## Decision requested

After the five selected mechanism designs and the accepted ADR are complete, Tom decides whether their exact repository, host, key, classifier, log, witness, monitor, storage, mirror, credential, and recovery changes may move from research/design into implementation and external operations.

## Packet required before decision

1. Re-audit every design result at one exact base and show the complete implementation dependency graph and non-overlapping authorities.
2. Enumerate exact providers, schemas, algorithms, roles/thresholds, paths, services, credentials, external mutations, storage/retention, costs, outage behavior, recovery drills, public boundaries, and secrets-handling consequences.
3. Show every selected negative control can reach its subject and name the exact expected failure; preserve any unexecuted external control as an explicit prerequisite.
4. Confirm `P+K+M+T` remains the selected architecture, every partial deployment stays provisional, and no implementation ticket can mint an accepted profile or authoritative qualification by itself.
5. Present any remaining nondominated mechanism choice one at a time with strongest counterargument and reversal evidence. If one design dominates, recommend it without manufacturing a choice.

## Non-goals

- Do not reopen the already accepted `P+K+M+T` target merely because implementation is expensive.
- Do not authorize profile contents, qualification, a public API, secrets disclosure, or an unspecified external mutation.
- Do not close this ticket on a worker/coordinator summary; Tom alone decides movement into implementation/operations.

## Stop conditions

Remain `awaiting-decision` while any design, authority, provider, scope, negative control, recovery path, or external mutation is implicit. Split new research rather than treating an unknown as implementation detail.

## Acceptance

- Tom's answer and provenance name the exact accepted design revisions and external authority boundaries.
- Every implementation/operations child has exact scopes, dependencies, stop conditions, secrets discipline, and recovery evidence before dispatch.
- A decline or deferral leaves authority unavailable and qualification nonzero; it does not downgrade the long-term architecture silently.

## Refs

- [`record-the-pkmt-conformance-authority-architecture`](record-the-pkmt-conformance-authority-architecture.md)
- [`design-protected-review-authority-for-conformance-policy`](design-protected-review-authority-for-conformance-policy.md)
- [`design-threshold-signed-five-class-conformance-authority`](design-threshold-signed-five-class-conformance-authority.md)
- [`design-the-exact-source-pk-conformance-authority-composition`](design-the-exact-source-pk-conformance-authority-composition.md)
- [`design-the-external-mixed-diff-conformance-attestation`](design-the-external-mixed-diff-conformance-attestation.md)
- [`design-witnessed-conformance-authority-history-and-recovery`](design-witnessed-conformance-authority-history-and-recovery.md)

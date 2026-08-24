---
id: bind-protected-review-and-signed-conformance-authority
title: Bind protected review and signed conformance authority
status: todo
priority: p2
dependencies: [design-the-exact-source-pk-conformance-authority-composition, authorize-the-pkmt-conformance-authority-mechanism-implementation, establish-protected-review-authority-for-conformance-policy, establish-threshold-signed-five-class-conformance-authority]
related: [spike-a-red-yellow-first-full-conformance-suite, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [implementation, operations, security, identity, conformance-progress, conformance-authority]
---
# Bind protected review and signed conformance authority

## Goal

An implemented exact-source `P+K` verifier realizing the accepted composition state machine over the established `P` and `K` authorities.

## Work

1. Re-audit the composition design and the exact installed `P` and `K` identities.
2. Implement the state machine requiring both approvals over one canonical closure and expose the typed composed-authority product to later consumers.
3. Run mismatched identity, stale approval/signature, A3-only, A6-only, coalition, either-outage, rotation, revocation, and survivor-assisted recovery perturbations.
4. Verify role/credential/administrator independence and report any correlation without overstating defense in depth.
5. Retain exact receipts, measured host cost/memory, offline behavior, and recovery evidence.

## Non-goals

- Do not redesign the accepted state machine, let either `P` or `K` substitute for the other, or add `M`/`T` implicitly.
- Do not select profile contents, evidence requirements, or semantic oracles.
- Do not call a repository commit hash alone proof of host approval.

## Stop conditions

Stop if no independently checkable latest-state host approval exists, if the two authorities cannot name one exact canonical closure, or if composition requires defaulting freshness, role, root, or predecessor state.

## Acceptance

- Both approvals are mandatory over one exact identity and either missing/mismatched authority fails closed.
- A3-only and A6-only perturbations demonstrate the surviving authority; the coalition limit is explicit.
- Outage, rotation, revocation, and recovery preserve the other authority without silently degrading to it.
- The product exposes one typed authority result for later `M`, `T`, and qualifier work.

## Refs

- [`design-the-exact-source-pk-conformance-authority-composition`](design-the-exact-source-pk-conformance-authority-composition.md)
- [`authorize-the-pkmt-conformance-authority-mechanism-implementation`](authorize-the-pkmt-conformance-authority-mechanism-implementation.md)
- [`establish-protected-review-authority-for-conformance-policy`](establish-protected-review-authority-for-conformance-policy.md)
- [`establish-threshold-signed-five-class-conformance-authority`](establish-threshold-signed-five-class-conformance-authority.md)

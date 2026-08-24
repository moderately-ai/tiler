---
id: design-the-exact-source-pk-conformance-authority-composition
title: Design the exact-source P+K conformance authority composition
status: todo
priority: p1
dependencies: [design-protected-review-authority-for-conformance-policy, design-threshold-signed-five-class-conformance-authority]
related: [spike-a-red-yellow-first-full-conformance-suite, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, design, security, identity, conformance-progress, conformance-authority]
---
# Design the exact-source P+K conformance authority composition

## Goal

An exact `P+K` state machine requiring fresh protected-host approval and policy-threshold signing over the same canonical five-class source/closure identity, with independently testable compromise, outage, rotation, and recovery paths.

## Work

1. Re-audit the `P` approval condition and `K` manifest/root; reject two unbound successful workflows.
2. Define the singular identity, event ordering, freshness, retained state, and acceptance transitions linking latest host approval to threshold signing.
3. State role, credential, administrator, and recovery independence required for defense-in-depth credit, including correlated-authority reporting.
4. Specify negative controls for approval A/signature B, stale approval/signature, A3-only and A6-only compromise, coalition compromise, either outage, rotations, and survivor-assisted recovery.
5. Define one typed composed-authority product for `M`, `T`, and qualifier contracts without choosing their behavior.
6. Produce the exact implementation packet for the binding ticket.

## Non-goals

- Do not deploy `P` or `K`, let one substitute for the other, add `M`/`T`, select profile content, or call a commit hash proof of approval.

## Stop conditions

Stop if latest-state host approval is not independently checkable, one canonical closure cannot be named, or role/root/freshness/predecessor state would be defaulted.

## Acceptance

- Both approvals are mandatory over one identity, with mismatch and absence failing closed.
- Independent and correlated trust, A3/A6 controls, outage, rotation, revocation, recovery, cost, and reversal evidence are complete.
- The binding ticket receives an exact state machine, schema, checks, scopes, and stop conditions.

## Refs

- [`design-protected-review-authority-for-conformance-policy`](design-protected-review-authority-for-conformance-policy.md)
- [`design-threshold-signed-five-class-conformance-authority`](design-threshold-signed-five-class-conformance-authority.md)

---
id: design-the-external-mixed-diff-conformance-attestation
title: Design the external mixed-diff conformance attestation
status: todo
priority: p1
dependencies: [design-the-exact-source-pk-conformance-authority-composition]
related: [spike-a-red-yellow-first-full-conformance-suite, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, design, security, conformance-progress, conformance-authority]
---
# Design the external mixed-diff conformance attestation

## Goal

A Pareto-complete `M` design for an externally governed exact-diff classifier that rejects any work item mixing an authority class with implementation/evidence and binds its non-replayable attestation into `P+K` acceptance.

## Work

1. Derive a complete five-authority-class versus implementation/evidence path/identity taxonomy with fail-closed unknown handling.
2. Compare external placements/formats on independence, exact predecessor/successor binding, host/signing integration, availability, rotation, auditability, cost, and recovery.
3. Define rename, move, generated-file, submodule, taxonomy-version, configuration-authority, expiry, replay, latest-push, and attestor-identity behavior.
4. Specify a perturbation for every authority-class × implementation/evidence pair, plus single-class, unclassified, stale, outage, rotation, and taxonomy-update controls.
5. State the exact semantic limit: `M` enforces work-item form and does not approve authority meaning.
6. Produce an implementation/service packet for the establishment ticket.

## Non-goals

- Do not trust a repository-local classifier against repository rewrite, deploy a service, or claim protection against separated malicious approvals.

## Stop conditions

Stop for Tom if multiple nondominated external placements survive. Stop as evidence-blocked if taxonomy closure, configuration independence, or exact diff/source binding is unavailable.

## Acceptance

- Every path has exactly one disposition and every class-pair negative control has an exact expected result.
- Attestation identity, binding, replay, outage, rotation, recovery, cost, terminal trust, unsupported threats, and reversal evidence are complete.
- The establishment ticket receives exact service/configuration scopes and stop conditions.

## Refs

- [`design-the-exact-source-pk-conformance-authority-composition`](design-the-exact-source-pk-conformance-authority-composition.md)
- [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md)

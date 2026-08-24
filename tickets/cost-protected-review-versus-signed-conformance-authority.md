---
id: cost-protected-review-versus-signed-conformance-authority
title: Cost protected review versus signed conformance authority
status: in-progress
priority: p1
dependencies: []
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, decision, conformance-progress, security]
claimed_from: todo
assignee: conformance-authority-sol
lease_expires_at: 1787601327
---
# Cost protected review versus signed conformance authority

## Goal

A Pareto-complete threat model and authority decision packet for preventing denominator, exception, verifier, and evidence-baseline manipulation from masquerading as conformance progress.

## Work

1. Name distinct actors and powers: accidental contributor error; an LLM agent with repository write access; a reviewer; a maintainer who may merge protected paths; an actor able to rewrite profile, verifier, tests, and baseline together; and compromise of any signing key or external store.
2. Enumerate the status quo, path-protected human review, split approval for profile/exception versus implementation changes, an independently signed profile/exception root, an append-only external transparency record, and deferral.
3. For each option state exactly which attacks it detects, prevents, or cannot address; operational setup; key/reviewer recovery; offline development behavior; rotation and revocation; repository and host cost; and failure mode when the authority is unavailable.
4. Treat “the repository can restrain an actor authorized to rewrite all repository checks” as an eliminated false claim.
5. Compare survivors on correctness, fail-closed strictness, maintainability, compatibility, host cost, and recovery burden. State each survivor's strongest counterargument and reversal evidence.
6. Recommend the smallest authority sufficient for the first profile, and name a trigger for adopting a stronger one.

## Non-goals

- Do not install hooks, branch protection, signing infrastructure, or CI.
- Do not claim cryptography provides review quality or protects a compromised signer.
- Do not let an availability failure silently downgrade a required authority.

## Stop conditions

Stop for Tom when every sound option requires authority outside ordinary repository writes, or when the chosen threat model changes who is permitted to approve project policy.

## Acceptance

- The threat matrix distinguishes prevention, detection, attribution, recovery, and unavoidable trust.
- Only nondominated authority options reach Tom, with costed deployment and strongest counterarguments.
- The packet states the first-profile recommendation and the evidence that would reverse it.
- Follow-up dependencies are explicit; no security work is hidden inside profile implementation.

## Refs

- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)

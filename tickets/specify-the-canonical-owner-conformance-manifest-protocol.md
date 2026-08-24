---
id: specify-the-canonical-owner-conformance-manifest-protocol
title: Specify the canonical owner conformance manifest protocol
status: todo
priority: p1
dependencies: [decide-how-owner-private-conformance-inventories-cross-crate-boundaries, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles]
related: [spike-a-red-yellow-first-full-conformance-suite, define-the-conformance-obligation-and-evidence-requirement-algebra]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, design, conformance-progress, schema, identity]
---
# Specify the canonical owner conformance manifest protocol

## Goal

A decision-ready, authority-neutral protocol for carrying owner-minted subject and obligation declarations into conformance without moving their meaning into the common layer.

## Work

1. Re-audit the owner-private boundary and obligation algebra at the exact base.
2. Define bounded canonical envelopes for owner, family, subject, obligation, tombstone, predecessor, source, reporter, and completion identities.
3. Define the exact expected-owner-family closure, required declaration-configuration matrix, duplicate/unknown rejection, cardinality accounting, canonical ordering, limits, and fail-closed versioning.
4. Keep evidence receipts, goal profiles, presentation colors, family semantics, and process orchestration outside the protocol.
5. Compare a new dependency-bottom crate, a conformance-private schema with duplicated owner encoders, and deferral; include the Metal-AOT empty-closure consequence.
6. Separate stable semantic obligation identity from checker/site witness and invocation-graph identities; do not make implementation refactoring revise obligation meaning.
7. Provide independently derived identity framing and subject perturbations for missing, extra, stale, duplicated, reordered, over-limit, wrong-owner, wrong-target/features/cfg, and paired subject-plus-declaration conditional rows.

## Non-goals

- Do not add the crate or implement serialization.
- Do not define a goal profile, support status, owner inventory, or evidence authority.
- Do not put the protocol in tiler-ir, tiler-digest, or tiler-conformance by assumption.

## Stop conditions

Stop for Tom on a new crate, dependency change, public namespace, or identity-domain decision. Stop if the protocol needs an authority default, opaque callback, source parser, build script, or unbounded allocation.

## Acceptance

- The complete schema and identity packet has no hidden authority owner.
- Exact owner-set completion makes omission an audit failure.
- Target-independent applicability or exact configuration-scoped roots prevent conditional denominator shrinkage.
- Limits and unknown-version behavior fail closed before allocation/evaluation.
- Follow-up pilots can implement the same protocol without inventing fields.

## Refs

- [Owner-private conformance inventory boundary](../docs/research/verification/owner-private-conformance-inventory-boundary.md)
- [Conformance obligation and evidence-requirement algebra](../docs/research/verification/conformance-obligation-evidence-algebra.md)

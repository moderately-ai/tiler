---
id: derive-artifact-proof-and-publication-conformance-obligations
title: Derive artifact, proof, and publication conformance obligations
status: todo
priority: p1
dependencies: [inventory-the-closed-world-conformance-claim-universe-by-owner, define-the-conformance-obligation-and-evidence-requirement-algebra]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [research/verification, project/tickets]
paths: []
tags: [research, design, conformance-progress, verification]
---
# Derive artifact, proof, and publication conformance obligations

## Goal

Produce an owner-derived, fail-loud obligation manifest for artifact program/ABI construction, canonical encoding and decoding, intrinsic verification, proof-sidecar validation, and publication guarantees. Preserve the separate authorities of artifact semantics, proof evidence, and cache/runtime consumers.

## Authority

Accepted artifact ABI and proof contracts, exact owning source, and accepted ADRs outrank this ticket. Re-audit every Fact at the implementation base.

## Work

- Read complete artifact and proof construction, encoding, decoding, validation, refusal, identity, schema, publication, and consumption paths.
- Enumerate typed owner vocabularies and correctness-bearing invariants; use explicit unknowns where a complete owner does not exist.
- Define owner-local subject identities and revision rules without inventing a public boundary.
- Design one subject perturbation per independent property and show the future manifest would reject an undisposed item.
- Relate its rows to the candidate system-universe inventory without choosing a goal profile or evidence disposition.

## Non-goals

No API implementation, schema/domain step, proof-policy choice, runtime completion model, cache protocol redesign, or goal profile.

## Stop conditions

Stop and file a decision packet if artifact and proof authorities compete, if a singular owner cannot be derived, or if enumeration requires a consequential public surface.

## Acceptance

- Every artifact/ABI/proof/publication family is exact or explicitly unknown.
- Every complete census is owner-derived and has a subject perturbation.
- Identity, schema, revision, construction, consumption, and refusal consequences are explicit.
- Unknowns have bounded descendants rather than silent zeroes.

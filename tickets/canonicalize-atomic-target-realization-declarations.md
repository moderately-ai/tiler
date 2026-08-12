---
id: canonicalize-atomic-target-realization-declarations
title: Canonicalize atomic target realization declarations
status: todo
priority: p1
dependencies: []
related: [declare-cpu-vector-realization-facts-in-the-target-profile, admit-the-first-typed-synchronization-point-and-atomic-target-authority]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, identity, canonicalization, correctness]
---
## User-visible outcome

Two target profiles declaring the same atomic realization rows in different insertion orders have one descriptor identity, while duplicate or contradictory rows refuse.

## Fact — 2026-08-11

Checked synchronization facts are sorted by their canonical `(subject, phase)` key, but the public builder's complete-descriptor path does not sort synchronization declarations before encoding. Copying that precedent into vector declarations would make insertion order identity-bearing accidentally.

## Required delivery

- Re-read every atomic realization builder, checked fact population, descriptor encoder, and duplicate/contradiction check.
- Canonically sort each repeated row family by its complete uniqueness key before both checked and complete descriptor encoding.
- Reject exact duplicates and same-key contradictory verdicts independently; never let sort order choose the winner.
- Perturb insertion order, duplicate, contradictory verdict, phase, source, and subject independently and quote the failure/output equality.

## Closes when

Atomic realization descriptor identity is order-independent and contradictions cannot coexist in any builder or decoded population.

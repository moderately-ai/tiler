---
id: decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles
title: Decide the authority and change policy for conformance universes and goal profiles
status: in-progress
priority: p1
dependencies: [inventory-the-closed-world-conformance-claim-universe-by-owner, cost-protected-review-versus-signed-conformance-authority]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, decision, conformance-progress, verification]
claimed_from: todo
assignee: codex
lease_expires_at: 1787606631
---
# Decide the authority and change policy for conformance universes and goal profiles

## Goal

A decision-ready authority and change-policy packet separating the owner-derived system universe, the human-accepted goal profile, exceptions/applicability, and the evidence snapshot so none can silently redefine another.

## Work

1. Re-audit the inventory and threat-model findings at the exact base.
2. Name the owner and minting rule for the system-universe identity, goal-profile identity, exception/applicability reasons, tombstones, profile lineage, evidence snapshot, and acceptance provenance.
3. Decide what additions, removals, replacements, required-to-optional changes, and required-to-`N/A` changes mean. Preserve denominator shrinkage as an authority event rather than evidence progress.
4. Compare the status quo, a profile-owned denominator, owner-derived universe plus profile selection, an append-only profile lineage, a signed external root, and deferral.
5. Eliminate any option that lets policy omit newly declared features silently, lets execution mint normative authority, or lets an implementation edit its own requirement and baseline unnoticed.
6. State review separation, backwards/forwards comparison rules, tombstone lifetime, profile supersession, and how an unsupported-but-correctly-refused capability is represented.
7. Present only the nondominated frontier with strongest counterarguments, reversal evidence, negative controls, and required follow-up tickets.

## Non-goals

- Do not implement a profile schema or signing mechanism.
- Do not decide which exact features the first profile includes.
- Do not turn the system universe into every source function or test.

## Stop conditions

Stop for Tom when more than one top-tier authority placement survives, or when sound enforcement requires an external authority whose ownership he has not accepted.

## Acceptance

- Universe, goal policy, applicability/exception, and evidence authorities are singular and non-overlapping.
- Every denominator-changing operation has a fail-loud identity and review consequence.
- The packet is Pareto-complete and names one recommendation or one concrete decision between frontier candidates.
- No implementation work is implied by an assumed authority.

## Refs

- [`inventory-the-closed-world-conformance-claim-universe-by-owner`](inventory-the-closed-world-conformance-claim-universe-by-owner.md)
- [`cost-protected-review-versus-signed-conformance-authority`](cost-protected-review-versus-signed-conformance-authority.md)
- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)

---
id: design-the-conformance-audit-regress-and-qualify-command-contracts
title: Design the conformance audit regress and qualify command contracts
status: todo
priority: p1
dependencies: [define-the-conformance-obligation-and-evidence-requirement-algebra, decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles, define-the-canonical-conformance-receipt-join-and-freshness-model, design-witnessed-conformance-authority-history-and-recovery]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, design, conformance-progress, verification]
---
# Design the conformance audit regress and qualify command contracts

## Goal

Exact command contracts and exit semantics for harness integrity, like-for-like evidence regression, and selected-profile qualification, with no command able to turn denominator or authority changes into implementation progress.

## Work

1. Define inputs, outputs, identities, comparison base, and exit codes for `conformance audit`, `conformance regress`, and `conformance qualify <profile>`.
2. Make `audit` require an exact discovered population, complete classification, valid identities/receipts/applicability, and nonzero census visibility while accepting an honest mixed-color report.
3. Make `regress` compare like-for-like source/profile/context evidence, retain newly observed failures honestly, distinguish profile/denominator changes, and reject unexplained loss or weakening of previously sufficient evidence.
4. Make `qualify` nonzero until every required obligation meets its evidence predicate; do not let optional or `N/A` defaults absorb missing authority.
5. Define comparison behavior for stale evidence, unavailable current environments, expanded universes, changed goal profiles, expired receipts, historical best evidence, and yellow-to-red discovery.
6. Compare a single command, three commands, subcommands over one verifier, and deferral. Eliminate any shape whose ordinary gate must stay permanently red or whose audit can pass over unclassified cells.
7. Specify machine-readable diagnostics and exact negative controls; presentation belongs to the report ticket.

## Non-goals

- Do not implement a CLI, change `Makefile`, or select the first profile.
- Do not make one scalar score authoritative.
- Do not treat every red capability as a harness failure.

## Stop conditions

Stop if an exit code cannot be derived from the accepted authority/evidence model or if comparison requires defaulting a missing identity/context.

## Acceptance

- The three commands have disjoint, exhaustive contracts and reproducible exit examples.
- Mixed red/yellow/green audit succeeds while incomplete qualification fails.
- Denominator changes and evidence changes cannot masquerade as each other.
- Every negative control names the subject perturbation and exact expected failure class.

## Refs

- [`decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles`](decide-the-authority-and-change-policy-for-conformance-universe-and-goal-profiles.md)
- [`define-the-conformance-obligation-and-evidence-requirement-algebra`](define-the-conformance-obligation-and-evidence-requirement-algebra.md)
- [`define-the-canonical-conformance-receipt-join-and-freshness-model`](define-the-canonical-conformance-receipt-join-and-freshness-model.md)

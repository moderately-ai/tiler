---
id: migrate-retained-performance-evidence-to-owner-claim-identities
title: Migrate retained performance evidence to owner claim identities
status: todo
priority: p1
dependencies: [define-the-owner-emitted-performance-claim-manifest-contract, define-the-canonical-conformance-receipt-join-and-freshness-model]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, migration, conformance-progress, verification, performance]
---

# Migrate retained performance evidence to owner claim identities

## Goal

An exact migration ledger mapping every retained performance-bearing statement to an owner claim identity and exact evidence receipt, or explicitly classifying it as non-claim evidence, provisional policy, superseded history, or unresolved authority.

## Authority

The accepted manifest contract and canonical receipt/freshness join outrank this ticket. A research document, topic, test, or benchmark success cannot mint a claim or accept its own baseline.

## Work

1. Rebuild the exact research-record and experiment populations at the working base. Audit all records carrying `bounded-measurement`, then search and read performance-bearing records in other evidence classes; do not assume the current 38-record bridge remains complete.
2. Read every candidate record, raw/derived retained evidence set, harness, accepted contract it informs, construction and consumption site, threshold/baseline source, and supersession note in full.
3. For each statement record `{record, claim owner/key/revision or unresolved, claim form, metric, subject, workload, comparator/baseline, environment/profile, procedure, raw evidence, freshness, authority, current disposition}`.
4. Map the private target cost-row population into the optimizer manifest exactly once while retaining its measurement receipt here. Keep cache, artifact, frontend/build, compiler-host, device, and model-level claims visible.
5. Separate exact correctness/identity invariants and hard feasibility bounds from performance claims; preserve their existing obligations rather than dropping them as non-performance.
6. Dual-read old prose and new declarations until every old statement maps or is explicitly rejected. Retain counts before and after; do not delete historical records.
7. Perturb every owner family with one undisposed claim and every receipt family with one changed subject, environment, procedure, threshold, and baseline identity.

## Non-goals

- Do not select first-profile requirements, invent missing owners, normalize incomparable metrics, tune a cost model, rerun benchmarks merely to make a record fresh, or retire historical evidence.
- Do not convert every number into a claim or every missing measurement into a defect.

## Stop conditions

Stop and split a decision if one statement has competing owners, a threshold or baseline lacks acceptance provenance, an environment equivalence would widen a claim, or migration needs a public schema. Leave the row unresolved and keep the goal-profile dependency blocked.

## Acceptance

- The audited record population and the migrated claim population are separately exact, counted, and reproducible.
- Every candidate statement has one explicit disposition and every accepted claim has one owner identity plus comparable receipt path.
- Subject, workload, baseline, environment, procedure, and freshness changes cannot reuse evidence silently.
- The first goal profile can consume the result without treating unknowns as absence.

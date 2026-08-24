---
id: produce-the-conformance-duplicate-equivalence-ledger
title: Produce the conformance duplicate-equivalence ledger
status: todo
priority: p2
dependencies: [derive-the-five-family-structural-conformance-manifest, spike-the-serial-sum-canonical-receipt-spine]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, audit, conformance-progress, testing]
---
# Produce the conformance duplicate-equivalence ledger

## Goal

An exact keep/share/retire ledger for apparently duplicated conformance tests and fixtures, proving equivalence by subject, obligation, layer, oracle, perturbation, construction/consumption reach, and architectural claim before any retirement ticket is allowed.

## Work

1. Re-read every proposed old/new test and its construction, consumption, refusal, oracle, identity, dependency-closure, public-boundary, and process-boundary sites in full.
2. Record each row as `{old case, proposed CaseId, subject, obligation, layer, oracle, positive/negative/unavailable/refusal population, perturbation, construction reach, consumption reach, architectural claim, disposition}`.
3. Apply “shadow, prove, then prune”: stable mapping, identical subject/obligation, preserved populations and perturbations, dual-run equivalence, at least equal reach, preserved architectural claims, and visible before/after counts.
4. Treat shared input as insufficient evidence of duplication. Keep semantic encoding versus reference mathematics, selection versus KIR identity versus execution, runtime dependency-closure tests, and independent process/file claims distinct unless all fields match.
5. Include structural fixtures, compiler/schedule repetitions, BF16 selected-plan assertions, serial-sum/contraction sidecars, applicability/preflight/dispatch, retained records, Candle integration, and the producer/consumer pair.
6. For every retire candidate, identify the replacement evidence dependency and file a narrow later retirement ticket; unresolved rows remain keep/share.
7. Demonstrate a ledger census that fails when a row or unique perturbation is removed.

## Non-goals

- Do not delete, move, or weaken any test.
- Do not call tests duplicates from names, source similarity, or shared inputs.
- Do not collapse independent oracles into shared fixture logic.

## Stop conditions

Stop and mark `keep` when equivalence cannot be proven or when removal could erase dependency closure, public boundary, independent process, or other architectural evidence.

## Acceptance

- Every candidate has a complete row and evidence-backed keep/share/retire disposition.
- Every retire row depends on landed replacement evidence and preserves exact population/perturbation reach.
- The ledger fails loud on row or perturbation deletion.
- No retirement ticket is filed for an unresolved equivalence.

## Refs

- [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md)
- [`derive-the-five-family-structural-conformance-manifest`](derive-the-five-family-structural-conformance-manifest.md)
- [`spike-the-serial-sum-canonical-receipt-spine`](spike-the-serial-sum-canonical-receipt-spine.md)

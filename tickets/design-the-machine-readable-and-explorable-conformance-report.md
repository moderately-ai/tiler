---
id: design-the-machine-readable-and-explorable-conformance-report
title: Design the machine-readable and explorable conformance report
status: todo
priority: p2
dependencies: [derive-the-optimizer-and-planner-capability-obligation-manifest, derive-the-five-family-structural-conformance-manifest, design-the-conformance-audit-regress-and-qualify-command-contracts, assemble-the-first-versioned-conformance-goal-profile]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, design, conformance-progress, reporting]
---
# Design the machine-readable and explorable conformance report

## Goal

A report design generated from one authoritative machine representation and projected across feature, layer, operation, dtype, shape, target, numerical contract, optimizer strategy, evidence kind, and dependency/critical-path views without turning a visualization or scalar percentage into authority.

## Work

1. Derive the report input from the accepted universe, profile, obligation, receipt, and command contracts; do not define a parallel schema for presentation.
2. Define an exact machine-readable report with schema identity, source/profile/evidence identities, population counts, per-cell observations/verdicts/evidence, unavailable/stale reasons, and denominator/profile diffs.
3. Design compact human views: owner/layer tree, feature and obligation matrix, dependency blockers, evidence-history timeline, and critical-path summary. Use a flame-style view only if its area/ordering has a precise non-misleading meaning.
4. Keep every view as a projection over the same claim graph; operation, dtype, shape, target, optimizer, and evidence are orthogonal facets rather than parent/child products.
5. Show exact expected, discovered, classified, observed, satisfied, contradicted, unavailable, stale, and not-applicable populations. Print zeros.
6. Permit derived percentages only as labelled presentation over exact counts; never use one as qualification authority.
7. Compare static text/JSON, static HTML, interactive HTML, and deferred visualization on host cost, reproducibility, reviewability, and artifact retention.
8. Specify negative controls for omitted cells, duplicated cells, denominator shrinkage, mismatched profile, and a view disagreeing with machine data.

## Non-goals

- Do not implement or publish the report.
- Do not create independent status storage or hand-authored colors.
- Do not hide red/yellow cells to improve readability.

## Stop conditions

Stop if a view requires information not present in the accepted machine model or if its visual encoding implies a false ordering or scalar authority.

## Acceptance

- One machine schema supplies every human view.
- Exact counts and identities remain inspectable beneath all aggregations.
- The report exposes submerged internal features and their blockers as directly as user-visible operations.
- The design selects a nondominated first renderer with bounded cost and explicit accessibility/reproducibility constraints.

## Refs

- [`design-the-conformance-audit-regress-and-qualify-command-contracts`](design-the-conformance-audit-regress-and-qualify-command-contracts.md)
- [`assemble-the-first-versioned-conformance-goal-profile`](assemble-the-first-versioned-conformance-goal-profile.md)
- [`derive-the-optimizer-and-planner-capability-obligation-manifest`](derive-the-optimizer-and-planner-capability-obligation-manifest.md)

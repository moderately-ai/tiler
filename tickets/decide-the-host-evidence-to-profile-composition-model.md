---
id: decide-the-host-evidence-to-profile-composition-model
title: Decide the host evidence to profile composition model
status: todo
priority: p2
dependencies: []
related: [declare-metal-subgroup-realization-facts-in-the-target-profile]
scopes: [research/target-profiles, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

Tiler has one decided model for how measured evidence scoped to a single host composes into the profile identities consumers dispatch on — so profile keys encode claim scope (platform, GPU family, language version, fact family) while host attestation lives in per-row measurement provenance, and single-host evidence has a defined, fail-closed path toward (or a defined exclusion from) family-scoped claims.

## Why this exists

Filed 2026-08-18 from Tom's naming objection on the M3 Pro subgroup declaration: the delivered host-named profile key (`macos-m3pro-apple9`) was truthful pure-evidence discipline, but it coupled durable identity vocabulary to procurement. The current ledger convention (one execution environment per profile) is what forces per-host profiles; the candidate to beat is family-keyed profiles whose individual rows carry per-row execution provenance, with feasibility answering `Unknown` unless the querying context matches a row's scope. The standard-profile measurement deferral (`measure-thread-execution-width-on-the-standard-metal-profiles-own-host`) and the demotion carrier are both consumers of this decision.

## Closes when

A Pareto-complete packet (per the repository readiness gate) fixes the composition model — profile key scope, row provenance shape, the single-host-to-family rule, identity consequences — passes independent review, and Tom accepts one exact model.

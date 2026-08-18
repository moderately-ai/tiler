---
id: measure-thread-execution-width-on-the-standard-metal-profiles-own-host
title: Measure threadExecutionWidth on the standard Metal profile's own host
status: deferred
priority: p3
dependencies: []
related: [declare-metal-subgroup-realization-facts-in-the-target-profile, measure-metal-thread-execution-width-across-prepared-pipelines, correct-the-metal-profile-authority-ledgers-stale-identity-pins]
scopes: [research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [measurement, metal, subgroup, target-profiles, evidence, needs-tom]
---
## User-visible outcome

The standard macOS Apple9 profile (`tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`) can state a subgroup realization backed by a width measured on its own ledger host, instead of staying subgroup-silent forever or inheriting the M3 Pro row.

## Why deferred rather than ready

**Fact — the standard profile is subgroup-silent by its evidence, not by an oversight.** The retained width measurement (`spikes/target-profiles/metal-thread-execution-width`) is M3 Pro evidence whose frozen protocol pre-scoped it away from this profile by name; `declare-metal-subgroup-realization-facts-in-the-target-profile` therefore landed the evidence-backed row on a new M3 Pro-scoped declaration and asserted the standard profile's silence by test.

**Fact — no host currently matches the standard profile's ledger execution row.** The ledger row is `Apple M4 Max`, macOS 27.0 build `26A5388g`; the coordination M4 Max observed on 2026-08-18 reports build `26A5406e`. A width measured there could not source the existing row either — it would be a different execution environment, exactly the inheritance the ledger refuses.

So closing this needs one of two things, and choosing between them is Tom's: an M4 Max host restored to (or found at) the ledger's exact row plus an authorized quiet device window; or a profile revision that re-rows the standard declaration to a currently observable M4 Max environment — a decision about the profile's identity, since every measured row's validity context would move with it. Host and toolchain changes for measurements are Tom's under the evidence-environment rule either way.

## What a run would do

Repeat the frozen 34-pipeline protocol of `measure-metal-thread-execution-width-across-prepared-pipelines` — same matrix, flags, repetitions, custody, and stop conditions — on the qualified host, retain the record beside the existing one, and declare (through the Metal-owned factory pattern `BoundMetalSubgroupDeclaration` established) only what the record evidences: whole-subject equality, nothing extrapolated, silence for every unobserved subject.

## Closes when

The standard profile either carries a `Realized` subgroup row backed by a retained measurement on its own execution row, or a recorded decision retires the question.

## Trigger check log

- 2026-08-18 — **not fired.** `ssh m3` is an M3 Pro; the coordination M4 Max reports `sw_vers -buildVersion` → `26A5406e`, not the ledger's `26A5388g`. Reproduce: `sw_vers -buildVersion` and `sysctl -n machdep.cpu.brand_string` on the candidate host, compared against the ledger's execution table.

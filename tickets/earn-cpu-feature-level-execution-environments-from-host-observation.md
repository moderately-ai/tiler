---
id: earn-cpu-feature-level-execution-environments-from-host-observation
title: Earn CPU feature-level execution environments from host observation
status: todo
priority: p1
dependencies: []
related: [declare-cpu-vector-realization-facts-in-the-target-profile]
scopes: [implementation/runtime, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, runtime, eligibility, preflight, fail-closed]
---
## User-visible outcome

A CPU vector variant is eligible only after the runtime has observed the exact ISA feature level it claims, rather than trusting a caller-restated artifact profile or architecture name.

## Fact — 2026-08-11

Current variant eligibility compares a caller-stated `ExecutionEnvironment`; it performs no CPUID/HWCAP discovery. The scalar prototype's profile can be derived from the compilation itself, which is not independent host evidence.

## Required delivery

- Define a backend-owned, typed CPUID/HWCAP or equivalent observation that earns one exact feature-level execution environment.
- Require callers to choose the CPU approach explicitly; no architecture-only preset, compilation-profile echo, feature superset inference, or silent scalar fallback.
- Keep host eligibility distinct from compile-profile feasibility. Unknown/unowned probes refuse before routing commit with a typed reason.
- Bind observation environment, process/OS assumptions, provider revision, and resulting profile identity canonically where the accepted eligibility contract requires them.
- Perturb each feature bit/row, observation authority, cross-compile host, profile key/descriptor, and missing probe independently.

## Closes when

An artifact cannot make itself eligible by restating its own CPU feature profile and every accepted feature-level environment is earned from the bound host.

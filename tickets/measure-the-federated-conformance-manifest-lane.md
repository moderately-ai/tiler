---
id: measure-the-federated-conformance-manifest-lane
title: Measure the federated conformance manifest lane
status: todo
priority: p2
dependencies: [pilot-a-declaration-backed-private-registry-manifest, pilot-id-bearing-private-verifier-obligations, decide-the-installed-provider-conformance-declaration-surface, decide-the-zero-dependency-metal-aot-conformance-declaration-route]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, spike, conformance-progress, performance, tooling]
---
# Measure the federated conformance manifest lane

## Goal

Determine whether the proposed owner-local reporters, configured snapshots, canonical validation, and owner-set join are affordable and operationally reliable as the long-term audit denominator.

## Work

1. Build a bounded orchestrator spike over the completed private-registry and verifier pilots plus the decided configured-provider fixture.
2. Use explicit output directories and invocation identities; reject missing, extra, stale, duplicate, partial, and cross-run files.
3. Run the exact required target/feature/cfg matrix; prove an unexecuted configuration remains `Unknown` and a paired conditional subject/declaration cannot shrink the accepted universe.
4. Measure clean and warm compile cost, process count, wall time, output bytes, validation peak memory, and incremental cost after one owner changes.
5. Compare one process per family, one process per owner crate, and safe batching without allowing one failed family or configuration to disappear.
6. Kill reporters at each publication phase and prove partial output cannot be accepted.
7. State the expected cost at the full owner-family/configuration population and whether the lane belongs in routine audit, full qualification, or both.

## Non-goals

- Do not optimize by weakening exact owner-set completion.
- Do not parse stdout, introduce a consumer build step, or make qualification depend on network services.
- Do not report a scalar progress score.

## Stop conditions

Stop if a missing reporter can look like an empty family, concurrent runs can cross-contaminate, cleanup can erase retained evidence, or projected full-scale cost is unaffordable without an architectural change.

## Acceptance

- Measurements include environment, workload, repetitions, variance, and failure-path costs.
- Every publication perturbation shows exact failure text.
- The recommended batching policy preserves complete owner/family accounting.
- Reversal thresholds for the owner-local transport are explicit.

## Refs

- [Owner-private conformance inventory boundary](../docs/research/verification/owner-private-conformance-inventory-boundary.md)

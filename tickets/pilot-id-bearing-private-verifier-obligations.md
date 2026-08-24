---
id: pilot-id-bearing-private-verifier-obligations
title: Pilot ID-bearing private verifier obligations
status: todo
priority: p1
dependencies: [specify-the-canonical-owner-conformance-manifest-protocol]
related: [spike-a-red-yellow-first-full-conformance-suite, derive-the-optimizer-and-planner-capability-obligation-manifest]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, spike, conformance-progress, verification, kernel-ir]
---
# Pilot ID-bearing private verifier obligations

## Goal

Prove that individual owner-private verifier invariants and their checker invocation graph can become closed conformance populations even when many checks intentionally map to one public diagnostic.

## Work

1. Read one complete schedule, kernel, or kernel-program verifier family, its builders, diagnostics, consumers, and all positive/negative tests.
2. Select a family with several independent checks sharing one diagnostic, such as kernel reduction-contract validation.
3. Separate stable semantic `ObligationId` from `CheckerId` and optional `ViolationSiteId`; permit and validate justified many-to-one site-to-obligation mappings without making refactoring revise semantic identity.
4. Compare failure tagging, a declaration-driven checker executor/table, and a typestate chain. The selected route must make removal of an entire checker invocation mechanically visible, not merely require IDs on failures that still execute.
5. Derive semantic-obligation and checker-graph manifests from their owning declarations while preserving the current outer public diagnostic.
6. Add each real subject perturbation separately: a new unnamed guard, reused obligation/site ID, omitted manifest row, wrong diagnostic mapping, disabled validation call, deletion of the whole checker invocation, and a checker declaration whose implementation never runs.
7. Determine the smallest scalable owner-local structure that prevents a new or removed check from silently bypassing declaration and invocation closure.

## Non-goals

- Do not publish invariant IDs or change the public diagnostic schema.
- Do not enumerate every verifier in this pilot.
- Do not treat diagnostic variants, test functions, or source locations as subject identities.

## Stop conditions

Stop if coverage relies only on source grep, if deleting a complete checker call leaves the checker root/audit green, if semantic obligation identity must change for ordinary site refactoring, if public errors must change, or if the selected family cannot be bounded independently.

## Acceptance

- Semantic-declaration addition/removal and checker/invocation addition/removal fail loudly.
- Public diagnostics remain semantically unchanged.
- The manifest enumerates obligations rather than failure classes and separately accounts for checker invocation completeness.
- The report states the scalable migration pattern and its strongest counterexample.

## Refs

- [Owner-private conformance inventory boundary](../docs/research/verification/owner-private-conformance-inventory-boundary.md)

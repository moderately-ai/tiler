---
id: decide-how-owner-private-conformance-inventories-cross-crate-boundaries
title: Decide how owner-private conformance inventories cross crate boundaries
status: todo
priority: p1
dependencies: [inventory-the-closed-world-conformance-claim-universe-by-owner, define-the-conformance-obligation-and-evidence-requirement-algebra]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, decision, conformance-progress, architecture]
---
# Decide how owner-private conformance inventories cross crate boundaries

## Goal

A decision-ready boundary for observing owner-private capability inventories and evidence from the conformance system without publishing mutable compiler internals, duplicating authorities, or reversing dependency direction.

## Work

1. Read the complete construction and consumption paths for the semantic registry iterator, reference capabilities, compiler rewrite registry, lowering and physical providers, public compilation/session views, typed explain ownership, schedule/KIR identities, and the conformance crate's dependency/public-surface rules.
2. Inventory which required subjects are already observable, which are owner-private, and which have no canonical identity yet.
3. Compare: minimal immutable owner accessors; owner-emitted machine-readable manifests; feature-gated test-support surfaces; owner-local reporters composed by an orchestrator; moving neutral vocabulary into `tiler-ir`; a new shared crate; and deferral.
4. Account for Rust's cross-crate `cfg(test)` behavior: a dependency is not compiled as its own test target merely because the consuming crate is running tests.
5. Eliminate options that expose construction/mutation, create a second authority, require a consumer build step, make `tiler-conformance` a dependency of owners, or publish a boundary without a real consumer.
6. State schema, identity, versioning, dependency, host-cost, and future-consumer consequences for every survivor.
7. Use independent derivation for any public-boundary recommendation and present only the nondominated frontier.

## Non-goals

- Do not add an accessor, feature, crate, serialization format, or public API.
- Do not move layer-local tests into `tiler-conformance`.
- Do not make rendered explain text a parse contract.

## Stop conditions

Stop for Tom if the dominant solution changes a consequential public crate/module/type boundary or introduces a new crate or generated workflow.

## Acceptance

- Every required private subject has a proposed observation route or an explicit unsupported result.
- Dependency direction and authority ownership remain valid for every survivor.
- The packet gives the smallest safe boundary, its strongest counterargument, and evidence that could reverse it.
- Follow-up implementation is split by owner and public-boundary risk.

## Refs

- [`survey-what-belongs-in-the-conformance-crate`](survey-what-belongs-in-the-conformance-crate.md)
- [`decide-the-backend-provider-conformance-harness-public-surface`](decide-the-backend-provider-conformance-harness-public-surface.md)
- [`inventory-the-closed-world-conformance-claim-universe-by-owner`](inventory-the-closed-world-conformance-claim-universe-by-owner.md)

---
id: activate-a-public-composed-realization-oracle-for-a-named-consumer
title: Activate a public composed-realization oracle for a named consumer
status: deferred
priority: p3
dependencies: [implement-the-composed-realization-evaluation-driver]
related: [decide-the-safe-cross-crate-composed-reference-boundary, define-the-composed-realization-driver-subject-bridge]
scopes: [implementation/conformance, implementation/reference, implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [deferred, public-boundary, conformance, reference, compatibility]
---
## Outcome

A named non-test consumer receives a supported composed-realization oracle from the narrowest reusable crate boundary, without turning the device-evidence crate into a dependency by default or exposing raw tensor pinning.

## Deferred boundary

[`decide-the-safe-cross-crate-composed-reference-boundary`](decide-the-safe-cross-crate-composed-reference-boundary.md) accepted a test-only `pub(crate)` plan-binding wrapper for the current population. That is complete for every named consumer today. This ticket is the durable remainder: when a non-test consumer exists, compare promoting a public entry from `tiler-conformance` against admitting a narrow oracle crate depending only on compiler, IR, and reference. The latter is the current long-term leader because consumers would not inherit artifact/build/cache/Metal/runtime evidence dependencies, but creating a crate before a consumer exists would freeze a namespace and dependency role without evidence.

Raw `(ValueId, Tensor)` pin/observe remains private under either outcome. The safe reference-owned session remains explicit about registry/work authority and accepts no caller-supplied internal tensor. No default implementation, governed registry, baseline, or backend fallback is authorized.

## Trigger

A concrete non-test crate or external integration needs to invoke composed plan conformance as reusable functionality rather than as Tiler's own gated evidence.

## Trigger check log

- 2026-08-12 — **not fired**. Every current caller is inside `tiler-conformance`'s `#[cfg(test)]` population; the workspace is `publish = false`, nothing may depend on the evidence-top crate, and no external consumer was named. Reproduce with `rg -n 'PlanAlternative|strict_partitioned_sum' crates/tiler-conformance/src` plus the `# Public surface` and `Every module is #[cfg(test)]` anchors in `crates/tiler-conformance/src/lib.rs`.

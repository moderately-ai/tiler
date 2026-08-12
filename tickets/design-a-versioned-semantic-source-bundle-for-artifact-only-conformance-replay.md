---
id: design-a-versioned-semantic-source-bundle-for-artifact-only-conformance-replay
title: Design a versioned semantic source bundle for artifact-only conformance replay
status: deferred
priority: p3
dependencies: []
related: [retain-the-selected-semantic-candidate-for-the-conformance-oracle, implement-the-composed-realization-evaluation-driver]
scopes: [research/reference, research/artifacts, implementation/compiler, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [deferred, research, conformance, artifact, identity, schema]
---
## User-visible outcome

If a real consumer needs to reproduce conformance from an artifact without the live compilation, Tiler decides a versioned, self-contained semantic source bundle rather than pretending an identity digest can rehydrate a program.

## Why deferred

The accepted in-process driver retains the exact Arc-backed semantic candidate on the compilation alternative. Current artifacts intentionally carry semantic identity and reached execution evidence, not a serializable semantic graph. No current consumer requires artifact-only replay, so adding program bytes, codec limits, validation, lifecycle, and cache policy now would be speculative schema expansion.

## Trigger

Fire only when a named consumer must run conformance in a process or environment that has the artifact/proof record but cannot retain or receive the originating `Compilation`/`PlanAlternative`.

When fired, compare an artifact component, a separately authenticated proof/source sidecar, and a content-addressed external bundle. Require complete semantic registry/definition/admission/shape-environment authority, exact canonical decoding, bounded allocation, explicit unavailable/corrupt/version refusals, and no network/default resolution fallback. Decide whether bundle identity is folded into artifact identity or bound by a separately authenticated receipt.

## Non-goals

Using `ProgramAlternativeIdentity`, graph digest, stable label, explain trace, KIR, or emitted code as a reconstructable semantic program; making current in-process conformance wait on this work.

## Trigger check log

- **2026-08-12 — not fired.** Source audit at `1f9629ad46b3717b1ef741f5cce36527e533b86d`: the accepted composed driver has no artifact-only caller, and `tiler-conformance` runs above the live compiler/reference stack. Recheck by searching callers and requirements for artifact/proof replay without `Compilation` or `PlanAlternative`; a named non-test consumer is required to fire.

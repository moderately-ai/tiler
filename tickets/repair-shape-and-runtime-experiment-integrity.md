---
id: repair-shape-and-runtime-experiment-integrity
title: Repair shape and runtime experiment integrity
status: todo
priority: p1
dependencies: []
related: []
scopes: [research/shapes, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [research, correctness, experiments]
---

Repair shape-evidence and runtime-validation experiment provenance found by the
fixed-point audit at `ad6e9f463de6eabad44af47eaddad9317e0935fd`.

## Required outcome

- Make stable shape measurement entrypoints regenerate or independently verify
  the checked-in summaries through a governed transformation from raw output.
- Derive nightly measurement dates from the actual run and give every compiler
  subprocess an overall deadline. Reconcile stale toolchain text and make
  platform-specific `time`/`stat` prerequisites explicit.
- Publish the runtime semantic-validation benchmark command, mark it as a
  bounded measurement, and retain individual samples plus exact environment
  rather than only medians.
- Make the Candle source audit verify the exact expected commit/revision and
  source cleanliness before accepting the structural pattern.

## Acceptance

From a clean checkout, each documented command must either regenerate the cited
result or compare a fresh result against it, with exact run date, toolchain,
source revision, samples, and timeout behavior. Missing provenance must fail
closed.

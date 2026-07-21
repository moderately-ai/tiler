---
id: repair-apple-target-experiment-integrity
title: Repair Apple target experiment integrity
status: todo
priority: p1
dependencies: []
related: []
scopes: [research/apple-targets, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [research, correctness, experiments]
---

Repair Apple compatibility/runtime probes and the Metal crate ownership text
found inconsistent by the fixed-point audit at
`ad6e9f463de6eabad44af47eaddad9317e0935fd`.

## Required outcome

- Make the runtime-failure probe return failure for unexpected library lookup,
  pipeline creation, or successful execution outcomes rather than merely
  printing them.
- Make the compatibility probe fail closed when host, SDK, Metal compiler, or
  toolchain provenance is missing or malformed; compile-matrix success alone
  is not a complete evidence record.
- Preserve exact commands and compact raw result/artifact metadata required by
  the accepted compatibility claim, or narrow the historical ticket/report.
- Correct `crates/tiler-metal/src/lib.rs` and its Cargo package description so
  the pure lowering crate does not claim ownership of Apple offline compiler
  invocation.

## Acceptance

Tests must inject each unexpected runtime stage and each missing provenance
field and require nonzero failure. The published procedure must regenerate or
verify the retained evidence on a supported Apple host.

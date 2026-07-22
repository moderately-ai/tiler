---
id: repair-apple-target-experiment-integrity
title: Repair Apple target experiment integrity
status: done
priority: p1
dependencies: []
related: []
scopes: [research/apple-targets, implementation/metal, implementation/workspace]
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

## Outcome

- `runtime_failure_probe.swift` now exits nonzero for every unexpected library,
  function, and pipeline outcome. Its declared stages share one fatal path;
  the macOS mutation harness compiles the probe and verifies every injected
  stage fails.
- `compatibility_probe.sh` now writes a versioned evidence record containing
  host/SDK/tool provenance, exact commands, SDK extracts, command-log and
  artifact digests, and reproducibility comparisons. The validator rejects
  duplicate, missing, malformed, or retained-file-mismatched evidence before
  the probe can succeed.
- The retained Xcode 26.6 / Metal 32023.883 run regenerated and validated all
  twelve compilations on the supported Apple host. The report distinguishes
  the incomplete historical raw record from the new checked-in evidence.
- `tiler-metal` now claims only pure structured-kernel-to-MSL lowering; the
  canonical workspace manifest checker enforces the corrected ownership text.
- Validation passed with the probe mutation suite, the live runtime control,
  the retained full compatibility matrix, documentation gates, 142 Python
  tests, and `uv run --locked python scripts/check_repository.py`.

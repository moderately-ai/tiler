---
id: repair-macro-and-embedding-harness-integrity
title: Repair macro and embedding harness integrity
status: todo
priority: p1
dependencies: []
related: []
scopes: [research/macro-environment, research/embedding, research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [research, correctness, experiments]
---

Repair false-success, nontermination, nonreproducibility, and provenance gaps
in macro-environment and embedding experiments found by the fixed-point audit at
`ad6e9f463de6eabad44af47eaddad9317e0935fd`.

## Required outcome

- Make macro-environment probes assert the environment fields, fingerprints,
  cache attribution, and post-`cargo test` expansion counts that support the
  report; preserve an auditable trace rather than deleting the only evidence.
  The cross-target entrypoint must require a genuinely distinct requested
  target, and family-cfg probes must run or reject explicitly on every
  documented host rather than relying on a macOS-only compile error.
- Make embedding measurements bounded, fail on missing/unparseable RSS or
  Mach-O section metrics, record every inherited output-affecting Cargo/Rust
  input plus source revision, and reconcile the claimed retained raw output.
- Give every embedding compiler/tool subprocess an overall deadline.
- Fix direct-entrypoint executable mode or document interpreter invocation.
  Keep platform-specific shell/tool requirements explicit and reproducible.

## Acceptance

Each harness must have an explicit success predicate, overall timeout, required
metric/provenance schema, and a test for malformed/missing tool output. The
documented command must regenerate or verify every cited result fixture from a
clean checkout without silently substituting a weaker metric.

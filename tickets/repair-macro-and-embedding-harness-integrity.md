---
id: repair-macro-and-embedding-harness-integrity
title: Repair macro and embedding harness integrity
status: done
priority: p1
dependencies: []
related: []
scopes: [research/macro-environment, research/embedding, research/extensions, implementation/workspace, contracts/navigation]
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

## Outcome

- Macro-environment results now use a versioned, strictly parsed trace and
  assert tokens, complete environment-field states, fingerprints, cache
  attribution, and the `1,1,1,2,2,3,4,7` expansion sequence including
  `cargo test`. Native and family-cfg raw/decoded evidence is retained and
  source-bound. A target must be installed and genuinely distinct from the
  host; this host still has no such target, so that measurement remains
  explicitly unavailable.
- Embedding commands have per-command process-group deadlines, the complete
  run has an independent one-hour default deadline, and subprocess output has
  a combined 16-MiB streaming cap. Required time, RSS, Mach-O, binary, source,
  payload, executable, complete inherited-environment, and harness identities
  fail closed. New runs clean required scratch state before atomically
  publishing `complete.json` into a fresh output directory. The historical
  81-result/12-freshness fixture is checked against an exact digest manifest
  and verified as internally consistent derived evidence, but its formerly
  claimed raw logs were absent; that limitation is recorded rather than hidden
  or reconstructed on a changed toolchain.
- Extension entrypoints use a documented interpreter wrapper and a bounded
  runner with strict positive/cycle predicates, tool/source provenance, trace
  retention, process-group timeout, and a four-MiB streaming output cap.
- The default locked pytest/Ruff configuration includes these harnesses. Unit,
  malformed-output, timeout, retained-evidence, native/family, extension, and
  fresh three-case macOS embedding-smoke checks passed on 2026-07-21.

---
id: prototype-semantic-owner-and-commit
title: Enforce semantic graph ownership and compact commitment
status: done
priority: p0
dependencies: [prototype-workspace-scaffold]
related: [prototype-semantic-reference-slice]
scopes: [implementation/ir, research/semantic-graph]
shared_scopes: [project/tickets, contracts/foundation, contracts/decisions, implementation/cargo-lock]
paths: []
tags: [implementation, prototype, semantics, rust-api]
---
Implement the settled graph-owner and draft-to-program commitment contracts in
ADRs 0058, 0063, and 0064 without adding semantic operations.

- make owner-token allocation fallible and prevent live-owner aliasing on
  exhaustion;
- distinguish foreign-owner, invalid-local-handle, and argument-role failures;
- validate every handle-consuming query, refinement, witness, output, and
  insertion before indexing or mutation;
- compute the output-reachable closure, compact live values and operations,
  rewrite all internal references, and assign a distinct completed owner;
- preserve retained source/explanation provenance and the internal commit remap
  needed for a future additive build report; and
- establish output-selector mechanics that the later typed-handle ticket can
  refine without stabilizing arena indices.

The ticket succeeds when coincident local indices from different graphs fail
closed, every failed insertion leaves the builder unchanged, successful build
invalidates all draft handles, completed counts and iterators contain exactly
the live closure, output selectors resolve to new completed handles, and injected
owner exhaustion returns a typed error. Do not implement resolved value types,
shape evidence, semantic operations, optimizer rewrites, or a public mandatory
build report.

## Outcome

- Made draft and completed owner allocation fallible without wraparound reuse,
  including an injected completed-owner exhaustion path that returns the intact
  builder.
- Added typed foreign-owner and invalid-local failures with operand/output role
  diagnostics; all existing handle-consuming APIs validate before mutation.
- Compacted the exact output-reachable input, operation, and value closure into
  dense storage, rewrote every retained edge/interface, and assigned a distinct
  completed owner so draft handles fail closed.
- Added origin-bound, arena-index-free `OutputSelector` resolution as the
  untyped foundation for the subsequent `Output<T>` work.
- Covered coincident indices, transactional failures, dead-state removal,
  selector provenance, dense remapping, and allocation exhaustion with tests.

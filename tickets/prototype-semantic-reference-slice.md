---
id: prototype-semantic-reference-slice
title: Implement the serial Sum semantic and reference slice
status: in-progress
priority: p0
dependencies: [prototype-workspace-scaffold, prototype-shaped-value-api, prototype-reference-evaluator-crate]
related: []
scopes: [implementation/ir, research/semantic-graph, research/shapes, implementation/workspace, research/documentation, implementation/reference]
shared_scopes: [project/tickets, contracts/foundation, contracts/numerics, contracts/decisions, contracts/navigation]
paths: [AGENTS.md, CLAUDE.md, .gitignore]
tags: [implementation, prototype, semantics, vertical-slice]
assignee: codex
lease_expires_at: 1784605806
---
Build and evaluate the complete target-independent input to the first value
proof:

- typed `f32` input, constant, multiply, add, strict serial `Sum`, and named
  output nodes;
- recoverable construction, immutable completion, graph and shape validation,
  canonical contributor order, and deterministic semantic identity;
- the normative host reference evaluator, including canonical arithmetic-NaN,
  signed-zero, infinity, subnormal, contraction-sensitive, empty-domain,
  singleton, overflow, and malformed-graph cases; and
- an ordinary public experimental Rust construction path whose types preserve
  the accepted semantic/physical and operation/property boundaries.

The slice succeeds when equivalent construction orders produce the same
identity and result, invalid programs produce specific typed diagnostics, and
the program can be consumed by the next compiler slice without any frontend,
optimizer, artifact, Metal, or runtime dependency.

This is the integration gate for the dependency-ordered owner/commit,
resolved-type registry, exact typed-handle, shape-evidence spike, and checked
shaped-value tickets. It does not reimplement those components. It migrates the
bounded evaluator and operation set onto them, versions semantic identity away
from the prototype's implicit graph-wide `f32` assumption, completes malformed
and numerical boundary cases, and proves the assembled public path.

## Outcome

- Integrated the dependency-ordered semantic authority, operation registry,
  typed handles, checked shape evidence, graph ownership, recoverable build,
  output-reachable compaction, and downstream reference-evaluator boundaries.
- Exercised the ordinary public API from a downstream-style integration test:
  an exact-shaped `f32` input flows through scalar Multiply, scalar Add, strict
  serial Sum, two ordered named outputs, immutable commitment, and reference
  evaluation without a frontend, optimizer, artifact, Metal, or runtime
  dependency.
- Proved equivalent live graphs retain identical canonical identity and output
  tensors despite different dead construction history.
- Completed the first-profile numerical vectors for constant NaN payloads,
  canonical arithmetic NaN, infinities, preserved and produced subnormals,
  signed zero, finite overflow, separate Multiply/Add rounding, strict
  contributor order, singleton reductions, and both empty-domain cases.
- Added fail-closed coverage showing corrupted private draft state cannot be
  committed and external reference capabilities cannot return an incorrect
  result arity or shape.

This completes the bounded integration gate. The API remains experimental and
unstable; completion does not broaden the implemented operation or dtype
profile.

---
id: prototype-reference-evaluator-crate
title: Extract the reference evaluator capability crate
status: todo
priority: p0
dependencies: [prototype-typed-value-handles]
related: [prototype-semantic-reference-slice]
scopes: [implementation/reference, implementation/ir, implementation/workspace, research/reference]
shared_scopes: [project/tickets, contracts/foundation, contracts/numerics, contracts/decisions, contracts/navigation]
paths: []
tags: [implementation, reference, rust-api]
---
Create `tiler-reference` as a target-independent downstream consumer of
`tiler-ir` and migrate the bounded numerical oracle into it.

- move host tensor values, input bindings, evaluator traversal, evaluation
  errors, and all numerical oracle cases out of `tiler-ir`;
- define an explicit frozen reference-capability registry keyed by `OpKey`,
  resolved signature, provider identity, and capability revision;
- register the governed F32 operation evaluators through the same capability
  path used by an external reference provider;
- reject a semantically valid operation lacking reference capability with a
  typed missing-capability diagnostic rather than `MalformedProgram`;
- preserve exact separate multiply/add boundaries, canonical arithmetic NaN,
  signed zero, strict contributor order, and empty-domain behavior; and
- ensure compiler, artifact, Metal, and runtime production dependencies do not
  acquire `tiler-reference`.

Add the workspace dependency and ticketsplease scope, update ADR 0056 through
its accepted supersessor, and move conformance tests without weakening them.

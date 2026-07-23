---
schema: "tiler-doc/v1"
id: "ADR-0058"
kind: "decision"
title: "Use a recoverable consuming semantic-program build"
topics: ["rust", "semantics", "graph", "api"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.architecture", "tiler.contract.ir"]
evidence: ["tiler.research.semantic-graph.rust-construction-lifecycle", "tiler.research.shapes.shape-environment-contract"]
ticket: "prototype-semantic-owner-and-commit"
---

# 0058: Use a recoverable consuming semantic-program build

**Status:** accepted

## Context

Frontends need incremental construction and precise insertion errors. Compiler
passes need a complete immutable program, and large arena-backed drafts must not
be cloned through an innocuous-looking API. Public newtypes must preserve
domain distinctions without exposing storage or encouraging semantic handles
to be reused by physical IRs.

## Decision

Use a non-`Clone`, append-only `SemanticProgramBuilder` with transactional
fallible edits. Every failed edit leaves the draft unchanged. Provide borrowed
validation for diagnostics, then make the commitment point a consuming,
recoverable conversion:

```text
build(self) -> Result<SemanticProgram, ProgramBuildError>
```

`build` defensively verifies whole-program invariants and transfers owned
storage without cloning the draft. ADR 0064 permits and requires consuming
reachability compaction during that transfer; it does not require preserving
draft arena numbering or handle validity. On failure, `ProgramBuildError` owns
structured diagnostics and the original builder so the caller can inspect,
correct, and retry. Borrowed accessors expose both; `into_builder` and
`into_parts` recover ownership.

`SemanticProgram` is immutable and owns private `Arc<ProgramData>` storage.
Completed programs are cheap to clone; compiler, optimizer, and evaluator APIs
borrow them. A shared `OnceLock` may cache derived canonical identity, but
allocation identity never enters semantic identity.

Expose conceptual `shape`, `semantic`, and `reference` namespaces in
`tiler-ir`. Keep implementation submodules and fields private. Shape vocabulary
belongs to `shape`; graph handles and interface keys belong to `semantic`.
Do not create a generic `newtypes` module or a separate identifier crate.

ADR 0065 supersedes the `reference` namespace named above: reference evaluation
moved to the separate `tiler-reference` crate and `tiler_ir::reference` no
longer exists. The `shape` and `semantic` namespaces and every other rule in
this paragraph remain in force.

Public semantic handles are opaque, typed, graph-owned, and invalid across
graphs. They have no public raw constructor or serialization promise. Internal
edges use private compact typed `u32` indices; later IR levels define their own
handle types.

Do not provide implicit snapshots, mutable thawing, builder `Clone`, or hidden
copy-on-write behavior. A future explicit `snapshot` or `fork` requires a
measured need and a separately reviewed cost and identity contract.

## Consequences

- The public type boundary distinguishes editable drafts from verified
  compiler input.
- Successful build avoids cloning draft storage, although required
  output-reachability compaction performs O(graph-size) commitment work.
- Failed terminal validation preserves the caller's graph and allocations.
- Public handles fail closed when mixed across graphs, while internal graph
  edges remain compact.
- Frontends that need unfinished-graph branching must wait for an explicit
  persistent-draft design or replay their construction; no accidental
  performance promise is made now.

## Alternatives considered

`build(&self)` hides a deep clone unless storage is persistent. A clonable
mutable builder makes the same unsupported performance promise. Allowing passes
to accept mutable drafts makes validity depend on call ordering. A global
newtype module or microcrate groups unrelated identities by representation
rather than by the invariants they protect.

## Traceability

The [Rust lifecycle research](../research/semantic-graph/rust-construction-lifecycle.md)
separates the relevant guarantees from DataFusion, Cranelift, Naga, Polars,
`egg`, Rust API guidance, and recoverable standard-library conversions. The
[IR contract](../ir.md) owns graph invariants and public semantic boundaries;
the [architecture contract](../architecture.md) owns the verified-program
handoff to compilation.

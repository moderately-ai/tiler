---
schema: "tiler-doc/v1"
id: "tiler.research.semantic-graph.rust-construction-lifecycle"
kind: "research"
title: "Rust semantic-program construction lifecycle"
topics: ["rust", "semantics", "graph", "api"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "adopted"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.architecture", "tiler.contract.ir"]
adopted_by: ["ADR-0058"]
ticket: "prototype-semantic-reference-slice"
---

# Rust semantic-program construction lifecycle

## Question

What Rust ownership and module boundary should let frontends construct a
semantic tensor graph incrementally, reject invalid edits without corrupting
the draft, and hand compiler passes a cheap-to-share immutable program without
quietly cloning an arena-sized graph?

## Primary-source findings

The versioned crate documentation inspected on 2026-07-20 was DataFusion
54.0.0, Cranelift 0.134.0, Naga 30.0.0, Polars 0.54.4, and `egg` 0.11.0. The
Rust API Guidelines and stable standard-library pages are unversioned moving
references; the cited ownership APIs themselves predate Tiler's Rust 1.89
minimum.

### Rust API guidance

**Fact:** The Rust API Guidelines distinguish non-consuming and consuming
builders. They prefer a non-consuming terminal method when producing the result
does not require ownership, but prescribe a consuming terminal method when the
result must take ownership from the builder. The same guidance recommends
newtypes for static distinctions, private fields for invariant-bearing public
types, and taking ownership rather than borrowing and cloning when a function
requires ownership. See [type safety], [future proofing], and [flexibility].

**Inference:** Moving Tiler's graph arenas into a completed program is a real
ownership transfer. `build(self)` makes that cost explicit and avoids an
otherwise hidden graph clone. Mutable `&mut self` insertion remains appropriate
for conditional, incremental frontend construction even though the terminal
operation consumes the builder.

### DataFusion

**Fact:** DataFusion's `LogicalPlanBuilder` owns its chaining methods and ends
with `build(self) -> Result<LogicalPlan>`. It is also `Clone`, exposes the
current plan by reference, and can be reconstructed from a completed plan.
See [`LogicalPlanBuilder`].

**Inference:** DataFusion validates the ordinary Rust ergonomics of a consuming
plan transition, but not the claim that cloning an unfinished arena graph is
cheap. Tiler should copy the explicit terminal ownership transition, not infer
a `Clone` promise from a differently represented relational plan.

### Cranelift

**Fact:** Cranelift describes `FunctionBuilder` as a temporary object for
building one function. Instructions are inserted through mutable borrows and
`finalize(self, ...)` consumes the builder when translation completes.
Independent verification is still performed on the resulting function. See
[`FunctionBuilder`] and the [Cranelift frontend example].

**Inference:** Mutable incremental construction, a consuming commitment point,
and defensive whole-IR verification are compatible responsibilities. A
terminal transition need not imply that all earlier checks were deferred.

### Naga

**Fact:** Naga stores most module contents in typed `Arena`s referenced by
typed `Handle`s. Its valid-module contract requires arena references to obey
ordering and acyclicity constraints. A separate `Validator::validate(&Module)`
returns derived `ModuleInfo` only after checking the module, including its
handles. See [`Module`], [`Arena`], and [`Validator`].

**Inference:** Compact typed indices are effective internal wiring, but arena
membership and validity remain properties of the containing IR. Tiler's public
handles therefore need graph ownership checks, while its internal edges can
store private compact indices. Neither form is a durable semantic identity.

### Polars

**Fact:** Polars' user-facing `DslPlan` uses `Arc<DslPlan>` for many recursive
inputs. Its consuming `to_alp(self)` conversion creates `Arena<IR>` and
`Arena<AExpr>` and returns an `IRPlan` containing the arenas and a root `Node`.
Some inspection paths explicitly clone the DSL plan before this conversion.
See [`DslPlan`] and its [`to_alp` source].

**Inference:** Persistent trees and mutable arena IRs have different cloning
economics. Tiler should make the completed program cheap to share with `Arc`,
without implying that an unfinished arena-backed draft has persistent-snapshot
semantics.

### egg

**Fact:** `egg::EGraph` is mutated through `add` and `union`. Its documentation
warns that clients must call `rebuild` after mutation to restore congruence and
uniqueness invariants; queries before rebuild may be stale or incorrect. See
[`EGraph::rebuild`].

**Inference:** An explicit dirty/clean lifecycle can be appropriate inside an
optimizer whose API is built around rebuilding. It is a poor default compiler
input contract for Tiler: downstream passes should accept only an immutable,
verified `SemanticProgram`, not a mutable draft whose query validity depends on
call ordering.

### Recoverable ownership failures

**Fact:** `String::from_utf8(Vec<u8>)` consumes its allocation, but
`FromUtf8Error::into_bytes` returns the original vector without reallocating.
Likewise, `CString::into_string` returns an `IntoStringError` that can yield the
original `CString`. See [`FromUtf8Error`] and [`IntoStringError`].

**Inference:** A fallible consuming conversion need not destroy expensive input
state. A Tiler build error can own both structured diagnostics and the original
builder, permitting inspection, correction, and retry without making builder
cloning part of the API contract.

## Adopted design synthesis

**Proposal adopted by ADR 0058:** use three conceptual public namespaces in
`tiler-ir`: `shape`, `semantic`, and `reference`. Organize implementation files
inside those concepts rather than collecting unrelated wrappers in a generic
`newtypes` module. Keep fields and storage private.

`SemanticProgramBuilder` is append-only and non-`Clone`. Fallible insertions are
transactional: an error leaves the observable and internal draft unchanged. A
borrowed `validate(&self)` supports diagnostics, while
`build(self) -> Result<SemanticProgram, ProgramBuildError>` defensively checks
whole-program invariants and moves the arenas into the completed program.
`ProgramBuildError` retains the builder and diagnostics.
Borrowed accessors expose the diagnostics and draft for inspection;
`into_builder` and `into_parts` recover ownership without cloning.

`SemanticProgram` owns private `Arc<ProgramData>` storage. Under the later ADR
0064 refinement, successful commitment compacts the output-reachable closure
and assigns a new completed-program owner rather than preserving draft handles.
Completed programs
are immutable and cheap to clone; compiler, optimizer, and evaluator APIs
borrow them. Canonical identity may be cached in the shared data, but excludes
the `Arc`, graph-owner token, arena numbering, dead insertion order, and other
allocation identity. The initial shared cache uses `OnceLock`, so identity is
computed at most once for all clones of the same completed program.

Public `ValueId` and `OperationId` are opaque, typed, graph-owned handles with
no raw public constructor, serialization promise, or cross-graph validity.
Private internal edges use compact typed `u32` indices. Shape vocabulary
(`Axis`, `Extent`, `Shape`) lives under `shape`; semantic handles and interface
keys live under `semantic`. Future physical and kernel representations receive
distinct handle newtypes even when their storage representation is identical.

No implicit draft snapshot, builder `Clone`, copy-on-write arena, or mutable
thawing API is included. If a measured frontend later needs unfinished-graph
branching, it requires an explicit `snapshot`/`fork` contract and a data
structure whose cost is visible. Completed programs already branch cheaply.
Draft-to-program correlation may later be exposed by an additive build report;
typed output selectors and retained provenance carry the initially required
cross-boundary references without stabilizing arena indices.

## Rejected alternatives

- A generic `newtypes` module groups by Rust syntax instead of domain
  invariant, encouraging accidental reuse across semantic and physical IRs.
- A separate identifier crate creates dependency and compatibility surface
  without an independent consumer: these types are inseparable from
  `tiler-ir` validation.
- `build(&self)` would need to clone arena storage or introduce hidden
  persistence. Neither cost has evidence.
- `Clone` on the mutable builder would imply cheap snapshots that its initial
  contiguous storage does not provide.
- Allowing compiler passes to consume a draft and call `validate` themselves
  weakens the type boundary and makes query safety dependent on convention.

## Measurement boundary and future triggers

This is API and source precedent, not a benchmark of the prototype's graph
sizes or `Arc` contention. Reconsider draft snapshotting only when a concrete
frontend demonstrates branching requirements and measurements compare deep
copy, chunked/persistent arenas, and replay. Reconsider extracting shared shape
vocabulary only when an independent crate must use it without otherwise
depending on `tiler-ir`.

## Sources

- [Rust API Guidelines: type safety][type safety]
- [Rust API Guidelines: future proofing][future proofing]
- [Rust API Guidelines: flexibility][flexibility]
- [DataFusion `LogicalPlanBuilder`][`LogicalPlanBuilder`]
- [Cranelift `FunctionBuilder`][`FunctionBuilder`]
- [Cranelift frontend example]
- [Naga `Module`][`Module`], [`Arena`], and [`Validator`]
- [Polars `DslPlan`][`DslPlan`] and [`to_alp` source]
- [`egg::EGraph::rebuild`][`EGraph::rebuild`]
- [`std::string::FromUtf8Error`][`FromUtf8Error`]
- [`std::ffi::IntoStringError`][`IntoStringError`]

[type safety]: https://rust-lang.github.io/api-guidelines/type-safety.html
[future proofing]: https://rust-lang.github.io/api-guidelines/future-proofing.html
[flexibility]: https://rust-lang.github.io/api-guidelines/flexibility.html
[`LogicalPlanBuilder`]: https://docs.rs/datafusion/latest/datafusion/logical_expr/struct.LogicalPlanBuilder.html
[`FunctionBuilder`]: https://docs.rs/cranelift-frontend/latest/cranelift_frontend/struct.FunctionBuilder.html
[Cranelift frontend example]: https://docs.rs/cranelift-frontend/latest/cranelift_frontend/
[`Module`]: https://docs.rs/naga/latest/naga/ir/struct.Module.html
[`Arena`]: https://docs.rs/naga/latest/naga/struct.Arena.html
[`Validator`]: https://docs.rs/naga/latest/naga/valid/struct.Validator.html
[`DslPlan`]: https://docs.rs/polars/latest/polars/prelude/enum.DslPlan.html
[`to_alp` source]: https://docs.rs/crate/polars-plan/latest/source/src/dsl/plan.rs
[`EGraph::rebuild`]: https://docs.rs/egg/latest/egg/struct.EGraph.html#method.rebuild
[`FromUtf8Error`]: https://doc.rust-lang.org/stable/std/string/struct.FromUtf8Error.html
[`IntoStringError`]: https://doc.rust-lang.org/stable/std/ffi/struct.IntoStringError.html

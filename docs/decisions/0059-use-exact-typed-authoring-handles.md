---
schema: "tiler-doc/v1"
id: "ADR-0059"
kind: "decision"
title: "Use exact typed authoring handles over runtime-typed semantic values"
topics: ["rust", "semantics", "dtypes", "api"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.ir", "tiler.contract.numerical-semantics", "tiler.contract.operation-extensions"]
evidence: ["tiler.research.numerics.dtype-resolution-precedents", "tiler.research.numerics.dtype-identity-admission-policy", "tiler.research.extensions.operation-extension-surface"]
ticket: "prototype-semantic-reference-slice"
---

# 0059: Use exact typed authoring handles over runtime-typed semantic values

**Status:** accepted

## Context

Ordinary Rust construction should reject known dtype/signature mistakes at the
call site. A public API made only of erased value IDs would postpone those
errors despite having enough static information to prevent them.

The canonical compiler graph must nevertheless contain heterogeneous values,
support parsed frontends and externally registered nominal dtypes, and remain
serializable independently of Rust monomorphization and `TypeId`. Treating one
value as TypeScript-like `any` would weaken both boundaries: callers could use
it without proof while the graph lost an authoritative type contract.

Result types also depend on operation-specific numerical choices. Ambient
promotion would make authoring convenient at the cost of obscuring accumulator,
conversion, rounding, and result semantics that already belong to semantic
identity.

## Decision

The primary Rust construction surface uses exact nominal typed handles of the
conceptual form `Value<T>`. `T` identifies a complete static semantic tensor
type, such as `F32`, rather than only a coarse `Float` or `Integer` family.
Builder methods express every statically knowable operand/result relationship
through these handles.

The canonical graph remains heterogeneous and runtime represented. Under ADR
0062, each value stores an authoritative, complete, versioned resolved value
type alongside its
shape and definition. The graph does not store Rust `T`, `TypeId`, type names,
or monomorphized operation data. `Value<T>` is a zero-cost authoring capability
over a graph-owned opaque `ValueId`; only the owning builder or program may
construct or recover it after checking that the stored resolved value type
exactly matches `T`'s frozen registry binding.

`ValueId` means that the dtype is unknown at the current Rust call site, not
that the value may be used as any dtype. It grants identity and lookup only.
Borrowed `ValueRef` inspection exposes the authoritative runtime type, shape,
and definition. There is no `AnyValue`, unchecked `Value<T>::from_id`, implicit
retyping, or unvalidated general insertion API. Erasure to identity is explicit;
recovering a typed handle is graph-owned and fallible.

Result-affecting promotion, accumulator, conversion, rounding, and output
choices are explicit operation signature/contract inputs. Typed builders return
the statically resolved result handle. Frontends may implement their own
promotion or autocast policy before semantic admission, but they lower it into
explicit operands, conversions, contracts, and result types. Tiler applies no
ambient or backend-selected promotion after admission.

The initial prototype need not expose public runtime-resolved construction.
When a concrete parsed frontend requires it, the only admitted dynamic path is
a separately reviewed, registry-backed builder: the registered semantic
authority validates ordered operands, canonical attributes, and exact result
types through host-owned mutation. It does not create typeless values or permit
callers to declare arbitrary result types.

ADR 0061 refines the shape portion of this decision. `Value<T>` remains the
canonical typed graph capability, while optional graph-checked
`ShapedValue<T, E>` handles may carry non-authoritative rank or static-shape
evidence. Symbolic extents, broadcasting, graph ownership, global constraints,
capabilities, and target feasibility remain authoritative at their earliest
sound graph/compiler boundary.

## Consequences

- Known nominal dtype/signature mistakes fail during Rust compilation.
- Dynamic and heterogeneous compiler traversal remains possible through opaque
  IDs and runtime type metadata without an `any` escape hatch.
- Exact F16/F32, integer, complex, and quantized distinctions can participate in
  authoring APIs; a coarse kind marker is insufficient.
- Serialization and semantic identity remain independent of Rust implementation
  types and monomorphization.
- Mixed-precision call sites are more explicit, but their numerical behavior is
  visible, deterministic, and cacheable.
- External marker governance and ergonomic operation-trait design require
  further decisions; neither may weaken registry authority or diagnostics.
- Optional shape evidence may improve Rust call-site checking under ADR 0061,
  but it never replaces canonical graph shape metadata or verification.
- The current untyped prototype builder must be revised before its public API is
  approved.

## Alternatives considered

Only exposing `ValueId` is simpler but delays preventable errors. Making the
canonical graph generic cannot represent heterogeneous programs without
erasure and couples serialization to Rust types. Coarse `Float`/`Integer`
markers do not distinguish numerically observable formats. A closed dtype enum
prevents the accepted extension direction. Mandatory authoritative type-level
shapes overconstrain symbolic and parsed graphs, while omitting optional checked
shape evidence misses useful static diagnostics. Implicit promotion hides
semantic choices already required by the numerical contract.

## Traceability

The [IR contract](../ir.md) owns value handles, graph metadata, and verification
boundaries. [Numerical semantics](../numerical-semantics.md) owns resolved
signatures and promotion lowering. The [operation extension contract](../operation-extensions.md)
owns registry authority and durable external type identity.

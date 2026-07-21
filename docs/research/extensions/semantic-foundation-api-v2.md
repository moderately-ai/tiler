---
schema: "tiler-doc/v1"
id: "tiler.research.extensions.semantic-foundation-api-v2"
kind: "research"
title: "Corrected semantic foundation API"
topics: ["extensions", "semantics", "rust", "reference"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "adopted"
implementation_status: "implemented"
evidence_classes: ["executable-model", "primary-source-synthesis"]
informs: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.operation-extensions"]
adopted_by: ["ADR-0065", "ADR-0066"]
ticket: "prototype-semantic-foundation-api-v2"
---

# Corrected semantic foundation API

## Question

Which boundaries exposed by the first semantic/reference implementation must
be corrected before typed handles and compiler passes depend on them?

## Findings

**Fact:** ADRs 0005, 0006, and 0044 require durable operations to use
`OpKey`, canonical attributes, ordered operands/results, and one explicit
semantic-authority registry for built-ins and extensions. The implemented
closed `OperationKind` enum cannot satisfy that extension contract.

**Fact:** ADRs 0059, 0060, and 0062 make Rust markers local checked authoring
capabilities. They do not require every semantic type to have a marker. The
implemented `register_value_type::<T>` couples semantic existence to Rust
ergonomics and cannot naturally admit parsed or generated concrete types.

**Fact:** parameterized constructors and encoded-numeric schemes define
families of concrete resolved types. Exact registration of every instance
turns an open family into an enumerated catalog and makes registry construction
depend on the program that will later be parsed.

**Fact:** the reference evaluator owns host tensors, allocation, execution,
and optional executable capabilities. Those responsibilities consume semantic
IR but are not required to represent or validate it.

**Measurement:** the checked-in four-crate spike compiles with the reference
crate depending only on the IR crate. Its consumer admits `complex<f32>` and an
encoded value backed by `i8` without marker bindings for either concrete type;
it constructs F32 through a generic typed input and executes an operation only
through registered semantic and reference capabilities.

## Adopted correction

One frozen semantic registry contains type-family definitions, operation
definitions, optional marker bindings, and canonical provenance. Registration
callbacks stage a transaction and are discarded. Immutable semantic
definition objects may remain in the frozen snapshot when instance validation
or result inference requires them.

Semantic type authority and marker binding are distinct:

```rust,ignore
registrar.register_type_definition(definition)?;
registrar.bind_marker::<MyF8>(&resolved_type)?;
```

Nominal definitions validate exact keys. Parameterized constructors and
encoded schemes validate concrete bounded instances on demand. Host-owned
schemas canonicalize structure, defaults, bounds, and referenced types;
immutable provider validators may enforce additional deterministic semantic
predicates. Provider outputs are rechecked by the host.

Durable graph operations contain an `OpKey`, canonical operation attributes,
ordered operands, and inferred results. A checked erased `apply` path is the
only mutation primitive. Typed built-in and external facades delegate to it;
providers do not inject Rust methods or declare result types at the call site.

Reference evaluation moves to `tiler-reference`. Its separately frozen
capability registry is keyed by operation identity and resolved signature.
Missing reference knowledge is an explicit capability failure, not a malformed
semantic graph.

## Public API consequences

- Remove the public F32-suffixed builder methods.
- Use `input::<T>` for marker-backed typed authoring and a distinctly named
  checked erased input for parsed frontends.
- Keep operation-specific numerical distinctions explicit; generic spelling
  does not imply universal dtype support.
- Give type arguments, encoded contracts, definition facts, and operation
  attributes distinct public wrappers even when they share canonical storage.
- Keep graph identity provider-independent; ADR 0072 subsequently separates
  reached provider-independent definitions, admission-provider provenance, and
  selected executable capabilities.

## Implementation order

Correct type authority first, then operation authority and generic typed
construction, then extract reference evaluation. Compiler and backend work
remain blocked until the assembled semantic/reference slice passes again.

## Implementation result

The corrected boundaries are now compile-checked in the workspace. `tiler-ir`
owns independently registered semantic type authority, optional Rust marker
bindings, open operation authority, generic typed handles, erased frontend
construction, and deterministic reached-authority projection.
`tiler-reference` depends only on `tiler-ir` and owns host tensors, traversal,
numerical oracle implementations, and a separately frozen exact-signature
capability registry. Governed and external operations pass through the same
semantic and reference registration mechanisms. No F32-specific input or
builder convenience remains in the public semantic builder.

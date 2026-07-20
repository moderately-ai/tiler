---
schema: "tiler-doc/v1"
id: "ADR-0066"
kind: "decision"
title: "Separate semantic type authority from Rust marker bindings"
topics: ["rust", "semantics", "dtypes", "extensions", "registry"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "implemented"
applies_to: ["tiler.contract.ir", "tiler.contract.operation-extensions", "tiler.contract.numerical-semantics"]
evidence: ["tiler.research.extensions.semantic-foundation-api-v2", "tiler.research.extensions.operation-extension-api"]
refines: ["ADR-0044", "ADR-0059", "ADR-0060", "ADR-0062"]
ticket: "prototype-semantic-type-authority-v2"
---

# 0066: Separate semantic type authority from Rust marker bindings

**Status:** accepted

## Context

The first registry implementation made `register_value_type::<T>` the only way
to introduce a type definition. That incorrectly made a process-local Rust
marker mandatory for semantic existence and required exact registration of
every parameterized or encoded concrete type.

## Decision

Semantic authorities register nominal definitions, parameterized constructor
definitions, and encoded-numeric scheme definitions independently of Rust
markers. Concrete parameterized and encoded resolved types are validated on
demand against their registered family definition.

Marker binding is a separate optional registry operation. It may bind a local
marker only to a complete resolved type admitted by the same frozen semantic
snapshot. The existing one-marker/one-type coherence rule remains; `TypeId`,
type names, layouts, and addresses remain process-local lookup data.

The host owns bounded canonical values, structural schemas, default resolution,
referenced-type validation, deterministic ordering, diagnostics, and durable
encoding. Immutable `Send + Sync + 'static` authority objects may validate
additional semantic predicates but cannot mutate canonical identity or bypass
host checks. Registration callbacks stage one atomic batch and are discarded.

Type-constructor arguments, encoded contracts, definition facts, and operation
attributes use distinct public newtypes over shared canonical machinery. A
normative definition reference is likewise a validated identity-bearing type,
not an unconstrained string.

## Consequences

- Parsed and generated programs can use valid concrete types without inventing
  Rust marker declarations.
- Common Rust call sites retain exact `Value<T>` ergonomics through optional
  bindings.
- Novel constructors and encoding schemes extend a governed family rather than
  pre-enumerating every concrete combination.
- Provider semantic validation remains trusted code but its authority,
  determinism, inputs, output, and durable revision are explicit.

## Alternatives considered

Mandatory markers simplify generic lookup but make a language-specific
ergonomic device part of semantic admission. Exact concrete registration is
simple but cannot represent open constructor/scheme families without building
a program-specific registry. Unrestricted provider serialization or callbacks
would weaken canonical identity and validation.

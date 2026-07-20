---
schema: "tiler-doc/v1"
id: "ADR-0044"
kind: "decision"
title: "Use an explicit capability-based operation registry"
topics: ["extensions", "registry", "operations"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.operation-extensions"]
evidence: ["tiler.research.extensions.operation-extension-surface", "tiler.research.extensions.operation-extension-api"]
ticket: "operation-extension-surface"
---

# 0044: Use an explicit capability-based operation registry

**Status:** accepted

## Context

Tiler accepts a public experimental operation-extension boundary. A closed
operation enum would not exercise that requirement, while one monolithic Rust
trait would couple semantic meaning to every optimizer, scheduler, target, and
runtime capability. Automatic linker registration would introduce hidden
global state and still would not make consumer-local providers visible to a
separately compiled proc macro.

Durable semantic identity must also remain independent of Rust implementation
addresses and provider selection.

## Decision

Compiler sessions construct an explicit operation registry and freeze it before
graph verification or optimization. Durable IR stores only operation semantic
keys, host-canonical attributes, and ordered operands/results.

Exactly one semantic authority owns each `OpKey`. It provides a bounded schema,
pure effect declaration, deterministic inference and validation, normative
semantic identity, conformance vectors, and host-readable names. Executable
reference evaluation may be supplied directly or obtained through an exact
verified decomposition; phases that require it reject or conservatively stop
when neither is available.

Decomposition, rewriting, access lowering, physical implementation, kernel
lowering, accuracy evidence, target feasibility, and costing are separately
versioned optional capability providers. Registration never grants a pass more
authority than the capability and proof obligations it queries.

The host owns canonical data encoding, collisions, deterministic ordering,
provider selection, mutation transactions, budgets, stable diagnostics,
identity projection, and reverification. Initial providers are trusted,
statically linked, immutable `Send + Sync + 'static` compiler code.

Semantic graph identity excludes provider implementations. Compilation-request
provenance records the frozen snapshot. Selected plan and artifact identity
include only reached semantic authorities and selected capability providers,
their revisions, and resulting output-affecting content.

## Consequences

- Built-in and external operations use one continuously exercised path.
- Missing optional knowledge is a visible optimization or execution boundary.
- New capability families can be added without expanding one downstream trait.
- Registry setup is explicit for ordinary compiler API users.
- Proc-macro visibility remains a separate feasibility problem.
- Native providers remain a trust boundary; panic containment is not a
  sandbox.

The semantic-authority portion is implemented in `tiler-ir`: bounded
host-owned arity/attribute schemas, canonical facts and conformance identity,
checked inference, transactional graph admission, reached-authority
projection, and multi-result identity all use the same path for built-ins and
external providers. The separately versioned optional capability registries
remain future work.

## Alternatives considered

A closed built-in set is simpler but violates the accepted public extension
goal. One universal operation trait makes unsupported phases ambiguous and
creates semver pressure. Global/linker registration hides inputs and ordering.
Serializing trait objects or Rust type identities would make durable IR and
cache identity build-dependent.

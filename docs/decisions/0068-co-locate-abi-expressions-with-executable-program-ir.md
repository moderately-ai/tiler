---
schema: "tiler-doc/v1"
id: "ADR-0068"
kind: "decision"
title: "Co-locate ABI expressions with executable program IR"
topics: ["program-planning", "abi", "expressions", "rust"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "spike-only"
applies_to: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.artifact-abi"]
evidence: ["tiler.research.program-planning.abi-expression-ownership"]
ticket: "prototype-target-neutral-baseline-slice"
---

# 0068: Co-locate ABI expressions with executable program IR

**Status:** accepted

## Context

`KernelProgram` uses a closed typed expression DAG for applicability guards,
output and temporary sizes, launch geometry, and scalar ABI values. The
artifact envelope must serialize and evaluate the same expressions at runtime.
Owning `KernelProgram` in `tiler-ir` while owning its expression type in
`tiler-artifact` creates a dependency cycle or leaves the program dependent on
an external side table for verification and identity.

`AbiExpr` roots are not generic variables: they name typed program/interface
facts and availability phases. A new lower expression crate would therefore
need to own program-specific identities or weaken them behind generic or opaque
sources before an independent reuse boundary exists.

## Decision

Place the public `AbiExpr` domain type, admitted roots, validation, canonical
identity, and authoritative pure checked evaluation semantics with the
experimental executable-program representations in `tiler-ir`.

`tiler-artifact` owns the versioned wire encoding, compatibility policy,
runtime fact binding and phase enforcement, failure classification, and
backend-payload mappings. `tiler-compiler` owns lowering into and construction
of `AbiExpr`; it is not the runtime expression authority.

`ShapeExpr`, index expressions, and `AbiExpr` remain distinct newtyped domain
IRs. Their implementations may share private checked-arithmetic components
without merging source vocabularies, validation, identity, or versioning.

Do not add a public generic expression crate for the prototype. Reconsider an
extraction only after an independent lower-level consumer, a material measured
build/code-size boundary, or a stable public algebra below the domain-specific
root vocabularies exists.

## Consequences

- A `KernelProgram` remains self-contained and independently verifiable before
  artifact construction.
- Compiler, artifact, backend, and future third-party plan producers share one
  expression meaning without depending on optimizer implementation.
- Artifact deserialization cannot introduce a second editable expression
  authority.
- `tiler-ir` includes a small pure evaluator because evaluation is part of the
  IR contract; runtime orchestration and live fact acquisition remain outside.
- Shared implementation mechanics can be extracted later without changing the
  nominal domain IRs.

## Alternatives considered

Artifact ownership creates a Cargo cycle or an incomplete `KernelProgram`.
Compiler ownership forces runtime dependence on optimizer internals or schema
duplication. A new public expression crate adds a package boundary without a
semantic boundary and pressures program-specific roots into generic types.

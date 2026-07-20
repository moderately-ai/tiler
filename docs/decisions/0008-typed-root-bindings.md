---
schema: "tiler-doc/v1"
id: "ADR-0008"
kind: "decision"
title: "Separate extent symbols from typed root bindings"
topics: ["shapes", "bindings", "semantics"]
decision_status: "accepted"
implementation_status: "spike-only"
applies_to: ["tiler.contract.ir"]
evidence: ["tiler.research.shapes.shape-environment-contract", "tiler.research.shapes.constraint-prover-boundary"]
ticket: "shape-environment-contract"
---

# 0008: Separate extent symbols from typed root bindings

**Status:** accepted

## Context

Logical output extents may depend on static values, input dimensions, explicit
host parameters, or—in deliberately target-parameterized programs—properties
of the selected compilation profile or live device. Encoding each source as a
different arithmetic node couples shape algebra to value provenance. Treating
every source as an indistinguishable parameter instead hides target dependence
from validation, diagnostics, fallback, and artifact identity.

The public semantic graph must remain backend-neutral, but backend-neutrality
does not require every closed program to produce identical shapes on every
target. It requires target dependence to be explicit, typed, and realizable
without embedding runtime device objects in compiler core.

## Decision

`ShapeExpr` references scoped extent symbols. `ShapeEnv` separately maps every
root symbol to one typed binding:

```text
Static
InputDimension
InterfaceParameter
TargetProperty
```

A target-property binding carries a stable versioned key, integer domain, and
binding phase. Initial semantic extents may use properties available from a
compile profile or live-device preflight before allocation and device work.
Properties available only after preparing a selected pipeline are reserved;
allowing them to affect semantic shapes requires a later explicit acyclic
two-phase or fixed-point execution contract.

Operation capabilities declare which transitive binding classes and phases
each semantic factor supports. The reference evaluator, generated fallback,
guards, and compiled path use the same bound semantic environment. Binding
declarations participate in program-interface identity; a concretely
specialized value additionally participates in scheduled/artifact identity.

Physical-only target properties remain inputs to physical planning or
`AbiExpr`. They do not become semantic roots merely because they affect
layout, allocation padding, schedule selection, or launch geometry.
ADR 0043 models those properties as phased typed capability facts. A physical
capability affects semantic shapes or values only through an explicitly
authored `TargetProperty` root binding; scheduling use alone never changes
graph meaning.

Every semantic target property is admitted and bound exactly once from the
declared compile profile or live-device preflight before semantic shape
evaluation and plan routing. Artifact, prepared-kernel, and launch facts cannot
overwrite or refine that semantic environment. Supporting a later source would
require revising this ADR's initial acyclic execution contract.

## Consequences

- Shape arithmetic, canonicalization, and proving remain independent of value
  source mechanisms.
- Target-dependent tensor semantics are expressible and explainable without
  implicit environment queries.
- The same graph can be caller-bound, target-bound, or specialized through
  different explicit interfaces.
- Compiler core stores target-property contracts and binding evidence but does
  not depend on live device objects or backend runtime APIs.
- Target-property support and availability become validation requirements;
  absence cannot silently select a fallback with different semantics.
- The runtime and artifact layers need a versioned binding table and must bind
  semantic roots before output-shape evaluation.

## Alternatives considered

Adding a `TargetParameter` expression node makes provenance visible but embeds
binding mechanisms in the arithmetic IR. Treating target values as ordinary
parameters with an out-of-band or hidden binder preserves a small expression
language but loses source capabilities, diagnostics, and complete identity.
Prohibiting target-derived semantic extents preserves identical visible shapes
across targets but unnecessarily excludes explicitly device-adaptive tensor
programs.

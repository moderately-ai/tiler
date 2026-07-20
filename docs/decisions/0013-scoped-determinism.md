---
schema: "tiler-doc/v1"
id: "ADR-0013"
kind: "decision"
title: "Scope deterministic numerical guarantees"
topics: ["numerics","determinism","reductions"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality"]
ticket: "numerical-policy-contract"
---

# 0013: Scope deterministic numerical guarantees

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

`Deterministic` is ambiguous without stating what changes are held fixed. It
can mean repeatability for one compiled plan, equality across devices, or
portable bitwise equality across backends. These promises have materially
different feasibility and optimization costs.

A deterministic reduction topology can also change after recompilation when
the optimizer, cost model, target profile, or compiler version changes.

## Decision

Tiler does not expose an unqualified deterministic boolean in canonical
contracts. Determinism names an explicit stability scope.

The practical initial guarantee is **plan deterministic**. Given identical
input bits and runtime bindings, the same artifact digest and selected plan
variant, executed in the same declared target environment, produce identical
output bits. A plan claiming this guarantee cannot use timing-dependent
atomics or other runtime choices that may change evaluation.

**Portable bitwise** is a separate, stronger conformance level. It requires
identical output bits across every target conforming to its declared contract.
Backends may emulate operations or reject plans that cannot meet it.

Artifact identity bounds plan determinism. Recompilation can produce another
artifact with a different legal deterministic topology and is not required to
reproduce the old artifact's result unless a stronger contract independently
requires that equality.

The exact fields forming the declared target-environment compatibility identity
remain part of the target and artifact contract work; they must be explicit and
machine-checkable rather than inferred from a marketing device name.

## Consequences

- Tests can hold the promised stability inputs fixed and check exact output
  bits.
- Ordinary deterministic GPU plans need not claim cross-backend bitwise
  portability.
- Portable bitwise behavior can intentionally trade plan freedom and native
  performance for reproducibility.
- Artifact manifests and explain output must state the determinism scope and
  selected topology identity.
- A cache or compiler-version change cannot be mistaken for the same
  plan-deterministic execution context.

## Alternatives considered

A boolean deterministic flag is concise but untestably ambiguous. Defining all
determinism as portable bitwise equality gives a clear result but excludes many
useful target-specific schedules and elementary functions. Defining it only by
live device identity is too weak for artifact validation and reproducibility
because the rest of the execution contract would remain implicit.

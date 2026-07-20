---
schema: "tiler-doc/v1"
id: "ADR-0056"
kind: "decision"
title: "Use four libraries and two proof executables"
topics: ["rust", "workspace", "dependencies", "prototype"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.architecture"]
evidence: ["tiler.research.workspace.prototype-crate-layout-and-msrv"]
ticket: "prototype-foundation-contract"
---

# 0056: Use four libraries and two proof executables

**Status:** accepted

## Context

One crate would scaffold quickly but could let runtime code import optimizer or
backend internals. Creating every future component crate would instead harden
packaging before its APIs have implementation evidence.

## Decision

The authorized prototype uses reusable `tiler-ir`, `tiler-artifact`,
`tiler-compiler`, and `tiler-metal` libraries plus non-published
`prototype-compile` and `prototype-run` executables. Dependencies follow the
exact DAG recorded in the workspace research. The runner depends on the
artifact contract and live Metal bindings, never the compiler.

Multiple target-independent IRs remain modules in `tiler-ir`; compiler passes
remain modules in `tiler-compiler`; MSL emission and AOT invocation remain
modules in `tiler-metal`. No frontend, proc-macro, Candle, generalized cache, or
reusable Metal-runtime crate is created for the first proof.

## Consequences

- Cargo mechanically checks the compiler/runtime and target-neutral boundaries
  that the proof is meant to validate.
- Package count remains smaller than the mature conceptual architecture.
- All APIs remain explicitly unstable; later component splits may preserve
  source compatibility through re-exports when evidence justifies them.
- `tiler-artifact` may use lockstep internal IR types during the prototype but
  may not invoke compiler passes. A durable public wire format requires
  artifact-owned DTOs and checked conversion later.

## Alternatives considered

One core crate plus one executable does not enforce the runtime boundary. A
crate for every IR, frontend, AOT service, and runtime adapter adds ceremony and
suggests stability the prototype does not possess.

## Traceability

The [workspace research](../research/workspace/prototype-crate-layout-and-msrv.md)
derives this layout from the [architecture contract](../architecture.md) and
accepted compiler/artifact/runtime dependency direction.

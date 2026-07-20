---
schema: "tiler-doc/v1"
id: "ADR-0005"
kind: "decision"
title: "Expose a public semantic graph and extension boundary"
topics: ["semantics", "extensions", "api"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.architecture", "tiler.contract.vision", "tiler.contract.operation-extensions"]
evidence: ["tiler.research.semantic-graph.contract-memo", "tiler.research.extensions.operation-extension-surface"]
ticket: "synthesize-core-contracts"
---

# 0005: Expose a public semantic graph and extension boundary

**Status:** accepted

## Context

Tiler is intended to be a tensor compiler toolkit rather than an optimizer
owned by einops, Candle, Metal, a Rust macro, or any other initial consumer.
Frontends need one common representation that lets arbitrary tensor languages
submit complete computation graphs to shared optimization and code generation.

## Decision

Tiler's primary public input is an experimental, backend-neutral semantic
tensor graph. Frontends lower their syntax and APIs into this graph. The graph
describes a function over explicit inputs and extent symbols; a typed semantic
interface declares how root symbols are bound when a program is compiled or
executed. Target-dependent semantics are permitted only through those explicit
bindings, as refined by ADR 0008. Compiler passes, target backends, artifact
packaging, and runtime adapters consume later verified representations without
depending on the originating frontend or consumer runtime.

The public API includes an experimental vertical operation-extension contract.
Built-in and third-party tensor operations use the same extension path, with
explicit invariants even where capabilities are initially reserved or
unsupported. The exact decomposition into semantic inference, verification,
identity, reference behavior, rewriting, iteration/access lowering, physical
implementation, and explanation traits remains proposed and may evolve.

## Consequences

- Einops and Candle are initial validation integrations, not compiler-core
  abstractions.
- Metal AOT and inline macro delivery may impose integration-specific
  constraints without defining every frontend or backend workflow.
- The semantic graph and extension APIs are public while experimental; their
  early availability is not a promise of immediate long-term compatibility.
- Adding an officially supported operation should exercise the same extension
  path available to external dialects.
- Compiler-core crates cannot depend on frontend syntax, runtime tensor
  objects, or target device objects.
- Backend-neutral does not mean that every closed program has identical
  observable shapes on every target. A program may explicitly bind a semantic
  parameter from a versioned target property while keeping target queries and
  device objects outside the graph's arithmetic IR.

## Alternatives considered

Building the compiler directly around candle-einops operations or Candle tensor
types would simplify one demonstration but make the toolkit consumer-specific.
A private fixed IR would postpone the extension problem and fail to test that
new operations can receive vertical support without architectural surgery.

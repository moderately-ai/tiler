# 0005: Expose a public semantic graph and extension boundary

**Status:** accepted

## Context

Tiler is intended to be a tensor compiler toolkit rather than an optimizer
owned by einops, Candle, Metal, a Rust macro, or any other initial consumer.
Frontends need one common representation that lets arbitrary tensor languages
submit complete computation graphs to shared optimization and code generation.

## Decision

Tiler's primary public input is an experimental, target-independent semantic
tensor graph. Frontends lower their syntax and APIs into this graph. Compiler
passes, target backends, artifact packaging, and runtime adapters consume later
verified representations without depending on the originating frontend or
consumer runtime.

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

## Alternatives considered

Building the compiler directly around candle-einops operations or Candle tensor
types would simplify one demonstration but make the toolkit consumer-specific.
A private fixed IR would postpone the extension problem and fail to test that
new operations can receive vertical support without architectural surgery.

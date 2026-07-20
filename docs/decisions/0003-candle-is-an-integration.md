---
schema: "tiler-doc/v1"
id: "ADR-0003"
kind: "decision"
title: "Keep the compiler independent of Candle"
topics: ["candle", "integrations", "architecture"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.candle-integration"]
evidence: ["tiler.research.runtime.execution-contract"]
---

# 0003: Keep the compiler independent of Candle

**Status:** accepted

## Context

`candle-einops` and Candle Metal motivate the first implementation, but tensor
semantics, index algebra, fusion, scheduling, and MSL emission do not require
Candle storage types. Coupling them would limit reuse and force compiler tooling
to depend on runtime internals.

## Decision

Compiler and backend components will not depend on Candle. A separate adapter
translates Candle `Storage` and `Layout` into versioned artifact bindings,
allocates outputs, evaluates guards, encodes dispatches, and supplies fallback.

## Consequences

- Other frontends and runtimes can reuse the compiler.
- The artifact ABI becomes a real, verified boundary.
- Candle layout and autograd behavior remain localized.
- Some metadata translation is required at integration time.

## Alternatives considered

Embedding Candle types in semantic IR would simplify the first demo but mix
runtime allocation/layout behavior with compile-time tensor meaning.

## Traceability

Applies to the Candle adapter contract and is supported by the consumer-neutral
runtime execution research.

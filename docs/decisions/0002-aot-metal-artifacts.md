---
schema: "tiler-doc/v1"
id: "ADR-0002"
kind: "decision"
title: "Generate Metal artifacts ahead of time"
topics: ["metal", "aot", "artifacts"]
catalog_group: "artifacts-build-toolchains"
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.artifact-abi", "tiler.contract.metal-backend", "tiler.contract.frontend-integration"]
evidence: ["tiler.research.apple-targets.compatibility"]
---

# 0002: Generate Metal artifacts ahead of time

**Status:** accepted

## Context

The initial frontend knows the operation graph and rank relationships during a
Rust build. Runtime source compilation would add latency, deployment concerns,
and cache complexity without being required for runtime dimensions and strides.

## Decision

Tiler's initial Metal path will emit MSL and compile metallibs during proc-macro
expansion.
Runtime values such as extents, strides, and offsets remain typed ABI metadata.
A small number of schedule variants may be selected with guards and Metal
function constants. Runtime creates and caches pipeline objects from compiled
artifacts but does not compile MSL source.

## Consequences

- Runtime startup and deployment do not require a source compiler.
- Expansion tooling must handle macOS toolchain availability, content caching,
  cross-process locking, deterministic identity, direct embedding, and
  source-spanned diagnostics.
- Exact-shape specialization must be controlled to prevent artifact explosion.
- Runtime variant selection and fallback remain necessary.

## Alternatives considered

Runtime JIT offers shape-specific specialization but is not justified for the
initial use case. Shipping only handwritten kernels cannot express arbitrary
known frontend compositions.

## Traceability

Applies to the artifact, Metal backend, and frontend contracts; bounded Apple
compatibility research is the current implementation evidence.

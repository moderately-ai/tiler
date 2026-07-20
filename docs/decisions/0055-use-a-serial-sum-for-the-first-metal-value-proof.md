---
schema: "tiler-doc/v1"
id: "ADR-0055"
kind: "decision"
title: "Use a serial Sum for the first Metal value proof"
topics: ["prototype", "metal", "fusion", "reductions"]
catalog_group: "physical-planning-lowering"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.architecture", "tiler.contract.numerical-semantics", "tiler.contract.fusion-and-scheduling", "tiler.contract.metal-backend", "tiler.contract.correctness-and-testing"]
evidence: ["tiler.research.numerics.reduction-semantics-and-legality", "tiler.research.scheduling.scheduled-region-model", "tiler.research.kernel-ir.structured-kernel-ir-verifier"]
ticket: "research-readiness-gate"
---

# 0055: Use a serial Sum for the first Metal value proof

**Status:** accepted

## Context

A reduction-free reindex/pointwise kernel would validate the compilation and
runtime plumbing, but it would weakly demonstrate why Tiler exists. A narrow
map/reduce chain can prove that the architecture eliminates both a materialized
intermediate and a dispatch while exercising semantic order, scheduling,
structured-kernel refinement, artifact generation, and guarded execution.

Beginning with a broad parallel reduction would instead introduce SIMD-group,
threadgroup, reassociation, partial-state, barrier, and multi-pass questions
before the vertical boundaries have been implemented.

## Decision

The first Metal-backed value proof uses the already-researched, deliberately
narrow strict serial `f32` `Sum` profile. One thread owns each output and visits
the canonical contributor sequence in semantic order. The fused region admits a
small resolved `f32` pointwise prologue, such as multiply and add, without an
observable low-precision materialization boundary.

The proof compares one fused kernel against a deliberately materialized
pointwise-plus-reduction reference. It must demonstrate equal reference results
under the strict contract and fewer dispatches or intermediates. It does not
claim a performant general reduction implementation.

The versioned first-profile arithmetic-NaN contract uses the canonical quiet
binary32 pattern `0x7fc00000`. Constants and bit-preserving values retain their
declared payload bits. `f32` Multiply and Add canonicalize every produced NaN;
strict Sum applies that Add rule after every combine and canonicalizes again at
the reduction result boundary, including singleton results. This policy is
part of semantic, plan, artifact, and cache identity rather than an ambient
host or Metal behavior.

This decision authorizes a bounded, explicitly unstable implementation
prototype. It does not authorize stable public APIs, broad operation coverage,
parallel reductions, runtime JIT, or a production Metal backend.

## Consequences

- The prototype exercises every architectural layer with a useful fusion case.
- Serial execution supplies one unambiguous numerical baseline and avoids
  premature reassociation or permutation permissions.
- Performance is not the first success criterion; architectural correctness and
  removal of a materialization/dispatch are.
- Crate layout and MSRV must be resolved before scaffolding, then implementation
  proceeds through dependency-ordered vertical-slice tickets.

## Alternatives considered

A reduction-free pointwise proof is simpler but provides weaker evidence of
Tiler's optimizer value. A parallel or multi-pass reduction is more performant
but expands the numerical, synchronization, resource, and verification surface
too early. A handwritten Metal-only kernel would not validate the compiler
boundaries.

## Traceability

The [reduction contract](../research/numerics/reduction-semantics-and-legality.md)
defines the proposed serial baseline. The
[scheduled-region](../research/scheduling/scheduled-region-model.md) and
[structured-kernel](../research/kernel-ir/structured-kernel-ir-verifier.md)
reports define its physical and lowering proof boundaries.

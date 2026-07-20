---
schema: "tiler-doc/v1"
id: "tiler.portal.research"
kind: "portal"
title: "Research catalog"
topics: ["research", "evidence"]
---

# Research catalog

Research reports provide evidence and rationale; they are not normative merely
because a ticket is complete. `Disposition` states how current contracts use a
report, while `Evidence` states what kind of support it provides.

<!-- BEGIN GENERATED RESEARCH CATALOG -->
### Foundation, semantics, and extensions

- [Experimental operation API sketch](../research/extensions/operation-extension-api.md) — partially-adopted; executable-model
- [Normative reference evaluator slice](../research/reference/normative-reference-slice.md) — adopted; executable-model
- [Operation-extension surface research](../research/extensions/operation-extension-surface.md) — adopted; primary-source-synthesis, executable-model
- [Proc-macro visibility of operation extensions](../research/extensions/proc-macro-extension-visibility.md) — adopted; primary-source-synthesis, bounded-measurement
- [Semantic tensor graph contract research memo](../research/semantic-graph/contract-memo.md) — adopted; primary-source-synthesis
- [Shape constraint prover boundary](../research/shapes/constraint-prover-boundary.md) — adopted; primary-source-synthesis, executable-model
- [Shape environment contract research memo](../research/shapes/shape-environment-contract.md) — adopted; primary-source-synthesis, executable-model
- [Symbolic index and access model](../research/indexing/index-access-model.md) — adopted; primary-source-synthesis, executable-model

### Numerical operations

- [Floating-point extrema precedents](../research/numerics/floating-point-extrema-precedents.md) — adopted; primary-source-synthesis
- [Floating-point to integer conversion precedents](../research/numerics/float-to-integer-conversion-precedents.md) — adopted; primary-source-synthesis
- [Initial operation conformance matrix](../research/numerics/operation-conformance-matrix.md) — adopted; primary-source-synthesis
- [Integer arithmetic overflow precedents](../research/numerics/integer-overflow-precedents.md) — adopted; primary-source-synthesis
- [Integer division and remainder precedents](../research/numerics/integer-division-precedents.md) — adopted; primary-source-synthesis
- [Reduction semantics and legality](../research/numerics/reduction-semantics-and-legality.md) — partially-adopted; primary-source-synthesis, executable-model
- [Region accuracy contracts and analyzable error budgets](../research/numerics/region-accuracy-contract.md) — partially-adopted; primary-source-synthesis, bounded-measurement
- [Sound region-accuracy analyzer integration spike](../research/numerics/sound-region-analyzer-spike.md) — informational; primary-source-synthesis, sound-proof, bounded-measurement
- [Transcendental accuracy precedents](../research/numerics/transcendental-accuracy-precedents.md) — adopted; primary-source-synthesis

### Dtypes and quantization

- [Affine quantization numerical semantics](../research/numerics/affine-quantization-semantics.md) — adopted; primary-source-synthesis
- [Dtype identity admission policy](../research/numerics/dtype-identity-admission-policy.md) — adopted; primary-source-synthesis
- [Dtype resolution and mixed-precision precedent](../research/numerics/dtype-resolution-precedents.md) — adopted; primary-source-synthesis
- [Mature tensor dtype taxonomy](../research/numerics/mature-dtype-taxonomy.md) — partially-adopted; primary-source-synthesis
- [Quantization representation in tensor IRs](../research/numerics/quantization-ir-precedents.md) — adopted; primary-source-synthesis
- [Quantized value and transformation contract](../research/numerics/quantized-value-and-transform-contract.md) — adopted; primary-source-synthesis

### Physical planning and lowering

- [Exhaustive fusion-region oracle](../research/region-search/exhaustive-region-oracle.md) — adopted; exhaustive-finite, executable-model
- [First-class scheduled-region model](../research/scheduling/scheduled-region-model.md) — adopted; primary-source-synthesis, executable-model
- [Initial cost model and calibration plan](../research/cost-model/bootstrap-cost-model.md) — adopted; executable-model
- [KernelProgram and conservative buffer planning](../research/program-planning/kernel-program-buffer-plan.md) — adopted; primary-source-synthesis, executable-model
- [Structured kernel IR and verifier boundary](../research/kernel-ir/structured-kernel-ir-verifier.md) — adopted; primary-source-synthesis, executable-model
- [Target profiles and phased physical feasibility](../research/target-profiles/physical-feasibility-model.md) — adopted; primary-source-synthesis, executable-model

### Artifacts, build, and toolchains

- [Apple Metal artifact compatibility](../research/apple-targets/artifact-compatibility.md) — partially-adopted; primary-source-synthesis, bounded-measurement
- [Direct embedded-artifact costs across Rust crates](../research/embedding/embedded-artifact-costs.md) — partially-adopted; bounded-measurement
- [Expansion cache crash and race protocol](../research/cache/crash-and-race-protocol.md) — adopted; primary-source-synthesis, executable-model, bounded-measurement
- [Proc-macro build environment and freshness](../research/macro-environment/proc-macro-build-environment.md) — adopted; primary-source-synthesis, bounded-measurement
- [Target-neutral artifact and backend payload envelope](../research/artifacts/target-neutral-artifact-envelope.md) — adopted; primary-source-synthesis, executable-model

### Runtime, integration, and placement

- [Candle Metal post-wait error checking](../research/runtime/candle-metal-post-wait-error-checking.md) — partially-adopted; primary-source-synthesis, executable-model
- [Consumer-neutral runtime execution contract](../research/runtime/runtime-execution-contract.md) — adopted; primary-source-synthesis, executable-model
- [Device placement and memory-domain contract](../research/placement/device-placement-and-memory-domains.md) — adopted; primary-source-synthesis, executable-model
- [Semantic validation enforcement](../research/runtime/semantic-validation-enforcement.md) — adopted; primary-source-synthesis, executable-model
- [Transfer synchronization and resource-lifetime contract](../research/transfers/transfer-synchronization-and-resource-lifetime.md) — adopted; primary-source-synthesis, executable-model

### Documentation governance

- [Information architecture and provenance audit](../research/documentation/information-architecture-audit.md) — adopted; primary-source-synthesis, bounded-measurement
<!-- END GENERATED RESEARCH CATALOG -->

Use the linked report for exact environment, source revision, limitations, and
remaining unknowns. Reproduction entry points are indexed separately in the
[experiment catalog](../../spikes/README.md).

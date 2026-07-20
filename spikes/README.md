---
schema: "tiler-doc/v1"
id: "tiler.portal.experiments"
kind: "portal"
title: "Experiment catalog"
topics: ["experiments", "evidence"]
---

# Experiment catalog

Spikes are preserved executable evidence for bounded questions. They are not a
production implementation or a universal guarantee. `Reproducible` means the
checked-in procedure is complete under its recorded prerequisites; it does not
mean dependency-free, hermetic, portable to every host, or rerun by CI.

<!-- BEGIN GENERATED EXPERIMENT CATALOG -->
### Foundation, semantics, and extensions

- [Index and access-model experiment](indexing/README.md) — reproducible; executable-model; supports: [Symbolic index and access model](../docs/research/indexing/index-access-model.md)
- [Normative reference-evaluator experiment](reference/README.md) — reproducible; executable-model; supports: [Normative reference evaluator slice](../docs/research/reference/normative-reference-slice.md)
- [Operation-extension experiments](extensions/README.md) — reproducible; executable-model, bounded-measurement; supports: [Operation-extension surface research](../docs/research/extensions/operation-extension-surface.md), [Experimental operation API sketch](../docs/research/extensions/operation-extension-api.md), [Proc-macro visibility of operation extensions](../docs/research/extensions/proc-macro-extension-visibility.md)
- [Semantic foundation API v2 compile-checking spike](extensions/semantic-foundation-api-v2/README.md) — reproducible; executable-model; supports: [Corrected semantic foundation API](../docs/research/extensions/semantic-foundation-api-v2.md)
- [Stable-Rust shape-evidence feasibility spike](shapes/shape-evidence/README.md) — reproducible; executable-model, bounded-measurement; supports: [Stable-Rust shape-evidence feasibility](../docs/research/shapes/stable-rust-shape-evidence.md)

### Numerical operations

- [Reduction contract probe](numerics/reduction_contract/README.md) — reproducible; executable-model, exhaustive-finite; supports: [Reduction semantics and legality](../docs/research/numerics/reduction-semantics-and-legality.md)
- [Region accuracy observation probe](numerics/region_accuracy/README.md) — reproducible; bounded-measurement; supports: [Region accuracy contracts and analyzable error budgets](../docs/research/numerics/region-accuracy-contract.md)
- [Sound accuracy probe](numerics/sound_accuracy/README.md) — reproducible; sound-proof, bounded-measurement; supports: [Sound region-accuracy analyzer integration spike](../docs/research/numerics/sound-region-analyzer-spike.md), [Region accuracy contracts and analyzable error budgets](../docs/research/numerics/region-accuracy-contract.md)

### Physical planning and lowering

- [Bootstrap cost-model experiment](cost-model/README.md) — reproducible; executable-model; supports: [Initial cost model and calibration plan](../docs/research/cost-model/bootstrap-cost-model.md)
- [Exhaustive fusion-region oracle experiment](region-search/README.md) — reproducible; exhaustive-finite, executable-model; supports: [Exhaustive fusion-region oracle](../docs/research/region-search/exhaustive-region-oracle.md)
- [Kernel-program planning experiment](program-planning/README.md) — reproducible; executable-model; supports: [KernelProgram and conservative buffer planning](../docs/research/program-planning/kernel-program-buffer-plan.md)
- [Scheduled-region model experiment](scheduling/README.md) — reproducible; executable-model; supports: [First-class scheduled-region model](../docs/research/scheduling/scheduled-region-model.md)
- [Structured kernel-IR verifier experiment](kernel-ir/README.md) — reproducible; executable-model; supports: [Structured kernel IR and verifier boundary](../docs/research/kernel-ir/structured-kernel-ir-verifier.md)

### Artifacts, build, and toolchains

- [Apple Metal target compatibility spikes](apple-targets/README.md) — reproducible; bounded-measurement; supports: [Apple Metal artifact compatibility](../docs/research/apple-targets/artifact-compatibility.md)
- [Artifact envelope spike](artifacts/README.md) — reproducible; executable-model; supports: [Target-neutral artifact and backend payload envelope](../docs/research/artifacts/target-neutral-artifact-envelope.md)
- [Embedded-artifact cost probe](embedding/README.md) — reproducible; bounded-measurement; supports: [Direct embedded-artifact costs across Rust crates](../docs/research/embedding/embedded-artifact-costs.md)
- [Expansion cache crash and race spike](cache/README.md) — reproducible; executable-model, bounded-measurement; supports: [Expansion cache crash and race protocol](../docs/research/cache/crash-and-race-protocol.md)
- [Proc-macro environment and artifact-family spikes](macro-environment/README.md) — reproducible; bounded-measurement; supports: [Proc-macro build environment and freshness](../docs/research/macro-environment/proc-macro-build-environment.md)

### Runtime, integration, and placement

- [Placement and memory-domain model](placement/README.md) — reproducible; executable-model; supports: [Device placement and memory-domain contract](../docs/research/placement/device-placement-and-memory-domains.md)
- [Runtime execution and validation spikes](runtime/README.md) — reproducible; executable-model; supports: [Consumer-neutral runtime execution contract](../docs/research/runtime/runtime-execution-contract.md), [Semantic validation enforcement](../docs/research/runtime/semantic-validation-enforcement.md), [Candle Metal post-wait error checking](../docs/research/runtime/candle-metal-post-wait-error-checking.md)
- [Transfer synchronization and lifetime model](transfers/README.md) — reproducible; executable-model; supports: [Transfer synchronization and resource-lifetime contract](../docs/research/transfers/transfer-synchronization-and-resource-lifetime.md)

### Documentation governance

- [Documentation integrity gate](documentation/README.md) — reproducible; executable-model; supports: [Information architecture and provenance audit](../docs/research/documentation/information-architecture-audit.md), [Blank-agent documentation acceptance audit](../docs/research/documentation/blank-agent-acceptance-audit.md)
<!-- END GENERATED EXPERIMENT CATALOG -->

Each experiment entry identifies its supported research claim, exact entry
point, prerequisites, retained results, and measurement boundary. Generated
local caches remain ignored; cited fixtures remain tracked.

---
schema: "tiler-doc/v1"
id: "tiler.portal.experiments"
kind: "portal"
title: "Experiment catalog"
topics: ["experiments", "evidence"]
---

# Experiment catalog

Spikes are preserved executable evidence for bounded questions. They are not a
production implementation or a universal guarantee.

<!-- BEGIN GENERATED EXPERIMENT CATALOG -->
### Foundation, semantics, and extensions

- [Index and access-model experiment](indexing/README.md) — reproducible; executable-model
- [Normative reference-evaluator experiment](reference/README.md) — reproducible; executable-model
- [Operation-extension experiments](extensions/README.md) — reproducible; executable-model, bounded-measurement

### Numerical operations

- [Reduction contract probe](numerics/reduction_contract/README.md) — reproducible; executable-model, exhaustive-finite
- [Region accuracy observation probe](numerics/region_accuracy/README.md) — reproducible; bounded-measurement
- [Sound accuracy probe](numerics/sound_accuracy/README.md) — reproducible; sound-proof, bounded-measurement

### Physical planning and lowering

- [Bootstrap cost-model experiment](cost-model/README.md) — reproducible; executable-model
- [Exhaustive fusion-region oracle experiment](region-search/README.md) — reproducible; exhaustive-finite, executable-model
- [Kernel-program planning experiment](program-planning/README.md) — reproducible; executable-model
- [Scheduled-region model experiment](scheduling/README.md) — reproducible; executable-model
- [Structured kernel-IR verifier experiment](kernel-ir/README.md) — reproducible; executable-model

### Artifacts, build, and toolchains

- [Apple Metal target compatibility spikes](apple-targets/README.md) — reproducible; bounded-measurement
- [Artifact envelope spike](artifacts/README.md) — reproducible; executable-model
- [Embedded-artifact cost probe](embedding/README.md) — reproducible; bounded-measurement
- [Expansion cache crash and race spike](cache/README.md) — reproducible; executable-model, bounded-measurement
- [Proc-macro environment and artifact-family spikes](macro-environment/README.md) — reproducible; bounded-measurement

### Runtime, integration, and placement

- [Placement and memory-domain model](placement/README.md) — reproducible; executable-model
- [Runtime execution and validation spikes](runtime/README.md) — reproducible; executable-model, bounded-measurement
- [Transfer synchronization and lifetime model](transfers/README.md) — reproducible; executable-model
<!-- END GENERATED EXPERIMENT CATALOG -->

Each experiment entry identifies its supported research claim, exact entry
point, prerequisites, retained results, and measurement boundary. Generated
local caches remain ignored; cited fixtures remain tracked.

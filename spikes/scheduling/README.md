---
schema: "tiler-doc/v1"
id: "tiler.spike.scheduling"
kind: "experiment"
title: "Scheduled-region model experiment"
topics: ["scheduling", "ir", "gpu"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.scheduling.scheduled-region-model"]
entrypoints: ["spikes/scheduling/scheduled_region_model.rs"]
last_verified: "2026-07-20"
ticket: "scheduled-region-model"
---

# Scheduled-region model experiment

This dependency-free Rust model checks normalized execution mappings,
ownership, tails, staging, barriers, reductions, launch expressions, and stable
identity for representative regions.

Run from the repository root:

```sh
rustc --edition 2021 --test spikes/scheduling/scheduled_region_model.rs -o /tmp/tiler-schedule
/tmp/tiler-schedule
```

It models the common schedule contract without claiming Metal, CUDA, or CPU
backend completeness.

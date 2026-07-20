---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning"
kind: "experiment"
title: "Kernel-program planning experiment"
topics: ["program-planning", "buffers", "scheduling"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.program-planning.kernel-program-buffer-plan"]
entrypoints: ["spikes/program-planning/kernel_program_model.rs"]
last_verified: "2026-07-20"
ticket: "kernel-program-buffer-plan"
---

# Kernel-program planning experiment

This dependency-free Rust model checks a single-device stage DAG, conservative
buffer lifetimes and reuse handoffs, host-preflight expressions, named outputs,
and rejection after routing commit.

Run from the repository root:

```sh
rustc --edition 2021 --test spikes/program-planning/kernel_program_model.rs -o /tmp/tiler-program-plan
/tmp/tiler-program-plan
```

It deliberately excludes asynchronous timelines, suballocation, and
multi-device execution.

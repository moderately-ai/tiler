---
schema: "tiler-doc/v1"
id: "tiler.spike.kernel-ir"
kind: "experiment"
title: "Structured kernel-IR verifier experiment"
topics: ["kernel-ir", "verification", "scheduling"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.kernel-ir.structured-kernel-ir-verifier"]
entrypoints: ["spikes/kernel-ir/structured_kernel_ir.rs"]
last_verified: "2026-07-20"
ticket: "structured-kernel-ir-verifier"
---

# Structured kernel-IR verifier experiment

The dependency-free Rust model checks type/effect rules and schedule-linked
bounds, ownership, convergence, reduction-order, conversion, and launch
evidence.

Run from the repository root:

```sh
rustc --edition 2021 --test spikes/kernel-ir/structured_kernel_ir.rs -o /tmp/tiler-kernel-ir
/tmp/tiler-kernel-ir
```

Passing proves only the finite modeled cases, not arbitrary lowering
refinement or backend acceptance.

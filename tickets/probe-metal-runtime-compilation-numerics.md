---
id: probe-metal-runtime-compilation-numerics
title: Probe Metal runtime-compilation numerics in the checked-in harness
status: todo
priority: p2
dependencies: []
related: [check-in-apple-numerical-behaviour-probe]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, measurement]
---
`spikes/apple-targets/numerical_probe.py` compiles every probe offline through `xcrun metal` and `xcrun metallib`. The outcome of `prototype-metal-numerical-realization` additionally reported that the subnormal flush is identical through runtime compilation — `newLibraryWithSource:options:` with `MTLCompileOptions.mathMode` set to `MTLMathModeSafe`/`Relaxed`/`Fast` — on the same device. That observation is not re-established by the checked-in harness and `docs/research/apple-targets/numerical-behaviour.md` records it as an explicit measurement boundary rather than a claim.

The gap matters because the two paths reach the driver through different front ends. If a future toolchain diverges between them, the offline row would stay green while the runtime row changed, and nothing would notice. It also bounds a question ADR 0076 leaves open about where a delivered realization can be read from.

Add a runtime-compilation mode to `numerical_probe_host.m` that takes MSL source and an `MTLMathMode` instead of a linked metallib, run the same kernels and the same execution-witness guard through it, and record the comparison in the retained record so a divergence between the offline and runtime paths fails the gate. Keep the existing self-skip classification: a host with no `MTLCreateSystemDefaultDevice` still skips.

Note that `MTLCompileOptions` exposes no `-ffp-contract` equivalent, so the contraction findings may not be expressible on the runtime path. If so, record that as a precise limitation rather than working around it.

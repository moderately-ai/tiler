---
id: scope-launch-granularity-optimization-for-the-decode-dominated-regime
title: Scope launch-granularity optimization for the decode-dominated regime
status: todo
priority: p3
dependencies: [prototype-metal-runtime-proof]
related: [decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode]
scopes: [research/program-planning, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [research, performance, runtime, trigger-fired]
---
## User-visible outcome

The launch economics that dominate LLM decode — many small kernels, per-launch overhead, cross-stage synchronization — become a scoped optimization surface: persistent/mega-kernel forms, command-buffer reuse, and program-level fusion across stages, decided at the kernel-program layer where the stage DAG already lives.

## Why this exists, and what the fired trigger now permits

**Fact.** The decoder-layer assembly measured 62 occurrences per decode step across three programs executed 28× per token; the kernel-program model owns the stage DAG and cross-stage dependencies. **Inference.** At decode batch sizes the per-launch cost plausibly exceeds any single kernel's arithmetic, making launch granularity worth more than further intra-kernel optimization — but that is an inference, and the scoping's first obligation is the measurement that confirms or refutes it (launch overhead on the Metal runtime, measured on the designated host, against a decode step's actual kernel population). Literature when fired: persistent-kernel and megakernel work, CUDA Graphs as the cross-vendor precedent for replayable command capture, and Metal's own indirect command buffers as the platform primitive. Deferred behind the multi-layer execution work: scoping launch fusion before a multi-stage program executes end-to-end would scope against a simulation.

## Trigger

A multi-stage program executing end-to-end through the runtime (the contraction vertical generalizing past single-member dispatch, or the model-level execution thread reaching a multi-layer run).

## Trigger check log

- 2026-08-05 — **not fired.** The runtime dispatches single proof members; no multi-stage end-to-end execution exists. Recheck: `grep -rn "prove_contraction\|proof_member" prototypes/serial-sum-run/src/proof.rs | head -3` still showing the per-member proof shape as the only route.
- 2026-08-09 — **fired.** `prototype-metal-runtime-proof` is `done` and records the first device execution of the multi-stage path: the materialized program ran two dispatches over one shared allocation and agreed bit-for-bit with the selected one-dispatch program across the proof matrix. That is an end-to-end multi-stage runtime subject, so this scoping work is now runnable. The decode-dominated claim remains an inference to measure; the proof fires the trigger without supplying the decoder-layer measurement.

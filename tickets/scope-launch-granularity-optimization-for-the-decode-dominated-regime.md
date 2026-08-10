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

The launch economics that dominate LLM decode — many small kernels, per-launch overhead, cross-stage synchronization — become a scoped optimization surface: fused / multi-stage program forms, command-buffer reuse, and program-level fusion across stages, decided at the kernel-program layer where the stage DAG already lives. (Literature may still say "mega-kernel" conversationally; prefer fused kernel / fusion region in normative interfaces per the glossary.)

## Why this exists, and what the fired trigger now permits

**Fact.** The decoder-layer assembly measures **62 semantic occurrences** at the C1 decode row (`T = 1`; 58 at prefill `T = 10`) in `crates/tiler-reference/tests/decoder_layer.rs`. Whole-model composition is **three programs** (P1, P2, P3) with **30 executions per forward pass** — P2 runs **28×** (once per layer); P1 and P3 once each. Occurrences are not kernel launches. **Fact.** The kernel-program model owns the stage DAG and cross-stage dependencies. **Inference.** At decode batch sizes the per-launch cost plausibly exceeds any single kernel's arithmetic, making launch granularity worth more than further intra-kernel optimization — but that is an inference, and the scoping's first obligation is the measurement that confirms or refutes it (launch overhead on the Metal runtime, measured on the designated host, against a decode step's actual kernel population). Literature when fired: persistent-kernel and megakernel work, CUDA Graphs as the cross-vendor precedent for replayable command capture, and Metal's own indirect command buffers as the platform primitive. Deferred behind a multi-stage end-to-end runtime subject (contraction vertical past single-member dispatch, or model-level multi-layer run): scoping launch fusion before any multi-stage program executes end-to-end would scope against a simulation.

**Correction — 2026-08-10.** An earlier compound Fact read "62 occurrences per decode step across three programs executed 28× per token." That glued three distinct quantities: (1) one P2 layer graph's occurrence count at decode `T = 1` (62), (2) the three-program whole-model partition, and (3) P2's 28× layer loop. Only P2 runs 28×; P1 and P3 run once each (30 executions per forward pass). Occurrences are not launches.

## Required analysis

- **Measure launch economics on a stated subject before recommending fusion policy.** Name host, procedure, and subject: serial-sum multi-stage is available now; decoder-layer and full decode-step populations when those paths exist. Confirm or refute the Inference that per-launch overhead dominates single-kernel arithmetic at decode batch sizes.
- **Name the optimization surface at the kernel-program layer.** What is in and out among fused / multi-stage program forms, command-buffer reuse (including Metal indirect command buffers), and cross-stage fusion — and what identity, feasibility, and ABI consequences each option carries.
- **Separate hard feasibility from estimated cost.** Impossible multi-stage launch plans must fail clearly; cost ranking is only among valid plans under the numerical contract and target profile.
- **End in one of the four research outcomes.** A contract update, an accepted decision, a bounded experiment, or an explicitly deferred question with a reconsideration trigger. Product prose alone is not an outcome.

## Non-goals

Implementing persistent kernels, Metal ICB capture, or cross-stage fusion. Changing decoder-layer assembly, whole-model partition, or runtime multi-entry ABI. Treating occurrence counts as launch counts. Replacing the serial-sum multi-stage subject with a simulated multi-layer model run as the first measurement host.

## Closes when

(a) A launch-overhead measurement plan (host, procedure, subject) exists and either confirms or refutes the Inference against a stated kernel population — serial-sum multi-stage now; decoder-layer / full decode step when available. (b) The kernel-program-layer optimization surface is named with in/out for fused multi-stage forms, command-buffer reuse / Metal ICB, and cross-stage fusion, plus identity and feasibility consequences. (c) Remainder work is filed as tickets or explicit deferrals with triggers. Exit is a contract update, accepted decision, bounded experiment, or deferred question — not only an open note.

## Trigger

A multi-stage program executing end-to-end through the runtime (the contraction vertical generalizing past single-member dispatch, or the model-level execution thread reaching a multi-layer run).

## Trigger check log

- 2026-08-05 — **not fired.** The runtime dispatches single proof members; no multi-stage end-to-end execution exists. Recheck: `grep -rn "prove_contraction\|proof_member" prototypes/serial-sum-run/src/proof.rs | head -3` still showing the per-member proof shape as the only route.
- 2026-08-09 — **fired.** `prototype-metal-runtime-proof` is `done` and records the first device execution of the multi-stage path: the materialized program ran two dispatches over one shared allocation and agreed bit-for-bit with the selected one-dispatch program across the proof matrix. That is an end-to-end multi-stage runtime subject, so this scoping work is now runnable. The decode-dominated claim remains an inference to measure; the proof fires the trigger without supplying the decoder-layer measurement.

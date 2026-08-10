---
id: integrate-the-attention-block-into-the-runtime
title: Run one complete attention block end to end and compare it with the reference
status: todo
priority: p1
dependencies: [plan-the-materialized-attention-decomposition, integrate-the-contraction-vertical-into-the-runtime, reclassify-language-model-work-as-a-conformance-track]
related: [design-attention-program-vertical, retain-the-c1-attention-block-conformance-evidence, design-autoregressive-state-and-kv-cache, retain-the-qwen-conformance-reference-logit-fixture, prototype-metal-runtime-proof]
scopes: [implementation/runtime, implementation/metal-aot, implementation/artifact, implementation/compiler, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, artifacts, attention, transformer, language-model, class-conformance-fixture]
---
## User-visible outcome

**Rung L4's stated capability becomes true:** one complete causal self-attention block of the pinned workload compiles through the accepted AOT route, executes on a Metal device, and its result is compared with the normative reference. This is the ticket the whole L4 delivery graph exists to reach, and it is the first time a transformer block of any kind runs on a Metal device through the accepted AOT and runtime route (host reference evaluation of the assembled block already exists via [`assemble-the-causal-self-attention-block-program`](assemble-the-causal-self-attention-block-program.md)).

## Evidence prerequisite

**Fact — the route already exists for a narrower program.** [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) carries one contraction end to end through the accepted AOT and runtime path; the retained [runtime proof](prototype-metal-runtime-proof.md) bit-compared thirty cases on one Apple M4 Max under `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32` (historical ticket spelling `FlushSubnormalsToZeroF32`). What this ticket adds is a multi-stage program with twelve inputs, three outputs, six contractions across three index structures, two transcendental families, and a mask.

**Fact — the comparison subject is the block, not the model.** The [C1 conformance fixture](../spikes/program-planning/qwen3-conformance-fixture/README.md) holds per-position logits for the whole model and is not what this compares against. The block-level reference is the pinned `transformers` 4.51.0 composition at the C1 prefill shape, which the [attention-block probe](../spikes/program-planning/attention-block-reference/README.md) already reproduces at 0 differing elements against `eager_attention_forward`.

**Unknown — the admissible numeric bound, and stating one here would be the defect.** L1 fixes that an error bound is a relation between two complete computations and cannot be composed from per-operation tolerances, and [`design-model-level-qualification-and-optimization`](design-model-level-qualification-and-optimization.md) owns the model-level bound. What this ticket owes is a *derived* block-level comparison: measure the reference's own sensitivity on this block under two independently legal F32 orderings, and report the Tiler result against that measured envelope. A threshold chosen before the evidence is exactly the ad hoc number that outcome forbids.

## Required delivery

- **The complete route**: verified semantic program, selected physical plan, structured KIR, Apple offline compilation, neutral artifact assembly, runtime validation and preflight, one-way routing commit, dispatch, exact command-buffer terminal success, and only then host readback. No host validation may read a result before terminal success is observed.
- **Five observables at the block level**, mirroring L1's model-level oracle rather than inventing a new one: exact F32 agreement or a stated per-element deviation against the reference; agreement of the two retained KV outputs, not only of the residual stream; the measured reference-sensitivity envelope the deviation is judged against; a determinism check — N repeated executions of the same artifact with the same inputs on the same device produce bit-identical results, which needs no reference and separates "disagrees with the reference" from "disagrees with itself"; and the exact host, toolchain, and numerical realization every number is bound to.
- **The named numerical cases run on the device, not only in the reference.** The masked-position signed zero (query position 0, a negative `v` at the attended key, seed `0x80000000`, completed fold `0x00000000`); a row where the probabilities do not sum to exactly one, since 49 of the C1 tensor's 160 rows do not; and the `tiled` precondition refusing structure 3 at `S = 10` rather than padding it.
- **Failure-path evidence.** Preflight refuses before routing commit; no fallback exists after allocation, partial encoding, or submission; an output element that was never written is never read as a result — seed the output allocation with a pattern no admitted case can produce and report the surviving count, treating a nonzero count as inadmissible rather than as suspicious.
- **The exact dispatch count and the exact peak transient bytes**, compared against the design's predicted `n · 16 · T · S · 4` plus the enumerated remainder. A prediction that is not compared with the run is not a prediction.

## Non-goals

The decode step and any KV cache, which are [`design-autoregressive-state-and-kv-cache`](design-autoregressive-state-and-kv-cache.md)'s — the two retained outputs are the seam and nothing consumes them here. The MLP half of the decoder layer, the twenty-eight-layer stack, the embedding gather, the vocabulary projection, ingestion, any B1 benchmark row, and any performance claim beyond the recorded timings. A model-level tolerance.

## Closes when

One complete attention block at the C1 prefill shape executes on a Metal device through the accepted AOT and runtime route, its three outputs are compared against the pinned reference under a derived rather than assumed envelope, its determinism check passes, and its named failure paths are demonstrated firing.

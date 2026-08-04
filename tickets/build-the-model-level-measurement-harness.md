---
id: build-the-model-level-measurement-harness
title: Build the model-level measurement harness
status: todo
priority: p2
dependencies: [land-the-model-level-qualification-record, drive-the-complete-forward-pass-over-three-artifacts, reclassify-language-model-work-as-a-conformance-track]
related: [define-the-model-level-conformance-corpus, qualify-the-model-level-claims-per-apple-device-and-toolchain-row, supply-the-model-level-benchmark-protocol-to-cost-calibration, define-the-model-level-regression-policy, measure-b1-d-peak-residency-on-a-named-host, prove-the-c1-complete-model-execution]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, measurement, performance, harness, language-model, metal, class-performance-study]
---
## User-visible outcome

One harness produces every measured row of the model-level qualification, on a named host under a named procedure, with a record schema that can say `refused`, `failed`, and `disagreed` as three distinct things and that states the population of every check it ran.

## Evidence prerequisite

The L8 qualification record's *Measured performance* and *The bench-host discipline* sections. The producer, validator, staged-publication, and demonstrated-failure conventions are those of [`spikes/program-planning/qwen3-conformance-fixture/`](../spikes/program-planning/qwen3-conformance-fixture/README.md) and [`spikes/apple-targets/`](../spikes/apple-targets/README.md); reuse them rather than inventing a third shape.

## Required work

- **Three outcome states, never two.** `refused` carries a typed reason and a phase; `failed` carries the execution ordinal and the token in flight; `disagreed` carries the observable and the position and deliberately carries no ordinal. A missing row is not a pass, and the schema must make a population that never ran distinguishable from one that agreed.
- **Counted populations on every row.** 18 positions, 9 passes, 270 executions, 3 artifact identities, 310 bound tensors, 28 × 18 attribution slices. A check that cannot state how many answers it expected cannot distinguish silence from success.
- **A demonstrated failing perturbation per check**, recorded in the harness's own README the way the conformance fixture records its perturbations. A check never watched failing is not yet evidence.
- **Assert the exact invariants rather than reporting them as metrics.** 30 executions per forward pass and 270 for the C1 row; exactly 3 artifact identities; exactly 3 cold pipeline creations and 0 warm; the tiled value-contraction variant selected at exactly one of C1's nine executions, at `S = 16`; and no library or pipeline cache key varying with `S`, `C`, or the cursor. A fourth artifact identity or a fourth pipeline creation is a build defect, not a regression to triage.
- **The bench-host discipline, with the model-level amendment.** One process; interleaved A/B rounds; one warm-up per round; settled minimum over rounds 1..N−1 with round 0 reported separately; spread reported beside every figure. The amendment is not optional and its evidence is [`docs/research/scheduling/first-metal-contraction-realizations.md`](../docs/research/scheduling/first-metal-contraction-realizations.md), whose own `w_decode_kv` cell measured 15.5 µs implying 270 GB/s — above the host's DRAM bandwidth — because one operand buffer was reused across dispatches: **interleave whole forward passes rather than dispatches, never share one weight allocation between the interleaved variants, and report achieved bytes per second beside every latency so a rate above the host's memory bandwidth is named as a residency artefact rather than read as a fast kernel.**
- **Two hosts, never mixed.** Correctness and attribution on the host that produced the retained digests; timing on the bench host. Every row carries its host, OS build, Xcode build, and offline compiler build, and the native translator identity is recorded as `Unknown` for any row taken through the AOT route.
- **A timing row carries the conformance verdict of the same build.** A wrong weight binding produces a correctly shaped, correctly typed, plausible logit vector that every layer accepts, so a build that is timing a different computation is not visibly different from one that is not. A timing row without a conformance verdict from the same build is malformed rather than merely unlabelled.
- **Decode latency as two numbers.** GPU time from the command buffer's own timestamps and wall-clock per token, with the gap stated — the per-step host round trip is forced by the greedy loop crossing the device completion boundary in full and is a property of the design rather than of the implementation. Prefill is reported separately and says that its 28 layer executions share one submission.
- **Time to first token as four terms, not one:** artifact preparation (build time), runtime preparation (three pipeline creations cold, zero warm), model load (the BF16→F32 widening of 596,049,920 parameters into 2,384,199,680 bytes), and the prefill pass. The expansion cache and the runtime library/pipeline caches are reported as two caches with two cold/warm axes, because they have different keys and different lifetimes.
- Report tokens per second only as the reciprocal of a measured per-token latency, labelled as that. Batch is 1 and an aggregate throughput figure would need batching the workload profile excludes.

## Explicit non-goals

No threshold, no baseline claim, and no optimization. No cost-model calibration — [`supply-the-model-level-benchmark-protocol-to-cost-calibration`](supply-the-model-level-benchmark-protocol-to-cost-calibration.md) owns the protocol handoff and [`calibrate-device-cost-models`](calibrate-device-cost-models.md) owns the fitting. No second correctness host.

## Closes when

The harness produces a retained record for the C1 row on both hosts under its own conventions, every check has a recorded failing perturbation, every exact invariant is asserted rather than reported, the achieved-bandwidth cross-check is present on every timing row, and no timing row exists without a same-build conformance verdict.

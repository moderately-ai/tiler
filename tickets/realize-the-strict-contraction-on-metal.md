---
id: realize-the-strict-contraction-on-metal
title: Realize the strict contraction as a tiled Metal scheduled kernel
status: in-progress
priority: p1
dependencies: [admit-the-contraction-normative-reference]
related: [prototype-optimizer-conformance-gate, prototype-metal-runtime-proof, broaden-governed-physical-support-for-reassociated-programs, scope-einsum-contraction-support]
scopes: [implementation/compiler, implementation/ir, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, metal, contraction, language-model]
claimed_from: todo
assignee: worker-contraction
lease_expires_at: 1785580761
---
## User-visible outcome

One contraction of the workload's projection structure compiles to a Metal kernel whose results are bit-identical to the reference evaluator at the profile's own extents — the realization the L3 elimination left standing, rather than the fastest one.

## Which realization, and why that one

**Measurement — from the [L3 realization record](../docs/research/scheduling/first-metal-contraction-realizations.md).** Six realizations were measured. The `tiled` kernel — 16x16 threadgroup-memory tiles over the two free indices and contiguous chunks of the contracted index, with each thread still folding its own output in ascending `d` — is attributed uniquely to `strict_fold+ftz` over an eight-case corpus with the other twenty-one topologies refuted, is byte-identical to the `direct` kernel at all six workload cells, and is 2.6x to 4.3x faster than it at prefill. It consumes no numerical permission.

The `simdgroup_float8x8` and `MPSMatrixMultiplication` routes are eliminated under the governed contract by measurement, not by cost; the split reductions consume permissions this profile does not grant. Do not substitute one of them to make a number better — [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md) forbids exactly that, and the L3 record states the measured price of not doing it.

## Exact blocker, which this ticket owns

**Fact — a two-input program cannot reach the compiler.** `crates/tiler-compiler/src/request.rs` rejects any program whose `input_count() != 1` at lines 1840 and 1977, and `check_recognized_operation_cover` requires the recognized operations to exhaust the reachable graph. A binary contraction fails at the first check. `broaden-governed-physical-support-for-reassociated-programs` is the precedent for widening this correctly: it generalized recognition around verified semantic occurrences rather than forcing a new shape into `NormalizedSerialSum`, and it added a checked physical representation instead of reusing one that denotes different arithmetic. Follow that shape.

**Fact — the Q-SEM-015 planning gate's stated conditions are met.** `prototype-optimizer-conformance-gate`, `prototype-metal-aot-slice`, and `prototype-metal-runtime-proof` are all `done`. The remaining limit is the recognizer above, which is this ticket's work rather than a reason to wait.

## Required delivery

- Request recognition, an index-access lowering capability for the contraction occurrence, a `ScheduledKernel` carrying the tiled schedule, structured-kernel verification, and program assembly — extended together, so every retained alternative covers the exact semantic program.
- **The tile precondition is a typed refusal, never a pad.** The tiled schedule requires `K` a positive multiple of its tile width. Every contracted extent in this profile — 1024, 2048, 3072 — satisfies it, and a K-padding schedule would owe the neutrality proof [Numerical semantics](../docs/numerical-semantics.md) requires, because `+0.0` is the strict sum's empty result and is not its bitwise-neutral padding. Refuse rather than acquire that obligation.
- **The emission must not lower a per-contributor step to a fused multiply-add.** The governed strict and flush-to-zero contracts forbid ADR 0015 contraction and require `-fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`. **Measurement — the flag is not sufficient on its own**: the spike shows `simdgroup_multiply_accumulate` fusing under `-ffp-contract=off`, reproducing [finding 16](../docs/research/apple-targets/numerical-behaviour.md) at a new construct. The per-statement emission rule is what holds the line.
- Bit-comparison against the reference at all six of the L3 profile's correctness cells, with the retained `result_sha256` values as the drift check.

## Non-goals

Structures 2 and 3, the split alternatives, the matrix-instruction route, any opaque call, and any cost model. Each has its own ticket or is deliberately absent.

## Closes when

A contraction of the profile compiles through the ordinary entry point, its results are bit-identical to the reference at every profile cell, the `K` precondition refuses with a typed reason that was watched firing, and an emitted module carries no fused multiply-add on the contraction's accumulation path.

---
id: qualify-the-simdgroup-matrix-contraction-realization
title: Qualify or refuse the simdgroup-matrix contraction realization
status: todo
priority: p2
dependencies: [realize-the-strict-contraction-on-metal]
related: [carry-the-dtype-on-the-metal-subnormal-flush-fact, declare-metal-numerical-honourability, exercise-opaque-admissions-downstream-of-the-frontier]
scopes: [implementation/metal, contracts/artifacts, research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics, contraction, target-facts]
---
## User-visible outcome

The fastest hand-written realization measured — `simdgroup_float8x8`, 1.7x the tiled strict kernel at the prefill cells and 4.4x slower than the library route there — either becomes a declared realization whose numerical contract a caller can ask for, or is refused with a reason a reader can act on. Today it is neither: it is measured to disagree with the governed contract and nothing records that as a target fact.

## What was measured, and what it eliminates

**Measurement — from the [L3 realization record](../docs/research/scheduling/first-metal-contraction-realizations.md), on an Apple M4 Max under macOS build `26A5388g`.** Two independent incompatibilities with the governed contract, not one:

1. **It delivers a fused multiply-add.** On the `contraction_pair` case the scalar kernels return the separately rounded `0x3fc58f9e` and `contract_simdgroup`, compiled in the same module under `-fmetal-math-mode=safe -ffp-contract=off`, returns the fused `0x3fc58f9d`. This is [finding 16](../docs/research/apple-targets/numerical-behaviour.md) at a new construct: the flag is a defence against the compiler contracting a written pair and is no defence against a fused operation the source asked for. ADR 0015's contraction permission is Forbidden under both the strict and flush-to-zero contracts.
2. **It seeds its accumulator at `+0.0`.** On `negative_zero_seed` it returns `0x00000000` where the strict fold returns `0x80000000`. That is not a defect by itself — it is a reduction carrying an explicit `initial` — but it is a defect for a node that declares no seed.

A third, weaker ground: it refuses `M = 1` and `M = 10` on its own `M, N, K` multiple-of-8 precondition, which are the workload's decode and C1 shapes, so the decode path would need output-side `M`-padding.

**Measurement — the attribution, and its exact strength.** Over the eight-case corpus the kernel is consistent with exactly one of twenty-two named topologies, `fma_zero_seed_fold+ftz` — a fused left fold over a `+0.0`-seeded accumulator — with the other twenty-one refuted and the refuting case named for each. That is exhaustive elimination over a finite candidate set on one device at `K = 16`, which under this repository's evidence classes qualifies a bounded profile and establishes nothing universal. Apple publishes no accumulation order or internal precision for `simdgroup_multiply_accumulate`; treat the attribution as an empirical qualification and say so wherever it is cited.

## Required delivery

One of two outcomes, decided by evidence rather than preference:

- **Qualification.** A declared realization stating the fused per-contributor step and the `+0.0` seed, carried as a versioned target fact with its measurement boundary — the exact device, OS build, SDK, and offline compiler — so a caller whose contract permits contraction, and whose node declares `initial = +0.0`, can ask for it and a caller whose contract does not is refused with the declaring profile's identity. The declaration must carry the dtype, on the same rule `carry-the-dtype-on-the-metal-subnormal-flush-fact` establishes for the subnormal flush.
- **Refusal.** A recorded decision that an unpublished accumulation order cannot be declared at all, with the reason stated once so the question does not get reopened by the next reader who notices the speed.

Either way, a widened corpus is worth having first: eight cases at `K = 16` on one GPU is the whole evidence base, and a case that separates an intra-tile reordering from an ascending fused fold does not exist yet.

## Non-goals

Making it the default. `M`-padding. Any claim about another Apple GPU family, another dtype, or the runtime-compilation path, none of which the spike exercised.

## Closes when

The route is either a declared realization with provenance a rejection can name, or a refusal with its reason recorded — and the `contraction_pair` and `negative_zero_seed` observations are reproduced by whatever check the outcome installs.

---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.metal-transcendental-emission"
kind: "experiment"
title: "Metal transcendental emission probe"
topics: ["numerics", "transcendentals", "metal", "math-modes", "softmax", "normalization"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.numerics.transformer-nonlinear-normalization-and-reductions"]
entrypoints: ["spikes/numerics/metal_transcendental_emission/probe.sh", "spikes/numerics/metal_transcendental_emission/probe.metal"]
last_verified: "2026-07-31"
ticket: "scope-transformer-nonlinear-normalization-and-reductions"
---

# Metal transcendental emission probe

## The named question

For the pinned offline Metal toolchain, **which AIR intrinsic does each MSL spelling of the workload's transcendentals select, and which compiler flag decides that selection?** The workload's exponential, reciprocal square root, division, extremum, and SIMD-reduction primitives all pass through this choice, and a spelling that silently selects a `fast_` intrinsic is a different semantic operation from one that does not.

This question exists because the [Apple GPU numerical behaviour](../../../docs/research/apple-targets/numerical-behaviour.md) record swept `-fmetal-math-fp32-functions` over multiply, add, divide, and a fused multiply-add, found it inert, and stated exactly what that does not establish: "`sin`, `sqrt`, `rsqrt`, and every other function it actually governs remain unmeasured." This probe measures the emission half of that gap and no more.

## What it does and does not establish

**It establishes**, for one pinned compiler and one flag set, which AIR call the compiler emits for a spelling. That is a compile-side fact of exactly the kind the numerical-behaviour record calls "compile-side" and separates from delivered numerics.

**It establishes nothing about values.** No device is opened, no kernel runs, and no result bit pattern is compared. `air.exp.f32` being emitted says the compiler selected the precise-family intrinsic; it does not say what that intrinsic returns, what its ULP bound is, how it behaves at `-inf`, at the overflow threshold, or on a subnormal, or whether the device even implements it as a single instruction. Every accuracy and exceptional-value question about these functions remains `Unknown` after this probe, and the research record that cites it says so.

**It installs nothing and mutates no toolchain component.** It reads the compiler already present on the host and writes only into this directory.

## Reproduce

From **this directory** (no `make` target reaches `spikes/`):

```sh
./probe.sh                  # print the record to stdout
./probe.sh > record.tsv     # capture it for comparison
```

`probe.sh` compiles [`probe.metal`](probe.metal) to AIR under five flag sets and emits one tab-separated row per `(flag set, kernel, emitted AIR callee, call-site fast-math flags)`. `governed` is the flag set the [workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md) records as the qualified Apple9/F32 baseline: `-fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`. The remaining four are the compiler default and three non-governed modes, present so the governed row is a comparison rather than an isolated observation.

Each kernel isolates exactly one MSL spelling, so a row attributes to that spelling alone.

## Retained record

[`results/2026-07-31-air-emission-msl4-macos26-metal32023.883/`](results/2026-07-31-air-emission-msl4-macos26-metal32023.883) holds `record.tsv` and the `environment.tsv` that bounds it. The record is a positive claim that outlives its producer: only re-running `probe.sh` on the same toolchain detects drift from it, and a different toolchain revision is expected to differ rather than to fail.

## The checks can say no

Three deliberate perturbations were run on 2026-07-31 and each produced the failure it was supposed to, so a row in the record is evidence that the probe read the source rather than that it ran at all.

1. **The probe reads the spelling.** Changing `exp_precise` to call `fast::exp` flipped its `governed` row from `air.exp.f32` to `air.fast_exp.f32`; restoring the source restored the row. A probe that reported a constant would not have moved.
2. **A flag value is not silently ignored.** `-fmetal-math-fp32-functions=bogus` fails compilation with `unsupported argument 'bogus' to option '-fmetal-math-fp32-functions='`, so a flag set that produced rows is a flag set the compiler accepted.
3. **The absent family really is absent.** A kernel calling `sigmoid(x)` fails with `use of undeclared identifier 'sigmoid'`, which is the check behind the record's statement that MSL exposes no sigmoid and that SiLU is therefore a composition at the source level as well as at the semantic level.

## Traceability

- **Supported claim:** [Transformer non-linear, normalization, and reduction contracts](../../../docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md).
- **The other half of the question, taken from the specification rather than the compiler:** [Metal elementary-function accuracy guarantee](../../../docs/research/numerics/metal-elementary-function-accuracy.md). It records what Apple normatively guarantees for the intrinsic families this probe measures the *selection* of, and it treats this probe's rows as corroboration of an applicability reading rather than as evidence of any bound — which is the boundary stated above, unchanged.
- **Neighbouring measurement whose gap this fills:** [Apple GPU numerical behaviour](../../../docs/research/apple-targets/numerical-behaviour.md), finding 18 and its stated boundary.
- **Normative owner:** [Numerical semantics](../../../docs/numerical-semantics.md).
- **Work record:** [`scope-transformer-nonlinear-normalization-and-reductions`](../../../tickets/scope-transformer-nonlinear-normalization-and-reductions.md).

---
id: widen-the-f16-operation-vocabulary-to-contraction-and-reassociation
title: Widen the f16 operation vocabulary to contraction and reassociation
status: in-progress
priority: p3
dependencies: []
related: [widen-the-apple-numerical-probe-to-a-second-dtype, broaden-the-apple-numerical-probe-matrix]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, measurement, dtypes]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785199695
---
The `f16` kernels added by `widen-the-apple-numerical-probe-to-a-second-dtype` cover exactly what its question needed: materialization, the multiply in both flush directions, a bare add taking a subnormal straight from the buffer, a surviving `fdiv` in both directions, the identity multiply, and the trap. Contraction, a source-level `fma`, and reassociation have no `f16` counterpart, so findings 6, 16, and 17 of [the record](../docs/research/apple-targets/numerical-behaviour.md) are `f32`-only.

That is a deliberate boundary and not an oversight, but the reason it was cheap to leave has weakened. Finding 21 established that the two dtypes' *arithmetic* differs while their emitted modules do not, which removes the assumption that an `f32` measurement of what a licence does carries to `f16`. Three specific claims now rest on `f32` alone and are cited as if they were about the compiler rather than about a dtype:

- `-ffp-contract=off` is the measured defence against contraction (finding 6), and it is not a defence against a source-level `fma` (finding 16). Whether an `f16` multiply-add contracts under the same settings is unmeasured.
- `relaxed` and `fast` reassociate a two-add chain (finding 17), which is the measurement behind "a target profile that admits `relaxed` or `fast` cannot promise a reduction order". `f16`'s ulp is 2**-10 at 1.0, so the same shape is expressible with `1.0h`, `2**-11`, and `2**-11` and needs no new machinery.

**Why this is live now.** Numerical behavior is already recorded per dtype, and
both `f16` and `bf16` have landed while contraction, source-level FMA, and
reassociation remain measured only for `f32`. The record must either reproduce
findings 6, 16, and 17 for `f16` or state those conclusions as dtype-specific.

**Cost.** One kernel each for the contraction pair, the fused pair, and the reassociation chain, plus a contraction axis for the first two. Keep the covering set bounded: the contraction settings belong behind `TILER_APPLE_NUMERICS_EXHAUSTIVE` unless a finding cites them.

**What closes this.** Each of findings 6, 16, and 17 either reproduces at `f16` and says so, or does not and is restated as a per-dtype measurement; and the record's "the `f16` vocabulary is narrower" boundary is removed rather than reworded.

## Progress 2026-07-27 — all three findings measured at `f16`; two obligations remain

**Status: not done.** The measurements exist and are on `tkt/f16-operation-vocabulary`; the record is not yet updated and the harness is red on two of its own guards. What follows is what the run established, so none of it is re-derived.

### The kernels needed a different constant, and the reason is the finding behind the finding

The obvious spelling — `x * 1.5h + 1.0h`, the `f32` pair's constants respelled — **cannot measure contraction at all**. Checked exhaustively at `float16`: for scale `1.5h` and bias `1.0h`, single rounding and double rounding agree for **every one of the eight operands** in the shared `f16` vector. 1,876 of the 32,768 finite non-negative `f16` patterns would discriminate; none is in the vector. So that kernel returns byte-identical results under every contraction setting while proving nothing — finding 7's no-witness trap in contraction clothing, and it would have been reported as "contraction does not occur at `f16`".

The scale used is `0x3E02` (1.501953125), one ulp off `1.5h` — the smallest perturbation that makes the property observable at the vector's ordinary normal `0x3555`. The witness operand `1.0h` yields `0x4101` under both fused and unfused evaluation, so the witness stays contraction-independent exactly as the `f32` pair's does. Every constant was computed at `float16` rather than by hand.

### What the three findings measure at `f16` (M4 Max, Xcode 26.6, `metalfe-32023.883`)

All three **reproduce the `f32` conclusion**, at operand `0x3555` unless noted:

- **Finding 6.** `contraction_pair_f16`: `contract-off` → `3e00`, `contract-on` → `3e00`, `contract-fast` → `3e01`. Separate rounding is preserved by `off` and `on`; `fast` fuses. `-ffp-contract=off` is the measured defence at `f16` too. The canonicalized control returns `3e00` at all three settings.
- **Finding 16.** `fused_pair_f16` returns `3e01` at **every** contraction setting including `off`. A source-level `fma` is not unfused at `f16` either.
- **Finding 17.** `reassociation_chain_f16` at operand `1.0h`: `safe` → `3c00`, `relaxed` → `3c01`, `fast` → `3c01`, at `-O0` and `-O2` and on both compilation paths. The relaxed modes reassociate at `f16`. Half an ulp at `1.0h` is `2**-11` = `0x1000`; the witness operand `0x0400` evaluates left-to-right to `0x1440`.

### Two obligations remain, and the second is a new finding rather than a fix

1. **`bf16` parity.** `test_every_kernel_names_its_dtype_exactly_when_it_is_not_the_default` requires the two narrow dtypes to carry *identical* kernel sets, so that a difference between them is a difference in the format. Adding four `f16`-only kernels breaks it. Four `bf16` twins are needed, and each needs its own discriminator analysis — `bf16` has seven mantissa bits, so the `1.5`/`1.0` degeneracy above is if anything more likely, and the constant must be derived the same way rather than copied.

2. **The runtime compiler contracts at `f16` where the offline one does not.** `test_the_two_compilation_paths_agree_case_by_case_when_a_toolchain_and_gpu_resolve` fails with `contraction_pair_f16` runtime returning `3e01` under `relaxed` and `fast`, on both `macos` and `ios-simulator`, against an offline `contract-off` candidate of `3e00`. The harness's own message says this is load-bearing and to report before changing anything, so it is reported here and **not** worked around. `MTLCompileOptions` exposes no contraction property, so a runtime kernel takes whatever the runtime compiler chooses; this measures that choice differing from the offline `off` selection at `f16`. That deserves its own finding, and it bears directly on ADR 0076's provenance discipline — a profile declaring contraction honourability from an offline measurement would be wrong about the runtime path.

### Unrelated pre-existing failure, verified not mine

`test_probes.py::test_compatibility_evidence_mutations` fails with "validator accepted missing provenance field: `probe.project_sha256`". It fails identically on a clean detached worktree at `origin/main`, so it predates this work. Nothing runs the spike suites, which is how it stayed red — the exposure `AGENTS.md` names for spikes. It needs its own ticket.

### Fixtures

`results/2026-07-27-numerics-{covering,exhaustive}-xcode26.6-metal32023.883/` are generated and the three citations in `test_numerical_probe.py`, `spikes/apple-targets/README.md`, and the record are repointed at them. They must be regenerated once the `bf16` twins land, because the kernel set changes again.

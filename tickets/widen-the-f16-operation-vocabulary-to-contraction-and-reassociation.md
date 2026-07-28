---
id: widen-the-f16-operation-vocabulary-to-contraction-and-reassociation
title: Widen the f16 operation vocabulary to contraction and reassociation
status: done
priority: p3
dependencies: []
related: [widen-the-apple-numerical-probe-to-a-second-dtype, broaden-the-apple-numerical-probe-matrix]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, measurement, dtypes]
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

## Outcome — done; findings 28, 29, and 30 (2026-07-27)

**All three questions now have narrow-dtype answers and all three reproduce the `f32` conclusion.** The `f16` boundary is removed rather than reworded, and `bf16` came along because the harness requires it.

- **Finding 28, contraction.** `contraction_pair_f16` at `3555` → `3e00` under `off` and `on`, `3e01` under `fast`; `contraction_pair_bf16` at `3eab` → `3fc0` and `3fbf`. `-ffp-contract=off` is the defence at both narrow widths. One per-dtype difference, in the strictest cell: under `safe` with `contract-fast`, `f16` fuses and `bf16` does not.
- **Finding 29, source-level `fma`.** `fused_pair_f16` → `3e01` at every contraction setting including `off`. Not unfusable at `f16` either. There is no `bf16` counterpart and there cannot be — `metal` rejects `bfloat v6 = fma(...)` because the call promotes to `float`, met here as a compile failure rather than as an IR inspection.
- **Finding 30, and this one is new rather than a twin.** The runtime compiler contracts under `relaxed` and `fast` at **all three widths**, whatever the offline `-ffp-contract` selection says: `f32` `3fc58f9d`, `f16` `3e01`, `bf16` `3fbf`, against the separately rounded value under `safe`.

### The kernel this ticket describes would have measured nothing

`x * 1.5 + 1.0` respelled at either narrow width **cannot discriminate fused from unfused for any operand in either vector** — checked exhaustively, with 1,876 of 32,768 finite non-negative `f16` patterns able to discriminate and none of them in the vector. That kernel returns byte-identical bytes under every contraction setting, and would have been reported as "contraction does not occur at the narrow widths".

The guard does not catch this, which is the part worth carrying forward: the arithmetic *does* execute and the witness *does* report `executed`. What is absent is not evidence that arithmetic ran but evidence that the two roundings differ — a second species of the finding-7 trap that the existing witness layer is not built to see. The scales are `0x3E02` and `0x3FBE`, each one ulp from 1.5 and each the nearest value that discriminates at its vector's ordinary normal, with `1.0` left contraction-independent at both. Every constant was derived at the target format, not by hand.

### Finding 30 came from a failing test, and was not worked around

`test_the_two_compilation_paths_agree_case_by_case` failed with the narrow contraction kernels diverging from their offline candidates, and its message says the divergence is load-bearing and to report before changing anything. The cause was that contraction had only ever been compiled under `safe`, so `contraction_pair` had no `relaxed` or `fast` runtime partner and the question had never been asked — while `NARROW_KERNELS` sweeps the narrow dtypes in every mode, which is why adding them surfaced it. Adding the `f32` relaxed and fast contraction cases turned "`f16` diverges" into "the runtime compiler contracts under the relaxed modes at every width". The fix was to ask the question at `f32` too, not to silence the comparison.

**It bears on ADR 0076.** Finding 9 records the two paths agreeing bit for bit while stating that this "does not make the offline build's declared realization *transferable* to a runtime-compiled kernel". This is a case where they do not coincide, and the offline flag has no runtime counterpart in which the difference could be seen. A profile declaring contraction honourability from an offline `-ffp-contract=off` compilation would be wrong about every runtime-compiled kernel under `relaxed` or `fast`.

### `bf16` parity, and a check kept able to say no

The harness holds the two narrow dtypes to identical kernel sets so a difference between them is a difference in the format. `fused_pair_bf16` cannot exist, so the assertion now names that one exclusion explicitly instead of being relaxed to a subset test. Verified reachable: setting the exclusion set empty fails it with the new message, and it was restored.

### Evidence

Fixtures regenerated at `results/2026-07-27-numerics-{covering,exhaustive}-xcode26.6-metal32023.883/`, with the three citations repointed. `uv run --with pytest pytest spikes/apple-targets` reports **81 passed, 1 failed**; the failure is `test_probes.py::test_compatibility_evidence_mutations`, verified pre-existing on a clean detached worktree at `origin/main` and filed as `fix-the-red-compatibility-evidence-mutation-test`.

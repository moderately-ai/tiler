---
id: measure-code-domain-integer-arithmetic-on-the-qualified-apple-row
title: Measure code-domain integer arithmetic on the qualified Apple row
status: done
priority: p2
dependencies: [scope-first-quantized-lm-profile]
related: [broaden-the-apple-numerical-probe-matrix, implement-first-quantized-backend-profile, admit-a-normal-scale-precondition-for-target-honourable-strict-affine-decode]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, quantization, measurement]
---
## User-visible outcome

The integer machinery a quantized decode actually executes — a `u8` buffer read, an `int` subtraction, an `int`-to-`float` conversion, and the multiply that follows — is measured on the qualified Apple row instead of assumed from a derivation. Today no integer arithmetic of any kind has ever been measured on an Apple GPU in this repository.

## What is unmeasured, stated exactly so it can be refuted

**Fact.** The retained Apple numerical probe's dtype axis is exactly `f32`, `f16`, and `bf16`; its generated kernels use `uint`/`ushort` only as bit-pattern carriers inside `as_type` immediates and the NaN-canonicalization helper, and contain no integer arithmetic operation. [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md) says so itself: "every integer and quantized format remain entirely unmeasured, and an unmeasured dtype must not be generalized from a neighbour". The one sub-byte construct in the repository, the U4 extraction expression in `crates/tiler-metal/src/emit.rs`, is checked at the string level by a test whose name ends in `_is_refused_on_the_measured_apple_profile`, is absent from the compiled golden fixtures, and has never been dispatched.

**Inference — the residual risk is narrow, which is what makes the experiment small rather than what makes it unnecessary.** [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md) derives that the decode's values are exact by construction over the finite code domain: the subtraction cannot overflow, the conversion is exact for magnitudes at most 255, and no operand or result is subnormal when the scale is normal. So this is not a numerical-behaviour sweep asking what the hardware rounds to — it is a compile-and-dispatch check asking whether the emitted MSL computes what the contract says.

## The bounded experiment

- **Inputs.** Kernels on the `apple9-f32-unified-msl4-macos26` row under the governed flags (`-target air64-apple-macos26.0 -std=metal4.0 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`) that read a `u8` buffer, widen to `int`, subtract an `int` zero point, convert to `float`, and multiply by a `float` scale. The complete 256 × 256 code and zero-point grid, against a scale corpus spanning the measured workload range (`1.358e-5` to `1.536e-1`), the `f32` minimum normal `2^-126`, and at least one deliberately subnormal scale.
- **Outputs.** The returned bits against exact rational evaluation rounded once to binary32, per case; and the delivered behaviour at the subnormal scale, recorded separately.
- **Stop condition.** Either every cell of the grid matches the reference and the subnormal case is observed flushing exactly where the derivation predicts, or a divergence is found and named with its exact inputs.
- **What it must not do.** It must not generalize to another Apple family, another dtype, a packed sub-byte extraction it did not run, or integer arithmetic outside the code domain it measured. A `u4` extraction is a *different* measurement and belongs in its own row if a `u4` profile is ever selected.

## Closes when

The measurement is run and retained beside the existing Apple records under its own dated result directory with its own schema row, its environment and toolchain identity recorded exactly, its population named and counted so a check that did not run is distinguishable from a check that found nothing, the subnormal case demonstrated, and [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md) updated with the finding and its boundary — or the measurement is blocked and the exact blocker is recorded in its place.

## Graph maintenance

- Filed by [`scope-first-quantized-lm-profile`](scope-first-quantized-lm-profile.md) as experiment E-1 of [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md).
- [`implement-first-quantized-backend-profile`](implement-first-quantized-backend-profile.md) depends on this before claiming device executability; an unmeasured `(target family, dtype)` pair is `Unknown` and cannot produce an executable artifact.
- This measures correctness, not speed. The bandwidth comparison is experiment E-2 and belongs with cost calibration.

## Outcome

**Measurement — the stop condition's first branch: every cell matched, and the subnormal case flushed exactly where the derivation predicts.** Retained as [`spikes/apple-targets/code-domain-integer-decode/results/2026-07-31-decode-u8-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`](../spikes/apple-targets/code-domain-integer-decode/results/2026-07-31-decode-u8-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv), schema `tiler.apple-code-domain-integer-decode/v1`, produced from commit `538778a` on an Apple M4 Max reporting `supportsFamily:MTLGPUFamilyApple9`, arm64 macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, macOS SDK 26.5 build 25F70, offline Metal/AIR-LLD 32023.883, runtime `GPUCompiler.framework` build `metalfe-32023.921`. **28 cases and 1,835,008 dispatched cells**; `divergence.*` is empty.

- **Twenty cases, 1,310,720 cells, bit-identical to the exact rational reference.** Five normal scales — `1.0`, the workload's measured `1.358e-5` and `1.536e-1`, the selected profile's own `2.352e-5`, and the `f32` minimum normal `2**-126` — across both compilation paths and both optimization levels, each with `exact_matches` and `flush_matches` of 65,536.
- **The subnormal scales flush on the *operand*, not on the result.** The eight subnormal-scale cases return the flush model in 65,536 cells and the exact model in 256 (the diagonal, where the two agree); `distinct_returned` is **2** where a normal scale gives **511**. At `00400000` (`2**-127`) only 510 of the 65,280 off-diagonal cells have an exactly subnormal product, and the other **64,770 have an exactly normal product and still return a signed zero** — which is what makes that scale a discriminator rather than a second reading of `2**-149`.
- **Nothing flushes at the boundary.** At `2**-126` no exact result is subnormal, the smallest nonzero magnitude returned is exactly `00800000`, and the case verdict is agreement — the derivation measured at its tightest point.
- **`code == zero_point` produces `+0.0`, never `-0.0`, in all 28 cases** (`256/256`), including at the subnormal scales.
- **Both compilation paths agree cell by cell in all 14 comparisons.**
- **The emitted module retains the integer machinery, and the conversion is a call.** `-O2`: `zext:i32-to-i64 zext:i8-to-i32 zext:i8-to-i32 sub+nsw:i32 call:air.convert.f.f32.s.i32 fmul:float`; `-O0` adds the bool materialization's `zext:i1-to-i8` and `trunc:i8-to-i1`. Every module declares `air.compile.denorms_disable air.compile.fast_math_disable air.compile.framebuffer_fetch_enable` with no relaxation flag on any floating operation.

**Fact — the harness is a sibling of `numerical_probe.py`, not an axis on it, and the trade was priced.** The kernel table there is shared by every profile, so a new kernel family moves `probe.harness_sha256` in all four retained records — the 2026-07-31 permutation landing is the measured precedent — for a question none of them asks; a 65,536-cell population is not a `case.*.results` row; and the verdict vocabulary there classifies a subnormal observation where this one classifies agreement with a computed reference. `aot-runtime-compiler-observer` is the precedent for a sibling sharing this host row and nothing else. New files live under `spikes/apple-targets/code-domain-integer-decode/`.

**Fact — what replaces the two-layer execution guard.** Every operand of the arithmetic under test arrives in a buffer, so no stage of either compiler can fold it; a device-free test walks the generated source to keep that true. The dispatch host still seeds `deadbeef`, which the producer and the validator both prove is unreachable — necessary, because the kernel writes a genuine `+0.0` for 256 cells of every grid.

**Fact — an error this experiment nearly published.** This front end lowers `float(int)` to `@air.convert.f.f32.s.i32`, not to `sitofp`. A recognizer naming the LLVM conversion opcodes would have reported the conversion stage absent from every module — the reading of a *deleted* stage. The recognizer matches every named call for that reason, pinned against verbatim `-O0` and `-O2` fragments this toolchain emitted. It is the `air.fma.f32` retraction met again in a new spelling.

**Fact — every check was watched failing and restored.** Against the retained record: rewriting `case.offline.O2.min_normal.exact_matches` to `65535` exits 2 naming the re-derived verdict; zeroing `reference.mid_subnormal.flush_sha256` exits 2 with `does not match the recomputed grid`, because the validator recomputes both grids rather than trusting the row; the untouched record exits 0. Against real returned bits: temporarily replacing `flush` with the identity and re-running the whole matrix on the device turned all eight subnormal cases `divergent` with 522,240 named cells, the first `code=1,zero_point=0,scale=00400000,returned=00000000,exact=00400000,flush=00400000`, while every normal-scale case still agreed; the harness was restored byte-identical and the record revalidated. Device-free, `test_decode_probe.py` perturbs 27 record rows one at a time plus the retained source, an unlisted `sources/` file, and a deleted `divergence.*` row of a genuinely divergent case, and requires each refusal.

**Fact — recorded where, and what was deliberately not touched.** [Apple GPU numerical behaviour](../docs/research/apple-targets/numerical-behaviour.md) carries this as **finding 32**, with the boundary paragraph that previously read "every integer and quantized format remain entirely unmeasured" corrected in one narrow direction only, a new boundary stating that finding 32's harness is not the one behind findings 1 to 31, and a Proposal separating this measurement from a dispatchability capability claim. [`spikes/apple-targets/README.md`](../spikes/apple-targets/README.md) gains the sibling's entrypoints and its section. The measurement-gap rows in [the first quantized language-model profile](../docs/research/numerics/first-quantized-lm-profile.md) are in `research/numerics`, which this ticket does not hold, so [`record-the-closure-of-the-quantized-profile-e-1-measurement-gap`](record-the-closure-of-the-quantized-profile-e-1-measurement-gap.md) was filed rather than the record edited off-scope.

**Fact — what this does not measure.** One family, one GPU, one toolchain row, one flag row (`safe`, `precise`, `-ffp-contract=off`; the relaxed modes are not swept because this profile does not admit them). `u8` codes with `u8` zero points, an `f32` scale, an `f32` result. One subtraction of two values in `[0, 255]` that cannot overflow — nothing about integer overflow, division, remainder, shifts, wider or signed integer types, or any integer operation absent from this kernel. No packed sub-byte extraction: a `u4` extraction adds a carrier and a shift-and-mask this kernel does not contain and is a different measurement belonging to its own row. No timing; E-2 is untouched. The decode as a materializing kernel over a grid, not as the fused operand access of a contraction.

**Commands.**

```sh
TILER_REQUIRE_METAL_TOOLCHAIN=1 uv run python decode_probe.py \
  --result-dir results/2026-07-31-decode-u8-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883
uv run python validate_decode_record.py results/<result>/record.tsv
uv run --with pytest pytest spikes/apple-targets
```

# Code-domain integer decode probe

This bounded experiment asks whether the emitted MSL for the registered strict-affine `u8` decode — a `uchar` code read, a widening to `int`, an `int` subtraction, an `int`-to-`float` conversion, and a multiply by an `f32` scale — computes what the contract says, over the complete 256 × 256 code and zero-point grid, on the `apple9-f32-unified-msl4-macos26` row. Before it ran, no integer arithmetic of any kind had ever been measured on an Apple GPU in this repository.

It is a **sibling** of the [numerical-behaviour probe](../README.md#numerical-behaviour-probe) rather than an axis on it, and the derivation is in `decode_probe.py`'s module docstring: the shared kernel table moves `probe.harness_sha256` in every retained numerical record, so an integer axis would have forced re-running all four for a question none of them asks — the 2026-07-31 permutation landing is the measured precedent — and a 65,536-cell population is not a `case.*.results` row. The numerical probe's verdict vocabulary classifies a *subnormal observation*; the classification wanted here is agreement with a computed reference over a population. [`aot-runtime-compiler-observer`](../aot-runtime-compiler-observer/README.md) is the precedent for a sibling that shares this host row and nothing else.

## What it measures, and what replaces the execution witness

One kernel, generated in the Metal emitter's output shape with one statement per operation exactly as `crates/tiler-metal/src/emit.rs` writes them. **Every operand of the arithmetic under test arrives in a buffer** — both `u8` components and the `f32` scale — so no stage of either compiler can fold the arithmetic away. That is the failure the numerical probe's two-layer guard exists to catch, met here as a property of the kernel rather than as a witness, and `test_no_decode_operand_is_a_compile_time_constant` walks the generated source to keep it one. The dispatch host additionally seeds its output buffer with `deadbeef`, which the producer and the validator both prove is absent from every reference value of every case — the decode writes a genuine `+0.0` for a whole diagonal of the grid, so an unwritten cell had to be distinguishable from a written zero by something the kernel cannot produce.

Two references are computed for every cell and both are retained. `exact` evaluates the decode in exact rational arithmetic and rounds **once** to `binary32`, ties to even; going through Python's `float` would round twice, and while that happens to be exact for these operands, relying on it is relying on the property under test. `flush` models what this row is measured to deliver — findings 2 and 3 of the [numerical-behaviour record](../../../docs/research/apple-targets/numerical-behaviour.md): a subnormal operand flushed to a sign-preserving zero before the multiply, one rounding, then a subnormal result flushed the same way.

For a **normal** scale the two models are identical in every cell, which is the finite derivation this experiment tests. For a **subnormal** scale they differ in exactly the 65,280 cells whose code differs from its zero point, and which model the device matches is then a measurement rather than a restatement of either.

## The scale corpus, and why each member is in it

| Scale | Pattern | Exact value | Class | Why |
| --- | --- | --- | --- | --- |
| `unit` | `3f800000` | `0x1.0p+0` | normal | isolates the widen, subtract, and convert stages: multiplying by exactly 1.0 is exact |
| `workload_min` | `3763d5a8` | `0x1.c7ab5p-17` | normal | the smallest scale measured anywhere in the pinned checkpoint (`1.358e-5`) |
| `profile_min` | `37c54cd1` | `0x1.8a99a2p-16` | normal | the smallest scale of the selected per-channel U8 profile (`2.352e-5`) |
| `workload_max` | `3e1d4952` | `0x1.3a92a4p-3` | normal | the largest scale measured anywhere in the checkpoint (`1.536e-1`) |
| `min_normal` | `00800000` | `0x1.0p-126` | normal | the exact boundary of the normal-scale precondition |
| `mid_subnormal` | `00400000` | `0x1.0p-127` | subnormal | separates input flushing from result flushing |
| `min_subnormal` | `00000001` | `0x1.0p-149` | subnormal | every nonzero product is subnormal under either mechanism |

`mid_subnormal` is the member that earns its place. At `2**-127` every widened difference of magnitude at least two has an exactly **normal** product, so a device that flushed only subnormal *results* would return those products unchanged while a device that flushes subnormal *inputs* returns a signed zero. The two hypotheses make different predictions there and the same prediction at `2**-149`.

## Running it

On a macOS host with the Apple Metal toolchain, from this directory:

```sh
TILER_REQUIRE_METAL_TOOLCHAIN=1 uv run python decode_probe.py \
  --result-dir results/<yyyy-mm-dd>-decode-u8-<profile>-<toolchain>
```

`--record <path>` writes a bare record without the retained inputs; `--result-dir` is the retaining form. `--work-dir` keeps the generated source, IR, AIR, metallibs, and result buffers for inspection. A missing toolchain, a rejected MSL version, a non-Apple9 device, a failed compile, link, pipeline, or command buffer, an unwritten output cell, or an invalid record is a nonzero refusal that publishes nothing: the producer stages everything, validates it, and renames the directory into place only after the validator agrees.

Validate a published row with `uv run python validate_decode_record.py results/<result>/record.tsv`. The device-free assertions run anywhere, including on a host with no Apple toolchain at all:

```sh
uv run --with pytest pytest spikes/apple-targets
```

Nothing runs any of this for you. `make` reaches no spike; the device dispatch is hand-run, and a toolchain change that moved a measured value would not fail a gate.

## Result on 2026-07-31

The retained record is [`results/2026-07-31-decode-u8-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`](results/2026-07-31-decode-u8-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv), schema `tiler.apple-code-domain-integer-decode/v1`, produced from commit `538778a` on an Apple M4 Max reporting `supportsFamily:MTLGPUFamilyApple9`, arm64 macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, macOS SDK 26.5 build 25F70, offline Metal/AIR-LLD 32023.883, runtime `GPUCompiler.framework` build `metalfe-32023.921`.

**28 cases and 1,835,008 dispatched cells.** Fourteen offline cases at `-O0` and `-O2` under `-target air64-apple-macos26.0 -std=metal4.0 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`, and fourteen runtime cases through `newLibraryWithSource:options:` at both `MTLLibraryOptimizationLevel` values with `math=safe,fpfun=precise,lang=4.0` read back from the options object.

**Every cell of every normal-scale case is bit-identical to the exact rational reference.** Twenty cases — five normal scales across both paths and both levels — carry `exact_matches` and `flush_matches` of 65,536 each, verdict `matches-both-models-agree`: **1,310,720 cells with no divergence**. The `divergence.*` population is empty.

**The subnormal scales flush exactly where the derivation predicts, and the flush is on the input.** The eight subnormal-scale cases return 65,536 cells matching the flush model and 256 matching the exact one — the diagonal, where both agree. The 65,280 off-diagonal cells return `00000000` for a positive widened difference and `80000000` for a negative one, so `distinct_returned` is **2** where a normal scale gives **511**. At `mid_subnormal` only 510 of those cells have an exactly *subnormal* product; the other **64,770 have an exactly normal product and still returned a signed zero**, which is direct evidence that the subnormal *scale operand* is flushed before the multiply rather than the result being flushed after it.

**The `f32` minimum normal is measured at the boundary and nothing flushes there.** At `2**-126` no exact result is subnormal, the smallest nonzero magnitude returned is exactly `00800000`, and the case verdict is agreement — the derivation's claim that "the exact product is below `2**-126` only if the scale itself is" measured at the tightest point it has.

**The registered exceptional contract holds over the whole diagonal.** `code_equals_zero_point_positive_zero` is `256/256` in all 28 cases: a code equal to its zero point produces `+0.0` and never `-0.0`, at every scale including the subnormal ones.

**Both compilation paths agree in all 14 comparisons**, cell by cell, with no `differ` row.

**The emitted module retains the integer machinery, and the conversion is not `sitofp`.** At `-O2` the operation sequence is `zext:i32-to-i64 zext:i8-to-i32 zext:i8-to-i32 sub+nsw:i32 call:air.convert.f.f32.s.i32 fmul:float`; at `-O0` the same plus the `zext:i1-to-i8` / `trunc:i8-to-i1` the bool materialization adds. Every module declares `air.compile.denorms_disable air.compile.fast_math_disable air.compile.framebuffer_fetch_enable`, and no floating-point operation carries a relaxation flag. **This front end lowers `float(int)` to a call to `air.convert.f.f32.s.i32`**, so a recognizer naming only the LLVM conversion opcodes would have reported the conversion stage absent from every module — indistinguishable from a stage a compiler deleted. The recognizer matches every named call for that reason; it is the same failure the numerical probe retracted for `air.fma.f32`, met again in a new spelling.

## Every check here was watched failing

A check that cannot say no is not evidence, so each was perturbed, observed refusing, and restored.

*Against the retained record, with the real validator.* Rewriting `case.offline.O2.min_normal.exact_matches` from `65536` to `65535` exits 2 with `case.offline.O2.min_normal.verdict is 'matches-both-models-agree' but its match counts derive 'matches-flush-model-where-models-differ'` — the verdict is re-derived rather than read. Zeroing `reference.mid_subnormal.flush_sha256` exits 2 with `does not match the recomputed grid`, because the validator recomputes both grids from the producer's own exact evaluation instead of trusting the row. Revalidating the untouched record exits 0.

*Against real returned bits, on the device.* Temporarily replacing `flush` with the identity and re-running the whole matrix turned all eight subnormal-scale cases from `matches-flush-model-where-models-differ` into `divergent` and produced 522,240 `divergence.*` rows, the first reading `code=1,zero_point=0,scale=00400000,returned=00000000,exact=00400000,flush=00400000`. Every normal-scale case still agreed, which is what distinguishes a perturbation of the model from a perturbation of the measurement. The harness was restored to byte-identical and the retained record revalidated.

*Device-free, on every run of the test suite.* `test_decode_probe.py` perturbs 27 record rows one at a time — schema, profile, grid size, sentinel, each population count, an environment row deleted, a scale pattern, a reference digest, a model-difference count, a predicted verdict, a match count, a verdict, a derivation-agreement flag, a witness, a required case row deleted, a runtime applied-options row, a comparison value and a comparison deleted, each producer digest, the recorded revision, and the status — and requires each to be refused. It also perturbs the retained kernel source, adds an unlisted file to `sources/`, and deletes the `divergence.*` row of a genuinely divergent case, and requires each to be refused. The recognizer, the verdict classifier, and the divergence namer are each shown producing their negative answers.

## What this does not measure

Everything outside the row and the domain it ran on, stated so it cannot be inherited:

- **One family, one GPU, one toolchain row.** macOS on an Apple M4 Max. No iOS device, no iOS Simulator, no other Apple GPU family, no other OS, SDK, offline compiler, runtime compiler, MSL version, or deployment minimum.
- **One dtype and one code width.** `u8` codes with `u8` zero points widened to `i32`, an `f32` scale, and an `f32` result. A `u4` extraction is a **different** measurement: it adds a packed carrier and a shift-and-mask this kernel does not contain, and it belongs in its own row if a `u4` profile is ever selected.
- **One flag row.** `safe` math, `precise` fp32 functions, `-ffp-contract=off`. The relaxed modes are not swept, because this profile does not admit them.
- **Integer arithmetic inside this code domain only.** One subtraction of two values in `[0, 255]`, which cannot overflow. Nothing here is evidence about integer overflow, division, remainder, shifts, wider or signed integer types, or any integer operation this kernel does not contain.
- **Correctness, not speed.** No timing is taken. The fused decode's achieved bandwidth is experiment E-2 of [the first quantized language-model profile](../../../docs/research/numerics/first-quantized-lm-profile.md) and belongs with cost calibration.
- **The decode alone, not its use.** This is a materializing kernel over a grid, not the fused operand access of a contraction, and it says nothing about what fusing it into a reduction would deliver.

The finding and its boundary are recorded in [Apple GPU numerical behaviour](../../../docs/research/apple-targets/numerical-behaviour.md) as finding 32.

---
schema: "tiler-doc/v1"
id: "tiler.research.apple-targets.numerical-behaviour"
kind: "research"
title: "Apple GPU f32 numerical behaviour"
topics: ["apple-targets", "metal", "numerics", "subnormals", "math-modes", "contraction"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "partially-adopted"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
informs: ["tiler.contract.metal-backend", "tiler.contract.numerical-semantics"]
adopted_by: ["ADR-0076"]
ticket: "check-in-apple-numerical-behaviour-probe"
---

# Apple GPU f32 numerical behaviour

**Status:** bounded measurement on one host, one toolchain build, and one GPU. Every value below is reproduced by a checked-in harness that the repository gate runs; none of it is a portable guarantee about Metal.

**Probe date:** 2026-07-24.

This record owns the measurements [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) depends on. Before it existed those measurements lived only in the outcome of [prototype-metal-numerical-realization](../../../tickets/prototype-metal-numerical-realization.md) and in the ADR's own re-verification, taken by a hand-built Objective-C host that was never checked in. Nothing re-established them, and a hand-run measurement in this repository has already stopped being true within the hour once unrelated work changed the compiled source. The harness is [`spikes/apple-targets/numerical_probe.py`](../../../spikes/apple-targets/numerical_probe.py) with the dispatch host [`spikes/apple-targets/numerical_probe_host.m`](../../../spikes/apple-targets/numerical_probe_host.m); the assertions are [`spikes/apple-targets/test_numerical_probe.py`](../../../spikes/apple-targets/test_numerical_probe.py), which `scripts/check_repository.py` collects; and the retained values are [`results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv`](../../../spikes/apple-targets/results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv).

## Evidence classification

This memo uses the four repository labels. **Fact** is supported by inspected source or primary documentation. **Measurement** is a direct observation tied to the exact environment and procedure below. **Inference** is derived from stated facts and measurements. **Proposal** remains to be accepted or tested.

## Measurement environment

**Measurement — the qualified row.** Apple M4 Max (`MTLCreateSystemDefaultDevice` reports `Apple M4 Max`), macOS 27.0 build 26A5388g, arm64; Xcode 26.6 build 17F113; macOS SDK `macosx` 26.5 build 25F70; `Apple metal version 32023.883 (metalfe-32023.883)` and `AIR-LLD 32023.883 (metalfe-32023.883)`.

**Fact — the procedure.** Each probe kernel is generated in the Metal emitter's output shape, with every `f32` immediate written as `as_type<float>(0x…u)` so no decimal rendering stands between the stated constant and the compiled one. Each is compiled three ways from identical source bytes: `xcrun --sdk macosx metal -target air64-apple-macos13.0 -std=metal3.1 -O<level> -fmetal-math-mode=<mode> -fmetal-math-fp32-functions=precise -ffp-contract=<contract> -S -emit-llvm` for the emitted LLVM IR, the same command with `-c` for AIR, and `xcrun --sdk macosx metallib` to link. The linked library is dispatched by a hand-written Objective-C host using `MTLCreateSystemDefaultDevice`, `newLibraryWithURL:`, `MTLResourceStorageModeShared` buffers, and one thread per output element, with `f32` bit patterns in and out. The host requires `MTLCommandBufferStatusCompleted` and a nil `commandBuffer.error` before reading the shared allocation back, and seeds the output buffer with a pattern no probe kernel can produce so an unwritten element is distinguishable from a written zero.

**Fact — the operand vector.** Every dispatch uses one vector, so a single launch answers every question about a kernel: `00000001` (smallest positive subnormal), `00400000` (mid subnormal, whose double is the smallest normal), `007fffff` (largest subnormal), `00800000` (smallest positive normal, whose half is subnormal), `80400000` (negative mid subnormal), `80000000` (negative zero, which is not subnormal), `3eb97ef9` (an ordinary normal that reveals fusion), and `3f800000` (`1.0`, the execution witness for the scaling kernels).

## Why a returned bit pattern is not, by itself, evidence

This is the load-bearing methodological point and it is encoded in the harness as a guard, not merely described here.

**Measurement — a relaxed mode can appear to honour a strict contract by deleting the arithmetic.** The emitter's `MultiplyThenAdd { scale 1.0, bias +0.0 }` shape computes an identity on every operand. At `-O2` the emitted module retains exactly one floating-point operation under `safe` — the `fadd` of `+0.0`, which cannot be removed without `nsz` — and **zero** under `relaxed` and under `fast`. Dispatched, it returns `00000000` for every subnormal operand under `safe` and returns every subnormal operand **unchanged** under `relaxed` and `fast`. Read from the returned patterns alone, the relaxed modes look like the ones that preserve subnormals. They are the ones in which no arithmetic ran.

**Measurement — counting operations in the emitted IR is necessary and not sufficient.** At `-O0` the same kernel under `relaxed` and under `fast` still carries **two** floating-point operations in the front end's LLVM IR, and the GPU nonetheless returns every operand unchanged, negative zero included. Something below the IR this harness can read — the AIR-to-ISA compilation the driver performs at pipeline-state creation — removed them. ADR 0076's account ("counting floating-point operations in the emitted LLVM IR explains it") is correct at `-O2` and incomplete at `-O0`.

**Fact — the guard the harness encodes.** `numerical_probe.subnormal_verdict` refuses to classify an observation as `preserved` or `flushed-to-zero` unless two conditions hold. First, the emitted module must contain at least one floating-point arithmetic instruction; `fcmp` is deliberately excluded, so a NaN test cannot stand in for a surviving multiply. Second, the same kernel in the same configuration must return its **execution witness**: a designated non-subnormal operand whose result differs from the operand exactly when the arithmetic ran. `multiply_two` witnesses on `3f800000 → 40000000`; the `scale 1.0, bias +0.0` kernel witnesses on `80000000 → 00000000`. A kernel that is an identity on every operand has no possible witness, `Kernel.witness` is `None` for exactly those, and every observation from them is classified `no-execution-witness` and is inadmissible. The remaining verdicts — `no-emitted-arithmetic`, `arithmetic-not-executed`, `witness-disagrees`, `unexpected-result` — each name a precise reason the observation proves nothing, rather than collapsing into a boolean.

**Inference — this is the same conclusion ADR 0076 draws, reached independently and strengthened.** Because a relaxation can delete the arithmetic, and because it can do so at a stage below the emitted IR, observing preserved subnormals from a compiled kernel is not evidence that a target preserves them. A target's numerical honourability must be a stated, versioned target fact. No amount of probing a kernel can substitute for it, and a design that tried would infer the wrong fact precisely under the modes least worth trusting.

**Fact — whether the operation was deleted or special-cased is not distinguished.** `x * 1.0f` at `-O0` retains an `fmul` in the emitted IR and returns every subnormal operand unchanged. Two explanations fit: the backend folded the multiply, or the hardware multiplier passes a denormal through when the other operand is exactly `1.0`. The harness does not separate them and does not need to; neither supports a claim about what arithmetic does, and the witness guard rejects the observation either way.

## The seven findings

Each is stated with the exact case key in the retained record, so a reader can locate the raw row in one step.

### 1. `-fmetal-math-mode=safe` still disables denormals

**Measurement.** `air.compile.denorms_disable` appears in the emitted module's `air.compile_options` under `safe`, `relaxed`, and `fast` alike, at every `-ffp-contract` setting (`case.scale_two_bias_one.*.compile_options`). Under `safe` it appears alongside `air.compile.fast_math_disable`, and no emitted `fmul` or `fadd` carries a fast-math flag at `-ffp-contract=off` or `=on`. The strictest selection the offline driver offers therefore declares fast math disabled and denormals disabled in the same module.

**Measurement — the module flag is not a summary of the licences applied.** `air.compile.fast_math_disable` is also emitted under `relaxed`, where every floating-point operation carries `reassoc nsz arcp afn`. Only `fast` emits `air.compile.fast_math_enable`. An artifact-side reader that inferred the delivered realization from the module flag would read the opposite of the truth for `relaxed`, which is the measurement ADR 0076 item 4 relies on.

**Measurement — the exact flag sets, which are contraction-dependent.**

| math mode | `-ffp-contract=off` | `=on` | `=fast` |
| --- | --- | --- | --- |
| `safe` | (none) | (none) | `contract` |
| `relaxed` | `reassoc nsz arcp afn` | `reassoc nsz arcp afn` | `reassoc nsz arcp contract afn` |
| `fast` | `reassoc nnan ninf nsz arcp afn` | `reassoc nnan ninf nsz arcp afn` | `fast` |

### 2. Input flushing and result flushing are separable, and both occur

**Measurement.** An emitted `x * 2.0f` returns `00000000` for the operand `00400000`, where preserving the operand would give the *normal* value `00800000`. Because the exact result is not itself subnormal, this isolates **input** flushing (`case.multiply_two.*`). An emitted `x * 0.5f` returns `00000000` for the *normal* operand `00800000`, where the exact result is the subnormal `00400000`. Because the operand is normal, this isolates **result** flushing (`case.multiply_half.*`). Both hold at `-O0` and `-O2` and under `safe`, `relaxed`, and `fast`, without variation. Both kernels carry an execution witness that reports `executed` in every one of those twelve configurations, so each observation is admissible under the guard above.

### 3. The flush preserves the sign of zero

**Measurement.** `0x80400000 * 2.0f` returns `80000000`, not `00000000`, in every configuration of finding 2. ADR 0076 item 1 makes this load-bearing: a flush-to-zero behaviour that does not state which zero it produces cannot be checked against this hardware and cannot be reference-evaluated.

### 4. Materialization is unaffected

**Measurement.** A load-then-store kernel with no arithmetic returns `00000001 00400000 007fffff 00800000 80400000 80000000 3eb97ef9 3f800000` unchanged under `safe`, `relaxed`, and `fast`, and its emitted module contains zero floating-point operations (`case.materialize.*`). The limit is a property of arithmetic, not of materialization, which is what lets the Metal emitter record the obligation per arithmetic statement rather than per kernel.

### 5. The math mode changes a conforming result

**Measurement.** For the emitter's `MultiplyThenAdd { scale 1.0, bias +0.0 }` shape, the operand `80000000` returns `00000000` under `safe` and `80000000` under `relaxed` and `fast`, at both `-O0` and `-O2` (`case.scale_one_bias_zero.*`). IEEE-754 round-to-nearest requires the former, since `(-0.0) + (+0.0) = +0.0`.

### 6. Contraction changes a conforming result

**Measurement.** A multiply and an add written as two separate statements over `scale = 1.5`, `bias = 1.0` return the separately rounded `3fc58f9e` for the operand `3eb97ef9` under `-ffp-contract=off` and `-ffp-contract=on`, and the fused `3fc58f9d` under `-ffp-contract=fast` (`case.contraction_pair.safe.O2.contract-*`). The per-statement emission rule is therefore a measured defence against `on` and measurably not a defence against `fast`. The fusion is not visible in the emitted IR, which retains two operations carrying the `contract` flag under `=fast`; the pair is fused below that stage.

**Measurement — the NaN canonicalization is not a contraction barrier.** With the emitter's canonicalization interposed between the multiply and the add, the same operand returns `3fc58f9e` at every contraction setting including `=fast` (`case.contraction_pair_canonicalized.*`). This is not a defence: the identical source without the canonicalization does fuse under the same flags, so the absence of fusion here is a scheduling outcome on this toolchain row rather than a guarantee. `-ffp-contract=off` remains the only thing closing the case. This reproduces the caveat `prototype-metal-numerical-realization` recorded.

### 7. The trap

**Measurement.** `x * 1.0` retains zero floating-point operations at `-O2` under every math mode (`case.multiply_one.*.O2.*`). The `scale 1.0, bias +0.0` kernel retains exactly one under `safe` at `-O2`, an `fadd`, and zero under `relaxed` and `fast`. The surviving `fadd` is what flushes. The identical `nsz` licence that breaks signed zero in finding 5 also deletes the operation that would have flushed, so one mechanism produces both observations. The full statement of this finding, its `-O0` refinement, and the guard it forces are in "Why a returned bit pattern is not, by itself, evidence" above.

## Where this record disagrees with the values it reproduces

**Measurement — `x * 1.0f` does not flush; the originating ticket says it does.** `prototype-metal-numerical-realization` records that "an emitted `x * 1.0f` returns `0x00000000` for the operand `0x00000001`". It does not, on this row: it returns `00000001`, unchanged, under `safe`, `relaxed`, and `fast` at both `-O0` and `-O2`. ADR 0076's re-verification already contradicted the ticket on exactly this point, and this record confirms the ADR against the ticket. The correction matters beyond bookkeeping: the ticket's `x * 1.0f` row was one of the three cited in support of "Apple GPU `f32` arithmetic flushes subnormals in every mode", and it is the one row of the three that proves nothing. The claim itself survives on the other two, which this record reproduces with execution witnesses.

**Measurement — the recorded fast-math flag spellings hold only at `-ffp-contract=fast`.** ADR 0076 records that "under `relaxed` each carries `reassoc nsz arcp contract afn`; under `fast` each carries `fast`", without naming a contraction setting. On this row those exact spellings appear only at `-ffp-contract=fast`. At `off` and `on` the spellings are `reassoc nsz arcp afn` and `reassoc nnan ninf nsz arcp afn`; the table in finding 1 gives all nine. Nothing in the ADR's argument depends on the spelling, so this refines the record rather than unsettling a conclusion. It is recorded because a future reader comparing a new toolchain against the ADR's text would otherwise see a spurious difference.

**Measurement — the emitted operation count is not the whole mechanism.** Stated in full above. ADR 0076's inference is correct at `-O2` and does not cover `-O0`, where both operations survive into the emitted IR and still do not execute.

## Measurement boundaries

**Fact — one row.** One machine, one GPU, one macOS build, one Xcode build, one Metal toolchain build, one SDK, one target triple (`air64-apple-macos13.0`), and one MSL version (`metal3.1`). Nothing here is evidence about another Apple GPU family, another toolchain build, iOS device or simulator artifacts, Catalyst, or any non-Apple Metal implementation. The [artifact-compatibility record](artifact-compatibility.md) already establishes that "Xcode 26.6" is an insufficient toolchain identity, and the same applies to every value here.

**Fact — `-fmetal-math-fp32-functions` is pinned.** Every measurement uses `=precise`. `prototype-metal-numerical-realization` reported that the signed-zero divergence also reproduces under `=fast`; that is not re-measured here and is not claimed.

**Fact — optimization levels.** `-O0` and `-O2` are measured. `-O1`, `-O3`, and `-Os` are not.

**Fact — the runtime compilation path is not measured here.** Every compilation is offline through `xcrun metal` and `xcrun metallib`. `prototype-metal-numerical-realization` reported that the subnormal flush is identical through runtime `newLibraryWithSource:options:` with `MTLCompileOptions.mathMode`; that observation is not re-established by this harness, and closing it is [probe-metal-runtime-compilation-numerics](../../../tickets/probe-metal-runtime-compilation-numerics.md).

**Fact — the operation vocabulary is multiply and add.** Division, transcendental functions, `fma` written as a source-level intrinsic, `half`, and every reduction shape are unmeasured. Reduction reassociation was probed in `prototype-metal-numerical-realization` over three fixtures and found no counterexample; that bounded negative result is not reproduced here.

**Inference — what the harness protects and what it cannot.** The harness fails the gate when a case row diverges from the retained record on the same environment row, so a toolchain change that alters any measured value is loud rather than silent. It cannot detect a change on a different row: when the environment differs it announces the difference and declines to compare, because a different toolchain build legitimately produces different values and quietly accepting them would defeat the purpose.

## Consequences for the contracts

**Fact — what is already adopted.** `MetalTargetFacts::subnormal_arithmetic` in `tiler-metal` takes `MetalSubnormalArithmetic::FlushesToZero` as a required caller-stated fact with the measurement recorded on the type, and `MetalNumericalGap::SubnormalFlushInArithmetic` records the unrealizable obligation. Both were introduced by `prototype-metal-numerical-realization` on the strength of the measurements this record now owns.

**Proposal — what ADR 0076 should cite.** ADR 0076 is `proposed`, and its `evidence` frontmatter currently names the Apple compatibility probe for that probe's own disclaimer, because no record about the numerical behaviour existed. It should name `tiler.research.apple-targets.numerical-behaviour` instead, keeping the compatibility probe only where the flag-acceptance row is meant. Its fifth open question, "Where the Apple numerical measurement should durably live", is answered by this record and the checked-in harness and can be closed. That edit belongs to whoever holds `contracts/decisions`.

**Proposal — what `docs/backends/metal.md` should record.** The Metal backend contract currently carries the compatibility probe's statement that the strict flag row "did not observe the numerical behavior these flags request". This record closes that gap in one direction and only one: the strict row is measured **not** to deliver subnormal preservation. It is measured to deliver a signed-zero-conforming result and a non-contracted result, which the same contract already treats as `MetalNumericalRequirement`s.

## Traceability

- Harness and retained record: the [Apple target spike](../../../spikes/apple-targets/README.md).
- Decision this evidence serves: [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md), proposed.
- Contracts informed: [Metal AOT backend](../../backends/metal.md) and [numerical semantics](../../numerical-semantics.md).
- Sibling measurement on the same host with a different question: [Apple Metal artifact compatibility](artifact-compatibility.md).
- Work record: [check-in-apple-numerical-behaviour-probe](../../../tickets/check-in-apple-numerical-behaviour-probe.md).

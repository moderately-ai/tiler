---
schema: "tiler-doc/v1"
id: "tiler.research.apple-targets.numerical-behaviour"
kind: "research"
title: "Apple GPU f32 numerical behaviour"
topics: ["apple-targets", "metal", "numerics", "subnormals", "math-modes", "contraction", "runtime-compilation"]
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

**Status:** bounded measurement on one host, one GPU, and the two compiler builds that host resolves — the offline `xcrun metal` toolchain and the runtime `MTLCompiler` the OS ships. Every value below is reproduced by a checked-in harness that the repository gate runs; none of it is a portable guarantee about Metal.

**Probe date:** 2026-07-24.

This record owns the measurements [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md) depends on. Before it existed those measurements lived only in the outcome of [prototype-metal-numerical-realization](../../../tickets/prototype-metal-numerical-realization.md) and in the ADR's own re-verification, taken by a hand-built Objective-C host that was never checked in. Nothing re-established them, and a hand-run measurement in this repository has already stopped being true within the hour once unrelated work changed the compiled source. The harness is [`spikes/apple-targets/numerical_probe.py`](../../../spikes/apple-targets/numerical_probe.py) with the dispatch host [`spikes/apple-targets/numerical_probe_host.m`](../../../spikes/apple-targets/numerical_probe_host.m); the assertions are [`spikes/apple-targets/test_numerical_probe.py`](../../../spikes/apple-targets/test_numerical_probe.py), which `scripts/check_repository.py` collects; and the retained values are [`results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv`](../../../spikes/apple-targets/results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv).

Findings 1 to 7 and their disagreements with the values they reproduce were established by [check-in-apple-numerical-behaviour-probe](../../../tickets/check-in-apple-numerical-behaviour-probe.md) through the offline compilation path alone. Findings 8, 9, and 10, and the account of what replaces the emitted-IR guard layer, were added by [probe-metal-runtime-compilation-numerics](../../../tickets/probe-metal-runtime-compilation-numerics.md), which put the byte-identical source through `newLibraryWithSource:options:` as well and compares the two paths case by case. The retained record's directory name identifies the offline toolchain; its `environment.runtime_compiler` row names the second compiler, which on this host is a different build.

## Evidence classification

This memo uses the four repository labels. **Fact** is supported by inspected source or primary documentation. **Measurement** is a direct observation tied to the exact environment and procedure below. **Inference** is derived from stated facts and measurements. **Proposal** remains to be accepted or tested.

## Measurement environment

**Measurement — the qualified row.** Apple M4 Max (`MTLCreateSystemDefaultDevice` reports `Apple M4 Max`), macOS 27.0 build 26A5388g, arm64; Xcode 26.6 build 17F113; macOS SDK `macosx` 26.5 build 25F70; offline `Apple metal version 32023.883 (metalfe-32023.883)` and `AIR-LLD 32023.883 (metalfe-32023.883)`; **runtime `Apple metal version 32023.921 (metalfe-32023.921)`**. The last of those is a different compiler build from the first and is discussed in finding 8.

**Fact — the procedure.** Each probe kernel is generated in the Metal emitter's output shape, with every `f32` immediate written as `as_type<float>(0x…u)` so no decimal rendering stands between the stated constant and the compiled one. Each is compiled three ways from identical source bytes: `xcrun --sdk macosx metal -target air64-apple-macos13.0 -std=metal3.1 -O<level> -fmetal-math-mode=<mode> -fmetal-math-fp32-functions=precise -ffp-contract=<contract> -S -emit-llvm` for the emitted LLVM IR, the same command with `-c` for AIR, and `xcrun --sdk macosx metallib` to link. The linked library is dispatched by a hand-written Objective-C host using `MTLCreateSystemDefaultDevice`, `newLibraryWithURL:`, `MTLResourceStorageModeShared` buffers, and one thread per output element, with `f32` bit patterns in and out. The host requires `MTLCommandBufferStatusCompleted` and a nil `commandBuffer.error` before reading the shared allocation back, and seeds the output buffer with a pattern no probe kernel can produce so an unwritten element is distinguishable from a written zero.

**Fact — the second procedure, added by [probe-metal-runtime-compilation-numerics](../../../tickets/probe-metal-runtime-compilation-numerics.md).** The byte-identical generated source is also compiled *in the dispatch host's own process* by `[device newLibraryWithSource:options:error:]` with an `MTLCompileOptions` whose `mathMode`, `mathFloatingPointFunctions`, `languageVersion`, and `optimizationLevel` are all set explicitly, and the resulting library takes the identical path to the GPU — same pipeline creation, same shared buffers, same terminal-status check, same readback. A difference between the two therefore cannot be an artefact of dispatching them differently. The host rejects an unrecognized option key or value with a usage exit rather than compiling with it defaulted, which the gate checks directly, because `mathFloatingPointFunctions` defaults to `Fast` and a silently ignored selection would make the record name a configuration the library was not built with.

**Fact — the operand vector.** Every dispatch uses one vector, so a single launch answers every question about a kernel: `00000001` (smallest positive subnormal), `00400000` (mid subnormal, whose double is the smallest normal), `007fffff` (largest subnormal), `00800000` (smallest positive normal, whose half is subnormal), `80400000` (negative mid subnormal), `80000000` (negative zero, which is not subnormal), `3eb97ef9` (an ordinary normal that reveals fusion), and `3f800000` (`1.0`, the execution witness for the scaling kernels).

## Why a returned bit pattern is not, by itself, evidence

This is the load-bearing methodological point and it is encoded in the harness as a guard, not merely described here.

**Measurement — a relaxed mode can appear to honour a strict contract by deleting the arithmetic.** The emitter's `MultiplyThenAdd { scale 1.0, bias +0.0 }` shape computes an identity on every operand. At `-O2` the emitted module retains exactly one floating-point operation under `safe` — the `fadd` of `+0.0`, which cannot be removed without `nsz` — and **zero** under `relaxed` and under `fast`. Dispatched, it returns `00000000` for every subnormal operand under `safe` and returns every subnormal operand **unchanged** under `relaxed` and `fast`. Read from the returned patterns alone, the relaxed modes look like the ones that preserve subnormals. They are the ones in which no arithmetic ran.

**Measurement — counting operations in the emitted IR is necessary and not sufficient.** At `-O0` the same kernel under `relaxed` and under `fast` still carries **two** floating-point operations in the front end's LLVM IR, and the GPU nonetheless returns every operand unchanged, negative zero included. Something below the IR this harness can read — the AIR-to-ISA compilation the driver performs at pipeline-state creation — removed them. ADR 0076's account ("counting floating-point operations in the emitted LLVM IR explains it") is correct at `-O2` and incomplete at `-O0`.

**Fact — the guard the harness encodes.** `numerical_probe.subnormal_verdict` refuses to classify an observation as `preserved` or `flushed-to-zero` unless two conditions hold. First, the emitted module must contain at least one floating-point arithmetic instruction; `fcmp` is deliberately excluded, so a NaN test cannot stand in for a surviving multiply. Second, the same kernel in the same configuration must return its **execution witness**: a designated non-subnormal operand whose result differs from the operand exactly when the arithmetic ran. `multiply_two` witnesses on `3f800000 → 40000000`; the `scale 1.0, bias +0.0` kernel witnesses on `80000000 → 00000000`. A kernel that is an identity on every operand has no possible witness, `Kernel.witness` is `None` for exactly those, and every observation from them is classified `no-execution-witness` and is inadmissible. The remaining verdicts — `no-emitted-arithmetic`, `arithmetic-not-executed`, `witness-disagrees`, `unexpected-result` — each name a precise reason the observation proves nothing, rather than collapsing into a boolean.

**Inference — this is the same conclusion ADR 0076 draws, reached independently and strengthened.** Because a relaxation can delete the arithmetic, and because it can do so at a stage below the emitted IR, observing preserved subnormals from a compiled kernel is not evidence that a target preserves them. A target's numerical honourability must be a stated, versioned target fact. No amount of probing a kernel can substitute for it, and a design that tried would infer the wrong fact precisely under the modes least worth trusting.

**Fact — whether the operation was deleted or special-cased is not distinguished.** `x * 1.0f` at `-O0` retains an `fmul` in the emitted IR and returns every subnormal operand unchanged. Two explanations fit: the backend folded the multiply, or the hardware multiplier passes a denormal through when the other operand is exactly `1.0`. The harness does not separate them and does not need to; neither supports a claim about what arithmetic does, and the witness guard rejects the observation either way.

### What replaces the emitted-IR layer on the runtime path

**Fact — layer 1 is unavailable there, and the harness says so rather than substituting for it.** `newLibraryWithSource:options:` returns an opaque `MTLLibrary`; there is no emitted module to read. `Observation.operations` is therefore `None` for a runtime case and never `()`, because `()` asserts a *measured* absence of arithmetic while `None` records that the question could not be asked. `subnormal_verdict` skips layer 1 only for `None`, and the retained record omits the `case.*.float_operations` row for a runtime case entirely instead of writing an empty one, so no reader can mistake the two. A portable guard test pins that distinction on every gate run, including on a host with no Apple toolchain at all.

**Inference — the layer that is lost is the weaker one.** Layer 2 is device-side and *sufficient* for "the arithmetic under test executed"; layer 1 is compile-side and merely *necessary*. Any observation layer 1 would reject emitted no arithmetic, so nothing ran, so the kernel returns its operands, so layer 2 rejects it as `arithmetic-not-executed`. The converse fails, and this harness measured it failing: at `-O0` layer 1 passed with two emitted operations and layer 2 was the layer that caught the deletion. Losing layer 1 on the runtime path costs a compile-side cross-check and a distinct diagnostic, not the admissibility decision.

**Fact — what the runtime path does instead, on every run.** A guard that never refuses anything is not a guard, so with only one layer left the harness must keep demonstrating that the layer still discriminates *on that path*: `test_the_runtime_guard_still_discriminates_when_a_toolchain_and_gpu_resolve` requires the trap kernel to be refused under `relaxed` and `fast` — on results whose unguarded reading is `preserved` — and admitted under `safe`, in the same process, in the same run.

**Measurement — one compile-side artefact survives, and it is corroboration rather than evidence.** Serializing the runtime-built pipeline into an `MTLBinaryArchive` and scanning the container recovers the compiler's version string and the presence of individual `air.compile.*` names. The container has no published layout and stores its strings concatenated without separators, so a scan can decide whether a given byte sequence is *present* but can recover neither the option set nor which strings the module attached to its `air.compile_options` node — which is exactly what the offline path resolves properly rather than substring-matching. Nothing in the admissibility guard consults it.

## The findings

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

### 8. The offline and runtime compilers are different builds on this host

**Measurement.** `xcrun --sdk macosx metal --version` reports `Apple metal version 32023.883 (metalfe-32023.883)`, resolved from the Xcode 26.6 MetalToolchain asset mounted at `/private/var/run/com.apple.security.cryptexd/mnt/com.apple.MobileAsset.MetalToolchain-v17.6.109.0.6Dib3q`. A library compiled in process by `newLibraryWithSource:options:` and serialized through an `MTLBinaryArchive` embeds `Apple metal version 32023.921 (metalfe-32023.921)`; the runtime compiler is `/System/Library/PrivateFrameworks/MTLCompiler.framework`, `CFBundleVersion` 382.5, shipped with the OS rather than with Xcode (`environment.runtime_compiler` against `environment.metal_version`).

**Inference — this is what makes finding 9 worth having.** An agreement between the two paths is agreement between two separately built compilers, not one compiler invoked twice, so it is a stronger result than it would otherwise be. It is also the reason the two versions are recorded as separate environment fields: an artifact's recorded offline toolchain provenance does **not** identify the compiler that will compile a kernel through `newLibraryWithSource:`, and a future OS update can move one without moving the other. This is the axis along which the two paths are most likely to drift apart, and the retained record is what would notice.

### 9. Every case agrees across the two compilation paths

**Measurement.** All 40 runtime cases — eight kernels across `MTLMathModeSafe`, `Relaxed`, and `Fast`, at both `MTLLibraryOptimizationLevelDefault` and `Size` — return bit patterns identical to their offline counterparts at `-O2`, for every operand in the vector (`comparison.*`). No case diverges. Findings 2, 3, 4, and 5 therefore hold through runtime compilation exactly as through offline compilation: input flushing and result flushing both occur and are separable, the flush preserves the sign of zero, materialization is untouched, and `(-0.0) * 1.0 + (+0.0)` returns `00000000` under `mathMode = Safe` and `80000000` under `Relaxed` and `Fast`. Finding 7's device-observable half reproduces too: the trap kernel returns every operand unchanged under `Relaxed` and `Fast`, the unguarded reading of those results is `preserved`, and the guard refuses them.

**Fact — the comparison fails the gate rather than being rewritten.** Each `comparison.*` row is part of the retained record and is compared on every gate run whose environment row matches, alongside the case rows. A divergence appearing on this row would fail `test_the_two_compilation_paths_agree_case_by_case_when_a_toolchain_and_gpu_resolve` by name and kernel, not in aggregate.

**Fact — what agreement does not establish.** It is one host row and two specific compiler builds. It is not evidence that the two paths agree on another toolchain pair, and it does not make the offline build's declared realization *transferable* to a runtime-compiled kernel; it makes the two happen to coincide here. Finding 8 is the reason to keep re-measuring rather than to conclude the question is settled.

### 10. `MTLCompileOptions` exposes a different surface from the offline flag set

**Fact — the counterparts, enumerated by reading the complete `@interface MTLCompileOptions` in `Metal.framework/Headers/MTLLibrary.h` of macOS SDK 26.5.** `mathMode` (`MTLMathModeSafe`/`Relaxed`/`Fast`) corresponds exactly to `-fmetal-math-mode`; `mathFloatingPointFunctions` to `-fmetal-math-fp32-functions`, and it is pinned to `Precise` because its documented default is `Fast`; `languageVersion` to `-std`. `preprocessorMacros` has no offline selection in use here to correspond to.

**Fact — `-target` has no counterpart.** There is no target property. The runtime compiler targets the device and OS it runs on, so the offline `air64-apple-macos13.0` deployment floor has no runtime analogue and nothing was substituted for it.

**Fact — `-O0` has no counterpart.** `MTLLibraryOptimizationLevel` offers `Default` and `Size` only. Both are swept and neither changes any measured value. Finding 7's `-O0` refinement — two operations surviving into the emitted IR and still not executing — therefore has no runtime counterpart and remains an offline-only measurement.

**Fact — `-ffp-contract` has no counterpart, and what was measured instead of substituting one.** There is no contraction property. Rather than choosing an offline contraction row to compare against, each runtime case is compared against *every* offline contraction setting recorded for its kernel and mode, so a kernel on which contraction is unobservable yields a plain agreement and a kernel on which it is observable reports which offline setting the runtime path behaves like. **Measurement:** for the contraction pair over `scale = 1.5`, `bias = 1.0`, the runtime path returns the separately rounded `3fc58f9e` for the operand `3eb97ef9` at both optimization levels, matching offline `-ffp-contract=off` and `=on` and not the fused `3fc58f9d` of `=fast` (`comparison.contraction_pair.runtime.safe.*`, recorded as `agree-on-some`). The runtime default does not fuse this pair on this row.

**Measurement — a source-level pragma does control contraction, and was deliberately not used.** `#pragma METAL fp contract(off)` and `#pragma clang fp contract(off)` are both accepted without diagnostic by `xcrun metal -Wall -Werror -std=metal3.1` on this row and both remove the `contract` fast-math flag from the emitted IR under `-ffp-contract=fast`. It is a source-level control and not an `MTLCompileOptions` counterpart, and using it in the runtime probe would have changed the source bytes and destroyed the byte-identical pairing the whole comparison rests on. Recorded as the available mechanism, not adopted as a substitute.

## Where this record disagrees with the values it reproduces

**Measurement — `x * 1.0f` does not flush; the originating ticket says it does.** `prototype-metal-numerical-realization` records that "an emitted `x * 1.0f` returns `0x00000000` for the operand `0x00000001`". It does not, on this row: it returns `00000001`, unchanged, under `safe`, `relaxed`, and `fast` at both `-O0` and `-O2`. ADR 0076's re-verification already contradicted the ticket on exactly this point, and this record confirms the ADR against the ticket. The correction matters beyond bookkeeping: the ticket's `x * 1.0f` row was one of the three cited in support of "Apple GPU `f32` arithmetic flushes subnormals in every mode", and it is the one row of the three that proves nothing. The claim itself survives on the other two, which this record reproduces with execution witnesses.

**Measurement — the recorded fast-math flag spellings hold only at `-ffp-contract=fast`.** ADR 0076 records that "under `relaxed` each carries `reassoc nsz arcp contract afn`; under `fast` each carries `fast`", without naming a contraction setting. On this row those exact spellings appear only at `-ffp-contract=fast`. At `off` and `on` the spellings are `reassoc nsz arcp afn` and `reassoc nnan ninf nsz arcp afn`; the table in finding 1 gives all nine. Nothing in the ADR's argument depends on the spelling, so this refines the record rather than unsettling a conclusion. It is recorded because a future reader comparing a new toolchain against the ADR's text would otherwise see a spurious difference.

**Measurement — the emitted operation count is not the whole mechanism.** Stated in full above. ADR 0076's inference is correct at `-O2` and does not cover `-O0`, where both operations survive into the emitted IR and still do not execute.

## Measurement boundaries

**Fact — one row.** One machine, one GPU, one macOS build, one Xcode build, one offline Metal toolchain build, one runtime `MTLCompiler` build, one SDK, one target triple (`air64-apple-macos13.0`), and one MSL version (`metal3.1`). Nothing here is evidence about another Apple GPU family, another toolchain build, iOS device or simulator artifacts, Catalyst, or any non-Apple Metal implementation. The [artifact-compatibility record](artifact-compatibility.md) already establishes that "Xcode 26.6" is an insufficient toolchain identity, and finding 8 shows it is doubly insufficient here: it does not identify the runtime compiler at all, which moves with the OS rather than with Xcode.

**Fact — `-fmetal-math-fp32-functions` is pinned.** Every measurement uses `=precise`. `prototype-metal-numerical-realization` reported that the signed-zero divergence also reproduces under `=fast`; that is not re-measured here and is not claimed.

**Fact — optimization levels.** Offline, `-O0` and `-O2` are measured; `-O1`, `-O3`, and `-Os` are not. On the runtime path the whole surface — `MTLLibraryOptimizationLevelDefault` and `Size` — is measured, because that is all there is.

**Fact — the runtime compilation path is measured, and only through its own surface.** Findings 8, 9, and 10 close the boundary this record previously carried, which is what `prototype-metal-numerical-realization` had reported and nothing re-established. What remains unmeasured there is what `MTLCompileOptions` cannot express: no `-target`, no `-ffp-contract`, and no `-O0`, each recorded in finding 10 rather than approximated. The per-operation fast-math flag list of finding 1 has no runtime counterpart at all, because the runtime path emits no readable module; only the module-level `air.compile.*` names are recoverable, and only as a presence test over an undocumented container.

**Fact — the runtime compile-side observations are containment tests, not resolutions.** Where the offline path resolves the `!air.compile_options` named metadata node and reports exactly the strings the module attached, the runtime path can only ask whether a given byte sequence occurs in a serialized `MTLBinaryArchive`. The two agree on this row for every math mode, but the runtime form cannot establish an option *set* and cannot establish attachment, and it is treated as corroboration everywhere in this record.

**Fact — the operation vocabulary is multiply and add.** Division, transcendental functions, `fma` written as a source-level intrinsic, `half`, and every reduction shape are unmeasured. Reduction reassociation was probed in `prototype-metal-numerical-realization` over three fixtures and found no counterexample; that bounded negative result is not reproduced here.

**Inference — what the harness protects and what it cannot.** The harness fails the gate when a case row diverges from the retained record on the same environment row, so a toolchain change that alters any measured value is loud rather than silent. It cannot detect a change on a different row: when the environment differs it announces the difference and declines to compare, because a different toolchain build legitimately produces different values and quietly accepting them would defeat the purpose.

## Consequences for the contracts

**Fact — what is already adopted.** `MetalTargetFacts::subnormal_arithmetic` in `tiler-metal` takes `MetalSubnormalArithmetic::FlushesToZero` as a required caller-stated fact with the measurement recorded on the type, and `MetalNumericalGap::SubnormalFlushInArithmetic` records the unrealizable obligation. Both were introduced by `prototype-metal-numerical-realization` on the strength of the measurements this record now owns.

**Proposal — what ADR 0076 should cite.** ADR 0076 is `proposed`, and its `evidence` frontmatter currently names the Apple compatibility probe for that probe's own disclaimer, because no record about the numerical behaviour existed. It should name `tiler.research.apple-targets.numerical-behaviour` instead, keeping the compatibility probe only where the flag-acceptance row is meant. Its fifth open question, "Where the Apple numerical measurement should durably live", is answered by this record and the checked-in harness and can be closed. That edit belongs to whoever holds `contracts/decisions`.

**Proposal — what `docs/backends/metal.md` should record.** The Metal backend contract currently carries the compatibility probe's statement that the strict flag row "did not observe the numerical behavior these flags request". This record closes that gap in one direction and only one: the strict row is measured **not** to deliver subnormal preservation. It is measured to deliver a signed-zero-conforming result and a non-contracted result, which the same contract already treats as `MetalNumericalRequirement`s.

**Proposal — what the contracts should now say about the second compiler.** Finding 8 is the durable consequence, and it is about identity rather than about numbers. A Metal artifact's toolchain provenance records the offline `metal` and `metallib` builds; on this host those do not identify the compiler that would compile a kernel through `newLibraryWithSource:`, which ships with the OS and is a different build. Two edits follow, neither of which this record's owner holds:

- `docs/backends/metal.md` should state that the AOT backend's toolchain provenance identifies the offline compiler only, and that a runtime-compiled kernel is compiled by a separate, separately versioned compiler whose identity is not part of the artifact's recorded provenance. It should also record the measured agreement of finding 9 as a bounded measurement on one host row rather than as a property of Metal, and say plainly that a delivered realization read off an offline build is not thereby true of a runtime-compiled one — it merely coincided here.
- ADR 0076 needs no change to its conclusion. Finding 9 supports rather than unsettles it, and finding 8 strengthens its central argument: numerical honourability must be a stated, versioned target fact precisely because the compiler that delivers it is not always the one the artifact names. If ADR 0076 gains a sentence, it should be that the versioned target fact has to identify *which* compiler the realization was measured on, because a Metal host resolves two. `repoint-adr-0076-evidence-at-the-numerical-record` holds `contracts/decisions` and should carry the citation there.

## Traceability

- Harness and retained record: the [Apple target spike](../../../spikes/apple-targets/README.md).
- Decision this evidence serves: [ADR 0076](../../decisions/0076-declare-target-honourable-numerical-realizations.md), proposed.
- Contracts informed: [Metal AOT backend](../../backends/metal.md) and [numerical semantics](../../numerical-semantics.md).
- Sibling measurement on the same host with a different question: [Apple Metal artifact compatibility](artifact-compatibility.md).
- Work records: [check-in-apple-numerical-behaviour-probe](../../../tickets/check-in-apple-numerical-behaviour-probe.md) for findings 1 to 7, and [probe-metal-runtime-compilation-numerics](../../../tickets/probe-metal-runtime-compilation-numerics.md) for findings 8 to 10.

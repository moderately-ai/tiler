---
id: measure-numerics-across-apple-artifact-families
title: Measure Apple numerical behaviour across all three artifact families
status: done
priority: p1
dependencies: []
related: [check-in-apple-numerical-behaviour-probe, probe-metal-runtime-compilation-numerics, declare-metal-numerical-honourability]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, measurement]
---
`tiler_metal::target::MetalPlatform` declares three artifact families — `MacOs`, `IOsDevice`, `IOsSimulator` — and `MetalTargetFacts::new` requires a caller to state `subnormal_arithmetic` for whichever one it is emitting for. The evidence behind that requirement covers **one**.

`docs/research/apple-targets/numerical-behaviour.md` states its boundary exactly: "one target triple (`air64-apple-macos13.0`) … Nothing here is evidence about another Apple GPU family, another toolchain build, iOS device or simulator artifacts, Catalyst, or any non-Apple Metal implementation." Every downstream claim inherits that scope, including the Metal backend's `MetalNumericalGap::SubnormalFlushInArithmetic` and proposed ADR 0076's assertion that the strict reading is unhonourable on Apple. Those claims are true of macOS and *unmeasured* elsewhere, which is not the same as true elsewhere.

**This was never blocked by a missing component**, which is worth stating so nobody re-derives it: `xcodebuild -showsdks` on the development host lists iOS 26.5, iOS Simulator 26.5, tvOS 26.5, visionOS, and macOS 26.5, and `xcrun simctl list runtimes` reports an installed iOS 26.0 runtime. The gap is that the harness sweeps math modes, optimization levels, and contraction settings while holding the target triple fixed.

## Split the work by what each half actually needs

**The compile-side half needs no device and should be complete.** Re-run every finding that reads the emitted module — `air.compile.denorms_disable` under all three math modes, the fast-math licence spellings across contraction settings, the emitted floating-point operation counts — against `air64-apple-ios*` device and simulator triples with the matching `--sdk`. If `denorms_disable` is emitted unconditionally for macOS but *not* for another family, or the licence spellings differ, that is a first-class finding: it would mean the flush is a per-family property rather than an Apple-wide one, and `MetalSubnormalArithmetic` would have to vary by family instead of being one declared constant.

**The device-side half is bounded by hardware.** `IOsSimulator` runs against the host GPU, so a dispatch measurement is likely reachable on this machine — establish whether it is, and if so measure it. `IOsDevice` needs real hardware this host does not have. Do **not** treat a simulator result as evidence about a device: the simulator's GPU is the Mac's. Record the device-side gap as a precise, reproducible limitation rather than filling it with an assumption, and say exactly what hardware would close it.

## Carry the existing guard forward — the part most likely to be dropped

The harness's `subnormal_verdict` refuses to classify an observation unless the emitted module retains floating-point arithmetic **and** the kernel returns its declared execution witness. That guard exists because a relaxed math mode can appear to honour a strict contract by *deleting* the arithmetic — and at `-O0` two operations survived into the readable IR and still did not execute. A compile-side-only measurement has layer 1 and no layer 2. Follow the precedent `probe-metal-runtime-compilation-numerics` set for the opposite case: `Observation.operations` is `None` rather than `()` when the question could not be asked, because `()` asserts a measured absence. Whatever the analogous distinction is for a missing device-side result, encode it in the data model rather than in prose, and never let a compile-side observation alone support a `preserved` or `flushed-to-zero` verdict.

## Boundaries

Extend `spikes/apple-targets/numerical_probe.py`; do not build a second harness. Read it and the research record in full first. Keep the retained-record shape, the environment row, and the fail-closed refusal to compare across environment rows. Self-skip where a family's toolchain or device does not resolve, following the existing `TOOLCHAIN`/`SDK`/`DEVICE` classification, and keep `TILER_REQUIRE_METAL_TOOLCHAIN` able to turn a skip into a failure. `spikes/apple-targets` is in the gate's `testpaths`, so whatever you add runs on every gate invocation — report what it costs in wall-clock.

Record the runtime `MTLCompiler` build alongside the offline one. `probe-metal-runtime-compilation-numerics` established that they are separately versioned — `metalfe-32023.883` offline versus `metalfe-32023.921` from `/System/Library/PrivateFrameworks/MTLCompiler.framework` — so a per-family row must identify both.

`docs/decisions/**` is `contracts/decisions` and `docs/backends/metal.md` is `contracts/artifacts`; this ticket holds neither. Report what they should say and let the coordinator route it.

## What closes this

Every compile-side finding measured for all three families with its exact triple and SDK; the simulator dispatch measured or its unreachability recorded precisely; the device gap stated with the hardware that would close it; and a clear statement of whether the subnormal flush is an Apple-wide property or a per-family one — because `declare-metal-numerical-honourability` needs that answer to decide whether honourability is declared once or per family.

## Outcome

The single-family probe now covers all three `MetalPlatform` families, split by reach exactly as the ticket required, and every case runs in the gate. I verified independently: 53 tests pass with zero skips on this host, so the simulator dispatch is a real measurement rather than an inherited record.

**The answer: the subnormal flush is Apple-wide on this bounded row, so `MetalSubnormalArithmetic` can be one declared constant rather than a per-family field.** All 42 compiled cases are byte-identical across `MacOs`, `IOsSimulator`, and `IOsDevice` in both `compile_options` and `float_operations` — `air.compile.denorms_disable` under every math mode, the finding-1 licence table, the `x*1.0` fold. The two dispatchable families return identical bit patterns for all 82 dispatched cases. The physical-iOS-device leg is explicitly an Inference, not a Fact: its device side was not reached. `declare-metal-numerical-honourability` should declare the flush once and record the physical-iOS-device dispatch as the trigger to reopen it.

**Per family.** `MacOs`: compile side and device side fully measured on the host GPU (Apple M4 Max). `IOsSimulator`: compile side fully measured; device side measured on a booted iOS 26.0 (23A8464) runtime via `simctl spawn`. `IOsDevice`: compile side fully measured (emitted module read for all 42 cases); device side unreachable **precisely because no physical iPhone or iPad is attached** — recorded as `Execution.NONE` with no `results` rows and the reason stated in the record, not inferred.

**What installed: nothing.** Confirmed against `xcodebuild -showsdks` and `xcrun simctl list runtimes` that every needed SDK and the iOS 26.0 simulator runtime were already present.

**The execution guard carried forward, encoded not prose.** A family with no attached device supplies layer 1 (emitted arithmetic) and not layer 2 (execution witness). Mirroring the runtime path's `operations=None`, `Observation.results` is `None` (never `()`), and a new `no-device-observation` verdict returns before any classification, so a compile-side-only observation can never yield `preserved` or `flushed-to-zero`. Sound because layer 2 is the *sufficient* layer — it caught the `-O0` deletion layer 1 passed — so losing it must forfeit admissibility rather than fall through to the weaker layer. Pinned by portable guard tests that run on a host with no Apple toolchain.

**A first-class per-family finding: three distinct compilers.** The runtime compiler belongs to the execution environment, so it differs by family — macOS loads `GPUCompiler.framework` build `metalfe-32023.921`, the iOS Simulator loads `metalfe-32023.830.1`, and the offline driver shared by all SDKs is `metalfe-32023.883`. The simulator's build was recovered from the loaded dyld image path because `MTLBinaryArchive` serialization aborts the process there (probed in a one-entry batch before any measurement manifest). `registryID` is identical for the Mac and the simulator, which is exactly why a simulator result is not evidence about a device.

**Gate cost.** `spikes/apple-targets` warm rose from ~15 s to ~13–20 s; the cold simulator boot adds a one-time ~8 s to the first run needing it, and the device is left booted so later runs pay only one `simctl spawn` per family. Skips in under a second with no Apple toolchain.

**Routing (scopes this ticket does not hold).** `docs/backends/metal.md`: AOT toolchain provenance identifies the offline compiler only; the runtime compiler belongs to the execution environment and differs between macOS and the simulator on one machine; carry findings 11 and 13 as bounded one-host measurements. ADR 0076: conclusion unchanged, findings 9/11/12 support it; a versioned target fact should identify which compiler and which execution environment the realization was measured on — `repoint-adr-0076-evidence-at-the-numerical-record` (already merged) may want a follow-up citation.

**Record note.** New v3 record at `results/2026-07-24-numerics-families-…/record.tsv`. The prior v2 macOS-only record is kept because two `todo` tickets (`supersede-the-multiply-by-one-subnormal-claim`, `record-metal-runtime-compiler-provenance-gap`) cite it with v2-specific keys; repointing those at the v3 `case.macos.*` keys is left to whoever holds them.

**Follow-up:** `measure-apple-numerics-on-physical-ios-device` (blocked, p3) — the one boundary this ticket named and could not close, hardware-gated.

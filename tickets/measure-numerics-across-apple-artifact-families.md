---
id: measure-numerics-across-apple-artifact-families
title: Measure Apple numerical behaviour across all three artifact families
status: in-progress
priority: p1
dependencies: []
related: [check-in-apple-numerical-behaviour-probe, probe-metal-runtime-compilation-numerics, declare-metal-numerical-honourability]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, measurement]
claimed_from: todo
assignee: agent-measure-numerics-across-apple-artifact-families
lease_expires_at: 1784925958
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

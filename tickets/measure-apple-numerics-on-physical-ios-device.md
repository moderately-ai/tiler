---
id: measure-apple-numerics-on-physical-ios-device
title: Measure Apple numerics on a physical iOS device
status: deferred
priority: p3
dependencies: []
related: [measure-numerics-across-apple-artifact-families, declare-metal-numerical-honourability, broaden-the-apple-numerical-probe-matrix]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, metal, measurement]
---
`measure-numerics-across-apple-artifact-families` closed the compile side for all three `MetalPlatform` families and the device side for `MacOs` and `IOsSimulator`, but the `IOsDevice` device side is unmeasured: this host has no physical iPhone or iPad attached. `docs/research/apple-targets/numerical-behaviour.md` finding 13 records the gap precisely, and `broaden-the-apple-numerical-probe-matrix` explicitly puts a physical device out of its scope.

**Scope — three dtypes, not `f32` alone.** The record itself states this ticket's span: it "leaves open for all three dtypes" the physical-device measurement, "and … is one of the two ways to close finding 26's `bf16` device gap" (`docs/research/apple-targets/numerical-behaviour.md:461`). The `bf16` half is the sharper of the two: `bf16` is `Unknown` for **both** iOS families, and for two different reasons — the simulator was asked and refused the `bfloat` pipeline, while the device was never asked at all (`:342`). The measurement boundary states the same shape from the other side: three families compiled, two GPUs dispatched, and a physical iOS device still unmeasured (`:388`). A run that measured only `f32` on an attached device would close finding 13's `f32` leg and leave finding 26's `bf16` gap exactly where it is, so the harness must carry `f32`, `f16`, and `bf16` cases — including the dtype-dispatchability probe, so a device that refuses `bfloat` is recorded as `DEVICE_REFUSED_DTYPE` rather than as an absence. This ticket's title still says `f32`; the scope is the three dtypes above.

**Blocked on hardware.** Closing this needs a physical Apple-silicon iPhone or iPad running iOS, connected to a host, and dispatching the `air64-apple-ios16.0` metallib on that device's own GPU through the same terminal-status-checked readback the other families use. The iOS Simulator does not substitute: finding 13 measured its `registryID` equal to the Mac's, so the simulator's arithmetic runs on the host GPU, and finding 14 shows the Mac will run the iOS-device module directly — recorded under `hazard.*` and refused as evidence for exactly that reason.

**What it would confirm or overturn.** Finding 11 measured the subnormal flush declared identically in every family's emitted module and observed identically on the two dispatchable families, so `declare-metal-numerical-honourability` can declare the flush once as an Apple-toolchain property rather than per family. A physical-device dispatch that flushed the same way would upgrade the iOS-device leg from Inference (compile-side agreement) to Fact; one that differed would force `MetalSubnormalArithmetic` to vary by family and reopen that decision. This ticket is the explicit trigger the honourability decision should name.

**Who does what with the result.** Whoever attaches a device and runs the harness owns writing the findings into `docs/research/apple-targets/numerical-behaviour.md` — the `IOsDevice` legs of findings 11 (the Apple-wide flush claim), 13 (the gap this ticket exists to close), 24 (`bfloat16` flushes), and 26 (the two kinds of absence) — and owns updating the measurement-boundary paragraph at `:388`, which currently reads "two GPUs wide on the device side". They then report the result to `declare-metal-numerical-honourability`, which is **`done`**: a flush that matches upgrades that decision's iOS-device leg from Inference to Fact and needs no change to it, while a flush that differs forces `MetalSubnormalArithmetic` to vary by family and **reopens an accepted decision** — so a differing result is escalated rather than recorded and closed.

**Harness reuse.** Reuse the existing case manifests, expected-row schema,
execution witness, and terminal-status-checked comparison logic. Add the
smallest signed iOS runner needed to install and execute those cases on an
attached physical device. Record device model, iOS build, Xcode/toolchain,
provisioning route, GPU identity, and exact deployment procedure. A simulator
or a Mac loading an iOS metallib is not device evidence.

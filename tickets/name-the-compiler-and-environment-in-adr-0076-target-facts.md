---
id: name-the-compiler-and-environment-in-adr-0076-target-facts
title: Name the compiler and execution environment in ADR 0076's target facts
status: todo
priority: p2
dependencies: []
related: [record-metal-runtime-compiler-provenance-gap, declare-metal-numerical-honourability]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [docs, numerics, adr, metal]
---
ADR 0076 is accepted and its conclusion is unchanged by the three-compiler finding — findings 9, 11, and 12 of [the Apple numerical record](../docs/research/apple-targets/numerical-behaviour.md) support it and finding 8 strengthens its central argument. This ticket adds one sentence, not a conclusion.

**Measurement — the fact the sentence carries.** On the recorded row (Apple M4 Max, macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113) one machine resolves three Metal compiler builds at one instant: offline `xcrun metal` is `metalfe-32023.883` from the Xcode MetalToolchain asset, the macOS host runtime compiler is `metalfe-32023.921` from the OS-shipped `GPUCompiler.framework`, and the booted iOS 26.0 Simulator runtime compiler is `metalfe-32023.830.1` from the simulator runtime's own bundled copy. `record-metal-runtime-compiler-provenance-gap` recorded the artifact-side consequence in `docs/backends/metal.md` and `docs/artifact-abi.md` and holds neither `contracts/decisions` nor the authority to widen an accepted ADR.

## The work

Item 3 requires a target honourability declaration to carry "an availability phase, a validity scope, an authority, and the declaring profile's identity". Add to that provenance discipline that a versioned target numerical fact must identify **which compiler and which execution environment** the realization was measured on, because a single Metal host resolves one offline compiler and two runtime ones, and they move independently — the runtime compiler with the OS build or the simulator runtime, the offline one with Xcode. Cross-reference it from item 4, whose delivered-realization record inherits the same requirement: a record naming a realization without naming the compiler that produced it is not readable in the sense item 4 requires.

Proposed sentence, to be sited in item 3's provenance paragraph and adapted to its surrounding prose:

> The validity scope must identify which compiler build and which execution environment the declared behaviour was measured on. One Apple host resolves an offline compiler from Xcode and a separate runtime compiler per execution environment, measured as three distinct builds on one machine, and they version independently; a target fact that names only "Metal on Apple silicon" therefore names no compiler at all.

## What this ticket must not do

Do not widen the decision. Do not restate the measurement — the research record owns it and `docs/backends/metal.md` owns the artifact-side consequence. Do not touch `decision_status`.

## Closes when

ADR 0076 items 3 and 4 state the requirement, the renderer has run, and the repository gate passes.

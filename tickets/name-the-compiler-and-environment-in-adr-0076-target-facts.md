---
id: name-the-compiler-and-environment-in-adr-0076-target-facts
title: Name the compiler and execution environment in ADR 0076's target facts
status: done
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

## Outcome

Item 3's provenance paragraph now requires a validity scope to identify which compiler build and which execution environment the declared behaviour was measured on, and item 4 records that its delivered-realization record inherits that requirement rather than adding a second one. The decision is not widened and `decision_status` is untouched.

**Fact — the measurement is cited, not restated.** The added text names the shape of the fact — one Apple host resolves an offline compiler from the Xcode toolchain asset and a separate runtime compiler per execution environment, three distinct builds on one machine, versioning independently — and links to [the Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) as the owner. The exact builds, SDKs, and OS row stay there. Enough of the fact is in the ADR to make the requirement non-arbitrary, and no more; a reader who cannot see *why* "Metal on Apple silicon" names no compiler cannot apply the rule.

**Fact — the measurement was re-verified against the record rather than taken from this ticket.** The qualified row states the offline driver `metalfe-32023.883` resolved from the Xcode 26.6 MetalToolchain asset and shared by all three SDKs, the macOS host runtime compiler `metalfe-32023.921` served by `GPUCompiler.framework`, and the booted iOS 26.0 Simulator runtime compiler `metalfe-32023.830.1` from the simulator runtime's own bundled copy. Finding 12 states the three-build conclusion in terms and notes the image path is recovered from `dyld` rather than assumed — on that row no image whose path contains `MTLCompiler` is loaded into either process, so a probe matching only the expected name would have identified nothing.

**Fact — one further edit the ticket did not name, and it was required rather than optional.** ADR 0076's status line claimed "no proposal below has been amended since" acceptance. Adding a sentence to item 3 falsifies that claim in the same change that adds it, so the status line now records the refinement, states that no conclusion and no other proposal moved, and names the work record and the evidence findings. Leaving it would have made the record contradict itself about whether it had been amended, which is worse than the drift the sentence exists to report.

**Decision — item 4 states an inheritance, not a second obligation.** The target facts item 4's record carries are the ones item 3 governs, so they arrive already identifying their compiler and environment; the added paragraph says the record carries that identification forward rather than discarding it, and gives the reason in item 4's own terms — a record naming a realization without naming the compiler that produced it is not *readable* in the sense that item requires, because a reader cannot tell whether the realization was established on the compiler that built these bytes. Writing it as a fresh obligation would have created a second authority over the same provenance discipline.

**Not done, and deliberately.** The measurement is not restated, `docs/backends/metal.md` and `docs/artifact-abi.md` are untouched — `record-metal-runtime-compiler-provenance-gap` already recorded the artifact-side consequence there — and no item beyond 3 and 4 changed.

**Measurement.** `uv run --locked python scripts/docs.py render` reported "documentation render passed (183 records)". `uv run --locked python scripts/check_repository.py` exited 0 with "complete repository validation passed". Host macOS arm64, toolchain `nightly-2026-07-19`.

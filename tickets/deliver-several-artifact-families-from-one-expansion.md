---
id: deliver-several-artifact-families-from-one-expansion
title: Deliver several artifact families from one expansion
status: in-progress
priority: p2
dependencies: [prototype-inline-aot-integration-proof, first-authoritative-ios-metal-compile-declaration, carry-one-payload-per-artifact-family-in-one-envelope]
related: []
scopes: [implementation/frontend, implementation/build, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, inline-dx, artifacts]
---
## Why this exists

Tom decided on 2026-07-25 that one selection produces **one envelope carrying one payload per built family**, and `tiler_macros::delivery::DeliveryPlan` implements the emission half completely: positional outcomes, a total `#[cfg]` selector, and one byte-string literal. Nothing produces a multi-payload envelope for it.

**Fact.** `tiler_build::accept_or_publish_single_payload_metal_artifact` refuses anything but exactly one payload (`MetalArtifactProtocolError::PayloadPortfolio`), and `accept_or_publish_metal_plan` reads position 0 alone.

**Fact.** `tiler_build::BoundMetalCompileDeclaration` publishes one constructor, `first_macos_apple9`, and its documentation states that widening to another Apple family is "a new measurement rather than a new argument". So a second family has no compile-time declaration to be compiled against even if the envelope could carry it.

**Consequence, today.** `deliver ios;` and `deliver macos-and-ios;` are refused by `tiler_macros::aot::require_buildable`, naming the one target the frontend builds. `crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.rs` and its golden pin both refusals.

## Closes when

A selection naming several families compiles each against its own bound declaration, produces one envelope carrying one payload per built family in canonical order, and the emitted selector routes each consumer target to its own payload — with a test that a wrong-family payload position is a build error rather than a wrong artifact. The measured second declaration is a prerequisite and may be its own ticket.

## Outcome

**Not closed.** Two prerequisites were established as facts rather than estimated, and both are now their own tickets and this one's dependencies. What landed is the honest determination, the measurement behind it, and the corrected consumer-visible refusal.

### The determination: no second declaration is constructible from retained evidence

The ticket's own second Fact was the gate, and it holds for a sharper reason than "no constructor exists".

**Fact — the record the ledger reads has no iOS row.** The authority ledger names its source at line 51: `spikes/apple-targets/results/2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`. Every case key in it is `case.macos.*`; `cut -f1 "$R" | grep -c ios` prints `0`.

**Fact — the ledger refuses the inheritance move by name**, at line 133: "No iOS family, physical or simulated, gains a row from this one." The refused rows are F32 dispatchability and the seven numerical rows — exactly the ones without which the profile resolves `Unknown` and `FlushSubnormalsToZeroF32` has nothing to be honoured against.

**Fact — the iOS rows that exist are the superseded MSL 3.1 record.** `2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv` carries `case.ios-device.*` and `case.ios-simulator.*` at `-std=metal3.1` against `air64-apple-ios16.0`. `metal_declaration.rs` already refuses that record for the macOS profile in the same words, and `PROFILE_MSL_VERSION` is `Metal4_0`, so it could not serve `deliver ios;` even if admissible.

**Fact — `IOsDevice` has no execution-side row, and `IOsSimulator`'s is the Mac's GPU** (`registryID` equal; `numerical-behaviour.md:219`). `deliver ios;` selects both families, so the device half is required and is hardware-blocked by `measure-apple-numerics-on-physical-ios-device` (deferred, p3).

So: **(b)**. Filed as `first-authoritative-ios-metal-compile-declaration` (blocked).

### The second prerequisite the brief did not anticipate: the envelope is not expressible

Building the multi-payload machinery would have produced code no test could exercise, because the neutral artifact model cannot represent the envelope. Two existing rules close on each other:

- every declared payload must be realized by an entry (`payloads_are_referenced`, `UnusedPayload`, pinned by `rejects_a_payload_no_entry_realizes`);
- a variant's entries are exactly its program's stages, each naming one payload — so a second payload needs a second variant, and two variants over one program under one guard are `DuplicateVariant` (pinned by `rejects_a_duplicate_plan_variant`).

**Measurement, and it ran.** `tiler_build`'s new `a_second_artifact_family_cannot_yet_share_one_envelope` drives the production seam — one compilation, one selected plan, two emissions and two AOT compilations for `air64-apple-macos26.0` and `air64-apple-ios26.0`, both carried into one `assemble_plan_artifact` — and observes exactly `[ArtifactDiagnostic::UnusedPayload]`. Perturbation: dropping the second payload makes the same assembly succeed, so the refusal is the two-payload one and not an unrelated failure.

That work lives in `tiler-artifact` and `tiler-runtime`, neither of which this ticket holds. Filed as `carry-one-payload-per-artifact-family-in-one-envelope`, whose public artifact boundary is Tom's.

**Fact — the artifact family is not a compiler-profile axis**, which is what makes that ticket tractable. The same test measures two declarations differing only in `MetalTargetFacts::platform` sharing a profile key and a byte-identical canonical descriptor. Several families are one compilation, one plan, one kernel program, and N compiled objects — not N target profiles. Worth reconciling there: `docs/artifact-abi.md:327` already asserts a program may "share one compiled object across variants declaring different profiles", which `check_subject`'s `TargetProfileMismatch` currently makes unreachable.

### What landed

- `crates/tiler-build/src/metal_declaration.rs`: `second_artifact_family_fixture`, `#[cfg(test)]` and crate-private. It moves one non-projecting field, so it is a second *artifact family* and explicitly not a second measured declaration; its doc says why it may not escape `cfg(test)`.
- `crates/tiler-build/src/metal_plan.rs`: the measurement above, retained as a pinned limitation that fails the day the model can express the envelope.
- `crates/tiler-macros/src/aot.rs`: `require_buildable` now names only the stated families that lack a declaration, and the diagnostic names the missing *measurement* and its ticket instead of "this frontend compiles exactly one target today". The empty-selection refusal is kept deliberately and its reason written down — perturbation showed that dropping it lets a `FallbackOnly` selection run the entire toolchain before failing at `MalformedPlan`. The stale module-doc sentence attributing the multi-family refusal to the single-payload cache orchestration is corrected: the missing measurement is the binding constraint and is upstream of every machinery question.
- `crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.{rs,stderr}`: golden updated. `deliver macos-and-ios;` no longer names `macos` on both sides of one sentence.

### What a consumer writing `deliver macos-and-ios;` gets

Still a refusal, at the `deliver` token, now reading: names `ios-device 26.0 at MSL 4.0, ios-simulator 26.0 at MSL 4.0`, "and no measured Metal compile-time declaration exists for it. One does exist, for macos 26.0 at MSL 4.0, and it is the only one … the retained MSL 4.0 measurement covers macOS alone … `first-authoritative-ios-metal-compile-declaration` is the work that measures a second one."

### Deliberately not done

No N-payload `accept_or_publish_metal_plan`, no N-payload cache subject, no per-family compilation loop in `tiler_macros::aot`. Each would be machinery with no artifact able to receive it and no test able to exercise it, which is the speculative abstraction the architectural contract forbids. They belong to `carry-one-payload-per-artifact-family-in-one-envelope`, where the model change makes them checkable.

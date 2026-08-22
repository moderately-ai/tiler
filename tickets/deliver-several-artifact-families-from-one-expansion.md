---
id: deliver-several-artifact-families-from-one-expansion
title: Deliver several artifact families from one expansion
status: todo
priority: p2
dependencies: [prototype-inline-aot-integration-proof, first-authoritative-ios-metal-compile-declaration, carry-one-payload-per-artifact-family-in-one-envelope]
related: []
scopes: [implementation/frontend, implementation/build, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, inline-dx, artifacts]
---
## Why this exists

Tom decided on 2026-07-25 that one selection produces **one envelope carrying one payload per built family**, and `tiler_macros::delivery::DeliveryPlan` implements the emission half completely: positional outcomes, a total `#[cfg]` selector, and one byte-string literal.

**Fact correction — the N-payload machinery has landed.** `crates/tiler-build/src/metal_plan.rs`, anchors `pub fn accept_or_publish_metal_plan`, `One emitted unit and one prepared compilation per delivery position`, and `for delivery in 0..declarations.len()`, accepts a bound declaration per delivery position, emits and compiles each one, and assembles the payloads in delivery order. `crates/tiler-build/src/payload_cache.rs` validates the same declaration/payload population. The dependency [`carry-one-payload-per-artifact-family-in-one-envelope`](carry-one-payload-per-artifact-family-in-one-envelope.md) is `done`; the former single-payload Facts below are historical evidence, not the current blocker.

**Fact.** `tiler_build::BoundMetalCompileDeclaration` publishes one constructor, `first_macos_apple9`, and its documentation states that widening to another Apple family is "a new measurement rather than a new argument". So a second family has no compile-time declaration to be compiled against even if the envelope could carry it.

**Consequence, today.** `deliver ios;` and `deliver macos-and-ios;` remain refused by `tiler_macros::aot::require_buildable`, but the binding constraint is now only the missing authoritative iOS Metal compile declaration. [`first-authoritative-ios-metal-compile-declaration`](first-authoritative-ios-metal-compile-declaration.md) remains deferred; the completed envelope path must not be described as absent.

## Closes when

A selection naming several families compiles each against its own bound declaration, produces one envelope carrying one payload per built family in canonical order, and the emitted selector routes each consumer target to its own payload — with a test that a wrong-family payload position is a build error rather than a wrong artifact. The measured second declaration is a prerequisite and may be its own ticket.

## Historical outcome before the envelope dependency landed

**Not closed.** Two prerequisites were established as facts rather than estimated, and both are now their own tickets and this one's dependencies. What landed is the honest determination, the measurement behind it, and the corrected consumer-visible refusal.

### The determination: no second declaration is constructible from retained evidence

The ticket's own second Fact was the gate, and it holds for a sharper reason than "no constructor exists".

**Fact — the record the ledger reads has no iOS row.** The authority ledger binds its source under the sentence beginning `Both rows are transcribed from`: `spikes/apple-targets/results/2026-08-02-numerics-covering-apple9-f32-bf16-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`. The 2026-07-31 F32-only path (`2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv`) is retained lineage only; the ledger says neither older control "is the source bound by this ledger now." Every case key in the bound record is `case.macos.*`; `cut -f1 "$R" | grep -c ios` prints `0`.

**Fact — the ledger refuses the inheritance move by name**: "No iOS family, physical or simulated, gains a row from this one." The refused rows are F32 dispatchability and the seven numerical rows — exactly the ones without which the profile resolves `Unknown` and `FlushSubnormalsToZeroF32` has nothing to be honoured against.

**Fact — the iOS rows that exist are the superseded MSL 3.1 record.** `2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv` carries `case.ios-device.*` and `case.ios-simulator.*` at `-std=metal3.1` against `air64-apple-ios16.0`. `metal_declaration.rs` already refuses that record for the macOS profile in the same words, and `PROFILE_MSL_VERSION` is `Metal4_0`, so it could not serve `deliver ios;` even if admissible.

**Fact — `IOsDevice` has no execution-side row, and `IOsSimulator`'s is the Mac's GPU** (`registryID` equal; anchor `the simulator result is admissible as a simulator measurement and not as an iOS-device one` in `docs/research/apple-targets/numerical-behaviour.md`). `deliver ios;` selects both families, so the device half is required and is hardware-blocked by `measure-apple-numerics-on-physical-ios-device` (deferred, p3).

So: **(b)**. Filed as `first-authoritative-ios-metal-compile-declaration` (blocked).

### The second prerequisite the brief did not anticipate: the envelope is not expressible

Building the multi-payload machinery would have produced code no test could exercise, because the neutral artifact model cannot represent the envelope. Two existing rules close on each other:

- every declared payload must be realized by an entry (`payloads_are_referenced`, `UnusedPayload`, pinned by `rejects_a_payload_no_entry_realizes`);
- a variant's entries are exactly its program's stages, each naming one payload — so a second payload needs a second variant, and two variants over one program under one guard are `DuplicateVariant` (pinned by `rejects_a_duplicate_plan_variant`).

**Measurement, and it ran.** `tiler_build`'s new `a_second_artifact_family_cannot_yet_share_one_envelope` drives the production seam — one compilation, one selected plan, two emissions and two AOT compilations for `air64-apple-macos26.0` and `air64-apple-ios26.0`, both carried into one `assemble_plan_artifact` — and observes exactly `[ArtifactDiagnostic::UnusedPayload]`. Perturbation: dropping the second payload makes the same assembly succeed, so the refusal is the two-payload one and not an unrelated failure.

That work lives in `tiler-artifact` and `tiler-runtime`, neither of which this ticket holds. Filed as `carry-one-payload-per-artifact-family-in-one-envelope`, whose public artifact boundary is Tom's.

**Fact — the artifact family is not a compiler-profile axis**, which is what makes that ticket tractable. The same test measures two declarations differing only in `MetalTargetFacts::platform` sharing a profile key and a byte-identical canonical descriptor. Several families are one compilation, one plan, one kernel program, and N compiled objects — not N target profiles. ~~Worth reconciling there: `docs/artifact-abi.md` already asserts a program may "share one compiled object across variants declaring different profiles", which `check_subject`'s `TargetProfileMismatch` currently makes unreachable.~~ **Corrected 2026-08-02: there is nothing left to reconcile, and this sentence should not be read as scheduling work.** `docs/artifact-abi.md` no longer asserts it. The phrase now appears only under the live claim that records the sentence "was withdrawn rather than made true" precisely because no artifact could exercise it, and pins both halves — `program::tests::refuses_a_second_variant_declaring_a_different_target_profile` for the refusal and `program::tests::packages_one_payload_per_delivery_position` for the reachable case. The refusal itself is unchanged at `ArtifactProgramBuilder::check_subject`'s `TargetProfileMismatch` site in `crates/tiler-artifact/src/program/builder.rs`, so the *code* half of the observation still holds; only the documentation contradiction is gone. Reproduce: `rg -n 'share one compiled object across variants|was withdrawn rather than made true' docs/artifact-abi.md`; `rg -n 'TargetProfileMismatch' crates/tiler-artifact/src/program/builder.rs`.

### What landed

**Correction — 2026-08-19, at the compilation-selection carrier's integration (`320d4a0e`): three items in the list below no longer exist.** The required compilation-selection provenance falsified the second-family fixture's own premise. That fixture moved only `MetalTargetFacts::platform`, which the ledger records as backend-only, and relied on the two declarations therefore sharing a compiler profile — but a second family's production selection can never equal the macOS-measured records' recorded selection, so the fixture now *refuses by population name* instead of assembling. `second_artifact_family_fixture`, `one_envelope_carries_one_payload_per_artifact_family`, and `a_payload_at_another_familys_delivery_position_is_refused` are all deleted; the refusal that replaced the fixture is `a_second_artifact_family_cannot_wear_this_profiles_measured_rows` in `metal_declaration`. The multi-position delivery machinery is unchanged in production code, but its end-to-end evidence is owed again under [`restore-multi-family-metal-delivery-evidence-under-per-family-profiles`](restore-multi-family-metal-delivery-evidence-under-per-family-profiles.md). *(Update — 2026-08-22: that ticket restored the evidence at the neutral seam rather than at the Metal one. `crates/tiler-build/tests/custom_backend` now declares two delivery positions and covers payload-per-position resolution, per-position retention, and the swapped-position refusals, and `metal_cache`'s own tests cover this backend's per-position stage labelling. A two-family **Metal** publication end to end still waits on `first-authoritative-ios-metal-compile-declaration`.)* The list below is retained as the record of what landed at its own date.

- `crates/tiler-build/src/metal_declaration.rs`: `second_artifact_family_fixture`, `#[cfg(test)]` and crate-private. It moves one non-projecting field, so it is a second *artifact family* and explicitly not a second measured declaration; its doc says why it may not escape `cfg(test)`.
- `crates/tiler-build/src/metal_plan.rs`: the measurement above; that historical limitation test (`a_second_artifact_family_cannot_yet_share_one_envelope`) was **replaced** by `one_envelope_carries_one_payload_per_artifact_family` (and wrong-position refusal `a_payload_at_another_familys_delivery_position_is_refused`) when the carry dependency landed.
- `crates/tiler-macros/src/aot.rs`: `require_buildable` now names only the stated families that lack a declaration, and the diagnostic names the missing *measurement* and its ticket instead of "this frontend compiles exactly one target today". The empty-selection refusal is kept deliberately and its reason written down — perturbation showed that dropping it lets a `FallbackOnly` selection run the entire toolchain before failing at `MalformedPlan`. The stale module-doc sentence attributing the multi-family refusal to the single-payload cache orchestration is corrected: the missing measurement is the binding constraint and is upstream of every machinery question.
- `crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.{rs,stderr}`: golden updated. `deliver macos-and-ios;` no longer names `macos` on both sides of one sentence.

### What a consumer writing `deliver macos-and-ios;` gets

Still a refusal, at the `deliver` token, now reading: names `ios-device 26.0 at MSL 4.0, ios-simulator 26.0 at MSL 4.0`, "and no measured Metal compile-time declaration exists for it. One does exist, for macos 26.0 at MSL 4.0, and it is the only one … the retained MSL 4.0 measurement covers macOS alone … `first-authoritative-ios-metal-compile-declaration` is the work that measures a second one."

### Deliberately not done at that historical boundary

No N-payload `accept_or_publish_metal_plan`, no N-payload cache subject, no per-family compilation loop in `tiler_macros::aot`. Each would be machinery with no artifact able to receive it and no test able to exercise it, which is the speculative abstraction the architectural contract forbids. They belong to `carry-one-payload-per-artifact-family-in-one-envelope`, where the model change makes them checkable.

## Current correction — 2026-08-09

The dependency named above subsequently landed the artifact model and the build path now exercises it. Preserve the historical determination because it explains why the work was split, but do not dispatch this ticket to rebuild N-payload assembly or caching. Its remaining delivery is the end-to-end several-family selection after a second authoritative declaration exists, including the wrong-position refusal named in `Closes when`.

## Fact audit — 2026-08-10

**Correction — 2026-08-10** (audit base `c99ac54950f2`). (1) The authority ledger binds the 2026-08-02 F32+BF16 MSL 4.0 record under `Both rows are transcribed from`, not the historical 2026-07-31 F32-only path; that older path is retained lineage only. (2) Pure line-number citations in the historical determination (`line 51`, `line 133`, `numerical-behaviour.md:219`, `docs/artifact-abi.md:385`, `builder.rs:1254`) are withdrawn in favour of phrase anchors already used above. (3) `a_second_artifact_family_cannot_yet_share_one_envelope` is absent from the tree; positive multi-payload and wrong-position coverage live as `one_envelope_carries_one_payload_per_artifact_family` and `a_payload_at_another_familys_delivery_position_is_refused` under the carry Outcome.

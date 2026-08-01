---
id: carry-one-payload-per-artifact-family-in-one-envelope
title: Carry one payload per artifact family in one envelope
status: todo
priority: p2
dependencies: []
related: [deliver-several-artifact-families-from-one-expansion, first-authoritative-ios-metal-compile-declaration, generate-cfg-gated-artifact-family-delivery, carry-a-compatibility-contract-reference-on-the-payload-descriptor]
scopes: [implementation/artifact, implementation/build, implementation/runtime, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifacts, inline-dx]
---
Tom decided on 2026-07-25 that one selection produces **one envelope carrying one payload per built family**, so the whole selection has one identity and a partial delivery is impossible by construction. `tiler_macros::delivery::DeliveryPlan` implements the emission half completely — positional outcomes in canonical family order, a total `#[cfg]` selector, one byte-string literal — and `tiler::RouteFacts::payload` is already the position it resolves to. The neutral artifact model cannot yet express the envelope those halves describe.

## Fact — the two rules close on each other

**Every declared payload must be realized by an executable entry.** `tiler_artifact::program::verify::payloads_are_referenced` requires it, `ArtifactDiagnostic::UnusedPayload` is the refusal, and `rejects_a_payload_no_entry_realizes` pins it.

**A variant's entries are exactly its program's stages.** `assemble_plan_artifact` calls `declare_entry` once per stage, and `push_variant` refuses any other count with `ArtifactBuildError::EntryCardinality`. Each entry names exactly one payload through `BackendEntryRef::payload`.

**So a second payload needs a second variant** — and `push_variant` refuses two variants packaging one program under one guard with `ArtifactBuildError::DuplicateVariant`, pinned by `rejects_a_duplicate_plan_variant`. A variant's guard is derived from the program's own applicability guard, so two variants over one plan always collide.

**Measurement.** `tiler_build`'s `a_second_artifact_family_cannot_yet_share_one_envelope` runs the production seam: one compilation, one selected plan, two emissions and two AOT compilations for `air64-apple-macos26.0` and `air64-apple-ios26.0`, both carried into one `assemble_plan_artifact` call. It refuses with exactly `[ArtifactDiagnostic::UnusedPayload]`. Dropping the second payload makes the same assembly succeed, which is what separates this refusal from an unrelated failure.

## Fact — the artifact family is not a compiler-profile axis, and that is what makes this tractable

The ledger's projection table records `MetalTargetFacts::platform` as backend-only, and the same test measures the consequence: two declarations differing *only* in platform share a profile key and a byte-identical canonical descriptor. So the two families are one compilation, one plan, one kernel program, and two compiled objects — not two target profiles. `ArtifactProgramBuilder::check_subject` would refuse two variants declaring different profiles with `TargetProfileMismatch`, but that is not the constraint that binds here.

Worth reconciling while doing this: `docs/artifact-abi.md:327` already states that a payload's `TargetProfileRef` is "the payload's contract, not the plan's", and that carrying it per payload "lets a program share one compiled object across variants declaring different profiles" — a sentence `check_subject` currently makes unreachable.

## What this has to decide

How an artifact expresses several backend objects that realize *the same* entries and are chosen by the consumer's build target rather than by the device. Today `tiler::route::select_embedded_route` only checks `RouteFacts::payload` for `None`; the index selects nothing, because there is nothing yet for it to select among. Whatever shape this takes has to keep the properties the current rules exist to protect: no unreferenced payload (which would give one artifact two byte identities — `docs/artifact-abi.md:188`), no ambiguous entry-to-payload mapping, and identity folding every carried payload so a two-family envelope and a one-family envelope are never one artifact.

## Closes when

One envelope carries one payload per built family with a cache subject covering the whole selection, the consumer's `#[cfg]`-selected position routes to that family's payload, a wrong-position payload is a build error rather than a wrong artifact, and `a_second_artifact_family_cannot_yet_share_one_envelope` is replaced by the positive test. It is only observable end-to-end once a second family has a measured declaration (`first-authoritative-ios-metal-compile-declaration`); the envelope work itself is exercisable before that through the `#[cfg(test)]` second-family fixture in `crates/tiler-build/src/metal_declaration.rs`.

The public artifact boundary is Tom's to accept.

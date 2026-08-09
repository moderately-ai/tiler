---
id: record-the-compilation-selection-in-target-measurement-provenance
title: Record the compilation selection in target measurement provenance
status: awaiting-decision
priority: p2
dependencies: []
related: [construct-and-bind-the-first-authoritative-metal-compile-profile, declare-the-bf16-rows-on-the-authoritative-metal-profile, measure-macos-apple9-bf16-under-unified-msl4-profile]
scopes: [implementation/compiler, contracts/decisions, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, provenance, identity, numerics, decision, needs-tom, public-boundary]
---
## User-visible outcome

Two target-profile rows measured under compilations that differ in language standard, requested target, or any other producer-defining compilation selection are distinguishable in the profile descriptor, so a profile cannot silently claim that one compilation produced rows another one did.

## The gap, exactly located

**Fact.** `TargetCompileProfileMeasurementSource::new` (`crates/tiler-compiler/src/target.rs`, source anchor `pub struct TargetCompileProfileMeasurementSource`) takes a producer identity and a set of `TargetMeasurementContext`. The surrounding types are located by `pub struct TargetMeasurementContext`, `pub struct TargetCompilerBuild`, and `pub struct TargetExecutionEnvironmentBuilder`. `TargetCompilerBuild::new` carries `role`, `implementation`, `version`, and `build`; `TargetExecutionEnvironmentBuilder::build` requires and carries exactly `platform`, `platform-version`, `platform-build`, `architecture`, and `hardware`. **No field holds the language standard, the requested or emitted target triple, or the compilation flags.**

**Measurement — the gap is reachable today, not hypothetical.** The two retained Apple records `2026-07-31-numerics-covering-xcode26.6-metal32023.883` and `2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883` were produced on one host by one toolchain and differ in exactly `-std` (`metal3.1` against `metal4.0`) and `requested_target` (`air64-apple-macos13.0` against `air64-apple-macos26.0`). Every field the provenance vocabulary can hold is byte-identical:

```sh
R3=spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv
R4=spikes/apple-targets/results/2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv
for k in environment.xcode environment.os_version environment.os_build environment.machine \
         environment.family.macos.sdk_version environment.family.macos.sdk_build \
         environment.family.macos.metal_version environment.family.macos.metallib_version \
         environment.family.macos.device environment.family.macos.requested_target; do
  printf '%-46s %-24s %s\n' "$k" "$(grep -m1 "^$k	" $R4 | cut -f2)" "$(grep -m1 "^$k	" $R3 | cut -f2)"
done
```

The first nine agree; only `requested_target` differs. A row sourced from either record therefore produces the same provenance bytes and the same profile descriptor.

**Inference — what that costs.** The authority ledger's whole first section, "The two environments this ledger keeps apart", is built on the compilation selection being part of what a measured row is valid for; it tabulates the requested target and the language standard as components of the offline environment. The compiler's provenance type cannot represent either, so the discipline lives entirely in prose and in `BoundMetalCompileDeclaration`'s single-`LedgerRows` construction. That construction is a good defence and it is not the type system: a declaration assembling rows from two compilations would be accepted, would encode identically to one that did not, and would be undetectable from the descriptor that artifact and cache identity are taken over.

**This is a missing authority, not a defect with a known repair.** Whether the fix is a field on `TargetCompilerBuild`, a compilation-selection record on `TargetMeasurementContext`, or a refusal that keeps the compiler out of the business of naming a backend's flags is a genuine design question — the compiler core must stay independent of Metal, and `-std=metal4.0` is a Metal spelling. Reserving a producer-defined, backend-opaque selection identity is the shape most likely to survive that constraint, and it is a proposal rather than a conclusion.

## Scope keys

- Decide whether the compilation selection belongs on the compiler's provenance vocabulary at all, or whether the profile is right to be silent and the obligation belongs to the bound declaration that owns the backend facts. State the elimination rather than only the choice.
- If it belongs: it must be backend-opaque. The compiler may not learn what `-std` means, and a typed `MslLanguageVersion` reaching `tiler-compiler` would violate the consumer-neutrality invariant.
- Any new field is identity-bearing and moves every pinned descriptor. Enumerate the moved pins and recompute them on the tree the step lands into.
- Do not widen `TargetFactSource::external_guarantee`'s normative-reference route to stand in for this. A normative reference names a document; a compilation selection names an invocation.

## Decision packet — 2026-08-09

- **Option A — add one backend-opaque compilation-selection identity to each measurement context (recommended).** The producing backend hashes or keys its complete invocation selection; the compiler preserves and compares opaque bytes without learning Metal flags. This distinguishes the two measured records and keeps consumer neutrality.
- **Option B — leave compiler provenance silent and make the bound backend declaration the sole owner.** This preserves descriptor identity but accepts that a profile descriptor alone cannot distinguish rows produced by different compilation selections.

Tom must choose which authority owns the distinction. Option A is an identity-bearing public provenance field and must move every descriptor pin coherently.

## Required evidence

- A perturbation moving only the compilation selection and observing the profile descriptor move with it, of the same shape as `every_measurement_context_field_moves_the_profile_descriptor` in `crates/tiler-build/src/metal_declaration.rs`, watched failing before the fix.
- Confirmation that the compiler core still names no backend type.
- Every moved pin enumerated with its before and after value.

## Closes when

Either the vocabulary distinguishes two compilations differing only in their selection and a perturbation test proves it, or the decision to leave it out is recorded with its derivation and the obligation is stated on the owner that does carry it, with the authority ledger's environment section updated to say which.

## Graph maintenance

- Discovered by `declare-the-bf16-rows-on-the-authoritative-metal-profile`, which hit the gap while trying to attribute an MSL 3.1 BF16 measurement to an MSL 4.0 profile. That ticket does **not** depend on this one: `measure-macos-apple9-bf16-under-unified-msl4-profile` removes its need by measuring on the profile's own compilation row, which is the correct repair for that ticket regardless of how this question is decided.
- Related to `construct-and-bind-the-first-authoritative-metal-compile-profile`, whose ledger states the discipline this type cannot enforce.

## Scope repair — 2026-08-09

`implementation/build` is declared because the required perturbation, the authoritative Metal declaration, and the descriptor pins this decision may move are in `crates/tiler-build`; compiler-only scope could not deliver either option A's evidence or its complete identity accounting.

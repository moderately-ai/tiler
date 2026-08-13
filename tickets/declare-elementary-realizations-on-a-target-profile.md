---
id: declare-elementary-realizations-on-a-target-profile
title: Declare elementary realizations on a target profile
status: done
priority: p1
dependencies: [admit-the-registered-unary-families-at-the-compiler-request-boundary, require-both-elementary-evidence-halves-before-target-admission]
related: [carry-the-elementary-numerical-dimensions-in-the-region-realization, establish-hard-exceptional-value-evidence-for-metal-elementary-realizations]
scopes: [implementation/compiler, contracts/decisions, implementation/build, contracts/numerics, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, numerics, feasibility, public-boundary, decision]
---
## User-visible outcome

A caller that builds its own `TargetProfile` can state which elementary-function accuracy contracts its target realizes, and gets a structured refusal naming the operation when it has not — instead of a profile that is silently unable to compile any program containing `tiler::silu-f32@1`.

## Source-first audit at `187ecabd5fe74446401d9fe1ba7d71f6f23837ad`

Tickets are written against a tree that has since moved. Re-read at this exact base:

**Fact — verified. Assessment is on the compile path and the installed set is still gated on one profile.** `request::require_elementary_accuracy` still calls `assess_program_elementary_accuracy`. `declared_elementary_realizations` still returns `installed_elementary_realizations()` only when `canonical_descriptor()` equals the governed profile's, else empty. There is still no `declare_elementary_realization` under `crates/tiler-compiler`.

**Fact — verified. The three Metal rows remain honest records that cannot discharge.** `installed_elementary_realizations` still pairs normative bound evidence with empirical exceptional-value evidence, each attributed to `governed_profile_source()`. `ElementaryRealization::declare` and `assess_elementary_accuracy` both require both halves to discharge. Prerequisite [`require-both-elementary-evidence-halves-before-target-admission`](require-both-elementary-evidence-halves-before-target-admission.md) is `done`.

**Fact — verified. The whole-subject builder precedent is still `declare_synchronization_realization`.** It takes one complete subject, refuses an exact duplicate, and has no per-dimension spelling.

**Fact — false as written; repaired below.** The "Why this is filed" section said `target_compile_failure` in `crates/tiler-compiler/src/session.rs` deliberately produces no `TargetCompileRefusal` variant for `UnrealizedElementaryAccuracy`. After the evidence-halves ticket, that mapping already exists: `TargetCompileRefusal::ElementaryAccuracy` carries `TargetElementaryAccuracyRefusal` with the operation, assessed target profile, and `TargetElementaryAccuracyReason` (`NoInstalledRealization` / `Unrefined` / `UndischargedEvidence`). What is still missing is deterministic candidate-declaration provenance on that refusal, plus the public builder declaration that would populate candidates.

**Scope addition.** `contracts/optimizer` is required by the already-authorized documentation delivery: `docs/compiler/optimizer.md` currently describes the governed-descriptor shortcut and must name stored declared rows instead.

## Decision accepted — 2026-08-11

Tom accepted a strict whole-subject declaration, conditional on first landing [`require-both-elementary-evidence-halves-before-target-admission`](require-both-elementary-evidence-halves-before-target-admission.md). This section supersedes any looser or stale delivery wording below.

- The public value is a validated whole record with private fields and read-only accessors.
- Its operation is derived from a verified `AccuracyContract`; callers do not supply a second operation key that can disagree.
- It carries the complete bound and exceptional-value evidence records and a compile-profile-phase source. Later-phase or generic source values are not accepted.
- `TargetProfileBuilder` stores canonical rows. Multiple genuinely different contracts for one operation remain legal; only an exact duplicate is rejected. No row is silently replaced, merged, or preferred.
- The profile exposes a borrowed canonical row view. It does not reconstruct governed rows during assessment.
- Structured refusals distinguish: no installed row for the assessed target profile; installed rows whose contracts do not refine the requirement; and rows whose bound or exceptional evidence does not discharge. Candidate details are deterministic when several rows exist.
- The canonical profile row encodes the complete verified contract, both complete evidence records, and its source in a terminal, domain-separated section. The implementation ticket must rederive whether the owning descriptor domain steps; it must not assume either an unconditional version bump or byte stability.
- There are no per-field setters, defaults, key/profile inference, governed-profile shortcut, or backend callback registry.

The three current governed Metal rows are not grandfathered into this API. They remain absent/fail closed until [`establish-hard-exceptional-value-evidence-for-metal-elementary-realizations`](establish-hard-exceptional-value-evidence-for-metal-elementary-realizations.md) establishes evidence that can discharge both halves.

## Why this is filed rather than done

**Fact — the assessment is on the compile path and its installed set is gated on one profile.** [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](admit-the-registered-unary-families-at-the-compiler-request-boundary.md) wired `assess_program_elementary_accuracy` into `verify_request`, so a program carrying a registered elementary family now requires the target to declare a realization that provably refines the family's contract. `declared_elementary_realizations` in `crates/tiler-compiler/src/target/accuracy.rs` returns `installed_elementary_realizations()` only when the target's `canonical_descriptor()` equals the governed profile's, and an empty set otherwise.

**That gate is correct and it is not the destination.** Every row of `installed_elementary_realizations` is attributed to `governed_profile_source()`, which is the governed profile's own fact source; reading those rows onto another profile would attribute a quoted Metal specification guarantee and a bounded measured corpus to a declaration that never made either. Comparing descriptors rather than keys is what makes the gate unforgeable — a key is a caller-chosen string and the descriptor is the complete declared fact set. So the refusal is right, and what is missing is the *declaration* that would let a caller-built profile answer.

**Fact — the addition is a public boundary and therefore Tom's.** `crates/tiler-compiler/src/target.rs` is `pub mod target`, and `TargetProfileBuilder` is public. A `declare_elementary_realization` method adds public API. **Inference — the atomic-realization precedent in `docs/compiler/optimizer.md` supports one whole subject:** `declare_synchronization_realization` takes one *whole* subject with no per-dimension spelling, so a profile's neighbouring facts cannot compose into a permission for a subject none of them is about. Tom accepted that whole-subject shape above.

**Fact — the failure class is already public; candidate provenance is not.** `RequestError::UnrealizedElementaryAccuracy` reaches a caller as `CompileFailureClass::UnsupportedCapability` carrying the refusing authority's stable key. `target_compile_failure` already maps that request error onto `TargetCompileRefusal::ElementaryAccuracy`. A no-installed refusal can name only the assessed target profile because no declaration exists. An unrefined or undischarged refusal may additionally identify deterministic candidate declaration provenance. That provenance belongs with the declaration it would explain, which is why the remaining refusal work is here.

## Required delivery

- A `TargetProfileBuilder` declaration taking one whole verified elementary-realization subject, validated at build time, encoded into the profile's canonical descriptor, and refusing an exact duplicate. Distinct same-operation contracts remain separate candidates.
- `declared_elementary_realizations` reading the profile's stored declared rows. There is no governed shortcut; the governed profile declares only rows whose two evidence halves discharge.
- The existing `TargetCompileRefusal::ElementaryAccuracy` variant naming the operation, assessed target profile, deterministic candidate provenance where one exists, and the refusing reason, with `no-installed-realization`, `unrefined-realization`, and `undischarged-evidence` kept distinct. The variant itself already exists; this ticket adds the missing candidate provenance, not a second refusal type.
- A negative test: a caller-built profile declaring a realization whose contract does *not* refine the requirement is refused with `unrefined-realization`, observed failing, so a passing declaration test cannot be explained by the assessment ignoring its argument.

## Boundaries

Acceptance of the public method and the public refusal variant is Tom's. A tested implementation is a concrete draft, not implicit approval of its interface.

## Decision packet — 2026-08-09

This packet proposed one whole-subject `TargetProfileBuilder` declaration plus the structured refusal variant, with no per-dimension setters and no governed-profile shortcut. Tom accepted the corrected exact surface in the 2026-08-11 section. That acceptance does not accept new elementary contracts or target evidence.

## Closes when

A caller-built target profile with a verified refining contract and two discharging evidence halves compiles a program containing `tiler::silu-f32@1`; a profile that declares nothing, one that declares only unrefined candidates, and one with an undischarged evidence half refuse through three distinct structured paths. The governed profile does not regain its three current rows until their separate evidence ticket supports them.

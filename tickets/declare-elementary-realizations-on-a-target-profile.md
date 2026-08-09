---
id: declare-elementary-realizations-on-a-target-profile
title: Declare elementary realizations on a target profile
status: awaiting-decision
priority: p2
dependencies: [admit-the-registered-unary-families-at-the-compiler-request-boundary]
related: [carry-the-elementary-numerical-dimensions-in-the-region-realization]
scopes: [implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, numerics, feasibility, public-boundary, decision, needs-tom]
---
## User-visible outcome

A caller that builds its own `TargetProfile` can state which elementary-function accuracy contracts its target realizes, and gets a structured refusal naming the operation when it has not — instead of a profile that is silently unable to compile any program containing `tiler::silu-f32@1`.

## Why this is filed rather than done

**Fact — the assessment is on the compile path and its installed set is gated on one profile.** [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](admit-the-registered-unary-families-at-the-compiler-request-boundary.md) wired `assess_program_elementary_accuracy` into `verify_request`, so a program carrying a registered elementary family now requires the target to declare a realization that provably refines the family's contract. `declared_elementary_realizations` in `crates/tiler-compiler/src/target/accuracy.rs` returns `installed_elementary_realizations()` only when the target's `canonical_descriptor()` equals the governed profile's, and an empty set otherwise.

**That gate is correct and it is not the destination.** Every row of `installed_elementary_realizations` is attributed to `governed_profile_source()`, which is the governed profile's own fact source; reading those rows onto another profile would attribute a quoted Metal specification guarantee and a bounded measured corpus to a declaration that never made either. Comparing descriptors rather than keys is what makes the gate unforgeable — a key is a caller-chosen string and the descriptor is the complete declared fact set. So the refusal is right, and what is missing is the *declaration* that would let a caller-built profile answer.

**Fact — the addition is a public boundary and therefore Tom's.** `crates/tiler-compiler/src/target.rs` is `pub mod target`, and `TargetProfileBuilder` is public. A `declare_elementary_realization` method adds public API, and the atomic-realization precedent in `docs/compiler/optimizer.md` fixes its shape: `declare_synchronization_realization` takes one *whole* subject with no per-dimension spelling, so a profile's neighbouring facts cannot compose into a permission for a subject none of them is about. An elementary declaration owes the same discipline — the operation, the stated contract, the bound evidence, and the exceptional-value evidence arrive as one subject, not as separately settable fields.

**Fact — the structured refusal is withheld for the same reason.** `RequestError::UnrealizedElementaryAccuracy` reaches a caller as `CompileFailureClass::UnsupportedCapability` carrying the refusing authority's stable key, and `target_compile_failure` in `crates/tiler-compiler/src/session.rs` deliberately produces no `TargetCompileRefusal` variant for it. A structured refusal would name the operation and the declaring profile as well — but no public boundary lets a caller-built profile declare a realization today, so the only refusal this build can produce says "this profile declares none", which the key already says. The richer refusal belongs with the declaration it would explain, which is why both are here.

## Required delivery

- A `TargetProfileBuilder` declaration taking one whole elementary-realization subject, validated at build time, encoded into the profile's canonical descriptor, and refusing a duplicate or contradictory row rather than preferring one.
- `declared_elementary_realizations` reading the profile's own declared rows, with the governed profile declaring its three through the same public path rather than through a private table — so the installation test cannot be explained by a governed shortcut.
- A `TargetCompileRefusal` variant naming the operation, the declaring profile, and the refusing reason, with the two reasons (`no-installed-realization`, `unrefined-realization`) kept distinct.
- A negative test: a caller-built profile declaring a realization whose contract does *not* refine the requirement is refused with `unrefined-realization`, observed failing, so a passing declaration test cannot be explained by the assessment ignoring its argument.

## Boundaries

Acceptance of the public method and the public refusal variant is Tom's. A tested implementation is a concrete draft, not implicit approval of its interface.

## Decision packet — 2026-08-09

The exact public boundary is already specified above and should not remain hidden in the implementation queue. Recommendation: accept one whole-subject `TargetProfileBuilder` declaration plus the structured refusal variant, with no per-dimension setters and no governed-profile shortcut. This accepts the declaration/reporting surface only; it does not accept new elementary contracts or target evidence.

## Closes when

A caller-built target profile compiles a program containing `tiler::silu-f32@1`, a caller-built profile that declares nothing refuses with a structured refusal naming the operation, and a caller-built profile that declares an insufficient contract refuses distinctly from one that declares nothing.

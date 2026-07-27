---
id: harden-public-enums-non-exhaustive
title: Harden public growth seams without weakening total maps
status: done
priority: p2
dependencies: [resolve-non-exhaustive-recognizer-hole]
related: [prototype-apple-aot-driver, prototype-scheduled-region-ir, resolve-non-exhaustive-recognizer-hole, harden-kernel-vocabulary-recognizer-completeness, admit-semi-affine-index-expression-class]
scopes: [implementation/ir, implementation/metal-aot, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, api-hardening]
---
Classify selected public enums and output records from their current
construction and match sites so future growth is either compatible by design or
deliberately compile-time breaking.

## Rules

- Mark a type `#[non_exhaustive]` only when every out-of-crate consumer is a
  partial or forwarding consumer that can handle an unknown future value.
- Keep total identity maps, support recognizers, and other closed vocabularies
  exhaustive so a new variant produces a compile error at every authority that
  must classify it.
- For each verdict, record the current construction site and out-of-crate
  consumer that justifies it. Declarations and historical inventories are not
  evidence.

## Search seed

Re-derive the six schedule growth seams, the relevant Metal AOT input/output
types (`AppleSdk`, `OptimizationLevel`, `ArtifactProvenance`,
`CompiledArtifact`), and `IndexExprClass`. The list is a search seed, not a
closed authority; `admit-semi-affine-index-expression-class` depends on this
ticket because `IndexExprClass` currently lacks an additive-growth boundary.

Do not bundle unrelated doctest cleanup into this work.

## Closes when

Each selected type is classified from current call sites, compatible growth
seams are non-exhaustive, total maps and recognizers remain deliberately
exhaustive, negative compile coverage protects both directions, and `make full`
passes.

## Outcome — four types classified from call sites; two questions settled (2026-07-27)

Each verdict below names the out-of-crate consumer that justifies it, per this ticket's own rule that declarations are not evidence.

| type | verdict | evidence |
| --- | --- | --- |
| `AppleSdk` | **`#[non_exhaustive]`** | Out-of-crate use is construction only — `tiler-metal/src/golden_compilation.rs:151,222`, `prototypes/serial-sum-compile/src/target.rs:63-65,186-188`, `prototypes/serial-sum-run/src/proof.rs:1929`. No out-of-crate exhaustive match: `sdk_for` matches `MetalPlatform` and *produces* an `AppleSdk`. |
| `OptimizationLevel` | **`#[non_exhaustive]`** | Construction only — `golden_compilation.rs:194`, `serial-sum-compile/src/main.rs:230,347`, `serial-sum-run/src/proof.rs:1933`. |
| `ArtifactProvenance` | **`#[non_exhaustive]`** | A driver output. `tiler-metal` and the serial-sum producer read its fields; neither builds one. |
| `CompiledArtifact` | **`#[non_exhaustive]`** | Same: read out-of-crate, constructed only in `tiler-metal-aot`. |
| `IndexExprClass` | **left exhaustive, deliberately** | see below |

**`IndexExprClass` has no out-of-crate consumer.** It is exported through `index/mod.rs` and matched at six sites, all inside `tiler-ir`. This ticket's rule is that a verdict needs a current out-of-crate consumer as evidence, and there is none — so marking it would be a type-system reservation protecting nobody, which `AGENTS.md` warns is a different maturity claim from an implemented seam. **This settles the question `admit-semi-affine-index-expression-class` depends on rather than deferring it:** the growth boundary does not arise until an out-of-crate consumer exists, and until then the six internal matches are the authority and must stay exhaustive, so a new class is a compile error at every site that classifies one.

**Negative coverage, and it caught its own mistake.** Two `compile_fail` doctests — doctests compile as separate crates, so `#[non_exhaustive]` applies to them where it would not apply to a unit test in the same crate. `E0004` for an out-of-crate exhaustive match on `AppleSdk`, `E0639` for an out-of-crate `CompiledArtifact` literal, each beside a positive case proving construction and field reads still compile.

The first attempt failed for the *wrong reason* — `E0432`, an unresolved import, because `AppleSdk` is exported at `tiler_metal_aot::input::AppleSdk` and not at the crate root. Naming the expected error code is what surfaced that; a bare `compile_fail` would have passed while testing nothing, which is the failure mode a negative test is most prone to.

**Fact: the workspace compiled unchanged after the four attributes were added**, which independently confirms the call-site evidence — no consumer was doing anything `non_exhaustive` forbids.

## Not covered

The search seed also named "the six schedule growth seams". Those are not classified here. They are a distinct vocabulary with their own consumers and deserve the same call-site-by-call-site treatment rather than being swept in on the strength of these four; the seed says in terms that it is not a closed authority. Split as `classify-the-schedule-growth-seams`.

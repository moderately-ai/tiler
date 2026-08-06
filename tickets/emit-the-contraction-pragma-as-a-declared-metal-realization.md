---
id: emit-the-contraction-pragma-as-a-declared-metal-realization
title: Emit the contraction pragma as a declared Metal realization
status: deferred
priority: p2
dependencies: []
related: [declare-the-metal-emitted-pragma-unfused-realization, probe-the-bf16-contraction-pragma-on-the-metal-runtime-path, admit-bf16-into-the-schedule-and-kernel-vocabulary]
scopes: [implementation/metal]
shared_scopes: []
paths: []
tags: [numerics, bf16, contraction, metal, apple-targets]
---
## Activation trigger

Activate when a `Forbidden` contraction resolution must be honoured on a Metal path where the offline `-ffp-contract=off` selection is not the discharge — the first of: a Metal profile is offered a BF16 (or any) contract declaring contraction forbidden and the delivered-realization record must state *how* it was honoured; or a compilation route reaches the Metal emitter without the strict baseline flag row.

The authorization already exists and is not what this ticket waits on. Tom decided on 2026-08-01 that the emitter may write `#pragma METAL fp contract(off)`, recorded with its provenance, derivation, boundary, and checked assumption in [the Metal backend contract](../docs/backends/metal.md#the-emitted-contraction-pragma-is-a-declared-realization-not-an-inherited-default). What is deferred is the implementation, because today the offline flag already discharges the requirement on every Tiler path (finding 28) and `newLibraryWithSource:options:` is on no Tiler path at all under ADR 0002.

## Required outcome

`realization_requirements` in `crates/tiler-metal/src/emit.rs` today discharges `NumericalPermission::Forbidden` on contraction by reporting `MetalNumericalRequirement::NoFloatingPointContraction`, a compiler-flag requirement. Move that discharge — or add the second one — into the emitted source: the translation unit carries the file-scope pragma, the emitted realization records `HonouringMeans::SupportedWithExactEmulation` on `numerics.contraction` for the affected scalar-arithmetic subject rather than `SupportedExactly`, and the declaration names runtime compiler build `metalfe-32023.921` as the environment the behaviour was measured on. A snapshot test must show the emitted bytes containing the pragma exactly once and a perturbation must show the realization record changing means, not only text.

Before landing, re-check the assumption the contract's carve-out rests on: no kernel this backend can emit contains an operation whose contract requires fusion. Reproduce with `grep -n 'pub enum BinaryOp' -A 80 crates/tiler-ir/src/kernel/model.rs`. If a fused variant has appeared, the file-scope placement is no longer admissible unmeasured, and the contract section's "Reopens when" clause governs: measure the pragma against a written source-level `fma`, move to block scope, or split the unit.

## Trigger check log

- 2026-08-06 — **not fired.** No contract declaring contraction forbidden is offered to a Metal profile: `ScalarArithmetic` rows for BF16 are not projected into any production profile (see the bounded F32 adapter's stated exclusion in `docs/backends/metal.md`), and every Tiler compilation path selects the strict baseline flag row whose `-ffp-contract=off` already discharges the requirement offline. Recheck: `grep -rn 'NoFloatingPointContraction' crates/tiler-metal/src`.

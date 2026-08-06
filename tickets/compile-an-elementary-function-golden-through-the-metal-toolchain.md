---
id: compile-an-elementary-function-golden-through-the-metal-toolchain
title: Compile an elementary-function golden through the Metal toolchain
status: todo
priority: p2
dependencies: []
related: [admit-the-registered-unary-families-at-the-compiler-request-boundary, admit-the-silu-activation-family, emit-the-structural-region-on-metal]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, goldens, numerics, support-matrix]
---
## User-visible outcome

A golden carrying `tiler::silu-f32@1`'s emitted body — `precise::exp` and the division operator — compiles and links through the real Apple offline toolchain beside the other nine, and a SiLU translation unit is observed *accepted* by a declared numerical realization rather than only refused by one. Together those are the two facts the activation's support-matrix row needs before it can claim R6.

## Why this exists

**Fact — the emission evidence is a string assertion and nothing more.** `the_silu_kernel_emits_the_precise_exponential_and_a_division` (`crates/tiler-metal/src/tests.rs:1810`) drives `emit_translation_unit` over a `VerifiedKernel` carrying the activation's expression and asserts exactly one `precise::exp(` and one division operator, with `fast::exp`, `fast_exp`, `metal::divide(`, and a reciprocal spelling absent. Nothing compiles that text. The exact check is `grep -rn 'precise::' crates/tiler-metal/goldens/`, which returns nothing, against `golden_compilation.rs`, which names nine goldens — none carrying an exponential or a division.

**Fact — the only recorded realization outcome for a SiLU unit is a refusal.** `the_silu_kernel_records_the_f32_subnormal_gap` (`:1872`) asserts a non-empty gap set and `require_declared_realization().expect_err()`, because the fixture declares subnormal preservation on the measured flushing row. No test observes a SiLU unit accepted. The BF16 fixture already shows the shape the acceptance half should take: `a_strict_bf16_contract_is_refused_on_the_measured_macos_row` asserts the strict refusal *and* then emits under a flush-honouring declaration and unwraps `require_declared_realization`, so the refusal is a decision about the contract rather than a blanket one.

**Inference — this is the class of claim only compilation catches.** `lower-bf16-to-metal` found `as_type<bfloat>(0x4000u)` rejected at the `metal` stage and kept the narrowing as a compile requirement rather than a style; without its golden that carrier would have been asserted correct by a string match. `precise::exp` is a namespace-qualified call under the governed flags and is exactly as unverified today.

## Implementation keys

- A golden emitted from the existing `silu_kernel` fixture, added to `golden_compilation.rs`'s list, compiling and linking under the same flags and on the same recorded toolchain row the other goldens use — and the row recorded, not assumed, because that row is deliberately not the compile-profile authority ledger's.
- A perturbation that is observed failing, in the shape the BF16 golden's carrier check takes: something about the emitted elementary spelling that the toolchain rejects when removed, so the compile evidence is not vacuous.
- The acceptance half: the SiLU unit emitted under a flush-honouring declared realization, with an empty gap set and `require_declared_realization` succeeding, beside the existing refusal.

## Non-goals

Dispatching anything. Putting a compiler-derived region through `emit` — `tiler-metal` cannot depend on `tiler-compiler`, which is a separate residual the structural row also carries. Moving the support-matrix row: `docs/roadmap.md` is `contracts/navigation`, which this ticket does not hold, so the rung move is reported for a navigation ticket rather than written here.

## Closes when

An elementary-function golden compiles and links on a recorded toolchain row with a perturbation observed failing, a SiLU unit is observed accepted by a declared numerical realization, and the support-matrix consequence is reported to the coordinator rather than claimed in a scope this ticket does not hold.

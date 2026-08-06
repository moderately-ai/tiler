---
id: compile-an-elementary-function-golden-through-the-metal-toolchain
title: Compile an elementary-function golden through the Metal toolchain
status: review
priority: p2
dependencies: []
related: [admit-the-registered-unary-families-at-the-compiler-request-boundary, admit-the-silu-activation-family, emit-the-structural-region-on-metal]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, goldens, numerics, support-matrix]
claimed_from: todo
assignee: agent-elementary-golden
lease_expires_at: 1786034532
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

## Outcome

**All three implementation keys are delivered, and the compile evidence went one step further than the ticket asked: the linked library was read for *which* exponential it references, which turns the row's "never `air.fast_exp.f32`" claim from a source assertion into a measurement.**

### The golden

`crates/tiler-metal/goldens/elementary_silu_activation.metal`, emitted from the existing `silu_kernel` fixture under the same strict declared realization (`tiler.test.strict-f32`) and the same emitter facts as every other golden, and added to `GOLDENS` in `golden_compilation.rs` (nine → **ten**). It is the only fixture whose body calls a function and divides; every other golden's arithmetic is `*`, `+`, and comparison, which is why it is the first checked-in artifact whose acceptance depends on a *name* resolving. `crate::tests::silu_matches_its_golden_source` pins its bytes; `every_checked_in_golden_is_compiled_by_this_module` makes the directory-to-list correspondence a failing assertion rather than a convention.

Body of interest, emitted exactly as the string assertions claimed: `float v8 = precise::exp(v7);` and `float v12 = v3 / v11;`, each arithmetic result canonicalized. **The toolchain did not reject it**, so the stop condition did not fire and the emitter is untouched by this ticket.

### Measurement — toolchain row, recorded from the host at run time

Read from `xcodebuild -version`, `sw_vers`, `xcrun --sdk macosx --show-sdk-version`/`--show-sdk-build-version`, `sysctl -n machdep.cpu.brand_string`, and the harness's own resolution line, not copied from a neighbouring ticket:

| Field | Value |
| --- | --- |
| Host | Apple M4 Max, macOS 27.0 (build 26A5388g) |
| Xcode | 27.0 (build 27A5228h) |
| `metal` | Apple metal version 32023.921 (`metalfe-32023.921`) |
| `metallib` | AIR-LLD 32023.921 (`metalfe-32023.921`) |
| SDK | macOS 27.0 (build 26A5388f) |
| Flags | `-target air64-apple-macos14.0 -std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off` |

`elementary_silu_activation.metal` links **3,779 bytes**; the linked library names `tiler_kernel_b1e08c4feb69be47`. All ten fixtures compile and link on this row. **This is deliberately not the [compile-profile authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)'s row** — that ledger sources its profile from `32023.883` and excludes `metalfe-32023.921` by name — which is why the row is recorded here rather than inherited.

### The perturbations, both observed failing before being trusted

- **`the_elementary_golden_without_its_operand_is_rejected_when_a_toolchain_resolves`.** Replacing `precise::exp(v7)` with `precise::exp()` is rejected at the `metal` stage: `error: no matching function for call to 'exp'`, the diagnostic naming the `metal_math` candidates (`METAL_FUNC float exp(float x)` and the three vector overloads) it resolved against. That is what makes the acceptance a binding to a declared function rather than a parse — the analogue of the BF16 carrier check at the construct only this fixture has.
- **`the_precise_namespace_survives_a_fast_row_when_a_toolchain_resolves`.** The AIR intrinsic is named in the linked library, so the selection can be observed rather than argued. Measured two-by-two on the row above:

  | | governed row | `-fmetal-math-mode=fast -fmetal-math-fp32-functions=fast` |
  | --- | --- | --- |
  | `precise::exp(v7)` | `air.exp.f32`, 3,779 bytes | `air.exp.f32`, 3,955 bytes |
  | `exp(v7)` | `air.exp.f32`, 3,779 bytes | **`air.fast_exp.f32`**, 3,971 bytes |

  Both governed-row libraries are **byte-identical**, so no compilation under the flags Tiler selects can distinguish the two spellings — that is the honest bound on every other compile test in the module, and it is asserted so the next reader cannot cite the golden's compilation as evidence for the namespace. The fast column is the divergence, and it is the measurement that `emit.rs`'s stated reason for writing `precise::` is real: the namespace, not the flag, is what holds the contracted intrinsic when the flag is absent. The fast row is a perturbation, not a supported configuration; nothing in Tiler compiles under it.

**Each new check was watched failing.** The operand perturbation pointed at `exp(v7)` (which compiles) fails with "a nullary call to a unary elementary function must not compile". The namespace test compiled with `unqualified` in the qualified position fails with "the emitted namespace must hold the precise selection without the flag". The golden test against a one-space-drifted fixture fails as stale. The acceptance half with `SubnormalMode::Preserve` substituted for the flush fails at `honoured.numerical_gaps().is_empty()`.

### The acceptance half

`the_silu_kernel_records_the_f32_subnormal_gap` is now two-sided, in the shape `a_strict_bf16_contract_is_refused_on_the_measured_macos_row` takes at the other width. The existing refusal is unchanged; beside it the same SiLU kernel is emitted under `tiler.test.flush-f32` — a declaration moving **only** the two subnormal dimensions to a sign-preserving flush, with contraction and reassociation still forbidden — against the same measured flushing target. The gap set is empty, `unstated_subnormal_arithmetic()` is empty (so the empty gap set is a decision, not an incomplete computation), and `require_declared_realization()` succeeds. The accepted unit is asserted to still carry `precise::exp` once and to still place `PreciseFp32Functions`, so the acceptance is about a *SiLU translation unit* rather than about arithmetic that happens to share its shape. `silu_kernel_under(realization)` was factored out for it; `silu_kernel()` is that helper at the strict realization, so no existing caller changed.

This makes the refusal a decision about the declared realization rather than about elementary functions — which is the distinction a reader of the refusal alone could not make, and the reason the ticket asked for the second verdict.

### Support-matrix consequence — reported, not claimed

`docs/roadmap.md:469` (`Elementwise activation: tiler::silu-f32@1`) states its own R6 criteria: "a translation unit carrying this family's exponential and division observed accepted by a declared numerical realization, and compiled and linked through the Apple toolchain the way every other R6 row in this table was". **Both conjuncts are now supported**, so the row moves **R5 → R6**, bounded to offline translation on the one measured row above, with R7 unmet (no dispatch, and no compiler-derived region through `emit` — `tiler-metal` cannot depend on `tiler-compiler`).

`docs/roadmap.md` is `contracts/navigation` (read from `ticketsplease.toml`), which this ticket does not hold, so nothing there was touched. Filed as [`move-the-elementary-activation-row-to-r6`](move-the-elementary-activation-row-to-r6.md), which also carries the correction a sweep would otherwise miss: the cell's two embedded reproduction commands are now false — `grep -rn 'precise::' crates/tiler-metal/goldens/` returns hits, and the list names ten goldens, not nine.

### Non-goals held

Nothing dispatched. No compiler-derived region put through `emit`. No emitter change. No navigation edit.

### One out-of-scope observation, not filed

`FpContract::FastHonorPragmas` in `tiler-metal-aot` is rejected by this toolchain: `metal: error: unsupported argument 'fast-honor-pragmas' to option '-ffp-contract='`, at the `metal` stage on Metal 32023.921. Found while constructing the fast row (`FpContract::Off` was used instead, which also keeps the perturbation about function selection alone). The driver fails closed with a typed `ToolFailure`, so this is not a correctness defect, but the enum offers a selection this row cannot deliver and nothing records that. `implementation/metal-aot` is not held here; reported for the coordinator rather than filed, since whether that is a documentation fix or a validation gap is that scope's decision.

### Checks

`cargo fmt --check -p tiler-metal`; `cargo check -p tiler-metal --all-targets`; `cargo clippy -p tiler-metal --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-metal`; `TILER_REQUIRE_METAL_TOOLCHAIN=1 cargo nextest run -p tiler-metal` → **115 tests run: 115 passed, 0 skipped**; `cargo test -p tiler-metal --doc` → 3 passed + 4 compile-fail passed; `git diff --check` clean; `tkt lint` clean after every ticket edit; `tkt guard --base 1a2d8b26` after committing.

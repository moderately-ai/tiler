---
id: emit-the-structural-region-on-metal
title: Emit the structural region on Metal
status: done
priority: p1
dependencies: []
related: [reach-a-verified-kernel-through-the-structural-families, admit-the-structural-families-into-the-scheduled-region-vocabulary, own-operation-family-support-matrix]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, structural, backend]
---
## User-visible outcome

A structural region — a widening broadcast read, or a reindexed operand feeding a pointwise neighbour — reaches emitted Metal source, so the structural row's R6 rung rests on a backend that emits it rather than on a vocabulary that could.

## Why this is separate from the compiler-side vertical

**Fact — the vertical stops one crate short, and the residual is small and named.** [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md) carried both families from `compile()` to a `VerifiedKernel` whose result is the reference evaluator's bit for bit. What it could not carry is emission: `tiler-metal` depends on `tiler-ir` and `tiler-artifact` and *not* on `tiler-compiler`, so its fixtures build regions by hand and no test in this repository puts a compiler-derived structural region through `emit`. That is `implementation/metal` scope, which the compiler-side ticket does not hold.

**Fact — one construct is definitively unemittable, and nothing catches it.** `emit_binary` in `crates/tiler-metal/src/emit.rs` maps `IndexAdd`, `IndexMultiply`, `IndexDivide`, `IndexModulo`, `I32Subtract`, the `f32`/`bf16` arithmetic, and `F32Maximum`, and falls through every other tag to `MetalEmitError::UnsupportedOperation { family: Binary }`. `BinaryOp::IndexSubtract` — appended at tag `0x0c` by [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) — has no arm. It is emitted by exactly one producer, `emit_offset` in `crates/tiler-ir/src/kernel/lower.rs:2093`, for a reindex mirror's `extent - 1 - c`, so the `reverse-axis` form verifies as a kernel and refuses at the backend. Reproduce:

```sh
rg -n 'IndexSubtract' crates/
```

Four hits, none of them under `crates/tiler-metal/`.

**Fact — every *other* ingredient is already emitted, which is what makes this bounded.** `crates/tiler-metal/goldens/reduction_multi_axis.metal` emits `/`, `%`, `*`, and `+` over `uint64_t` structured index arithmetic (lines 59–65), which is exactly the construct set a broadcast replication decode and every non-mirrored reindex decode produce. `crates/tiler-metal/goldens/contraction_strict_tensor.metal` declares two read buffers at *different* element counts (8 and 12) beside one write, which is the signature shape a widening broadcast needs. So the residual is the region never having been run through the emitter, plus the one missing arm — not a vocabulary redesign.

## Required delivery

- **An `IndexSubtract` arm in `emit_binary`**, with the unsigned-wrap question answered rather than inherited: the KIR operation's contract is that its result is *proven* non-negative (the compiler's own test interpreter asserts `lhs >= rhs` rather than wrapping), and `uint64_t` subtraction in MSL wraps silently. Decide, and record, whether the emitter asserts, emits a widened signed form, or rests on the producer's proof — and say which in the emitted comment block if it rests on a proof.
- **At least one golden covering a structural region**, added to `crates/tiler-metal/src/golden_compilation.rs`'s complete-directory list so it compiles through the offline driver. A broadcast replication and a mirrored reindex are the two shapes worth pinning; the mirror is the one the new arm exists for.
- **A totality check over `BinaryOp` reachable from lowering**, or an explicit statement of why one is not wanted. Today a tag appended in `tiler-ir` reaches the backend's wildcard arm with nothing red, which is how this gap arrived.

## Non-goals

Device dispatch (that is R7 and belongs with the runtime integration), any new `LogicalAccess` relation, and the compiler-side recognition — all three are delivered or separately owned.

## Closes when

A structural region emits Metal source that compiles through the offline driver, the `reverse-axis` form among it, and the structural row's rung records what a backend now actually emits.

## Outcome

**Both structural relations now reach compiled Metal, and the construct that blocked the mirror is emitted with its contract stated in the emitted text rather than inherited.** The residual this ticket named was real and was exactly one arm wide; what it did not anticipate is that the wrap hazard has *two* defences and only one of them belongs to this backend, which is the finding that shaped the delivery.

**Fact — the arm.** `BinaryOp::IndexSubtract` has a realization in `crates/tiler-metal/src/emit.rs`. The mapping moved out of `emit_binary` into `binary_realization`, a pure function of the operation, so it can be exercised over the whole vocabulary; `emit_binary` matches on the resulting `BinaryRealization` and the emitted bytes of every pre-existing fixture are unchanged.

**Fact — the non-wrap derivation, recorded on `binary_realization`.** The index role spells `uint64_t`, MSL subtraction on it is modular, and a violated proof produces an index near `2^64` that the next statement scales into a buffer subscript rather than trapping. Two candidates were eliminated rather than weighed: a clamped `minuend - min(minuend, subtrahend)` keeps the index in range by reading a *different element*, which is the silently wrong result the fail-closed rule forbids, and a widened signed form converts defined wrapping into C++ signed overflow and still yields a huge unsigned index. The plain difference is emitted, and the proof it rests on is stated: the mirror is `extent - 1 - c`, and `c < extent` reaches the emitted body either from the decode's own `%` (whose divisor *is* that extent) or, where `linearize_axis_decodes` elides that wrap as redundant for a leading window, from the emitted `if (index < element_count)` domain guard together with `c = index / divisor` and `divisor * extent == element_count`. Neither derivation is recomputed in the emitter — reconstructing an access relation is what `emit.rs`'s own contract forbids — so the emitted statement carries `// unsigned; v8 <= v9 by the IR's proof, not by a test`, which attributes the bound instead of implying a check happened.

**Fact — what the backend does check.** `constant_minuend` refuses an index subtraction whose minuend is not an index constant, the shape the IR's own contract describes (`the left operand is the constant extent − 1`). This is defence in depth on the `non-positive-divisor` model, not the thing standing between the mirror and a wrapped index.

**Fact — the wrap perturbation is caught in `tiler-ir`, not here, and that was worth learning.** A body whose mirror subtracts in the other order fails the structured kernel verifier's body-refinement check with `KernelDiagnostic::BodyRefinement`; the exchange therefore never reaches emission through `lower_scheduled_region`. It was reached experimentally, by building the exchanged body through the public `KernelBuilder` and watching `build()` refuse it, and is now pinned by `the_verifier_refuses_a_reordered_mirror_before_emission_sees_it`. **Inference** — this is why no compile-stage perturbation can carry the wrap claim: `c - (extent - 1)` is well-formed MSL that translates and links cleanly, so a `metal` diagnostic is structurally incapable of seeing it. The compile-stage perturbation added instead targets the *bound*: deleting the emitted `%` leaves the difference referring to an undefined identifier and is rejected at the `metal` stage.

**Fact — two goldens, not one.** `structural_mirrored_reindex.metal` is the `reverse(a)` program on a `[2, 2]` operand reversing axis 1 — the same shape, axis, and admitted form as the compiler's `a_reindex_reaches_a_kernel_matching_the_reference_evaluator`, restated by hand because `tiler-metal` cannot depend on `tiler-compiler`. `structural_widening_broadcast.metal` widens a `[2]` operand across a `[2, 2]` result and is the only one-read fixture whose read and write buffers declare different element counts. Both are the first goldens carrying no arithmetic at all, so they are the only checked-in artifacts pinning the *empty* unrealizable-obligation provenance block.

**Fact — the totality check exists and is a build error, not a test.** `every_binary_construct_has_a_metal_realization` declares its array at `core::mem::variant_count::<BinaryOp>()`, the mechanism `applicability::MetalGpuFamily::ALL` already established and the reason `#![feature(variant_count)]` is on the crate. Omitting one construct is a `[BinaryOp; 12]` vs `[BinaryOp; 11]` type error; a repeated construct is caught by the distinctness assertion beside it. `BinaryOp` is `#[non_exhaustive]`, so `rustc` will never close the match itself — which is how this gap arrived.

**Measurement — offline compilation, Apple M4 Max, macOS 27.0 (build 26A5388g), Xcode 27.0 (build 27A5228h), Metal 32023.921 (`metalfe-32023.921`, AIR-LLD 32023.921), macOS SDK 27.0 (build 26A5388f).** All nine fixtures compile and link under `-target air64-apple-macos14.0 -std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`. `structural_mirrored_reindex.metal` links 3,555 bytes naming `tiler_kernel_c757efe9a62e2e41`; `structural_widening_broadcast.metal` links 3,555 bytes naming `tiler_kernel_27edf18e107d5dde`. Reproduce with `TILER_REQUIRE_METAL_TOOLCHAIN=1 cargo nextest run -p tiler-metal -E 'test(golden_compilation)' --no-capture`. **This is a translation fact and not a dispatch claim**: it says the mirrored `uint64_t` divide/wrap/subtract/scale/add chain translates and links, and says nothing about the values a device computes — a wrapped index compiles just as cleanly.

**Fact — pins.** No existing pinned identity moved. The seven pre-existing `goldens/*.metal` files are byte-unmodified (`git status` shows two additions and no golden modification) and their byte-comparison tests pass; `tiler-build`'s `the_standard_metal_path_publishes_its_recorded_identities` still holds `d22c0d11…` / `6dee9552…`. The two new goldens are the eighth and ninth entries, additive. The emitted-numerics prologue was deliberately not touched; `widen-the-emitted-numerics-prologue-past-one-width` still owns that identity step.

### Residual

The ticket's non-goals stand: device dispatch of a structural region is R7, and no compiler-side test puts a *compiler-derived* structural region through `emit` — `tiler-metal` still cannot depend on `tiler-compiler`, so the two fixtures are restatements verified by shape rather than by import. Closing that would mean an integration test in a crate that may depend on both.

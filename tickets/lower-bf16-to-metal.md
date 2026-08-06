---
id: lower-bf16-to-metal
title: Lower a BF16 kernel to Metal and dispatch it on the measured macOS row
status: review
priority: p1
dependencies: [admit-bf16-into-the-schedule-and-kernel-vocabulary, declare-the-bf16-rows-on-the-authoritative-metal-profile]
related: [spike-bf16-through-the-second-dtype-seams, measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, widen-the-f16-operation-vocabulary-to-contraction-and-reassociation]
scopes: [implementation/metal, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, metal, lowering, apple-targets]
claimed_from: todo
assignee: agent-bf16-metal
lease_expires_at: 1785982711
---
## User-visible outcome

A verified BF16 kernel emits `bfloat` MSL, compiles, dispatches on the measured macOS row, and returns results that agree with `tiler-reference` within the target's declared numerical realization. The same kernel is refused before submission on the iOS Simulator.

## What the target already fixes

**Fact.** `MetalFloatArithmeticType::Bf16` exists in `crates/tiler-metal/src/target.rs` and already carries the measured BF16 flush in its own slot, inheriting nothing from `f32`.

**Fact.** `msl_type` maps `KernelType::F32` to `"float"`; the BF16 spelling is `bfloat`. `KernelConstant::F32Bits` emits `as_type<float>(0x...u)`; BF16 needs its own reinterpretation, and MSL's `ushort` is the carrier the Apple probe harness uses for it.

**Measurement — the flush is not optional and the agreement must account for it.** Finding 24 records that BF16 arithmetic on the macOS row **flushes** subnormal operands and results, sign-preserving. A reference that preserves subnormals will therefore disagree with the device on exactly the subnormal cases, and that disagreement is *correct behaviour on both sides*. The comparison must apply the declared `SubnormalMode` rather than expecting bit equality everywhere, and `ReferenceNumericalConformance` is the existing mechanism for that.

**Measurement — no fused form exists.** Finding 29 records that `metal` rejects `bfloat v6 = fma(v3, v4, v5)` with "cannot initialize a variable of type 'bfloat' with an rvalue of type 'float'". There is no `bfloat` overload of `fma`, so a BF16 contraction cannot lower to one.

**Measurement — contraction defence differs from `f16`.** Finding 28 records that under `safe` with `-ffp-contract=fast`, `f16` fuses and `bf16` does not. Do not carry an `f16` contraction conclusion across to BF16.

## Implementation keys

- `msl_type` gains `bfloat`, and BF16 constants emit through the `ushort` reinterpretation rather than `float`. **It will already have a BF16 arm when this ticket starts, and that arm is a refusal.** `admit-the-bf16-type-and-carrier-into-every-total-map` makes `msl_type` fallible and rejects BF16 by name, because `KernelType` is not `#[non_exhaustive]` and `crates/tiler-metal/src/emit.rs:812` stops compiling the moment the variant exists — and spelling `bfloat` there would have published an unmeasured capability while this ticket's profile dependency was still blocked. Replacing that refusal with the spelling is this ticket's job, and doing so is only admissible once the measured MSL 4.0 row is declared.
- BF16 binary operations map to the operator and to `MetalFloatArithmeticType::Bf16`, so the subnormal obligation is recorded against the right dtype. The existing machinery already refuses to answer an unstated dtype from a neighbour's fact; do not weaken it.
- A BF16 NaN canonicalization helper, distinct from `tiler_canonicalize_nan_f32_7fc00000`. The Apple harness's mangled name for the BF16 helper is in its recognizer and is the shape to match.
- Emission refuses when the target states no BF16 subnormal fact — the `Unknown` path, which is what the iOS device gets.
- `-ffp-contract=off` remains the contraction defence, measured at BF16 as well as `f32`.

## Required evidence

- A BF16 kernel emits, compiles offline, dispatches on macOS, and agrees with `tiler-reference` on every element **after** the declared flush is applied to the reference — with the subnormal elements shown to be the ones the flush moves, not silently excluded.
- An execution witness on a non-subnormal operand reports `executed`. Without it, "flushed" and "the arithmetic was optimized away" are the same observation.
- The same program is refused for the iOS-Simulator profile before any submission, by the dispatchability fact rather than by a pipeline failure.
- A target stating no BF16 subnormal fact refuses emission with the unstated-fact diagnostic, observed failing.
- A strict subnormal-preserving contract is refused on the macOS row with a named numerical gap.
- The F32 golden compilation is unchanged.

## Closes when

A BF16 kernel dispatches on the measured macOS row and agrees with the reference under the declared realization, the simulator refusal happens before submission, every refusal above is observed failing, the execution witness is present, and the `Backend lowering` and `Backend execution` cells for BF16 move with their host/toolchain boundary stated.

## Graph maintenance

- Depends on the kernel vocabulary and on the profile carrying the BF16 rows; emission consults the target fact and would fail closed without it.
- Does not depend on the artifact ticket: offline emission and dispatch do not need the artifact round trip, and keeping them independent lets the two land in parallel.
- Nothing here may claim an iOS-device BF16 result. That family is `Unknown` and only `measure-apple-numerics-on-physical-ios-device` can close it.
- Contraction, reassociation, and FMA are out of scope, and finding 29 makes the last one unimplementable at the source level. `design-the-bf16-computation-and-accumulator-contract` owns that question.

## Outcome (2026-08-05)

**A BF16 kernel now emits `bfloat` MSL and compiles and links through the real Apple offline toolchain, and the dispatch half of this ticket's evidence is not reachable from the scopes it holds.** The lowering, the constant carrier, the per-width canonicalization helper, and all four refusals landed and are observed failing. Device dispatch, the reference comparison, the execution witness, and the dispatchability refusal did not, because none of them is reachable from `implementation/metal` + `implementation/metal-aot` — the derivation is below, and the work is already owned by this ticket's two dependents rather than absent from the graph.

### Fact — what landed

`crates/tiler-metal/src/emit.rs`:

- `msl_type` spells `KernelType::Bf16` as `bfloat`, replacing the named refusal. The `Err` arm is now vacant and the signature stays fallible, because it is the seam a widened `KernelType` must land on; `MetalEmitError::UnsupportedValueType` is kept and `the_unspelled_value_type_refusal_keeps_its_rule_and_rendering` exercises its rule and rendering directly, so the retained variant is not an unchecked surface.
- `KernelConstant::Bf16Bits` emits `as_type<bfloat>(ushort(0x…u))`. Never through `float`: `as_type` requires equal sizes and an unsuffixed MSL integer literal is `uint`, so the narrowing is a compile requirement rather than a style, measured below.
- `BinaryOp::Bf16Add` and `BinaryOp::Bf16Multiply` map to `+`/`*` and to `MetalFloatArithmeticType::Bf16`, so `record_subnormal_obligation` reads the BF16 row.
- `MetalHelper::CanonicalizeBf16Nan { bits: u16 }` is a separate variant from the binary32 one, emitting `static inline bfloat tiler_canonicalize_nan_bf16_7fc0(bfloat value)` with a `ushort` integer predicate. `bf16_canonical_nan` narrows the 32-bit declared payload and refuses a high-half-set or non-NaN pattern.
- The declared canonical NaN reaches the helper through the region's own field; `ConvertOp::CanonicalizeBf16Nan` never reaches the `f32` helper.

`crates/tiler-metal/goldens/pointwise_scale_bias_bf16.metal` is new, and `crates/tiler-metal/src/golden_compilation.rs` compiles it with the other six.

### Fact — the helper name matches the Apple harness recognizer

The harness derives `tiler_canonicalize_nan_bf16_7fc0` (`spikes/apple-targets/numerical_probe.py:587`, BF16 row at `:753`) and its recognizer pins the C++-mangled `_ZL32tiler_canonicalize_nan_bf16_7fc0DF16b` (`spikes/apple-targets/test_numerical_probe.py:1031`), whose `32` is the identifier's length and whose `DF16b` is the `bfloat` parameter. The emitted symbol is that identifier character for character, and the test asserts its length is 32 from the string itself rather than from a copied number. **This is a name-shape agreement, not a run**: nothing here dispatches, and nothing here shows the harness classifying a module this backend emitted.

### Measurement — offline compilation

Apple M4 Max, macOS 27.0 build 26A5388g, Xcode 27.0 build 27A5228h, Metal 32023.921 (`metalfe-32023.921`, AIR-LLD 32023.921), macOS SDK 27.0 build 26A5388f. Under `-target air64-apple-macos14.0 -std=metal3.1 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`, all seven goldens compile and link; the BF16 fixture links 3,635 bytes and the library names `tiler_kernel_7c905e3938dc8d91`.

**The compile row and the numerical row are different rows, deliberately.** The golden family's governed target is MSL 3.1 / macOS 14.0; the authoritative profile's measured BF16 subnormal row is MSL 4.0 / `air64-apple-macos26.0` (`crates/tiler-build/src/metal_declaration.rs:281`). Finding 24's original table was also taken at `-std=metal3.1`, so `bfloat` compiling there is not an extrapolation — but nothing here re-measures the numerical row, and the module documentation says so.

### Measurement — the carrier is load-bearing, observed failing

Stripping the narrowing from the golden's constant (`as_type<bfloat>(ushort(0x4000u))` → `as_type<bfloat>(0x4000u)`) is rejected at the `metal` stage:

```
error: as_type cast from 'unsigned int' to 'bfloat' is not allowed
```

`the_bf16_golden_without_its_ushort_carrier_is_rejected_when_a_toolchain_resolves` holds it. Without this the BF16 compile evidence would be vacuous — it would show the toolchain accepting a file, not that the carrier this backend emits is the reason it is accepted.

### Fact — every refusal, observed failing

- **Strict subnormal-preserving BF16 contract on the measured macOS row** → `MetalNumericalGap::SubnormalFlushInArithmetic`, `require_declared_realization` errs with `UnrealizableNumericalObligation`, and the gap is in the emitted header. `a_strict_bf16_contract_is_refused_on_the_measured_macos_row`, which also shows the flush the row *does* deliver being honoured, so the refusal is a decision about the contract and not a blanket one.
- **Unstated BF16 fact** → `unstated_subnormal_arithmetic() == [Bf16]`, empty gap list, `unstated-subnormal-arithmetic: bf16`. `an_unstated_bf16_fact_refuses_the_unit_naming_the_dtype`.
- **iOS-Simulator profile** → same refusal, before any compilation, with an `f32` kernel on the same profile unaffected. `the_ios_simulator_profile_refuses_a_bf16_unit_before_any_compilation`.
- **Non-`bf16`-NaN canonical payload, including the binary32 canonical NaN** → `InvalidCanonicalNan`. `only_a_bf16_nan_encoding_is_accepted_as_a_canonical_bf16_nan`. The binary32 case is the one that matters: `0x7fc00000`'s low half is `0x0000`, so a truncating narrowing would have emitted a canonicalization to positive zero and compiled.

The `bf16` fact is read and the `f32` fact beside it is not: `bf16_arithmetic_reads_the_bf16_fact_and_not_the_f32_one` holds the `f32` entry at the measured flush and moves only the `bf16` entry, and the verdict moves with it. The two measured rows agree on this host, which is exactly why the test states a target where they disagree.

### Measurement — contraction at BF16, not carried from `f16`

The BF16 unit records `SafeMathMode` and `NoFloatingPointContraction`, and `the_strict_realization_honours_what_the_bf16_unit_records` checks the driver's actual flag list carries `-ffp-contract=off` for a BF16 compilation. Per finding 28 the `f16` conclusion is not transferable, so this is asserted at BF16 directly. **It checks the selection, not the fusion** — that `-ffp-contract=off` suppresses a BF16 fusion is the retained probe's measurement. No `fma` is emitted: finding 29 makes a source-level `bfloat` fused form unstatable.

### Fact — the F32 goldens are unchanged, and one identity step was declined to keep them so

`git diff --name-only -- crates/tiler-metal/goldens/` is empty against the base for all six F32 fixtures. An earlier iteration widened the emitted prologue's `every f32 immediate` to `every floating-point immediate`, which is the truthful wording; `cargo nextest run --workspace` then failed at exactly one test — `tiler-build metal_plan::tests::the_standard_metal_path_publishes_its_recorded_identities`, artifact identity `d22c0d11f8…` → `5c366e9409…`. The emitted source is artifact content, so the wording is inside an identity domain this branch holds no scope for, and a second live branch is already moving those same pins. The wording was reverted, the reason recorded in `assemble`, and `widen-the-emitted-numerics-prologue-past-one-width` filed to take the step whole.

### Fact — why dispatch, the reference comparison, and the dispatchability refusal are not here

Exact checks, each reproducible in one line:

- `crates/tiler-metal/Cargo.toml`: dependencies are `tiler-artifact` and `tiler-ir`; the only dev-dependency is `tiler-metal-aot`. No Metal runtime, no `tiler-reference`.
- `crates/tiler-metal-aot/Cargo.toml` has no `[dependencies]` section at all — its empty dependency closure is the property `crates/tiler-metal/src/target.rs:31` records as the reason the vocabulary is owned twice.
- `grep -rn 'metal.workspace' --include=Cargo.toml`: the `metal` crate is used by `prototypes/serial-sum-run` alone, which `ticketsplease.toml` maps to `implementation/runtime`.
- `DTypeDispatchability` is `crates/tiler-compiler/src/target.rs:1415` (`implementation/compiler`); `ReferenceNumericalConformance` is `crates/tiler-reference/src/conformance.rs:123` (`implementation/reference`).

So the dispatch evidence needs `implementation/runtime`, `implementation/reference`, and `implementation/compiler`. Acquiring a dispatch capability inside the two scopes held instead would mean adding a Metal runtime dependency to a crate documented as owning no live device APIs, or to the one whose whole value is an auditable empty closure — an architectural change, which is Tom's.

**And the graph already owns the work.** `validate-bf16-at-the-runtime-routing-boundary` (`implementation/runtime`, depends on this ticket) requires "A BF16 program on the macOS profile routes and executes" and the `Unsupported`/`Unknown` preflight refusals shown to precede the one-way routing commit — that is the dispatchability refusal, by the profile fact, which is what this ticket's evidence list asked for. `conform-the-bf16-vertical-end-to-end` (`implementation/reference`, depends on the above) requires the flush applied to the reference before comparison with the moved elements named, and the execution witness on a non-subnormal operand. This ticket's required-evidence list restates its two dependents' deliverables; adding their exclusive scopes here would do their work under this ticket's name. Nothing was descoped — the evidence is scheduled, not missing.

### Deliberately not done

No dispatch, no `tiler-reference` comparison, no execution witness, no iOS-device claim (that family stays `Unknown`; only `measure-apple-numerics-on-physical-ios-device` can close it), no contraction/reassociation/FMA, no artifact round trip, no public boundary added or changed on `tiler-metal` or `tiler-metal-aot`.

### Remainder for the coordinator

The `Backend lowering` cell for BF16 in `docs/roadmap.md` / `docs/dtype-support.md` may move on this evidence, bounded to *offline* emission and compilation on the row above. **`Backend execution` must not move**: nothing here dispatched. Both cells are `contracts/navigation`, which this branch does not hold.

### Commands run

`cargo fmt --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-metal -p tiler-metal-aot --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-metal -p tiler-metal-aot --no-deps`; `cargo nextest run --workspace`; `cargo test --workspace --doc`; `TILER_REQUIRE_METAL_TOOLCHAIN=1 cargo nextest run -p tiler-metal -E 'test(golden_compilation)' --no-capture`; `tkt lint`; `git diff --check`; `tkt guard`; `make full`.

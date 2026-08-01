---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.bf16-second-dtype"
kind: "experiment"
title: "BF16 through the second-dtype seams"
topics: ["numerics", "dtypes", "bf16", "target-profiles", "apple-targets", "reference"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "exhaustive-finite", "bounded-measurement"]
supports: ["tiler.research.numerics.mature-dtype-taxonomy", "tiler.research.apple-targets.numerical-behaviour"]
entrypoints: ["spikes/numerics/bf16-second-dtype/src/main.rs"]
last_verified: "2026-08-01"
verified_at_commit: "ef3c051"
ticket: "spike-bf16-through-the-second-dtype-seams"
---

# BF16 through the second-dtype seams

The first non-F32 dtype carried against every accepted boundary a second scalar float has to cross: the governed identity, the descriptor, the accuracy metric's dtype-compatibility check, the reference element carrier, the caller-declared target profile's dispatchability seam, and its numerical-honourability seam.

It exists to answer one question before any BF16 implementation ticket is written: **which of those seams already admit a second dtype, which are legitimately F32-specific, and which are missing extension points wearing an F32 name?** The answer this run supports is *the identity and evidence layers are already generic, the target-dispatch layer is already generic, and the entire arithmetic layer is F32-specific by construction* — with the boundary between them falling at one private constructor, which is nameable and is the payload this spike owes its children.

Nothing here is production BF16 support. No `crates/` file is modified, no BF16 operation is registered with the standard provider, no `bfloat` MSL is emitted, and no GPU work is submitted. `docs/dtype-support.md` moves two cells and no more.

## Running it

From this directory. `rust-toolchain.toml` is resolved by directory ancestry from the repository root, so no selector is passed and this spike deliberately carries no toolchain file of its own.

```sh
cd spikes/numerics/bf16-second-dtype
CARGO_TARGET_DIR=./target cargo run
```

The binary's only product is a verdict: every stage that fails exits non-zero with the stage named, and there is no partial success. `CARGO_TARGET_DIR` is set explicitly because this is a nested workspace and sharing one target directory across unrelated workspaces is forbidden.

It needs no GPU, no Xcode, and no simulator. The device facts it consumes are the *retained* Apple measurements cited below, transcribed as declared target facts — this spike does not re-measure them, and `measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes` remains their owner.

## What one run does, in order

1. **Probes seven seams** (`src/seams.rs`), each paired with a control so a refusal is evidence about BF16 rather than about a harness that refuses everything.
2. **Checks its own format constants against the registered catalog descriptor.** The oracle's width, sign, exponent, and trailing-significand parameters are read back out of `builtin_scalar_value_type_facts(tiler::bf16@1)` and compared. This is the one seam probe whose negative answer is a defect rather than a finding.
3. **Runs the exact-rational reference oracle** (`src/bf16.rs`) over two populations: all 65,536 encodings, and 24 hand-derived named witnesses (`src/corpus.rs`).
4. **Resolves a three-family routing matrix** (`src/routing.rs`) and asserts its *shape* — that the three BF16 answers are three distinct resolutions — rather than six independent facts that could all have collapsed to one value.
5. **Runs seven perturbations** (`src/perturb.rs`), each watched failing.

## Result

**Measurement**, Apple M-series arm64 macOS, `rust-toolchain.toml`'s pinned nightly, base commit `ef3c051`:

Every stage agreed and every perturbation was detected. The oracle round-tripped all 65,536 encodings and agreed with an independent binary32-widening route on all 65,536; the census is 2 zeros, 254 subnormals, 65,024 normals, 2 infinities, 254 NaNs. All 24 witnesses agreed across six named categories. The routing matrix produced `Dispatchable` / `Unsupported` / `Unknown` for BF16 on the three families while F32 stayed `Dispatchable` on all three.

## The seam audit

Every F32-named surface the vertical reaches, classified three ways. **Legitimately F32-specific** means the F32 in the name is the subject and a BF16 peer is a *new* thing beside it, not a generalization of it. **Scalar-float-generic** means the mechanism already works for BF16 and the name is the only F32 in it. **Missing typed extension point** means a BF16 program needs something that does not exist, and the shape of the hole is known.

| Surface | Location | Class | What this run observed |
| --- | --- | --- | --- |
| `builtin_scalar_value_type_facts` | `crates/tiler-ir/src/semantic/catalog.rs:799` | scalar-float-generic | Returns BF16's complete descriptor unmodified. Keyed by resolved type, not by an enum. |
| `tiler::bf16@1` catalog row | `crates/tiler-ir/src/semantic/catalog.rs:377` | scalar-float-generic | Already registered with class `bfloat`, width 16, sign 1, exponent 8, trailing 7, all five special values. |
| `UlpFormat::from_value_type_facts` | `crates/tiler-ir/src/semantic/accuracy/metric.rs:412` | scalar-float-generic | Derives BF16's value set from the descriptor; refuses `u8` as `accuracy.metric.incompatible-dtype`. The `bfloat` rule is already in `ULP_FORMAT_RULES`. |
| `ReferenceElement` | `crates/tiler-reference/src/tensor.rs:36` | scalar-float-generic | A byte vector whose width the enclosing tensor's type fixes. Held a 2-byte BF16 element; refused an empty payload. |
| `ValueTypeMarker` | `crates/tiler-ir/src/semantic/registry.rs:91` | scalar-float-generic | Documented as an open local marker. This spike minted a `Bf16` marker out of tree with no production edit. |
| `declare_dtype_dispatchability` | `crates/tiler-compiler/src/target.rs:2498` | scalar-float-generic | Keyed by full `ResolvedValueType`. Accepted BF16 in both verdicts on three profiles. |
| `DTypeDispatchabilityResolution` | `crates/tiler-compiler/src/target.rs:1501` | scalar-float-generic | Produced all three of `Dispatchable`, `Unsupported`, `Unknown` for BF16 without a dtype list anywhere. |
| `MetalFloatArithmeticType::Bf16` | `crates/tiler-metal/src/target.rs:404` | scalar-float-generic | Already carries the measured BF16 flush in its own slot, inheriting nothing from F32. |
| `ArithmeticType::Bf16` | `crates/tiler-ir/src/schedule/numerics.rs:253` | scalar-float-generic | Already names `tiler::bf16@1` with its own identity tag `0x02`. |
| **`ScalarArithmetic::new`** | **`crates/tiler-compiler/src/target.rs:1286`** | **missing typed extension point** | **The hard seam.** Rejects every pair but `(F32, tiler::f32@1)`, and `f32()` is the only public constructor — so a caller cannot even *name* a BF16 numerical row. Every `declare_input_subnormals`-family method is unreachable for BF16. |
| `ByteAlignment::F32_NATURAL` | `crates/tiler-compiler/src/boundary.rs:651` | missing typed extension point | Its own doc names the hole: a widened dtype vocabulary "must derive this from the boundary value's element type rather than from the profile, and that derivation needs a field the scheduled-region IR does not have today". |
| `ExactRational::from_f32` | `crates/tiler-ir/src/semantic/accuracy/rational.rs:243` | missing typed extension point | The only float ingress on the in-tree exact rational. A BF16 value can only reach it through a host `f32`; there is no descriptor-parameterized `from_format_bits`. |
| `KernelType::F32` | `crates/tiler-ir/src/kernel/model.rs:85` | legitimately F32-specific | The sole float variant of the kernel type vocabulary. A BF16 kernel type is a new variant with its own tag, not a rename. |
| `StorageScalar::F32` | `crates/tiler-ir/src/program/model.rs:268` | legitimately F32-specific | Two variants, `U8` and `F32`, each with a `byte_width`. BF16 is a third with width 2. |
| `BinaryOp::{F32Add, F32Multiply}` | `crates/tiler-ir/src/kernel/model.rs:253` | legitimately F32-specific | Operand type is part of the operation's identity, and its tag is in the artifact encoding. |
| `NumericalContract::{StrictF32, …}` | `crates/tiler-compiler/src/session.rs:1278` | legitimately F32-specific | Four presets, each resolving one `ArithmeticType` and saying so in its key. A BF16 contract is a fifth key, not a widened fourth. |
| `PointwiseF32Expression` family | `crates/tiler-ir/src/schedule/pointwise.rs` | legitimately F32-specific | Its `Constant { bits }` is a binary32 payload; the whole module is one dtype's scalar program. |
| `constant_f32_op` / `multiply_f32_op` / `add_f32_op` | `crates/tiler-ir/src/semantic/operation.rs:186` | legitimately F32-specific | Operation *keys*. `tiler::multiply-bf16@1` is a different registered operation with its own signature and reference evaluator, which is what ADR 0026 requires. |
| `ELEMENT_TYPES` | `crates/tiler-macros/src/region.rs:100` | legitimately F32-specific | A length-1 table mapping the inline DSL's `f32` spelling. Widening it is frontend surface work, downstream of everything above. |
| `AvailabilityPhase` re-export | `crates/tiler-compiler/src/target.rs:135` | missing typed extension point (minor) | `TargetProfile::dtype_dispatchability` is public and takes it, but it is `pub(crate)` here; a caller must import it from `tiler_ir::program::abi`. An ergonomic gap, not a correctness one. |

### The one-line source checks

Two F32-only assumptions, each reproducible in one line and each assigned to a child below.

```sh
# 1. The numerical-honourability subject cannot name BF16. One public constructor,
#    and a private `new` that rejects every other pair.
rg -n 'pub fn f32\(\) -> Self|arithmetic != ArithmeticType::F32' crates/tiler-compiler/src/target.rs

# 2. Boundary alignment is a profile constant rather than a derivation from the
#    element type, and the comment above it says so.
rg -n -B6 'F32_NATURAL: Self' crates/tiler-compiler/src/boundary.rs
```

And one generic seam accepting BF16 with no modification at all:

```sh
# The accuracy metric's dtype-compatibility rule table already carries `bfloat`.
rg -n 'class: "bfloat"' crates/tiler-ir/src/semantic/accuracy/metric.rs
```

## Findings

1. **The dtype-identity layer is already dtype-generic, and it is generic by *keying*, not by enumeration.** (Fact) Every seam that admits BF16 does so because it is keyed by a full `ResolvedValueType` or reads a registered descriptor. Every seam that refuses it does so because it is keyed by a Rust enum variant or a `&'static str` operation key. **Inference:** the design rule that separates the two halves is already stated and already followed; the second dtype does not require a new abstraction, it requires the arithmetic layer to adopt the keying the identity layer already uses.

2. **The negative route is expressible, and it lands before the routing commit.** (Fact) `declare_dtype_dispatchability(bf16, Unsupported)` resolves at `AvailabilityPhase::CompileProfile`. This matters because the *measured* simulator failure occurs at `PreparedKernelPreflight` — one phase after the one-way commit of ADR 0051 — so a design that discovered BF16 unavailability from the device would already have committed. Declaring it as a profile fact is what moves the refusal before the commit, and `decide-per-dtype-dispatchability-as-a-target-capability` already decided this; this run is its first non-F32 exercise.

3. **`Unknown` is reachable and distinct.** (Fact) The unmeasured-family profile returns `Unknown` for BF16 while returning `Dispatchable` for F32 on the same profile. The perturbation asking an *undeclared* dtype of the profile that declares BF16 also returned `Unknown`. So an unmeasured `(family, dtype)` pair fails closed rather than inheriting a neighbour, which is exactly the property the `IOsDevice` row depends on.

4. **The numerical contract cannot be stated for BF16 at all, and this is the single blocking seam.** (Fact) `ScalarArithmetic` has one public constructor, `f32()`. Its private `new` rejects every other `(ArithmeticType, ResolvedValueType)` pair with `UnvalidatedScalarArithmetic`, and the comment names F16, BF16, and F64 as deliberately blocked. So the twenty-four `declare_*` honourability methods are all unreachable for BF16, and the measured macOS BF16 flush — which `crates/tiler-metal` already carries in its own slot — has nowhere to be declared on a compiler target profile. **Inference:** the fail-closed behaviour is correct and should not be relaxed by widening the check; what is missing is a *validated* construction route that proves a BF16 arithmetic/type association from a named registry authority, which is what `admit-a-bf16-scalar-arithmetic-subject` is for.

5. **Host arithmetic could not have served as the oracle, and the reason is measured rather than stylistic.** (Fact) The retained Apple record states that no single operation can separate native `bfloat` arithmetic from `f32`-precision evaluation with one rounding, because `f32`'s 24-bit significand exceeds the 18 bits that would make a second rounding to BF16's 8-bit significand innocuous. **Inference:** an oracle that computed in `f32` and rounded to BF16 would agree with a double-rounding implementation *because it shares the defect*. The exact-rational oracle is what makes the corpus evidence about BF16's semantics rather than about the host's.

6. **BF16-to-binary32 widening is exact and total, and that is a property of BF16 specifically.** (Fact) All 65,536 encodings widen losslessly, including every subnormal, both zeros, both infinities, and every NaN payload — BF16 shares binary32's exponent width and bias. This run uses it only as an *independent cross-check* on the field decode, never as the rounding route. **Inference:** it is a convenience the next ordinary float will not necessarily have (F64 and F128 have no such carrier), so a dtype-addition recipe must not assume a lossless widening exists.

7. **`fma(bfloat, bfloat, bfloat)` does not exist, so the FMA non-goal is enforced by the target and not only by this spike's scope.** (Fact) The retained record has `metal` rejecting `bfloat v6 = fma(v3, v4, v5)` outright. This spike's operation vocabulary has exactly two members and no fused variant. **Inference:** a BF16 contraction or FMA design cannot inherit the F32 shape, because the primitive it would lower to is absent at the source level; that is why the computation/accumulator question is a separate ticket rather than a widening of this one.

8. **The spike lives at a path the ticket did not name, and the ticket's path maps to no scope.** (Fact) The ticket's `paths` reserve `spikes/dtypes/bf16-second-dtype/**`, but `ticketsplease.toml` maps `spikes/numerics/**` to `research/numerics` and `spikes/apple-targets/**` to `research/apple-targets` — both scopes the ticket holds — while `spikes/dtypes/**` matches **no scope at all**, so a branch writing there is a guard escape. The spike is therefore at `spikes/numerics/bf16-second-dtype/`, and the ticket's `paths` were corrected in the same change.

## Perturbation evidence

Each perturbs exactly one thing and was observed failing, against an unperturbed neighbour that agrees.

- **The tie rule.** `TieRule::AwayFromZero` replaces exactly one branch — the exact-halfway case — reusing the same decode, the same exact arithmetic, and the same binade search. Under ties-to-even 0 of 24 witnesses disagree; under ties-away-from-zero 2 do. This is the evidence that the rounding rule is load-bearing rather than incidental.
- **The widening shift.** Shifting by 15 instead of 16 makes 65,281 of 65,536 encodings disagree, so the cross-check in Finding 6 is a real comparison rather than a tautology.
- **The descriptor lookup.** `tiler::bf17@1` returns no descriptor where `tiler::bf16@1` returns one, so "the catalog recognizes it" is a check that can say no.
- **The invalid operations.** `inf * 0` is the canonical NaN while `inf * 1` is infinity, so the exceptional-value rules are decided rather than vacuous.
- **The unmeasured dtype.** `f16` resolves `Unknown` on the very profile where `bf16` resolves `Dispatchable`.
- **The accepted neighbour.** `f32` resolves `Dispatchable` on the simulator profile that refuses `bf16`, so the refusal is dtype-specific rather than a dead profile.
- **The operation vocabulary.** Exactly two operations are expressible and no fused variant exists.

## Measurement boundary

- **No device was touched by this spike.** The macOS positive and iOS-Simulator negative rows are *transcribed* from the retained Apple record at `spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv` (Apple M4 Max, macOS 27.0 build 26A5388g, Metal 32023.883, Xcode 26.6), findings 24 and 26 of [the Apple numerical behaviour record](../../../docs/research/apple-targets/numerical-behaviour.md). They are facts about those families on that row and are not portable claims about Apple GPUs. Re-running them is that spike's procedure, not this one's.
- **`IOsDevice` is unmeasured and stays unmeasured.** The third profile here models it as silence, which is a modelling choice this spike makes to exercise the `Unknown` path — not a measurement. `measure-apple-numerics-on-physical-ios-device` is blocked on hardware and remains the only route to closing it.
- **The oracle is exhaustive over the format and not over the operations.** All 65,536 encodings are round-tripped, but the *arithmetic* is checked on 24 named witnesses plus the overflow boundary, not on all 2^32 operand pairs. An exhaustive product sweep is feasible and was not done.
- **The semantic layer was not crossed.** No BF16 operation is registered, no BF16 semantic program is built, and no reference evaluator is installed. `SemanticRegistryProvider::register_operation` and `ReferenceRegistryProvider::register` are both public and were confirmed reachable, but exercising them is `register-the-bf16-semantic-operation-signatures` and `evaluate-bf16-reference-semantics`, not this spike.
- **No physical, ABI, artifact, KIR, lowering, or runtime layer was reached.** The audit rows for those surfaces are read from source, not exercised.
- **No performance claim.** Nothing is timed. The oracle uses arbitrary-precision rationals and is not fast, deliberately.

## Retained evidence

The run prints its verdict and retains no result fixture: every number it reports is derived from the source beside it plus the cited Apple record, so a fixture would duplicate rather than preserve. `Cargo.lock` is tracked, so the dependency set a recorded run was taken under is recoverable.

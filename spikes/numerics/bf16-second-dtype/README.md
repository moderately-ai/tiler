---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.bf16-second-dtype"
kind: "experiment"
title: "BF16 through the second-dtype seams"
topics: ["numerics", "dtypes", "bf16", "target-profiles", "apple-targets", "reference"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "exhaustive-finite", "bounded-measurement"]
supports: ["tiler.research.numerics.mature-dtype-taxonomy", "tiler.research.apple-targets.numerical-behaviour", "tiler.research.numerics.bf16-computation-accumulator-and-conversion"]
entrypoints: ["spikes/numerics/bf16-second-dtype/src/main.rs"]
last_verified: "2026-08-01"
verified_at_commit: "59a2fe2"
ticket: "spike-bf16-through-the-second-dtype-seams"
---

# BF16 through the second-dtype seams

The first non-F32 dtype carried against every accepted boundary a second scalar float has to cross: the governed identity, the descriptor, the accuracy metric's dtype-compatibility check, the reference element carrier, the caller-declared target profile's dispatchability seam, and its numerical-honourability seam.

It exists to answer one question before any BF16 implementation ticket is written: **which of those seams already admit a second dtype, which are legitimately F32-specific, and which are missing extension points wearing an F32 name?** The answer this run supports is *the identity and evidence layers are already generic, the target-dispatch layer is already generic, and the entire arithmetic layer is F32-specific by construction* — with the boundary between them falling at one private constructor, which is nameable and is the payload this spike owes its children.

It now carries a second question, added by `design-the-bf16-computation-and-accumulator-contract` because the exact-rational oracle here is the only tool that can answer it: **what may a BF16 program compute in, accumulate in, and convert to, and which of those choices are observable in the result bits?** The answer this run supports is *promotion through binary32 is exact for one multiply or add and inexact for a fused multiply-add, the accumulator's width is observable on its own, and the two conversion directions need different contract shapes* — with the design derived from it in [BF16 computation, accumulator, and conversion](../../../docs/research/numerics/bf16-computation-accumulator-and-conversion.md).

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
4. **Decides the computation, accumulator, and conversion questions** (`src/format.rs`, `src/promotion.rs`) over six stages, added by `design-the-bf16-computation-and-accumulator-contract`. Each derives bit patterns rather than arguing; the section below records what they answer.
5. **Resolves a three-family routing matrix** (`src/routing.rs`) and asserts its *shape* — that the three BF16 answers are three distinct resolutions — rather than six independent facts that could all have collapsed to one value.
6. **Runs ten perturbations** (`src/perturb.rs`), each watched failing.

## Result

**Measurement**, Apple M-series arm64 macOS, `rust-toolchain.toml`'s pinned nightly, base commits `ef3c051` (seams, oracle, routing) and `59a2fe2` (the promotion stages), 3.8 s in the release profile:

Every stage agreed and every perturbation was detected. The oracle round-tripped all 65,536 encodings and agreed with an independent binary32-widening route on all 65,536; the census is 2 zeros, 254 subnormals, 65,024 normals, 2 infinities, 254 NaNs. All 24 witnesses agreed across six named categories. The routing matrix produced `Dispatchable` / `Unsupported` / `Unknown` for BF16 on the three families while F32 stayed `Dispatchable` on all three. The six promotion stages returned the numbers tabulated below.

## The computation, accumulator, and conversion stages

Every number here is derived from exact rational arithmetic on the host and is host-independent. Nothing in this section runs on a GPU or is evidence about one; the Apple facts it reasons *against* are transcribed with their own boundaries in "Measurement boundary" below. The design these stages support is [BF16 computation, accumulator, and conversion](../../../docs/research/numerics/bf16-computation-accumulator-and-conversion.md).

| Stage | Population | Result |
| --- | --- | --- |
| The generic rounder agrees with the BF16 oracle | 196,608 exact values — every encoding's own value plus a product and a sum formed from it | 0 disagreements |
| A single multiply or add admits the promoted binary32 route | 524,288 cases — all 65,536 encodings against 4 named partners, both operations | 0 disagreements between one exact rounding and widen/evaluate/narrow |
| A fused multiply-add does **not** | 262,144 triples — all 65,536 second operands against 4 named addends, first operand `1.5` | witness `a=0x3fc0 b=0x3fb2 c=0xb300`: one exact rounding gives `0x4005`, widen/fma/narrow gives `0x4006`; 21,546 disagreements in the sweep |
| The accumulator width is observable | one derived witness plus its one-contributor control | `0x3f80` plus four copies of `0x3b00` gives `0x3f80` at a BF16 accumulator and `0x3f81` at a binary32 one; with one contributor both give `0x3f80` |
| BF16 widens to binary32 exactly, subnormals staying subnormal | all 65,536 encodings | 0 value disagreements with a 16-bit shift; 254 of 254 subnormals are binary32 subnormals; binary16's smallest subnormal exponent `-24` is above binary32's smallest normal exponent `-126`, so its subnormals are binary32 *normals* instead |
| The narrowing direction's rules are decided | 65,536 binary32 patterns covering all of `[1, 2)`, stepping the significand by 128 | nearest-even differs from truncation in 32,704 and from ties-away in 64; `0x7f7fffff` narrows to the infinity `0x7f80`; the signalling NaN `0x7f800001` *truncates* to `0x7f80`, an infinity |

The three perturbations these stages need: replacing the binary32 intermediate with a precision-9 format of BF16's own exponent range takes the multiply route from 0 disagreements to 15,770, so stage 2 is a real comparison; moving the fused witness's addend from `2^-25` to `2^-20` makes both routes return `0x4005`, so the witness is about the binary32 rounding boundary and not about its operands; reducing the accumulator witness to zero contributors makes both folds return `0x3f80`, so the difference is accumulated rounding rather than a disagreement about the seed.

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

5. **Host arithmetic could not have served as the oracle — and the original reason was backwards, which the promotion stages now settle.** (Fact) The retained Apple record states that no single operation can separate native `bfloat` arithmetic from `f32`-precision evaluation with one rounding, because `f32`'s 24-bit significand exceeds the 18 bits (`2p + 2` at `p = 8`) that would make a second rounding to BF16's 8-bit significand innocuous. **Correction, 2026-08-01.** This finding previously inferred from that sentence that "an oracle that computed in `f32` and rounded to BF16 would agree with a double-rounding implementation *because it shares the defect*", and `src/bf16.rs` carried the same reading with the inequality written the other way round. The inequality holds, so for a single multiply or add there **is** no defect to share: stage 2 checks it over 524,288 cases and finds zero disagreements, and the precision-9 perturbation shows the check can fail. **Inference — the conclusion survives with a different reason.** The exact-rational oracle is right because it does not rest on that bound at all: the bound covers `+ - * /` and square root, and it covers neither the fused multiply-add — where stage 3 exhibits operands on which a promoted route differs by one ulp — nor an accumulation, nor anything this spike's children add. An `f32`-based oracle would have been correct for the two operations admitted here and silently wrong for the first one added.

6. **BF16-to-binary32 widening is exact and total, and that is a property of BF16 specifically.** (Fact) All 65,536 encodings widen losslessly, including every subnormal, both zeros, both infinities, and every NaN payload — BF16 shares binary32's exponent width and bias. This run uses it only as an *independent cross-check* on the field decode, never as the rounding route. **Inference:** it is a convenience the next ordinary float will not necessarily have (F64 and F128 have no such carrier), so a dtype-addition recipe must not assume a lossless widening exists.

7. **`fma(bfloat, bfloat, bfloat)` does not exist, so the FMA non-goal is enforced by the target and not only by this spike's scope.** (Fact) The retained record has `metal` rejecting `bfloat v6 = fma(v3, v4, v5)` outright. This spike's operation vocabulary has exactly two members and no fused variant. **Inference:** a BF16 contraction or FMA design cannot inherit the F32 shape, because the primitive it would lower to is absent at the source level; that is why the computation/accumulator question is a separate ticket rather than a widening of this one.

8. **The spike lives at a path the ticket did not name, and the ticket's path maps to no scope.** (Fact) The ticket's `paths` reserve `spikes/dtypes/bf16-second-dtype/**`, but `ticketsplease.toml` maps `spikes/numerics/**` to `research/numerics` and `spikes/apple-targets/**` to `research/apple-targets` — both scopes the ticket holds — while `spikes/dtypes/**` matches **no scope at all**, so a branch writing there is a guard escape. The spike is therefore at `spikes/numerics/bf16-second-dtype/`, and the ticket's `paths` were corrected in the same change.

9. **Promotion through binary32 is exact for one BF16 multiply or add and inexact for a fused multiply-add, and the boundary is the operation shape rather than the dtype.** (Fact) Over 524,288 cases a widen/evaluate/narrow route and one exact rounding never disagree; over 262,144 fused triples they disagree 21,546 times, first exhibited at `a=0x3fc0 b=0x3fb2 c=0xb300` where the exact result is `0x4005` and the promoted one `0x4006`. **Inference:** because finding 7 establishes that a promoted route is the *only* realization a fused BF16 operation could ever be given, a contract naming itself a fused BF16 operation would be stating something the target cannot deliver. What is realizable is a mixed-precision operation whose contract says binary32 and one narrowing, which is a different contract wearing an honest name. [The design record](../../../docs/research/numerics/bf16-computation-accumulator-and-conversion.md) carries the elimination.

10. **The accumulator's width is observable with no promotion, no conversion, and no target involved.** (Fact) `0x3f80` folded with four copies of `0x3b00` returns `0x3f80` when each partial is rounded to BF16 and `0x3f81` when the partials are held at binary32 and rounded once; with one contributor both return `0x3f80`. **Inference:** "BF16 storage with an F32 accumulator" is therefore a *different program* from "BF16 storage with a BF16 accumulator", not a faster realization of it — which is what makes the accumulator an operation fact rather than a schedule choice, and what makes the landed pure vertical's `BF16_FACT_ACCUMULATOR_TYPE` load-bearing rather than ceremonial.

11. **The two conversion directions need different contract shapes, and the narrowing direction has three separable decisions the phrase "round to nearest" does not cover.** (Fact) Widening is exact and total and therefore carries no rounding rule at all; narrowing changes its answer under all three of rounding rule (nearest-even against truncation: 32,704 of 65,536 patterns in `[1, 2)`), tie direction (64 of the same population), and overflow (`0x7f7fffff` narrows to the infinity `0x7f80`, because binary32's largest finite magnitude is above BF16's overflow threshold). A fourth decision is forced rather than chosen: truncating a NaN payload is not total, since the signalling NaN `0x7f800001` truncates to `0x7f80`, an infinity. **Inference:** this is ADR 0041's shape — separate families each carrying only its meaningful fields — reached independently at a float-to-float boundary.

## Perturbation evidence

Each perturbs exactly one thing and was observed failing, against an unperturbed neighbour that agrees.

- **The tie rule.** `TieRule::AwayFromZero` replaces exactly one branch — the exact-halfway case — reusing the same decode, the same exact arithmetic, and the same binade search. Under ties-to-even 0 of 24 witnesses disagree; under ties-away-from-zero 2 do. This is the evidence that the rounding rule is load-bearing rather than incidental.
- **The widening shift.** Shifting by 15 instead of 16 makes 65,281 of 65,536 encodings disagree, so the cross-check in Finding 6 is a real comparison rather than a tautology.
- **The descriptor lookup.** `tiler::bf17@1` returns no descriptor where `tiler::bf16@1` returns one, so "the catalog recognizes it" is a check that can say no.
- **The invalid operations.** `inf * 0` is the canonical NaN while `inf * 1` is infinity, so the exceptional-value rules are decided rather than vacuous.
- **The unmeasured dtype.** `f16` resolves `Unknown` on the very profile where `bf16` resolves `Dispatchable`.
- **The accepted neighbour.** `f32` resolves `Dispatchable` on the simulator profile that refuses `bf16`, so the refusal is dtype-specific rather than a dead profile.
- **The operation vocabulary.** Exactly two operations are expressible and no fused variant exists.
- **The intermediate precision.** Replacing the promoted route's binary32 intermediate with a precision-9 format carrying BF16's *own* exponent range — so only the precision moves, and the double-rounding bound `q >= 2p + 2` becomes `9 >= 18` and fails — takes stage 2's multiply comparison from 0 disagreements to 15,770. A stage that can only report zero is not a check; this is what makes its zero one.
- **The fused witness's addend.** Moving it from `2^-25` to `2^-20`, five binades up and out of the range where binary32 rounds the sum back onto a BF16 halfway point, makes both routes return `0x4005`. So the witness is about double rounding rather than about its operands.
- **The accumulator's contributor count.** Reducing it to zero makes both folds return the seed `0x3f80`, so the one-ulp difference at four contributors is accumulated rounding rather than a disagreement the two accumulators have about the seed.

## Measurement boundary

- **No device was touched by this spike.** The macOS positive and iOS-Simulator negative rows are *transcribed* from the retained Apple record at `spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv` (Apple M4 Max, macOS 27.0 build 26A5388g, Metal 32023.883, Xcode 26.6), findings 24 and 26 of [the Apple numerical behaviour record](../../../docs/research/apple-targets/numerical-behaviour.md). They are facts about those families on that row and are not portable claims about Apple GPUs. Re-running them is that spike's procedure, not this one's.
- **`IOsDevice` is unmeasured and stays unmeasured.** The third profile here models it as silence, which is a modelling choice this spike makes to exercise the `Unknown` path — not a measurement. `measure-apple-numerics-on-physical-ios-device` is blocked on hardware and remains the only route to closing it.
- **The oracle is exhaustive over the format and not over the operations.** All 65,536 encodings are round-tripped, but the *arithmetic* is checked on 24 named witnesses plus the overflow boundary, plus the promotion stages' 524,288 multiply-and-add cases and 262,144 fused triples — each of which sweeps one operand exhaustively against a *named* set of partners rather than sweeping all 2^32 pairs or all 2^48 triples. So every stated count is a count over its own named population and none is a universal claim about the operation.
- **The promotion stages measure nothing and are host-independent.** Their arithmetic is exact rational and their comparisons are between two stated contracts, so the numbers would be identical on any host and under any toolchain. They are `executable-model` and `exhaustive-finite` evidence about BF16's semantics, never `bounded-measurement` evidence about a device. In particular, stage 3 does **not** claim that any Apple GPU computes the promoted result: it claims that *if* a fused BF16 operation is realized by promotion — which finding 7 establishes is the only route MSL offers — then it is not the correctly rounded BF16 fused result.
- **The semantic layer was not crossed.** No BF16 operation is registered, no BF16 semantic program is built, and no reference evaluator is installed. `SemanticRegistryProvider::register_operation` and `ReferenceRegistryProvider::register` are both public and were confirmed reachable, but exercising them is `register-the-bf16-semantic-operation-signatures` and `evaluate-bf16-reference-semantics`, not this spike.
- **No physical, ABI, artifact, KIR, lowering, or runtime layer was reached.** The audit rows for those surfaces are read from source, not exercised.
- **No performance claim.** Nothing is timed. The oracle uses arbitrary-precision rationals and is not fast, deliberately.

## Retained evidence

The run prints its verdict and retains no result fixture: every number it reports is derived from the source beside it plus the cited Apple record, so a fixture would duplicate rather than preserve. `Cargo.lock` is tracked, so the dependency set a recorded run was taken under is recoverable.

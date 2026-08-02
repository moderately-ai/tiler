---
id: admit-a-bf16-scalar-arithmetic-subject
title: Admit a BF16 scalar-arithmetic subject so a BF16 numerical row is statable
status: done
priority: p1
dependencies: []
related: [spike-bf16-through-the-second-dtype-seams, admit-a-caller-declared-target-profile, declare-metal-numerical-honourability, own-the-dtype-support-maturity-matrix]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, target-profiles, numerics]
---
## User-visible outcome

A caller can declare the measured BF16 subnormal behaviour on a target profile. Today it cannot state it at all, so the measured macOS BF16 flush — which `crates/tiler-metal` already carries in its own slot — has nowhere to go, and every downstream BF16 feasibility question is unanswerable rather than answered `Unknown`.

## Why this is the blocking seam

**Fact, at `ef3c051`.** `ScalarArithmetic` (`crates/tiler-compiler/src/target.rs`) exposes exactly one public constructor, `f32()`. Its private `new` returns `TargetProfileBuildError::UnvalidatedScalarArithmetic` for every other `(ArithmeticType, ResolvedValueType)` pair, and the comment above it names F16, BF16, and F64 as deliberately blocked. Reproduce in one line:

```sh
rg -n 'pub fn f32\(\) -> Self|arithmetic != ArithmeticType::F32' crates/tiler-compiler/src/target.rs
```

**Fact.** The twenty-four `declare_input_subnormals`-family methods on `TargetProfileBuilder` all take a `ScalarArithmetic`, so all twenty-four are unreachable for BF16.

**Fact.** [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) confirmed that everything *around* this gate is already dtype-generic: the honourability vocabulary in `crates/tiler-compiler/src/target/honourability.rs` is keyed by `(NumericalDimension, ArithmeticType)` throughout, `ArithmeticType::Bf16` exists and names `tiler::bf16@1`, and `declare_dtype_dispatchability` accepts BF16 unmodified.

**Inference.** The refusal is correct and is the reason to be careful: `docs/numerical-semantics.md` states the compiler "refuses to construct any other pair" precisely because an admitted value identity is not evidence that any arithmetic subject was calibrated for it. The fix is therefore **not** to widen the equality check. It is to add a validated construction route that proves the arithmetic/type association from a named registry authority, so that a pair reaches `ScalarArithmetic` only with evidence behind it.

## Implementation keys

- Decide and state the authority that admits an association. A `(ArithmeticType, ResolvedValueType)` pair is admissible when the resolved type is registered, its descriptor's class is one the arithmetic type's semantics are defined over, and the width the descriptor states matches the arithmetic type's. Derive it from the registered descriptor rather than from a second hard-coded table — `builtin_scalar_value_type_facts` is the existing reader.
- A pair whose descriptor does not exist, or disagrees, still fails closed with a typed reason. `UnvalidatedScalarArithmetic` keeps its meaning for that case.
- `ScalarArithmetic::f32()` must keep working and keep its exact current identity encoding. The profile descriptor's bytes are an identity; changing F32's encoding would move every existing profile identity and every artifact that names one.
- The new subject participates in `ScalarArithmetic::encode` the same way, so a BF16 row is distinguishable from an F32 row in the profile descriptor.
- This ticket declares no BF16 fact. It makes a BF16 row *statable*; `declare-the-bf16-rows-on-the-authoritative-metal-profile` states one.

## Required evidence

- A BF16 subject constructs, and an F16 subject also constructs — so the mechanism is not a second special case with BF16's name on it.
- A pair whose width disagrees with its descriptor is refused, and the refusal is observed rather than asserted from a build error.
- An unregistered identity paired with any arithmetic type is refused.
- The F32 profile descriptor's bytes are unchanged, pinned by the existing golden.
- A BF16 subject that is constructed but never declared still resolves `Unknown` for every dimension, so construction is not admission.

## Closes when

A BF16 scalar-arithmetic subject is constructible through a validated route with a named authority, the three refusals above are observed failing, F32's identity encoding is byte-identical, `docs/numerical-semantics.md`'s sentence about refusing every other pair is corrected to describe what the compiler now does, and no BF16 target fact is declared by this ticket.

## Graph maintenance

- This is the first of the BF16 children and gates `declare-the-bf16-rows-on-the-authoritative-metal-profile`. It does not gate the semantic or reference children, which are independent of the target layer.
- The dtype ledger's `Numerical contract and honourability` cell for BF16 stays `architectural seam` until a fact is actually declared. This ticket alone does not move it.
- Widening the subject to F16, F64, or any non-float family is out of scope. `docs/numerical-semantics.md` states that the scalar-arithmetic contract does not generalize to integer, boolean, or quantized families, and this ticket must not weaken that.

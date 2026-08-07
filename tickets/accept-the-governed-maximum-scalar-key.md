---
id: accept-the-governed-maximum-scalar-key
title: Accept the governed maximum scalar key
status: awaiting-decision
priority: p2
dependencies: []
related: [admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold, admit-the-registered-elementary-families-as-recognizable-program-stages]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, numerics]
---
## The exact surface offered

One free function and one registered definition, landed as a labelled draft by [`admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold`](admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold.md). Only Tom closes this node.

```rust
// crates/tiler-ir/src/index/scalar.rs, re-exported from crates/tiler-ir/src/index/mod.rs
pub fn maximum_f32_scalar_op() -> ScalarOpKey;   // tiler.scalar::maximum-f32@1
```

Registered into `ScalarRegistryBuilder::standard()` as the twelfth governed key: binary, homogeneous `f32` in and out (`StandardF32Homogeneous`), no attributes, `ScalarEffect::Pure`, conformance identity `tiler.scalar.conformance.maximum-f32`.

**Included.** The key, its name, its arity, its fact record, its registered normative definition, and its presence in the governed standard profile.

**Excluded.** No number-preferring sibling, no `minimum-f32`, no `bf16` sibling. No realization law names it yet, so no region can be built that carries it; that is [`register-the-softmax-realization-law`](register-the-softmax-realization-law.md)'s work.

## The three questions this key was split out to settle, and their answers

### 1. The NaN-result rule, and signalling-NaN scope

**The key names the existing [`CANONICAL_ARITHMETIC_NAN_PROFILE`], and the third vocabulary value the deriving ticket expected does not exist.** That is the most consequential finding here, and it is a derivation rather than a preference.

**Fact.** [ADR 0023](../docs/decisions/0023-floating-point-extrema-semantics.md)'s Decision section states, of both extrema families and beside the `-0.0 < +0.0` requirement: "Portable-bitwise NaN results use the existing canonical arithmetic-NaN contract."

**Fact.** [Numerical semantics](../docs/numerical-semantics.md)'s "Min and max" section, whose only subject is the two families, states: "Under portable-bitwise conformance, a produced NaN follows the canonical arithmetic-NaN contract."

**Fact — both delivered realizations agree, and neither propagates.** `maximum_helper` in `crates/tiler-metal/src/emit.rs` returns the canonical pattern `0x7fc00000` directly on its unordered arm, with its own comment recording that this is "rather than by producing some NaN and relying on a later canonicalization". `maximum_f32` in `crates/tiler-reference/src/softmax.rs` returns `f32::NAN`, which is that pattern.

**Inference — where the deriving ticket's premise fails.** It read `BinaryOp::F32Maximum`'s "performs no arithmetic … selects one of its operands' bit patterns" as implying the operation cannot install a payload. Those are different claims, and `canonicalize-nan-f32` already separates them: it is documented as "a named typed conversion, deliberately not arithmetic", computes nothing, reproduces every non-NaN pattern verbatim, and names this profile. On an *ordered* operand pair the maximum installs nothing — but that is a statement about non-NaN results, and `SCALAR_FACT_NAN_RESULT_RULE` decides the payload of a *NaN* result, which the profile value therefore does not overclaim.

**A signalling NaN is in scope and takes the same answer, with no clause of its own.** It makes the operand pair unordered exactly as a quiet NaN does, so the value contract is identical, and both delivered realizations reach it with no special case (`is_nan()` is true for either; the Metal helper's unordered arm fires for either). The invalid-operation signal IEEE 754 would raise is outside Tiler's observable contract altogether — [Numerical semantics](../docs/numerical-semantics.md) fixes exception observation as value-only (`RaiseNoFlag`) rather than leaving it to a host — so there is nothing further for a fact record to state.

**Evidence boundary, recorded rather than papered over.** IEEE Std 754-2019 is `metadata-only` in `docs/research/numerics/sources` — purchased by Tom and not redistributable — so the standard's own clause text for `maximum` is not readable from this tree. What the repository holds is the reading in [floating-point extrema precedents](../docs/research/numerics/floating-point-extrema-precedents.md): that `minimum`/`maximum` propagate NaN and order `-0.0 < +0.0`, separately from `minimumNumber`/`maximumNumber`. That record states **no** payload rule and **no** sNaN rule for these families. So the payload above is derived from Tiler's own accepted contract, not cited to the standard, and this node says so rather than inventing a citation.

### 2. Whether the name encodes the family

**`maximum-f32`, and the bare spelling is admissible because the sibling's name is already its complement in the standard's own vocabulary.** IEEE 754-2019 spells the propagating family `maximum` and the number-preferring one `maximumNumber`; ADR 0023 carries the pair over as `Maximum` and `MaximumNumber`. Under this module's naming rule — the spec's own name, kebab-cased, with the operand width appended, as `rsqrt-f32`, `exp-f32`, and `divide-f32` already are — a later sibling spells `maximum-number-f32` and this key reads as exactly its complement. It can never later read as the wrong one.

The counter-consideration, stated because it is the real hazard: a bare `maximum` does not on its own separate this family from a host or backend spelling that shares the word, and Rust's `f32::max` and Metal's `fmax` are both the *other* family. The separation is carried where this module already carries `divide-f32`'s exclusion of a reciprocal multiply — in the registered normative definition, which names the family, the NaN rule, the zero ordering, and both excluded spellings, and which is part of this definition's encoded identity. A disambiguating name such as `maximum-propagating-f32` was eliminated for diverging from the name ADR 0023 anchors on.

### 3. The signed-zero ordering

**`-0.0 < +0.0`, stated as Tiler's own fact.** ADR 0023 requires the ordering of both Tiler families, so `maximum(-0.0, +0.0)` is `+0.0` in either operand order. The reference model does not implement it and is cited only as contrast: `torch.max` over `[+0.0, -0.0]` is `-0.0` and over `[-0.0, +0.0]` is `+0.0`, while `torch.amax` answers the other way on both — each returns a fixed *position* rather than a fixed value. The four `torch_max_of_signed_zeros_*` and `torch_amax_of_signed_zeros_*` rows of `spikes/numerics/transformer_reference_semantics/results/2026-08-01-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv` carry the measurement. **Nothing in this key rests on them.**

## The fact record, and why it is shared

Byte-identical to `canonicalize-nan-f32`'s, and the shared constructor is now `exact_bit_pattern_f32_scalar_facts` (renamed from `canonicalize_nan_f32_facts`). All three fields say the same thing about both keys: neither rounds — each reproduces an operand's binary32 pattern verbatim, so `exact-binary32-bits` — both install the governed canonical arithmetic NaN for a NaN result, and neither is an arithmetic-contraction participant. This is the sharing `elementary_f32_scalar_facts` already does for the exponential and the reciprocal square root, on the same ground: two copies would be two authorities over one statement that could drift.

`the_maximum_shares_the_exact_bit_pattern_fact_record` asserts the equality so a later divergence is deliberate rather than accidental, and asserts beside it that the two keys stay distinct definitions — different arity, different conformance identity, different reached-definition projection bytes, and a normative definition that pins the family and the excluded spellings.

**The choice worth objecting to.** A reader who groups fact records by "installs the canonical payload" now finds an arithmetic multiply, a conversion, and a selection in one bucket. The alternative — a distinct record per key stating the same three things — was eliminated as drift surface, and what actually separates the three is a field the record does not carry (rounding separates the arithmetic one; arity and the normative definition separate the other two).

## Identity consequence, already paid

Registering the key widens `CanonicalScalarRegistrySnapshotIdentity` and therefore every whole-snapshot provenance derived from it. It leaves reached-only projections alone, so every existing occurrence's executable coverage — and so its kernel-program and artifact identity — is byte-identical. Exactly one pinned identity moved: `explain.rs`'s `deterministic_trace_is_sealed_and_rendered_separately`, from `6f153efeb2da5bb1` to `9478647f38ab8df5`.

## Closes when

Tom accepts or rejects the exact surface above. On acceptance the draft label at `crates/tiler-ir/src/index/scalar.rs` is removed in the same change that closes this node.

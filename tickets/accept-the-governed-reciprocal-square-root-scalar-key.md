---
id: accept-the-governed-reciprocal-square-root-scalar-key
title: Accept the governed reciprocal-square-root scalar key
status: done
priority: p2
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The exact surface offered

One free function and one registered definition, landed as a labelled draft by [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md). Only Tom closes this node.

```rust
// crates/tiler-ir/src/index/scalar.rs, re-exported from crates/tiler-ir/src/index/mod.rs
pub fn rsqrt_f32_scalar_op() -> ScalarOpKey;   // tiler.scalar::rsqrt-f32@1
```

Registered into `ScalarRegistryBuilder::standard()` as the eleventh governed key: unary, homogeneous `f32` in and out (`StandardF32Homogeneous`), no attributes, `ScalarEffect::Pure`, conformance identity `tiler.scalar.conformance.rsqrt-f32`.

**Included.** The key, its name, its arity, its fact record, and its presence in the governed standard profile.

**Excluded.** No `bf16` sibling, and no square-root key beside it. Realization of the key is not this acceptance's surface: it was owned by [`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md) and has landed — `IndexRealizationLaw::StagedRootMeanSquareScaleF32` applies `rsqrt_f32_scalar_op` on the biased mean.

## Why this shape and not another

**Fact.** `tiler::rms-norm-f32@1`'s registered reference semantics (`crates/tiler-ir/src/semantic/rms_norm.rs:228-238`) pin `r = Rsqrt(t)` and state "deliberately not `1 / Sqrt(t)`"; `RMS_NORM_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED` (`rms_norm.rs:198`) withholds the substitution. One key spelling one operation is what makes the two-rounding form unstatable rather than merely forbidden -- the argument `divide_f32_scalar_op` already states for its own missing reciprocal sibling (`scalar.rs:44-54`).

**Fact.** The fact record is byte-identical to `exp-f32`'s, and the shared constructor is now `elementary_f32_scalar_facts`. All three fields say the same thing about both keys: neither states a rounding rule this layer can name, both install the canonical arithmetic NaN, and neither declares contraction. `the_reciprocal_square_root_shares_the_elementary_fact_record` asserts the equality so a later divergence is deliberate rather than accidental.

**Inference.** What separates the two keys is not the scalar record but the resolved ADR 0042 contract their *operations* state -- `BoundedPiecewise` at twelve ULP for the activation's exponential, `Faithful` for this normalization's reciprocal square root (`rms_norm.rs:333-342`). That authority lives one layer up, and a second copy of it here would be a second authority over one obligation.

## Identity consequence, already paid

Registering the key widens `CanonicalScalarRegistrySnapshotIdentity` and therefore every whole-snapshot provenance derived from it. It leaves reached-only projections alone, so every existing occurrence's executable coverage -- and so its kernel-program and artifact identity -- is byte-identical. Exactly one pinned identity moved: `explain.rs`'s `deterministic_trace_is_sealed_and_rendered_separately`, from `8966151e455093ea` to `ce6f9106c1c5933b`, with its ledger entry in the same commit.

## Closes when

Tom accepts or rejects the exact surface above. On acceptance the draft label at `crates/tiler-ir/src/index/scalar.rs` is removed in the same change that closes this node.

## Accepted 2026-08-06

**Tom accepted the key at the live session's decision round, relayed and executed by the coordinator.** The surface follows the established scalar-key pattern (eleventh governed key at admission against the pre-rsqrt ten-key profile; elementary-unary / exp shape), with two-sided perturbation evidence and a live consumer (the normalization law in flight). The code-side label flip was routed to the law-widening worker's branch (which holds `implementation/ir`) so the sweep lands whole with that merge; this node records the acceptance and its provenance.

**Current-state correction — 2026-08-09.** The routed code half landed. [`widen-the-staged-realization-law-to-the-registered-elementary-families`](widen-the-staged-realization-law-to-the-registered-elementary-families.md), source anchor `The rsqrt acceptance's code half`, records the carried rewrite, and `rsqrt_f32_scalar_op` now carries the `Accepted boundary` paragraph naming this node. No label-flip work remains here.

**Correction — 2026-08-10.** The Excluded sentence that no realization law names the key yet was left standing after the 2026-08-09 code-half correction; it is false in present tense and is restated above. The Accepted ordinal "tenth key" contradicted the surface section's "eleventh governed key" and the pre-admission ten-key census; it is corrected to the admission-time eleventh-key wording. Later `standard()` registration added a twelfth key (`maximum-f32`); that does not change the admission-time ordinal.
